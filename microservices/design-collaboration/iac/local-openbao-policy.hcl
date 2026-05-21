path "kv/data/design-collaboration/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/design-collaboration-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/design-collaboration/local-ops" {
  capabilities = ["update"]
}
