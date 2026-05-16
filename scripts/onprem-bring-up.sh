#!/usr/bin/env bash
# onprem-bring-up.sh — production bring-up for this on-prem host.
#   1. Install python3-venv so OCI CLI can land into a user venv.
#   2. Run /home/oyatie/setup-oyatie-service.sh (creates `oya` user,
#      builds + installs oya-ops-workspace-shell, drops hardened
#      systemd unit + weekly restart timer, smoke-tests /health).
#   3. Install OCI CLI into ~/.oci-cli-venv.
#   4. Print final readiness summary.
#
# Run as: sudo bash /home/oyatie/projects/oyatie/scripts/onprem-bring-up.sh
#   (this script invokes the real user back to oyatie for the venv step;
#    only steps 1 + 2 need root.)
set -euo pipefail

REAL_USER=${SUDO_USER:-oyatie}
REAL_HOME=$(getent passwd "$REAL_USER" | cut -d: -f6)

echo "==> Step 1/4: apt install python3-venv (needed for OCI CLI install)"
apt-get update
apt-get install -y --no-install-recommends python3-venv

echo "==> Step 2/4: run setup-oyatie-service.sh (foundry cell + systemd)"
# The script itself uses sudo, but we're already root so sudo is a no-op.
sudo -u "$REAL_USER" -H bash -lc '/home/oyatie/setup-oyatie-service.sh'

echo "==> Step 3/4: install OCI CLI into $REAL_HOME/.oci-cli-venv (as $REAL_USER)"
sudo -u "$REAL_USER" -H bash -lc "
  set -euo pipefail
  python3 -m venv $REAL_HOME/.oci-cli-venv
  $REAL_HOME/.oci-cli-venv/bin/pip install --quiet --upgrade pip
  $REAL_HOME/.oci-cli-venv/bin/pip install --quiet oci-cli
  ln -sfn $REAL_HOME/.oci-cli-venv/bin/oci $REAL_HOME/.local/bin/oci 2>/dev/null || mkdir -p $REAL_HOME/.local/bin && ln -sfn $REAL_HOME/.oci-cli-venv/bin/oci $REAL_HOME/.local/bin/oci
  $REAL_HOME/.oci-cli-venv/bin/oci --version
"

echo
echo "==> Step 4/4: readiness summary"
echo "systemctl status oyatie.service ----------"
systemctl --no-pager status oyatie.service | head -15 || true
echo
echo "health endpoint --------------------------"
PORT=$(grep -oP '^OYATIE_OPS_WORKSPACE_PORT=\K\d+' /etc/oyatie/oyatie.env || echo 8080)
curl --silent --show-error --fail "http://127.0.0.1:$PORT/workspace/api/v1/health" && echo
echo
echo "OCI CLI ----------------------------------"
sudo -u "$REAL_USER" -H bash -lc "$REAL_HOME/.oci-cli-venv/bin/oci --version"
echo
echo "==> done. Add to ~/.bashrc if not present:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
