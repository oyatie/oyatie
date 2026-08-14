# OpenBao policy for payments µservice
# ADR-0296: library-first credential sidecar; TTL ≤60s per request lifecycle
# ADR-0254: Kata-pod isolation; sidecar runs within same VM boundary
# PCI DSS Req 7: restrict access by need-to-know

# ─── Tenant PSP credentials (provider BYOK per ADR-0255 §D-4) ────────────────
# Format: secret/<tenant_id>/payments/<psp>/<key_name>
#
# NO fleet-wide wildcard grants: a wildcard `secret/data/+/payments/*` would let
# one shared token read every tenant's Stripe/Adyen/Toss/... credentials, and
# the application-level "read only your request's tenant" check sits inside the
# same trust boundary as the workload, so it cannot contain a compromise.
# Instead, each tenant's sidecar receives a CHILD CREDENTIAL issued by the
# secrets-management service, bound to a per-tenant policy granting read on
# exactly `secret/data/<tenant_id>/payments/<psp>/*` for the tenant it serves
# (ADR-0296 library-first issuance; per-tenant policies are generated at
# issuance time, not checked in). The checked-in policy below therefore grants
# NO cross-tenant PSP read.

# ─── Platform-master account (oyatie-internal tenant only) ───────────────────
path "secret/data/oyatie/payments/stripe/*" {
  capabilities = ["read"]
}

path "secret/data/oyatie/payments/adyen/*" {
  capabilities = ["read"]
}

# ─── Webhook HMAC secrets per PSP ────────────────────────────────────────────
path "secret/data/payments/webhook-hmac/*" {
  capabilities = ["read"]
}

# ─── TLS cert (PQC hybrid chain) ────────────────────────────────────────────
path "pki/issue/payments-tls" {
  capabilities = ["create", "update"]
}

path "pki/cert/payments-tls" {
  capabilities = ["read"]
}

# ─── ECH config ──────────────────────────────────────────────────────────────
path "secret/data/payments/ech-config" {
  capabilities = ["read"]
}
