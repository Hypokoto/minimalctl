#define _GNU_SOURCE
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <math.h>
#include <poll.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/mman.h>
#include <sys/signalfd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/timerfd.h>
#include <sys/un.h>
#include <unistd.h>
#include <wayland-client.h>
#include <pixman.h>

#include "wlr-layer-shell-unstable-v1-client-protocol.h"
#include "xdg-shell-client-protocol.h"
#include "palette.h"

#define OSD_WIDTH   260
#define OSD_HEIGHT  38
#define OSD_RADIUS  19
#define FADE_MS     1200

typedef struct {
    struct wl_display *display;
    struct wl_compositor *compositor;
    struct wl_shm *shm;
    struct zwlr_layer_shell_v1 *layer_shell;
    struct wl_surface *surface;
    struct zwlr_layer_surface_v1 *layer_surface;
    struct wl_buffer *buffer;
    uint32_t *pixels;
    int shm_fd;
    size_t shm_size;

    int sock_fd;
    int timer_fd;
    int sig_fd;
    int epoll_fd;

    bool configured;
    bool visible;
    bool running;
    bool muted;
    char type; // 'V' for volume, 'B' for brightness
    int percent;
} MinOsd;

static void draw_pill(MinOsd *app) {
    uint32_t *buf = app->pixels;
    int w = OSD_WIDTH, h = OSD_HEIGHT, r = OSD_RADIUS;
    uint32_t bar_color = app->muted ? COLOR_MUTED : COLOR_BAR;

    for (int y = 0; y < h; y++) {
        for (int x = 0; x < w; x++) {
            // Check rounded pill boundary
            int cx = (x < r) ? r : ((x >= w - r) ? w - r - 1 : x);
            int cy = r;
            int dx = x - cx;
            int dy = y - cy;
            int dist2 = dx * dx + dy * dy;

            if (dist2 > r * r) {
                buf[y * w + x] = 0x00000000; // Transparent outside pill
            } else if (dist2 >= (r - 1) * (r - 1)) {
                buf[y * w + x] = COLOR_BORDER; // 1px border
            } else {
                buf[y * w + x] = COLOR_BG;     // Acrylic background
            }
        }
    }

    // Progress bar geometry
    int tx0 = 42, tx1 = w - 24;
    int ty0 = 14, ty1 = h - 14;
    int t_w = tx1 - tx0;
    int fill_w = (t_w * (app->percent < 0 ? 0 : (app->percent > 100 ? 100 : app->percent))) / 100;

    for (int y = ty0; y < ty1; y++) {
        for (int x = tx0; x < tx1; x++) {
            if (x < tx0 + fill_w) {
                buf[y * w + x] = bar_color;
            } else {
                buf[y * w + x] = COLOR_TRACK;
            }
        }
    }

    // Minimal 3x5 dot indicator for 'V' or 'B'
    static const uint8_t glyph_v[5] = { 0x5, 0x5, 0x5, 0x5, 0x2 }; // V
    static const uint8_t glyph_b[5] = { 0x6, 0x5, 0x6, 0x5, 0x6 }; // B
    const uint8_t *glyph = (app->type == 'B') ? glyph_b : glyph_v;

    int gx = 22, gy = (h - 10) / 2;
    for (int row = 0; row < 5; row++) {
        for (int col = 0; col < 3; col++) {
            if (glyph[row] & (1 << (2 - col))) {
                for (int sy = 0; sy < 2; sy++) {
                    for (int sx = 0; sx < 2; sx++) {
                        buf[(gy + row * 2 + sy) * w + (gx + col * 2 + sx)] = COLOR_TEXT;
                    }
                }
            }
        }
    }
}

static void show_osd(MinOsd *app) {
    if (!app->configured) return;

    draw_pill(app);
    wl_surface_attach(app->surface, app->buffer, 0, 0);
    wl_surface_damage_buffer(app->surface, 0, 0, OSD_WIDTH, OSD_HEIGHT);
    wl_surface_commit(app->surface);
    app->visible = true;

    // Reset 1.2s hide timer
    struct itimerspec its = {
        .it_interval = { 0, 0 },
        .it_value = { .tv_sec = FADE_MS / 1000, .tv_nsec = (FADE_MS % 1000) * 1000000ULL }
    };
    timerfd_settime(app->timer_fd, 0, &its, NULL);
}

static void hide_osd(MinOsd *app) {
    if (!app->visible) return;
    wl_surface_attach(app->surface, NULL, 0, 0);
    wl_surface_commit(app->surface);
    app->visible = false;
}

static void handle_command(MinOsd *app, const char *cmd) {
    char t = cmd[0];
    if (t != 'V' && t != 'B') return;
    app->type = t;

    if (t == 'V') {
        if (strcmp(cmd, "V+5") == 0) {
            system("wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 5%+ 2>/dev/null");
        } else if (strcmp(cmd, "V-5") == 0) {
            system("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%- 2>/dev/null");
        } else if (strcmp(cmd, "VM") == 0) {
            system("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle 2>/dev/null");
        }
        FILE *fp = popen("wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null", "r");
        if (fp) {
            char line[64];
            if (fgets(line, sizeof(line), fp)) {
                app->muted = (strstr(line, "[MUTED]") != NULL);
                float v = 0.0f;
                if (sscanf(line, "Volume: %f", &v) >= 1) {
                    app->percent = (int)(v * 100.0f + 0.5f);
                }
            }
            pclose(fp);
        }
    } else if (t == 'B') {
        app->muted = false;
        char dev_name[256] = {0};
        int cur = 0, max = 0;
        DIR *bl_dir = opendir("/sys/class/backlight");
        if (bl_dir) {
            struct dirent *de;
            while ((de = readdir(bl_dir)) != NULL) {
                if (de->d_name[0] == '.') continue;
                char cur_path[512], max_path[512];
                snprintf(cur_path, sizeof(cur_path), "/sys/class/backlight/%s/brightness", de->d_name);
                snprintf(max_path, sizeof(max_path), "/sys/class/backlight/%s/max_brightness", de->d_name);
                FILE *f_cur = fopen(cur_path, "r");
                FILE *f_max = fopen(max_path, "r");
                if (f_cur && f_max && fscanf(f_cur, "%d", &cur) == 1 &&
                    fscanf(f_max, "%d", &max) == 1 && max > 0) {
                    snprintf(dev_name, sizeof(dev_name), "%s", de->d_name);
                    fclose(f_cur);
                    fclose(f_max);
                    break;
                }
                if (f_cur) fclose(f_cur);
                if (f_max) fclose(f_max);
                cur = max = 0;
            }
            closedir(bl_dir);
        }

        const char *step = (strcmp(cmd, "B+10") == 0 || strcmp(cmd, "B+5") == 0) ? "+5%" : "5%-";
        char b_cmd[256];
        if (dev_name[0] != '\0') {
            snprintf(b_cmd, sizeof(b_cmd), "brightnessctl -d %s set %s >/dev/null 2>&1", dev_name, step);
        } else {
            snprintf(b_cmd, sizeof(b_cmd), "brightnessctl set %s >/dev/null 2>&1", step);
        }
        system(b_cmd);

        if (dev_name[0] != '\0' && max > 0) {
            char cur_path[512];
            snprintf(cur_path, sizeof(cur_path), "/sys/class/backlight/%s/brightness", dev_name);
            FILE *f_cur = fopen(cur_path, "r");
            if (f_cur && fscanf(f_cur, "%d", &cur) == 1) {
                app->percent = (cur * 100) / max;
            } else {
                app->percent = (cur * 100) / max;
            }
            if (f_cur) fclose(f_cur);
        } else {
            app->percent = 50;
        }
    }

    show_osd(app);
}

static void layer_surface_configure(void *data, struct zwlr_layer_surface_v1 *surface,
                                    uint32_t serial, uint32_t width, uint32_t height) {
    (void)width; (void)height;
    MinOsd *app = (MinOsd *)data;
    zwlr_layer_surface_v1_ack_configure(surface, serial);
    app->configured = true;
}

static void layer_surface_closed(void *data, struct zwlr_layer_surface_v1 *surface) {
    (void)surface;
    MinOsd *app = (MinOsd *)data;
    app->running = false;
}

static const struct zwlr_layer_surface_v1_listener layer_surface_listener = {
    .configure = layer_surface_configure,
    .closed = layer_surface_closed,
};

static void registry_global(void *data, struct wl_registry *registry,
                            uint32_t name, const char *interface, uint32_t version) {
    (void)version;
    MinOsd *app = (MinOsd *)data;
    if (strcmp(interface, wl_compositor_interface.name) == 0) {
        app->compositor = wl_registry_bind(registry, name, &wl_compositor_interface, 4);
    } else if (strcmp(interface, wl_shm_interface.name) == 0) {
        app->shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    } else if (strcmp(interface, zwlr_layer_shell_v1_interface.name) == 0) {
        app->layer_shell = wl_registry_bind(registry, name, &zwlr_layer_shell_v1_interface, 1);
    }
}

static void registry_global_remove(void *data, struct wl_registry *registry, uint32_t name) {
    (void)data; (void)registry; (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_global_remove,
};

static int init_shm_buffer(MinOsd *app) {
    int stride = OSD_WIDTH * 4;
    app->shm_size = stride * OSD_HEIGHT;

    app->shm_fd = memfd_create("minosd-shm", MFD_CLOEXEC | MFD_ALLOW_SEALING);
    if (app->shm_fd < 0) return -1;
    if (ftruncate(app->shm_fd, app->shm_size) < 0) return -1;

    app->pixels = mmap(NULL, app->shm_size, PROT_READ | PROT_WRITE, MAP_SHARED, app->shm_fd, 0);
    if (app->pixels == MAP_FAILED) return -1;

    struct wl_shm_pool *pool = wl_shm_create_pool(app->shm, app->shm_fd, app->shm_size);
    app->buffer = wl_shm_pool_create_buffer(pool, 0, OSD_WIDTH, OSD_HEIGHT, stride, WL_SHM_FORMAT_ARGB8888);
    wl_shm_pool_destroy(pool);
    return 0;
}

static int send_client_command(const char *cmd) {
    const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    snprintf(addr.sun_path, sizeof(addr.sun_path), "%s/minosd.sock",
             runtime_dir ? runtime_dir : "/run/user/1000");

    int fd = socket(AF_UNIX, SOCK_DGRAM, 0);
    if (fd < 0) return -1;

    ssize_t n = sendto(fd, cmd, strlen(cmd), 0, (struct sockaddr *)&addr, sizeof(addr));
    close(fd);
    return (n > 0) ? 0 : -1;
}
#ifdef MINCORE_UNIFIED
int minosd_main(int argc, char *argv[]) {
#else
int main(int argc, char *argv[]) {
#endif
    if (argc > 1) {
        if (strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0) {
            printf("minosd — Wayland Layer-Shell OSD Overlay (Synthwave)\n");
            printf("Usage: minosd [COMMAND | OPTIONS]\n");
            printf("Commands: V+5, V-5, VM, B+10, B-10\n");
            printf("Options: --dry-run, --daemon\n");
            return 0;
        }
        if (strcmp(argv[1], "--dry-run") != 0 && strcmp(argv[1], "--daemon") != 0) {
            // Client mode: dispatch command to daemon socket and exit
            return send_client_command(argv[1]);
        }
    }

    bool dry_run = (argc > 1 && strcmp(argv[1], "--dry-run") == 0);

    MinOsd app = {
        .running = true, .percent = 50, .type = 'V', .muted = false,
        .visible = false, .configured = false
    };

    app.display = wl_display_connect(NULL);
    if (!app.display) {
        fprintf(stderr, "minosd: Failed to connect to Wayland display\n");
        return dry_run ? 0 : 1;
    }

    struct wl_registry *registry = wl_display_get_registry(app.display);
    wl_registry_add_listener(registry, &registry_listener, &app);
    wl_display_roundtrip(app.display);

    if (!app.compositor || !app.shm || !app.layer_shell) {
        fprintf(stderr, "minosd: Missing required Wayland interfaces\n");
        wl_display_disconnect(app.display);
        return dry_run ? 0 : 1;
    }

    if (init_shm_buffer(&app) < 0) {
        fprintf(stderr, "minosd: Failed to create shm buffer\n");
        return 1;
    }

    app.surface = wl_compositor_create_surface(app.compositor);
    app.layer_surface = zwlr_layer_shell_v1_get_layer_surface(
        app.layer_shell, app.surface, NULL,
        ZWLR_LAYER_SHELL_V1_LAYER_OVERLAY, "minosd"
    );

    zwlr_layer_surface_v1_add_listener(app.layer_surface, &layer_surface_listener, &app);
    zwlr_layer_surface_v1_set_size(app.layer_surface, OSD_WIDTH, OSD_HEIGHT);
    zwlr_layer_surface_v1_set_anchor(app.layer_surface, ZWLR_LAYER_SURFACE_V1_ANCHOR_BOTTOM);
    zwlr_layer_surface_v1_set_margin(app.layer_surface, 0, 0, 64, 0); // 64px from bottom
    zwlr_layer_surface_v1_set_exclusive_zone(app.layer_surface, -1);
    zwlr_layer_surface_v1_set_keyboard_interactivity(app.layer_surface,
        ZWLR_LAYER_SURFACE_V1_KEYBOARD_INTERACTIVITY_NONE);

    wl_surface_commit(app.surface);
    wl_display_roundtrip(app.display);

    if (dry_run) {
        printf("minosd [dry-run]: Layer shell overlay bound successfully (%dx%d bottom)\n",
               OSD_WIDTH, OSD_HEIGHT);
        zwlr_layer_surface_v1_destroy(app.layer_surface);
        wl_surface_destroy(app.surface);
        wl_buffer_destroy(app.buffer);
        munmap(app.pixels, app.shm_size);
        close(app.shm_fd);
        wl_display_disconnect(app.display);
        return 0;
    }

    // Setup UNIX domain socket for commands
    const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
    struct sockaddr_un saddr;
    memset(&saddr, 0, sizeof(saddr));
    saddr.sun_family = AF_UNIX;
    snprintf(saddr.sun_path, sizeof(saddr.sun_path), "%s/minosd.sock",
             runtime_dir ? runtime_dir : "/run/user/1000");
    unlink(saddr.sun_path);

    app.sock_fd = socket(AF_UNIX, SOCK_DGRAM | SOCK_NONBLOCK | SOCK_CLOEXEC, 0);
    if (app.sock_fd < 0 || bind(app.sock_fd, (struct sockaddr *)&saddr, sizeof(saddr)) < 0) {
        fprintf(stderr, "minosd: Failed to bind command socket\n");
        return 1;
    }

    app.timer_fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK | TFD_CLOEXEC);

    sigset_t mask;
    sigemptyset(&mask);
    sigaddset(&mask, SIGINT);
    sigaddset(&mask, SIGTERM);
    sigprocmask(SIG_BLOCK, &mask, NULL);
    app.sig_fd = signalfd(-1, &mask, SFD_NONBLOCK | SFD_CLOEXEC);

    app.epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    int wl_fd = wl_display_get_fd(app.display);

    struct epoll_event ev;
    ev.events = EPOLLIN; ev.data.fd = wl_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, wl_fd, &ev);
    ev.events = EPOLLIN; ev.data.fd = app.sock_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, app.sock_fd, &ev);
    ev.events = EPOLLIN; ev.data.fd = app.timer_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, app.timer_fd, &ev);
    ev.events = EPOLLIN; ev.data.fd = app.sig_fd;
    epoll_ctl(app.epoll_fd, EPOLL_CTL_ADD, app.sig_fd, &ev);

    struct epoll_event events[8];
    char cmd_buf[32];

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
                ssize_t len = recv(app.sock_fd, cmd_buf, sizeof(cmd_buf) - 1, MSG_DONTWAIT);
                if (len > 0) {
                    cmd_buf[len] = '\0';
                    handle_command(&app, cmd_buf);
                }
            } else if (events[i].data.fd == app.timer_fd) {
                uint64_t exp;
                if (read(app.timer_fd, &exp, sizeof(exp)) > 0) {
                    hide_osd(&app);
                }
            } else if (events[i].data.fd == app.sig_fd) {
                app.running = false;
            }
        }

        if (wl_readable) {
            wl_display_read_events(app.display);
            wl_display_dispatch_pending(app.display);
        } else {
            wl_display_cancel_read(app.display);
        }
    }

    unlink(saddr.sun_path);
    close(app.sock_fd);
    close(app.timer_fd);
    close(app.sig_fd);
    close(app.epoll_fd);
    zwlr_layer_surface_v1_destroy(app.layer_surface);
    wl_surface_destroy(app.surface);
    wl_buffer_destroy(app.buffer);
    munmap(app.pixels, app.shm_size);
    close(app.shm_fd);
    wl_display_disconnect(app.display);

    return 0;
}
