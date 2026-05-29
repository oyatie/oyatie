# Per-pack node library publisher set + Ed25519 signing key references.
# Per threat-model.md T-S-04 (publisher impersonation) + T-T-03 (CDN tampering).
# Per ADR-0131 + ADR-0140 (retired per ADR-0145).

terraform {
  required_providers {
    openbao = {
      source  = "openbao/openbao"
      version = "~> 2.0"
    }
  }
}

# =============================================================================
# Per-pack Ed25519 signing key references.
# Keys are stored in OpenBao; this file references them — never embeds.
# Rotation 90d default (per threat-model.md T-S-04).
# =============================================================================

variable "packs" {
  type = list(string)
  default = ["kr", "eu", "us", "us-hc", "jp", "sg", "au", "in", "br", "ae", "ksa"]
}

resource "openbao_kv_secret_v2" "node_library_signing_key" {
  for_each = toset(var.packs)
  mount    = "secret"
  name     = "workflow-studio/node-library-signing/pack-${each.key}"
  data_json = jsonencode({
    key_type           = "ed25519"
    rotation_window_d  = 90
    publisher_set_ref  = "openbao://secret/workflow-studio/node-library-publishers/pack-${each.key}"
  })
}

# =============================================================================
# Per-pack publisher allowlist (which OIDC subjects can sign + publish).
# 2-person rule + signed-commit required (enforced by branch protection).
# =============================================================================

resource "openbao_kv_secret_v2" "node_library_publishers" {
  for_each = toset(var.packs)
  mount    = "secret"
  name     = "workflow-studio/node-library-publishers/pack-${each.key}"
  data_json = jsonencode({
    allowed_publisher_oidc_subs = [
      # Initial publisher set per pack; updated via terraform PR + ops-security review.
      "spiffe://oyatie.dev/workflow-studio/node-library-publisher-pack-${each.key}-primary",
      "spiffe://oyatie.dev/workflow-studio/node-library-publisher-pack-${each.key}-secondary"
    ]
    revoked_keys = []  # populated by template-marketplace-quarantine runbook on Sev-1
  })
}

# =============================================================================
# Per-pack CDN edge configuration for signed library distribution.
# Per threat-model.md T-T-03 + T-I-08 (CDN cache pollution).
# =============================================================================

resource "openbao_kv_secret_v2" "cdn_per_pack_edge_config" {
  for_each = toset(var.packs)
  mount    = "secret"
  name     = "workflow-studio/cdn-edge-config/pack-${each.key}"
  data_json = jsonencode({
    cache_key_template = "(tenant_hash, pack, version, path)"  # per-tenant cache key
    sri_required       = true   # AC-12 every WASM chunk has SRI
    immutable_path_pattern = "/v*/canvas.wasm"
    purge_propagation_sli_seconds = 60
    csp_template_ref = "openbao://secret/workflow-studio/csp/pack-${each.key}"
  })
}
