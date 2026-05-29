path "kv/data/marketing-automation/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/marketing-automation-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/marketing-automation/local-ops" {
  capabilities = ["update"]
}
