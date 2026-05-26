#!/usr/bin/env bash
# mkmedia.sh — write the preset Talos/Omni installer image to a USB stick.
#
#   curl -fsSL https://raw.githubusercontent.com/<org>/source/dev/infra/talos-baremetal/mkmedia.sh | bash -s -- \
#        --image https://<your-omni>/image/<id>/metal-amd64.iso --device /dev/disk4
#
# WHAT IT DOES: downloads (if a URL) the amd64 installer ISO you generated in Omni — already baked with
# the SideroLink join token + your chosen extensions (kata-containers) — and dd's it to a USB, with hard
# guards against writing to an internal disk. The helper then boots this stick once; the node phones home
# to Omni over WireGuard (outbound, NAT-friendly) and you finish everything remotely from Omni.
#
# This script does NOT bake config itself — Omni owns the join token, schematic, extensions, and (later)
# disk encryption + KubeSpan via the cluster template. Generating the image in Omni is what makes it preset.
set -euo pipefail

IMAGE="" DEVICE="" ASSUME_YES=0
while [ $# -gt 0 ]; do
  case "$1" in
    --image)  IMAGE="$2"; shift 2;;
    --device) DEVICE="$2"; shift 2;;
    --yes)    ASSUME_YES=1; shift;;
    *) echo "unknown arg: $1" >&2; exit 2;;
  esac
done
: "${IMAGE:?--image <path-or-https-url to the Omni amd64 ISO> is required}"
: "${DEVICE:?--device <target USB, e.g. /dev/disk4 (macOS) or /dev/sdb (linux)> is required}"

OS="$(uname -s)"
die() { echo "ERROR: $*" >&2; exit 1; }
confirm() {
  [ "$ASSUME_YES" = 1 ] && return 0
  printf '\nType the device name (%s) to confirm ERASE+WRITE: ' "$DEVICE"
  read -r ans; [ "$ans" = "$DEVICE" ] || die "confirmation mismatch — aborting"
}

# --- resolve the image to a local file ---
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
if printf '%s' "$IMAGE" | grep -qiE '^https?://'; then
  echo "==> downloading image"
  curl -fL --progress-bar "$IMAGE" -o "$WORK/installer.iso"
  IMG="$WORK/installer.iso"
else
  [ -f "$IMAGE" ] || die "image file not found: $IMAGE"
  IMG="$IMAGE"
fi
echo "    image: $IMG ($(du -h "$IMG" | cut -f1))"

# --- guard the target device (refuse internal disks) ---
case "$OS" in
  Darwin)
    diskutil info "$DEVICE" >/dev/null 2>&1 || die "no such disk: $DEVICE (see: diskutil list external)"
    internal="$(diskutil info "$DEVICE" | awk -F: '/Internal/{gsub(/ /,"",$2);print $2}')"
    [ "$internal" = "Yes" ] && die "$DEVICE is an INTERNAL disk — refusing to write"
    echo "==> target:"; diskutil info "$DEVICE" | grep -E 'Device / Media Name|Disk Size|Removable|Internal' || true
    confirm
    echo "==> writing (sudo)"
    diskutil unmountDisk "$DEVICE"
    RAW="${DEVICE/disk/rdisk}"   # raw node = much faster
    sudo dd if="$IMG" of="$RAW" bs=4m
    sync; diskutil eject "$DEVICE" || true
    ;;
  Linux)
    base="$(basename "$DEVICE")"
    [ -b "$DEVICE" ] || die "no such block device: $DEVICE"
    rm_flag="$(cat "/sys/block/$base/removable" 2>/dev/null || echo 0)"
    [ "$rm_flag" = "1" ] || die "$DEVICE is not removable (removable=$rm_flag) — refusing to write"
    echo "==> target:"; lsblk -d -o NAME,RM,SIZE,MODEL "$DEVICE" || true
    confirm
    echo "==> writing (sudo)"
    sudo dd if="$IMG" of="$DEVICE" bs=4M oflag=sync status=progress
    sync
    ;;
  *) die "unsupported OS: $OS (run on macOS or Linux)";;
esac

cat <<EOF

DONE — USB ready.
Hand it to the on-site helper with these instructions:
  1. Plug the USB into the 7800X3D.
  2. Power on; tap the boot-menu key (ASRock/most AM5: F11) and pick the USB.
  3. Wait ~5 min, then remove the USB and reboot once more. That's it.
The node will appear in your Omni UI within a few minutes (it dials out — no inbound ports).
Finish from Omni: assign it as control-plane via the cluster template, Omni bootstraps etcd remotely.
EOF
