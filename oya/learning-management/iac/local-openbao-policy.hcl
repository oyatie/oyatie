path "kv/data/learning-management/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/learning-management-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/learning-management/local-ops" {
  capabilities = ["update"]
}
