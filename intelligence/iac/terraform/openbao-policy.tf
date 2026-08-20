# OpenBao policy for intelligence µservice
# Authority: ADR-0296 (library-first credential sidecar)
#            ADR-0255 §D-4 (provider BYOK opt-in)
# Owner: ops-security
# Path shape: ${openbao:secret/<tenant_id>/<scope>/<name>} per §3.2.2 invariant 4

terraform {
  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "~> 4.0"
    }
  }
}

# Intelligence sidecar policy: read-only on provider credentials, write on handles
resource "vault_policy" "intelligence_sidecar" {
  name = "oya-intelligence-sidecar"

  policy = <<-EOT
    # Per-tenant provider-credential BYOK (ADR-0255 §D-4)
    # Path: secret/<tenant_id>/intelligence/provider/<provider_name>
    path "secret/+/intelligence/provider/*" {
      capabilities = ["read"]
    }

    # Platform-default provider credentials (oyatie tenant)
    path "secret/oyatie/intelligence/provider/*" {
      capabilities = ["read"]
    }

    # ECH key material (intelligence service account only)
    path "secret/oyatie/intelligence/ech/*" {
      capabilities = ["read"]
    }

    # PQC certificate private keys
    path "secret/oyatie/intelligence/pqc/*" {
      capabilities = ["read"]
    }

    # Credential handle issuance (sidecar issues short-lived handles)
    # TTL ≤ 60s per ADR-0296
    path "auth/token/create/oya-intelligence-credential-handle" {
      capabilities = ["create", "update"]
    }

    # Self-renewal of sidecar token
    path "auth/token/renew-self" {
      capabilities = ["update"]
    }

    # Deny everything else by default
    path "*" {
      capabilities = ["deny"]
    }
  EOT
}

# Intelligence service account Kubernetes auth binding
resource "vault_kubernetes_auth_backend_role" "intelligence" {
  backend                          = "kubernetes"
  role_name                        = "oya-intelligence"
  bound_service_account_names      = ["oya-intelligence"]
  bound_service_account_namespaces = ["intelligence"]
  token_policies                   = [vault_policy.intelligence_sidecar.name]
  token_ttl                        = 3600   # 1 hour; sidecar renews before expiry
  token_max_ttl                    = 86400  # 24 hours maximum
  audience                         = "vault"
}

# Audit mount for intelligence sidecar access log
resource "vault_audit" "intelligence_sidecar_audit" {
  type = "file"
  path = "intelligence-sidecar/"

  options = {
    file_path   = "/var/log/vault/intelligence-sidecar-audit.log"
    log_raw     = "false"
    hmac_accessor = "true"
  }
}
