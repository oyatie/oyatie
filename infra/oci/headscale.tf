// Headscale tailnet control plane — companion resources for the vpn-role
// E2.1.Micro provisioned by compute-aux.tf (oci_core_instance.e2_micro["vpn"]).
//
// This file declares only the NSG + reserved public IP + Cloudflare DNS
// glue specific to Headscale. The instance itself is in compute-aux.tf so
// the always-free-micro fleet stays in one place.
//
// Authority: ADR-0121 (Headscale tailnet control plane on OCI free tier).

# ---- NSG: Headscale-specific ingress (UDP/3478 DERP STUN) ----
# TCP 22/80/443 are already open via the default security list.
resource "oci_core_network_security_group" "vpn" {
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-vpn-nsg"
  freeform_tags  = local.common_tags
}

resource "oci_core_network_security_group_security_rule" "vpn_ingress_3478_udp" {
  network_security_group_id = oci_core_network_security_group.vpn.id
  direction                 = "INGRESS"
  protocol                  = "17" // UDP
  source_type               = "CIDR_BLOCK"
  source                    = "0.0.0.0/0"
  udp_options {
    destination_port_range {
      min = 3478
      max = 3478
    }
  }
  description = "Headscale DERP STUN — peer NAT traversal"
}

resource "oci_core_network_security_group_security_rule" "vpn_egress_all" {
  network_security_group_id = oci_core_network_security_group.vpn.id
  direction                 = "EGRESS"
  protocol                  = "all"
  destination_type          = "CIDR_BLOCK"
  destination               = "0.0.0.0/0"
  description               = "All egress (cert renewal, DERP relays, package mirrors)"
}

# ---- Outputs consumed by Cloudflare DNS (vpn.oyatie.com → vpn micro IP) ----
output "vpn_public_ip" {
  value       = oci_core_instance.e2_micro["vpn"].public_ip
  description = "Headscale public IP — wire into Cloudflare DNS A record for vpn.oyatie.com"
}

output "vpn_instance_id" {
  value = oci_core_instance.e2_micro["vpn"].id
}

output "vpn_hostname" {
  value = "vpn.${var.cloudflare_domain}"
}
