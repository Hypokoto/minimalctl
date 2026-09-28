use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/minbat/main.c");

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => return,
    };
    let target_dir = Path::new(&manifest_dir).join("target");
    let _ = fs::create_dir_all(&target_dir);

    let minbat_src = Path::new(&manifest_dir).join("src/minbat/main.c");
    if minbat_src.exists() {
        let compiler = cc::Build::new().get_compiler();
        let out_bin = target_dir.join("minbat");
        let status = Command::new(compiler.path())
            .arg("-O2")
            .arg("-Wall")
            .arg("-Wextra")
            .arg(&minbat_src)
            .arg("-lsystemd")
            .arg("-o")
            .arg(&out_bin)
            .status();

        match status {
            Ok(s) if s.success() => {},
            _ => eprintln!("cargo:warning=Failed to compile src/minbat/main.c into {:?}", out_bin),
        }
    }
}
