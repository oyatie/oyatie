variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "pack_overlay" {
  type    = list(string)
  default = ["HIPAA-2024", "CLIA", "CAP"]
}

resource "minio_s3_bucket" "diagnostics_evidence" {
  bucket = "oya-diagnostics-evidence-${var.tenant_id}-${var.cell_id}"
  acl    = "private"
}

locals {
  diagnostics_on_prem_profile = {
    tenant_id       = var.tenant_id
    cell_id         = var.cell_id
    evidence_bucket = minio_s3_bucket.diagnostics_evidence.bucket
    packs           = var.pack_overlay
  }
}

output "diagnostics_evidence_bucket" {
  value = minio_s3_bucket.diagnostics_evidence.bucket
}

output "diagnostics_on_prem_profile" {
  value = local.diagnostics_on_prem_profile
}
