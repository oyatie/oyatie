#!/usr/bin/env bash
# setup.sh — single-command on-prem bring-up: hardening → setup → verify.
# Authority: ADR-0118 (k8s stack), ADR-0043 (OpenBao), CLAUDE.md root-hub.
#
# Everything called from here is tracked in this repo under infra/onprem/.
# The historical scripts under /home/oyatie/ are mirrored here verbatim so
# git history covers every change.
#
# Idempotent. Safe to re-run at any point — each child step exits cleanly if
# its target state is already achieved.
#
# Usage:
#   sudo bash /home/oyatie/projects/oyatie/infra/onprem/setup.sh
#
# Phases (each is its own idempotent script under infra/onprem/<phase>/):
#   1. hardening/              sysctl, ZFS scrub, smartmontools, fail2ban, journald cap, BBR
#   2. sanoid/                 ZFS snapshot schedule (audit-chain, regional-packs, openbao, ...)
#   3. reboots/                weekly restart timers (oyatie.service + others)
#   4. foundry/                Foundry workspace-shell systemd cell (oyatie.service)
#   5. openbao/                OpenBao Wave-1 (file storage on ZFS, Shamir 5/3 seal)
#   6. podman/                 Rootless container runtime
#   7. containerd/             v2.3.0 LTS + runc 1.4.0 + CNI 1.6.0
#   8. kubeadm/                k8s 1.35 (iptables-legacy backend pinned for Debian 13)
#   9. istio/                  Istio 1.29.2 control plane + Envoy sidecars
#   10. cloudflared            tunnel daemon (token from tofu cloudflare module)
#   11. diagnose.sh            full health report
set -euo pipefail

REAL_USER=${SUDO_USER:-oyatie}
HERE=$(cd "$(dirname "$0")" && pwd)
LOG=/var/log/oyatie-setup.log
mkdir -p /var/log
: > "$LOG"
banner () { printf "\n══════════════════════════════════════════════════════════════════\n  %s\n══════════════════════════════════════════════════════════════════\n\n" "$*" | tee -a "$LOG"; }
log () { printf "[%s] %s\n" "$(date -u +%FT%TZ)" "$*" | tee -a "$LOG"; }
trap 'log "STEP FAILED at line $LINENO. Inspect $LOG and re-run."; exit 1' ERR

# ---------- 0. cleanup ----------
banner "0/12  system cleanup (apt autoremove + journald vacuum + image GC)"
bash "$HERE/cleanup/install.sh"

# ---------- 0b. security (scanners + auto-patching) ----------
banner "0b/12  security (gitleaks + trivy + debsecan + cargo-audit + unattended-upgrades)"
bash "$HERE/security/install.sh"

# ---------- 1. hardening ----------
banner "1/11  hardening (sysctl, ZFS scrub, smartmontools, journald cap, fail2ban, BBR)"
bash "$HERE/hardening/install.sh"

# ---------- 2. sanoid ----------
banner "2/11  sanoid (ZFS snapshot schedule)"
bash "$HERE/sanoid/install.sh"

# ---------- 3. reboots ----------
banner "3/11  weekly restart timers"
bash "$HERE/reboots/install.sh"

# ---------- 4. Foundry cell ----------
banner "4/11  Foundry workspace-shell systemd cell"
if systemctl is-active --quiet oyatie.service; then
  log "oyatie.service already active — skipping (use systemctl restart to refresh)"
else
  bash "$HERE/foundry/install.sh"
fi

# ---------- 5. OpenBao ----------
banner "5/11  OpenBao (KR primary cell secrets store, per ADR-0043)"
if systemctl is-active --quiet openbao.service; then
  log "openbao.service already active — skipping install"
  log "Seal status: $(BAO_ADDR=http://127.0.0.1:8200 /usr/bin/bao status 2>/dev/null | grep -E '^Sealed' || echo 'unknown')"
else
  bash "$HERE/openbao/install.sh"
  echo
  echo "→ ACTION REQUIRED: initialize and unseal OpenBao now (interactive):"
  echo "    BAO_ADDR=http://127.0.0.1:8200 bao operator init -key-shares=5 -key-threshold=3 > ~/openbao-init-output.txt && chmod 600 ~/openbao-init-output.txt"
  echo "    # then 3x: BAO_ADDR=http://127.0.0.1:8200 bao operator unseal"
  echo "    # then:    move keys offline, shred -u ~/openbao-init-output.txt"
fi

# ---------- 6. Podman ----------
banner "6/11  Podman (rootless container runtime)"
if command -v podman >/dev/null; then
  log "podman already installed ($(podman --version))"
else
  bash "$HERE/podman/install.sh"
fi

# ---------- 7. containerd ----------
banner "7/11  containerd 2.3.0 LTS + runc 1.4.0 + CNI 1.6.0"
if systemctl is-active --quiet containerd.service && [ -x /usr/local/bin/containerd ]; then
  log "containerd already active ($(/usr/local/bin/containerd --version))"
else
  bash "$HERE/containerd/install.sh"
fi

# ---------- 8. kubeadm ----------
banner "8/11  kubeadm k8s 1.35 (iptables-legacy backend pinned)"
if [ -f /etc/kubernetes/admin.conf ] && systemctl is-active --quiet kubelet; then
  log "kubernetes already initialized; verifying"
  KUBECONFIG=/etc/kubernetes/admin.conf kubectl get nodes -o wide 2>&1 | tee -a "$LOG"
else
  bash "$HERE/kubeadm/install.sh"
fi

# ---------- 9. Istio ----------
banner "9/11  Istio 1.29.2 + Envoy"
if KUBECONFIG=/etc/kubernetes/admin.conf kubectl get deploy -n istio-system istiod >/dev/null 2>&1; then
  log "istiod already installed; verifying"
  KUBECONFIG=/etc/kubernetes/admin.conf kubectl get pods -n istio-system 2>&1 | tee -a "$LOG"
else
  bash "$HERE/istio/install.sh"
fi

# ---------- 10. Cloudflare Tunnel ----------
banner "10/11  Cloudflare Tunnel daemon"
if systemctl is-active --quiet cloudflared 2>/dev/null; then
  log "cloudflared already active"
else
  echo
  echo "→ ACTION REQUIRED: apply tofu Cloudflare module first, then run setup-cloudflared.sh:"
  echo "    export TF_VAR_cloudflare_api_token='...'"
  echo "    export TF_VAR_cloudflare_account_id='...'"
  echo "    export TF_VAR_cloudflare_zone_id='...'"
  echo "    /home/oyatie/.local/bin/tofu -chdir=$HERE/../cloudflare apply -auto-approve"
  echo "    sudo -E env CF_TUNNEL_TOKEN=\"\$(/home/oyatie/.local/bin/tofu -chdir=$HERE/../cloudflare output -raw tunnel_token)\" \\"
  echo "      bash $HERE/cloudflared/setup-cloudflared.sh"
fi

# ---------- 11. diagnostics ----------
banner "11/11  diagnostics"
bash "$HERE/diagnose.sh" | tee -a "$LOG" || true

banner "setup.sh complete"
echo "Full log: $LOG"
echo "Re-run anytime — it's idempotent."
