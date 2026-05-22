# comms-email µservice — OpenBao policy per ADR-0296
# Path: ${openbao:secret/<tenant_id>/comms-email/<name>}

path "secret/data/+/comms-email/dkim-private-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/comms-email/ses-credentials" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/comms-email/mailgun-api-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/comms-email/postal-server-secret" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "sys/policies/acl/comms-email-killswitch" {
  capabilities = ["read"]
}
