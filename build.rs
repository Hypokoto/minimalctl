use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/minbat/main.c");
    println!("cargo:rerun-if-changed=src/minosd/main.c");
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
        let _ = Command::new(compiler.path())
            .args(&["-O2", "-Wall", "-Wextra"])
            .arg(&minbat_src)
            .args(&["-lsystemd", "-o"])
            .arg(&out_bin)
            .status();
    }

    // 2. minosd
    let minosd_src = Path::new(&manifest_dir).join("src/minosd/main.c");
    if minosd_src.exists() {
        let out_bin = target_dir.join("minosd");
        let minosd_dir = Path::new(&manifest_dir).join("src/minosd");
        let proto_layer = minosd_dir.join("wlr-layer-shell-unstable-v1-protocol.c");
        let proto_xdg = minosd_dir.join("xdg-shell-protocol.c");
        let _ = Command::new(compiler.path())
            .args(&["-O2", "-Wall", "-Wextra", "-Isrc/minosd", "-I/usr/include/pixman-1"])
            .arg(&minosd_src)
            .arg(&proto_layer)
            .arg(&proto_xdg)
            .args(&["-lwayland-client", "-lpixman-1", "-lm", "-o"])
            .arg(&out_bin)
            .status();
    }

    // 3. minclip
    let minclip_src = Path::new(&manifest_dir).join("src/minclip/main.c");
    if minclip_src.exists() {
        let out_bin = target_dir.join("minclip");
        let _ = Command::new(compiler.path())
            .args(&["-O2", "-Wall", "-Wextra"])
            .arg(&minclip_src)
            .args(&["-lwayland-client", "-o"])
            .arg(&out_bin)
            .status();
    }
}
