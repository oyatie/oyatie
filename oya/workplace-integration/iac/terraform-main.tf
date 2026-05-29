variable "tenant_scope" {
  type        = string
  description = "Workplace Integration tenant scope for terraform-main.tf"
}

resource "null_resource" "workplace_integration_terraform_main" {
  triggers = {
    service = "workplace-integration"
    adr = "ADR-0320"
    bnf_v4 = "workplace-integration.iac.terraform-main"
    layer_enum = "13-layer-enum"
  }
}
