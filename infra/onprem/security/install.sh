#!/usr/bin/env bash
# install.sh — secret scanning + vulnerability monitoring for the on-prem host.
# Authority: ADR-0118 § hygiene; per-tenant-per-cell HSM (ADR-0043).
# Idempotent. Hooks into setup.sh.
#
# What this installs (system + user mode, no Docker required):
#   - gitleaks         — git history + working-tree secret scanner
#   - trivy            — CVE scanner (filesystem, OCI image, git repo, Kubernetes)
#   - debsecan         — Debian package CVE tracker
#   - unattended-upgrades  — auto-install security updates (security archive only)
#   - cargo-audit      — RustSec advisory check (workspace Cargo.lock)
#
# Plus weekly systemd timers:
#   - oyatie-security-scan.service/.timer — runs all four scanners + emits a
#     redacted summary to /var/log/oyatie-security/<date>.log and audit-chain.
#
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/security/install.sh
set -euo pipefail

REAL_USER=${SUDO_USER:-oyatie}
HERE=$(cd "$(dirname "$0")" && pwd)
LOG_DIR=/var/log/oyatie-security
mkdir -p "$LOG_DIR"
chmod 0750 "$LOG_DIR"

banner () { printf "\n=== %s ===\n" "$*"; }

# ---------- 1. unattended-upgrades (Debian security archive only) ----------
banner "unattended-upgrades (security-archive-only auto-apply)"
apt-get update
apt-get install -y --no-install-recommends unattended-upgrades apt-listchanges
# Configure: security archive only; no autoremove of kernel meta-packages;
# email on changes (relies on user MTA — see hardening/install.sh follow-ups).
cat > /etc/apt/apt.conf.d/50unattended-upgrades-oyatie <<'EOF'
// Managed by infra/onprem/security/install.sh
Unattended-Upgrade::Origins-Pattern {
  "origin=Debian,codename=${distro_codename},label=Debian-Security";
  "origin=Debian,codename=${distro_codename}-security,label=Debian-Security";
};
Unattended-Upgrade::Package-Blacklist {
  "linux-image-.*";
  "linux-headers-.*";
};
Unattended-Upgrade::AutoFixInterruptedDpkg "true";
Unattended-Upgrade::MinimalSteps "true";
Unattended-Upgrade::InstallOnShutdown "false";
Unattended-Upgrade::Remove-Unused-Kernel-Packages "false";
Unattended-Upgrade::Remove-Unused-Dependencies "false";
Unattended-Upgrade::Automatic-Reboot "false";
Unattended-Upgrade::Mail "root";
Unattended-Upgrade::MailReport "on-change";
EOF
cat > /etc/apt/apt.conf.d/20auto-upgrades <<'EOF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Download-Upgradeable-Packages "1";
APT::Periodic::AutocleanInterval "7";
APT::Periodic::Unattended-Upgrade "1";
EOF
systemctl enable --now unattended-upgrades

# ---------- 2. debsecan ----------
banner "debsecan (Debian package CVE tracker)"
apt-get install -y --no-install-recommends debsecan

# ---------- 3. gitleaks (binary; user-mode-friendly) ----------
banner "gitleaks (git secret scanner)"
GITLEAKS_VERSION=${GITLEAKS_VERSION:-8.21.2}
if ! command -v gitleaks >/dev/null || ! gitleaks version 2>/dev/null | grep -q "$GITLEAKS_VERSION"; then
  curl -fsSL "https://github.com/gitleaks/gitleaks/releases/download/v${GITLEAKS_VERSION}/gitleaks_${GITLEAKS_VERSION}_linux_x64.tar.gz" \
    | tar -C /usr/local/bin -xz gitleaks
  chmod 0755 /usr/local/bin/gitleaks
fi
gitleaks version || true

# ---------- 4. trivy ----------
banner "trivy (CVE scanner: filesystem, image, repo, k8s)"
if ! command -v trivy >/dev/null; then
  curl -fsSL https://aquasecurity.github.io/trivy-repo/deb/public.key | gpg --dearmor -o /etc/apt/keyrings/trivy.gpg
  echo "deb [signed-by=/etc/apt/keyrings/trivy.gpg] https://aquasecurity.github.io/trivy-repo/deb $(lsb_release -cs 2>/dev/null || echo trixie) main" \
    > /etc/apt/sources.list.d/trivy.list
  apt-get update
  apt-get install -y --no-install-recommends trivy
fi
trivy --version | head -1

# ---------- 5. cargo-audit (RustSec advisory) ----------
banner "cargo-audit (Rust workspace advisory check)"
sudo -u "$REAL_USER" -H bash -lc '
  if ! command -v cargo-audit >/dev/null; then
    source ~/.cargo/env 2>/dev/null || true
    cargo install --locked cargo-audit >/dev/null 2>&1 || true
  fi
  command -v cargo-audit && cargo audit --version
'

# ---------- 6. scan service + timer ----------
banner "weekly security scan systemd timer (Sun 02:30)"
install -m 0755 "$HERE/scan.sh" /usr/local/bin/oyatie-security-scan
install -m 0644 "$HERE/oyatie-security-scan.service" /etc/systemd/system/oyatie-security-scan.service
install -m 0644 "$HERE/oyatie-security-scan.timer"   /etc/systemd/system/oyatie-security-scan.timer
systemctl daemon-reload
systemctl enable --now oyatie-security-scan.timer
systemctl list-timers oyatie-security-scan.timer --no-pager | head -3 || true

echo
echo "==> security.sh done."
echo "    Run an immediate scan:  sudo systemctl start oyatie-security-scan.service"
echo "    Inspect last scan:      ls -la $LOG_DIR && less \$(ls -t $LOG_DIR/*.log | head -1)"
