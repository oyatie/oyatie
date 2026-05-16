// Cloudflare Tunnel for the on-prem KR-primary cell.
// Exposes local services (OpenBao API, Foundry workspace-shell) via Cloudflare
// without opening inbound ports on the host.
//
// Secret inputs flow through TF_VAR_ env vars only — no secrets in tfvars.

terraform {
  required_version = ">= 1.12.0"
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }
  backend "local" {
    path = "terraform.tfstate"
  }
}

provider "cloudflare" {
  api_token = var.cloudflare_api_token
}

resource "random_id" "tunnel_secret" {
  byte_length = 35
}

# ---- Tunnel for the on-prem KR-primary cell ----
resource "cloudflare_zero_trust_tunnel_cloudflared" "onprem_kr" {
  account_id = var.cloudflare_account_id
  name       = "oyatie-onprem-kr"
  secret     = random_id.tunnel_secret.b64_std
}

# ---- Tunnel ingress rules ----
resource "cloudflare_zero_trust_tunnel_cloudflared_config" "onprem_kr" {
  account_id = var.cloudflare_account_id
  tunnel_id  = cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.id

  config {
    ingress_rule {
      hostname = "kms.${var.cloudflare_domain}"
      service  = "http://127.0.0.1:8200"
      origin_request {
        no_tls_verify   = true
        connect_timeout = "30s"
      }
    }

    ingress_rule {
      hostname = "foundry.${var.cloudflare_domain}"
      service  = "http://127.0.0.1:8080"
      origin_request {
        connect_timeout = "30s"
        http_host_header = "foundry.${var.cloudflare_domain}"
      }
    }

    # Ops portal — workspace-shell docs surface + ops/observability dashboard.
    ingress_rule {
      hostname = "ops.${var.cloudflare_domain}"
      service  = "http://127.0.0.1:8080"
      origin_request {
        connect_timeout = "30s"
        http_host_header = "ops.${var.cloudflare_domain}"
      }
    }

    # Catch-all: 404 anything we don't explicitly route.
    ingress_rule {
      service = "http_status:404"
    }
  }
}

# ---- DNS CNAMEs for the tunnel ----
resource "cloudflare_record" "kms" {
  zone_id = var.cloudflare_zone_id
  name    = "kms"
  content = "${cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.id}.cfargotunnel.com"
  type    = "CNAME"
  proxied = true
  comment = "OpenBao API + UI via Cloudflare Tunnel (managed-by: opentofu)"
}

resource "cloudflare_record" "foundry" {
  zone_id = var.cloudflare_zone_id
  name    = "foundry"
  content = "${cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.id}.cfargotunnel.com"
  type    = "CNAME"
  proxied = true
  comment = "Foundry workspace-shell via Cloudflare Tunnel (managed-by: opentofu)"
}

resource "cloudflare_record" "ops" {
  zone_id = var.cloudflare_zone_id
  name    = "ops"
  content = "${cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.id}.cfargotunnel.com"
  type    = "CNAME"
  proxied = true
  comment = "Ops portal / docs surface via Cloudflare Tunnel (managed-by: opentofu)"
}
