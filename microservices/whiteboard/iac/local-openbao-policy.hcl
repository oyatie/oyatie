path "kv/data/whiteboard/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/whiteboard-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/whiteboard/local-ops" {
  capabilities = ["update"]
}
