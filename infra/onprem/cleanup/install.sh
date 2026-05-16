#!/usr/bin/env bash
# install.sh — system cleanup for the on-prem host.
# Idempotent. Hooks into setup.sh phase 0 + can be run independently.
# Authority: ADR-0119 § hygiene; user directive 2026-05-16 (no unneeded packages or garbage).
#
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/cleanup/install.sh
set -euo pipefail

banner () { printf "\n=== %s ===\n" "$*"; }
log    () { printf "  %s\n" "$*"; }

# ---------- 1. apt: autoremove + autoclean ----------
banner "apt autoremove + autoclean"
log "Before:"
df -h / | tail -1
dpkg -l 2>/dev/null | wc -l | xargs -I {} echo "  installed packages: {}"

# Remove obsolete dependencies (e.g., old kernel images, build artifacts).
apt-get -y autoremove --purge

# Drop cached .deb files that are no longer the latest version.
apt-get -y autoclean

# Drop cached .deb files entirely (next apt-update will refetch metadata).
apt-get -y clean

log "After:"
df -h / | tail -1
dpkg -l 2>/dev/null | wc -l | xargs -I {} echo "  installed packages: {}"

# ---------- 2. systemd journal vacuum ----------
banner "journald vacuum (cap matches /etc/systemd/journald.conf SystemMaxUse=2G)"
journalctl --vacuum-size=2G --vacuum-time=30d 2>&1 | tail -5 || true

# ---------- 3. old kernels (belt + suspenders past autoremove) ----------
banner "old kernels"
CURRENT_KERNEL=$(uname -r)
log "current: $CURRENT_KERNEL"
INSTALLED=$(dpkg -l 'linux-image-*' 2>/dev/null | awk '/^ii/ {print $2}')
echo "$INSTALLED" | sed 's/^/  installed: /'
# Detect images that aren't the running kernel and aren't pinned by meta-pkg.
STALE=$(echo "$INSTALLED" | grep -v "$CURRENT_KERNEL" | grep -v 'meta\|generic\|signed' || true)
if [ -n "$STALE" ]; then
  log "stale kernel images detected; autoremove should have caught these but didn't:"
  echo "$STALE" | sed 's/^/    /'
  log "(skipping forced purge; review manually with: sudo apt purge <kernel-pkg>)"
fi

# ---------- 4. containerd / k8s image GC ----------
banner "containerd image GC"
if command -v crictl >/dev/null; then
  crictl --runtime-endpoint unix:///var/run/containerd/containerd.sock rmi --prune 2>&1 | tail -10 || true
elif command -v ctr >/dev/null; then
  # Container/ctr-based fallback. ctr doesn't have built-in prune; list dangling.
  ctr -n k8s.io images list 2>/dev/null | head -5 || true
  log "(crictl missing — install via /home/oyatie/projects/oyatie/infra/onprem/cleanup/install-crictl.sh for proper image pruning)"
fi

# ---------- 5. podman image GC ----------
banner "podman image GC"
if command -v podman >/dev/null; then
  REAL_USER=${SUDO_USER:-oyatie}
  sudo -u "$REAL_USER" -H podman system prune -f 2>&1 | tail -5 || true
else
  log "(podman not installed)"
fi

# ---------- 6. ZFS snapshot accumulation (sanoid policy enforces; just report) ----------
banner "ZFS snapshot accounting"
if command -v zfs >/dev/null; then
  for ds in oyatie-bulk/srv/audit-chain oyatie-bulk/srv/openbao oyatie-bulk/srv/regional-packs oyatie-bulk/srv/object-graph; do
    if zfs list "$ds" >/dev/null 2>&1; then
      COUNT=$(zfs list -t snapshot -H "$ds" 2>/dev/null | wc -l)
      SIZE=$(zfs list -H -o used "$ds@" 2>/dev/null | head -1 || echo "?")
      log "$ds: $COUNT snapshots, $SIZE used by snaps (sanoid enforces retention)"
    fi
  done
fi

# ---------- 7. /tmp + agent-scratch + cache cleanup ----------
# Path globs + cache dirs come from infra/onprem/agent-paths.sh so we stay in
# sync with the security scanner's inventory (claude/codex/gemini/cursor/etc.).
# shellcheck source=../agent-paths.sh
source "$(cd "$(dirname "$0")/.." && pwd)/agent-paths.sh"

banner "agent transient scratch (>${AGENT_CLEANUP_DAYS} days inactive)"
for glob in "${AGENT_CLEANUP_GLOBS[@]}"; do
  for path in $glob; do
    [ -e "$path" ] || continue
    if [ -d "$path" ]; then
      # Reap whole worktree dirs whose mtime is older than the threshold.
      find "$path" -maxdepth 0 -mtime +"$AGENT_CLEANUP_DAYS" -print -exec rm -rf {} + 2>/dev/null
    else
      find "$path" -maxdepth 0 -mtime +"$AGENT_CLEANUP_DAYS" -print -delete 2>/dev/null
    fi
  done
done

banner "agent cache directories (>${AGENT_CLEANUP_DAYS} days inactive at the file level)"
for cache in "${AGENT_CACHE_DIRS[@]}"; do
  [ -d "$cache" ] || continue
  log "  → $cache"
  find "$cache" -type f -mtime +"$AGENT_CLEANUP_DAYS" -delete 2>/dev/null
  find "$cache" -type d -empty -delete 2>/dev/null
done

banner "stale git worktrees (untracked / detached >${AGENT_CLEANUP_DAYS}d)"
for tmp_wt in /tmp/oyatie-* /tmp/claude-* /tmp/codex-* /tmp/agent-* /tmp/worktree-*; do
  [ -d "$tmp_wt" ] || continue
  if find "$tmp_wt" -maxdepth 0 -mtime +"$AGENT_CLEANUP_DAYS" -printf '' 2>/dev/null | grep -q .; then
    log "  reaping stale worktree: $tmp_wt"
    rm -rf "$tmp_wt"
  fi
done
# Also prune any worktree records the underlying directories no longer exist for.
for repo in /home/oyatie/projects/* "$REAL_HOME"/work/*; do
  [ -d "$repo/.git" ] || continue
  ( cd "$repo" && git worktree prune --expire="${AGENT_CLEANUP_DAYS}.days.ago" 2>/dev/null ) || true
done

# legacy generic /tmp + /var/tmp sweep (anything missed by the agent globs)
banner "/tmp + /var/tmp legacy sweep (>30 days)"
find /tmp     -mindepth 1 -maxdepth 1 -mtime +30 -print -exec rm -rf {} + 2>/dev/null | head -20 || true
find /var/tmp -mindepth 1 -maxdepth 1 -mtime +30 -print -exec rm -rf {} + 2>/dev/null | head -20 || true

# ---------- 8. cargo target/ size report (build artifacts) ----------
banner "cargo target/ size"
if [ -d /home/oyatie/projects/oyatie/target ]; then
  du -sh /home/oyatie/projects/oyatie/target 2>/dev/null || true
  log "(run 'cargo clean' under the repo to reclaim — only do this when not actively building)"
fi

# ---------- 9. final df ----------
banner "final disk usage"
df -h / /srv/oyatie/* 2>/dev/null | head -20

# ---------- 10. install/refresh the weekly cleanup timer ----------
banner "weekly cleanup timer (Sun 04:15)"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
if [ -f "$SCRIPT_DIR/oyatie-cleanup.service" ]; then
  install -m 0644 "$SCRIPT_DIR/oyatie-cleanup.service" /etc/systemd/system/oyatie-cleanup.service
  install -m 0644 "$SCRIPT_DIR/oyatie-cleanup.timer"   /etc/systemd/system/oyatie-cleanup.timer
  systemctl daemon-reload
  systemctl enable --now oyatie-cleanup.timer
  systemctl list-timers oyatie-cleanup.timer --no-pager 2>/dev/null | head -3 || true
fi

echo
echo "==> cleanup.sh done."
