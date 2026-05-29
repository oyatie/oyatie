# api-gateway — OpenBao policy
# Per ADR-0296 (library-first credential sidecar) + ADR-0295.
path "secret/data/tls/${cell_id}/*" {
  capabilities = ["read"]
  required_parameters = ["cell_id"]
}

path "secret/data/ech/${cell_id}/*" {
  capabilities = ["read"]
}

path "secret/data/pqc/${cell_id}/*" {
  capabilities = ["read"]
}

path "secret/data/audit-signing/${cell_id}/api-gateway" {
  capabilities = ["read"]
}

path "secret/data/spire-bundle/${cell_id}" {
  capabilities = ["read"]
}

# Forbidden — gateway should NEVER read tenant-tier secrets directly
path "secret/data/tenant/+/*" {
  capabilities = ["deny"]
}
