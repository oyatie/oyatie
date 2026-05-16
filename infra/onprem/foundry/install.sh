#!/usr/bin/env bash
# setup-oyatie-service.sh — wire up oya-ops-workspace-shell as a systemd service.
# Idempotent: safe to re-run after edits.
#
# Steps:
#   1. Create `oya` system user/group
#   2. Create /etc/oyatie + env file
#   3. Create /var/lib/oyatie subdirs + chown the ZFS /srv/oyatie mounts
#   4. cargo build --release the binary
#   5. install to /usr/local/bin/oya-ops-workspace-shell
#   6. drop oyatie.service + oyatie-restart.{service,timer}
#   7. daemon-reload, enable, start
#   8. smoke-test the /health endpoint
set -euo pipefail

REPO=/home/oyatie/projects/oyatie
BIN_NAME=oya-ops-workspace-shell
SERVICE_USER=oya
SERVICE_GROUP=oya

# ---- 1. Service user (system account, no shell, no home dir) ----
if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
  echo "==> creating system user $SERVICE_USER"
  sudo useradd --system --no-create-home --shell /usr/sbin/nologin \
    --home-dir /var/lib/oyatie --comment "Oyatie service" "$SERVICE_USER"
else
  echo "==> system user $SERVICE_USER already exists"
fi

# ---- 2. /etc/oyatie + env file ----
sudo install -d -m 0755 -o root -g "$SERVICE_GROUP" /etc/oyatie
if [ ! -f /etc/oyatie/oyatie.env ]; then
  sudo tee /etc/oyatie/oyatie.env > /dev/null <<'EOF'
# Environment for oya-ops-workspace-shell.service.
# See main.rs for accepted variables. Reload via:
#   sudo systemctl restart oyatie.service
OYATIE_OPS_WORKSPACE_PORT=8080
EOF
  sudo chmod 0640 /etc/oyatie/oyatie.env
  sudo chown root:"$SERVICE_GROUP" /etc/oyatie/oyatie.env
fi

# ---- 3. Data directories ----
sudo install -d -m 0750 -o "$SERVICE_USER" -g "$SERVICE_GROUP" /var/lib/oyatie
sudo install -d -m 0750 -o "$SERVICE_USER" -g "$SERVICE_GROUP" /var/lib/oyatie/cells
sudo install -d -m 0750 -o "$SERVICE_USER" -g "$SERVICE_GROUP" /var/lib/oyatie/outbox
sudo install -d -m 0750 -o "$SERVICE_USER" -g "$SERVICE_GROUP" /var/lib/oyatie/index

# ZFS mounts already exist; just chown them.
for d in /srv/oyatie/audit-chain /srv/oyatie/regional-packs /srv/oyatie/object-graph; do
  sudo chown "$SERVICE_USER":"$SERVICE_GROUP" "$d"
  sudo chmod 0750 "$d"
done

# ---- 4. Build the release binary ----
echo "==> cargo build --release -p oya-ops-workspace-shell-app --bin $BIN_NAME"
( cd "$REPO" && cargo build --release -p oya-ops-workspace-shell-app --bin "$BIN_NAME" )

# ---- 5. Install ----
sudo install -o root -g root -m 0755 \
  "$REPO/target/release/$BIN_NAME" "/usr/local/bin/$BIN_NAME"
echo "==> installed: $(ls -l /usr/local/bin/$BIN_NAME)"

# ---- 6. systemd units ----
sudo tee /etc/systemd/system/oyatie.service > /dev/null <<EOF
[Unit]
Description=Oyatie ops workspace-shell cell
Documentation=https://github.com/anthropics/oyatie
After=network-online.target zfs-mount.service
Wants=network-online.target zfs-mount.service

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_GROUP
EnvironmentFile=/etc/oyatie/oyatie.env
ExecStart=/usr/local/bin/$BIN_NAME
Restart=on-failure
RestartSec=5

# Hardening — don't loosen without thinking. ReadWritePaths is the allowlist.
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
NoNewPrivileges=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
LockPersonality=true
RestrictRealtime=true
RestrictNamespaces=true
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources
MemoryDenyWriteExecute=true

ReadWritePaths=/srv/oyatie /var/lib/oyatie
RuntimeDirectory=oyatie
RuntimeDirectoryMode=0750
LogsDirectory=oyatie

[Install]
WantedBy=multi-user.target
EOF

# Weekly restart at Sunday 03:45 — runs try-restart so a failed start
# doesn't leave the service down (try-restart only acts if it's running).
sudo tee /etc/systemd/system/oyatie-restart.service > /dev/null <<'EOF'
[Unit]
Description=Restart oyatie.service (triggered by oyatie-restart.timer)
After=oyatie.service

[Service]
Type=oneshot
ExecStart=/bin/systemctl try-restart oyatie.service
EOF

sudo tee /etc/systemd/system/oyatie-restart.timer > /dev/null <<'EOF'
[Unit]
Description=Weekly restart of oyatie.service at Sunday 03:45

[Timer]
OnCalendar=Sun 03:45:00
Persistent=true
RandomizedDelaySec=0

[Install]
WantedBy=timers.target
EOF

# ---- 7. Reload + enable + start ----
sudo systemctl daemon-reload
sudo systemctl enable --now oyatie.service
sudo systemctl enable --now oyatie-restart.timer

echo
echo "==> systemctl status oyatie.service"
systemctl --no-pager status oyatie.service || true

echo
echo "==> timer"
systemctl list-timers oyatie-restart.timer --no-pager

# ---- 8. Smoke test ----
sleep 1
PORT=$(grep -oP '^OYATIE_OPS_WORKSPACE_PORT=\K\d+' /etc/oyatie/oyatie.env || echo 8080)
echo
echo "==> smoke test: curl http://127.0.0.1:$PORT/workspace/api/v1/health"
curl --silent --show-error --fail "http://127.0.0.1:$PORT/workspace/api/v1/health" && echo
echo "==> done."
