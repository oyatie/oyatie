// Sub-compartments under root tenancy. Already exist live — import via:
//   tofu import oci_identity_compartment.foundry <OCID>
//   tofu import oci_identity_compartment.cloud   <OCID>
//   tofu import oci_identity_compartment.prod    <OCID>
//   tofu import oci_identity_compartment.nonprod <OCID>

resource "oci_identity_compartment" "foundry" {
  compartment_id = local.tenancy_ocid
  name           = "foundry"
  description    = "Oyatie foundry compartment (M02 substrate)"
  enable_delete  = true
  freeform_tags  = local.common_tags
}

resource "oci_identity_compartment" "cloud" {
  compartment_id = local.tenancy_ocid
  name           = "cloud"
  description    = "Oyatie cloud compartment (M02 substrate)"
  enable_delete  = true
  freeform_tags  = local.common_tags
}

resource "oci_identity_compartment" "prod" {
  compartment_id = local.tenancy_ocid
  name           = "prod"
  description    = "Oyatie prod compartment (M02 substrate)"
  enable_delete  = true
  freeform_tags  = local.common_tags
}

resource "oci_identity_compartment" "nonprod" {
  compartment_id = local.tenancy_ocid
  name           = "nonprod"
  description    = "Oyatie nonprod compartment (M02 substrate)"
  enable_delete  = true
  freeform_tags  = local.common_tags
}
