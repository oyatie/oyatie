#!/usr/bin/env bash
# uninstall.sh — reverse of cloudflared/setup-cloudflared.sh.
# Does NOT destroy the Cloudflare-side tunnel (that's tofu-managed via
# infra/cloudflare/). Run `tofu destroy -target=cloudflare_zero_trust_tunnel_cloudflared.onprem_kr`
# in infra/cloudflare/ if you want to remove the tunnel + DNS records too.
set -uo pipefail
cloudflared service uninstall 2>/dev/null || true
systemctl disable --now cloudflared 2>/dev/null || true
rm -f /etc/systemd/system/cloudflared.service
systemctl daemon-reload
dpkg -r cloudflared 2>/dev/null || true
rm -rf /etc/cloudflared
echo "cloudflared uninstalled (CF-side tunnel preserved; tofu destroy to remove)."
