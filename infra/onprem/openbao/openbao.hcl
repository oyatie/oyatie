# OpenBao server config — on-prem KR primary cell.
# Per ADR-0043: OpenBao (MPL-2) is the canonical secrets store; per-cell deploy.
# Per user directive 2026-05-16: large-state services run on-prem (ZFS bulk).
#
# This is the STAGE-0 single-node config. Production hardening per ADR-0043:
#   - HA cluster of 3-5 nodes per cell
#   - HSM auto-unseal (Thales Luna 7 for KR; AWS CloudHSM globally)
#   - KCMVP + FIPS 140-3 validated module
#   - Per-tenant per-cell HSM partition
# That upgrade lands in a follow-up phase.

# File storage on ZFS dataset (3.5 TB oyatie-bulk; sanoid-snapshotted).
storage "file" {
  path = "/srv/oyatie/openbao/data"
}

listener "tcp" {
  address     = "127.0.0.1:8200"
  tls_disable = true  # behind Cloudflare Tunnel / reverse proxy in production
                      # per ADR-0043 + cloudflared follow-up.
}

# Cluster address (single-node now; HA later).
api_addr     = "http://127.0.0.1:8200"
cluster_addr = "http://127.0.0.1:8201"

# Web UI on the same listener.
ui = true

# OpenBao 2.5.0+ dropped mlock support entirely (the prior `disable_mlock`
# directive is now a hard parse error). Per upstream guidance, disable or
# encrypt swap instead — see https://openbao.org/docs/install/#post-installation-hardening.
# The systemd unit's `LimitMEMLOCK=infinity` + `CAP_IPC_LOCK` are now no-ops
# but harmless; they remain so a downgrade is still serviceable.
#
# Swap audit on this host: `swapon --show`. If non-empty, either:
#   a) `sudo swapoff -a && sudo sed -i '/swap/d' /etc/fstab`
#   b) configure encrypted swap via dm-crypt.

# Audit logs go to audit-chain dataset on ZFS. OpenBao 2.5.x requires audit
# devices to be declared in the config file (the runtime `bao audit enable`
# API was removed). Type is an attribute inside the block; per-device
# parameters go under `options`.
audit {
  type = "file"
  path = "audit_file"
  options = {
    file_path     = "/srv/oyatie/audit-chain/openbao-audit.jsonl"
    log_raw       = "false"
    hmac_accessor = "true"
  }
}

# Log level: INFO for steady state; DEBUG only for incident triage.
log_level = "info"

# Performance: low default cache for the single-node dev shape.
default_lease_ttl = "768h"  # 32 days
max_lease_ttl     = "8760h" # 365 days
