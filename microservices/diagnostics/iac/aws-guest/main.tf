variable "tenant_id" { type = string }
variable "cell_id" { type = string }

resource "aws_s3_bucket" "diagnostics_evidence" {
  bucket = "oya-diagnostics-evidence-${var.tenant_id}-${var.cell_id}"
}

resource "aws_kms_key" "diagnostics_evidence" {
  description             = "KMS key for diagnostics lab/pathology evidence"
  deletion_window_in_days = 30
  enable_key_rotation     = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "diagnostics_evidence" {
  bucket = aws_s3_bucket.diagnostics_evidence.id

  rule {
    apply_server_side_encryption_by_default {
      kms_master_key_id = aws_kms_key.diagnostics_evidence.arn
      sse_algorithm     = "aws:kms"
    }
  }
}

output "diagnostics_evidence_bucket" {
  value = aws_s3_bucket.diagnostics_evidence.bucket
}

output "diagnostics_evidence_kms_key_arn" {
  value = aws_kms_key.diagnostics_evidence.arn
}
