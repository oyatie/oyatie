path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/crm/*" {
  capabilities = ["read"]
}

path "transit/sign/crm-audit" {
  capabilities = ["update"]
}
