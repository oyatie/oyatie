# Runbook: regional-sovereignty-violation

- Severity: P0 (sovereignty + potential regulatory disclosure)
- Trigger:
  - SLO `sovereignty-violation-zero` budget breach (target=1.0; any non-zero violation is breach).
  - `oya_consent_graph_sovereignty_violation_total` increment.
  - Nightly sovereignty audit job detects Pulsar topic in wrong region OR projection cache in
    forbidden region.

## Step 1 — Acknowledge + halt (≤5min)

1. Page security on-call + privacy officer + DPO.
2. Auto-suspend affected agreement(s) (already automatic on kernel invariant fail; verify):
   `oya consent-graph agreement suspend <id> --reason SecurityIncident`.
3. Destroy affected projection topic(s):
   `oya consent-graph projection destroy <topic-id> --force`.

## Step 2 — Identify scope (≤15min)

1. Query: which agreements + which topics + which regions?
   `oya consent-graph sovereignty-audit --since <ts>`.
2. For each: pull agreement → grantor.sovereignty.grantor_region vs topic.region.
3. List grantee subscribers that may have read data from wrong-region topic.

## Step 3 — Classify root cause

| Root cause | Action |
|------------|--------|
| Pulsar admin misconfigured georep | revoke replication; verify topic destroyed in non-grantor region |
| Code bug emitted to wrong region | code fix + replay missed sovereignty-asserts |
| Manual operator action (SRE bypass) | conduct discipline review; ADR-SVC-CG-* on operator-runtime guardrails |
| Cross-region failover misconfig | review failover scripts; runbook update |

## Step 4 — Forensic snapshot (≤30min)

1. Snapshot Pulsar admin logs for the affected topic.
2. Snapshot all consume events from that topic (Pulsar message-id range).
3. Snapshot Postgres rows for affected agreement.
4. Generate forensic report `evidence/sovereignty-violation-<id>.json`; seal in audit-chain.

## Step 5 — Data-residency remediation (≤4h)

1. Verify grantee-side projection cache (if any) is tombstoned. If not, force-tombstone:
   `oya ontology cross-tenant tombstone --agreement <id> --grantee-region <region>`.
2. Verify Pulsar topic destroyed in wrong region.
3. Verify any in-region pod logs containing the violating event were rotated to retention storage
   *and that retention storage is in correct jurisdiction*.

## Step 6 — Regulatory disclosure (≤72h)

Per `compliance.md` per-pack:
- GDPR: supervisory authority within 72h (DPO leads).
- KR PIPA: KCC within 24h.
- HIPAA: HHS + individuals within 60d if PHI involved.
- Other packs: per jurisdiction.

Disclosure threshold: ANY confirmed sovereignty violation = breach. No de minimis.

## Step 7 — Customer notification

- Affected grantor + grantee notified within 24h via designated security contacts.
- Public notification only if regulatory mandate triggers.

## Step 8 — Recovery (≤24h post-mitigation)

1. Code fix deployed (if root cause was code).
2. Run sovereignty-audit job ad-hoc; verify zero remaining violations.
3. Lift agreement suspensions only after audit-officer + DPO sign-off.
4. Re-mint affected projection topics (in correct region).
5. Re-handshake affected partner if their side received the data.

## Step 9 — Post-mortem (≤7d)

- 5-whys.
- Action items: code changes, runbook changes, monitoring additions, ADR-SVC-CG-* if structural.
- DPIA revisit: does the incident change risk treatment?

## Verification

- Sovereignty audit job over 7d window post-recovery: zero violations.
- Synthetic test: try to emit event to wrong-region topic; kernel invariant blocks it.

## Audit evidence

- Forensic snapshot.
- Disclosure events.
- Post-mortem doc.
- All sealed in audit-chain.
