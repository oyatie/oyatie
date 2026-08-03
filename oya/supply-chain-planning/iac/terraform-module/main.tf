# Claim boundary: target/provenance inventory only under the preview PRD.
# Do not run terraform/tofu apply from this stub; it carries no cloud activation,
# tenant namespace readiness, DR/SLO readiness, runtime audit-chain emission, or GA claim.
variable "service_name" { default = "oya-supply-chain-planning" }
variable "http_default" { default = "HTTP/3" }
variable "ech_enabled" { default = true }
variable "pqc_hybrid" { default = "X25519MLKEM768" }

output "service_name" { value = var.service_name }
output "sap_code" { value = "SCM/APO" }
