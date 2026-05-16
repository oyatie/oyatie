// Always-Free E2.1.Micro fleet — both free micros provisioned with explicit
// roles via per-role cloud-init templates. Authority: user directive 2026-05-16
// ("rust is the default + tofu preferred + everything should be declarative").
//
// Roles:
//   vpn       — Headscale tailnet control plane + DERP relay (public 443 + 3478)
//   watchdog  — blackbox_exporter + oya-cloud-watchdog (Rust) — pages OCI
//               Notifications on on-prem outage. Tailnet-only.
//
// Both share the same VCN + public subnet + default security list. The vpn
// role attaches an additional NSG (declared in headscale.tf) that opens
// UDP/3478 for DERP STUN.
//
// Memory budget per VM (1 GB E2.1.Micro):
//   vpn       — Headscale ~70 MB + DERP embedded + node_exporter ~30 MB
//                = ~100 MB committed, ~600 MB headroom
//   watchdog  — blackbox_exporter ~20 MB + oya-cloud-watchdog ~30 MB
//                + node_exporter ~30 MB = ~80 MB committed, ~620 MB headroom
//
// Headroom on each VM is reserved for SPIFFE issuer / step-ca (M3-P04) and
// future small-footprint workloads. NO heavy workloads (Prometheus/Loki/
// Grafana) ever land on these micros — those live in the on-prem k8s cluster
// where there's real CPU + ZFS bulk storage. See ADR-0121.

locals {
  // OL 10.1 x86_64 image — kept for parity with the previous fleet.
  e2_micro_image_ocid = "ocid1.image.oc1.ap-chuncheon-1.aaaaaaaa7dt7pyhhltpw2lfpgqvfhwy3b3g6jbbzm3vh5ag3masvvd2bo6ia"

  e2_micro_roles = {
    vpn = {
      hostname_label    = "vpn-kr-01"
      display_name      = "oyatie-vpn-kr-01"
      cloud_init_path   = "${path.module}/cloud-init/role-vpn.yaml.tftpl"
      additional_nsg_id = oci_core_network_security_group.vpn.id
      cloud_init_vars = {
        headscale_version = "0.27.0"
        headscale_domain  = "vpn.${var.cloudflare_domain}"
        server_url        = "https://vpn.${var.cloudflare_domain}"
      }
    }
    watchdog = {
      hostname_label    = "watchdog-kr-01"
      display_name      = "oyatie-watchdog-kr-01"
      cloud_init_path   = "${path.module}/cloud-init/role-watchdog.yaml.tftpl"
      additional_nsg_id = null
      cloud_init_vars = {
        notifications_topic_id = oci_ons_notification_topic.ops_alerts.topic_id
        watchdog_targets       = "https://foundry.${var.cloudflare_domain}/api/health,https://api.${var.cloudflare_domain}/workspace/api/v1/health,https://ops.${var.cloudflare_domain}/api/health"
        poll_interval_seconds  = "60"
        failure_threshold      = "3"
      }
    }
  }
}

resource "oci_core_instance" "e2_micro" {
  for_each = local.e2_micro_roles

  compartment_id      = oci_identity_compartment.nonprod.id
  availability_domain = var.stage0_availability_domain
  display_name        = each.value.display_name
  shape               = "VM.Standard.E2.1.Micro" // fixed Always-Free shape

  source_details {
    source_type = "image"
    source_id   = local.e2_micro_image_ocid
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.nonprod_public.id
    assign_public_ip = true
    hostname_label   = each.value.hostname_label
    nsg_ids          = each.value.additional_nsg_id == null ? [] : [each.value.additional_nsg_id]
  }

  metadata = {
    ssh_authorized_keys = join("\n", var.ssh_authorized_keys)
    user_data           = base64encode(templatefile(each.value.cloud_init_path, each.value.cloud_init_vars))
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

output "e2_micro_public_ips" {
  value = { for k, v in oci_core_instance.e2_micro : k => v.public_ip }
}

output "e2_micro_instance_ids" {
  value = { for k, v in oci_core_instance.e2_micro : k => v.id }
}
