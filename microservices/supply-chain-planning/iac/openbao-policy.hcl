path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/supply-chain-planning/*" {
  capabilities = ["read"]
}

path "transit/sign/supply-chain-planning-audit" {
  capabilities = ["update"]
}
