# compliance µservice — OpenBao policy
# Binding: ADR-0296 (library-first credential sidecar; ≤60s TTL plaintext)
# Path convention: ${openbao:secret/<tenant_id>/compliance/<name>}

path "secret/data/+/compliance/signing-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
  min_wrapping_ttl = "30s"
  max_wrapping_ttl = "60s"
}

path "secret/data/+/compliance/seal-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/compliance/dsar-encryption-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/metadata/+/compliance/*" {
  capabilities = ["list"]
}

# Kill-switch — bootstrap-tier-1 per ADR-0295
path "sys/policies/acl/compliance-killswitch" {
  capabilities = ["read"]
}
