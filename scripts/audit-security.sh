#!/usr/bin/env bash
set -euo pipefail

echo "=== Security Invariants Audit ==="
VIOLATION=0

# Safely iterate over shell scripts using NUL-delimited find to prevent word-splitting
while IFS= read -r -d '' script; do
	# Skip this audit script itself so it doesn't match its own regex strings
	if [[ "$script" == *"audit-security.sh"* ]]; then continue; fi

	if grep -n -E "eval\s+" "$script" | grep -v "Ignore/Safe" >/dev/null; then
		echo "[!] Unsafe 'eval' found in $script"
		VIOLATION=1
	fi
	if grep -n -E "(curl|wget).*\|\s*bash" "$script" >/dev/null; then
		echo "[!] Remote execution pattern (curl | bash) found in $script"
		VIOLATION=1
	fi
	if grep -n -E "/tmp/.*\.\\$\\$" "$script" >/dev/null; then
		echo "[!] Predictable temp file with PID (\$\$) found in $script"
		VIOLATION=1
	fi
done < <(
	find . \
		-type d \( -name .agents -o -name .git -o -name plugins -o -name target \) -prune \
		-o -type f \( -name '*.sh' -o -name '*.bash' -o -name '*.zsh' \) \
		-print0
)

# Check generated mako configuration syntax
if [ -f "mako/config" ]; then
	if grep -E "padding\s*=\s*[0-9]+\s+[0-9]+" mako/config >/dev/null; then
		echo "[!] Invalid space-separated padding in mako/config (must be comma-separated)"
		VIOLATION=1
	fi
fi

if [ $VIOLATION -eq 1 ]; then
	echo "Security audit failed!"
	exit 1
fi

echo "Security patterns passed."
exit 0
