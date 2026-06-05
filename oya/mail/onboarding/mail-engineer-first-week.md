---
doc_class: Onboarding
microservice: mail
persona: mail-engineer + email-deliverability-engineer + dkim-spf-dmarc-engineer
related_adrs: [ADR-MAIL-001, ADR-MAIL-0001, ADR-MAIL-0002, ADR-MAIL-0003, ADR-MAIL-0004, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# Mail Engineer onboarding — first 5 working days on `mail`

Audience: a new mail engineer, email-deliverability engineer, or DKIM/SPF/DMARC engineer joining the `mail` rotation. By Day-5 they will have: bootstrapped a `tenant_class=demo_trial` drill cell, verified a tenant domain, activated DKIM selectors, promoted DMARC policy through the soak window, exercised the mail-key recovery envelope, and walked a DMARC-reject-storm runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 40 min). Note the five-vendor displacement (Gmail/M365/Proton/Zoho/Tutanota) + per-tenant DKIM key custody doctrine.
2. Read `ARCHITECTURE.md` § mail-auth-policy + § signing-key-custody + § dmarc-policy-state-machine + § ediscovery-legal-hold (∼ 60 min).
3. Read `decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md` end-to-end (∼ 50 min). This is the binding architecture.
4. Read `decisions/ADR-MAIL-0001-personal-mail-key-recovery.md`, `ADR-MAIL-0002-backend-tenant-class-workload-policy.md`, `ADR-MAIL-0003-sdk-launch-order.md`, `ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md` (∼ 35 min total).
5. Read RFC 7208 (SPF), RFC 6376 (DKIM), RFC 7489 (DMARC), RFC 8617 (ARC) section overviews (∼ 90 min).
6. Open the Grafana folder `mail`. Reference boards: `mail-dkim-sign-latency`, `mail-dmarc-reject-rate`, `mail-spf-lookup-budget-exhaustion`, `mail-dns-cache-staleness`, `mail-outbound-auth-hold-queue`, `mail-jmap-push-latency`, `mail-key-recovery-attempt-total`.
7. Walk `runbooks/README.md`. The on-call runbooks: `dkim-key-rotation.md`, `dmarc-rollout-monitoring.md`, `account-compromise-recovery.md`, `dmarc-reject-storm.md`, `dns-resolver-partition.md`, `mta-sts-deliverability-failure.md`, `tlsrpt-spike.md`, `openbao-signing-key-unavailable.md`.
8. Sit in on the Wednesday mail-substrate handoff. Watch the outgoing rotation review the past-week DKIM sign p95 + DMARC false-positive rate + DMARC promotion candidates.

Acceptance: you can sketch the outbound path: tenant compose API → DKIM signing lease from OpenBao (≤ 60 s TTL) → DKIM-sign → SPF + DMARC alignment validation → outbound SMTP → audit-chain emit `EVT-MAIL-OUTBOUND-SIGNED`. And the inbound path: SMTP receive → SPF + DKIM + DMARC + ARC + TLS evaluation → typed `MailAuthResult` → Cedar policy on disposition → spam classifier (Rspamd or LLM per pack) → mailbox insert → JMAP push notification.

## Day 2 — demo_trial drill-cell bootstrap + domain verification

```text
Native operation: mail bootstrap
Route: cloud control-plane operation ledger (not local retired CLI/raw Cargo)
Required evidence:
- Buck2 target(s) for the changed contract/runtime
- Prow/Kubernetes-native `oya-ci-required` job URL
- operation ledger id and emitted audit-chain event ids
```

Expected runtime: ≤ 15 min. Verify:

```sh
oya mail health --cell drill-syd-1
# Expected:
#   postgres.mail-auth-policy: up (lag_ms=14)
#   seaweedfs-s3.mailboxes: up
#   opensearch.mail-index: up
#   postfix.inbound: up (smtp:25)
#   postfix.outbound: up
#   rspamd: up
#   openbao.dkim-keys: up (transit_keys=0)
#   audit-chain.emit: up
```

Create a tenant + verify a sending domain:

```sh
oya mail tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --display-name "ACME Mail" \
    --pack-set default \
    --tenant-class demo_trial

# Initiate domain verification (RFC 7489 DNS TXT challenge)
oya mail domain verify-init \
    --tenant drill-acme \
    --fqdn drill-acme.test
# Output:
#   domain_id: dom_drill_acme_test_001
#   verification_token: oyatie-verify-7c4a2b8e9f...
#   dns_record_required: TXT _oya-verify.drill-acme.test "oyatie-verify-7c4a2b8e9f..."
#   verification_state: pending
```

Add the DNS TXT record (in the drill harness, this is a mock resolver):

```sh
oya mail dns mock-add-record \
    --resolver-cell drill-syd-1 \
    --record "_oya-verify.drill-acme.test TXT oyatie-verify-7c4a2b8e9f..."

# Complete verification
oya mail domain verify-complete \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001
# Output:
#   verification_state: verified
#   audit_event_id: ae_mail_domain_verified_001
```

Acceptance: cell bootstrap + tenant + domain verified.

## Day 3 — DKIM selector pair + SPF + initial DMARC=none

Create DKIM selector pair (per ADR-MAIL-001 § Decision: at least two selectors before production send):

```sh
oya mail dkim selector create-pair \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --algorithm RSA-2048
# Output:
#   selectors:
#     - selector: s20260520a
#       state: pending
#       dns_record: s20260520a._domainkey.drill-acme.test TXT "v=DKIM1; k=rsa; p=MIIBIjANBg..."
#     - selector: s20260520b
#       state: pending
#       dns_record: s20260520b._domainkey.drill-acme.test TXT "v=DKIM1; k=rsa; p=MIIBIjANBg..."
#   openbao_path: secret/drill-acme/mail/dkim/dom_drill_acme_test_001/<selector>
```

Publish the DNS TXT records, then activate the first selector:

```sh
oya mail dkim selector activate \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --selector s20260520a
# Cedar requires step-up + domain ownership
# Output:
#   activated_at: 2026-05-20T14:32:17Z
#   audit_event_id: ae_mail_dkim_activated_001
```

Get all the DNS records the tenant needs to publish:

```sh
oya mail domain dns-records --tenant drill-acme --domain dom_drill_acme_test_001
# Output:
#   SPF:    drill-acme.test TXT "v=spf1 include:_spf.oyatie.local -all"
#   DKIM:   s20260520a._domainkey.drill-acme.test TXT "v=DKIM1; k=rsa; p=MIIBIjANBg..."
#   DKIM:   s20260520b._domainkey.drill-acme.test TXT "v=DKIM1; k=rsa; p=MIIBIjANBg..."
#   DMARC:  _dmarc.drill-acme.test TXT "v=DMARC1; p=none; rua=mailto:dmarc-rua@drill-acme.test; ruf=mailto:dmarc-ruf@drill-acme.test"
#   MTA-STS: mta-sts.drill-acme.test TXT "v=STSv1; id=20260520T143217;"
#   TLSRPT: _smtp._tls.drill-acme.test TXT "v=TLSRPTv1; rua=mailto:tlsrpt@drill-acme.test"
```

Send a test message:

```sh
oya mail message send \
    --tenant drill-acme \
    --from alice@drill-acme.test \
    --to bob@external.example \
    --subject "Test from oyatie mail" \
    --body "Hello! This is a DKIM-signed test."
# Output:
#   message_id: m_drill_001
#   dkim_signature: dkim=pass header.s=s20260520a header.d=drill-acme.test header.b=...
#   spf_result: pass
#   dmarc_result: pass (p=none; observation mode)
#   audit_event_id: ae_mail_outbound_signed_001
```

Acceptance: DKIM selector activated; first signed message sent; audit-chain emission verified.

## Day 4 — DMARC promotion + mail-key recovery envelope

Promote DMARC from `none` → `quarantine` (after 7-day soak in production; instant in drill):

```sh
oya mail dmarc policy promote \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --from-policy none \
    --to-policy quarantine \
    --skip-soak-check true   # drill only
# Cedar requires failure rate < pack threshold
# Output:
#   from_policy: none
#   to_policy: quarantine
#   sample_window_days: 7 (drill: 0)
#   failure_rate: 0.003%
#   approved_by: u-mail-admin@drill.test
#   audit_event_id: ae_mail_dmarc_promoted_001
```

Verify DNS record updated:

```sh
oya mail domain dns-records --tenant drill-acme --domain dom_drill_acme_test_001 | grep DMARC
# Output: _dmarc.drill-acme.test TXT "v=DMARC1; p=quarantine; rua=mailto:dmarc-rua@drill-acme.test; ruf=mailto:dmarc-ruf@drill-acme.test"
```

Set up mail-key recovery envelope for a user (per ADR-MAIL-0001 — tenant_class-gated feature, exercised in demo_trial drill):

```sh
oya mail recovery envelope create \
    --tenant drill-acme \
    --user u-alice@drill-acme.test \
    --recovery-passphrase-policy strong   # min 24 chars, mixed-class, dictionary-checked
# Output:
#   recovery_epoch: 1
#   envelope_handle: re_drill_001
#   recovery_code_returned_to_user: <printed once; user must save>
#   openbao_path: secret/drill-acme/mail/recovery/u-alice/1
#   audit_event_id: ae_mail_recovery_envelope_created_001
```

Test the recovery flow (user lost device + has recovery code):

```sh
# 1. User authenticates via passkey AAL3 step-up
oya identity stepup \
    --tenant drill-acme \
    --principal u-alice@drill-acme.test \
    --required-acr aal3_hardware_bound
# Output: stepup_id=su_alice_001

# 2. Initiate mail-key recovery
oya mail recovery initiate \
    --tenant drill-acme \
    --user u-alice@drill-acme.test \
    --stepup-id su_alice_001 \
    --recovery-code <user's recovery code from step above>
# Cedar requires passkey AAL3 + recovery-code verification
# Output:
#   recovery_grant_id: rg_alice_001
#   new_recovery_epoch: 2
#   restored_mailbox_keys: 1
#   audit_event_id: ae_mail_recovery_completed_001
```

Acceptance: DMARC promotion verified; recovery envelope created + redeemed.

## Day 5 — DMARC-reject-storm runbook + signing key rotation drill

DMARC-reject-storm scenario: a legitimate forwarder (e.g., a mailing-list service) breaks DKIM, causing DMARC-reject for legitimate mail. Read `runbooks/dmarc-reject-storm.md`. Walk through:

1. Identify from `mail-dmarc-reject-rate` panel (sudden spike).
2. Pull DMARC forensic reports for the affected domain.
3. Identify the forwarder via ARC chain analysis.
4. Decision tree:
   - If forwarder is legitimate + has valid ARC chain → grant ARC override (Cedar `mail::auth_override::grant`).
   - If forwarder is illegitimate → no action; DMARC reject is correct.
5. ARC override grant:

```sh
oya mail auth-override arc-forwarder-grant \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --forwarder-domain legit-mailing-list.example \
    --expires-at 2026-08-20T00:00:00Z \
    --justification "Mailing list breaks DKIM but adds valid ARC chain"
# Cedar requires no pack denial (HIPAA + FedRAMP-High block ARC overrides without council approval)
# Output:
#   grant_id: ag_drill_001
#   audit_event_id: ae_mail_arc_override_granted_001
```

6. Monitor DMARC reject rate; expect drop within 1 h as receivers re-evaluate.

DKIM signing key rotation drill (90-day cadence with 72-hour overlap per ADR-MAIL-001):

```sh
# 1. Create new selector pair
oya mail dkim selector create-pair \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --algorithm Ed25519   # enabled by tenant_class and pack policy per ADR-MAIL-001

# 2. Tenant publishes new DKIM DNS records (mock)
oya mail dns mock-add-record \
    --resolver-cell drill-syd-1 \
    --record "s20260820a._domainkey.drill-acme.test TXT 'v=DKIM1; k=ed25519; p=...'"

# 3. Activate new selector (overlap with old)
oya mail dkim selector activate \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --selector s20260820a \
    --overlap-with s20260520a

# 4. After 72-hour overlap, retire old selector
oya mail dkim selector retire \
    --tenant drill-acme \
    --domain dom_drill_acme_test_001 \
    --selector s20260520a
# Output:
#   retired_at: 2026-08-23T14:32:17Z
#   audit_event_id: ae_mail_dkim_retired_001
```

Acceptance: DMARC-reject-storm runbook walked; ARC override granted; DKIM selector rotation overlap drill green.

## What you've learned

- demo_trial bootstrap + tenant + domain verification (DNS TXT challenge).
- DKIM selector pair creation + activation (per-tenant OpenBao custody).
- DMARC policy progression `none → quarantine → reject` with soak.
- Mail-key recovery envelope (passkey AAL3 + recovery code).
- DMARC-reject-storm + ARC override flow.
- DKIM key rotation with 72-hour overlap.

Next week: demo_trial to paid conversion path (Ed25519 DKIM + multi-region + JMAP push + LLM spam classifier with EU-AI-Act gate), paid tenant_class pack tour (DMARC reject + MTA-STS enforced + transit signing for FIPS tenants), paid tenant_class custody tour (FIPS-140-3 L3 HSM + per-pack DKIM residency + per-mailbox DEK envelope), and your first production shadow on a DMARC reject promotion approval.
