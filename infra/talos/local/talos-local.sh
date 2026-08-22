#!/usr/bin/env bash
# talos-local.sh — one-shot local Talos substrate on this Apple-Silicon Mac via
# vfkit + Apple Virtualization.framework (`vz` backend). Production-fidelity local
# Kubernetes (real VMs, real disks/NICs, nested virt for Kata) — the laptop mirror
# of the bare-metal/cloud fleet in infra/capi (ADR-0375).
#
# Subcommands:
#   check                 preflight ONLY (read-only): arch, macOS, chip, RAM,
#                         nested-virt, vfkit/talosctl/kubectl. Exit 0 = ready.
#   setup                 install missing host deps (vfkit, talosctl, kubectl) via brew. Idempotent.
#   up   [--role R] [--name N] [--cpus N] [--ram-gb N] [--disk-gb N] [--config-patch F]
#                         create + boot the VM(s), apply Talos config, (for cp/single)
#                         bootstrap etcd, fetch kubeconfig + talosconfig. Idempotent-ish.
#                         --config-patch F (optional) layers a YAML patch file onto the
#                         base config at apply-config time. Used for ADR-0381 D2 per-node
#                         cell labels (infra/talos/local/patches/cell-*.yaml). Wired for
#                         worker and control-plane roles.
#   up-multinode          one-command 7-VM Oyatie topology (ADR-0381 D2): 3 CP HA +
#                         2 worker (tenant) + 1 CI specialty + 1 storage specialty.
#                         Each VM gets the matching cell patch from ./patches/.
#                         See MULTINODE-RUNBOOK.md for host sizing assumptions.
#   down [--name N] [--all]   stop + delete the VM(s) and the local config bundle.
#   status                show VM (vfkit pids) + node state.
#
#   --role control-plane | worker | single   (default: single — 1 schedulable CP, dev box)
#
# Why vfkit-vz (not UTM, not QEMU/HVF, not docker): Apple Virtualization.framework
# exposes nested virtualization on M3+/macOS 15+, which the Kata runtime tier
# (ADR-0147/0338) needs. vfkit (github.com/crc-org/vfkit) is the single-command,
# headless CLI front-end to that same `vz` backend — same Apple-VM fidelity, but
# scriptable. (UTM's utmctl can only drive GUI-created VMs; a hand-authored bundle
# is never imported headless, so UTM cannot do GUI-less bring-up. vfkit can.)
#
# HONESTY: vfkit NAT has no API to report the guest IP, so we pin a FIXED MAC per
# node and read the macOS DHCP lease for it from `/var/db/dhcpd_leases`. bootpd
# writes that file log-structured, type-prefixed (`1,`), and with per-octet leading
# zeros STRIPPED (e.g. `0c`→`c`); the lookup below normalizes both sides and takes
# the last lease block for the MAC. If NAT lease discovery proves unreliable on a
# given host, install `socket_vmnet` (brew) for a bridged interface and pass its
# socket to vfkit (`--device virtio-net,unixSocketPath=…`) — see README. Plain NAT
# + lease lookup is preferred and is what `up` uses.

set -euo pipefail

# ── constants ────────────────────────────────────────────────────────────────
TALOS_VERSION="${TALOS_VERSION:-v1.13.3}"          # Talos OS version (arm64)
K8S_VERSION="${K8S_VERSION:-1.36.1}"               # Kubernetes version
CLUSTER="${CLUSTER:-local}"
# Image Factory schematic with the Kata Containers extension baked in (matches the
# fleet's kataInstallerImage). arm64 metal raw image for the vz backend.
SCHEMATIC_ID="${SCHEMATIC_ID:-3da7f440f279f4814fa73bdf83c84710a8e93c40a4a3cbba4d969f14afb96298}"
WORKDIR="${WORKDIR:-$HOME/.oya/talos-local}"        # gen configs, image, per-VM disks/efi/pid/log live here
MIN_MACOS_MAJOR=15                                  # nested virt floor
DHCPD_LEASES="${DHCPD_LEASES:-/var/db/dhcpd_leases}" # macOS bootpd NAT lease database
IP_WAIT_TRIES="${IP_WAIT_TRIES:-60}"                # DHCP-lease poll attempts (×5s ≈ 5 min)

VFKIT="$(command -v vfkit 2>/dev/null || echo /opt/homebrew/bin/vfkit)"

log()  { printf '\n\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '  \033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\n\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# ── check ────────────────────────────────────────────────────────────────────
cmd_check() {
  local fail=0
  log "Preflight — local Talos substrate readiness"

  local arch; arch="$(uname -m)"
  if [ "$arch" = "arm64" ]; then ok "arch: arm64 (Apple Silicon)"; else
    warn "arch: $arch — this script targets Apple Silicon (arm64) VMs"; fail=1; fi

  local macos; macos="$(sw_vers -productVersion 2>/dev/null || echo 0)"
  local major="${macos%%.*}"
  if [ "${major:-0}" -ge "$MIN_MACOS_MAJOR" ]; then ok "macOS: $macos (>= $MIN_MACOS_MAJOR — nested virt available)"; else
    warn "macOS: $macos — nested virt (Kata) needs macOS >= $MIN_MACOS_MAJOR (M3+). VMs still run; kata tier won't."; fi

  local chip; chip="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
  ok "chip: $chip"

  local ramgb=$(( $(sysctl -n hw.memsize 2>/dev/null || echo 0) / 1073741824 ))
  if [ "$ramgb" -ge 8 ]; then ok "RAM: ${ramgb} GB"; else
    warn "RAM: ${ramgb} GB — a 3-CP + worker layout wants >= 16 GB; use --role single on small hosts"; fi

  local freegb; freegb=$(df -g "$HOME" 2>/dev/null | awk 'NR==2{print $4}')
  if [ "${freegb:-0}" -ge 20 ]; then ok "disk free: ${freegb} GB"; else
    warn "disk free: ${freegb:-?} GB — each Talos node disk defaults to 20 GB"; fi

  if [ -x "$VFKIT" ]; then ok "vfkit: $VFKIT"; else
    warn "vfkit: MISSING — run \`$0 setup\` (brew install vfkit)"; fail=1; fi
  if command -v talosctl >/dev/null; then ok "talosctl: $(command -v talosctl)"; else
    warn "talosctl: MISSING — run \`$0 setup\`"; fail=1; fi
  if command -v kubectl >/dev/null; then ok "kubectl: $(command -v kubectl)"; else
    warn "kubectl: MISSING — run \`$0 setup\`"; fail=1; fi
  if [ -r "$DHCPD_LEASES" ] || [ ! -e "$DHCPD_LEASES" ]; then
    ok "DHCP leases: $DHCPD_LEASES (created by bootpd on first NAT lease)"; else
    warn "DHCP leases: $DHCPD_LEASES exists but is unreadable — \`up\` may need sudo to read the VM IP"; fi

  if [ "$fail" -eq 0 ]; then log "READY — \`$0 up\` will bring up a local Talos cluster."; else
    die "Not ready — resolve the items above (\`$0 setup\` installs the missing deps)."; fi
}

# ── setup ────────────────────────────────────────────────────────────────────
cmd_setup() {
  log "Installing host dependencies (idempotent)"
  command -v brew >/dev/null || die "Homebrew required (https://brew.sh) — cannot auto-install deps."
  [ -x "$VFKIT" ] || command -v vfkit >/dev/null || { log "brew install vfkit"; brew install vfkit; }
  command -v talosctl >/dev/null || { log "brew install siderolabs/talos/talosctl"; brew install siderolabs/talos/talosctl || brew install talosctl; }
  command -v kubectl  >/dev/null || { log "brew install kubernetes-cli"; brew install kubernetes-cli; }
  VFKIT="$(command -v vfkit 2>/dev/null || echo "$VFKIT")"
  ok "host deps present"
  cmd_check
}

# ── helpers for up/down ──────────────────────────────────────────────────────
image_url() { echo "https://factory.talos.dev/image/${SCHEMATIC_ID}/${TALOS_VERSION}/metal-arm64.raw.xz"; }

ensure_image() {
  mkdir -p "$WORKDIR"
  local raw="$WORKDIR/talos-${TALOS_VERSION}-arm64.raw"
  if [ -f "$raw" ]; then echo "$raw"; return; fi
  log "Fetching Talos arm64 metal raw image (Image Factory, Kata-baked schematic)" >&2
  curl -fSL "$(image_url)" -o "$raw.xz" >&2
  xz -d -f "$raw.xz" >&2
  echo "$raw"
}

# Deterministic, locally-administered unicast MAC for a node name. Locally-
# administered + unicast => second-least-significant bit of byte0 set, LSB clear
# (0x52 satisfies both). Bytes 1-2 are a fixed vendor-ish tag; bytes 3-5 derive
# from the node name's hash so the same name always gets the same MAC (lets
# down/status find the VM by recomputing it). Lowercase, colon-separated.
node_mac() {
  local name="$1" h
  h="$(printf '%s' "$name" | shasum | cut -c1-6)"   # 6 hex chars = 3 octets
  printf '52:54:00:%s:%s:%s' "${h:0:2}" "${h:2:2}" "${h:4:2}"
}

# Normalize a MAC for comparison against /var/db/dhcpd_leases: lowercase, drop any
# leading "N," hardware-type prefix, then strip leading zeros from each octet
# (bootpd prints "0c" as "c"). Yields e.g. 52:54:0:6a:0:1 from 52:54:00:6a:00:01.
normalize_mac() {
  printf '%s' "$1" \
    | tr 'A-Z' 'a-z' \
    | sed -E 's/^[0-9]+,//' \
    | awk -F: '{ for (i=1;i<=NF;i++){ o=$i; sub(/^0+/,"",o); if(o=="")o="0"; printf "%s%s", (i>1?":":""), o } }'
}

# Look up the IP that bootpd handed a given MAC, scanning /var/db/dhcpd_leases.
# The file is log-structured (last block for a MAC wins) and groups fields in
# `{ … }` blocks. Prints the IP on success, nothing on miss.
ip_for_mac() {
  local mac want; mac="$1"; want="$(normalize_mac "$mac")"
  [ -r "$DHCPD_LEASES" ] || return 0
  awk -v want="$want" '
    function norm(m,  i,n,o,parts,out){
      gsub(/^[0-9]+,/, "", m); m=tolower(m);
      n=split(m,parts,":"); out="";
      for(i=1;i<=n;i++){ o=parts[i]; sub(/^0+/,"",o); if(o=="")o="0"; out=out (i>1?":":"") o }
      return out
    }
    /\{/   { ip=""; hw="" }
    /ip_address[ \t]*=/ { v=$0; sub(/.*ip_address[ \t]*=[ \t]*/,"",v); gsub(/[ \t]/,"",v); ip=v }
    /hw_address[ \t]*=/ { v=$0; sub(/.*hw_address[ \t]*=[ \t]*/,"",v); gsub(/[ \t]/,"",v); hw=norm(v) }
    /\}/   { if (hw==want && ip!="") found=ip }
    END    { if (found!="") print found }
  ' "$DHCPD_LEASES"
}

# Copy the cached raw image into a per-VM boot disk and grow it to disk_gb.
# Args: disk_path disk_gb raw_image
make_disk() {
  local disk="$1" disk_gb="$2" raw="$3"
  if [ ! -f "$disk" ]; then
    cp "$raw" "$disk"
    # Grow the boot disk; Talos resizes its partition to fill it on first boot.
    # macOS base has no `truncate`, so use BSD `dd` with an empty input (extends
    # the file to seek*bs without rewriting data). Keep the floor guard so we
    # never "shrink" below the image.
    local raw_mib; raw_mib=$(( $(stat -f %z "$disk") / 1048576 ))
    [ "$(( disk_gb * 1024 ))" -gt "$raw_mib" ] \
      || die "--disk-gb ${disk_gb} is smaller than the Talos image (${raw_mib} MiB); raise it"
    dd if=/dev/null of="$disk" bs=1m seek="$(( disk_gb * 1024 ))" 2>/dev/null
  fi
}

# Launch one vfkit VM headless in the background. Args: name cpus ram_mib disk mac
# Writes pid to $WORKDIR/<name>.pid and serial console to $WORKDIR/<name>.log.
# Sets VM_IP (global) to the discovered guest IP, or dies on timeout.
boot_vfkit() {
  local name="$1" cpus="$2" ram_mib="$3" disk="$4" mac="$5"
  local efi="$WORKDIR/${name}.efistore"
  local pidf="$WORKDIR/${name}.pid" logf="$WORKDIR/${name}.log"

  # Already running? (pid file + live process) — reuse it.
  if [ -f "$pidf" ] && kill -0 "$(cat "$pidf" 2>/dev/null)" 2>/dev/null; then
    ok "vfkit already running for $name (pid $(cat "$pidf"))"
  else
    log "Launching vfkit headless: $name (${cpus} vCPU, $(( ram_mib / 1024 )) GB, MAC $mac)"
    # EFI bootloader (creates the variable store on first boot), the per-VM boot
    # disk, a NAT NIC with the fixed MAC, an RNG source, and the serial console
    # piped to a log file. No --gui => headless. Backgrounded; pid captured.
    "$VFKIT" \
      --cpus "$cpus" \
      --memory "$ram_mib" \
      --bootloader "efi,variable-store=${efi},create" \
      --device "virtio-blk,path=${disk}" \
      --device "virtio-net,nat,mac=${mac}" \
      --device "virtio-rng" \
      --device "virtio-serial,logFilePath=${logf}" \
      >>"$logf" 2>&1 &
    echo $! > "$pidf"
    ok "vfkit pid $(cat "$pidf")  log: $logf"
  fi

  log "Waiting for the VM IP via DHCP lease for $mac (Talos boots to maintenance mode)…"
  VM_IP=""; local tries=0
  while [ -z "$VM_IP" ] && [ "$tries" -lt "$IP_WAIT_TRIES" ]; do
    # Guard: if the vfkit process died, surface the log instead of polling blind.
    if [ -f "$pidf" ] && ! kill -0 "$(cat "$pidf" 2>/dev/null)" 2>/dev/null; then
      die "vfkit for $name exited during boot — inspect $logf"
    fi
    VM_IP="$(ip_for_mac "$mac")"
    [ -z "$VM_IP" ] && { sleep 5; tries=$((tries+1)); }
  done
  [ -n "$VM_IP" ] || die "no DHCP lease for $mac after $(( IP_WAIT_TRIES * 5 ))s — check $logf for boot errors, or see the socket_vmnet bridged fallback in README"
  ok "VM IP: $VM_IP"
}

# ── up ───────────────────────────────────────────────────────────────────────
cmd_up() {
  local role="single" name="" cpus="" ram_gb="" disk_gb="20" config_patch=""
  while [ $# -gt 0 ]; do case "$1" in
    --role) role="$2"; shift 2;; --name) name="$2"; shift 2;;
    --cpus) cpus="$2"; shift 2;; --ram-gb) ram_gb="$2"; shift 2;; --disk-gb) disk_gb="$2"; shift 2;;
    --config-patch) config_patch="$2"; shift 2;;
    *) die "unknown up arg: $1";; esac; done
  if [ -n "$config_patch" ] && [ ! -r "$config_patch" ]; then
    die "--config-patch file not readable: $config_patch"
  fi
  case "$role" in control-plane|worker|single) ;; *) die "--role must be control-plane|worker|single";; esac
  [ -n "$name" ] || name="${CLUSTER}-${role}"
  [ -n "$cpus" ] || cpus=4
  [ -n "$ram_gb" ] || ram_gb=8

  cmd_check >/dev/null || die "preflight failed — run \`$0 check\`"
  [ -x "$VFKIT" ] || die "vfkit not found — run \`$0 setup\`"

  local raw ram_mib mac disk; raw="$(ensure_image)"; ram_mib=$(( ram_gb * 1024 ))
  mac="$(node_mac "$name")"; disk="$WORKDIR/${name}.img"
  log "Creating vfkit vz VM: $name (role=$role, ${cpus} vCPU, ${ram_gb} GB, ${disk_gb} GB disk)"
  make_disk "$disk" "$disk_gb" "$raw"
  ok "disk: $disk"

  boot_vfkit "$name" "$cpus" "$ram_mib" "$disk" "$mac"   # sets VM_IP
  local ip="$VM_IP"

  mkdir -p "$WORKDIR"
  if [ "$role" = "worker" ]; then
    [ -f "$WORKDIR/controlplane.yaml" ] || die "worker join needs an existing control plane (run --role control-plane first; reusing $WORKDIR secrets)"
    log "Applying worker config"
    if [ -n "$config_patch" ]; then
      talosctl apply-config --insecure --nodes "$ip" --file "$WORKDIR/worker.yaml" --config-patch "@$config_patch"
    else
      talosctl apply-config --insecure --nodes "$ip" --file "$WORKDIR/worker.yaml"
    fi
    ok "worker $name joined (kubelet registers once the CP admits it)"
    return
  fi

  # control-plane | single
  if [ ! -f "$WORKDIR/controlplane.yaml" ]; then
    log "Generating Talos cluster config (talosctl gen config)"
    local patch=""
    [ "$role" = "single" ] && patch="--config-patch @${WORKDIR}/allow-scheduling.json"
    [ "$role" = "single" ] && printf '{"cluster":{"allowSchedulingOnControlPlanes":true}}' > "$WORKDIR/allow-scheduling.json"
    talosctl gen config "$CLUSTER" "https://${ip}:6443" \
      --kubernetes-version "$K8S_VERSION" --output-dir "$WORKDIR" --force $patch
  fi
  log "Applying control-plane config to $ip"
  if [ -n "$config_patch" ]; then
    talosctl apply-config --insecure --nodes "$ip" --file "$WORKDIR/controlplane.yaml" --config-patch "@$config_patch"
  else
    talosctl apply-config --insecure --nodes "$ip" --file "$WORKDIR/controlplane.yaml"
  fi
  export TALOSCONFIG="$WORKDIR/talosconfig"
  talosctl config endpoint "$ip"; talosctl config node "$ip"
  log "Bootstrapping etcd (once per cluster)"
  local b=0; until talosctl bootstrap --nodes "$ip" 2>/dev/null; do b=$((b+1)); [ "$b" -ge 30 ] && die "bootstrap timed out"; sleep 10; done
  log "Fetching kubeconfig"
  talosctl kubeconfig "$WORKDIR/kubeconfig" --nodes "$ip" --force
  ok "kubeconfig: $WORKDIR/kubeconfig    talosconfig: $WORKDIR/talosconfig"
  log "Done. Use the cluster:"
  printf '  export KUBECONFIG=%s\n  kubectl get nodes -w   # Ready once Cilium lands (kubectl apply the CNI, or it stays NotReady)\n' "$WORKDIR/kubeconfig"
  warn "CNI: this node is cni:none like the fleet — install Cilium to go Ready:"
  printf '    helm install cilium cilium/cilium --version 1.19.4 -n kube-system -f %s\n' "$(git rev-parse --show-toplevel 2>/dev/null)/infra/talos/cilium-values.yaml"
}

# ── down ─────────────────────────────────────────────────────────────────────
# Stop the vfkit process for a node and remove its per-VM disk, EFI store, pid,
# and log. Args: node name.
teardown_node() {
  local t="$1"
  local pidf="$WORKDIR/${t}.pid"
  log "Stopping + deleting $t"
  if [ -f "$pidf" ]; then
    local pid; pid="$(cat "$pidf" 2>/dev/null || true)"
    [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
    # Give vz a moment to release the disk, then hard-kill if still alive.
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then sleep 2; kill -9 "$pid" 2>/dev/null || true; fi
  fi
  rm -f "$WORKDIR/${t}.pid" "$WORKDIR/${t}.log" "$WORKDIR/${t}.img"
  rm -rf "$WORKDIR/${t}.efistore"
  ok "removed $t"
}

cmd_down() {
  local name="" all=0
  while [ $# -gt 0 ]; do case "$1" in --name) name="$2"; shift 2;; --all) all=1; shift;; *) die "unknown down arg: $1";; esac; done
  local targets=()
  if [ "$all" -eq 1 ]; then
    # Every node we ever booted leaves a <name>.pid in WORKDIR.
    if [ -d "$WORKDIR" ]; then
      while IFS= read -r p; do
        [ -n "$p" ] || continue
        local base; base="$(basename "$p" .pid)"; targets+=("$base")
      done < <(find "$WORKDIR" -maxdepth 1 -name '*.pid' 2>/dev/null)
    fi
    [ "${#targets[@]}" -gt 0 ] || warn "no tracked VMs in $WORKDIR"
  else
    [ -n "$name" ] || name="${CLUSTER}-single"; targets=("$name")
  fi
  for t in "${targets[@]}"; do teardown_node "$t"; done
  if [ "$all" -eq 1 ]; then
    rm -f "$WORKDIR"/talosconfig "$WORKDIR"/kubeconfig "$WORKDIR"/controlplane.yaml \
          "$WORKDIR"/worker.yaml "$WORKDIR"/allow-scheduling.json
    ok "cleared cluster secrets + kubeconfig in $WORKDIR (cached image retained)"
  fi
}

# ── status ───────────────────────────────────────────────────────────────────
cmd_status() {
  log "vfkit VMs ($CLUSTER-* and tracked nodes)"
  local any=0
  if [ -d "$WORKDIR" ]; then
    while IFS= read -r pidf; do
      [ -n "$pidf" ] || continue
      any=1
      local base pid state; base="$(basename "$pidf" .pid)"; pid="$(cat "$pidf" 2>/dev/null || true)"
      if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then state="running (pid $pid)"; else state="stopped"; fi
      printf '  %-28s %s  ip=%s\n' "$base" "$state" "$(ip_for_mac "$(node_mac "$base")" || true)"
    done < <(find "$WORKDIR" -maxdepth 1 -name '*.pid' 2>/dev/null)
  fi
  [ "$any" -eq 1 ] || warn "no tracked vfkit VMs (run \`$0 up\`)"
  [ -f "$WORKDIR/kubeconfig" ] && { log "Nodes"; KUBECONFIG="$WORKDIR/kubeconfig" kubectl get nodes 2>/dev/null || warn "cluster unreachable"; }
}

# ── up-multinode (ADR-0381 D2) ───────────────────────────────────────────────
# One-command bring-up of the 7-VM Oyatie Talos topology per MULTINODE-RUNBOOK.md.
# Host sizing: recommended baseline 32-GiB+ macOS host (~22 vCPU + 46 GiB total).
#
# Order matters: cp-0 bootstraps etcd and generates the cluster config bundle
# (controlplane.yaml + worker.yaml). cp-1/cp-2 then join the quorum; workers
# and specialty pool nodes follow.
cmd_up_multinode() {
  local script_dir; script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local patches="$script_dir/patches"
  [ -d "$patches" ] || die "cell patches dir missing: $patches"

  log "ADR-0381 D2 multinode bring-up — 3 CP HA + 2 worker + 1 CI specialty + 1 storage specialty"

  # 1. cp-0 — bootstraps etcd + generates the cluster config bundle.
  log "[1/7] cp-0 (control plane, 2 vCPU + 2 GiB)"
  cmd_up --role control-plane --name "${CLUSTER}-cp-0" --cpus 2 --ram-gb 2 --disk-gb 20 \
         --config-patch "$patches/cell-foundation.yaml"

  # 2. cp-1 + cp-2 — join the etcd quorum.
  local i=1
  local n
  for n in cp-1 cp-2; do
    i=$((i+1))
    log "[$i/7] $n (control plane, 2 vCPU + 2 GiB)"
    cmd_up --role control-plane --name "${CLUSTER}-$n" --cpus 2 --ram-gb 2 --disk-gb 20 \
           --config-patch "$patches/cell-foundation.yaml"
  done

  # 3. Tenant workers (2x).
  for n in worker-0 worker-1; do
    i=$((i+1))
    log "[$i/7] $n (worker / tenant, 4 vCPU + 8 GiB)"
    cmd_up --role worker --name "${CLUSTER}-$n" --cpus 4 --ram-gb 8 --disk-gb 20 \
           --config-patch "$patches/cell-tenant.yaml"
  done

  # 4. CI specialty (1x).
  i=$((i+1))
  log "[$i/7] ci-0 (CI specialty, 6 vCPU + 16 GiB)"
  cmd_up --role worker --name "${CLUSTER}-ci-0" --cpus 6 --ram-gb 16 --disk-gb 40 \
         --config-patch "$patches/cell-ci.yaml"

  # 5. Storage specialty (1x).
  i=$((i+1))
  log "[$i/7] storage-0 (storage specialty, 2 vCPU + 8 GiB + 100 GiB disk)"
  cmd_up --role worker --name "${CLUSTER}-storage-0" --cpus 2 --ram-gb 8 --disk-gb 100 \
         --config-patch "$patches/cell-storage.yaml"

  log "Multinode topology up. Verify:"
  printf '  export KUBECONFIG=%s\n' "$WORKDIR/kubeconfig"
  printf '  kubectl get nodes -L oya.cell/foundation,oya.cell/tenant,oya.cell/ci,oya.cell/storage\n'
  printf '  kubectl describe nodes | grep -E "^Name:|oya\\.cell/|Taints:"\n'
  warn "CNI: cluster is cni:none — install Cilium next:"
  printf '    helm install cilium cilium/cilium --version 1.19.4 -n kube-system -f %s\n' "$(git rev-parse --show-toplevel 2>/dev/null)/infra/talos/cilium-values.yaml"
}

case "${1:-}" in
  check)        shift; cmd_check "$@";;
  setup)        shift; cmd_setup "$@";;
  up)           shift; cmd_up "$@";;
  up-multinode) shift; cmd_up_multinode "$@";;
  down)         shift; cmd_down "$@";;
  status)       shift; cmd_status "$@";;
  *) die "usage: $0 <check|setup|up|up-multinode|down|status> [args]   (see header for flags)";;
esac
