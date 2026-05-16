#!/usr/bin/env bash
# diagnose.sh — exhaustive on-prem health report for the oyatie host.
# Run anytime (no sudo required for most checks; some sections gracefully
# degrade if the user can't read root-owned state).
#
# Output is structured per section with a per-section GREEN/RED verdict.
# Exit code: 0 if all checks green, 1 if any red.
set +e

HERE=$(cd "$(dirname "$0")" && pwd)
GREEN="\033[32m"
RED="\033[31m"
YELLOW="\033[33m"
RESET="\033[0m"
ok () { printf "${GREEN}✓${RESET} %s\n" "$*"; }
warn () { printf "${YELLOW}⚠${RESET} %s\n" "$*"; }
bad () { printf "${RED}✗${RESET} %s\n" "$*"; RED_HITS=$((RED_HITS+1)); }
section () { echo; printf "─── %s ────────────────────────────────────────\n" "$*"; echo; }
RED_HITS=0

section "1. host basics"
uname -a
echo "uptime: $(uptime -p 2>/dev/null || true)"
[ "$(cat /proc/sys/net/ipv4/ip_forward 2>/dev/null)" = "1" ] && ok "ip_forward=1" || bad "ip_forward=0"
[ "$(cat /proc/sys/net/bridge/bridge-nf-call-iptables 2>/dev/null)" = "1" ] && ok "bridge-nf=1" || warn "bridge-nf-call-iptables not 1 (k8s pod net may misbehave)"
[ -f /sbin/iptables ] || [ -L /usr/sbin/iptables ] && ok "iptables symlink resolves: $(readlink -f /usr/sbin/iptables 2>/dev/null)" || bad "iptables missing"

section "2. ZFS"
if command -v zfs >/dev/null; then
  zfs list 2>&1 | head -10
  for ds in oyatie-bulk/srv/audit-chain oyatie-bulk/srv/openbao oyatie-bulk/srv/regional-packs; do
    zfs list "$ds" >/dev/null 2>&1 && ok "dataset $ds exists" || warn "dataset $ds missing"
  done
else
  warn "zfs CLI not present"
fi

section "3. systemd services"
# Critical = the runtime path. Failure here is RED.
for svc in oyatie openbao containerd kubelet cloudflared; do
  state=$(systemctl is-active "$svc" 2>/dev/null)
  case "$state" in
    active)   ok "$svc: $state" ;;
    inactive|failed) bad "$svc: $state" ;;
    *)        warn "$svc: $state" ;;
  esac
done
# Hardening polish — failure is YELLOW (warn), not critical-path.
for svc in smartmontools zfs-zed fail2ban; do
  state=$(systemctl is-active "$svc" 2>/dev/null)
  case "$state" in
    active)   ok "$svc (hardening): $state" ;;
    *)        warn "$svc (hardening): $state — re-run sudo bash $HERE/hardening/install.sh" ;;
  esac
done

section "4. local HTTP endpoints"
for url in "http://127.0.0.1:8200/v1/sys/health" "http://127.0.0.1:8080/workspace/api/v1/health"; do
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 "$url")
  case "$code" in
    200|429|473|503) ok "$url → $code" ;;
    *) bad "$url → $code" ;;
  esac
done

section "5. public HTTPS endpoints (via Cloudflare Tunnel)"
for url in "https://kms.oyatie.com/v1/sys/health" "https://foundry.oyatie.com/workspace/api/v1/health" "https://ops.oyatie.com/workspace/api/v1/health"; do
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 8 "$url")
  case "$code" in
    200|429|473|503) ok "$url → $code" ;;
    *) bad "$url → $code" ;;
  esac
done

section "6. Kubernetes (kubeadm + Istio)"
if command -v kubectl >/dev/null && [ -f "$HOME/.kube/config" ]; then
  kubectl get nodes -o wide 2>&1 | head -5
  bad_pods=$(kubectl get pods -A --no-headers 2>/dev/null | awk '$4!="Running" && $4!="Completed" {print $0}' | wc -l)
  if [ "$bad_pods" -eq 0 ]; then
    ok "all pods Running/Completed"
  else
    bad "$bad_pods pod(s) not Ready:"
    kubectl get pods -A --no-headers 2>/dev/null | awk '$4!="Running" && $4!="Completed" {print "    " $0}'
  fi
  if command -v istioctl >/dev/null; then
    /usr/local/bin/istioctl version 2>&1 | head -3
  fi
else
  warn "kubectl/kubeconfig not present (Kubernetes not installed?)"
fi

section "7. OpenBao seal status"
if command -v bao >/dev/null; then
  BAO_ADDR=http://127.0.0.1:8200 bao status 2>&1 | head -10
else
  warn "bao CLI not present"
fi

section "8. Cloudflare Tunnel"
if systemctl is-active --quiet cloudflared; then
  ok "cloudflared active"
  # Redact the tunnel token — never print raw secrets, even in diagnostics.
  systemctl show cloudflared -p ExecStart --value 2>/dev/null \
    | head -1 \
    | sed -E 's/(--token )[^ ]+/\1<REDACTED>/g; s/eyJ[A-Za-z0-9+\/._=-]{20,}/<REDACTED-JWT>/g'
else
  warn "cloudflared not active (tunnel hostnames will not resolve)"
fi

section "9. host firewall"
if command -v nft >/dev/null; then
  nft list ruleset 2>&1 | head -5 || echo "(no nftables rules)"
fi

section "10. process tree summary"
ps -eo comm= 2>/dev/null | grep -E '^(oya-ops-workspa|bao|containerd|kubelet|kube-|etcd|cloudflared|istiod|envoy)' | sort -u | head -20

section "11. security scanner status"
if systemctl list-timers oyatie-security-scan.timer >/dev/null 2>&1; then
  systemctl list-timers oyatie-security-scan.timer --no-pager 2>/dev/null | head -3
  LATEST=$(ls -t /var/log/oyatie-security/*.log 2>/dev/null | head -1)
  if [ -n "$LATEST" ]; then
    ok "last scan: $LATEST"
    sudo cat "$LATEST" 2>/dev/null | tail -6 | sed 's/^/    /' || \
      tail -6 "$LATEST" 2>/dev/null | sed 's/^/    /' || \
      echo "    (last log not readable to this user — sudo to inspect)"
  else
    warn "no security scan has run yet — start one now:  sudo systemctl start oyatie-security-scan.service"
  fi
else
  warn "oyatie-security-scan.timer not installed — run sudo bash $HERE/security/install.sh"
fi

for tool in trivy gitleaks debsecan cargo-audit; do
  if command -v "$tool" >/dev/null; then
    case "$tool" in
      trivy) v=$(trivy --version 2>/dev/null | head -1) ;;
      gitleaks) v=$(gitleaks version 2>/dev/null | head -1) ;;
      debsecan) v=$(dpkg -s debsecan 2>/dev/null | awk '/^Version:/ {print $2}') ;;
      cargo-audit) v=$(cargo audit --version 2>/dev/null | head -1) ;;
    esac
    ok "$tool installed: $v"
  else
    warn "$tool not installed"
  fi
done

section "VERDICT"
if [ "$RED_HITS" -eq 0 ]; then
  printf "${GREEN}HEALTHY${RESET} — all critical checks green\n"
  exit 0
else
  printf "${RED}DEGRADED${RESET} — $RED_HITS critical check(s) failed; see above\n"
  exit 1
fi
