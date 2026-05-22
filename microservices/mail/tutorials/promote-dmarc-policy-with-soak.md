---
doc_class: Tutorial
microservice: mail
persona: mail-engineer + deliverability-engineer
related_adrs: [ADR-MAIL-001]
date: 2026-05-20
doc_status: published
---

# Tutorial — Promote DMARC policy from `none` to `quarantine` to `reject` with full soak windows

You will: verify SPF + DKIM alignment posture, ingest DMARC aggregate reports, evaluate readiness for promotion, promote `none → quarantine` with 7-day soak, monitor false-positive rate, promote `quarantine → reject` with 14-day soak, and audit-chain-verify the full progression. Total time ≤ 21 days wall-clock (drill: 90 minutes with `--skip-soak-check`).

## Pre-requisites

- A tenant with `tenant_class=paid` and DMARC promotion enabled by pack policy.
- A verified sending domain (`mail domain verify`).
- At least two active DKIM selectors per ADR-MAIL-001.
- DMARC DNS record published with `rua=mailto:dmarc-rua@<tenant-domain>` (aggregate reports).
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `mail_admin` Cedar role.

## Step 1 — Confirm starting state: DMARC `none` (≤ 5 min)

Inspect the current DMARC policy:

```sh
oya mail domain show --tenant acme-corp --domain dom_acme_com_001 | jq '.dmarc_policy, .alignment_mode'
# Output:
#   "none"
#   "relaxed"
```

Verify DNS record:

```sh
dig +short TXT _dmarc.acme.com
# Expected: "v=DMARC1; p=none; rua=mailto:dmarc-rua@acme.com; ruf=mailto:dmarc-ruf@acme.com"
```

## Step 2 — Ingest DMARC aggregate reports + analyze (≤ 7 days wall-clock)

DMARC `rua` aggregate reports are sent by receivers (Gmail, M365, Yahoo, etc.) once per 24 h. They contain per-source-IP authentication results.

Ingest the reports (oyatie ingests automatically via the `dmarc-rua-ingestor` worker):

```sh
oya mail dmarc reports ingest \
    --tenant acme-corp \
    --domain dom_acme_com_001 \
    --since 7d
# Output:
#   reports_ingested: 32 (from 14 distinct receivers)
#   total_messages_evaluated: 1 248 921
#   dmarc_pass: 1 247 814 (99.91%)
#   dmarc_fail: 1 107 (0.089%)
```

Analyze the failures:

```sh
oya mail dmarc reports failure-analysis \
    --tenant acme-corp \
    --domain dom_acme_com_001 \
    --since 7d
# Output:
#   total_failures: 1 107
#   by_source_ip:
#     - source_ip: 192.0.2.42 (sendgrid.example)
#       failures: 412
#       reason: spf_pass dkim_fail (likely third-party tool not configured with DKIM)
#     - source_ip: 198.51.100.7 (legit-mailing-list.example)
#       failures: 280
#       reason: dkim_pass spf_fail dmarc_arc_chain_present (legitimate forwarder)
#     - source_ip: 203.0.113.15 (unknown)
#       failures: 415
#       reason: spf_fail dkim_fail (likely phishing or spam impersonator)
#   recommendation:
#     - Configure DKIM for sendgrid.example via tenant transactional-mail integration.
#     - Add ARC override for legit-mailing-list.example.
#     - Do NOT add the unknown IP; DMARC reject will protect against spoofing.
```

Action items:

```sh
# 1. Add ARC override for the legitimate forwarder
oya mail auth-override arc-forwarder-grant \
    --tenant acme-corp \
    --domain dom_acme_com_001 \
    --forwarder-domain legit-mailing-list.example \
    --expires-at 2027-05-20T00:00:00Z \
    --justification "Q2 audit confirmed legit"

# 2. Configure DKIM for sendgrid integration
oya mail integration sendgrid configure \
    --tenant acme-corp \
    --domain dom_acme_com_001 \
    --sendgrid-account-id sg_acme_001 \
    --dkim-cname-required true
# Output: returns the CNAME records SendGrid + tenant need to publish for DKIM-signed third-party sending
```

After ARC override + DKIM-for-third-party configured, re-run report analysis. Target: failure rate < 0.1% from non-spoofing sources.

## Step 3 — Promote `none → quarantine` (≤ 5 min wall-clock; 7 d if not skipping)

```sh
oya mail dmarc policy promote \
    --tenant acme-corp \
    --domain dom_acme_com_001 \
    --from-policy none \
    --to-policy quarantine \
    --sample-window-days 7 \
    --approved-by u-mail-admin@acme.com
# Cedar evaluates:
#   - failure rate < pack threshold (default 1%) ✓
#   - 7-day sample window satisfied ✓ (or --skip-soak-check for drill)
#   - approver has mail::dmarc_policy::promote ✓
# Output:
#   from_policy: none
#   to_policy: quarantine
#   sample_failure_rate: 0.04%
#   audit_event_id: ae_mail_dmarc_promoted_001
```

Verify DNS record updated:

```sh
oya mail domain show --tenant acme-corp --domain dom_acme_com_001 | jq '.dmarc_policy'
# Output: "quarantine"

dig +short TXT _dmarc.acme.com
# Expected: "v=DMARC1; p=quarantine; rua=mailto:dmarc-rua@acme.com; ruf=mailto:dmarc-ruf@acme.com"
```

## Step 4 — Soak at `quarantine` + monitor false-positives (14 d wall-clock)

Monitor:

```sh
oya mail dmarc dashboard --tenant acme-corp --domain dom_acme_com_001
# Output (Grafana panels):
#   - dmarc_quarantine_rate: 0.087% (target: stable; current trend: stable)
#   - dmarc_forensic_reports: 4 in last 7 d (all from valid receivers; no anomaly)
#   - false_positive_complaints: 0 (from tenant's helpdesk integration)
```

If `dmarc_quarantine_rate` rises above the pack threshold (default 0.5%) OR false-positive complaints > 2 in 7 d, the runbook `dmarc-reject-storm.md` triggers.

After 14 d soak with green metrics:

```sh
oya mail dmarc policy promote \
    --tenant acme-corp \
    --domain dom_acme_com_001 \
    --from-policy quarantine \
    --to-policy reject \
    --sample-window-days 14 \
    --approved-by u-mail-admin@acme.com \
    --approval-required council   # Cedar enforces dual-approval for `reject` promotion
# Cedar evaluates:
#   - failure rate < 0.1% ✓
#   - 14-day sample window satisfied ✓
#   - council approval present ✓ (council = 2 named principals with mail::dmarc_policy::promote_reject)
# Output:
#   from_policy: quarantine
#   to_policy: reject
#   sample_failure_rate: 0.04%
#   audit_event_id: ae_mail_dmarc_promoted_reject_001
```

## Step 5 — Verify enforcement (≤ 30 min)

Send a test message with valid auth → expect delivery:

```sh
oya mail message send \
    --tenant acme-corp \
    --from alice@acme.com \
    --to bob@external.example \
    --subject "Post-reject test" \
    --body "Testing post-reject."
# Output: delivered; dmarc=pass
```

Simulate a spoofing attempt (only possible with the drill harness; production would require an actual spoofer):

```sh
oya mail simulate spoof-inbound \
    --tenant acme-corp \
    --from alice@acme.com \
    --source-ip 203.0.113.42 \
    --spf-pass false \
    --dkim-pass false \
    --to bob@external.example
# Output:
#   inbound_disposition: rejected_at_smtp
#   reason: dmarc_reject_no_authentication
#   audit_event_id: ae_mail_inbound_dmarc_rejected_001
```

## Step 6 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "mail.auth.*" --since 30d
```

Expected events for our flow:

- `mail.auth.domain.verified.v1` (initial domain verification)
- `mail.auth.dkim.selector.activated.v1` (× 2; both selectors)
- `mail.auth.dmarc.policy.promoted.v1` (none → quarantine; quarantine → reject)
- `mail.auth.dmarc.disposition.v1` (every inbound DMARC evaluation; high volume)
- `mail.auth.arc-forwarder.granted.v1` (legit-mailing-list override)
- `mail.auth.signing.failure.v1` (× 0 expected)
- `mail.auth.dmarc.inbound.rejected.v1` (the spoofing simulation above)

```sh
oya audit verify-chain --tenant acme-corp --since 30d
# Output: chain verified, all events signed, signature_gaps: 0
```

## Step 7 — Long-term maintenance

- Monitor DMARC aggregate reports weekly via `oya mail dmarc dashboard`.
- Rotate DKIM selectors every 90 d (per ADR-MAIL-001).
- Review ARC overrides quarterly; revoke ones no longer needed.
- Re-evaluate DMARC reject rate monthly; investigate if >0.5%.

## What you've learned

- DMARC aggregate report ingestion + failure analysis.
- Third-party sending (SendGrid) DKIM CNAME configuration.
- ARC forwarder allowlist for legitimate forwarders.
- DMARC policy promotion `none → quarantine → reject` with soak windows.
- Cedar-gated promotion with pack threshold + council approval.
- Inbound DMARC-reject enforcement.
- Audit-chain verification of the full progression.

Next tutorial: `tutorials/migrate-rsa-to-ed25519-dkim.md` — promote tenant DKIM from RSA-2048 to Ed25519 (RFC 8463) with 72-hour overlap (`tenant_class=paid`).
