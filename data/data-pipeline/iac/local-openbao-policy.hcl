path "kv/data/data-pipeline/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/data-pipeline-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/data-pipeline/local-ops" {
  capabilities = ["update"]
}
