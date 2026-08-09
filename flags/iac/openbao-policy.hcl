# OpenBao policy — feature-flags
# Binding ADR: ADR-0296 (library-first credential sidecar; ≤60s TTL)
# All paths follow: secret/<tenant_id>/feature-flags/<scope>/<name>
# Principal isolation: each sidecar holds only the credentials for its role

# ─────────────────────────────────────────────────────────────────────────────
# feature-flags-evaluator: read-only access to flag definitions
# (Postgres read credentials; no provider credentials)
# ─────────────────────────────────────────────────────────────────────────────
path "secret/+/feature-flags/postgres/read" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# feature-flags-manager: write access to flag definitions
# ─────────────────────────────────────────────────────────────────────────────
path "secret/+/feature-flags/postgres/write" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# feature-flags-audit-emitter: access to audit-chain signing key
# ─────────────────────────────────────────────────────────────────────────────
path "secret/+/feature-flags/audit-signing-key" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# feature-flags-pack-overlay-agent: access to pack-overlay signing key
# ─────────────────────────────────────────────────────────────────────────────
path "secret/+/feature-flags/pack-overlay-signing-key" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# Per-tenant DEK (data encryption key) for flag definitions
# ─────────────────────────────────────────────────────────────────────────────
path "secret/+/feature-flags/dek" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# DSAR export encryption key
# ─────────────────────────────────────────────────────────────────────────────
path "secret/+/feature-flags/dsar-export-key" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# Kafka credentials (for kill-switch broadcast and event emission)
# ─────────────────────────────────────────────────────────────────────────────
path "secret/platform/feature-flags/kafka/credentials" {
  capabilities = ["read"]
  max_ttl = "60s"
}

# ─────────────────────────────────────────────────────────────────────────────
# DENY all other paths — defense-in-depth
# ─────────────────────────────────────────────────────────────────────────────
path "*" {
  capabilities = ["deny"]
}
