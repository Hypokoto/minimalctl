#!/usr/bin/env bash
# ==============================================================================
# Minimal OS — Thermals, Fan Curve, and ZRAM Memory Optimization Script
# Purpose:
# 1. Consolidate HP Victus fan control into victus-fand with an aggressive curve
#    (CPU On: 70°C, Off: 55°C) and disable conflicting victus-fan-curve.service.
# 2. Expand ZRAM to 100% of RAM with zstd compression to leverage CPU compute.
# 3. Optimize Linux VM sysctl parameters (swappiness=180, page-cluster=0).
# 4. Tune AMD CPU energy performance preference to balance_performance.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Ensure root execution
if [[ $EUID -ne 0 ]]; then
    echo "[*] Elevating to root privileges..."
    exec sudo "$0" "$@"
fi

echo "================================================================="
echo "  Minimal Thermals, Fan Curve & ZRAM Optimization Pipeline"
echo "================================================================="

# --- 1. HP Victus Fan Curve Consolidation ---
if [[ -d /etc/victus-fan || -f /usr/lib/victus-fan/victus-fand ]]; then
    mkdir -p /etc/victus-fan
    if [[ -f "$DOTFILES_DIR/security/victus-fan-config.json" ]]; then
        install -m 644 "$DOTFILES_DIR/security/victus-fan-config.json" /etc/victus-fan/config.json
    else
        cat <<'EOF' > /etc/victus-fan/config.json
{
  "enabled": true,
  "poll_interval_sec": 2,
  "low_battery_pct": 20,
  "override": "policy",
  "profile_map": {
    "performance": "performance",
    "balanced": "balanced",
    "quiet": "power-saver",
    "cool": "power-saver",
    "low-power": "power-saver"
  },
  "profiles": {
    "performance": {
      "always_max": true
    },
    "balanced": {
      "cpu_on": 70,
      "cpu_off": 55,
      "igpu_on": 65,
      "igpu_off": 55,
      "dgpu_on": 68,
      "dgpu_off": 55
    },
    "power-saver": {
      "cpu_on": 78,
      "cpu_off": 65,
      "igpu_on": 72,
      "igpu_off": 60,
      "dgpu_on": 75,
      "dgpu_off": 65
    }
  }
}
EOF
    fi
    echo "  [OK] Deployed aggressive fan curve config (CPU on: 70°C, off: 55°C)."
fi

# Stop and disable conflicting two-state curve service if active
if systemctl is-active --quiet victus-fan-curve.service 2>/dev/null || systemctl is-enabled --quiet victus-fan-curve.service 2>/dev/null; then
    echo "  [*] Disabling conflicting victus-fan-curve.service..."
    systemctl stop victus-fan-curve.service || true
    systemctl disable victus-fan-curve.service || true
    echo "  [OK] Disabled victus-fan-curve.service (eliminated controller conflict)."
fi

# Restart victus-fan daemon with the new curve
if systemctl is-enabled --quiet victus-fan.service 2>/dev/null || [[ -f /usr/lib/victus-fan/victus-fand ]]; then
    echo "  [*] Restarting victus-fan.service..."
    systemctl restart victus-fan.service || true
    echo "  [OK] victus-fan.service active with CPU-gated curve."
fi

# --- 2. ZRAM Memory Generator Configuration ---
echo "[2/4] Configuring ZRAM Memory Compression (100% of RAM)..."
if [[ -f "$DOTFILES_DIR/security/zram-generator.conf" ]]; then
    mkdir -p /etc/systemd
    install -m 644 "$DOTFILES_DIR/security/zram-generator.conf" /etc/systemd/zram-generator.conf
    echo "  [OK] Deployed /etc/systemd/zram-generator.conf (zram-size = ram, zstd)."
    
    echo "  [*] Regenerating and restarting ZRAM device..."
    systemctl daemon-reload
    systemctl restart "systemd-zram-setup@zram0.service" || echo "  [-] Note: ZRAM service will apply on next restart or swap re-init."
fi

# --- 3. Virtual Memory & Sysctl Tuning ---
echo "[3/4] Applying VM Sysctl Parameters..."
if [[ -f "$DOTFILES_DIR/security/99-security-hardening.conf" ]]; then
    mkdir -p /etc/sysctl.d
    install -m 644 "$DOTFILES_DIR/security/99-security-hardening.conf" /etc/sysctl.d/99-security-hardening.conf
fi

# Apply in-memory sysctl values immediately
sysctl -w vm.swappiness=180 >/dev/null 2>&1 || true
sysctl -w vm.watermark_boost_factor=0 >/dev/null 2>&1 || true
sysctl -w vm.watermark_scale_factor=125 >/dev/null 2>&1 || true
sysctl -w vm.page-cluster=0 >/dev/null 2>&1 || true
echo "  [OK] Applied vm.swappiness=180, vm.page-cluster=0, watermark parameters."

# --- 4. AMD CPU Energy Performance Preference ---
echo "[4/4] Setting AMD CPU Energy Performance Preference..."
for epp in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do
    if [[ -w "$epp" ]]; then
        echo balance_performance > "$epp" 2>/dev/null || true
    fi
done

if command -v powerprofilesctl >/dev/null 2>&1; then
    powerprofilesctl set balanced >/dev/null 2>&1 || true
    echo "  [OK] Set power-profiles-daemon to balanced (EPP: balance_performance)."
fi

echo "================================================================="
echo "  Verification & System Status"
echo "================================================================="
echo "- CPU Governor EPP: $(cat /sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference 2>/dev/null || echo 'N/A')"
echo "- vm.swappiness:    $(cat /proc/sys/vm/swappiness 2>/dev/null || echo 'N/A')"
echo "- vm.page-cluster:  $(cat /proc/sys/vm/page-cluster 2>/dev/null || echo 'N/A')"
echo "- ZRAM Status:"
if command -v zramctl >/dev/null 2>&1; then
    zramctl || true
fi
echo "- Memory & Swap:"
free -h
echo "================================================================="
echo "  Optimization complete!"
echo "================================================================="
