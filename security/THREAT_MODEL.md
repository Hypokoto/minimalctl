# Threat Model: Minimal OS Shell

This document defines the realistic threat model for this Labwc-based desktop environment. It explicitly defines what threats the architecture attempts to mitigate and what threats are considered out-of-scope.

## In-Scope Threats (Actively Mitigated)
1. **Accidental Background Bloat**: A major risk to desktop performance and security is the slow accumulation of persistent polling daemons.
   - *Mitigation*: Strict allowlist (`process-allowlist.txt`), native C event-driven daemons (`mincore`), and `minimalctl audit` checks.
2. **Predictable State/Temp File Attacks**: Scripts using predictable paths (e.g., `/tmp/file.$$`) allow local attackers to overwrite critical data via symlinks.
   - *Mitigation*: Native `mktemp` usage, and `XDG_RUNTIME_DIR` for user-specific volatile state.
3. **UI-driven Command Injection**: Untrusted or malformed strings returned by UI elements (like fuzzel) executing arbitrary shell commands.
   - *Mitigation*: Strict regex validation on inputs before passing them to execution contexts (e.g., clipboard ID regex validation).
4. **Supply Chain Alteration**: Third-party package repositories silently becoming defaults and pushing malicious updates.
   - *Mitigation*: Official Arch repositories and the AUR are the only default sources. External repos require explicit opt-in. AUR updates are gated.
5. **Unnoticed Security Degradation**: Background processes crashing or failing silently.
   - *Mitigation*: `minimalctl status` and `minimalctl audit` to proactively verify the dependency graph and running daemons.
6. **Clipboard Persistence**: Sensitive data (passwords, tokens) remaining on disk across reboots.
   - *Mitigation*: `minclip` is a zero-I/O in-memory ring buffer. Screen locks trigger a `SIGUSR1` privacy wipe to clear the ring instantly.
7. **AI Sandbox & LLM Execution**: Autonomous LLM agents running code with broad user permissions.
   - *Mitigation*: Ephemeral `bwrap` sandboxing explicitly blacklisting sensitive host directories (SSH, GnuPG) while providing isolated toolchains. Local registry prevents arbitrary global MCP pollution.

## Out-of-Scope Threats (Not Addressed by the Shell)
1. **Kernel-Level Exploits & Zero-Days**: This shell architecture relies on the underlying OS (Arch Linux) for kernel security.
2. **Browser Exploits & Sandbox Escapes**: If the browser is compromised, this desktop environment does not contain mandatory access controls (like SELinux/AppArmor) to sandbox the browser process itself.
3. **Physical Access Attacks**: Rely on LUKS Full Disk Encryption.
4. **Malicious AUR Packages**: While the supply chain is bounded, the user is still responsible for auditing AUR packages before installation.
5. **User-Service Trust Boundaries**: Systemd user services (`minimal-*.service`) share the same privilege level. A compromised user service can compromise other user services (IPC spoofing, Wayland snooping).
