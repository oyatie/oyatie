path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/warehouse/*" {
  capabilities = ["read"]
}

path "transit/sign/warehouse-audit" {
  capabilities = ["update"]
}
