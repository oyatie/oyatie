output "module_identity" {
  description = "ADR-0339 OpenTofu module identity for catalog consumers."
  value = {
    context            = "aws-guest"
    primitive          = "sg-baseline"
    module_path        = local.module_path
    release_identifier = local.release_identifier
    version            = local.module_version
  }
}

output "security_group_baseline" {
  description = "Metadata-only deny-by-default ingress and allow-listed egress policy."
  value       = local.security_group_baseline
}

output "required_labels" {
  description = "Labels downstream resource wrappers must preserve."
  value       = local.observability_labels
}

output "cosign_attestation_digest" {
  description = "Cosign attestation metadata digest supplied by the pinned wrapper."
  value       = var.cosign_attestation_digest
}
