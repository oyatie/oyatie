// Terraform module: containerd-config
// Per ADR-0121 §"containerd 2.3.0 LTS + runc 1.4.0"; deployed on every cluster node via cloud-iac's Terraform runner.

terraform {
  required_version = ">= 1.7.0"
}

variable "pack" {
  type        = string
  description = "Regional pack"
}

variable "node_ids" {
  type        = list(string)
  description = "Node identifiers (from cloud-iac) that receive containerd configuration"
}

variable "containerd_version" {
  type        = string
  default     = "2.3.0"  # first annual LTS per ADR-0121
}

variable "runc_version" {
  type        = string
  default     = "1.4.0"
}

variable "cni_plugins_version" {
  type        = string
  default     = "1.6.0"
}

variable "seccomp_default_profile" {
  type        = string
  default     = "runtime/default"
  description = "Per threat-model T-I-04; CIS K8s 4.2.x"
}

resource "null_resource" "containerd_install" {
  for_each = toset(var.node_ids)

  triggers = {
    node_id = each.key
    version = var.containerd_version
  }

  provisioner "local-exec" {
    command = <<-EOT
      bash ${path.module}/scripts/containerd-install.sh \
        --node-id ${each.key} \
        --containerd-version ${var.containerd_version} \
        --runc-version ${var.runc_version} \
        --cni-plugins-version ${var.cni_plugins_version} \
        --seccomp-profile ${var.seccomp_default_profile}
    EOT
  }
}

resource "null_resource" "containerd_config" {
  for_each = toset(var.node_ids)
  depends_on = [null_resource.containerd_install]

  triggers = {
    node_id = each.key
    config_sha = filesha256("${path.module}/config.toml.tmpl")
  }

  provisioner "local-exec" {
    command = <<-EOT
      bash ${path.module}/scripts/containerd-configure.sh \
        --node-id ${each.key} \
        --template ${path.module}/config.toml.tmpl
    EOT
  }
}

output "containerd_installed_nodes" {
  value = [for n in var.node_ids : n]
}
