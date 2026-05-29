path "kv/data/contact-center/local-ops/*" {
  capabilities = ["read"]
}

path "transit/sign/contact-center-audit-chain" {
  capabilities = ["update"]
}

path "sys/leases/revoke-prefix/kv/contact-center/local-ops" {
  capabilities = ["update"]
}
