# finops-portal — OpenBao policy per ADR-0296

path "secret/data/+/finops-portal/quarterly-evidence-signing-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/finops-portal/grafana-iframe-embed-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/finops-portal/focus-export-tenant-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "secret/data/+/finops-portal/credit-ledger-signing-key" {
  capabilities = ["read"]
  required_parameters = ["tenant_id"]
  max_ttl = "60s"
}

path "sys/policies/acl/finops-portal-killswitch" {
  capabilities = ["read"]
}
