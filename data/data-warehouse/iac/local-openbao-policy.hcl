path "kv/data/data-warehouse/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/data-warehouse-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/data-warehouse/local-ops" {
  capabilities = ["update"]
}
