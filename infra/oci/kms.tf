// KMS vault + master key in cloud compartment.
// Live resources already created via CLI; import with:
//   tofu import oci_kms_vault.cloud_default <vault-ocid>
//   tofu import oci_kms_key.cloud_master    "<vault-mgmt-endpoint>/<key-ocid>"

resource "oci_kms_vault" "cloud_default" {
  compartment_id = oci_identity_compartment.cloud.id
  display_name   = "bitween-default-vault"
  vault_type     = "DEFAULT"
  freeform_tags  = local.common_tags
}

resource "oci_kms_key" "cloud_master" {
  compartment_id      = oci_identity_compartment.cloud.id
  display_name        = "oyatie-cloud-master-key"
  management_endpoint = oci_kms_vault.cloud_default.management_endpoint
  protection_mode     = "SOFTWARE"
  freeform_tags       = local.common_tags

  key_shape {
    algorithm = "AES"
    length    = 32
  }
}
