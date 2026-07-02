locals {
  module_version     = "v0.1.0"
  module_path        = "cloud/cloud-iac/modules/aws-guest/sg-baseline"
  release_identifier = "cloud-iac/modules/aws-guest/sg-baseline/v0.1.0"

  observability_labels = {
    tenant_id       = var.tenant_id
    tenant_class    = var.tenant_class
    cell_id         = var.cell_id
    compliance_pack = var.compliance_pack
    module_version  = local.module_version
    service         = var.service_name
  }

  ingress_rules = []

  egress_rules = flatten([
    for cidr in var.allowed_egress_cidrs : [
      for port in var.allowed_egress_tcp_ports : {
        protocol    = "tcp"
        from_port   = port
        to_port     = port
        cidr_blocks = [cidr]
        description = "ADR-0339 canonical egress allow-list for ${var.service_name}"
      }
    ]
  ])

  security_group_baseline = {
    context                   = "aws-guest"
    primitive                 = "sg-baseline"
    release_identifier        = local.release_identifier
    tenant_id                 = var.tenant_id
    tenant_class              = var.tenant_class
    service_name              = var.service_name
    ingress_policy            = "deny-by-default"
    egress_policy             = "explicit-allow-list"
    ingress_rules             = local.ingress_rules
    egress_rules              = local.egress_rules
    required_labels           = local.observability_labels
    cosign_attestation_digest = var.cosign_attestation_digest
  }
}
