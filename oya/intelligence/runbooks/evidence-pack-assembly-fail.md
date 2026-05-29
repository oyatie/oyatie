---
doc_class: Runbook
title: Pack assembly failure — partial signal, schema drift, or builder crash
microservice: foundry-evidence
severity: Sev-2 (with Sev-1 escalation if sustained)
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-evidence
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-04, FM-05)
  - microservices/intelligence/policy/evidence-pack-integrity.md (EPI-01, EPI-02, EPI-06)
  - microservices/intelligence/incident-response.md
doc_status: published
---

# Runbook: Pack assembly failure

## Purpose

Recovery procedure when pack-builder fails to assemble a pack for one or more invocations. Three failure modes:
- FM-04: missing signal — one of foundry-runtime / eval / guardrails / supervisor signals never arrived within the assembly window.
- FM-05a: schema drift — a source µservice publishes an envelope version foundry-evidence does not recognise.
- FM-05b: pack-builder process crash mid-assembly.

## Trigger

- `oya:foundry_evidence_pack_assembly_failure_rate` > 0.001 for 5 min (Sev-2 page).
- `oya:foundry_evidence_pack_assembly_failure_rate` > 0.01 for 5 min (Sev-1 page).
- Single failure with `failure_class=schema_drift` (Sev-2 page; one-off).

## Severity

- **Sev-2** for normal recoverable failures.
- **Sev-1** if sustained > 5 min OR if failure_class=schema_drift on a stable contract.

## Procedure

### Phase 1: Triage (≤ 10 min)

1. Open the pack-assembly dashboard:
   ```
   open https://grafana.<pack>.internal/d/pack-assembly-rate
   ```
2. Classify failures by `failure_class` label:
   ```
   sum by (failure_class) (rate(oya_foundry_evidence_pack_assembly_failed_total[5m]))
   ```
3. Top-3 most common failure classes determine procedure branch.

### Phase 2A: Missing-signal (FM-04) procedure

1. Identify which signal source is missing:
   ```
   sum by (missing_signal_source) (rate(oya_foundry_evidence_pack_assembly_failed_total{failure_class="missing_signal"}[5m]))
   ```
2. Source-µservice health check:
   ```
   curl -sf https://<source-microservice>.<pack>.internal/health/ready
   ```
3. If source is healthy but no signals → check Workflow event-bus delivery:
   ```
   oya workflow events backlog --topic foundry.<source>.* --pack <pack>
   ```
4. If signals are stuck in transit → engage workflow-engine on-call.
5. If source is unhealthy → engage that source's on-call (axis-foundry-runtime / axis-foundry-eval / axis-foundry-guardrails / axis-foundry-supervisor).
6. Hold pack-assembly worker open for late-arriving signals via the `late_signal_grace_window` (default 60 s, configurable per-tenant). Beyond grace window, pack is assembled with `partial=true` and `missing_sources=[…]`; this is honest representation (no fabrication).

### Phase 2B: Schema drift (FM-05a) procedure

1. Identify the drifted source:
   ```
   sum by (drifted_source, observed_schema_version, expected_schema_version) \
     (rate(oya_foundry_evidence_pack_assembly_failed_total{failure_class="schema_drift"}[5m]))
   ```
2. Schema drift on a stable contract is a P0 governance violation:
   - LEAN lane `no-silent-regression` should have blocked it; investigate why not.
   - Engage governance + the drifted source's owner.
3. Roll the drifted source back to last-known-good schema version (preserves contract).
4. If the drift is intentional (planned schema bump) and contracts/sunset window are honoured:
   - Verify a new pack-builder release supports the new schema version.
   - Stage rollout per `docs/standards/promotion-readiness-lane.md`.

### Phase 2C: Builder crash (FM-05b) procedure

1. Builder pods crashing → check recent events:
   ```
   kubectl get events -n foundry-evidence --sort-by='.lastTimestamp' | head -20
   kubectl logs -n foundry-evidence deploy/evidence-pack-builder --previous --tail=300
   ```
2. Identify crash class:
   - OOM → bump memory request; redeploy.
   - Panic on unrecognised input → file P0 bug; quarantine the offending invocation envelope (it must be inspected, not silently dropped).
   - Substrate exhaustion (Postgres connection pool) → engage cloud-secrets + bump pool size per `iac/helm/evidence-builder/values.yaml`.
3. If the crash is reproducible on a specific envelope → preserve that envelope in a forensic bucket + Cedar-gated; engage axis-foundry-evidence to bug-hunt.

### Phase 3: Backlog drain (≤ 30 min after fix)

1. Builder resumes pulling envelopes from queue under bounded back-off.
2. Monitor `oya_foundry_evidence_pack_assembly_failure_rate` → returns to baseline within 10 min.
3. Failed envelopes that exceeded the late_signal_grace_window are assembled `partial=true` and audit-emitted with `foundry.evidence.pack.assembled.v1` carrying `partial=true`.
4. Verify the `partial=true` packs are correctly indexed in evidence-query:
   ```
   oya foundry-evidence evidence query --partial-only --pack <pack> --since <ts>
   ```

### Phase 4: Notify (≤ 4 h)

1. If `partial=true` packs are part of an active regulator engagement → council-privacy notifies regulator.
2. If tenant-portal showed assembly errors to tenant operators → tenancy notification.
3. Postmortem within 5 business days for Sev-1 (> 5 min sustained or schema drift on stable contract).

## Halt conditions

- Multiple sources simultaneously dropping signals → join workflow-engine Sev-1.
- Schema drift detected on a contract that has not yet entered its sunset window → halt rollout; engage governance.
- Builder process repeatedly OOMing despite scale-up → halt; engage axis-foundry-evidence.
- A specific envelope crashes the builder repeatedly even after isolation → quarantine; engage; investigate possible adversarial input.

## Verification (post-recovery)

- Failure rate back to baseline ≤ 0.0001 sustained 30 min.
- All partial-true packs documented in incident artifacts.
- Postmortem published.
- Any quarantined envelopes inspected + classified (legitimate-but-unhandled vs adversarial vs bug).

## References

- `microservices/intelligence/policy/evidence-pack-integrity.md` EPI-01 + EPI-02 + EPI-06.
- `microservices/intelligence/failure-modes.md` FM-04 + FM-05.
- `docs/standards/observability-slo.md`.
- ADR-0024 (eval-evidence integration; partial-pack rules).
