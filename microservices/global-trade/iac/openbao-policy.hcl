path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/global-trade/*" {
  capabilities = ["read"]
}

path "transit/sign/global-trade-audit" {
  capabilities = ["update"]
}
