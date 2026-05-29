# OpenBao (Vault-compatible) policy — connector microservice
# Binding ADR: ADR-0296 (library-first credential sidecar)
# Per-tenant OAuth credential isolation; connector-adapter-worker reads short-lived tokens only
# Path format: secret/<tenant_id>/connect/<scope>/<name>
# Hyperscaler precedent: HashiCorp Vault per-service policy + dynamic secrets

# ─── oauth-broker-rest: can write/read OAuth grants ──────────────────────────
path "secret/data/+/connect/oauth/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
  # Bound to: oauth-broker-rest Kubernetes service account
  # Required: SPIFFE SVID from connect namespace (ADR-0295)
}

path "secret/metadata/+/connect/oauth/*" {
  capabilities = ["read", "delete", "list"]
}

# ─── oauth-broker-rest: can read provider-credential BYOK client credentials (ADR-0255 §D-4)
path "secret/data/+/connect/oauth-clients/*" {
  capabilities = ["read"]
  # provider-credential BYOK client_id + client_secret stored by tenant at onboarding (ADR-0255 §D-4)
}

# ─── oauth-broker-rest: can write OAuth state nonces (10min TTL) ─────────────
path "secret/data/+/connect/oauth-state/*" {
  capabilities = ["create", "read", "delete"]
}

# ─── connector-adapter-worker: can read access tokens only (sidecar issues them) ─
# The sidecar (not the worker) reads refresh_tokens. Worker reads only access tokens.
path "secret/data/+/connect/oauth/access-token/*" {
  capabilities = ["read"]
  # Short-lived (≤60s TTL) access tokens written by sidecar, read by worker
}

# ─── webhook-receiver-edge: can read signing secrets ────────────────────────
path "secret/data/+/connect/webhooks/*" {
  capabilities = ["read"]
}

# ─── webhook-endpoint-register (catalog API): can write signing secrets ──────
path "secret/data/+/connect/webhooks/*" {
  capabilities = ["create", "update"]
}

# ─── schema-drift-monitor: read connector vendor schema cache ────────────────
path "secret/data/+/connect/schema-cache/*" {
  capabilities = ["read", "list"]
}

# ─── DLQ replay worker: read tenant-scoped DLQ replay tokens ─────────────────
path "secret/data/+/connect/dlq-replay-token/*" {
  capabilities = ["read"]
}

# ─── DENY: any path not explicitly granted ───────────────────────────────────
# Implicit deny — OpenBao default-deny baseline (no explicit deny needed;
# anything not permitted above is denied per OpenBao policy semantics)

# ─── PagerDuty service key (emergency-services class) ────────────────────────
# PagerDuty connector triggers must always succeed; service key stored separately
path "secret/data/+/connect/pagerduty/service-key" {
  capabilities = ["read"]
  # Emergency-services bypass: this path has elevated lease TTL (300s, not 60s)
  # per critical_path.emergency_services = true in pagerduty.yaml
}
