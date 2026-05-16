output "tunnel_id" {
  value       = cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.id
  description = "Tunnel UUID. Use this in cloudflared config.yml as `tunnel:`."
}

output "tunnel_name" {
  value = cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.name
}

output "tunnel_cname_target" {
  value       = "${cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.id}.cfargotunnel.com"
  description = "The CNAME target for the published hostnames."
}

output "tunnel_token" {
  value       = cloudflare_zero_trust_tunnel_cloudflared.onprem_kr.tunnel_token
  sensitive   = true
  description = "Connector authentication token. Pipe via `tofu output -raw tunnel_token` into the on-prem cloudflared installer; never paste this back into chat."
}

output "published_hostnames" {
  value = {
    kms     = "https://kms.${var.cloudflare_domain}     → http://127.0.0.1:8200 (OpenBao)"
    foundry = "https://foundry.${var.cloudflare_domain} → http://127.0.0.1:8080 (workspace-shell API)"
    ops     = "https://ops.${var.cloudflare_domain}     → http://127.0.0.1:8080 (workspace-shell docs/ops surfaces)"
    api     = "https://api.${var.cloudflare_domain}     → http://127.0.0.1:8080 (public REST gateway; repoints to OCI API GW at M3-P03)"
  }
}
