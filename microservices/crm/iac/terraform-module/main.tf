variable "service_name" { default = "oya-crm" }
variable "http_default" { default = "HTTP/3" }
variable "ech_enabled" { default = true }
variable "pqc_hybrid" { default = "X25519MLKEM768" }

output "service_name" { value = var.service_name }
output "sap_code" { value = "CRM" }
