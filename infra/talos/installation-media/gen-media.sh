#!/usr/bin/env bash
# gen-media.sh <control-plane|node> — build a bootable Talos installation-media image for bare-metal auto-install.
#
#   control-plane : config BAKED into the ISO (offline). Boot the control plane machine(s) -> auto-install ->
#          the CAPI management cluster forms itself. Needs CONTROLPLANE_ENDPOINT.
#   node : GENERIC image (Kata baked, no secrets). Boots with talos.config=$CONFIG_URL and
#          fetches its role/cluster config from the control plane. One image for ALL bare-metal nodes.
#
# Cloud spokes (OCI/AWS) are NOT built here — CAPI provisions them with platform images.
#
# Uses the Talos `imager` container (no Image Factory round-trip; bakes extensions + config).
# Output ISO -> write to install media (USB stick):  sudo dd if=_out/<preset>-metal-amd64.iso of=/dev/sdX bs=4M oflag=sync status=progress
set -euo pipefail

PRESET="${1:?usage: gen-media.sh <control-plane|node>}"
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/_out"; SECRETS="$HERE/secrets"
mkdir -p "$OUT" "$SECRETS"; chmod 700 "$SECRETS"

TALOS_VERSION="${TALOS_VERSION:-v1.13.3}"
K8S_VERSION="${K8S_VERSION:-1.36.1}"
ARCH="${ARCH:-amd64}"
INSTALL_DISK="${INSTALL_DISK:-/dev/sda}"
INSTALL_IMAGE="${INSTALL_IMAGE:-ghcr.io/siderolabs/installer:${TALOS_VERSION}}"
IMAGER="ghcr.io/siderolabs/imager:${TALOS_VERSION}"

# imager needs /dev + privileged to build images; outputs to the mounted /out.
imager() { docker run --rm -t --privileged -v /dev:/dev -v "$OUT:/out" -v "$SECRETS:/secrets:ro" "$IMAGER" "$@"; }

case "$PRESET" in
  control-plane)
    : "${CONTROLPLANE_ENDPOINT:?set CONTROLPLANE_ENDPOINT=https://<control-plane-ip-or-vip>:6443}"
    CLUSTER="${CLUSTER:-control-plane}"
    # Stable cluster secrets (reused across re-gen so the baked PKI is consistent).
    [ -f "$SECRETS/secrets.yaml" ] || talosctl gen secrets -o "$SECRETS/secrets.yaml"
    talosctl gen config "$CLUSTER" "$CONTROLPLANE_ENDPOINT" \
      --with-secrets "$SECRETS/secrets.yaml" \
      --kubernetes-version "$K8S_VERSION" \
      --talos-version "$TALOS_VERSION" \
      --install-disk "$INSTALL_DISK" \
      --install-image "$INSTALL_IMAGE" \
      --output-types controlplane \
      --output "$SECRETS/control-plane-config.yaml" \
      --config-patch "@$HERE/patches/control-plane.yaml" \
      --force
    echo ">> building control-plane ISO (vanilla + embedded config)"
    imager iso --arch "$ARCH" --embedded-config-path /secrets/control-plane-config.yaml
    mv -f "$OUT/metal-${ARCH}.iso" "$OUT/control-plane-metal-${ARCH}.iso" 2>/dev/null || true
    ;;
  node)
    : "${CONFIG_URL:?set CONFIG_URL=https://join.oyatie.dev/config (control-plane config endpoint)}"
    # No secrets in the node image (config is FETCHED), so build via Image Factory — it
    # resolves the Kata extension version for the Talos release automatically. The schematic
    # bakes the Kata extension + a talos.config kernel arg pointing at the control plane config endpoint.
    echo ">> requesting Image Factory schematic (kata + talos.config=$CONFIG_URL)"
    SCHEMATIC_DOC=$(printf 'customization:\n  systemExtensions:\n    officialExtensions:\n      - siderolabs/kata-containers\n  extraKernelArgs:\n    - talos.config=%s\n' "$CONFIG_URL")
    SCHEMATIC_ID=$(curl -fsSL -X POST --data-binary "$SCHEMATIC_DOC" https://factory.talos.dev/schematics \
                   | sed -E 's/.*"id":"([a-f0-9]+)".*/\1/')
    [ -n "$SCHEMATIC_ID" ] || { echo "schematic POST failed" >&2; exit 1; }
    echo "   schematic id: $SCHEMATIC_ID"
    echo ">> downloading node ISO from Image Factory"
    curl -fSL -o "$OUT/node-metal-${ARCH}.iso" \
      "https://factory.talos.dev/image/${SCHEMATIC_ID}/${TALOS_VERSION}/metal-${ARCH}.iso"
    ;;
  *) echo "unknown preset: $PRESET (want control-plane|node)" >&2; exit 1 ;;
esac

echo
echo "DONE. Image(s) in $OUT:"; ls -lh "$OUT"/*.iso 2>/dev/null | awk '{print "  "$5,$NF}'
echo "Write to install media (USB stick):  sudo dd if=$OUT/${PRESET}-metal-${ARCH}.iso of=/dev/sdX bs=4M oflag=sync status=progress"
