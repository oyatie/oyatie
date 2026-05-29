path "kv/data/financial-planning/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/financial-planning-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/financial-planning/local-ops" {
  capabilities = ["update"]
}
