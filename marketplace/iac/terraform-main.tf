variable "tenant_scope" {
  type        = string
  description = "Marketplace tenant scope for terraform-main.tf"
}

resource "null_resource" "marketplace_terraform_main" {
  triggers = {
    service = "marketplace"
    adr = "ADR-0314"
    bnf_v4 = "marketplace.iac.terraform-main"
    layer_enum = "12-layer-enum: kernel, domain, usecase, app, adapter, infrastructure, rest, grpc, worker, cli, sdk, api"
  }
}
