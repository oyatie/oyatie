#!/usr/bin/env bash
# talos-local.sh — one-shot local Talos substrate on this Apple-Silicon Mac via
# UTM + Apple Virtualization.framework (`vz` backend). Production-fidelity local
# Kubernetes (real VMs, real disks/NICs, nested virt for Kata) — the laptop
# mirror of the bare-metal/cloud fleet in infra/capi (ADR-0375).
#
# Subcommands:
#   check                 preflight ONLY (read-only): arch, macOS, chip, RAM,
#                         nested-virt, UTM/utmctl/talosctl/kubectl. Exit 0 = ready.
#   setup                 install missing host deps (UTM, talosctl, kubectl) via brew. Idempotent.
#   up   [--role R] [--name N] [--cpus N] [--ram-gb N] [--disk-gb N]
#                         create + boot the VM(s), apply Talos config, (for cp/single)
#                         bootstrap etcd, fetch kubeconfig + talosconfig. Idempotent-ish.
#   down [--name N] [--all]   stop + delete the VM(s) and the local config bundle.
#   status                show VM + node state.
#
#   --role control-plane | worker | single   (default: single — 1 schedulable CP, dev box)
#
# Why UTM-vz (not QEMU/HVF, not docker): Apple Virtualization.framework exposes
# nested virtualization on M3+/macOS 15+, which the Kata runtime tier (ADR-0147/0338)
# needs. Raw QEMU-HVF does not. This is the production-fidelity local substrate.
#
# HONESTY: UTM has no first-class "create VM from ISO" CLI. This script generates a
# `.utm` bundle (config.plist for the apple-vz backend) programmatically so bring-up
# is GUI-less. The plist schema is UTM-version-sensitive; if `up` fails at VM-create
# on a future UTM, fall back to the documented golden-VM + `utmctl clone` path in
# README.md (the talosctl config/bootstrap/kubeconfig legs below still apply).

set -euo pipefail

# ── constants ────────────────────────────────────────────────────────────────
TALOS_VERSION="${TALOS_VERSION:-v1.13.3}"          # Talos OS version (arm64)
K8S_VERSION="${K8S_VERSION:-1.36.1}"               # Kubernetes version
CLUSTER="${CLUSTER:-oya-local}"
# Image Factory schematic with the Kata Containers extension baked in (matches the
# fleet's kataInstallerImage). arm64 metal image for the vz backend.
SCHEMATIC_ID="${SCHEMATIC_ID:-3da7f440f279f4814fa73bdf83c84710a8e93c40a4a3cbba4d969f14afb96298}"
WORKDIR="${WORKDIR:-$HOME/.oya/talos-local}"        # gen configs + downloaded image live here
UTM_DOCS="$HOME/Library/Containers/com.utmapp.UTM/Data/Documents"
MIN_MACOS_MAJOR=15                                  # nested virt floor

UTMCTL="$(command -v utmctl 2>/dev/null || echo /Applications/UTM.app/Contents/MacOS/utmctl)"

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

  if [ -d /Applications/UTM.app ]; then ok "UTM.app: present"; else
    warn "UTM.app: MISSING — run \`$0 setup\` (brew install --cask utm)"; fail=1; fi
  if [ -x "$UTMCTL" ]; then ok "utmctl: $UTMCTL"; else
    warn "utmctl: MISSING (ships with UTM) — run \`$0 setup\`"; fail=1; fi
  if command -v talosctl >/dev/null; then ok "talosctl: $(command -v talosctl)"; else
    warn "talosctl: MISSING — run \`$0 setup\`"; fail=1; fi
  if command -v kubectl >/dev/null; then ok "kubectl: $(command -v kubectl)"; else
    warn "kubectl: MISSING — run \`$0 setup\`"; fail=1; fi

  if [ "$fail" -eq 0 ]; then log "READY — \`$0 up\` will bring up a local Talos cluster."; else
    die "Not ready — resolve the items above (\`$0 setup\` installs the missing deps)."; fi
}

# ── setup ────────────────────────────────────────────────────────────────────
cmd_setup() {
  log "Installing host dependencies (idempotent)"
  command -v brew >/dev/null || die "Homebrew required (https://brew.sh) — cannot auto-install deps."
  [ -d /Applications/UTM.app ] || { log "brew install --cask utm"; brew install --cask utm; }
  command -v talosctl >/dev/null || { log "brew install siderolabs/talos/talosctl"; brew install siderolabs/talos/talosctl || brew install talosctl; }
  command -v kubectl  >/dev/null || { log "brew install kubernetes-cli"; brew install kubernetes-cli; }
  ok "host deps present"
  cmd_check
}

# ── helpers for up/down ──────────────────────────────────────────────────────
image_url() { echo "https://factory.talos.dev/image/${SCHEMATIC_ID}/${TALOS_VERSION}/metal-arm64.raw.xz"; }

ensure_image() {
  mkdir -p "$WORKDIR"
  local raw="$WORKDIR/talos-${TALOS_VERSION}-arm64.raw"
  if [ -f "$raw" ]; then echo "$raw"; return; fi
  log "Fetching Talos arm64 image (Image Factory, Kata-baked schematic)" >&2
  curl -fSL "$(image_url)" -o "$raw.xz" >&2
  xz -d -f "$raw.xz" >&2
  echo "$raw"
}

# Generate a UTM apple-vz .utm bundle for one node. Args: name cpus ram_mib disk_gb raw_image
make_utm_bundle() {
  local name="$1" cpus="$2" ram_mib="$3" disk_gb="$4" raw="$5"
  local bundle="$UTM_DOCS/${name}.utm"
  mkdir -p "$bundle/Data"
  # Convert the raw Talos image into the VM's boot disk (qcow/raw copy).
  local disk="$bundle/Data/disk0.img"
  if [ ! -f "$disk" ]; then
    cp "$raw" "$disk"
    # Grow the boot disk to the requested size; Talos resizes its partition to
    # fill it on first boot. macOS base has no `truncate`, so use BSD `dd` with
    # an empty input (extends the file to seek*bs without rewriting data).
    local raw_mib; raw_mib=$(( $(stat -f %z "$disk") / 1048576 ))
    [ "$(( disk_gb * 1024 ))" -gt "$raw_mib" ] \
      || die "--disk-gb ${disk_gb} is smaller than the Talos image (${raw_mib} MiB); raise it"
    dd if=/dev/null of="$disk" bs=1m seek="$(( disk_gb * 1024 ))" 2>/dev/null
  fi
  # Minimal config.plist for the Apple Virtualization (vz) backend. Schema is
  # UTM-version-sensitive; see README for the golden-VM fallback if this drifts.
  cat > "$bundle/config.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Backend</key><string>apple</string>
  <key>ConfigurationVersion</key><integer>4</integer>
  <key>Information</key><dict><key>Name</key><string>${name}</string></dict>
  <key>System</key><dict>
    <key>Architecture</key><string>aarch64</string>
    <key>CPUCount</key><integer>${cpus}</integer>
    <key>MemorySize</key><integer>${ram_mib}</integer>
    <key>Boot</key><dict><key>OperatingSystem</key><string>Linux</string></dict>
  </dict>
  <key>Drive</key><array><dict>
    <key>Identifier</key><string>disk0</string>
    <key>ImageName</key><string>disk0.img</string>
    <key>ImageType</key><string>Disk</string>
  </dict></array>
  <key>Network</key><array><dict>
    <key>Mode</key><string>Shared</string>
    <key>Hardware</key><string>virtio-net-device</string>
  </dict></array>
</dict></plist>
PLIST
  echo "$bundle"
}

# ── up ───────────────────────────────────────────────────────────────────────
cmd_up() {
  local role="single" name="" cpus="" ram_gb="" disk_gb="20"
  while [ $# -gt 0 ]; do case "$1" in
    --role) role="$2"; shift 2;; --name) name="$2"; shift 2;;
    --cpus) cpus="$2"; shift 2;; --ram-gb) ram_gb="$2"; shift 2;; --disk-gb) disk_gb="$2"; shift 2;;
    *) die "unknown up arg: $1";; esac; done
  case "$role" in control-plane|worker|single) ;; *) die "--role must be control-plane|worker|single";; esac
  [ -n "$name" ] || name="${CLUSTER}-${role}"
  [ -n "$cpus" ] || cpus=4
  [ -n "$ram_gb" ] || ram_gb=8

  cmd_check >/dev/null || die "preflight failed — run \`$0 check\`"
  [ -x "$UTMCTL" ] || die "utmctl not found"

  local raw bundle ram_mib; raw="$(ensure_image)"; ram_mib=$(( ram_gb * 1024 ))
  log "Creating UTM vz VM: $name (role=$role, ${cpus} vCPU, ${ram_gb} GB, ${disk_gb} GB disk)"
  bundle="$(make_utm_bundle "$name" "$cpus" "$ram_mib" "$disk_gb" "$raw")"
  ok "bundle: $bundle"

  log "Starting VM (utmctl)"
  "$UTMCTL" start "$name" || die "utmctl start failed — open UTM once to register the bundle, then re-run, or use the golden-VM clone path (README)."
  log "Waiting for the VM IP (Talos boots to maintenance mode)…"
  local ip="" tries=0
  while [ -z "$ip" ] && [ "$tries" -lt 60 ]; do
    ip="$("$UTMCTL" ip-address "$name" 2>/dev/null | awk 'NR==1{print}')"; [ -z "$ip" ] && { sleep 5; tries=$((tries+1)); }
  done
  [ -n "$ip" ] || die "no VM IP after 5 min — check UTM console for boot errors"
  ok "VM IP: $ip"

  mkdir -p "$WORKDIR"
  if [ "$role" = "worker" ]; then
    [ -f "$WORKDIR/controlplane.yaml" ] || die "worker join needs an existing control plane (run --role control-plane first; reusing $WORKDIR secrets)"
    log "Applying worker config"
    talosctl apply-config --insecure --nodes "$ip" --file "$WORKDIR/worker.yaml"
    ok "worker $name joined (kubelet registers once the CP admits it)"
    return
  fi

  # control-plane | single
  if [ ! -f "$WORKDIR/controlplane.yaml" ]; then
    log "Generating Talos cluster config (talosctl gen config)"
    local patch=""
    [ "$role" = "single" ] && patch="--config-patch @${WORKDIR}/allow-scheduling.json"
    [ "$role" = "single" ] && printf '[{"op":"add","path":"/cluster/allowSchedulingOnControlPlanes","value":true}]' > "$WORKDIR/allow-scheduling.json"
    talosctl gen config "$CLUSTER" "https://${ip}:6443" \
      --kubernetes-version "$K8S_VERSION" --output-dir "$WORKDIR" --force $patch
  fi
  log "Applying control-plane config to $ip"
  talosctl apply-config --insecure --nodes "$ip" --file "$WORKDIR/controlplane.yaml"
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
cmd_down() {
  local name="" all=0
  while [ $# -gt 0 ]; do case "$1" in --name) name="$2"; shift 2;; --all) all=1; shift;; *) die "unknown down arg: $1";; esac; done
  [ -x "$UTMCTL" ] || die "utmctl not found"
  local targets=()
  if [ "$all" -eq 1 ]; then
    while IFS= read -r v; do case "$v" in ${CLUSTER}-*) targets+=("$v");; esac; done < <("$UTMCTL" list 2>/dev/null | awk 'NR>1{print $NF}')
  else
    [ -n "$name" ] || name="${CLUSTER}-single"; targets=("$name")
  fi
  for t in "${targets[@]}"; do
    log "Stopping + deleting $t"
    "$UTMCTL" stop "$t" 2>/dev/null || true
    "$UTMCTL" delete "$t" 2>/dev/null || true
    rm -rf "$UTM_DOCS/${t}.utm"
    ok "removed $t"
  done
  [ "$all" -eq 1 ] && { rm -rf "$WORKDIR"; ok "cleared $WORKDIR (cluster secrets + kubeconfig)"; }
}

# ── status ───────────────────────────────────────────────────────────────────
cmd_status() {
  log "UTM VMs ($CLUSTER-*)"; "$UTMCTL" list 2>/dev/null | awk "NR==1 || /${CLUSTER}-/" || warn "utmctl unavailable"
  [ -f "$WORKDIR/kubeconfig" ] && { log "Nodes"; KUBECONFIG="$WORKDIR/kubeconfig" kubectl get nodes 2>/dev/null || warn "cluster unreachable"; }
}

case "${1:-}" in
  check)  shift; cmd_check "$@";;
  setup)  shift; cmd_setup "$@";;
  up)     shift; cmd_up "$@";;
  down)   shift; cmd_down "$@";;
  status) shift; cmd_status "$@";;
  *) die "usage: $0 <check|setup|up|down|status> [args]   (see header for flags)";;
esac
