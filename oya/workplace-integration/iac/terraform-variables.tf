variable "tenant_scope" {
  type        = string
  description = "Workplace Integration tenant scope for terraform-variables.tf"
}

resource "null_resource" "workplace_integration_terraform_variables" {
  triggers = {
    service = "workplace-integration"
    adr = "ADR-0320"
    bnf_v4 = "workplace-integration.iac.terraform-variables"
    layer_enum = "13-layer-enum"
  }
}
