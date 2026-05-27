#!/usr/bin/env bash
# onprem-host-decommission.sh — tear down host-direct services so this box becomes
# a thin Talos substrate. Pairs with scripts/onprem-bring-up.sh.
#
# Doctrine: everything that's a workload moves into Talos. Host keeps only the bits
# that physically can't (Tailscale daemon, Cloudflared connector, VM hypervisor).
# Phased cutover — Docker stays for now (Omni's host) and is removed in a later step
# once Omni has migrated into the Talos bootstrap cluster.
#
# This is a CLEAN uninstall: apt purge (configs gone), data dirs deleted, system
# users removed, unit files removed. Idempotent: safe to re-run.

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "must run as root (sudo bash $0)" >&2
  exit 1
fi

step() { printf '\n=== %s ===\n' "$*"; }

step "1/11 stop + disable + mask stale systemd units"
UNITS=(
  openbao.service
  oya-auth-proxy.service
  oyatie.service
  oyatie-cleanup.timer  oyatie-cleanup.service
  oyatie-restart.timer  oyatie-restart.service
  kubelet.service
)
systemctl disable --now "${UNITS[@]}" 2>&1 | sed 's/^/  /' || true
# Mask so they can't be accidentally re-enabled by a runaway dependency.
systemctl mask "${UNITS[@]}" 2>&1 | sed 's/^/  /' || true

step "2/11 purge apt packages (configs go too)"
apt-get purge -y openbao kubernetes-cni kubelet kubeadm 2>&1 | tail -5 | sed 's/^/  /' || true

step "3/11 drop kubernetes apt repo (Talos brings its own kubelet)"
rm -fv /etc/apt/sources.list.d/kubernetes.list \
       /etc/apt/keyrings/kubernetes-apt-keyring.gpg | sed 's/^/  /'

step "4/11 remove stale binaries from /usr/local/bin"
# DO NOT remove /usr/local/bin/containerd* here. This host's containerd.service
# (an override unit) ExecStart-ed /usr/local/bin/containerd, and dockerd talks to
# THAT containerd via /run/containerd/containerd.sock. Deleting it broke the docker
# substrate (status=203/EXEC; new container ops fail) on 2026-05-26. If you want to
# consolidate onto the apt containerd.io binary, FIRST repoint the unit:
#   sudo sed -i 's|/usr/local/bin/containerd|/usr/bin/containerd|' \
#     /etc/systemd/system/containerd.service && sudo systemctl daemon-reload \
#     && sudo systemctl restart containerd docker
# THEN the /usr/local/bin/containerd* copies are safe to remove.
rm -fv \
  /usr/local/bin/oya-auth-proxy \
  /usr/local/bin/oya-ops-workspace-shell \
  /usr/local/bin/istioctl \
  /usr/bin/kubelet | sed 's/^/  /'

step "5/11 remove manually-deployed software trees (istio)"
rm -rfv /opt/istio | tail -10 | sed 's/^/  /' || true

step "6/11 remove unit files + override drop-ins + service data + configs"
rm -rfv \
  /etc/systemd/system/openbao.service \
  /etc/systemd/system/oya-auth-proxy.service \
  /etc/systemd/system/oyatie.service \
  /etc/systemd/system/oyatie.service.d \
  /etc/systemd/system/oyatie-cleanup.service \
  /etc/systemd/system/oyatie-cleanup.timer \
  /etc/systemd/system/oyatie-restart.service \
  /etc/systemd/system/oyatie-restart.timer \
  /etc/openbao \
  /etc/oyatie \
  /etc/oya \
  /var/lib/openbao \
  /var/lib/kubelet \
  /var/lib/etcd \
  /var/lib/cni \
  /etc/kubernetes \
  /etc/cni \
  /opt/cni \
  /var/log/openbao \
  /run/openbao | tail -30 | sed 's/^/  /'
systemctl daemon-reload

step "7/11 remove orphaned system users + groups"
for u in openbao oya; do
  if id "$u" &>/dev/null; then
    deluser --remove-home "$u" 2>&1 | sed 's/^/  /' || true
  fi
done
for g in oya openbao; do
  if getent group "$g" &>/dev/null; then
    delgroup "$g" 2>&1 | sed 's/^/  /' || true
  fi
done

step "8/11 apt autoremove + refresh index"
apt-get autoremove --purge -y 2>&1 | tail -5 | sed 's/^/  /'
apt-get update -qq

step "9/11 vacuum journal + apt cache + /root history"
journalctl --vacuum-time=1d 2>&1 | tail -3 | sed 's/^/  /' || true
apt-get clean 2>&1 | sed 's/^/  /'
rm -fv \
  /root/.bash_history \
  /root/.python_history \
  /root/.lesshst \
  /root/.viminfo \
  /root/.cache/anthropic 2>&1 | sed 's/^/  /'

step "10/11 wipe per-user AI assistant + shell history (fresh-install posture)"
# Resolve the invoking user's home dir; with sudo, $HOME is /root which is wrong.
TARGET_USER="${SUDO_USER:-$USER}"
TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
echo "  target user: $TARGET_USER  home: $TARGET_HOME"
# AI assistant state (Claude Code, Codex, and the usual suspects).
# NOTE: if this script is run from INSIDE a Claude Code session, the current
# session may re-create some of these files as it continues. For a fully clean
# wipe, exit the Claude/Codex session first, then re-run.
rm -rfv \
  "$TARGET_HOME/.claude" \
  "$TARGET_HOME/.codex" \
  "$TARGET_HOME/.config/codex" \
  "$TARGET_HOME/.config/anthropic" \
  "$TARGET_HOME/.config/aichat" \
  "$TARGET_HOME/.gemini" \
  "$TARGET_HOME/.continue" \
  "$TARGET_HOME/.aider.chat.history.md" \
  "$TARGET_HOME/.aider.input.history" \
  "$TARGET_HOME/.aider.tags.cache.v3" 2>&1 | tail -10 | sed 's/^/  /'
# Rust toolchain (rustup + cargo) — oya-* crates build in CI / inside the cluster.
# Removes ~5-10 GB of toolchain + downloaded registry/git deps.
rm -rfv \
  "$TARGET_HOME/.rustup" \
  "$TARGET_HOME/.cargo" 2>&1 | tail -5 | sed 's/^/  /'

# User-level caches (mostly node_modules cache + browser cache + tool caches).
rm -rfv \
  "$TARGET_HOME/.cache" \
  "$TARGET_HOME/.npm" 2>&1 | tail -3 | sed 's/^/  /'

step "11/11 sweep build artifacts (target/, node_modules/, __pycache__/, .terraform/) under /home"
# Stale duplicate clone (last touched ~10 days before this script runs).
if [ -d "$TARGET_HOME/projects/oyatie" ]; then
  echo "  removing stale duplicate clone: $TARGET_HOME/projects/oyatie"
  rm -rf "$TARGET_HOME/projects/oyatie"
  rmdir "$TARGET_HOME/projects" 2>/dev/null || true
fi
# Build-artifact dirs (regeneratable from source). .git/ is excluded so we don't
# accidentally nuke git internals that happen to share a name.
echo "  removing build-artifact dirs under $TARGET_HOME"
find "$TARGET_HOME" -depth -type d \
  \( -name target -o -name node_modules -o -name __pycache__ \
     -o -name .terraform -o -name .next -o -name .nuxt \) \
  -not -path '*/.git/*' \
  -exec rm -rf {} + 2>/dev/null || true
# Stray .pyc files outside __pycache__/.
find "$TARGET_HOME" -type f -name '*.pyc' -not -path '*/.git/*' -delete 2>/dev/null || true
# Shell + pager history.
rm -fv \
  "$TARGET_HOME/.bash_history" \
  "$TARGET_HOME/.zsh_history" \
  "$TARGET_HOME/.python_history" \
  "$TARGET_HOME/.node_repl_history" \
  "$TARGET_HOME/.lesshst" \
  "$TARGET_HOME/.local/share/recently-used.xbel" 2>&1 | sed 's/^/  /'

step "verify ports freed"
for p in 8200 9200 8081 8080 10250 10248 10257 10259 2379; do
  if ss -tlnH 2>/dev/null | awk '{print $4}' | grep -qE ":${p}\$"; then
    printf "  port %-6s : STILL LISTENING (investigate)\n" "$p"
  else
    printf "  port %-6s : freed\n" "$p"
  fi
done

step "what survives"
echo "  apt repos:"
ls /etc/apt/sources.list.d/ | sed 's/^/    /'
echo "  /usr/local/bin/:"
ls /usr/local/bin/ 2>/dev/null | sed 's/^/    /'
echo "  systemd running (filtered):"
systemctl list-units --type=service --state=running --no-pager 2>&1 \
  | awk '/\.service/ && !/systemd-/ {print "    " $1 " — " substr($0, index($0,$5))}' \
  | head -20

echo
echo "CLEANUP_OK"
echo "Next: install libvirt deps, then \`cd infra/talos/tofu && tofu init && tofu apply\` (OpenTofu cluster bring-up)."
