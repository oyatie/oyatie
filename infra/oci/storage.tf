// Object Storage cold-backup bucket for audit-chain mirror.
// Primary large-storage stays on-prem (ZFS bulk tier ~3.5 TB); OCI bucket
// is Archive-tier off-site cold backup only.
// Live bucket already created; import with:
//   tofu import oci_objectstorage_bucket.audit_cold_backup <namespace>/<bucket-name>

data "oci_objectstorage_namespace" "tenancy" {
  compartment_id = local.tenancy_ocid
}

resource "oci_objectstorage_bucket" "audit_cold_backup" {
  compartment_id = oci_identity_compartment.foundry.id
  namespace      = data.oci_objectstorage_namespace.tenancy.namespace
  name           = "oyatie-audit-cold-backup"
  access_type    = "NoPublicAccess"
  storage_tier   = "Archive"
  versioning     = "Disabled"
  freeform_tags  = local.common_tags
}
