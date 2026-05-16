// Cloudflare Access — zero-trust identity gate in front of sensitive surfaces.
// Per user directive 2026-05-16: "we should really be careful with exposing these"
// after spotting that https://kms.oyatie.com/ui/vault/auth?with=token was reachable
// publicly without prior identity check.
//
// Shape: Cloudflare Access challenges the request at the edge (before it reaches
// cloudflared / OpenBao). Only authenticated identities matching one of the
// `cloudflare_zero_trust_access_policy` rules can fetch the origin. Combines
// with the OpenBao API token gate (defense-in-depth).
//
// Identities: provide `var.access_allowed_emails` (list of emails permitted to
// reach the protected hostnames). Empty list = block everyone (locked down).

variable "access_allowed_emails" {
  description = "Email addresses allowed by Cloudflare Access to reach the protected hostnames (kms.oyatie.com, foundry.oyatie.com admin paths). Empty list = locked down."
  type        = list(string)
  default     = []
}

# ---- Application: kms.oyatie.com (OpenBao UI + API) ----
# Type=self_hosted so Access challenges all paths under this hostname.
resource "cloudflare_zero_trust_access_application" "kms" {
  account_id                = var.cloudflare_account_id
  name                      = "oyatie-kms"
  domain                    = "kms.${var.cloudflare_domain}"
  type                      = "self_hosted"
  session_duration          = "24h"
  auto_redirect_to_identity = true
  http_only_cookie_attribute = true
  same_site_cookie_attribute = "lax"
  app_launcher_visible      = false
}

# ---- Application: foundry.oyatie.com (Foundry control plane API) ----
resource "cloudflare_zero_trust_access_application" "foundry" {
  account_id                = var.cloudflare_account_id
  name                      = "oyatie-foundry"
  domain                    = "foundry.${var.cloudflare_domain}"
  type                      = "self_hosted"
  session_duration          = "24h"
  auto_redirect_to_identity = true
  http_only_cookie_attribute = true
  same_site_cookie_attribute = "lax"
  app_launcher_visible      = false
}

# ---- Application: api.oyatie.com (public REST API gateway) ----
# NOTE: a truly public API should NOT be Access-gated. Today api.oyatie.com is
# gated because we have no authenticated callers yet — when first external
# consumer arrives, switch this app to a service-token-based policy (bypass for
# valid tokens, identity-challenge for browsers).
resource "cloudflare_zero_trust_access_application" "api" {
  account_id                 = var.cloudflare_account_id
  name                       = "oyatie-api"
  domain                     = "api.${var.cloudflare_domain}"
  type                       = "self_hosted"
  session_duration           = "24h"
  auto_redirect_to_identity  = true
  http_only_cookie_attribute = true
  same_site_cookie_attribute = "lax"
  app_launcher_visible       = false
}

resource "cloudflare_zero_trust_access_policy" "api_allow" {
  account_id     = var.cloudflare_account_id
  application_id = cloudflare_zero_trust_access_application.api.id
  name           = "allow-listed-emails"
  precedence     = 1
  decision       = "allow"
  include {
    email = var.access_allowed_emails
  }
}

# ---- Application: ops.oyatie.com (ops portal) ----
resource "cloudflare_zero_trust_access_application" "ops" {
  account_id                = var.cloudflare_account_id
  name                      = "oyatie-ops"
  domain                    = "ops.${var.cloudflare_domain}"
  type                      = "self_hosted"
  session_duration          = "24h"
  auto_redirect_to_identity = true
  http_only_cookie_attribute = true
  same_site_cookie_attribute = "lax"
  app_launcher_visible      = false
}

# ---- Allow policy: only the configured emails ----
resource "cloudflare_zero_trust_access_policy" "kms_allow" {
  account_id     = var.cloudflare_account_id
  application_id = cloudflare_zero_trust_access_application.kms.id
  name           = "allow-listed-emails"
  precedence     = 1
  decision       = "allow"
  include {
    email = var.access_allowed_emails
  }
}

resource "cloudflare_zero_trust_access_policy" "foundry_allow" {
  account_id     = var.cloudflare_account_id
  application_id = cloudflare_zero_trust_access_application.foundry.id
  name           = "allow-listed-emails"
  precedence     = 1
  decision       = "allow"
  include {
    email = var.access_allowed_emails
  }
}

resource "cloudflare_zero_trust_access_policy" "ops_allow" {
  account_id     = var.cloudflare_account_id
  application_id = cloudflare_zero_trust_access_application.ops.id
  name           = "allow-listed-emails"
  precedence     = 1
  decision       = "allow"
  include {
    email = var.access_allowed_emails
  }
}

output "access_app_ids" {
  value = {
    kms     = cloudflare_zero_trust_access_application.kms.id
    foundry = cloudflare_zero_trust_access_application.foundry.id
    ops     = cloudflare_zero_trust_access_application.ops.id
    api     = cloudflare_zero_trust_access_application.api.id
  }
}
