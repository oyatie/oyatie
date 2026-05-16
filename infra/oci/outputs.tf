output "compartment_ocids" {
  value = {
    foundry = oci_identity_compartment.foundry.id
    cloud   = oci_identity_compartment.cloud.id
    prod    = oci_identity_compartment.prod.id
    nonprod = oci_identity_compartment.nonprod.id
  }
}

output "nonprod_vcn_id" {
  value = oci_core_vcn.nonprod.id
}

output "nonprod_subnet_id" {
  value = oci_core_subnet.nonprod_public.id
}

output "kms_vault_id" {
  value = oci_kms_vault.cloud_default.id
}

output "kms_vault_management_endpoint" {
  value = oci_kms_vault.cloud_default.management_endpoint
}

output "kms_master_key_id" {
  value = oci_kms_key.cloud_master.id
}

output "audit_cold_backup_bucket" {
  value = "${data.oci_objectstorage_namespace.tenancy.namespace}/${oci_objectstorage_bucket.audit_cold_backup.name}"
}

output "stage0_instance_id" {
  value = length(oci_core_instance.stage0) > 0 ? oci_core_instance.stage0[0].id : null
}

output "stage0_public_ip" {
  value = length(oci_core_instance.stage0) > 0 ? oci_core_instance.stage0[0].public_ip : null
}
