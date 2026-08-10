path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/plant-maintenance/*" {
  capabilities = ["read"]
}

path "transit/sign/plant-maintenance-audit" {
  capabilities = ["update"]
}
