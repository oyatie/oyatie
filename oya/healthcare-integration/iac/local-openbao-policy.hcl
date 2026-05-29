path "kv/data/healthcare-integration/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/healthcare-integration-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/healthcare-integration/local-ops" {
  capabilities = ["update"]
}
