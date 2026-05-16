#!/usr/bin/env bash
# install-from-tarball.sh — run on a deploy target. Detects the host's CPU
# class, picks the matching tarball from the current directory, verifies the
# sha256, and installs the binary to /usr/local/bin.
#
# Override autodetection with: OYATIE_LABEL=linux-x86_64-v3 ~/install-from-tarball.sh
set -euo pipefail

BIN=oya-ops-workspace-shell

autodetect_label() {
  local arch os
  arch=$(uname -m)
  os=$(uname -s)
  case "$os/$arch" in
    Linux/x86_64)
      # AMD family 25 = Zen 3/4 (Ryzen 5000/7000) -> znver4 build.
      # Anything else x86_64 -> znver1 build (Oracle E2.1.Micro = Naples Zen 1;
      # also a safe fallback for any AVX2-capable CPU).
      local vendor family
      vendor=$(awk -F: '/^vendor_id/ {gsub(/ /,"",$2); print $2; exit}' /proc/cpuinfo)
      family=$(awk -F: '/^cpu family/ {gsub(/ /,"",$2); print $2; exit}' /proc/cpuinfo)
      if [ "$vendor" = "AuthenticAMD" ] && [ "${family:-0}" -ge 25 ]; then
        echo linux-x86_64-znver4
      else
        echo linux-x86_64-znver1
      fi
      ;;
    Linux/aarch64)
      echo linux-aarch64-neov1
      ;;
    Darwin/arm64)
      # Default to the m5 variant; override with OYATIE_LABEL if you built m4 instead.
      echo darwin-aarch64-apple-m5
      ;;
    *)
      echo "unsupported host: $os/$arch" >&2
      exit 1
      ;;
  esac
}

LABEL=${OYATIE_LABEL:-$(autodetect_label)}
echo "==> selected variant: $LABEL"

# Find the most recent matching tarball in the current dir.
TARBALL=$(ls -t ${BIN}-*-${LABEL}.tar.gz 2>/dev/null | head -1 || true)
if [ -z "$TARBALL" ]; then
  echo "ERROR: no tarball matching ${BIN}-*-${LABEL}.tar.gz found in $(pwd)" >&2
  echo "available:" >&2
  ls ${BIN}-*.tar.gz 2>/dev/null >&2 || echo "  (none)" >&2
  exit 1
fi
echo "==> using $TARBALL"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

tar xzf "$TARBALL" -C "$TMP"

echo "==> manifest:"
cat "$TMP/manifest.txt" | sed 's/^/    /'

echo "==> verifying sha256"
( cd "$TMP" && sha256sum -c "$BIN.sha256" )

sudo install -o root -g root -m 0755 "$TMP/$BIN" "/usr/local/bin/$BIN"
echo "==> installed: $(ls -l /usr/local/bin/$BIN)"

# If the systemd unit exists, restart it so the new binary takes over.
if systemctl list-unit-files oyatie.service >/dev/null 2>&1; then
  echo "==> restarting oyatie.service"
  sudo systemctl restart oyatie.service
  systemctl --no-pager status oyatie.service | head -10
else
  echo "(oyatie.service not present — run setup-oyatie-service.sh on this host first if this is a fresh deploy)"
fi
