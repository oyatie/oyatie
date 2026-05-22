---
doc_class: Tutorial
microservice: ops-dashboard-control-center
persona: sre-on-call + release-manager + compliance-operator
date: 2026-05-20
doc_status: published
---

# Tutorial — Declare an incident, hold + rollback a deployment, and export a signed evidence pack

You will: declare a SEV-2 incident, hold + rollback a misbehaving deployment, attach evidence references, export a HSM-signed + L1/L2-notarized evidence pack, and audit the full operator-action chain. Total time ≤ 75 minutes.

## Pre-requisites

- paid tenant_class tier ODCC cell with HSM partition provisioned.
- Operator principal with `odcc::*` permits + WebAuthn-registered (Yubikey 5 FIPS or Touch ID Platform Authenticator).
- A drill tenant `drill-acme` + drill service `messenger@v2.34.0` deployed at 5 % canary.
- `oya-dev-cli` configured + `OYA_ODCC_API=https://odcc-api.drill-syd-1.oyatie.local` + `OYA_PRINCIPAL_JWT=<your operator JWT>`.

## Step 1 — Verify your operator scope + step-up readiness (≤ 5 min)

```sh
oya odcc whoami
```

Expected:

```
[principal] oncall-sre-syd-rotation
[tenant_scope] drill-acme, drill-syd-tenant-fleet-rw
[odcc_permits] incident::declare, incident::resolve, deployment::approve, deployment::hold, deployment::rollback, evidence-pack::export, tenant::view-posture, cluster::view-health, operator-actions::query (T1+T2; T3 requires 2-person)
[step_up_freshness] FRESH (last WebAuthn at 2026-05-20T13:42:18Z, freshness window 60 min)
[webauthn_authenticators] yubikey-5-fips-A12345 (registered 2025-03-15), touch-id-laptop-syd (registered 2025-09-02)
```

If step-up is STALE, refresh:

```sh
oya odcc step-up refresh --authenticator yubikey-5-fips-A12345
```

The CLI prompts for the Yubikey tap; freshness window resets to 60 min.

## Step 2 — Declare a SEV-2 incident (≤ 10 min)

The alert-manager fired `msgr-fanout-p99-slo-violation` on `drill-syd-1`. You're the on-call commander; you declare:

```sh
oya odcc incident declare \
    --cell drill-syd-1 \
    --severity SEV2 \
    --classification customer-impacting \
    --title "messenger fanout p99 above SLO during v2.34.0 canary" \
    --first-detected 2026-05-20T13:42:18Z \
    --observed-by "alert-manager-rule-msgr-fanout-p99" \
    --commander oncall-sre-syd-rotation \
    --comm-channel "#incident-2026-05-20-msgr-fanout" \
    --evidence-refs "evidence/grafana/board-msgr-fanout-2026-05-20.png,evidence/sentry/issue-msgr-fanout-12345,evidence/alertmanager/alert-msgr-fanout-2026-05-20-13-42-18Z.json" \
    --suspected-causes "messenger@v2.34.0 canary rollout, fanout-worker-pool-saturation" \
    --idempotency-key "$(uuidgen)"
```

Expected output:

```
[step-up] Tier 1 required for incident::declare
[step-up] WebAuthn freshness: FRESH (5 min ago) — no challenge needed
[cedar] odcc::incident::declare PERMIT (principal=oncall-sre-syd-rotation, resource_cell=drill-syd-1, classification=customer-impacting)
[audit-chain] seal-ed25519:0x7f3a9b2c8b9d4e2a1c4f5e6d7a8b9c0d
[incident_id] inc-2026-05-20-msgr-fanout-001
[state] DECLARED
[commander] oncall-sre-syd-rotation
[comm_channel] #incident-2026-05-20-msgr-fanout (created in oya-messenger)
```

Verify:

```sh
oya odcc incident view --incident-id inc-2026-05-20-msgr-fanout-001
```

## Step 3 — Place a deployment hold on the canary (≤ 10 min)

The incident's suspected cause is `messenger@v2.34.0`. Pause the canary:

```sh
oya odcc deployment hold \
    --service messenger \
    --version v2.34.0 \
    --cell drill-syd-1 \
    --rationale "incident inc-2026-05-20-msgr-fanout-001: p99 latency violation at 12% rollout; pausing canary until root-cause determined" \
    --hold-duration 60m \
    --linked-incident inc-2026-05-20-msgr-fanout-001 \
    --idempotency-key "$(uuidgen)"
```

Expected:

```
[step-up] Tier 2 required for deployment::hold
[step-up] WebAuthn challenge fired — TAP YOUR YUBIKEY
[step-up] freshness updated
[cedar] odcc::deployment::hold PERMIT
[audit-chain] seal-ed25519:0xab12cd34...
[hold] active until 2026-05-20T15:30:00Z
[canary_traffic_freeze] traffic frozen at 12% (no further increase, no traffic reset)
```

The deployment-pipeline-bot (workflow-engine system actor) observes the hold + freezes the rollout. Confirm:

```sh
oya odcc deployment status --service messenger --cell drill-syd-1
```

Expected:

```
- active_version: v2.34.0 (canary @ 12%) + v2.33.7 (88%)
- last_action: hold
- last_action_at: 2026-05-20T14:30:18Z
- last_action_actor: oncall-sre-syd-rotation
- hold_expires: 2026-05-20T15:30:00Z (60min)
- canary_state: FROZEN_AT_12_PERCENT
```

## Step 4 — Investigate + collect evidence (≤ 20 min)

You investigate using the cluster-health panel + audit-trail + Grafana boards. Evidence found: `messenger@v2.34.0` introduced a regression in `fanout-worker-pool` queue depth handling — under load, the worker pool starves new fanout jobs. Confirmed via:

- Grafana board: `messenger-fanout-worker-pool-depth` shows queue depth growing from baseline 50 to 5000 within 5 min of canary traffic.
- Sentry issue: `messenger-fanout-worker-pool-starvation` shows the new code path being hit.
- Git blame: commit `0x4f8c2d3a` introduced the regression.

Attach the evidence + decide to rollback. Pull the relevant boards:

```sh
oya odcc evidence attach \
    --incident-id inc-2026-05-20-msgr-fanout-001 \
    --evidence-source grafana-board-export \
    --board messenger-fanout-worker-pool-depth \
    --time-range "2026-05-20T13:00:00Z/2026-05-20T14:00:00Z" \
    --out-ref "evidence/grafana/board-msgr-fanout-worker-pool-depth-2026-05-20.png"
```

## Step 5 — Execute rollback (≤ 10 min)

```sh
oya odcc deployment rollback \
    --service messenger \
    --to-version v2.33.7 \
    --from-version v2.34.0 \
    --cell drill-syd-1 \
    --rollback-rationale "incident inc-2026-05-20-msgr-fanout-001: messenger@v2.34.0 confirmed to introduce fanout-worker-pool-starvation; rolling back to v2.33.7 last-known-good per commit 0x4f8c2d3a being the offending change" \
    --rollback-strategy "rapid-traffic-shift-100-percent" \
    --evidence-refs "evidence/grafana/board-msgr-fanout-worker-pool-depth-2026-05-20.png,evidence/sentry/issue-msgr-fanout-worker-pool-starvation,evidence/git/commit-0x4f8c2d3a-introduces-regression" \
    --linked-incident inc-2026-05-20-msgr-fanout-001 \
    --idempotency-key "$(uuidgen)"
```

Step-up Tier 2 fires (Yubikey tap). Cedar gate `odcc::deployment::rollback` evaluates. The rollback completes:

```
[step-up] Tier 2 required for deployment::rollback
[step-up] WebAuthn challenge fired — TAP YOUR YUBIKEY
[step-up] freshness updated
[cedar] odcc::deployment::rollback PERMIT
[audit-chain] seal-ed25519:0xfedcba98...
[rollback] traffic shift initiated
[rollback] 2026-05-20T14:42:18Z: 12% → 0% v2.34.0; 88% → 100% v2.33.7
[rollback_acked_at] 2026-05-20T14:42:23Z (5 sec)
[active_version_after] v2.33.7
```

Verify the rollback:

```sh
oya odcc deployment status --service messenger --cell drill-syd-1
```

Expected:

```
- active_version: v2.33.7 (100%)
- last_action: rollback
- last_action_at: 2026-05-20T14:42:18Z
- last_action_actor: oncall-sre-syd-rotation
- last_action_audit_seal: ed25519-seal:0xfedcba98...
```

## Step 6 — Resolve the incident (≤ 5 min)

```sh
oya odcc incident resolve \
    --incident-id inc-2026-05-20-msgr-fanout-001 \
    --resolution-cause "fanout-worker-pool-starvation introduced by messenger@v2.34.0 commit 0x4f8c2d3a; rolled back to v2.33.7" \
    --resolution-time 2026-05-20T14:45:00Z \
    --mitigation-applied "deployment rollback to v2.33.7" \
    --root-cause-confirmed-by "grafana-board-fanout-worker-pool-depth + sentry-issue-msgr-fanout-worker-pool-starvation + git-bisect" \
    --follow-up-actions "messenger team to author fix patch; re-canary v2.34.1 with worker-pool-depth alarm threshold lowered" \
    --idempotency-key "$(uuidgen)"
```

## Step 7 — Export a signed evidence pack (≤ 15 min)

A compliance officer asks for an evidence pack covering this incident for SOC2 Type II audit:

```sh
oya odcc evidence-pack export \
    --tenant drill-acme \
    --period-start 2026-05-20T13:30:00Z \
    --period-end 2026-05-20T15:00:00Z \
    --frameworks "SOC2-CC7.2,SOC2-CC7.4,ISO27001-A.16.1" \
    --evidence-scope "audit-chain-seals,cedar-decisions,operator-actions,deployment-records,incident-records,evidence-refs" \
    --incident-id-filter inc-2026-05-20-msgr-fanout-001 \
    --signing-mode hsm \
    --hsm-partition syd-hsm-cluster-prod-1/odcc-evidence-key-v3 \
    --notarize-to "qldb:oyatie-odcc-evidence-ledger,polygon:0xab12cd34...evidence-anchor-contract" \
    --requester compliance-officer-acme \
    --case-ref "SOC2-2026-Q2-INCIDENT-RESPONSE-EVIDENCE" \
    --idempotency-key "$(uuidgen)"
```

Step-up Tier 1 fires (Yubikey tap). The job is queued; ticket ID returned:

```
[step-up] Tier 1 required for evidence-pack::export
[step-up] WebAuthn freshness: FRESH — no challenge needed
[cedar] odcc::evidence-pack::export PERMIT
[audit-chain] seal-ed25519:0x12abcd34...
[ticket_id] evp-2026-05-20-7f3a9b2c
[state] QUEUED
[estimated_completion] 90s
```

Poll status:

```sh
oya odcc evidence-pack status --ticket-id evp-2026-05-20-7f3a9b2c
```

After ~ 90 s:

```
[state] SIGNED + NOTARIZED
[pack_size] 4.2 MB
[content_hash] sha256:0x7f3a9b2c8b9d4e2a1c4f5e6d7a8b9c0d
[hsm_signature] 0xab12cd34efab12cd34ef...
[hsm_partition_used] syd-hsm-cluster-prod-1/odcc-evidence-key-v3
[qldb_anchor] hash-tree-anchor:oyatie-odcc-evidence/1234567
[polygon_anchor] tx:0xfedcba9876543210...
[download_url] https://evp.syd-1.oyatie.local/packs/evp-2026-05-20-7f3a9b2c
[expires] 2026-05-27T15:15:00Z
[manifest]
  - audit_chain_seals: 12
  - cedar_decisions: 12
  - operator_actions: 4 (incident-declare, deployment-hold, deployment-rollback, incident-resolve)
  - deployment_records: 2 (hold + rollback)
  - incident_records: 1
  - evidence_refs: 7
```

## Step 8 — Verify the evidence pack offline (≤ 5 min)

Download the pack:

```sh
curl -O "https://evp.syd-1.oyatie.local/packs/evp-2026-05-20-7f3a9b2c"
```

Verify HSM signature:

```sh
oya odcc evidence-pack verify --pack-file evp-2026-05-20-7f3a9b2c \
    --hsm-public-key-source "kms://syd-hsm-cluster-prod-1/odcc-evidence-key-v3/public-key"
```

Expected:

```
[content_hash] sha256:0x7f3a9b2c8b9d4e2a1c4f5e6d7a8b9c0d MATCH
[hsm_signature] VERIFIED (HSM public key matched)
[qldb_anchor] VERIFIED (Hash-tree-anchor exists in oyatie-odcc-evidence-ledger at sequence 1234567)
[polygon_anchor] VERIFIED (tx 0xfedcba98... exists in block 12345678 with content_hash committed)
[audit_chain_seals] 12 / 12 VERIFIED
[cedar_decisions] 12 / 12 REPLAYABLE
```

## What you've learned

- The full incident lifecycle: declare → hold → rollback → resolve.
- The step-up-auth tier escalation per command type (Tier 1, Tier 2).
- The idempotency-key requirement on every mutating command.
- The audit-chain seal emission on every operator action.
- The evidence-pack export with HSM signing + L1/L2 notarization.
- The offline verification of an exported evidence pack against the HSM public key + the L1/L2 anchors.

Next tutorial: `tutorials/forensic-investigation-2-person-step-up.md` — execute a Tier-3 cross-tenant historical query with 2-person principal.
