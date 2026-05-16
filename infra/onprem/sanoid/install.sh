#!/usr/bin/env bash
# setup-sanoid.sh — write sanoid policy for oyatie-bulk, verify, run once.
# Idempotent: re-running just rewrites the config and re-runs sanoid.
set -euo pipefail

CONF=/etc/sanoid/sanoid.conf

sudo install -d -m 0755 /etc/sanoid

sudo tee "$CONF" > /dev/null <<'EOF'
[template_audit]
frequently = 0
hourly = 48
hourly_min = 5
daily = 60
daily_hour = 3
daily_min = 30
weekly = 12
monthly = 24
monthly_mday = 1
monthly_hour = 3
monthly_min = 30
yearly = 0
autosnap = yes
autoprune = yes

[template_bulk]
frequently = 0
hourly = 0
daily = 14
daily_hour = 3
daily_min = 30
weekly = 4
monthly = 3
monthly_mday = 1
monthly_hour = 3
monthly_min = 30
yearly = 0
autosnap = yes
autoprune = yes

[oyatie-bulk/srv/audit-chain]
use_template = audit

[oyatie-bulk/srv/regional-packs]
use_template = bulk

[oyatie-bulk/srv/object-graph]
use_template = bulk
EOF

echo "=== wrote $CONF ==="
sudo cat "$CONF"
echo

echo "=== sanoid --cron --verbose ==="
sudo sanoid --cron --verbose

echo
echo "=== snapshots now on disk ==="
zfs list -t snapshot

echo
echo "=== sanoid timer status ==="
systemctl list-timers sanoid.timer --no-pager
