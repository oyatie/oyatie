#!/usr/bin/env bash
# install.sh — install OpenBao on the on-prem host per ADR-0043.
#
# Requires sudo. Idempotent on re-run.
# Per the directive flow: this is wave-1 (single-node, file storage, Shamir
# unseal). Wave-2 (HA cluster + HSM auto-unseal) is a follow-up phase.
#
# Run as: sudo bash /home/oyatie/projects/oyatie/infra/onprem/openbao/install.sh

set -euo pipefail

DEB_PATH=${OPENBAO_DEB:-/tmp/openbao_2.5.3_linux_amd64.deb}
EXPECTED_SHA256=a01d4a5442216fd34b20eb3d9c9f3fd66ff6c10574e30a2bebd2c6e303233ef0
HCL_SRC=/home/oyatie/projects/oyatie/infra/onprem/openbao/openbao.hcl
UNIT_SRC=/home/oyatie/projects/oyatie/infra/onprem/openbao/openbao.service
ENV_TARGET=/etc/openbao/openbao.env
DATA_DIR=/srv/oyatie/openbao
AUDIT_DIR=/srv/oyatie/audit-chain

echo "==> Step 1/7: verify .deb sha256"
[ -f "$DEB_PATH" ] || { echo "ERROR: $DEB_PATH missing. Re-run the download step." >&2; exit 2; }
got=$(sha256sum "$DEB_PATH" | awk '{print $1}')
[ "$got" = "$EXPECTED_SHA256" ] || { echo "ERROR: sha256 mismatch ($got vs $EXPECTED_SHA256)"; exit 3; }
echo "    ok ($got)"

echo "==> Step 2/7: apt install"
dpkg -i "$DEB_PATH"

echo "==> Step 3/7: ensure openbao user/group exist"
if ! id -u openbao >/dev/null 2>&1; then
  useradd --system --no-create-home --shell /usr/sbin/nologin --comment 'OpenBao service' openbao
fi

echo "==> Step 4/7: ZFS dataset for OpenBao data + perms"
# Try creating the dataset; ignore if already present.
zfs list oyatie-bulk/srv/openbao >/dev/null 2>&1 || \
  zfs create -o mountpoint="$DATA_DIR" -o copies=2 -o compression=zstd oyatie-bulk/srv/openbao
install -d -o openbao -g openbao -m 0750 "$DATA_DIR" "$DATA_DIR/data"
chown -R openbao:openbao "$DATA_DIR"
chmod 0700 "$DATA_DIR/data"

# audit-chain dataset already exists; just ensure openbao can write.
setfacl -m u:openbao:rwx "$AUDIT_DIR" || chgrp openbao "$AUDIT_DIR" && chmod g+rwx "$AUDIT_DIR"

echo "==> Step 5/7: /etc/openbao"
install -d -m 0750 -o root -g openbao /etc/openbao
install -m 0640 -o root -g openbao "$HCL_SRC" /etc/openbao/openbao.hcl
if [ ! -f "$ENV_TARGET" ]; then
  cat > "$ENV_TARGET" <<'EOF'
# Environment for openbao.service. Reload with: systemctl restart openbao.
BAO_ADDR=http://127.0.0.1:8200
EOF
  chmod 0640 "$ENV_TARGET"
  chown root:openbao "$ENV_TARGET"
fi

echo "==> Step 6/7: systemd unit"
install -m 0644 -o root -g root "$UNIT_SRC" /etc/systemd/system/openbao.service
systemctl daemon-reload
systemctl enable --now openbao.service

echo "==> Step 7/7: smoke + unseal status"
sleep 2
systemctl --no-pager status openbao.service | head -10 || true
echo
BAO_ADDR=http://127.0.0.1:8200 /usr/bin/bao status || true
echo
echo "Next steps (manual, not idempotent):"
echo "  BAO_ADDR=http://127.0.0.1:8200 bao operator init -key-shares=5 -key-threshold=3"
echo "  (record the 5 unseal-key shares + the root token securely; rotate root immediately)"
echo "  BAO_ADDR=http://127.0.0.1:8200 bao audit enable file file_path=$AUDIT_DIR/openbao-audit.jsonl"
echo
echo "==> done."
