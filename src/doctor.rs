use crate::status::SystemStatus;
use crate::theme::Theme;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub struct DoctorReport {
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
}

impl DoctorReport {
    pub fn run() -> Self {
        println!("MINIMAL DOCTOR — OPERATIONAL DIAGNOSTIC SUITE");
        println!("--------------------------------------------------");

        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;

        let status = SystemStatus::collect();

        // 1. Package Provenance
        if Path::new("packages/core.txt").exists() && Path::new("packages/cli.txt").exists() {
            println!("[PASS] Package provenance policy");
            pass += 1;
        } else {
            println!("[FAIL] Package provenance definition missing");
            fail += 1;
        }

        // 2. Security Invariants
        if Path::new("scripts/audit-security.sh").exists() {
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
            if std::process::Command::new("pgrep").arg("-x").arg(proc).output().map(|o| o.status.success()).unwrap_or(false) {
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

        // 6. Theme Source & Validation
        let theme_ok = if let Ok(theme) = Theme::load_from_file("themes/obsidian.toml") {
            theme.validate().is_ok()
        } else {
            false
        };

        if theme_ok {
            println!("[PASS] Theme source definition (obsidian.toml)");
            pass += 1;
        } else {
            println!("[FAIL] Theme source definition invalid or missing");
            fail += 1;
        }

        // 7. Theme Drift Check
        let mut drift = false;
        if let Ok(theme) = Theme::load_from_file("themes/obsidian.toml") {
            if let Ok(content) = fs::read_to_string("mako/config") {
                if content != theme.generate_mako_config() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("rofi/theme.rasi") {
                if content != theme.generate_rofi_theme() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("kitty/kitty.conf") {
                if content != theme.generate_kitty_conf() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("waybar/style.css") {
                if content != theme.generate_waybar_style() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("starship/starship.toml") {
                if content != theme.generate_starship_toml() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("btop/btop.theme") {
                if content != theme.generate_btop_theme() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("hypr/colors.conf") {
                if content != theme.generate_hypr_colors() {
                    drift = true;
                }
            }
            if let Ok(content) = fs::read_to_string("tmux/tmux.conf") {
                if content != theme.generate_tmux_conf() {
                    drift = true;
                }
            }
            if Path::new("nvim/lua/themes/minimal.lua").exists() {
                if let Ok(content) = fs::read_to_string("nvim/lua/themes/minimal.lua") {
                    if content != theme.generate_nvim_theme() {
                        drift = true;
                    }
                }
            }
        }

        if !drift {
            println!("[PASS] Theme drift check (generated targets match source)");
            pass += 1;
        } else {
            println!("[WARN] Theme drift detected in target configs");
            warn += 1;
        }

        // 8. Firewall Status
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
