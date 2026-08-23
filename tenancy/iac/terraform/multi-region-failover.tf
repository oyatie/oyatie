# tenancy µservice — Multi-region failover Terraform
# Per ADR-0248 + multi-region.md + IP-019 DR-pairing controller

variable "regions" {
  type    = list(string)
  default = ["us-east-1", "us-west-2", "eu-central-1", "eu-west-1", "ap-northeast-2", "ap-southeast-1"]
}

variable "dr_pairs" {
  type = map(string)
  default = {
    "us-east-1"      = "us-west-2"
    "us-west-2"      = "us-east-1"
    "eu-central-1"   = "eu-west-1"
    "eu-west-1"      = "eu-central-1"
    "ap-northeast-2" = "ap-southeast-1"
    "ap-southeast-1" = "ap-northeast-2"
  }
}

resource "kubernetes_config_map" "dr_pairing" {
  for_each = toset(var.regions)
  metadata {
    name      = "tenancy-dr-pairing-${each.value}"
    namespace = "tenancy"
  }
  data = {
    home_region = each.value
    dr_region   = var.dr_pairs[each.value]
    rpo_seconds = "30"
    rto_seconds = "300"
  }
}
