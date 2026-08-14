# Terraform: payments secret bindings (ExternalSecret → OpenBao)
# ADR-0296: library-first credential sidecar; TTL ≤60s
# ADR-0251: compliance pack overlays for KR, EU, CN cells
# PCI DSS Req 7: access by need-to-know

# Provider requirements are consolidated in payments-crdb.tf (Terraform permits
# exactly one required_providers block per module).

# ─── ClusterSecretStore (OpenBao backend) ────────────────────────────────────

resource "kubernetes_manifest" "openbao_cluster_store" {
  manifest = {
    apiVersion = "external-secrets.io/v1beta1"
    kind       = "ClusterSecretStore"
    metadata = {
      name = "openbao-cluster-store"
      annotations = {
        "oyatie/adrs" = "ADR-0296"
      }
    }
    spec = {
      provider = {
        vault = {
          # Canonical deployed OpenBao listener: the oya-kms namespace Service
          # exposes the TLS migration endpoint at port 8202 (see
          # infra/kms/openbao-tls-migration.k8s.yaml).
          server  = "https://openbao.oya-kms.svc:8202"
          path    = "secret"
          version = "v2"
          auth = {
            kubernetes = {
              mountPath      = "kubernetes"
              role           = "external-secrets"
              serviceAccountRef = {
                name = "external-secrets-sa"
              }
            }
          }
          # Root CA comes from the installed ConfigMap (installed by the
          # trusted bootstrap), mirroring infra/external-secrets/*. No CA file
          # is vendored in-tree, so a file() reference would fail at load time.
          caProvider = {
            type      = "ConfigMap"
            name      = "openbao-offline-root-ca"
            key       = "ca.crt"
            namespace = "external-secrets"
          }
        }
      }
    }
  }
}

# ─── Webhook HMAC secrets (per PSP) ─────────────────────────────────────────

resource "kubernetes_manifest" "payments_webhook_hmac_secret" {
  manifest = {
    apiVersion = "external-secrets.io/v1beta1"
    kind       = "ExternalSecret"
    metadata = {
      name      = "payments-webhook-hmac-secrets"
      namespace = "payments"
      annotations = {
        "oyatie/adrs" = "ADR-0296,ADR-0295"
      }
    }
    spec = {
      refreshInterval = "60s"
      secretStoreRef = {
        name = "openbao-cluster-store"
        kind = "ClusterSecretStore"
      }
      target = {
        name              = "payments-webhook-hmac"
        creationPolicy    = "Owner"
        deletionPolicy    = "Retain"
      }
      data = [
        # Keys are relative to the store's KV-v2 mount (path="secret"); ESO
        # performs the mount/data translation itself, so no secret/data/ prefix.
        # One entry per advertised provider (OpenAPI enum: stripe, adyen, toss,
        # kakaopay, line-pay, wechat-pay, alipay) so every PSP callback can be
        # HMAC-authenticated.
        { secretKey = "stripe_webhook_secret",  remoteRef = { key = "payments/webhook-hmac/stripe",  property = "secret" } },
        { secretKey = "adyen_webhook_hmac",     remoteRef = { key = "payments/webhook-hmac/adyen",   property = "hmac_key" } },
        { secretKey = "toss_webhook_secret",    remoteRef = { key = "payments/webhook-hmac/toss",    property = "secret" } },
        { secretKey = "kakaopay_webhook_secret",remoteRef = { key = "payments/webhook-hmac/kakaopay",property = "secret" } },
        { secretKey = "linepay_webhook_secret", remoteRef = { key = "payments/webhook-hmac/line-pay",property = "secret" } },
        { secretKey = "wechatpay_webhook_secret",remoteRef = { key = "payments/webhook-hmac/wechat-pay",property = "secret" } },
        { secretKey = "alipay_webhook_secret",  remoteRef = { key = "payments/webhook-hmac/alipay",  property = "secret" } }
      ]
    }
  }
}

# ─── TLS cert ─────────────────────────────────────────────────────────────────
# TLS material is NOT read through the KV-v2 openbao-cluster-store: the
# certificates are issued by cert-manager into the `payments-pqc-hybrid-tls`
# Secret (iac/pqc-cert.yaml, ClusterIssuer oyatie-tier-0-offline-rooted-ca),
# which is the PKI-aware source of truth. An ExternalSecret pointing at
# pki/cert/... through the KV store would resolve a doubled KV path and never
# populate; removed rather than modeled against the wrong engine.
