#!/usr/bin/env bash
# Minimal OS Shell — Runtime Verification
# Measures the idle CPU, RAM, and process counts to prove the Process Budget.

echo "=== Runtime Verification ==="
echo ""
echo "[*] Total Memory Usage:"
free -h | awk 'NR==1 || NR==2'

echo ""
echo "[*] Persistent User Processes (excluding kernel/system):"
USER_PROCS=$(ps -U "$USER" -u "$USER" --no-headers | wc -l)
echo "Total processes for $USER: $USER_PROCS"

echo ""
echo "[*] Authorized Persistent Desktop Processes running:"
for proc in waybar mako hypridle awww-daemon polkit-gnome wl-paste swayosd-server; do
	count=$(pgrep -x "$proc" | wc -l)
	if [ "$count" -gt 0 ]; then
		echo " - $proc: Running ($count)"
	else
		echo " - $proc: Not running"
	fi
done

echo ""
echo "[*] Banned/Duplicate Processes Check:"
for proc in conky nm-applet nwg-drawer hyprlauncher; do
	if pgrep -x "$proc" >/dev/null; then
		echo " [!] VIOLATION: $proc is running!"
	else
		echo " - $proc: Clean"
	fi
done

echo ""
echo "[*] Battery Monitor (Event-Driven) CPU Usage:"
BAT_PID=$(pgrep -f "battery-monitor.sh" | head -n1 || true)
if [[ -n "${BAT_PID:-}" ]]; then
	ps -p "$BAT_PID" -o %cpu,%mem,cmd --no-headers
else
	echo " - battery-monitor.service is not currently running."
fi

echo ""
echo "=== End Verification ==="
