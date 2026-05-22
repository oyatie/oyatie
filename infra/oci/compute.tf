// Stage-0 ARM instance.
// Capacity is contested in ap-chuncheon-1. Shape selection is desired state:
// change var.stage0_shape / var.stage0_ocpus / var.stage0_memory_gbs and run
// OpenTofu through the root Makefile. Do not resize with OCI CLI or console
// edits; the deployment ops contract forbids manual mutation paths.

resource "oci_core_instance" "stage0" {
  count = var.create_stage0_a1 ? 1 : 0

  compartment_id      = oci_identity_compartment.nonprod.id
  availability_domain = var.stage0_availability_domain
  display_name        = var.stage0_display_name
  shape               = var.stage0_shape

  // shape_config only applies to .Flex shapes. E2.1.Micro is a fixed shape
  // (1/8 OCPU + 1 GB) and rejects shape_config blocks.
  dynamic "shape_config" {
    for_each = endswith(var.stage0_shape, ".Flex") ? [1] : []
    content {
      ocpus         = var.stage0_ocpus
      memory_in_gbs = var.stage0_memory_gbs
    }
  }

  source_details {
    source_type = "image"
    source_id   = var.stage0_image_ocid
  }

  create_vnic_details {
    subnet_id        = var.stage0_use_private_subnet ? oci_core_subnet.nonprod_private.id : oci_core_subnet.nonprod_public.id
    assign_public_ip = var.stage0_use_private_subnet ? false : true
    hostname_label   = var.stage0_hostname_label
  }

  metadata = {
    ssh_authorized_keys = join("\n", var.ssh_authorized_keys)
    user_data           = filebase64("${path.module}/cloud-init/stage0.yaml")
  }

  freeform_tags = local.common_tags

  // Shape and tag drift are OpenTofu-owned. If capacity requires a temporary
  // paid shape, encode that desired state in tfvars and converge via
  // `make apply`; do not preserve manual console/CLI drift here.

  lifecycle {
    // Metadata (user_data, ssh_authorized_keys) on a running instance is
    // intentionally NOT reconciled — changing it forces destroy+recreate of
    // the A1, risking loss of the contested A1.Flex host slot. Update via
    // a deliberate stop + relaunch when the user_data needs to change.
    ignore_changes = [metadata]
  }
}
