use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn run_daemon() -> std::io::Result<()> {
    // Start tmux in control mode using `script` to allocate a pseudo-TTY
    let mut child = Command::new("script")
        .arg("-q")
        .arg("-c")
        .arg("tmux -CC attach")
        .arg("/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let reader = BufReader::new(stdout);

    for line_res in reader.lines() {
        let line = line_res?;
        if line.starts_with("%layout ") {
            if let Some(rest) = line.strip_prefix("%layout ") {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let layout_str = parts[1];
                    if let Some(tree) = super::parser::parse_layout(layout_str) {
                        if let Err(e) = super::shm::write_layout(&tree) {
                            eprintln!("[tmux daemon] Failed to write layout: {}", e);
                        }
                    }
                }
            }
        }
    }

    let _ = child.wait()?;
    Ok(())
}
