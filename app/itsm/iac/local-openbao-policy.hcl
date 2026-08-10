path "kv/data/itsm/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/itsm-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/itsm/local-ops" {
  capabilities = ["update"]
}
