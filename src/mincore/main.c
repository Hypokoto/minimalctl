#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <libgen.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/prctl.h>

int minbat_main(int argc, char *argv[]);
int minosd_main(int argc, char *argv[]);
int minclip_main(int argc, char *argv[]);

static void print_help(const char *prog) {
    printf("mincore — Unified Multi-Call Wayland Desktop Daemon & Control Plane\n\n");
    printf("Usage:\n");
    printf("  %s bat [args...]       Battery netlink epoll daemon (minbat)\n", prog);
    printf("  %s osd [args...]       Layer-shell OSD daemon and IPC client (minosd)\n", prog);
    printf("  %s clip [args...]      In-memory clipboard daemon and client (minclip)\n", prog);
    printf("  %s status              Show running status of desktop daemons\n", prog);
    printf("  %s --daemon            Launch all 3 daemons (bat, osd, clip) in background\n", prog);
    printf("  %s --dry-run           Run dry-run verification across all subsystems\n", prog);
    printf("  %s help                Show this help message\n\n", prog);
    printf("Multi-call symlinks supported:\n");
    printf("  minbat  -> dispatches directly to battery monitor\n");
    printf("  minosd  -> dispatches directly to OSD overlay\n");
    printf("  minclip -> dispatches directly to clipboard manager\n");
}

static int run_status(void) {
    printf("=== MINIMAL DESKTOP STATUS (NATIVE) ===\n\n");
    const char *daemons[] = {"minbat", "minosd", "minclip", "labwc", "foot", "mako"};
    for (size_t i = 0; i < sizeof(daemons) / sizeof(daemons[0]); i++) {
        char cmd[256];
        snprintf(cmd, sizeof(cmd), "pgrep -u %d -x %s >/dev/null 2>&1 || pgrep -u %d -f 'mincore %s' >/dev/null 2>&1",
                 getuid(), daemons[i], getuid(), daemons[i]);
        int running = (system(cmd) == 0);
        printf("  %-18s %s\n", daemons[i], running ? "● Running" : "○ Stopped");
    }
    return 0;
}

static int run_dry_run(void) {
    printf("=== MINCORE DRY-RUN VERIFICATION ===\n");
    char *dry_argv[] = {"mincore", "--dry-run", NULL};

    printf("\n[1/3] Testing minbat...\n");
    int rc_bat = minbat_main(2, dry_argv);

    printf("\n[2/3] Testing minosd...\n");
    int rc_osd = minosd_main(2, dry_argv);

    printf("\n[3/3] Testing minclip...\n");
    int rc_clip = minclip_main(2, dry_argv);

    if (rc_bat != 0 || rc_osd != 0 || rc_clip != 0) {
        fprintf(stderr, "\n[FAIL] Subsystem verification failed: bat=%d osd=%d clip=%d\n",
                rc_bat, rc_osd, rc_clip);
        return 1;
    }

    printf("\n[OK] All 3 subsystems passed dry-run verification.\n");
    return 0;
}

static int run_daemon_supervisor(void) {
    printf("mincore: Launching background daemons (bat, osd, clip)...\n");

    pid_t p_bat = fork();
    if (p_bat < 0) {
        perror("mincore: fork minbat failed");
        return 1;
    }
    if (p_bat == 0) {
        prctl(PR_SET_NAME, "minbat", 0, 0, 0);
        char *b_argv[] = {"minbat", NULL};
        int rc = minbat_main(1, b_argv);
        _exit(rc);
    }

    pid_t p_osd = fork();
    if (p_osd < 0) {
        perror("mincore: fork minosd failed");
        return 1;
    }
    if (p_osd == 0) {
        prctl(PR_SET_NAME, "minosd", 0, 0, 0);
        char *o_argv[] = {"minosd", "--daemon", NULL};
        int rc = minosd_main(2, o_argv);
        _exit(rc);
    }

    pid_t p_clip = fork();
    if (p_clip < 0) {
        perror("mincore: fork minclip failed");
        return 1;
    }
    if (p_clip == 0) {
        prctl(PR_SET_NAME, "minclip", 0, 0, 0);
        char *c_argv[] = {"minclip", "--daemon", NULL};
        int rc = minclip_main(2, c_argv);
        _exit(rc);
    }

    /* Verify children didn't fail immediately on startup */
    usleep(50000);
    int status;
    if (waitpid(p_bat, &status, WNOHANG) > 0 && WIFEXITED(status) && WEXITSTATUS(status) != 0) {
        fprintf(stderr, "mincore: minbat exited early with code %d\n", WEXITSTATUS(status));
        return 1;
    }
    if (waitpid(p_osd, &status, WNOHANG) > 0 && WIFEXITED(status) && WEXITSTATUS(status) != 0) {
        fprintf(stderr, "mincore: minosd exited early with code %d\n", WEXITSTATUS(status));
        return 1;
    }
    if (waitpid(p_clip, &status, WNOHANG) > 0 && WIFEXITED(status) && WEXITSTATUS(status) != 0) {
        fprintf(stderr, "mincore: minclip exited early with code %d\n", WEXITSTATUS(status));
        return 1;
    }

    printf("mincore: Daemons spawned (bat: %d, osd: %d, clip: %d)\n", p_bat, p_osd, p_clip);
    return 0;
}

int main(int argc, char *argv[]) {
    char *base = basename(argv[0]);
    if (strstr(base, "minbat") != NULL) return minbat_main(argc, argv);
    if (strstr(base, "minosd") != NULL) return minosd_main(argc, argv);
    if (strstr(base, "minclip") != NULL) return minclip_main(argc, argv);

    if (argc < 2) {
        print_help(argv[0]);
        return 1;
    }

    if (strcmp(argv[1], "bat") == 0 || strcmp(argv[1], "minbat") == 0) {
        return minbat_main(argc - 1, argv + 1);
    }
    if (strcmp(argv[1], "osd") == 0 || strcmp(argv[1], "minosd") == 0) {
        return minosd_main(argc - 1, argv + 1);
    }
    if (strcmp(argv[1], "clip") == 0 || strcmp(argv[1], "minclip") == 0) {
        return minclip_main(argc - 1, argv + 1);
    }
    if (strcmp(argv[1], "status") == 0) {
        return run_status();
    }
    if (strcmp(argv[1], "--dry-run") == 0) {
        return run_dry_run();
    }
    if (strcmp(argv[1], "--daemon") == 0) {
        return run_daemon_supervisor();
    }
    if (strcmp(argv[1], "help") == 0 || strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0) {
        print_help(argv[0]);
        return 0;
    }

    fprintf(stderr, "mincore: unknown command '%s'\n", argv[1]);
    print_help(argv[0]);
    return 1;
}
