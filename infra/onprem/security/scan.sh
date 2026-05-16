#!/usr/bin/env bash
# scan.sh — single weekly security scan. No raw secrets in output.
# Runs as the service ExecStart. All output paths redact tokens/keys/JWT.
#
# Long-term best-fit tools (active multi-year roadmaps; hyperscaler-grade):
#   gitleaks       — Zricethezav / Gitleaks Foundation; OWASP-adjacent
#   trivy          — Aqua Security; GitHub Advanced Security integration
#   debsecan       — Debian-mainline; CVE tracking by package
#   cargo-audit    — RustSec; canonical Rust advisory check
#   unattended-upgrades — Debian-mainline auto-patching for security archive
#
# Scope (per user directive 2026-05-16: scan .claude / .codex / /tmp / logs / histories):
#   - Repo working tree + git history
#   - Host filesystem (CVE)
#   - Agent state: ~/.claude, ~/.codex, ~/.cursor, ~/.vscode
#   - Transient: /tmp, /var/tmp
#   - Auth files: ~/.ssh, ~/.aws, ~/.oci, ~/.kube, ~/.cargo, ~/.gitconfig*, ~/.docker
#   - Shell histories: ~/.bash_history, ~/.zsh_history, ~/.fish_history
#   - System logs: /var/log (excluding journalctl binary)
#   - Kubernetes cluster (if reachable)
#   - Rust workspace (RustSec)
#   - Debian package CVEs
#
# Permission audit: flag any 0644+ file in the auth file set above.
set -uo pipefail

REPO=${OYATIE_REPO:-/home/oyatie/projects/oyatie}
REAL_USER=${SUDO_USER:-oyatie}
REAL_HOME=$(getent passwd "$REAL_USER" | cut -d: -f6)
LOG_DIR=/var/log/oyatie-security
DATE=$(date -u +%Y-%m-%dT%H-%M-%SZ)
OUT="$LOG_DIR/$DATE.log"
mkdir -p "$LOG_DIR"
chmod 0750 "$LOG_DIR"

# Redaction filter — applied to every scanner's output. False-positives on
# prose are OK; missing a real secret is not.
redact () {
  sed -E '
    # Cloudflare API tokens
    s/cfut_[A-Za-z0-9]+/<REDACTED-CF-TOKEN>/g;
    # GitHub PATs (ghp_, gho_, ghu_, ghs_, ghr_) + classic PAT 40-hex
    s/gh[posur]_[A-Za-z0-9_]+/<REDACTED-GH-TOKEN>/g;
    # OpenAI / Anthropic / Google API keys
    s/sk-[A-Za-z0-9_-]{20,}/<REDACTED-AI-KEY>/g;
    s/sk-ant-[A-Za-z0-9_-]+/<REDACTED-ANTHROPIC-KEY>/g;
    # AWS / GCP / Azure / OCI
    s/AKIA[0-9A-Z]{16}/<REDACTED-AWS-KEY>/g;
    s/aws_secret_access_key[[:space:]]*=[[:space:]]*[A-Za-z0-9\/+=]+/aws_secret_access_key=<REDACTED>/g;
    # JWT-shaped strings (3 base64url segments)
    s/eyJ[A-Za-z0-9_+\/=-]{20,}\.[A-Za-z0-9_+\/=-]{10,}\.[A-Za-z0-9_+\/=-]{10,}/<REDACTED-JWT>/g;
    # Bearer + Authorization
    s/(Bearer +)[^[:space:]"]+/\1<REDACTED>/g;
    s/(Authorization: +[^ ]+ +)[^[:space:]"]+/\1<REDACTED>/g;
    # PEM-shaped private keys
    s/-----BEGIN [A-Z ]*PRIVATE KEY-----/<REDACTED-PRIVATE-KEY-BEGIN>/g;
    # Generic high-entropy hex (≥40 chars)
    s/[A-Fa-f0-9]{40,}/<REDACTED-HEX>/g;
  '
}

section () { printf "\n──── %s ────\n" "$*" >> "$OUT"; }

{
  echo "oyatie-security-scan $DATE"
  echo "host=$(hostname)  kernel=$(uname -r)"
  echo "scope: repo + host fs + agent state + transient + auth files + histories + logs + k8s + Rust + Debian CVE"
} > "$OUT"

HIGH_HITS=0

# ---------- 1. gitleaks: repo git history + working tree ----------
section "gitleaks (repo: history + working tree)"
if command -v gitleaks >/dev/null && [ -d "$REPO/.git" ]; then
  gitleaks detect --no-banner --source "$REPO" --redact 2>&1 | redact >> "$OUT"
  HIGH_HITS=$((HIGH_HITS + ${PIPESTATUS[0]} ))
fi

# ---------- 2. gitleaks: agent-state + transient + auth file paths ----------
# Path inventory is shared with cleanup/install.sh via infra/onprem/agent-paths.sh
# so every new agent we adopt gets scanned + cleaned consistently.
# shellcheck source=../agent-paths.sh
source "$(cd "$(dirname "$0")/.." && pwd)/agent-paths.sh"

section "gitleaks (agent state + transient + auth + histories)"
if command -v gitleaks >/dev/null; then
  for target in "${AGENT_SCAN_PATHS[@]}"; do
    [ -e "$target" ] || continue
    echo "  → $target" >> "$OUT"
    gitleaks detect --no-banner --source "$target" --no-git --redact 2>&1 | redact >> "$OUT" || true
  done

  # Walk repo + parent dirs for any .git/worktrees subtrees (agent worktree pattern)
  for repo in "$REPO" "$REAL_HOME"/projects/* "$REAL_HOME"/work/*; do
    [ -d "$repo/.git/worktrees" ] || continue
    for wt in "$repo"/.git/worktrees/*; do
      gitdir=$(cat "$wt/gitdir" 2>/dev/null) || continue
      wtroot=$(dirname "$(dirname "$gitdir")") 2>/dev/null
      [ -d "$wtroot" ] || continue
      echo "  → (worktree) $wtroot" >> "$OUT"
      gitleaks detect --no-banner --source "$wtroot" --redact 2>&1 | redact >> "$OUT" || true
    done
  done

  # Catch stray worktree dirs in /tmp matching common agent naming patterns
  for tmp_wt in /tmp/oyatie-* /tmp/claude-* /tmp/codex-* /tmp/agent-* /tmp/worktree-*; do
    [ -d "$tmp_wt" ] || continue
    echo "  → (tmp worktree) $tmp_wt" >> "$OUT"
    gitleaks detect --no-banner --source "$tmp_wt" --no-git --redact 2>&1 | redact >> "$OUT" || true
  done
fi

# ---------- 3. permission audit (sensitive files MUST be 0600/0700) ----------
section "permission audit (auth files must be 0600 or 0700)"
for path in \
  "$REAL_HOME/.ssh" \
  "$REAL_HOME/.oci" \
  "$REAL_HOME/.aws" \
  "$REAL_HOME/.kube" \
  "$REAL_HOME/.gnupg" \
  "$REAL_HOME/.docker" \
  "$REAL_HOME/.cargo/credentials.toml" \
  "$REAL_HOME/.cargo/credentials" \
  "$REAL_HOME/.git-credentials" \
  "$REAL_HOME/.netrc"
do
  [ -e "$path" ] || continue
  if [ -d "$path" ]; then
    MODE=$(stat -c '%a' "$path")
    if [ "$MODE" != "700" ] && [ "$MODE" != "750" ]; then
      echo "  WARN: $path is mode $MODE (expected 700/750)" >> "$OUT"
    fi
    find "$path" -type f -printf '%m %p\n' 2>/dev/null | while read -r mode file; do
      if [ "$mode" != "600" ] && [ "$mode" != "400" ]; then
        echo "  WARN: $file is mode $mode (expected 600)" >> "$OUT"
      fi
    done
  else
    MODE=$(stat -c '%a' "$path")
    if [ "$MODE" != "600" ] && [ "$MODE" != "400" ]; then
      echo "  WARN: $path is mode $MODE (expected 600)" >> "$OUT"
    fi
  fi
done

# ---------- 4. shell history sweep (last 5000 lines, grep for token patterns) ----------
section "shell history sweep (last 5000 lines per file)"
for h in "$REAL_HOME/.bash_history" "$REAL_HOME/.zsh_history" "$REAL_HOME/.fish_history" "/root/.bash_history"; do
  [ -r "$h" ] || continue
  HITS=$(tail -5000 "$h" 2>/dev/null | grep -cE 'cfut_|ghp_|sk-ant|AKIA|api[_-]?key|secret|token|password|BEGIN[[:space:]]+(RSA[[:space:]])?PRIVATE' || true)
  if [ "$HITS" -gt 0 ]; then
    echo "  WARN: $h has $HITS line(s) matching secret patterns" >> "$OUT"
    echo "    (review with: less '$h'; consider clearing if test/staging)" >> "$OUT"
  else
    echo "  ok:   $h clean" >> "$OUT"
  fi
done

# ---------- 5. trivy fs: workspace ----------
section "trivy fs (workspace)"
if command -v trivy >/dev/null; then
  trivy fs --quiet --skip-dirs target,.terraform,.git --severity HIGH,CRITICAL "$REPO" 2>&1 | redact >> "$OUT"
fi

# ---------- 6. trivy rootfs: host ----------
section "trivy rootfs (host)"
if command -v trivy >/dev/null; then
  trivy rootfs --quiet --severity HIGH,CRITICAL --skip-dirs /proc,/sys,/var/lib/containerd,/srv,/tmp,/var/tmp / 2>&1 | redact >> "$OUT" || true
fi

# ---------- 7. debsecan: Debian CVEs ----------
section "debsecan (Debian package CVEs)"
if command -v debsecan >/dev/null; then
  debsecan --suite "$(lsb_release -cs 2>/dev/null || echo trixie)" --only-fixed --format report 2>&1 | redact >> "$OUT"
fi

# ---------- 8. cargo audit: RustSec ----------
section "cargo audit (RustSec)"
if [ -f "$REPO/Cargo.lock" ]; then
  cd "$REPO"
  sudo -u "$REAL_USER" -H bash -lc 'source ~/.cargo/env 2>/dev/null; cargo audit' 2>&1 | redact >> "$OUT" || true
fi

# ---------- 9. trivy k8s ----------
section "trivy k8s (cluster)"
if command -v trivy >/dev/null && [ -f /etc/kubernetes/admin.conf ]; then
  KUBECONFIG=/etc/kubernetes/admin.conf trivy k8s --quiet --severity HIGH,CRITICAL 2>&1 | redact >> "$OUT" || true
fi

# ---------- 10. summary ----------
section "summary"
WARN_COUNT=$(grep -cE '^  WARN:' "$OUT" 2>/dev/null || echo 0)
CRIT_COUNT=$(grep -cE 'HIGH|CRITICAL' "$OUT" 2>/dev/null || echo 0)
echo "permission WARNings: $WARN_COUNT" >> "$OUT"
echo "HIGH/CRITICAL CVEs:  $CRIT_COUNT" >> "$OUT"
echo "scan complete: $OUT" >> "$OUT"

# ---------- 11. audit-chain emit ----------
if [ -d /srv/oyatie/audit-chain ]; then
  echo "{\"event_type\":\"EVT-SECURITY-SCAN\",\"timestamp_unix\":$(date +%s),\"session_id\":\"$DATE\",\"payload\":{\"log\":\"$OUT\",\"warns\":$WARN_COUNT,\"crits\":$CRIT_COUNT}}" \
    >> /srv/oyatie/audit-chain/security-scan-events.jsonl
fi

chmod 0640 "$OUT"
chown root:"$REAL_USER" "$OUT" 2>/dev/null || true
