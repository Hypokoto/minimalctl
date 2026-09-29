#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/signalfd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>

#define MAX_ENTRIES   50
#define MAX_ENTRY_LEN 16384
#define SOCK_NAME     "minclip.sock"

typedef struct {
    int id;
    size_t len;
    char *text;
} ClipEntry;

typedef struct {
    struct wl_display *display;
    struct wl_registry *registry;
    struct wl_seat *seat;
    struct wl_data_device_manager *data_device_mgr;
    struct wl_data_device *data_device;
    struct wl_data_offer *current_offer;
    char current_mime[128];
    uint32_t last_serial;

    ClipEntry ring[MAX_ENTRIES];
    int count;
    int next_id;

    int sock_fd;
    int epoll_fd;
    int sig_fd;
    bool running;
} MinClip;

static void ring_clear(MinClip *app) {
    for (int i = 0; i < app->count; i++) {
        free(app->ring[i].text);
        app->ring[i].text = NULL;
    }
    app->count = 0;
}

static void ring_add(MinClip *app, const char *text, size_t len) {
    if (!text || len == 0) return;

    // Deduplicate against latest entry
    if (app->count > 0 && app->ring[0].len == len && memcmp(app->ring[0].text, text, len) == 0) {
        return;
    }

    char *copy = malloc(len + 1);
    if (!copy) return;
    memcpy(copy, text, len);
    copy[len] = '\0';

    if (app->count == MAX_ENTRIES) {
        free(app->ring[MAX_ENTRIES - 1].text);
        for (int i = MAX_ENTRIES - 1; i > 0; i--) {
            app->ring[i] = app->ring[i - 1];
        }
    } else {
        for (int i = app->count; i > 0; i--) {
            app->ring[i] = app->ring[i - 1];
        }
        app->count++;
    }

    app->ring[0].id = ++app->next_id;
    app->ring[0].len = len;
    app->ring[0].text = copy;
}

// Data Offer Listener
static void offer_offer(void *data, struct wl_data_offer *offer, const char *mime) {
    (void)offer;
    MinClip *app = (MinClip *)data;
    if (strcmp(mime, "text/plain;charset=utf-8") == 0 ||
        (strcmp(mime, "text/plain") == 0 && app->current_mime[0] == '\0') ||
        (strcmp(mime, "UTF8_STRING") == 0 && app->current_mime[0] == '\0')) {
        snprintf(app->current_mime, sizeof(app->current_mime), "%s", mime);
    }
}

static void offer_source_actions(void *d, struct wl_data_offer *o, uint32_t a) { (void)d; (void)o; (void)a; }
static void offer_action(void *d, struct wl_data_offer *o, uint32_t a) { (void)d; (void)o; (void)a; }

static const struct wl_data_offer_listener offer_listener = {
    .offer = offer_offer,
    .source_actions = offer_source_actions,
    .action = offer_action,
};

// Data Device Listener
static void device_data_offer(void *data, struct wl_data_device *dev, struct wl_data_offer *offer) {
    (void)dev;
    MinClip *app = (MinClip *)data;
    app->current_offer = offer;
    app->current_mime[0] = '\0';
    wl_data_offer_add_listener(offer, &offer_listener, app);
}

static void device_selection(void *data, struct wl_data_device *dev, struct wl_data_offer *offer) {
    (void)dev;
    MinClip *app = (MinClip *)data;
    if (!offer || app->current_mime[0] == '\0') return;

    int pfd[2];
    if (pipe2(pfd, O_CLOEXEC) < 0) return;

    wl_data_offer_receive(offer, app->current_mime, pfd[1]);
    close(pfd[1]);
    wl_display_flush(app->display);

    char *buf = malloc(MAX_ENTRY_LEN);
    if (!buf) {
        close(pfd[0]);
        return;
    }

    size_t total = 0;
    while (total < MAX_ENTRY_LEN - 1) {
        ssize_t n = read(pfd[0], buf + total, MAX_ENTRY_LEN - 1 - total);
        if (n <= 0) break;
        total += n;
    }
    close(pfd[0]);
    buf[total] = '\0';

    if (total > 0) {
        ring_add(app, buf, total);
    }
    free(buf);
}

static void device_enter(void *d, struct wl_data_device *dev, uint32_t s, struct wl_surface *surf, wl_fixed_t x, wl_fixed_t y, struct wl_data_offer *o) {
    (void)d; (void)dev; (void)s; (void)surf; (void)x; (void)y; (void)o;
}
static void device_leave(void *d, struct wl_data_device *dev) { (void)d; (void)dev; }
static void device_motion(void *d, struct wl_data_device *dev, uint32_t t, wl_fixed_t x, wl_fixed_t y) { (void)d; (void)dev; (void)t; (void)x; (void)y; }
static void device_drop(void *d, struct wl_data_device *dev) { (void)d; (void)dev; }

static const struct wl_data_device_listener device_listener = {
    .data_offer = device_data_offer,
    .enter = device_enter,
    .leave = device_leave,
    .motion = device_motion,
    .drop = device_drop,
    .selection = device_selection,
};

// Data Source Listener for Copying
static void source_target(void *d, struct wl_data_source *s, const char *m) { (void)d; (void)s; (void)m; }
static void source_send(void *data, struct wl_data_source *source, const char *mime, int32_t fd) {
    (void)source; (void)mime;
    ClipEntry *e = (ClipEntry *)data;
    if (e && e->text) {
        ssize_t written = 0;
        while ((size_t)written < e->len) {
            ssize_t n = write(fd, e->text + written, e->len - written);
            if (n <= 0) break;
            written += n;
        }
    }
    close(fd);
}
static void source_cancelled(void *d, struct wl_data_source *s) { (void)d; wl_data_source_destroy(s); }
static void source_dnd_drop_performed(void *d, struct wl_data_source *s) { (void)d; (void)s; }
static void source_dnd_finished(void *d, struct wl_data_source *s) { (void)d; (void)s; }
static void source_action(void *d, struct wl_data_source *s, uint32_t a) { (void)d; (void)s; (void)a; }

static const struct wl_data_source_listener source_listener = {
    .target = source_target,
    .send = source_send,
    .cancelled = source_cancelled,
    .dnd_drop_performed = source_dnd_drop_performed,
    .dnd_finished = source_dnd_finished,
    .action = source_action,
};

static void copy_entry_to_clipboard(MinClip *app, int id) {
    ClipEntry *target = NULL;
    for (int i = 0; i < app->count; i++) {
        if (app->ring[i].id == id) {
            target = &app->ring[i];
            break;
        }
    }
    if (!target) return;

    struct wl_data_source *source = wl_data_device_manager_create_data_source(app->data_device_mgr);
    wl_data_source_add_listener(source, &source_listener, target);
    wl_data_source_offer(source, "text/plain;charset=utf-8");
    wl_data_source_offer(source, "text/plain");
    wl_data_source_offer(source, "UTF8_STRING");
    wl_data_device_set_selection(app->data_device, source, app->last_serial);
    wl_display_flush(app->display);
}

static void registry_global(void *data, struct wl_registry *r, uint32_t name, const char *iface, uint32_t ver) {
    (void)ver;
    MinClip *app = (MinClip *)data;
    if (strcmp(iface, wl_seat_interface.name) == 0) {
        app->seat = wl_registry_bind(r, name, &wl_seat_interface, 1);
    } else if (strcmp(iface, wl_data_device_manager_interface.name) == 0) {
        app->data_device_mgr = wl_registry_bind(r, name, &wl_data_device_manager_interface, 1);
    }
}
static void registry_global_remove(void *d, struct wl_registry *r, uint32_t n) { (void)d; (void)r; (void)n; }

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_global_remove,
};

static void handle_client_conn(MinClip *app, int cfd) {
    char req[64];
    ssize_t n = read(cfd, req, sizeof(req) - 1);
    if (n <= 0) { close(cfd); return; }
    req[n] = '\0';

    if (strncmp(req, "LIST", 4) == 0) {
        // Output format: "<id> <preview>\n"
        for (int i = 0; i < app->count; i++) {
            char line[256];
            // Format single line preview, replacing newlines with spaces
            char preview[128];
            size_t p = 0;
            for (size_t k = 0; k < app->ring[i].len && p < sizeof(preview) - 1; k++) {
                char ch = app->ring[i].text[k];
                preview[p++] = (ch == '\n' || ch == '\r' || ch == '\t') ? ' ' : ch;
            }
            preview[p] = '\0';
            int len = snprintf(line, sizeof(line), "%d\t%s\n", app->ring[i].id, preview);
            write(cfd, line, len);
        }
    } else if (strncmp(req, "COPY ", 5) == 0) {
        int id = atoi(req + 5);
        copy_entry_to_clipboard(app, id);
        write(cfd, "OK\n", 3);
    } else if (strncmp(req, "CLEAR", 5) == 0) {
        ring_clear(app);
        write(cfd, "OK\n", 3);
    }
    close(cfd);
}

static int client_ipc_request(const char *cmd) {
    const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s/%s",
             runtime_dir ? runtime_dir : "/run/user/1000", SOCK_NAME);

    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return 1;

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(fd);
        return 1;
    }

    write(fd, cmd, strlen(cmd));
    char buf[1024];
    ssize_t n;
    while ((n = read(fd, buf, sizeof(buf))) > 0) {
        write(STDOUT_FILENO, buf, n);
    }
    close(fd);
    return 0;
}
#ifdef MINCORE_UNIFIED
int minclip_main(int argc, char *argv[]) {
#else
int main(int argc, char *argv[]) {
#endif
    if (argc > 1) {
        if (strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0) {
            printf("minclip — Zero-I/O in-memory Wayland clipboard ring\n");
            printf("Usage: minclip [OPTIONS]\n");
            printf("  --daemon      Run Wayland clipboard daemon\n");
            printf("  --list, -l    Dump clipboard history to stdout (fuzzel integration)\n");
            printf("  --copy <id>   Copy item <id> back to Wayland clipboard\n");
            printf("  --clear       Clear clipboard ring\n");
            printf("  --dry-run     Validate Wayland data device binding and exit\n");
            return 0;
        }
        if (strcmp(argv[1], "--list") == 0 || strcmp(argv[1], "-l") == 0) {
            return client_ipc_request("LIST\n");
        }
        if (strcmp(argv[1], "--copy") == 0 && argc > 2) {
            char cmd[64];
            snprintf(cmd, sizeof(cmd), "COPY %s\n", argv[2]);
            return client_ipc_request(cmd);
        }
        if (strcmp(argv[1], "--clear") == 0) {
            return client_ipc_request("CLEAR\n");
        }
    }

    bool dry_run = (argc > 1 && strcmp(argv[1], "--dry-run") == 0);

    MinClip app = { .running = true, .count = 0, .next_id = 0, .last_serial = 1 };

    app.display = wl_display_connect(NULL);
    if (!app.display) {
        fprintf(stderr, "minclip: Failed to connect to Wayland display\n");
        return 1;
    }

    app.registry = wl_display_get_registry(app.display);
    wl_registry_add_listener(app.registry, &registry_listener, &app);
    wl_display_roundtrip(app.display);

    if (!app.seat || !app.data_device_mgr) {
        fprintf(stderr, "minclip: Missing wl_seat or wl_data_device_manager\n");
        wl_display_disconnect(app.display);
        return 1;
    }

    app.data_device = wl_data_device_manager_get_data_device(app.data_device_mgr, app.seat);
    wl_data_device_add_listener(app.data_device, &device_listener, &app);
    wl_display_roundtrip(app.display);

    if (dry_run) {
        printf("minclip [dry-run]: Wayland data_device_manager bound successfully\n");
        wl_data_device_destroy(app.data_device);
        wl_display_disconnect(app.display);
        return 0;
    }

    // UNIX Domain Socket Server
    const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
    struct sockaddr_un saddr;
    memset(&saddr, 0, sizeof(saddr));
    saddr.sun_family = AF_UNIX;
    snprintf(saddr.sun_path, sizeof(saddr.sun_path), "%s/%s",
             runtime_dir ? runtime_dir : "/run/user/1000", SOCK_NAME);
    unlink(saddr.sun_path);

    app.sock_fd = socket(AF_UNIX, SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC, 0);
    if (app.sock_fd < 0 || bind(app.sock_fd, (struct sockaddr *)&saddr, sizeof(saddr)) < 0) {
        fprintf(stderr, "minclip: Failed to bind IPC socket\n");
        wl_data_device_destroy(app.data_device);
        wl_display_disconnect(app.display);
        return 1;
    }
    listen(app.sock_fd, 8);

    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    sigaddset(&mask, SIGUSR1); // Swaylock / privacy wipe signal
    sigprocmask(SIG_BLOCK, &mask, NULL);
    app.sig_fd = signalfd(-1, &mask, SFD_NONBLOCK | SFD_CLOEXEC);

    app.epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    int wl_fd = wl_display_get_fd(app.display);

    struct epoll_event ev;
    ev.events = EPOLLIN; ev.data.fd = wl_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, wl_fd, &ev);
    ev.events = EPOLLIN; ev.data.fd = app.sock_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, app.sock_fd, &ev);
    ev.events = EPOLLIN; ev.data.fd = app.sig_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, app.sig_fd, &ev);

    struct epoll_event events[8];

    while (app.running) {
        while (wl_display_prepare_read(app.display) != 0) {
            wl_display_dispatch_pending(app.display);
        }
        wl_display_flush(app.display);

        int n = epoll_wait(app.epoll_fd, events, 8, -1);
        if (n < 0) {
            wl_display_cancel_read(app.display);
            if (errno == EINTR) continue;
            break;
        }

        bool wl_readable = false;
        for (int i = 0; i < n; i++) {
            if (events[i].data.fd == wl_fd) {
                wl_readable = true;
            } else if (events[i].data.fd == app.sock_fd) {
                int cfd = accept4(app.sock_fd, NULL, NULL, SOCK_CLOEXEC);
                if (cfd >= 0) {
                    handle_client_conn(&app, cfd);
                }
            } else if (events[i].data.fd == app.sig_fd) {
                struct signalfd_siginfo fdsi;
                if (read(app.sig_fd, &fdsi, sizeof(fdsi)) == sizeof(fdsi)) {
                    if (fdsi.ssi_signo == SIGUSR1) {
                        ring_clear(&app); // Screen locked -> privacy wipe!
                    } else {
                        app.running = false;
                    }
                }
            }
        }

        if (wl_readable) {
            wl_display_read_events(app.display);
            wl_display_dispatch_pending(app.display);
        } else {
            wl_display_cancel_read(app.display);
        }
    }

    ring_clear(&app);
    unlink(saddr.sun_path);
    close(app.sock_fd);
    close(app.sig_fd);
    close(app.epoll_fd);
    wl_data_device_destroy(app.data_device);
    wl_display_disconnect(app.display);

    return 0;
}
