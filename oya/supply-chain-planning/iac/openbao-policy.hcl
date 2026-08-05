# Claim boundary: target/provenance inventory only; do not write this policy to
# OpenBao from this tree without separate activation evidence and review.
path "secret/data/{{identity.entity.aliases.auth_kubernetes_*.metadata.service_account_namespace}}/supply-chain-planning/*" {
  capabilities = ["read"]
}

path "transit/sign/supply-chain-planning-audit" {
  capabilities = ["update"]
}
