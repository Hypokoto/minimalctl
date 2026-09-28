use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
pub struct AuditReport {
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
}

impl AuditReport {
    pub fn run<P: AsRef<Path>>(root_dir: P) -> Self {
        let root = root_dir.as_ref();
        println!("MINIMAL AUDIT — SECURITY & INTEGRITY VERIFICATION");
        println!("--------------------------------------------------");

        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;

        // 1. Script Security Invariants (No unsafe eval, remote execution, unquoted PID temp files)
        let mut script_violation = false;
        let mut script_paths: Vec<_> = ["deploy.sh", "install.sh", "tty-init.sh"]
            .iter()
            .map(|s| root.join(s))
            .filter(|p| p.exists())
            .collect();

        for dir in ["labwc/scripts", "scripts", "systemd", "zsh", "security"] {
            let p = root.join(dir);
            if !p.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(&p) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "sh" || ext == "zsh") {
                        script_paths.push(path);
                    }
                }
            }
        }

        for path in script_paths {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "audit-security.sh")
            {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                for (i, line) in content.lines().enumerate() {
                    if line.contains("eval ") && !line.contains("Ignore/Safe") {
                        println!("[FAIL] Unsafe eval in {}:{}", path.display(), i + 1);
                        script_violation = true;
                    }
                    if (line.contains("curl") || line.contains("wget")) && line.contains("| bash")
                    {
                        println!(
                            "[FAIL] Remote exec pattern in {}:{}",
                            path.display(),
                            i + 1
                        );
                        script_violation = true;
                    }
                    if line.contains("/tmp/") && line.contains("$$") {
                        println!(
                            "[FAIL] Predictable temp file with PID in {}:{}",
                            path.display(),
                            i + 1
                        );
                        script_violation = true;
                    }
                }
            }
        }
        if !script_violation {
            println!("[PASS] Script security invariants (no eval / curl|bash / predictable tmp)");
            pass += 1;
        } else {
            fail += 1;
        }

        // 2. Secret Scan (Gitleaks check)
        let gitleaks_available = Command::new("gitleaks")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if gitleaks_available {
            let res = Command::new("gitleaks")
                .args(["detect", "--no-banner", "--config", ".gitleaks.toml"])
                .current_dir(root)
                .output();
            match res {
                Ok(output) if output.status.success() => {
                    println!("[PASS] Secret scan clean (0 leaks detected via gitleaks)");
                    pass += 1;
                }
                Ok(_) => {
                    println!("[FAIL] Secret scan detected potential secrets or credentials");
                    fail += 1;
                }
                Err(e) => {
                    println!("[WARN] Failed to invoke gitleaks: {}", e);
                    warn += 1;
                }
            }
        } else {
            println!("[WARN] Gitleaks binary not found in PATH — skipping automated secret detection");
            warn += 1;
        }

        // 3. XML Syntax Integrity
        let mut xml_ok = true;
        for xml_file in &["labwc/rc.xml", "labwc/menu.xml"] {
            let p = root.join(xml_file);
            if p.exists() {
                let status = Command::new("xmllint")
                    .arg("--noout")
                    .arg(&p)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if !status {
                    println!("[FAIL] XML syntax error in {}", p.display());
                    xml_ok = false;
                }
            }
        }
        if xml_ok {
            println!("[PASS] Desktop XML syntax valid (labwc rc.xml and menu.xml)");
            pass += 1;
        } else {
            fail += 1;
        }

        // 4. Foot Configuration Validation
        let foot_available = Command::new("foot")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if foot_available {
            let foot_cfg = root.join("foot/foot.ini");
            let status = Command::new("foot")
                .args(["-C", "-c"])
                .arg(&foot_cfg)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if status {
                println!("[PASS] Terminal configuration valid (foot -C syntax verification)");
                pass += 1;
            } else {
                println!("[FAIL] Terminal configuration error in {}", foot_cfg.display());
                fail += 1;
            }
        } else {
            println!("[WARN] Foot terminal binary not found in PATH");
            warn += 1;
        }

        // 5. Script Executable Permissions
        let mut perms_ok = true;
        let scripts = [
            "deploy.sh",
            "install.sh",
            "tty-init.sh",
            "scripts/battery-monitor.sh",
            "labwc/scripts/dpms.sh",
            "labwc/scripts/nightlight.sh",
            "labwc/scripts/ocr.sh",
            "labwc/scripts/osd-brightness.sh",
            "labwc/scripts/osd-volume.sh",
            "labwc/scripts/powermenu.sh",
            "labwc/scripts/screen-record.sh",
            "labwc/scripts/clipboard.sh",
        ];

        #[cfg(unix)]
        use std::os::unix::fs::PermissionsExt;

        for s in scripts {
            let p = root.join(s);
            if p.exists() {
                if let Ok(meta) = fs::metadata(&p) {
                    #[cfg(unix)]
                    if meta.permissions().mode() & 0o111 == 0 {
                        println!("[FAIL] Executable bit missing on {}", p.display());
                        perms_ok = false;
                    }
                }
            }
        }
        if perms_ok {
            println!("[PASS] Executable permissions verified on deployment & desktop scripts");
            pass += 1;
        } else {
            fail += 1;
        }

        // 6. Pre-commit Configuration
        if root.join(".pre-commit-config.yaml").exists() {
            println!("[PASS] Pre-commit configuration present (.pre-commit-config.yaml)");
            pass += 1;
        } else {
            println!("[FAIL] Missing .pre-commit-config.yaml");
            fail += 1;
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
