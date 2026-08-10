---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-CUSTOM-DOMAIN-DNS-DRIFT
severity_class: sev-2
related_adrs: [ADR-SITES-0004]
related_slos: [acme-renew-latency]
owner_team: axis-sites + ops-security
date: 2026-05-17
doc_status: published
---

# Runbook: custom-domain DNS drift

## Symptom

A previously-bound custom domain has had its DNS records changed
externally (tenant or attacker), causing one of:

- DNS no longer points to our CDN; the tenant's site is unreachable
  at their custom domain.
- DNS points to a different IP not under our control (potential
  subdomain takeover).
- The `_acme-challenge.<domain>` TXT record has changed; cert renewal
  will fail next cycle.
- Tenant report: "my site is not reachable" or "my custom domain
  points somewhere weird."

Visible as:

- `oya_sites_dns_drift_detected_total{domain}` increments (scheduled
  DNS verify job).
- `oya_sites_acme_renewal_failed_total{domain,reason="dns01_challenge_failed"}`
  rising.
- External monitoring (UptimeRobot, blackbox-exporter) showing
  the site unreachable.

## Severity

**Sev-2** by default. **Sev-1** if the drift indicates a subdomain
takeover attempt (DNS points to attacker IP).

## First responder

axis-sites on-call. Escalate to ops-security on suspected takeover.

## Diagnosis

### Step 1 — Confirm DNS state

```bash
# What does external DNS say?
dig +short A <domain>
dig +short AAAA <domain>
dig +short CNAME <domain>
dig +short TXT _acme-challenge.<domain>

# What do we expect?
cargo run -p oya-dev-cli -- vcs domain-binding-expected \
  --microservice sites \
  --domain <domain>
```

### Step 2 — Check our domain-binding record state

```bash
cargo run -p oya-dev-cli -- vcs domain-binding-status \
  --microservice sites \
  --tenant <tenant_id> \
  --domain <domain>
```

Look for:
- `dns_verified_at` (when we last verified).
- `cert_expiry`.
- `last_acme_renewal_attempt`.
- `expected_a_record` / `expected_cname`.

### Step 3 — Determine drift category

| Drift category | Signal | Severity |
|---|---|---|
| Tenant changed DNS away (cancelled service?) | DNS points outside our IP space; no contact from tenant | Sev-3 (informational) |
| Tenant changed DNS provider, forgot to re-add records | DNS is partial; some records match, others don't | Sev-2 |
| Subdomain takeover attempt | DNS points to a different cloud provider's IP; no tenant action | **Sev-1** |
| DNS provider outage | NXDOMAIN or SERVFAIL; partial visibility | Sev-2 |

## Mitigation

### Case A — Tenant changed DNS away

1. Contact tenant via support; confirm intent.
2. If intentional, mark domain-binding for off-boarding:
   ```bash
   cargo run -p oya-dev-cli -- vcs domain-unbind \
     --microservice sites \
     --tenant <tenant_id> \
     --domain <domain> \
     --reason "tenant DNS drift; confirmed intentional"
   ```
3. Cert auto-revoked on unbind per ADR-SITES-0004 (prevents subdomain takeover later).

### Case B — Tenant DNS provider migration error

1. Contact tenant; provide expected DNS records (A, CNAME, TXT for
   _acme-challenge).
2. Hold the domain-binding active for 14 days (grace window); cert
   renewal will attempt but will fail until DNS fixed.

### Case C — Subdomain takeover attempt

1. **Page ops-security immediately.**
2. Revoke the cert at Let's Encrypt:
   ```bash
   cargo run -p oya-dev-cli -- vcs acme-cert-revoke \
     --microservice sites \
     --tenant <tenant_id> \
     --domain <domain> \
     --reason "suspected subdomain takeover; ops-security RB-SITES-CUSTOM-DOMAIN-DNS-DRIFT"
   ```
3. Open audit-chain forensic snapshot.
4. Engage external pen-test or red-team to confirm finding.
5. Tenant notification per `incident-response.md` chain.

### Case D — DNS provider outage

1. Wait for DNS to recover (typically minutes to hours).
2. Re-trigger DNS verify after recovery:
   ```bash
   cargo run -p oya-dev-cli -- vcs domain-dns-reverify \
     --microservice sites \
     --domain <domain>
   ```

## Verification

After mitigation:

```bash
# Verify DNS reverification succeeded
cargo run -p oya-dev-cli -- vcs domain-binding-status \
  --microservice sites \
  --domain <domain> |
  grep -i dns_verified_at

# Verify cert valid (if not revoked)
echo | openssl s_client -servername <domain> -connect <domain>:443 2>/dev/null |
  openssl x509 -noout -dates
```

## Post-incident

- If Case C (takeover), write a post-mortem with:
  - Detection timeline.
  - Cert revocation timestamp.
  - Tenant notification timeline.
  - Forensic preservation (audit-chain hash).
  - Improvements to DNS-drift watchdog cadence.
- Update DNS-provider compatibility matrix.

## References

- ADR-SITES-0004 — ACME + custom-domain flow.
- RFC 8555 — ACME.
- `microservices/sites/runbooks/acme-cert-renewal-failure.md`.
- `microservices/sites/threat-model.md` STRIDE "Spoofing" matrix.
- OWASP subdomain takeover guidance.
