---
doc_class: Onboarding
microservice: governance
persona: governance-engineer + compliance-engineer + sre-evidence-engineer
related_adrs: [ADR-GOV-001, ADR-0003, ADR-0010, ADR-0128, ADR-0329, ADR-0330, ADR-0331, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Governance Engineer onboarding — first 5 working days on `governance`

Audience: a new governance engineer, compliance engineer, or SRE evidence-engineer joining the `governance` rotation. By Day-5 they will have: bootstrapped a demo_trial tenant_class cell, configured retention policies, run an aggregation replay, granted + revoked a bypass, walked the cross-pack-conflict + retention-shortening runbooks.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 40 min). Note the five-vendor displacement + projection-not-source-of-truth doctrine.
2. Read `ARCHITECTURE.md` § evidence-emitter + § aggregation-indexer + § lane-runtime + § retention-policy-engine + § industry-baseline-conformance (∼ 60 min).
3. Read `decisions/ADR-GOV-001-audit-event-aggregation-pack-retention.md` end-to-end (∼ 50 min).
4. Read `decisions/SVC-ADR-WASM-001-envoy-wasm-canonical-governance.md` + IP series IP-001 through IP-015 (∼ 50 min).
5. Read `docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md` + `ADR-0010-regional-pack-architecture.md` (∼ 30 min).
6. Open the Grafana folder `governance`. Primary boards: `governance-evidence-freshness-lag-seconds`, `governance-aggregation-replay-events-per-second`, `governance-retention-conflict-total`, `governance-bypass-active-total`, `governance-lane-evaluation-latency`, `governance-pack-coverage-percent`.
7. Walk `runbooks/README.md`. The on-call runbooks: `aggregation-rebuild.md`, `evidence-replay.md`, `lane-failure-triage.md`, `industry-baseline-refresh.md`, `lane-bypass-emergency.md`, `retention-conflict-spike.md`, `projection-stale.md`, `regulator-export-failed.md`, `wasmtime-filter-stuck.md`.
8. Sit in on the Wednesday governance-substrate handoff.

Acceptance: you can sketch the ingestion path: µservice audit-chain emit → Kafka topic `oya.<svc>.audit.*` → governance ingestion worker verifies source_hash against audit-chain anchor → projection row inserted in `GovernanceEvidenceIndex` → ClickHouse columnar replica updated → `governance.evidence.freshness.green.v1` emitted. And the retention decision path: audit event → retention evaluator collects candidate `PackRetentionPolicy` rules → applies precedence (higher-restriction wins; legal hold blocks) → produces `RetentionDecision` row → `governance.retention.decision.evaluated.v1` emitted → delete worker honors decision at expiry.

## Day 2 — demo_trial tenant_class cell bootstrap + first retention policy

```sh
# Submit governance bootstrap request through the control plane.
# Inputs: tenant_class=demo_trial, cell=drill-syd-1, packs=kr,eu
# Evidence: evidence/multispectrum/governance-first-week-<date>.json
```

Expected runtime: ≤ 12 min. Verify:

```sh
oya governance health --cell drill-syd-1
# Expected:
#   postgres.governance_evidence_index: up (lag_ms=12)
#   clickhouse.evidence_columnar: up
#   kafka.governance-ingestion: connected (consumer lag=0)
#   valkey.retention-decision-cache: up
#   audit-chain.source: up
#   projection_version: governance-evidence-index-v1
```

Create a tenant retention policy:

```sh
# 1. Configure tenant pack retention
oya governance retention policy create \
    --tenant drill-acme \
    --pack-id default \
    --event-class "drive.file.uploaded.v1" \
    --data-class PII_SENSITIVE \
    --minimum-duration 365d \
    --max-duration 7y \
    --legal-hold-behavior block-delete \
    --delete-approval-class dual_approval
# Output:
#   policy_id: rp_drill_001
#   pack_id: default
#   effective_from: 2026-05-20T00:00:00Z
#   audit_event_id: ae_gov_retention_policy_001

# 2. Add another retention policy for a stricter pack
oya governance retention policy create \
    --tenant drill-acme \
    --pack-id hipaa \
    --event-class "drive.file.uploaded.v1" \
    --data-class PII_SENSITIVE \
    --minimum-duration 6y \
    --max-duration 6y \
    --legal-hold-behavior block-delete \
    --delete-approval-class regulator_attested
# Output:
#   policy_id: rp_drill_002

# 3. Now query an event's effective retention
oya governance retention evaluate \
    --tenant drill-acme \
    --source-event-id ae_drive_file_uploaded_alice_001 \
    --data-class PII_SENSITIVE
# Output:
#   candidate_rules: [rp_drill_001 (default), rp_drill_002 (hipaa)]
#   winning_rule: rp_drill_002 (hipaa)
#   effective_retention_until: 2032-05-20T00:00:00Z
#   reason: higher-restriction-wins (hipaa: 6y vs default: 1y minimum)
```

Acceptance: cell bootstrap + retention policies + retention decision verified.

## Day 3 — Aggregation replay + projection rebuild

Suppose the projection becomes stale (e.g., schema drift, partial index corruption). Run a replay:

```sh
oya governance aggregation replay start \
    --tenant drill-acme \
    --from-event-time 2026-05-01T00:00:00Z \
    --to-event-time 2026-05-20T23:59:59Z \
    --projection-version governance-evidence-index-v1 \
    --partition-topic governance.evidence.partition.drill-syd-1.shard-001 \
    --replay-rate-per-sec 5000
# Cedar evaluates:
#   - governance::aggregation::replay ✓
#   - operator role ✓
# Output:
#   replay_job_id: rep_drill_001
#   projection_version: governance-evidence-index-v1
#   estimated_events: 1 247 821
#   estimated_duration: 4 min
#   audit_event_id: ae_gov_replay_started_001

# Monitor
oya governance aggregation replay watch --job rep_drill_001
# (streamed)
#   rep_drill_001: 250k/1247k events (20%)
#   rep_drill_001: 500k/1247k events (40%)
#   rep_drill_001: 750k/1247k events (60%)
#   rep_drill_001: 1000k/1247k events (80%)
#   rep_drill_001: 1247k/1247k events (100%) - completed
#   audit_event_id: ae_gov_replay_completed_001
#   high_watermark: 2026-05-20T23:59:59Z
```

Verify the projection freshness:

```sh
oya governance projection status --tenant drill-acme
# Output:
#   projection_version: governance-evidence-index-v1
#   partitions:
#     - partition_id: governance.evidence.partition.drill-syd-1.shard-001
#       high_watermark: 2026-05-20T23:59:59Z
#       freshness_state: green
#       last_replay_at: 2026-05-20T14:32:17Z
#       events_count: 1 247 821
#   total_events_indexed: 1 247 821
#   total_freshness_lag_seconds_p95: 18
```

Acceptance: replay drill complete; projection rebuilt.

## Day 4 — Cross-pack conflict + retention shortening

Simulate a cross-pack conflict (paid tenant_class feature; shadow at demo_trial tenant_class):

```sh
# 1. List active retention conflicts for the tenant
oya governance retention conflicts list --tenant drill-acme
# Output:
#   - conflict_id: cf_drill_001
#     event_class: drive.file.uploaded.v1
#     data_class: PII_SENSITIVE
#     candidate_rules:
#       - rule_id: rp_drill_001 (default pack; minimum 365d)
#       - rule_id: rp_drill_002 (hipaa pack; minimum 6y)
#     winning_rule_id: rp_drill_002
#     reason_code: higher_restriction_wins
#     transparency_report_ref: tr_drill_001
#     emitted_at: 2026-05-20T14:32:17Z
```

Generate transparency report:

```sh
oya governance retention conflict transparency-report \
    --conflict cf_drill_001 \
    --include-legal-basis true
# Output:
#   report:
#     conflict_id: cf_drill_001
#     winning_rule_id: rp_drill_002
#     pack: hipaa
#     legal_basis: "45 CFR § 164.530(j); HIPAA Privacy Rule documentation retention 6 years from creation"
#     losing_rule_id: rp_drill_001
#     losing_pack: default
#     restriction_level_winning: 6 (hipaa)
#     restriction_level_losing: 3 (default)
#     decision_owner: axis-compliance
#     historical_precedent: "ADR-COMP-001 § Decision precedence-step-3-higher-restriction-wins"
```

Now demonstrate retention shortening (dual-approval requirement):

```sh
# Try to shorten without dual approval (DENIED)
oya governance retention policy shorten \
    --policy-id rp_drill_002 \
    --new-minimum-duration 1y \
    --requesting-principal u-compliance-admin@drill.test
# Cedar denies (per ADR-GOV-001 § Decision):
#   - governance::retention::shorten requires dual_approval
#   - currently only 1 approver
# Output: 403 Forbidden

# Provide both approvals
oya governance retention policy shorten \
    --policy-id rp_drill_002 \
    --new-minimum-duration 1y \
    --requesting-principal u-compliance-admin@drill.test \
    --co-approver u-legal-counsel@drill.test \
    --justification "HIPAA retention requirement does not apply to this synthetic test data"
# Cedar evaluates:
#   - governance::retention::shorten ✓
#   - dual approvers present ✓
#   - no active legal hold ✓
#   - projection freshness green ✓ (per ADR-GOV-001 § Decision)
# Output:
#   policy_id: rp_drill_002
#   from_minimum_duration: 6y
#   to_minimum_duration: 1y
#   approved_by: [u-compliance-admin@drill.test, u-legal-counsel@drill.test]
#   soak_period: 60s
#   effective_at: 2026-05-20T14:33:17Z
#   audit_event_id: ae_gov_retention_shortened_001
```

Acceptance: cross-pack conflict + transparency report + dual-approval shortening verified.

## Day 5 — Bypass grants + lane bypass emergency runbook

Walk the lane-bypass-emergency runbook. Read `runbooks/lane-bypass-emergency.md`. Scenario: a CI lane is erroneously failing; engineer needs to bypass to ship a critical hotfix.

```sh
# 1. Request bypass with expiry + justification
oya governance bypass grant create \
    --tenant drill-acme \
    --lane-id docs/decisions/ADR-0329-ADR-0330-ADR-0331-tenant-class-adoption \
    --action "skip-doc-coverage-check" \
    --reason "Hotfix shipping; doc-coverage check has false positive on auto-generated file" \
    --requested-by u-engineer@drill.test \
    --expires-at 2026-05-21T14:32:17Z \
    --approver-1 u-team-lead@drill.test \
    --approver-2 u-engineering-director@drill.test
# Cedar evaluates:
#   - governance::bypass::grant ✓
#   - dual approver present ✓
#   - expiry within pack maximum (24h) ✓
# Output:
#   grant_id: bg_drill_001
#   expires_at: 2026-05-21T14:32:17Z
#   audit_event_id: ae_gov_bypass_granted_001
```

The bypass is now active. Use it:

```sh
# CI lane sees the bypass grant and proceeds
oya lane run docs-coverage \
    --tenant drill-acme \
    --bypass-grant bg_drill_001
# Output: lane passed via bypass
```

After the expiry:

```sh
# Try to use bypass after expiry
oya lane run docs-coverage \
    --tenant drill-acme \
    --bypass-grant bg_drill_001
# Cedar denies (bypass expired)
# Output: 403 Forbidden
```

Active bypasses are reviewed in the daily report:

```sh
oya governance bypass list --tenant drill-acme --status active
# Output:
#   - grant_id: bg_drill_001
#     lane_id: docs/decisions/ADR-0329-ADR-0330-ADR-0331-tenant-class-adoption
#     state: expired (auto-revoked)
#     expired_at: 2026-05-21T14:32:17Z
```

Acceptance: bypass grant + auto-expiry verified; runbook walked.

## What you've learned

- demo_trial tenant_class bootstrap + retention policy creation.
- Cross-pack retention conflict resolution (higher-restriction-wins).
- Aggregation replay from audit-chain.
- Transparency reports for conflict decisions.
- Retention shortening with dual approval.
- Bypass grants with expiry + dual approval.

Next week: paid tenant_class promotion (cross-region projection convergence + 1M-event-sec ingestion), paid tenant_class scaled deployment tour (lane runtime acceleration + cross-µservice fan-in + regulator dashboards), paid tenant_class sovereign-pack tour (per-pack projection residency + regulator-observable retention shortening + 7-y horizon), and your first production shadow on a retention policy approval.
