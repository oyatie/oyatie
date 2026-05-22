// Auxiliary stage-0 instances using all Always-Free E2.1.Micro x86 capacity.
// Per user direction 2026-05-16 ('get 2 E2 micro up anyways' + 'same with e2
// should be ol 10'): provision both E2.1.Micro instances Always Free includes
// to maximize free-tier coverage alongside the A1.Flex primary.
//
// Roles (assign post-provision):
//   stage0_aux_e2["bastion"]   — SSH jump host / Cloudflare tunnel endpoint
//   stage0_aux_e2["ops"]       — ops collector / cron / smoke runner / canary
//
// Both share the same VCN + public subnet + sec list as the A1 primary.

resource "oci_core_instance" "stage0_aux_e2" {
  // Roles come from var.stage0_aux_e2_roles so each tenancy can match the
  // live fleet shape (display_name + hostname_label, both immutable in OCI).
  // Skipped entirely when var.create_stage0_aux_e2 is false.
  for_each = var.create_stage0_aux_e2 ? var.stage0_aux_e2_roles : {}

  compartment_id      = oci_identity_compartment.nonprod.id
  availability_domain = coalesce(each.value.availability_domain, var.stage0_availability_domain)
  display_name        = each.value.display_name
  shape               = "VM.Standard.E2.1.Micro" // Always Free, fixed shape

  source_details {
    source_type = "image"
    source_id   = var.stage0_aux_e2_image_ocid
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.nonprod_public.id
    assign_public_ip = true
    hostname_label   = each.value.hostname_label
  }

  metadata = {
    ssh_authorized_keys = join("\n", var.ssh_authorized_keys)
    user_data           = filebase64("${path.module}/cloud-init/stage0-aux.yaml")
  }

  freeform_tags = merge(local.common_tags, {
    "role" = each.key
    "tier" = "always-free-e2"
  })

  lifecycle {
    // Metadata reconciliation is intentionally skipped (same rationale as
    // compute.tf stage0): user_data + ssh keys are immutable-in-effect; any
    // change forces destroy+recreate, which we avoid on the running fleet.
    ignore_changes = [
      freeform_tags["last-console-edit"],
      metadata,
    ]
  }
}

output "stage0_aux_e2_public_ips" {
  value = { for k, v in oci_core_instance.stage0_aux_e2 : k => v.public_ip }
}

output "stage0_aux_e2_instance_ids" {
  value = { for k, v in oci_core_instance.stage0_aux_e2 : k => v.id }
}
