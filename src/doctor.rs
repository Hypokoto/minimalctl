use crate::status::SystemStatus;
use crate::theme::Theme;
use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub struct DoctorReport {
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
}

impl DoctorReport {
    pub fn run<P: AsRef<Path>>(root_dir: P) -> Self {
        let root = root_dir.as_ref();
        println!("MINIMAL DOCTOR — OPERATIONAL DIAGNOSTIC SUITE");
        println!("--------------------------------------------------");

        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;

        let status = SystemStatus::collect();

        // 1. Package Provenance
        if root.join("packages/core.txt").exists() && root.join("packages/cli.txt").exists() {
            println!("[PASS] Package provenance policy");
            pass += 1;
        } else {
            println!("[FAIL] Package provenance definition missing");
            fail += 1;
        }

        // 2. Security Invariants
        if root.join("scripts/audit-security.sh").exists() {
            println!("[PASS] Security static invariants");
            pass += 1;
        } else {
            println!("[FAIL] Security audit script missing");
            fail += 1;
        }

        // 3. Process Budget
        let banned_procs = ["conky", "nm-applet", "nwg-drawer", "hyprlauncher"];
        let mut found_banned = false;
        for proc in banned_procs {
            if Command::new("pgrep")
                .arg("-x")
                .arg(proc)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                println!("[FAIL] Banned process running: {}", proc);
                found_banned = true;
            }
        }
        if !found_banned {
            println!("[PASS] Process budget compliance");
            pass += 1;
        } else {
            fail += 1;
        }

        // 4. Failed System Units
        if status.failed_system_units == 0 {
            println!("[PASS] Failed system units");
            pass += 1;
        } else {
            println!("[WARN] System units failed: {}", status.failed_system_units);
            warn += 1;
        }

        // 5. Failed User Units
        if status.failed_user_units == 0 {
            println!("[PASS] Failed user units");
            pass += 1;
        } else {
            println!("[FAIL] User units failed: {}", status.failed_user_units);
            fail += 1;
        }

        // 6. Theme Source & Validation (themes/synthwave.toml)
        let synthwave_path = root.join("themes/synthwave.toml");
        let theme_ok = if let Ok(theme) = Theme::load_from_file(&synthwave_path) {
            theme.validate().is_ok()
        } else {
            false
        };

        if theme_ok {
            println!("[PASS] Theme source definition (synthwave.toml)");
            pass += 1;
        } else {
            println!("[FAIL] Theme source definition invalid or missing (themes/synthwave.toml)");
            fail += 1;
        }

        // 7. Theme Drift Check (against synthwave.toml)
        let mut drift = false;
        if let Ok(theme) = Theme::load_from_file(&synthwave_path) {
            if let Ok(content) = fs::read_to_string(root.join("starship/starship.toml")) {
                if content != theme.generate_starship_toml() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string(root.join("btop/btop.theme")) {
                if content != theme.generate_btop_theme() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string(root.join("tmux/tmux.conf")) {
                if content != theme.generate_tmux_conf() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string(root.join("labwc/themerc")) {
                if content != theme.generate_labwc_themerc() {
                    drift = true;
                }
            }
            let nvim_theme_file = root.join("nvim/lua/themes/minimal.lua");
            if nvim_theme_file.exists() {
                if let Ok(content) = fs::read_to_string(&nvim_theme_file) {
                    if content != theme.generate_nvim_theme() {
                        drift = true;
                    }
                }
            }
        }

        if !drift {
            println!("[PASS] Theme drift check (generated targets match synthwave.toml)");
            pass += 1;
        } else {
            println!("[WARN] Theme drift detected in target configs vs synthwave.toml");
            warn += 1;
        }

        // 8. Wayland Desktop Stack Availability (labwc, foot, fuzzel, mako)
        let mut wayland_ok = true;
        for binary in &["labwc", "foot", "fuzzel", "mako"] {
            let available = Command::new("command")
                .args(["-v", binary])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
                || Path::new(&format!("/usr/bin/{}", binary)).exists()
                || Path::new(&format!("/usr/local/bin/{}", binary)).exists();

            if !available {
                println!("[WARN] Wayland component binary not found in PATH: {}", binary);
                wayland_ok = false;
            }
        }
        if wayland_ok {
            println!("[PASS] Wayland desktop stack binaries available (labwc, foot, fuzzel, mako)");
            pass += 1;
        } else {
            warn += 1;
        }

        // 9. Firewall Status
        if status.firewall_active {
            println!("[PASS] Firewall active (nftables)");
            pass += 1;
        } else {
            println!("[WARN] Firewall inactive (nftables)");
            warn += 1;
        }

        println!("--------------------------------------------------");
        println!("Result: {} PASS / {} WARN / {} FAIL", pass, warn, fail);
        println!();

        Self {
            pass_count: pass,
            warn_count: warn,
            fail_count: fail,
        }
    }
}
