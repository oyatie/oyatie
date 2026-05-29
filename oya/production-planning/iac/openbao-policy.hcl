path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/production-planning/*" {
  capabilities = ["read"]
}

path "transit/sign/production-planning-audit" {
  capabilities = ["update"]
}
