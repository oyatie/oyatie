// Terraform module: kubeadm-cluster
// Per ADR-0121 §"On-prem k8s stack"; managed by cloud-iac's Terraform runner.
// cloud-k8s µservice declares this module; the underlying compute is provisioned by cloud-iac.
// kubeadm + containerd are Terraform-applied (not Helm) per ADR-0121 §"Required successor-IP work".

terraform {
  required_version = ">= 1.7.0"
}

variable "pack" {
  type        = string
  description = "Regional pack (e.g., pack-kr, pack-eu, pack-us)"
  validation {
    condition = contains(["pack-kr", "pack-eu", "pack-us", "pack-us-healthcare", "pack-jp", "pack-sg", "pack-au", "pack-in", "pack-br", "pack-ae", "pack-ksa"], var.pack)
    error_message = "pack must be one of the 11 oyatie regional packs"
  }
}

variable "region" {
  type        = string
  description = "Cloud region; e.g., ap-seoul-1"
}

variable "kubeadm_version" {
  type        = string
  default     = "v1.35.0"
  description = "kubeadm minor version; must match docs/standards/cloud-k8s-stack.md LTS pin"
}

variable "control_plane_count" {
  type        = number
  default     = 1
  description = "Control-plane node count; M01=1 (per ADR-0121 §Migration triggers); 3 (M04-onward HA topology)"
  validation {
    condition = var.control_plane_count == 1 || var.control_plane_count == 3
    error_message = "control_plane_count must be 1 (M01) or 3 (M04-onward HA)"
  }
}

variable "worker_count" {
  type        = number
  default     = 17  # per capacity-model.md XS tier
}

variable "control_plane_node_ids" {
  type        = list(string)
  description = "Pre-provisioned control-plane node identifiers (from cloud-iac)"
}

variable "worker_node_ids" {
  type        = list(string)
}

variable "kms_key_id" {
  type        = string
  description = "KMS key for etcd at-rest envelope encryption (per pack)"
}

resource "null_resource" "kubeadm_init" {
  triggers = {
    pack            = var.pack
    kubeadm_version = var.kubeadm_version
  }

  provisioner "local-exec" {
    command = <<-EOT
      bash ${path.module}/scripts/kubeadm-init.sh \
        --pack ${var.pack} \
        --region ${var.region} \
        --kubeadm-version ${var.kubeadm_version} \
        --control-plane-count ${var.control_plane_count} \
        --kms-key-id ${var.kms_key_id}
    EOT
  }
}

resource "null_resource" "kubeadm_join" {
  for_each = toset(var.worker_node_ids)
  depends_on = [null_resource.kubeadm_init]

  triggers = {
    node_id = each.key
  }

  provisioner "local-exec" {
    command = <<-EOT
      bash ${path.module}/scripts/kubeadm-join.sh \
        --node-id ${each.key} \
        --pack ${var.pack}
    EOT
  }
}

output "cluster_id" {
  value = "${var.pack}-cluster-1"
}

output "kubeconfig_path" {
  value     = "/etc/kubernetes/admin.conf"
  sensitive = true
}
