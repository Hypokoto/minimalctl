use crate::status::SystemStatus;
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

        // 6. Theme Source Definition (optional or standalone)
        let synthwave_path = root.join("themes/synthwave.toml");
        if synthwave_path.exists() {
            let theme_ok = if let Ok(content) = fs::read_to_string(&synthwave_path) {
                content.parse::<toml::Value>().map(|v| v.get("tokens").is_some()).unwrap_or(false)
            } else {
                false
            };

            if theme_ok {
                println!("[PASS] Theme source definition (synthwave.toml valid)");
                pass += 1;
            } else {
                println!("[FAIL] Theme source definition invalid (themes/synthwave.toml)");
                fail += 1;
            }
        } else {
            println!("[PASS] Theme system: standalone static configurations (themes directory omitted)");
            pass += 1;
        }

        // 7. Desktop Target Configurations Integrity
        let mut targets_ok = true;
        let targets = [
            "starship/starship.toml",
            "btop/btop.theme",
            "tmux/tmux.conf",
            "labwc/themerc",
            "foot/foot.ini",
            "fuzzel/fuzzel.ini",
            "nvim/lua/themes/minimal.lua",
        ];
        for target in targets {
            let p = root.join(target);
            if !p.exists() || fs::metadata(&p).map(|m| m.len() == 0).unwrap_or(true) {
                println!("[FAIL] Missing or empty target config: {}", target);
                targets_ok = false;
            }
        }
        if targets_ok {
            println!("[PASS] Desktop configuration targets intact (starship, btop, tmux, labwc, foot, fuzzel, nvim)");
            pass += 1;
        } else {
            println!("[WARN] One or more configuration targets missing or empty");
            warn += 1;
        }

        // 8. Wayland Desktop Stack Availability (labwc, foot, fuzzel, mako)
        let mut wayland_ok = true;
        let path_env = std::env::var("PATH").unwrap_or_default();
        let path_dirs: Vec<std::path::PathBuf> = std::env::split_paths(&path_env).collect();

        for binary in &["labwc", "foot", "fuzzel", "mako"] {
            let available = path_dirs.iter().any(|dir| {
                let candidate = dir.join(binary);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = candidate.metadata() {
                        meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
                    } else {
                        false
                    }
                }
                #[cfg(not(unix))]
                {
                    candidate.is_file()
                }
            });

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
