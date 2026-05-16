variable "cloudflare_api_token" {
  description = "Cloudflare API token with Tunnel:Edit + DNS:Edit + Account:Read scopes. Provide via TF_VAR_cloudflare_api_token env var — DO NOT put in tfvars."
  type        = string
  sensitive   = true
}

variable "cloudflare_account_id" {
  description = "Cloudflare account ID (sidebar of any dashboard page)."
  type        = string
}

variable "cloudflare_zone_id" {
  description = "Cloudflare zone ID for the parent domain (sidebar of the domain overview page)."
  type        = string
}

variable "cloudflare_domain" {
  description = "Apex domain that owns the subdomains we create."
  type        = string
  default     = "oyatie.com"
}

variable "headscale_public_ip" {
  description = "Reserved public IP of the Headscale OCI VM (output of infra/oci/headscale.tf). Provide via TF_VAR_headscale_public_ip or wire as a remote-state lookup."
  type        = string
}

variable "access_auth_domain" {
  description = "Cloudflare Access auth_domain FQDN (e.g., oyatie.cloudflareaccess.com)."
  type        = string
  default     = "oyatie.cloudflareaccess.com"
}
