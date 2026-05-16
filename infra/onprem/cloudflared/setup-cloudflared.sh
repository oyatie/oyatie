#!/usr/bin/env bash
# setup-cloudflared.sh — install cloudflared on the on-prem host and connect
# the tunnel created by infra/cloudflare/. Requires sudo.
#
# Inputs (env vars, MUST be set by the caller — none touch repo files):
#   CF_TUNNEL_TOKEN  — output of `tofu -chdir=infra/cloudflare output -raw tunnel_token`
#
# Optional:
#   CF_PKG_URL       — override the cloudflared package URL
#
# Run:
#   CF_TUNNEL_TOKEN=$(cd /home/oyatie/projects/oyatie/infra/cloudflare && /home/oyatie/.local/bin/tofu output -raw tunnel_token) \
#     sudo -E bash /home/oyatie/projects/oyatie/infra/onprem/cloudflared/setup-cloudflared.sh

set -euo pipefail

PKG_URL=${CF_PKG_URL:-https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb}

[ -n "${CF_TUNNEL_TOKEN:-}" ] || { echo "ERROR: CF_TUNNEL_TOKEN env var is required" >&2; exit 2; }

echo "==> Step 1/5: download cloudflared .deb"
curl -fL -o /tmp/cloudflared.deb "$PKG_URL"

echo "==> Step 2/5: install"
dpkg -i /tmp/cloudflared.deb

echo "==> Step 3/5: register tunnel as a systemd service"
# The install subcommand creates /etc/systemd/system/cloudflared.service for us,
# wired to the tunnel token. Idempotent (re-running re-registers).
cloudflared service install "$CF_TUNNEL_TOKEN"

echo "==> Step 4/5: enable + start"
systemctl daemon-reload
systemctl enable --now cloudflared

echo "==> Step 5/5: status"
sleep 2
systemctl --no-pager status cloudflared | head -15 || true
echo
journalctl -u cloudflared -n 20 --no-pager | tail -20 || true

echo
echo "==> done. Verify connector health:"
echo "  curl -sI https://bao.<your-domain>/v1/sys/health"
echo "  curl -sI https://foundry.<your-domain>/workspace/api/v1/health"
