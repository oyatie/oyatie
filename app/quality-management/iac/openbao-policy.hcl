path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/quality-management/*" {
  capabilities = ["read"]
}

path "transit/sign/quality-management-audit" {
  capabilities = ["update"]
}
