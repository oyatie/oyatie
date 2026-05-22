// bominal-oci.tfvars — OpenTofu inputs for the bominal-oci tenancy (us-ashburn-1).
//
// Apply pattern (workspace-scoped):
//   tofu workspace select bominal-oci
//   tofu plan  -var-file=bominal-oci.tfvars
//   tofu apply -var-file=bominal-oci.tfvars
//
// This tenancy is Always Free; always_free_mode=true disables NAT Gateway and
// Service Gateway (AF cap = 0 per VCN in us-ashburn-1). Flip the flag to false
// after upgrading to PAYG to get the full topology in one apply.

tenancy_ocid       = "ocid1.tenancy.oc1..aaaaaaaakdaslkhvri7nkvyvgenzlxqaoqt4gevibkegsnluovx3yr5b4lhq"
region             = "us-ashburn-1"
oci_config_profile = "bominal-oci"

// SSH key currently authorized on the live oyatie A1 instance. Replace or extend
// as the access-control posture evolves; never commit private keys here.
ssh_authorized_keys = [
  "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ6dcjCZ33c4wU1XaGXLhvDjdabGAQ1YZelM5L37AUwP oyatie@onprem-2026-05-16"
]

// Boot images — match the live instances' current images so import doesn't
// flag a ForceNew on source_id. A1 = aarch64; E2 = x86_64.
stage0_image_ocid          = "ocid1.image.oc1.iad.aaaaaaaa4c6yebvwdyv44ggynzkiq4ostfomzkwkadnvy3yxbkwwkhq32caa"
stage0_aux_e2_image_ocid   = "ocid1.image.oc1.iad.aaaaaaaalkf4mkx2efw7xghafasr2ehia42ombnybkbmejjtvfa6nd3yttfa"
stage0_availability_domain = "XPYE:US-ASHBURN-AD-1"

// Full Always-Free A1 envelope.
stage0_shape      = "VM.Standard.A1.Flex"
stage0_ocpus      = 4
stage0_memory_gbs = 24

// Existing A1 ("oyatie") attributes — must match live exactly so Tofu's
// import + plan shows only the compartment move, not a forced replacement.
// hostname_label is immutable in OCI; "bominal-app" is what the bootstrap
// created and cannot be changed without recreating the instance.
create_stage0_a1          = true
stage0_display_name       = "oyatie"
stage0_hostname_label     = "bominal-app"
stage0_use_private_subnet = true

// bominal-oci already hosts a single E2.1.Micro named monitoring (the
// auxiliary role), not the bitween bastion+ops pair. Map the single role so
// for_each matches the live instance.
create_stage0_aux_e2 = true
stage0_aux_e2_roles = {
  // The live E2 was launched in AD-3, not AD-1 like the A1. Override per-role
  // to avoid a forced replacement on import.
  monitoring = {
    display_name        = "oyatie-stage0-e2-monitoring"
    hostname_label      = "bominal-monitoring"
    availability_domain = "XPYE:US-ASHBURN-AD-3"
  }
}

// Gate NAT GW + SGW + private-subnet routes. AF tenancy has zero capacity for
// these; without this flag, plan would attempt to create them and fail at API.
always_free_mode = true

// DNS labels are immutable in OCI; these match what the live VCN + subnets
// were created with by the bominal-app bootstrap. Without these overrides
// Tofu would see a dns_label diff and try to destroy+recreate the VCN, which
// would cascade-delete the running oyatie A1 + oyatie-stage0-e2-monitoring.
vcn_dns_label            = "bominalvcn"
subnet_public_dns_label  = "pubsubnet"
subnet_private_dns_label = "privsubnet"

// Budget alerts — empty by default keeps alerts in the console only. To email,
// set to "ops@example.com" or comma-separated list. Always-Free spend = $0, so
// any alert here is signal of misconfiguration.
budget_alert_recipients = ""
