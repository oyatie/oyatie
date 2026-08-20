# tenancy µservice — OpenBao policy per ADR-0296
# Path convention: ${openbao:secret/<tenant_id>/tenancy/<name>}

path "secret/data/+/tenancy/rls-jwt-signing-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/tenancy/kyb-kyc-encryption-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/tenancy/dr-coord-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "sys/policies/acl/tenancy-killswitch" {
  capabilities = ["read"]
}
