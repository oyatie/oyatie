terraform {
  required_version = ">= 1.8.0"
}

variable "tenant_id" { type = string }
variable "tenant_class" { type = string }
variable "cell_id" { type = string }
variable "compliance_pack" {
  type    = string
  default = "soc2"
}
variable "allowed_egress_cidrs" {
  type    = list(string)
  default = ["10.0.0.0/8"]
}

module "aws_guest_sg_baseline" {
  source = "../../../../iac/modules/aws-guest/sg-baseline"

  tenant_id                = var.tenant_id
  tenant_class             = var.tenant_class
  cell_id                  = var.cell_id
  service_name             = "cloud-billing"
  compliance_pack          = var.compliance_pack
  allowed_egress_cidrs     = var.allowed_egress_cidrs
  allowed_egress_tcp_ports = [443]
}

output "aws_guest_security_group_baseline" {
  value = module.aws_guest_sg_baseline.security_group_baseline
}
