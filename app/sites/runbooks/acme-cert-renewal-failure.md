---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-ACME-CERT-RENEWAL-FAILURE
severity_class: sev-1-on-expiry-near
related_adrs: [ADR-SITES-0004]
related_slos: [acme-renew-latency]
owner_team: axis-sites + ops-security
date: 2026-05-17
doc_status: published
---

# Runbook: ACME cert renewal failure

## Symptom

A custom-domain TLS cert is approaching expiry and ACME renewal is
failing. Visible as:

- `oya_sites_cert_expiry_seconds{domain}` < 86400 * 7 (7-day-pre-expiry warning).
- `oya_sites_cert_expiry_seconds{domain}` < 86400 (1-day-pre-expiry page).
- `oya_sites_acme_renewal_failed_total{domain,reason}` rising.
- `oya_sites_acme_rate_limit_remaining` near 0 (Let's Encrypt 50
  cert/wk/account exhausted).

## Severity

**Sev-1** if expiry < 24h.
**Sev-2** if expiry < 7d.
**Sev-3** if expiry > 7d but ACME renewal failing.

## First responder

axis-sites on-call. Escalate to ops-security if Sev-1.

## Diagnosis

### Step 1 — Identify failed-renewal signatures

```bash
kubectl -n sites logs deploy/oya-sites-domain-binding-worker --since=24h |
  jq -s 'map(select(.event == "acme_renewal_failed")) |
         group_by(.reason) |
         map({reason: .[0].reason, count: length, sample: .[0]})'
```

Common reasons:

- `dns01_challenge_failed` — DNS provider not honouring our TXT record.
- `rate_limit_exceeded` — Let's Encrypt 50 certs/wk/account exhausted.
- `tls_alpn_failed` — TLS-ALPN-01 challenge failed (rare; we prefer DNS-01).
- `account_locked` — Let's Encrypt account locked due to abuse signal.
- `network_unreachable` — egress to `acme-v02.api.letsencrypt.org` blocked.

### Step 2 — Check ACME account pool state

```bash
cargo run -p oya-dev-cli -- vcs acme-account-pool-status \
  --microservice sites \
  --pack <pack_tag>
```

### Step 3 — Verify DNS state for the failing domain

```bash
dig +short TXT _acme-challenge.<domain>
# expected: TXT record matching our challenge response

# Compare against what we expect:
cargo run -p oya-dev-cli -- vcs acme-challenge-expected \
  --microservice sites \
  --domain <domain>
```

## Mitigation

### Case A — DNS-01 challenge fails (tenant DNS provider issue)

1. Contact the tenant; their DNS provider is not propagating our
   `_acme-challenge.<domain>` TXT record.
2. Provide tenant with the expected TXT record value + provider
   troubleshooting (Cloudflare, Route53, AWS-managed-DNS, Bunny.net,
   Google DNS, custom self-hosted).
3. While waiting: temporarily issue from staging Let's Encrypt (does
   not count against rate limit; produces invalid cert; clearly
   tenant-facing error to nudge fix).

### Case B — Rate limit exhausted

1. Switch to backup ACME account:
   ```bash
   cargo run -p oya-dev-cli -- vcs acme-account-rotate \
     --microservice sites \
     --pack <pack_tag> \
     --to-backup
   ```
2. Resume renewals on the new account.
3. Open a fix-up to widen multi-account pool per ADR-SITES-0004.

### Case C — Account locked

1. Page ops-security. Account locked typically signals abuse signal
   detection by Let's Encrypt.
2. Engage Let's Encrypt support (ISRG); supply tenant attestation +
   evidence chain.
3. Use staging while resolved; document timeline.

### Case D — Network unreachable

1. Verify egress NetworkPolicy (per `iac/helm/templates/networkpolicy.yaml`):
   ACME endpoint is on the egress allowlist.
2. Test from within the cluster:
   ```bash
   kubectl -n sites exec deploy/oya-sites-domain-binding-worker -- \
     curl -v https://acme-v02.api.letsencrypt.org/directory
   ```
3. If still blocked, raise with cloud-network owner.

### Case E — Imminent expiry, no recovery path within window

1. Page ops-security + council-product.
2. Issue a self-signed cert as TEMPORARY fallback with clear
   tenant-facing warning + 24h sunset.
3. Engage tenant directly for emergency DNS fix.

## Verification

After mitigation:

```bash
# Verify cert valid + expiry > 30 days
echo | openssl s_client -servername <domain> -connect <domain>:443 2>/dev/null |
  openssl x509 -noout -dates

# acme-renew-latency SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice sites --slo acme-renew-latency
```

## Post-incident

- File fix-up if root cause was code-side (e.g., DNS-01 implementation bug).
- Update DNS-provider compatibility matrix at
  `microservices/sites/specs/dns-provider-compatibility.json`.
- If Case B, evaluate whether tenant onboarding load justifies adding
  another ACME account or migrating to a self-managed ACME (Step CA).

## References

- ADR-SITES-0004 — ACME + custom-domain flow.
- RFC 8555 — ACME.
- RFC 8737 — TLS-ALPN-01.
- Let's Encrypt rate limits — `letsencrypt.org/docs/rate-limits/`.
- `microservices/sites/slos/acme-renew-latency.openslo.yaml`.
- `microservices/sites/runbooks/custom-domain-dns-drift.md`.
