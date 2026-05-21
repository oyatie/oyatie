---
doc_class: Onboarding
microservice: ops-dashboard-control-center
persona: sre-on-call + release-manager
related_adrs: [ADR-0316, ADR-0243, ADR-0244, ADR-0131, ADR-0111]
date: 2026-05-20
doc_status: published
---

# SRE On-Call Onboarding — first 5 working days at ops-dashboard-control-center

Audience: an SRE who has just joined the oyatie ops rotation. By Day-5 they will have: declared an incident in a drill cell, walked a deployment hold + rollback, exported a signed evidence pack, exercised the KR-localization escalation overlay, and shadowed a Cedar step-up auth flow during a forensic peek.

## Day 1 — Tour the operator surface

1. Read `PRD.md` § Users + § Acceptance criteria (AC-01 through AC-07) + `decisions/ADR-0243-cedar-universal-gate.md` § operator-action section + `IP-002-incident-command-workflows.md` + `IP-006-evidence-pack-export.md`.
2. Open the Grafana folder `odcc`. Identify boards: `odcc-operator-action-rate`, `odcc-step-up-auth-latency`, `odcc-evidence-pack-export-rate`, `odcc-cedar-decision-cache-hitrate`, `odcc-cluster-health-panel-refresh`, `odcc-incident-declaration-rate`, `odcc-deployment-approval-rate`.
3. Walk the runbook index. On-call runbooks: `incident-declaration-stall.md`, `deployment-approval-divergence.md`, `evidence-pack-export-failure.md`, `cedar-step-up-loop.md`, `cluster-panel-stale.md`, `kr-localization-escalation.md`, `audit-chain-emission-stall.md`.
4. Sit in on Tuesday's incident-handoff stand-up.

Acceptance: you can sketch the operator-action path: operator auth → step-up if required → Cedar gate → idempotency-key check → command emit to backend → audit-chain seal → response.

## Day 2 — Declare an incident in a drill cell

The on-call runbook flow:

```sh
# Open the incident-declare modal
oya odcc incident declare \
    --cell drill-syd-1 \
    --severity SEV2 \
    --classification customer-impacting \
    --title "messenger fanout p99 above SLO" \
    --first-detected 2026-05-20T13:42:18Z \
    --observed-by "alert-manager-rule-msgr-fanout-p99" \
    --commander oncall-pager-rotation-sev2-syd \
    --comm-channel "#incident-2026-05-20-msgr-fanout" \
    --evidence-refs "evidence/grafana/board-msgr-fanout-2026-05-20.png,evidence/sentry/issue-msgr-fanout-12345" \
    --idempotency-key "$(uuidgen)"
```

The flow:

1. The CLI submits the command to `odcc-api.drill-syd-1.oyatie.local`.
2. Step-up auth: WebAuthn challenge fires (Yubikey 5 FIPS or Touch ID); the operator authenticates within 30 s window.
3. Cedar gate `odcc::incident::declare` evaluates the principal + cell + classification.
4. The command is staged in Postgres with idempotency-key bound.
5. Audit-chain seal emitted via `audit-chain` µservice (Ed25519 sig + Merkle leaf).
6. The incident record is created; ID returned.

Verify:

```sh
oya odcc incident status --incident-id inc-2026-05-20-msgr-fanout-001
```

Expected:

- `state`: DECLARED
- `commander`: oncall-pager-rotation-sev2-syd
- `audit_seal_ref`: ed25519-seal:0x7f3a9b2c...
- `evidence_refs`: 2

Acceptance: you can declare + verify an incident; the audit-chain seal reference appears in `oya audit query`.

## Day 3 — Deployment approval, hold, and rollback drill

A release manager is pushing `messenger@v2.34.0` to production. As the deploy-approver-on-rotation:

```sh
oya odcc deployment approve \
    --service messenger \
    --version v2.34.0 \
    --cell drill-syd-1 \
    --change-ticket DRILL-2026-05-20-001 \
    --approver-rationale "messenger@v2.34.0 passed canary at 5% for 24h; metric drift within 2%" \
    --canary-stage 5% \
    --propagation-window "2026-05-20T17:00:00Z..2026-05-20T19:00:00Z" \
    --idempotency-key "$(uuidgen)"
```

Step-up auth fires (deployment approvals require it). Cedar gate `odcc::deployment::approve` evaluates.

Mid-canary, alerts fire. Place a HOLD:

```sh
oya odcc deployment hold \
    --service messenger \
    --version v2.34.0 \
    --cell drill-syd-1 \
    --rationale "p99 latency spike above SLO at 12% rollout" \
    --hold-duration 60m \
    --idempotency-key "$(uuidgen)"
```

The hold pauses the rollout; canary holds at current percentage.

Investigation shows the bad version. Execute rollback:

```sh
oya odcc deployment rollback \
    --service messenger \
    --to-version v2.33.7 \
    --from-version v2.34.0 \
    --cell drill-syd-1 \
    --rollback-rationale "p99 latency confirmed by alert manager rule msgr-fanout-p99; rolling back to v2.33.7 last-known-good" \
    --rollback-strategy "rapid-traffic-shift-100-percent" \
    --evidence-refs "evidence/grafana/board-msgr-fanout-during-canary-2026-05-20.png" \
    --idempotency-key "$(uuidgen)"
```

Step-up auth required (rollback is high-risk). Cedar gate `odcc::deployment::rollback` evaluates. Audit-chain seal emitted. The rollback is decoupled from the hold; they are separate command types per AC-02.

Verify:

```sh
oya odcc deployment status --service messenger --cell drill-syd-1
```

Expected:

- `active_version`: v2.33.7
- `last_action`: rollback
- `last_action_at`: 2026-05-20T17:42:18Z
- `last_action_actor`: oncall-pager-rotation-sev2-syd
- `last_action_audit_seal`: ed25519-seal:0xab12cd34...

Acceptance: you can articulate why approve, hold, and rollback are separate Cedar gates (AC-02 separation of concerns) and why each carries its own idempotency key.

## Day 4 — Evidence-pack export + KR localization overlay

A compliance officer requests an evidence pack for SOC2 Type II audit window 2026-04-01 to 2026-04-30 for tenant `drill-acme`.

```sh
oya odcc evidence-pack export \
    --tenant drill-acme \
    --period-start 2026-04-01T00:00:00Z \
    --period-end 2026-04-30T23:59:59Z \
    --frameworks "SOC2-CC6.1,SOC2-CC6.6,SOC2-CC7.2" \
    --evidence-scope "audit-chain-seals,cedar-decisions,operator-actions,deployment-records,incident-records" \
    --signing-mode hsm \
    --hsm-partition syd-hsm-cluster-prod-1/odcc-evidence-key-v3 \
    --notarize-to "qldb:oyatie-odcc-evidence-ledger,polygon:0xab12...evidence-anchor-contract" \
    --idempotency-key "$(uuidgen)"
```

The flow:

1. Cedar gate `odcc::evidence-pack::export` evaluates + step-up auth required.
2. The compliance-officer's tenant scope is validated (Cedar resource attribute `tenant_id=drill-acme`).
3. The job is queued; ticket ID returned (AC-05: returns ticket + audit seal, not opaque side effect).
4. Backend assembles the pack: audit-chain seals + Cedar decision log + operator action log + deployment records + incident records.
5. Pack is signed with HSM partition key (FIPS 140-2 Level 3).
6. Notarization to AWS QLDB + Polygon zkEVM L2 anchor.
7. Pack URL (signed presigned URL, 7-day expiry) returned via webhook to operator's email.

Verify:

```sh
oya odcc evidence-pack status --ticket-id evp-2026-05-20-7f3a9b2c
```

Expected:

- `state`: SIGNED + NOTARIZED
- `pack_size`: 8.2 GB
- `sha256`: 0x7f3a9b2c...
- `hsm_sig`: 0xab12cd34...
- `qldb_anchor`: hash-tree-anchor:oyatie-odcc-evidence/1234567
- `polygon_anchor`: tx:0xfedc...
- `download_url`: https://evp.syd-1.oyatie.local/packs/evp-2026-05-20-7f3a9b2c (expires 2026-05-27T18:30:00Z)

Now, the KR localization overlay drill. A tenant `drill-seoul-corp` (pack-kr-pipa) has a P1 incident. The localization-aware escalation routes:

```sh
oya odcc incident declare \
    --cell drill-kr-seoul-1 \
    --severity SEV1 \
    --classification customer-impacting \
    --tenant drill-seoul-corp \
    --pack pack-kr-pipa \
    --title "KR-resident PII exfil suspected" \
    --escalation-overlay kr-pipa-ombudsman \
    --korean-language-runbook "runbooks/kr-pipa-data-exfil-investigation-ko.md" \
    --idempotency-key "$(uuidgen)"
```

The escalation overlay (per IP-007):

1. Routes the incident commander to KR-resident SRE first (`oncall-sre-kr-resident-rotation`).
2. Notifies the KR-PIPA Ombudsman within 24 h (statutory requirement per PIPA Art. 39-4).
3. Pulls the Korean-language runbook (subtitle: `KR-PIPA-DESI-001 — PII Exfil Investigation`).
4. Audit-chain entries include `pack_id=pack-kr-pipa` + `kr_kic_breach_notification_clock_start=<timestamp>`.

Acceptance: you can export a signed + notarized evidence pack; you can declare an incident with KR-PIPA pack escalation overlay.

## Day 5 — Forensic peek + step-up auth + audit emission

A compliance investigator needs to view a 6-month-old operator action history for tenant `drill-acme` (suspected cross-tenant peek). This is a Tier-3 step-up action.

```sh
oya odcc operator-actions query \
    --tenant drill-acme \
    --actor-glob "*" \
    --since 2025-11-20T00:00:00Z \
    --until 2026-05-20T23:59:59Z \
    --action-type "tenant-peek,evidence-export,cross-tenant-act" \
    --justification "incident-2026-05-19-cross-tenant-data-leak-investigation" \
    --case-ref "INVEST-2026-05-19-001" \
    --requires-step-up 3 \
    --idempotency-key "$(uuidgen)"
```

Step-up Tier 3 fires: two-person approval is required (the investigator + a council-architecture lead). Both must complete WebAuthn step-up within a 5-minute window. Cedar gate `odcc::operator-actions::query` evaluates with 2-person principal.

Investigation result:

```
[query result]
- 47 operator actions in window
- 3 tenant-peek actions by operator 'release-mgr-east-rotation' (justifications recorded)
- 0 evidence-export actions
- 1 cross-tenant-act action by operator 'finops-bridge-bot' (Cedar rule emission)
- audit-chain seal references: 47 (all verified)
```

The investigator pulls the relevant audit-chain seals + observes their content for the case file. The query itself was audit-logged (AC-09: every operator action emits a seal event).

Acceptance: you can articulate the 2-person step-up Tier 3 + the Cedar gate for forensic queries + the audit-chain self-emission.

## What you've learned

- The operator surface contract: every mutating API requires idempotency-key + Cedar gate + step-up where applicable + audit-chain seal.
- The incident declare → hold → rollback flow + the AC-02 separation of concerns.
- The evidence-pack export with HSM signing + L1/L2 notarization.
- The KR-PIPA localization-aware escalation overlay + statutory 24-h ombudsman notification.
- The Tier-3 step-up forensic-query flow with 2-person principal.
- The self-emission audit-chain seal on every operator action.

Next week: ADR-promotion-triage panel walk-through, FinOps embed panel, cross-region operator handoff (follow-the-sun).
