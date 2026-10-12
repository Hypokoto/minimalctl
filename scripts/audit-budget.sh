#!/usr/bin/env bash
set -euo pipefail

echo "=== Process Budget Audit ==="
ALLOWLIST="${1:-security/policy/process-allowlist.txt}"

if [ ! -f "$ALLOWLIST" ]; then
	echo "ERROR: Allowlist not found."
	exit 1
fi

VIOLATION=0

# Define explicitly banned processes to quickly flag known regressions
BANNED_PROCS=("conky" "nm-applet" "nwg-drawer" "hyprlauncher")

for proc in "${BANNED_PROCS[@]}"; do
	if pgrep -x "$proc" >/dev/null; then
		echo "[!] REGRESSION DETECTED: Banned process '$proc' is running."
		VIOLATION=1
	fi
done

# Read allowlist
declare -A AUTH_PROCS
while IFS=':' read -r proc class; do
	[[ -z "$proc" || "$proc" == "#"* ]] && continue
	AUTH_PROCS["$proc"]="$class"
done <"$ALLOWLIST"

echo ""
echo "[*] Authorized Process Status:"
for proc in "${!AUTH_PROCS[@]}"; do
	count=$(pgrep -f "^[^ ]*$proc" | wc -l || true)
	class="${AUTH_PROCS[$proc]}"

	if [ "$count" -gt 0 ]; then
		echo " - $proc ($class): Running ($count)"
	else
		echo " - $proc ($class): Not running"
	fi
done

echo ""
echo "[*] Uncategorized Processes Check:"
UNCATEGORIZED_FOUND=0
if pgrep -x "dunst" >/dev/null; then
	echo " - VIOLATION: dunst (duplicate notification daemon)"
	VIOLATION=1
	UNCATEGORIZED_FOUND=1
fi
if pgrep -x "swaybg" >/dev/null; then
	echo " - VIOLATION: swaybg (duplicate wallpaper daemon)"
	VIOLATION=1
	UNCATEGORIZED_FOUND=1
fi

if [ $UNCATEGORIZED_FOUND -eq 0 ]; then
	echo "  none"
fi

if [ $VIOLATION -eq 1 ]; then
	echo "Process budget violated!"
	exit 1
fi

echo "Process budget intact."
exit 0
