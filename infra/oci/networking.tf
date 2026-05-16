// VCN + IGW + subnet + default security list for nonprod stage-0.
// Live resources already created via CLI; import with:
//   tofu import oci_core_vcn.nonprod                   <vcn-ocid>
//   tofu import oci_core_internet_gateway.nonprod      <igw-ocid>
//   tofu import oci_core_default_route_table.nonprod_default <rt-ocid>
//   tofu import oci_core_subnet.nonprod_public         <subnet-ocid>
//   tofu import oci_core_default_security_list.nonprod_default <sl-ocid>

resource "oci_core_vcn" "nonprod" {
  compartment_id = oci_identity_compartment.nonprod.id
  display_name   = "oyatie-nonprod-vcn"
  cidr_blocks    = ["10.0.0.0/16"]
  dns_label      = "oyatienpvcn"
  freeform_tags  = local.common_tags
}

resource "oci_core_internet_gateway" "nonprod" {
  compartment_id = oci_identity_compartment.nonprod.id
  vcn_id         = oci_core_vcn.nonprod.id
  display_name   = "oyatie-nonprod-igw"
  enabled        = true
  freeform_tags  = local.common_tags
}

resource "oci_core_default_route_table" "nonprod_default" {
  manage_default_resource_id = oci_core_vcn.nonprod.default_route_table_id

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.nonprod.id
  }
}

resource "oci_core_default_security_list" "nonprod_default" {
  manage_default_resource_id = oci_core_vcn.nonprod.default_security_list_id

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"
    stateless   = false
    tcp_options {
      min = 22
      max = 22
    }
  }

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"
    stateless   = false
    tcp_options {
      min = 80
      max = 80
    }
  }

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"
    stateless   = false
    tcp_options {
      min = 443
      max = 443
    }
  }

  ingress_security_rules {
    protocol    = "1" // icmp
    source      = "10.0.0.0/16"
    source_type = "CIDR_BLOCK"
    stateless   = false
    icmp_options {
      type = 3
      code = 4
    }
  }

  ingress_security_rules {
    protocol    = "1"
    source      = "0.0.0.0/0"
    source_type = "CIDR_BLOCK"
    stateless   = false
    icmp_options {
      type = 3
    }
  }

  egress_security_rules {
    protocol         = "all"
    destination      = "0.0.0.0/0"
    destination_type = "CIDR_BLOCK"
    stateless        = false
  }
}

resource "oci_core_subnet" "nonprod_public" {
  compartment_id             = oci_identity_compartment.nonprod.id
  vcn_id                     = oci_core_vcn.nonprod.id
  display_name               = "oyatie-nonprod-subnet"
  cidr_block                 = "10.0.1.0/24"
  dns_label                  = "oyatienpsn"
  prohibit_public_ip_on_vnic = false
  freeform_tags              = local.common_tags
}
