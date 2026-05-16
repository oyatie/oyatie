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

locals {
  stage0_aux_e2_image_ocid = "ocid1.image.oc1.ap-chuncheon-1.aaaaaaaa7dt7pyhhltpw2lfpgqvfhwy3b3g6jbbzm3vh5ag3masvvd2bo6ia" // OL 10.1 x86_64 2026-04-30-3
}

resource "oci_core_instance" "stage0_aux_e2" {
  for_each = toset(["bastion", "ops"])

  compartment_id      = oci_identity_compartment.nonprod.id
  availability_domain = var.stage0_availability_domain
  display_name        = "oyatie-stage0-e2-${each.key}"
  shape               = "VM.Standard.E2.1.Micro" // Always Free, fixed shape

  source_details {
    source_type = "image"
    source_id   = local.stage0_aux_e2_image_ocid
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.nonprod_public.id
    assign_public_ip = true
    hostname_label   = "oyatie-e2-${each.key}"
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
    ignore_changes = [
      freeform_tags["last-console-edit"],
    ]
  }
}

output "stage0_aux_e2_public_ips" {
  value = { for k, v in oci_core_instance.stage0_aux_e2 : k => v.public_ip }
}

output "stage0_aux_e2_instance_ids" {
  value = { for k, v in oci_core_instance.stage0_aux_e2 : k => v.id }
}
