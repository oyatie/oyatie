variable "tenant_id" { type = string }
variable "compartment_ocid" { type = string }
variable "namespace" { type = string }

resource "oci_objectstorage_bucket" "diagnostics_lab_pathology_sandbox" {
  compartment_id = var.compartment_ocid
  namespace      = var.namespace
  name           = "oya-diagnostics-lab-pathology-sandbox-${var.tenant_id}"
  access_type    = "NoPublicAccess"
}

output "always_free_bucket" {
  value = oci_objectstorage_bucket.diagnostics_lab_pathology_sandbox.name
}
