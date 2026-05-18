# OCI Terraform for foundry-evidence cross-µservice IAM grants.
#
# foundry-evidence READS from the audit-chain-owned WORM bucket and writes to
# its own dead-letter + regulator-export-staging buckets. No write access to
# substrate WORM (per ADR-0131 substrate split; the bridge does it via emit API,
# not direct S3 write).

variable "pack" {
  type        = string
  description = "Pack identifier (pack-kr, pack-eu, etc.)"
}

variable "pack_region" {
  type        = string
  description = "OCI region for this pack"
}

variable "tenancy_ocid" {
  type        = string
  description = "Root tenancy OCID"
}

# Compartment for foundry-evidence within the pack
data "oci_identity_compartment" "foundry_evidence" {
  id = "ocid1.compartment.${var.pack}.foundry-evidence"
}

# Substrate WORM bucket (owned by audit-chain µservice; we only reference it)
data "oci_objectstorage_bucket" "audit_chain_worm" {
  name      = "audit-chain-worm-${var.pack}"
  namespace = "oyatie"
}

# Dynamic group that includes the foundry-evidence SPIFFE-bound workload identity
resource "oci_identity_dynamic_group" "foundry_evidence_workload" {
  compartment_id = var.tenancy_ocid
  name           = "foundry-evidence-workload-${var.pack}"
  description    = "Workload identities for foundry-evidence in ${var.pack}"
  matching_rule  = "ALL { resource.compartment.id = '${data.oci_identity_compartment.foundry_evidence.id}', tag.oyatie.microservice.value = 'foundry-evidence' }"
}

# Policy: foundry-evidence READS the audit-chain WORM bucket
resource "oci_identity_policy" "foundry_evidence_read_substrate_worm" {
  compartment_id = var.tenancy_ocid
  name           = "foundry-evidence-read-substrate-worm-${var.pack}"
  description    = "Allow foundry-evidence to read audit-chain WORM bucket (no write)"
  statements = [
    "Allow dynamic-group ${oci_identity_dynamic_group.foundry_evidence_workload.name} to read objects in compartment audit-chain where target.bucket.name = '${data.oci_objectstorage_bucket.audit_chain_worm.name}'",
    "Allow dynamic-group ${oci_identity_dynamic_group.foundry_evidence_workload.name} to inspect buckets in compartment audit-chain where target.bucket.name = '${data.oci_objectstorage_bucket.audit_chain_worm.name}'"
  ]
}

# foundry-evidence-owned dead-letter bucket
resource "oci_objectstorage_bucket" "deadletter" {
  compartment_id = data.oci_identity_compartment.foundry_evidence.id
  namespace      = "oyatie"
  name           = "foundry-evidence-deadletter-${var.pack}"
  storage_tier   = "Standard"
  access_type    = "NoPublicAccess"
  versioning     = "Disabled"

  kms_key_id = oci_kms_key.foundry_evidence.id

  object_lifecycle_policy_details {
    rules {
      name        = "dead-letter-expire-30d"
      action      = "DELETE"
      time_amount = 30
      time_unit   = "DAYS"
      is_enabled  = true
    }
  }
}

# foundry-evidence-owned regulator-export staging bucket (NOT WORM; ephemeral)
resource "oci_objectstorage_bucket" "regulator_export_staging" {
  compartment_id = data.oci_identity_compartment.foundry_evidence.id
  namespace      = "oyatie"
  name           = "foundry-evidence-regulator-export-staging-${var.pack}"
  storage_tier   = "Standard"
  access_type    = "NoPublicAccess"
  versioning     = "Disabled"

  kms_key_id = oci_kms_key.foundry_evidence.id

  object_lifecycle_policy_details {
    rules {
      name        = "staging-expire-7d"
      action      = "DELETE"
      time_amount = 7
      time_unit   = "DAYS"
      is_enabled  = true
    }
  }
}

resource "oci_kms_key" "foundry_evidence" {
  compartment_id      = data.oci_identity_compartment.foundry_evidence.id
  display_name        = "foundry-evidence-${var.pack}"
  key_shape {
    algorithm = "AES"
    length    = 32
  }
  management_endpoint = "ocid1.kms.${var.pack_region}.management"
}

output "deadletter_bucket" {
  value = oci_objectstorage_bucket.deadletter.name
}

output "regulator_export_staging_bucket" {
  value = oci_objectstorage_bucket.regulator_export_staging.name
}
