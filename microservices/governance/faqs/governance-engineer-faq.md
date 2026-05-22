---
doc_class: FAQ
microservice: governance
persona: governance-engineer + compliance-engineer + sre-evidence-engineer
related_adrs: [ADR-GOV-001, ADR-0003, ADR-0010, ADR-0128]
date: 2026-05-20
doc_status: published
---

# Governance Engineer FAQ — governance

## Why is governance a projection over audit-chain instead of the source of truth?

Per ADR-GOV-001 § Decision + ADR-0003. Two principles:

1. **Audit-chain is immutable + cryptographically signed**: every event in audit-chain is Ed25519-signed by the emitting µservice. Tampering would break the chain.
2. **Governance is query-optimized**: ClickHouse columnar + per-tenant partitioning + 90-day hot index gives auditors fast queries.

If governance were the source of truth, it would need to handle both query optimization AND immutability + cryptographic integrity, which conflict (mutable indexes vs immutable chains). Separating them gives strong audit-chain semantics + fast queries.

Replay from audit-chain rebuilds projection if it gets corrupted; the chain is always recoverable. The Cedar `governance::aggregation::replay` permit gates this.

## How does higher-restriction-wins work in retention?

Per ADR-GOV-001 § Decision + ADR-COMP-001. When multiple packs apply:

1. Each pack rule has `restriction_level` (0-10; higher = stricter).
2. For each event_class + data_class, candidate rules are collected.
3. Highest `restriction_level` wins.

Example: tenant subscribes to "default" pack (90-d retention) + "hipaa" pack (6-y retention). For `drive.file.uploaded.v1` events with `data_class=PHI`:

- default pack: 90-d minimum (restriction_level=3)
- hipaa pack: 6-y minimum (restriction_level=8)

Higher-restriction-wins: hipaa wins; effective retention = 6 y.

A weaker (later-added) pack CAN NOT weaken a stricter (existing) pack. This is enforced at policy-create time.

## Why does retention shortening require dual approval?

Per ADR-GOV-001 § Decision + Constraint GOV-C12. Shortening retention deletes data:

- Loss of compliance evidence (auditor cannot reconstruct).
- Loss of legal-hold-defensible record (regulator can subpoena, expects evidence).
- Loss of forensic capability (post-incident review).

Dual approval (two named approvers with `governance::retention::shorten` permission) prevents:
- Single compromised principal from destroying evidence.
- Honest mistakes (typo on retention class).

The Cedar policy enforces this:

```cedar
permit (principal, action == Action::"governance::retention::shorten", resource)
when {
    context.approvers.length() >= 2 &&
    context.approvers.all(a => has_permission(a, "governance::retention::shorten_co_approve"))
};
```

Cross-pack conflict resolution ALSO never shortens — it can only RAISE retention (higher-restriction-wins).

## How does aggregation replay work?

Per ADR-GOV-001 § Decision + IP-013-aggregation-index-generation-lane. Replay is:

1. Idempotent by source_event_id + projection_version (per ADR-GOV-001 § Decision).
2. Driven by partition (typically by tenant_id_hash range).
3. Throttle-controlled (max events/sec to avoid Kafka backpressure).
4. Source-hash-verified (each event verified against audit-chain anchor).

Replay scenarios:
- **Schema drift**: governance projection schema changed; need to re-index existing events.
- **Partition corruption**: ClickHouse merge tree corruption (rare); rebuild from audit-chain.
- **Onboarding new pack**: tenant subscribed to a new pack; need to re-evaluate retention.
- **Onboarding new µservice**: existing µservice added new event class; need to backfill index.

Replay throughput target: ≥ 5 000 events/sec/partition. Larger replays are partitioned + parallelized.

## What is "evidence freshness lag" and why does it matter?

Per ADR-GOV-001 metric `governance_evidence_freshness_lag_seconds`. The lag between when an event is emitted to audit-chain and when it's queryable in the governance projection.

Why it matters:
- Auditors query the projection (not audit-chain directly; faster).
- Stale projection = auditor sees outdated state.
- Retention decisions depend on freshness (per ADR-GOV-001 § Decision: "Deny destructive retention actions while projection freshness is red").
- Compliance scorecards depend on real-time evidence.

Target: p95 ≤ 60 s at demo_trial tenant_class; ≤ 30 s at paid tenant_class scaled deployment.

If lag > threshold: Cedar denies destructive ops; alert fires; investigate (Kafka consumer lag, ClickHouse merge tree, etc.).

## How does cross-microservice evidence fan-in work?

Per IP-011-cross-microservice-evidence-fan-in. paid tenant_class feature.

A single query can join evidence from drive + messenger + mail + calendar + identity + tenancy + compliance + audit-chain:

```sh
oya governance evidence query \
    --tenant acme-corp \
    --principal u-alice@acme-corp.com \
    --time-range 2026-05-01..2026-05-20 \
    --include-microservices drive,messenger,mail,calendar,identity \
    --output-format json
# Output: union of all events involving u-alice across all µservices
```

The fan-in is server-side; client sees a unified view. ClickHouse columnar + per-tenant materialized views make this efficient.

Use cases:
- Investigating a user's activity across all systems.
- DSAR (Data Subject Access Request) per GDPR Art 15.
- eDiscovery for a litigation hold.
- Forensic post-incident reconstruction.

## How does the industry-baseline conformance lane work?

Per IP-011-industry-best-practice-conformance-lane. The lane auto-evaluates each µservice's compliance with:

- **SOC 2** Trust Service Criteria (Security, Availability, Processing Integrity, Confidentiality, Privacy).
- **ISO 27001** Annex A controls.
- **HIPAA** Security Rule § 164.308-312.
- **PCI DSS** v4.0.
- **GDPR** Art 25 (Privacy by Design) + Art 32 (Security of Processing).
- **NIST 800-53** moderate baseline.

Each µservice's controls (e.g., "all access requires Cedar" or "all data encrypted with per-tenant CMK") are mapped to baseline rules. The lane runs nightly + emits conformance scorecards.

Output: per-tenant per-pack conformance percentage + missing control list.

## How does the lane runtime work (IP-004)?

Per IP-004-lane-runtime-kernel-domain. Lane runtime is the per-pack evaluation engine:

- Each pack defines a set of CI lanes (e.g., for HIPAA: PHI-data-class-coverage, BAA-evidence-presence, audit-chain-completeness).
- Lanes execute in parallel where possible.
- Per-lane state: queued → running → green/red.
- Per-lane evidence emitted to audit-chain.

Lane scheduling optimization at paid tenant_class scaled deployment: parallel execution + lane dependency graph.

## What's the bypass-grant model (ADR-GOV-001 § Decision)?

Per ADR-GOV-001 § Decision. A `GovernanceBypassGrant` records:

- `grant_id`, `tenant_id`, `lane_id`, `action`, `reason`.
- `approved_by` (dual; per pack policy).
- `expires_at` (mandatory; no open-ended bypasses).
- `audit_event_id`.

Lifecycle:
- Engineer requests bypass with justification.
- Two approvers (named principals) review + approve.
- Bypass active for the requested window (capped at pack maximum, typically 24 h).
- Cedar permits the bypassed action during the window.
- Auto-revocation at expiry.
- Daily report of active bypasses.

Bypasses are visible to auditors. Frequent bypasses on the same lane suggest the lane is mis-tuned + needs review.

## How does the regulator dashboard work (paid tenant_class)?

Per paid tenant_class capability + IP-regulator-dashboards. Per-pack regulator views:

- HIPAA: audit-trail completeness, encryption posture, BAA status, breach notification SLAs.
- GDPR: DSAR response times, breach notification (Art 33), Art 32 security posture, Art 25 by-design + by-default.
- SOX: financial-data immutability evidence, control-test results, segregation-of-duties violations.

Each dashboard has a "regulator export" function: bundles relevant evidence into a signed report bundle (Ed25519-signed by governance issuer key). Regulator can verify signature; auditor can re-export.

## How does the projection handle high-cardinality tenant labels?

Per ADR-GOV-001 § Decision Constraint GOV-C9 + tenant ID hash-tokenization. Two strategies:

1. **Hash-tokenized in metrics**: `tenant_id_hash` (BLAKE3-256) used in Prometheus + Grafana metrics. This bounds cardinality for monitoring.
2. **Encrypted in queries**: full `tenant_id` available for Cedar-scoped queries but always presented under permission gate.

This prevents the monitoring infrastructure from becoming a data-leak vector (e.g., Prometheus scrapes leaking tenant existence).

## How does cross-jurisdictional governance work?

Per ADR-GOV-001 sovereign path + ADR-0010 regional pack architecture. Multi-jurisdiction tenant:

- Pack residency drives projection residency.
- Cross-region projection: metadata-only (no raw payload pointers cross pack boundaries).
- Cross-jurisdictional transfer evidence (PIPL Art 38, GDPR Art 49, KR-PIPA Art 28) required for any cross-region query.

At paid tenant_class sovereign-pack scope, governance projections stay in jurisdiction; central views are federated summaries (not joinable raw data).

## How does the legal-hold pipeline work?

Per ADR-GOV-001 § Decision + IP-005-legal-hold-pipeline. Legal hold:

- Applied to tenant + event class + data class (typically all data of a tenant during litigation).
- Blocks retention-based deletion regardless of pack rule.
- Lifts only via explicit release ceremony with legal counsel attestation.
- Audit-chain emits `governance.legal-hold.{applied,released}.v1`.

Per ADR-GOV-001 § Implementation Notes: "Cedar forbid `governance::retention::delete` when `resource.has_active_legal_hold == true`".

## How is migration from Drata / Vanta handled?

See `migration-playbooks/from-drata.md` for the full playbook. Short version:

1. Export Drata/Vanta evidence + control mappings.
2. Run `oya governance migrate import-drata` (creates retention policies + pack subscriptions).
3. Re-issue auditor access via Cedar + identity µservice.
4. Phased cutover: lane-by-lane CI integration.
5. Decommission Drata/Vanta after retention window.
