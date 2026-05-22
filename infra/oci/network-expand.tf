// Additional network primitives — all Always Free.
// Per user directive 2026-05-16 ('make sure everything else is setup first
// before you try a1 again. like vnic and stuff' + 'everything done through
// opentofu'). Adds: NAT GW, Service GW, NSGs per role, private subnet.

# ---- NAT Gateway (private-subnet outbound; NOT Always-Free-eligible) ----
# AF tenancies cap NAT gateways at 0 per region. Created only when
# var.always_free_mode is false (PAYG tier).
resource "oci_core_nat_gateway" "nonprod" {
  count          = var.always_free_mode ? 0 : 1
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nonprod-nat"
  freeform_tags  = local.common_tags
}

# ---- Service Gateway (OCI service access without internet; NOT AF-eligible) ----
# AF tenancies cap Service Gateways at 0. Created only on PAYG.
data "oci_core_services" "all_oci_services" {
  count = var.always_free_mode ? 0 : 1

  filter {
    name   = "name"
    values = ["All .* Services In Oracle Services Network"]
    regex  = true
  }
}

resource "oci_core_service_gateway" "nonprod" {
  count          = var.always_free_mode ? 0 : 1
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nonprod-svcgw"

  services {
    service_id = data.oci_core_services.all_oci_services[0].services[0].id
  }

  freeform_tags = local.common_tags
}

# ---- Private subnet (for future private-only workloads) ----
resource "oci_core_route_table" "nonprod_private" {
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nonprod-private-rt"
  freeform_tags  = local.common_tags

  // On Always Free the private route table is intentionally empty —
  // no NAT/SGW exist, so private workloads have no internet egress and
  // must use OCI Bastion (free 5 sessions) for managed access. Routes
  // are added only when the PAYG-tier NAT/SGW resources exist.
  dynamic "route_rules" {
    for_each = var.always_free_mode ? [] : [1]
    content {
      destination       = "0.0.0.0/0"
      destination_type  = "CIDR_BLOCK"
      network_entity_id = oci_core_nat_gateway.nonprod[0].id
    }
  }
  dynamic "route_rules" {
    for_each = var.always_free_mode ? [] : [1]
    content {
      destination       = data.oci_core_services.all_oci_services[0].services[0].cidr_block
      destination_type  = "SERVICE_CIDR_BLOCK"
      network_entity_id = oci_core_service_gateway.nonprod[0].id
    }
  }
}

resource "oci_core_subnet" "nonprod_private" {
  compartment_id             = oci_identity_compartment.nonprod.id
  vcn_id                     = oci_core_vcn.nonprod.id
  display_name               = "oyatie-nonprod-private-subnet"
  cidr_block                 = "10.0.2.0/24"
  dns_label                  = var.subnet_private_dns_label
  prohibit_public_ip_on_vnic = true
  route_table_id             = oci_core_route_table.nonprod_private.id
  freeform_tags              = local.common_tags
}

# ---- Network Security Groups: per-role granular controls ----
#
# nsg_stage0_a1 — protects the A1 application-shell stage-0 host.
# Allows the bastion + ops NSGs to reach it; no public ingress except via LB.
resource "oci_core_network_security_group" "stage0_a1" {
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nsg-stage0-a1"
  freeform_tags  = local.common_tags
}

resource "oci_core_network_security_group" "bastion" {
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nsg-bastion"
  freeform_tags  = local.common_tags
}

resource "oci_core_network_security_group" "ops" {
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nsg-ops"
  freeform_tags  = local.common_tags
}

# ---- NSG rules ----
# Bastion: SSH from anywhere; HTTPS for cloudflared.
resource "oci_core_network_security_group_security_rule" "bastion_ssh_in" {
  network_security_group_id = oci_core_network_security_group.bastion.id
  direction                 = "INGRESS"
  protocol                  = "6" // tcp
  source                    = "0.0.0.0/0"
  source_type               = "CIDR_BLOCK"
  stateless                 = false

  tcp_options {
    destination_port_range {
      min = 22
      max = 22
    }
  }
}

resource "oci_core_network_security_group_security_rule" "bastion_egress_all" {
  network_security_group_id = oci_core_network_security_group.bastion.id
  direction                 = "EGRESS"
  protocol                  = "all"
  destination               = "0.0.0.0/0"
  destination_type          = "CIDR_BLOCK"
  stateless                 = false
}

# Ops: SSH only from bastion NSG; egress all.
resource "oci_core_network_security_group_security_rule" "ops_ssh_from_bastion" {
  network_security_group_id = oci_core_network_security_group.ops.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = oci_core_network_security_group.bastion.id
  source_type               = "NETWORK_SECURITY_GROUP"
  stateless                 = false

  tcp_options {
    destination_port_range {
      min = 22
      max = 22
    }
  }
}

resource "oci_core_network_security_group_security_rule" "ops_egress_all" {
  network_security_group_id = oci_core_network_security_group.ops.id
  direction                 = "EGRESS"
  protocol                  = "all"
  destination               = "0.0.0.0/0"
  destination_type          = "CIDR_BLOCK"
  stateless                 = false
}

# Stage-0 A1: SSH from bastion only; HTTP/8080 + HTTPS/443 from anywhere (will
# be fronted by LB later; for now public-LB-equivalent via NSG).
resource "oci_core_network_security_group_security_rule" "stage0_a1_ssh_from_bastion" {
  network_security_group_id = oci_core_network_security_group.stage0_a1.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = oci_core_network_security_group.bastion.id
  source_type               = "NETWORK_SECURITY_GROUP"
  stateless                 = false

  tcp_options {
    destination_port_range {
      min = 22
      max = 22
    }
  }
}

resource "oci_core_network_security_group_security_rule" "stage0_a1_http_in" {
  network_security_group_id = oci_core_network_security_group.stage0_a1.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = "0.0.0.0/0"
  source_type               = "CIDR_BLOCK"
  stateless                 = false

  tcp_options {
    destination_port_range {
      min = 8080
      max = 8080
    }
  }
}

resource "oci_core_network_security_group_security_rule" "stage0_a1_https_in" {
  network_security_group_id = oci_core_network_security_group.stage0_a1.id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = "0.0.0.0/0"
  source_type               = "CIDR_BLOCK"
  stateless                 = false

  tcp_options {
    destination_port_range {
      min = 443
      max = 443
    }
  }
}

resource "oci_core_network_security_group_security_rule" "stage0_a1_egress_all" {
  network_security_group_id = oci_core_network_security_group.stage0_a1.id
  direction                 = "EGRESS"
  protocol                  = "all"
  destination               = "0.0.0.0/0"
  destination_type          = "CIDR_BLOCK"
  stateless                 = false
}

output "nonprod_private_subnet_id" {
  value = oci_core_subnet.nonprod_private.id
}

output "nat_gateway_id" {
  value = length(oci_core_nat_gateway.nonprod) > 0 ? oci_core_nat_gateway.nonprod[0].id : null
}

output "service_gateway_id" {
  value = length(oci_core_service_gateway.nonprod) > 0 ? oci_core_service_gateway.nonprod[0].id : null
}

output "nsg_ids" {
  value = {
    stage0_a1 = oci_core_network_security_group.stage0_a1.id
    bastion   = oci_core_network_security_group.bastion.id
    ops       = oci_core_network_security_group.ops.id
  }
}
