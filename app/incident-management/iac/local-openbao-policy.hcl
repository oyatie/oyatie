path "kv/data/incident-management/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/incident-management-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/incident-management/local-ops" {
  capabilities = ["update"]
}
