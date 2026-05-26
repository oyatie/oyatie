#!/usr/bin/env bash
# Scripted creation of the Talos 3+2 cluster VMs on Parallels Desktop 26 (Apple Silicon).
# Zero-GUI: prlctl creates/configures/boots every VM headlessly with nested virt on.
# After this, run bootstrap.sh with the printed CP_IPS / WORKER_IPS.
#
# Sizing (see README §Topology): CP ×3 = 4 vCPU / 8 GB / 64 GB; worker ×2 = 6 vCPU / 24 GB / 120 GB.
# Boot order "hdd0 cdrom1": empty disk falls through to the ISO (maintenance) on first boot; after
# Talos installs to disk, the populated disk takes precedence — no boot-order flip needed.
# Nodes get DHCP leases on the Parallels Shared net (10.211.55.0/24); the control-plane VIP is a
# high static IP (10.211.55.240) that won't collide with sequential low leases.
set -euo pipefail
ISO="$HOME/talos-mac/talos-kata-arm64.iso"
LEASES="/Library/Preferences/Parallels/parallels_dhcp_leases"
NODES=(talos-cp-1 talos-cp-2 talos-cp-3 talos-w-1 talos-w-2)
# name -> "cpus memMB diskMB"
spec() { case "$1" in
  talos-cp-*) echo "4 8192 65536";;
  talos-w-*)  echo "6 24576 122880";;
esac; }

[ -f "$ISO" ] || { echo "ISO not found: $ISO (run the Image Factory download first)"; exit 1; }

for name in "${NODES[@]}"; do
  if prlctl list --all --output name --no-header 2>/dev/null | grep -qx "$name"; then
    echo "  $name already exists — skipping create"; continue
  fi
  read -r cpus mem disk <<< "$(spec "$name")"
  echo "  creating $name ($cpus vCPU / $mem MB / $disk MB)"
  prlctl create "$name" -o linux >/dev/null
  prlctl set "$name" --cpus "$cpus" --memsize "$mem"            >/dev/null
  prlctl set "$name" --device-set hdd0 --size "$disk"           >/dev/null
  prlctl set "$name" --nested-virt on                           >/dev/null   # Kata cloud-hypervisor needs nested virt
  prlctl set "$name" --device-add cdrom --image "$ISO" --connect>/dev/null
  prlctl set "$name" --device-set net0 --type shared            >/dev/null
  prlctl set "$name" --device-bootorder "hdd0 cdrom1"           >/dev/null
  prlctl start "$name"                                          >/dev/null
done

echo "waiting for DHCP leases (Talos maintenance boot)..."
sleep 30
declare -a CP_IPS=() WORKER_IPS=()
for name in "${NODES[@]}"; do
  mac=$(prlctl list -i "$name" 2>/dev/null | grep -iE '^\s*net0' | grep -oiE 'mac=[0-9A-Fa-f]+' | head -1 | cut -d= -f2)
  ip=""
  for _ in $(seq 1 12); do
    ip=$(grep -i "$mac" "$LEASES" 2>/dev/null | tail -1 | cut -d= -f1)
    [ -n "$ip" ] && break; sleep 5
  done
  echo "  $name  mac=$mac  ip=${ip:-<none yet>}"
  case "$name" in talos-cp-*) CP_IPS+=("$ip");; talos-w-*) WORKER_IPS+=("$ip");; esac
done

echo
echo "=== run the bootstrap with these (VIP is static, outside the lease range): ==="
echo "export CP_IPS=\"${CP_IPS[*]}\" WORKER_IPS=\"${WORKER_IPS[*]}\" VIP=10.211.55.240"
echo "bash $(dirname "$0")/bootstrap.sh"
