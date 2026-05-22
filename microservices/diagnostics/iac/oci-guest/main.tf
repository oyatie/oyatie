variable "tenant_id" { type = string }
variable "cell_id" { type = string }
variable "compartment_ocid" { type = string }
variable "namespace" { type = string }

resource "oci_objectstorage_bucket" "diagnostics_evidence" {
  compartment_id = var.compartment_ocid
  namespace      = var.namespace
  name           = "oya-diagnostics-evidence-${var.tenant_id}-${var.cell_id}"
  access_type    = "NoPublicAccess"
}

resource "oci_kms_vault" "diagnostics_vault" {
  compartment_id = var.compartment_ocid
  display_name   = "diagnostics-evidence-${var.tenant_id}"
  vault_type     = "DEFAULT"
}

resource "oci_kms_key" "diagnostics_key" {
  compartment_id      = var.compartment_ocid
  display_name        = "diagnostics-evidence-${var.tenant_id}"
  management_endpoint = oci_kms_vault.diagnostics_vault.management_endpoint

  key_shape {
    algorithm = "AES"
    length    = 32
  }
}

output "diagnostics_evidence_bucket" {
  value = oci_objectstorage_bucket.diagnostics_evidence.name
}

output "diagnostics_evidence_kms_key_ocid" {
  value = oci_kms_key.diagnostics_key.id
}
