pub fn audit_command(cmd: &str) -> Result<(), &'static str> {
    // List of highly destructive or suspicious signatures
    let blacklist = [
        "rm -rf /",
        "rm -rf /*",
        "curl | bash",
        "curl | sh",
        "wget -qO-",
        "mkfs",
        "dd if=/dev/zero",
        "> /dev/sda",
    ];

    let cmd_lower = cmd.to_lowercase();

    // Naive check
    for bad in blacklist.iter() {
        if cmd_lower.contains(bad) {
            return Err("Blocked by minimalctl command firewall: Malicious pattern detected.");
        }
    }

    Ok(())
}
