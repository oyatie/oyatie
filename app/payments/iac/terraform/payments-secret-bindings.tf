# Terraform: payments secret bindings (ExternalSecret → OpenBao)
# ADR-0296: library-first credential sidecar; TTL ≤60s
# ADR-0251: compliance pack overlays for KR, EU, CN cells
# PCI DSS Req 7: access by need-to-know

terraform {
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.30"
    }
  }
}

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
          server  = "https://openbao.oyatie-secrets.svc.cluster.local:8200"
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
          caBundle = "${base64encode(file(\"certs/openbao-ca.crt\"))}"
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
        { secretKey = "stripe_webhook_secret",  remoteRef = { key = "secret/data/payments/webhook-hmac/stripe",  property = "secret" } },
        { secretKey = "adyen_webhook_hmac",     remoteRef = { key = "secret/data/payments/webhook-hmac/adyen",   property = "hmac_key" } },
        { secretKey = "toss_webhook_secret",    remoteRef = { key = "secret/data/payments/webhook-hmac/toss",    property = "secret" } },
        { secretKey = "kakaopay_webhook_secret",remoteRef = { key = "secret/data/payments/webhook-hmac/kakaopay",property = "secret" } }
      ]
    }
  }
}

# ─── TLS cert secret binding ─────────────────────────────────────────────────

resource "kubernetes_manifest" "payments_tls_external_secret" {
  manifest = {
    apiVersion = "external-secrets.io/v1beta1"
    kind       = "ExternalSecret"
    metadata = {
      name      = "payments-tls-external-secret"
      namespace = "payments"
    }
    spec = {
      refreshInterval = "1h"
      secretStoreRef = {
        name = "openbao-cluster-store"
        kind = "ClusterSecretStore"
      }
      target = {
        name           = "payments-tls-cert"
        creationPolicy = "Owner"
      }
      data = [
        { secretKey = "tls.crt", remoteRef = { key = "pki/cert/payments-tls", property = "certificate" } },
        { secretKey = "tls.key", remoteRef = { key = "pki/cert/payments-tls", property = "private_key" } },
        { secretKey = "ca.crt",  remoteRef = { key = "pki/cert/payments-tls", property = "issuing_ca" } }
      ]
    }
  }
}
