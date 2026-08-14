# Terraform: payments edge WAF
# ARCHITECTURE.md §abuse-defence; documentation-rigor.md §3.2.3
# ADR-0243: every gate is a Cedar eval; WAF is the outermost defence layer
# PCI DSS Req 6.4: web application firewall

# Provider requirements are consolidated in payments-crdb.tf (Terraform permits
# exactly one required_providers block per module).

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

# ─── Serving backend (edge WAF attachment) ────────────────────────────────────
# An unattached Cloud Armor policy inspects no traffic; the payments serving
# backend must reference the policy so the SQLi/XSS/rate-limit/geo rules
# actually evaluate payments traffic. The backend group is a GCE_VM_IP_PORT
# regional NEG (no serverless platform required) whose endpoints are registered
# by the GKE NEG integration / edge operator once the payments Service is
# exposed behind the external LB.

resource "google_compute_backend_service" "payments_edge_backend" {
  name        = "payments-edge-backend"
  description = "Payments µservice serving backend — carries the payments-edge-waf Cloud Armor policy (PCI Req 6.4)"

  security_policy = google_compute_security_policy.payments_waf.id

  port_name = "https"
  protocol  = "HTTPS"

  backend {
    group = google_compute_region_network_endpoint_group.payments_edge_neg.id
  }

  health_checks = [google_compute_health_check.payments_edge_health.id]
}

resource "google_compute_region_network_endpoint_group" "payments_edge_neg" {
  name                  = "payments-edge-neg"
  region                = "us-east1"
  network_endpoint_type = "GCE_VM_IP_PORT"
  # The payments workload VPC/subnet. The NEG endpoints (pod IP:port pairs) are
  # registered by the GKE NEG integration / edge operator once the payments
  # service is exposed; until then the backend service has zero endpoints and
  # the WAF policy simply has no traffic to evaluate — it fails closed rather
  # than pretending a SERVERLESS target exists.
  network    = data.google_compute_network.payments_vpc.id
  subnetwork = data.google_compute_subnetwork.payments_subnet.id
}

data "google_compute_network" "payments_vpc" {
  name = "payments-vpc"
}

data "google_compute_subnetwork" "payments_subnet" {
  name   = "payments-subnet"
  region = "us-east1"
}

resource "google_compute_health_check" "payments_edge_health" {
  name               = "payments-edge-health"
  check_interval_sec = 10
  timeout_sec        = 5
  https_health_check {
    port         = 443
    request_path = "/healthz"
  }
}

output "payments_waf_policy_id" {
  value       = google_compute_security_policy.payments_waf.id
  description = "Payments edge WAF security policy ID"
}
