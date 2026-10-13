#!/usr/bin/env bash
# ==============================================================================
# Minimal System Hardening Pipeline
# Mitigates CVE-2026-31431 (Copy Fail) and CVE-2026-53362 (IPv6 Kernel)
# ==============================================================================
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

INTERACTIVE=0
for arg in "$@"; do
    if [[ "$arg" == "-i" || "$arg" == "--interactive" ]]; then
        INTERACTIVE=1
    fi
done

DO_UPDATE=1
DO_SYSCTL=1
DO_NFTABLES=1
DO_APPARMOR=1

if [[ "$INTERACTIVE" -eq 1 ]]; then
    echo "=== Interactive Hardening Setup ==="
    read -r -p "[?] Update system packages (pacman -Syu) and check kernel? [Y/n]: " ans
    if [[ "${ans,,}" == "n"* ]]; then DO_UPDATE=0; fi

    read -r -p "[?] Deploy sysctl rules & module blacklists? [Y/n]: " ans
    if [[ "${ans,,}" == "n"* ]]; then DO_SYSCTL=0; fi

    read -r -p "[?] Deploy nftables firewall rules? [Y/n]: " ans
    if [[ "${ans,,}" == "n"* ]]; then DO_NFTABLES=0; fi

    read -r -p "[?] Deploy AppArmor MAC & bootloader injection? [Y/n]: " ans
    if [[ "${ans,,}" == "n"* ]]; then DO_APPARMOR=0; fi
    echo ""
fi

echo "=== System Hardening ==="

# 1. System Package Updates (Kernel patching)
if [[ "$DO_UPDATE" -eq 1 ]]; then
    echo "[*] Updating system packages to patch recent vulnerabilities (Copy Fail / IPv6)..."
    if command -v pacman >/dev/null 2>&1; then
        sudo pacman -Syu --noconfirm || echo "[-] WARN: System update failed. Continuing with local hardening..."
    else
        echo "[-] WARN: Arch Linux package manager not found. Skipping pacman updates."
    fi

    # Kernel Check
    if command -v pacman >/dev/null 2>&1; then
        # Extract just the X.Y.Z version parts
        RUNNING_KERNEL=$(uname -r | sed -E 's/^([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
        INSTALLED_KERNEL=$(pacman -Q linux | awk '{print $2}' | sed -E 's/^([0-9]+\.[0-9]+\.[0-9]+).*/\1/')

        # Check if a reboot is pending for a kernel update
        if [[ "$RUNNING_KERNEL" != "$INSTALLED_KERNEL" ]]; then
            echo "[!] WARNING: REBOOT REQUIRED! Running kernel ($RUNNING_KERNEL) differs from installed ($INSTALLED_KERNEL)"
        fi

        # Check for Copy Fail (CVE-2026-31431) mitigation in running kernel
        # Fixed in: 6.18.22, 6.19.12, 7.0.0
        ver_gte() {
            [ "$1" = "$2" ] && return 0
            [ "$1" = "$(printf '%s\n%s' "$1" "$2" | sort -V | tail -n1)" ]
        }

        IS_SAFE=0
        if [[ "$RUNNING_KERNEL" == 6.18.* ]] && ver_gte "$RUNNING_KERNEL" "6.18.22"; then IS_SAFE=1; fi
        if [[ "$RUNNING_KERNEL" == 6.19.* ]] && ver_gte "$RUNNING_KERNEL" "6.19.12"; then IS_SAFE=1; fi
        if ver_gte "$RUNNING_KERNEL" "7.0.0"; then IS_SAFE=1; fi

        if [ "$IS_SAFE" -eq 0 ]; then
            echo "[!] CRITICAL: Running kernel ($RUNNING_KERNEL) is VULNERABLE to CVE-2026-31431 (Copy Fail)!"
            echo "    Ensure algif_aead is blacklisted or update kernel and reboot."
        else
            echo "[OK] Running kernel ($RUNNING_KERNEL) mitigates CVE-2026-31431 natively."
        fi

        # Check for IPv6 Frag Escape (CVE-2026-53362) mitigation
        # Fixed in: 6.1.176, 6.6.143, 6.12.94, 6.18.37, 7.1.2
        IS_IPV6_SAFE=0
        if [[ "$RUNNING_KERNEL" == 6.1.* ]] && ver_gte "$RUNNING_KERNEL" "6.1.176"; then IS_IPV6_SAFE=1; fi
        if [[ "$RUNNING_KERNEL" == 6.6.* ]] && ver_gte "$RUNNING_KERNEL" "6.6.143"; then IS_IPV6_SAFE=1; fi
        if [[ "$RUNNING_KERNEL" == 6.12.* ]] && ver_gte "$RUNNING_KERNEL" "6.12.94"; then IS_IPV6_SAFE=1; fi
        if [[ "$RUNNING_KERNEL" == 6.18.* ]] && ver_gte "$RUNNING_KERNEL" "6.18.37"; then IS_IPV6_SAFE=1; fi
        if ver_gte "$RUNNING_KERNEL" "7.1.2"; then IS_IPV6_SAFE=1; fi

        if [ "$IS_IPV6_SAFE" -eq 0 ]; then
            echo "[!] CRITICAL: Running kernel ($RUNNING_KERNEL) is VULNERABLE to CVE-2026-53362 (IPv6 Frag Escape)!"
            echo "    Ensure ipv6.disable=1 is set on the kernel command line, or update kernel and reboot."
        else
            echo "[OK] Running kernel ($RUNNING_KERNEL) mitigates CVE-2026-53362 natively."
        fi
    fi
fi

# 2. Sysctl & Kernel Modules Deployment
if [[ "$DO_SYSCTL" -eq 1 ]]; then
    echo "[*] Deploying security hardening rules..."
    if [[ -d /etc/sysctl.d ]]; then
        sudo install -m 644 -o root -g root "$DIR/99-security-hardening.conf" /etc/sysctl.d/
        sudo sysctl --system
    else
        echo "[-] WARN: /etc/sysctl.d/ not found. Are you on a supported Linux?"
    fi

    if [[ -d /etc/modprobe.d ]]; then
        sudo install -m 644 -o root -g root "$DIR/99-security-blacklist.conf" /etc/modprobe.d/
    fi
fi

# 3. Nftables deployment
if [[ "$DO_NFTABLES" -eq 1 ]]; then
    if [[ -d /etc/nftables.d || -d /etc/nftables ]]; then
        if [[ -f "/etc/nftables.conf" ]]; then
            echo "[*] Deploying nftables firewall..."
            sudo install -m 644 -o root -g root "$DIR/nftables.conf" /etc/
            sudo systemctl enable --now nftables.service || true
        fi
    fi
fi

# 4. AppArmor MAC Deployment
if [[ "$DO_APPARMOR" -eq 1 ]]; then
    echo "[*] Deploying AppArmor Mandatory Access Control..."
    if command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --needed --noconfirm apparmor || true

        # Enable apparmor service
        sudo systemctl enable apparmor.service || true

        # Inject kernel parameters for Unified Kernel Images (UKI)
        if [[ -f /etc/kernel/cmdline ]]; then
            echo "    Injecting AppArmor parameters into /etc/kernel/cmdline for UKI..."
            cmdline_modified=0
            if grep -q "lsm=" /etc/kernel/cmdline; then
                if ! grep -q -E "lsm=[^ ]*apparmor" /etc/kernel/cmdline; then
                    sudo sed -i -E 's/(lsm=[^ ]*)/\1,apparmor/' /etc/kernel/cmdline
                    cmdline_modified=1
                fi
            else
                sudo sed -i 's/$/ lsm=landlock,lockdown,yama,integrity,apparmor,bpf/' /etc/kernel/cmdline
                cmdline_modified=1
            fi
            if ! grep -q "apparmor=1" /etc/kernel/cmdline; then
                sudo sed -i 's/$/ apparmor=1 security=apparmor/' /etc/kernel/cmdline
                cmdline_modified=1
            fi
            if [[ "$cmdline_modified" -eq 1 ]]; then
                echo "      Updated /etc/kernel/cmdline. Rebuilding UKI..."
                sudo mkinitcpio -P || true
            fi
        fi

        # Fallback for standard systemd-boot entries
        if [[ -d /boot/loader/entries ]] && [[ "$(ls -A /boot/loader/entries/ 2>/dev/null)" ]]; then
            echo "    Injecting AppArmor parameters into systemd-boot entries..."
            for conf in /boot/loader/entries/*.conf; do
                if grep -q "^options " "$conf"; then
                    if grep -q "lsm=" "$conf"; then
                        if ! grep -q -E "lsm=[^ ]*apparmor" "$conf"; then
                            sudo sed -i -E 's/(lsm=[^ ]*)/\1,apparmor/' "$conf"
                            echo "      Updated LSM in: $conf"
                        fi
                    else
                        sudo sed -i 's/^options .*/& lsm=landlock,lockdown,yama,integrity,apparmor,bpf/' "$conf"
                    fi
                    if ! grep -q "apparmor=1" "$conf"; then
                        sudo sed -i 's/^options .*/& apparmor=1 security=apparmor/' "$conf"
                        echo "      Updated AppArmor flags in: $conf"
                    fi
                fi
            done
        fi

        # Deploy minimalctl / mincore profile
        if [[ -d /etc/apparmor.d ]]; then
            sudo install -m 644 -o root -g root "$DIR/apparmor.d/home.local.bin.mincore" /etc/apparmor.d/
            echo "    Deployed AppArmor profile for mincore."
            # Reload profile if apparmor is currently running
            if systemctl is-active --quiet apparmor.service; then
                sudo apparmor_parser -r /etc/apparmor.d/home.local.bin.mincore || true
            fi
        fi
    fi
fi

# 5. ZRAM Memory Generator Deployment
if [[ -f "$DIR/zram-generator.conf" ]]; then
    echo "[*] Deploying ZRAM generator configuration..."
    sudo install -m 644 -o root -g root "$DIR/zram-generator.conf" /etc/systemd/
    sudo systemctl daemon-reload
    sudo systemctl restart "systemd-zram-setup@zram0.service" || echo "[-] Note: ZRAM service will apply on next restart or swap re-init."
fi

echo "=== Hardening Complete ==="
