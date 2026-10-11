use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn require_success(status: std::io::Result<std::process::ExitStatus>, label: &str) {
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => panic!("{} failed with exit status: {}", label, status),
        Err(error) => panic!("failed to run {}: {}", label, error),
    }
}

fn get_pkg_config_cflags(lib: &str) -> String {
    if let Ok(output) = Command::new("pkg-config").args(["--cflags", lib]).output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    String::new()
}

fn main() {
    println!("cargo:rerun-if-changed=src/mincore/main.c");
    println!("cargo:rerun-if-changed=src/minosd/main.c");
    println!("cargo:rerun-if-changed=src/minosd/palette.h");
    println!("cargo:rerun-if-changed=src/minclip/main.c");
    println!("cargo:rerun-if-changed=src/minosd/wlr-layer-shell-unstable-v1-protocol.c");
    println!("cargo:rerun-if-changed=src/minosd/xdg-shell-protocol.c");

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => return,
    };

    let target_dir = Path::new(&manifest_dir).join("target");
    let _ = fs::create_dir_all(&target_dir);

    let compiler = cc::Build::new().get_compiler();
    let pixman_cflags = get_pkg_config_cflags("pixman-1");

    // 2. minosd
    let minosd_src = Path::new(&manifest_dir).join("src/minosd/main.c");
    if minosd_src.exists() {
        let out_bin = target_dir.join("minosd");
        let minosd_dir = Path::new(&manifest_dir).join("src/minosd");
        let proto_layer = minosd_dir.join("wlr-layer-shell-unstable-v1-protocol.c");
        let proto_xdg = minosd_dir.join("xdg-shell-protocol.c");

        let mut cmd = Command::new(compiler.path());
        cmd.args(["-O2", "-Wall", "-Wextra", "-Isrc/minosd"]);
        if !pixman_cflags.is_empty() {
            cmd.arg(&pixman_cflags);
        } else {
            cmd.arg("-I/usr/include/pixman-1"); // fallback
        }
        cmd.arg(&minosd_src)
           .arg(&proto_layer)
           .arg(&proto_xdg)
           .args(["-lwayland-client", "-lpixman-1", "-lm", "-o"])
           .arg(&out_bin);

        require_success(cmd.status(), "minosd compilation");
    }

    // 3. minclip
    let minclip_src = Path::new(&manifest_dir).join("src/minclip/main.c");
    if minclip_src.exists() {
        let out_bin = target_dir.join("minclip");
        require_success(
            Command::new(compiler.path())
                .args(["-O2", "-Wall", "-Wextra"])
                .arg(&minclip_src)
                .args(["-lwayland-client", "-o"])
                .arg(&out_bin)
                .status(),
            "minclip compilation",
        );
    }

    // 4. mincore (unified multi-call binary)
    let mincore_src = Path::new(&manifest_dir).join("src/mincore/main.c");
    let minbat_src = Path::new(&manifest_dir).join("src/minbat/main.c");
    if mincore_src.exists() && minosd_src.exists() && minclip_src.exists() {
        let out_bin = target_dir.join("mincore");
        let minosd_dir = Path::new(&manifest_dir).join("src/minosd");
        let proto_layer = minosd_dir.join("wlr-layer-shell-unstable-v1-protocol.c");
        let proto_xdg = minosd_dir.join("xdg-shell-protocol.c");

        let mut cmd = Command::new(compiler.path());
        cmd.args(["-O2", "-Wall", "-Wextra", "-DMINCORE_UNIFIED", "-Isrc/minosd"]);
        if !pixman_cflags.is_empty() {
            cmd.arg(&pixman_cflags);
        } else {
            cmd.arg("-I/usr/include/pixman-1"); // fallback
        }
        cmd.arg(&mincore_src)
           .arg(&minosd_src)
           .arg(&proto_layer)
           .arg(&proto_xdg)
           .arg(&minclip_src);
        if minbat_src.exists() {
            cmd.arg(&minbat_src);
        }
        cmd.args(["-lsystemd", "-lwayland-client", "-lpixman-1", "-lm", "-o"])
           .arg(&out_bin);

        let status = cmd.status();

        match status {
            Ok(st) if st.success() => {}
            Ok(st) => panic!("mincore compilation failed with exit status: {}", st),
            Err(e) => panic!("failed to execute {}: {}", compiler.path().display(), e),
        }
    }
}
