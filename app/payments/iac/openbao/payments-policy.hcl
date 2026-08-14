# OpenBao policy for payments µservice
# ADR-0296: library-first credential sidecar; TTL ≤60s per request lifecycle
# ADR-0254: Kata-pod isolation; sidecar runs within same VM boundary
# PCI DSS Req 7: restrict access by need-to-know

# ─── Tenant PSP credentials (provider BYOK per ADR-0255 §D-4) ────────────────
# Format: secret/<tenant_id>/payments/<psp>/<key_name>
# The payments sidecar reads credentials for the specific (tenant, psp) pair
# determined at request time. Wildcard on tenant_id is intentional — the
# application is responsible for only reading its request's tenant.

path "secret/data/+/payments/stripe/*" {
  capabilities = ["read"]
}

path "secret/data/+/payments/adyen/*" {
  capabilities = ["read"]
}

path "secret/data/+/payments/toss/*" {
  capabilities = ["read"]
}

path "secret/data/+/payments/kakaopay/*" {
  capabilities = ["read"]
}

path "secret/data/+/payments/line-pay/*" {
  capabilities = ["read"]
}

path "secret/data/+/payments/wechat-pay/*" {
  capabilities = ["read"]
}

path "secret/data/+/payments/alipay/*" {
  capabilities = ["read"]
}

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
