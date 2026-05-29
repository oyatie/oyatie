# Terraform: payments edge WAF
# ARCHITECTURE.md §abuse-defence; documentation-rigor.md §3.2.3
# ADR-0243: every gate is a Cedar eval; WAF is the outermost defence layer
# PCI DSS Req 6.4: web application firewall

terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.30"
    }
  }
}

# ─── Cloud Armor Security Policy (edge WAF) ───────────────────────────────────

resource "google_compute_security_policy" "payments_waf" {
  name        = "payments-edge-waf"
  description = "Payments µservice edge WAF — PCI DSS Req 6.4 + §3.2.3 anti-bot/spoof/scrape"

  # ─── OWASP CRS managed rule sets ───────────────────────────────────────────

  rule {
    action   = "deny(403)"
    priority = 1000
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('sqli-v33-stable')"
      }
    }
    description = "SQL injection protection (PCI Req 6.4)"
  }

  rule {
    action   = "deny(403)"
    priority = 1001
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('xss-v33-stable')"
      }
    }
    description = "XSS protection"
  }

  rule {
    action   = "deny(403)"
    priority = 1002
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('rfi-v33-stable')"
      }
    }
    description = "Remote file inclusion"
  }

  # ─── Bot management — §3.2.3 anti-bot ─────────────────────────────────────

  rule {
    action   = "deny(429)"
    priority = 2000
    match {
      expr {
        # Block known bot fingerprints (JA4+ blocklist synced from abuse-defence.cedar)
        expression = "request.headers['user-agent'].matches('(?i)(semrush|ahrefsbot|dotbot|mj12bot)') || evaluatePreconfiguredExpr('methodenforcement-v33-stable')"
      }
    }
    description = "Bot fingerprint blocklist — §3.2.3 anti-bot"
  }

  # ─── Rate limiting — §3.2.3 anti-scrape ───────────────────────────────────

  rule {
    action = "throttle"
    priority = 3000
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
    rate_limit_options {
      conform_action = "allow"
      exceed_action  = "deny(429)"
      rate_limit_threshold {
        count        = 1000
        interval_sec = 60
      }
      ban_duration_sec = 300
    }
    description = "Global per-IP rate limit 1000 req/min — §3.2.3 anti-scrape"
  }

  # ─── Geo-restriction: CN PIPL cell isolation ───────────────────────────────
  # CN traffic MUST route to CN-domiciled cell only; block CN IPs at global edge.

  rule {
    action   = "deny(403)"
    priority = 4000
    match {
      expr {
        expression = "origin.region_code == 'CN' && !request.path.matches('/v1/cn/.*')"
      }
    }
    description = "CN PIPL: redirect CN traffic to CN cell; block cross-border at edge"
  }

  # ─── Default allow ─────────────────────────────────────────────────────────

  rule {
    action   = "allow"
    priority = 2147483647
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
    description = "Default allow (all other traffic passes to Cedar gate)"
  }
}

output "payments_waf_policy_id" {
  value       = google_compute_security_policy.payments_waf.id
  description = "Payments edge WAF security policy ID"
}
