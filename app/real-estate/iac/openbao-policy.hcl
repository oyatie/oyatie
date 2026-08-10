path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/real-estate/*" {
  capabilities = ["read"]
}

path "transit/sign/real-estate-audit" {
  capabilities = ["update"]
}
