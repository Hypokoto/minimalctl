use std::process::Command;

pub fn handle_hook(hook_type: &str, payload: &str) {
    if hook_type == "display" {
        println!("minctl hook: Received display profile event [{}]", payload);

        // 1. Reload Wallpaper daemon gracefully
        let _ = Command::new("systemctl")
            .args(&["--user", "restart", "minimal-wallpaper.service"])
            .status();

        // 2. Restart Layer-shell OSD cleanly via systemd
        println!("minctl hook: Restarting OSD overlay");
        let _ = Command::new("systemctl")
            .args(&["--user", "restart", "minimal-minosd.service"])
            .status();

    } else {
        eprintln!("Unknown hook type: {}", hook_type);
        std::process::exit(1);
    }
}
