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

fn main() {
    println!("cargo:rerun-if-changed=src/minbat/main.c");
    println!("cargo:rerun-if-changed=src/minosd/main.c");
    println!("cargo:rerun-if-changed=src/minosd/palette.h");
    println!("cargo:rerun-if-changed=src/minclip/main.c");

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => return,
    };

    let target_dir = Path::new(&manifest_dir).join("target");
    let _ = fs::create_dir_all(&target_dir);

    let compiler = cc::Build::new().get_compiler();

    // 1. minbat
    let minbat_src = Path::new(&manifest_dir).join("src/minbat/main.c");
    if minbat_src.exists() {
        let out_bin = target_dir.join("minbat");
        require_success(
            Command::new(compiler.path())
                .args(["-O2", "-Wall", "-Wextra"])
                .arg(&minbat_src)
                .args(["-lsystemd", "-o"])
                .arg(&out_bin)
                .status(),
            "minbat compilation",
        );
    }

    // 2. minosd
    let minosd_src = Path::new(&manifest_dir).join("src/minosd/main.c");
    if minosd_src.exists() {
        let out_bin = target_dir.join("minosd");
        let minosd_dir = Path::new(&manifest_dir).join("src/minosd");
        let proto_layer = minosd_dir.join("wlr-layer-shell-unstable-v1-protocol.c");
        let proto_xdg = minosd_dir.join("xdg-shell-protocol.c");
        require_success(
            Command::new(compiler.path())
                .args([
                    "-O2",
                    "-Wall",
                    "-Wextra",
                    "-Isrc/minosd",
                    "-I/usr/include/pixman-1",
                ])
                .arg(&minosd_src)
                .arg(&proto_layer)
                .arg(&proto_xdg)
                .args(["-lwayland-client", "-lpixman-1", "-lm", "-o"])
                .arg(&out_bin)
                .status(),
            "minosd compilation",
        );
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
    if mincore_src.exists() && minbat_src.exists() && minosd_src.exists() && minclip_src.exists() {
        let out_bin = target_dir.join("mincore");
        let minosd_dir = Path::new(&manifest_dir).join("src/minosd");
        let proto_layer = minosd_dir.join("wlr-layer-shell-unstable-v1-protocol.c");
        let proto_xdg = minosd_dir.join("xdg-shell-protocol.c");
        let status = Command::new(compiler.path())
            .args([
                "-O2",
                "-Wall",
                "-Wextra",
                "-DMINCORE_UNIFIED",
                "-Isrc/minosd",
                "-I/usr/include/pixman-1",
            ])
            .arg(&mincore_src)
            .arg(&minbat_src)
            .arg(&minosd_src)
            .arg(&proto_layer)
            .arg(&proto_xdg)
            .arg(&minclip_src)
            .args(["-lsystemd", "-lwayland-client", "-lpixman-1", "-lm", "-o"])
            .arg(&out_bin)
            .status();

        match status {
            Ok(st) if st.success() => {}
            Ok(st) => panic!("mincore compilation failed with exit status: {}", st),
            Err(e) => panic!("failed to execute {}: {}", compiler.path().display(), e),
        }
    }
}
