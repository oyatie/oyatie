# Preview planning inventory only; not a live OpenBao policy binding.
path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/treasury/*" {
  capabilities = ["read"]
}

path "transit/sign/treasury-audit" {
  capabilities = ["update"]
}
