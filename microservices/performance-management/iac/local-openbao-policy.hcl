path "kv/data/performance-management/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/performance-management-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/performance-management/local-ops" {
  capabilities = ["update"]
}
