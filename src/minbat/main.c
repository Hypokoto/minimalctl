#define _GNU_SOURCE
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/netlink.h>
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
#include <systemd/sd-bus.h>
#include <unistd.h>

#define MAX_PATH_LEN  512
#define BUFFER_SIZE   1024
#define FALLBACK_SECS 60

typedef struct {
    char path[MAX_PATH_LEN];
    int capacity;
    char status[32];
} BatteryInfo;

typedef struct {
    int notified_20;
    int notified_15;
    int last_crit_level;
    int was_discharging;
    uint32_t notif_id;
} MonitorState;

static bool discover_battery(char *out_path, size_t max_len) {
    if (access("/sys/class/power_supply/BAT0/capacity", R_OK) == 0) {
        snprintf(out_path, max_len, "/sys/class/power_supply/BAT0");
        return true;
    }
    DIR *dir = opendir("/sys/class/power_supply");
    if (!dir) return false;

    struct dirent *de;
    bool found = false;
    while ((de = readdir(dir)) != NULL) {
        if (strncmp(de->d_name, "BAT", 3) == 0 || strcmp(de->d_name, "battery") == 0) {
            char cap[MAX_PATH_LEN];
            snprintf(cap, sizeof(cap), "/sys/class/power_supply/%s/capacity", de->d_name);
            if (access(cap, R_OK) == 0) {
                snprintf(out_path, max_len, "/sys/class/power_supply/%s", de->d_name);
                found = true;
                break;
            }
        }
    }
    closedir(dir);
    return found;
}

static bool read_sysfs_attr(const char *path, char *buf, size_t max_len) {
    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0) return false;

    // Read attribute via mmap, falling back to read() if kernel returns ENODEV
    void *addr = mmap(NULL, 4096, PROT_READ, MAP_SHARED, fd, 0);
    if (addr != MAP_FAILED) {
        strncpy(buf, (char *)addr, max_len - 1);
        buf[max_len - 1] = '\0';
        munmap(addr, 4096);
    } else {
        ssize_t n = read(fd, buf, max_len - 1);
        if (n < 0) {
            close(fd);
            return false;
        }
        buf[n] = '\0';
    }
    close(fd);

    size_t len = strlen(buf);
    while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r' || buf[len - 1] == ' ')) {
        buf[--len] = '\0';
    }
    return true;
}

static bool read_battery_data(const char *bat_path, BatteryInfo *info) {
    char cap_path[MAX_PATH_LEN], stat_path[MAX_PATH_LEN], buf[64];
    snprintf(cap_path, sizeof(cap_path), "%s/capacity", bat_path);
    snprintf(stat_path, sizeof(stat_path), "%s/status", bat_path);

    if (!read_sysfs_attr(cap_path, buf, sizeof(buf))) return false;
    info->capacity = atoi(buf);

    if (!read_sysfs_attr(stat_path, buf, sizeof(buf))) return false;
    strncpy(info->status, buf, sizeof(info->status) - 1);
    info->status[sizeof(info->status) - 1] = '\0';

    strncpy(info->path, bat_path, sizeof(info->path) - 1);
    info->path[sizeof(info->path) - 1] = '\0';
    return true;
}

static void send_notification(sd_bus **bus_p, uint32_t *replaces_id, uint8_t urgency,
                              int32_t timeout_ms, const char *icon, const char *summary,
                              const char *body) {
    if (!bus_p) return;
    if (!*bus_p && sd_bus_open_user(bus_p) < 0) return;

    sd_bus_message *reply = NULL;
    sd_bus_error error = SD_BUS_ERROR_NULL;
    uint32_t in_id = replaces_id ? *replaces_id : 0;

    int r = sd_bus_call_method(*bus_p,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
        &error,
        &reply,
        "susssasa{sv}i",
        "minbat",
        in_id,
        icon,
        summary,
        body,
        0,                             // empty string array (actions)
        1, "urgency", "y", urgency,     // hints dict: { "urgency": <urgency> }
        timeout_ms);

    if (r >= 0 && reply && replaces_id) {
        uint32_t out_id = 0;
        if (sd_bus_message_read(reply, "u", &out_id) >= 0) {
            *replaces_id = out_id;
        }
    } else if (r < 0) {
        sd_bus_close(*bus_p);
        sd_bus_unref(*bus_p);
        *bus_p = NULL;
        if (replaces_id) *replaces_id = 0;
    }

    sd_bus_message_unref(reply);
    sd_bus_error_free(&error);
}

static void evaluate_battery(sd_bus **bus, const BatteryInfo *info, MonitorState *st) {
    int cap = info->capacity;
    const char *status = info->status;

    if (strcmp(status, "Charging") == 0 || strcmp(status, "Full") == 0) {
        if (st->was_discharging == 1) {
            char body[64];
            snprintf(body, sizeof(body), "Battery is charging (%d%%)", cap);
            send_notification(bus, &st->notif_id, 0, 3000, "battery-charging-symbolic",
                              "Power Connected", body);
            st->was_discharging = 0;
        }
        if (cap > 20) st->notified_20 = 0;
        if (cap > 15) st->notified_15 = 0;
        if (cap >= 7) st->last_crit_level = 7;
    } else if (strcmp(status, "Discharging") == 0) {
        if (st->was_discharging == 0) {
            st->was_discharging = 1;
            if (cap < 7) st->last_crit_level = cap + 1;
        }
        if (cap < 7) {
            if (cap < st->last_crit_level) {
                char sum[64];
                snprintf(sum, sizeof(sum), "Battery Critically Low: %d%%", cap);
                send_notification(bus, &st->notif_id, 2, 0, "battery-empty", sum,
                                  "Battery rapidly depleting! Plug in AC power immediately.");
                st->last_crit_level = cap;
            }
        } else if (cap <= 15) {
            if (!st->notified_15) {
                char sum[64];
                snprintf(sum, sizeof(sum), "Battery Warning: %d%%", cap);
                send_notification(bus, &st->notif_id, 2, 10000, "battery-caution-symbolic",
                                  sum, "Battery level dropped below 15%. Plug in charger.");
                st->notified_15 = 1;
                st->notified_20 = 1;
            }
        } else if (cap <= 20) {
            if (!st->notified_20) {
                char sum[64];
                snprintf(sum, sizeof(sum), "Battery Notice: %d%%", cap);
                send_notification(bus, &st->notif_id, 1, 6000, "battery-low-symbolic",
                                  sum, "Battery level is at 20%. Consider connecting charger.");
                st->notified_20 = 1;
            }
        }
    }
}

int main(int argc, char *argv[]) {
    bool dry_run = false;
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--dry-run") == 0) {
            dry_run = true;
        } else if (strcmp(argv[i], "--help") == 0 || strcmp(argv[i], "-h") == 0) {
            printf("minbat - Zero-wakeup battery daemon\nUsage: minbat [--dry-run]\n");
            return 0;
        }
    }

    char bat_path[MAX_PATH_LEN];
    if (!discover_battery(bat_path, sizeof(bat_path))) {
        fprintf(stderr, "minbat: No battery found in /sys/class/power_supply\n");
        return 0;
    }

    BatteryInfo info;
    if (!read_battery_data(bat_path, &info)) {
        fprintf(stderr, "minbat: Failed to read battery data from %s\n", bat_path);
        return 1;
    }

    if (dry_run) {
        printf("minbat [dry-run]: Discovered %s\n", info.path);
        printf("minbat [dry-run]: Status: %s, Capacity: %d%%\n", info.status, info.capacity);
        return 0;
    }

    sd_bus *bus = NULL;
    sd_bus_open_user(&bus);

    MonitorState state = { .notified_20 = 0, .notified_15 = 0, .last_crit_level = 7,
                           .was_discharging = 0, .notif_id = 0 };
    evaluate_battery(&bus, &info, &state);

    int nl_fd = socket(PF_NETLINK, SOCK_RAW | SOCK_CLOEXEC | SOCK_NONBLOCK,
                       NETLINK_KOBJECT_UEVENT);
    if (nl_fd >= 0) {
        struct sockaddr_nl sa = { .nl_family = AF_NETLINK, .nl_groups = 1 };
        if (bind(nl_fd, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
            close(nl_fd);
            nl_fd = -1;
        }
    }

    int timer_fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK | TFD_CLOEXEC);
    if (timer_fd >= 0) {
        struct itimerspec its = {
            .it_interval = { .tv_sec = FALLBACK_SECS, .tv_nsec = 0 },
            .it_value = { .tv_sec = FALLBACK_SECS, .tv_nsec = 0 }
        };
        timerfd_settime(timer_fd, 0, &its, NULL);
    }

    sigset_t sigmask;
    sigemptyset(&sigmask);
    sigaddset(&sigmask, SIGINT);
    sigaddset(&sigmask, SIGTERM);
    sigprocmask(SIG_BLOCK, &sigmask, NULL);
    int sig_fd = signalfd(-1, &sigmask, SFD_NONBLOCK | SFD_CLOEXEC);

    int epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    if (epoll_fd < 0) return 1;

    struct epoll_event ev;
    if (nl_fd >= 0) {
        ev.events = EPOLLIN; ev.data.fd = nl_fd;
        epoll_ctl(epoll_fd, EPOLL_CTL_ADD, nl_fd, &ev);
    }
    if (timer_fd >= 0) {
        ev.events = EPOLLIN; ev.data.fd = timer_fd;
        epoll_ctl(epoll_fd, EPOLL_CTL_ADD, timer_fd, &ev);
    }
    if (sig_fd >= 0) {
        ev.events = EPOLLIN; ev.data.fd = sig_fd;
        epoll_ctl(epoll_fd, EPOLL_CTL_ADD, sig_fd, &ev);
    }

    struct epoll_event events[8];
    char netlink_buf[BUFFER_SIZE];
    bool running = true;

    while (running) {
        int nfds = epoll_wait(epoll_fd, events, 8, -1);
        if (nfds < 0) {
            if (errno == EINTR) continue;
            break;
        }
        for (int i = 0; i < nfds; ++i) {
            if (events[i].data.fd == sig_fd) {
                running = false;
                break;
            } else if (events[i].data.fd == nl_fd) {
                ssize_t len = recv(nl_fd, netlink_buf, sizeof(netlink_buf) - 1, MSG_DONTWAIT);
                if (len > 0) {
                    netlink_buf[len] = '\0';
                    bool power_event = false;
                    for (ssize_t p = 0; p < len; p += (ssize_t)strlen(&netlink_buf[p]) + 1) {
                        if (strstr(&netlink_buf[p], "SUBSYSTEM=power_supply") ||
                            strstr(&netlink_buf[p], "POWER_SUPPLY_NAME=")) {
                            power_event = true;
                            break;
                        }
                    }
                    if (power_event && read_battery_data(bat_path, &info)) {
                        evaluate_battery(&bus, &info, &state);
                    }
                }
            } else if (events[i].data.fd == timer_fd) {
                uint64_t exp;
                if (read(timer_fd, &exp, sizeof(exp)) > 0 && read_battery_data(bat_path, &info)) {
                    evaluate_battery(&bus, &info, &state);
                }
            }
        }
    }

    if (nl_fd >= 0) close(nl_fd);
    if (timer_fd >= 0) close(timer_fd);
    if (sig_fd >= 0) close(sig_fd);
    close(epoll_fd);
    if (bus) {
        sd_bus_close(bus);
        sd_bus_unref(bus);
    }
    return 0;
}
