---
doc_class: Runbook
title: Audit-chain bridge backlog — substrate unavailable or slow
microservice: foundry-evidence
severity: Sev-2 (with Sev-1 escalation if sustained)
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-evidence
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-02, FM-03)
  - microservices/intelligence/policy/evidence-pack-integrity.md (EPI-10)
  - microservices/intelligence/incident-response.md
  - microservices/audit-chain/runbooks/audit-chain-restart.md (substrate)
doc_status: published
---

# Runbook: Audit-chain bridge backlog

## Purpose

Recovery procedure when the bridge between foundry-evidence pack-builder and the audit-chain substrate develops a backlog. Two trigger classes:
- FM-02: audit-chain substrate is fully unavailable → packs queue in foundry-evidence dead-letter store.
- FM-03: audit-chain substrate is degraded (slow seal) → bridge worker keeps draining but slips its SLO.

Per `policy/evidence-pack-integrity.md` EPI-10, `record_invocation` still returns receipt to the caller during backlog with `sealed=false`. The caller is not blocked. Backlog drain must complete within SLO.

## Trigger

- `oya:foundry_evidence_audit_chain_emit_backlog_depth_seconds` > 60 for 5 min (Sev-2 page).
- `oya:foundry_evidence_audit_chain_emit_backlog_depth_seconds` > 600 for 10 min (Sev-1 page).
- `oya:foundry_evidence_audit_chain_emit_failure_rate` > 0.01 for 5 min (Sev-2 page).

## Severity

- **Sev-2** for normal backlog or degraded substrate.
- **Sev-1** if backlog > 10 min sustained OR if audit-chain substrate has declared Sev-1 itself (escalation joins).

## Procedure

### Phase 1: Triage (≤ 5 min)

1. Identify whether the substrate is the proximate cause or the bridge is:
   ```
   # Substrate health
   curl -sf https://audit-chain.<pack>.internal/health/ready
   # Bridge worker
   kubectl logs -n foundry-evidence deploy/audit-chain-bridge --tail=200
   # Backlog depth
   curl -s 'https://mimir.<pack>.internal/api/v1/query?query=oya_foundry_evidence_audit_chain_emit_backlog_depth_seconds'
   ```
2. If substrate is the cause → engage axis-audit-chain on-call + follow `microservices/audit-chain/runbooks/audit-chain-restart.md`.
3. If bridge is the cause → continue Phase 2 below.

### Phase 2: Bridge diagnosis (≤ 15 min)

| Suspect | Indicator | Action |
|---|---|---|
| Bridge pod CPU exhaustion | container CPU > 90% sustained | scale: `kubectl scale deploy/audit-chain-bridge --replicas=+50%` |
| Bridge pod OOM | recent OOMKilled in events | bump memory request via Helm `evidence-builder` values; redeploy |
| Network partition | `kubectl exec ... -- nc -zv audit-chain.<pack>.internal 443` fails | engage netops; verify SPIFFE rotation hasn't broken mTLS |
| SPIFFE rotation lag | bridge logs show `auth_failed` | engage cloud-secrets; force SPIFFE re-issue |
| Idempotency-key dedup storm | `oya_foundry_evidence_dedup_collision_total` spiking | investigate caller; foundry-runtime may be retrying with reused idempotency_key |
| Cedar policy compile slow | bridge bootstrap shows long policy load | check recent policy reload; rollback if needed |

### Phase 3: Substrate engagement (≤ 30 min)

If bridge is healthy but substrate is the bottleneck:

1. Join the audit-chain Sev-2 channel; pair on substrate recovery.
2. Verify substrate WAL is not lagging via `oya audit-chain emit-wal-depth --pack <pack>`.
3. If substrate recovery is > 1 h, council-privacy assesses whether to enable the **degraded-receipt mode**:
   - Receipts still issued with `sealed=false`.
   - Dashboards + portals indicate "pending seal" for affected packs.
   - audit-chain bridge worker drains as fast as substrate accepts.

### Phase 4: Drain + verify (≤ 1 h after substrate recovery)

1. Bridge worker resumes draining the dead-letter store under bounded back-off.
2. Monitor backlog depth: target ≤ 60 s within 10 min of substrate recovery.
3. Verify every drained pack now carries an `audit_event_id`:
   ```
   psql ... -c "SELECT count(*) FROM evidence_pack WHERE audit_chain_emit_pending=true;"
   ```
   Expect: drops to 0 (or to the in-flight working set, which is < 100).
4. Any pack that fails after 24 h of bounded retries is moved to the **permanent dead-letter** + emits a `foundry.evidence.pack.assembly_failed.v1` event with `reason=substrate_persistent_unavailable` → Sev-1 page to council-privacy.

### Phase 5: Tenant + regulator comms (≤ 4 h)

1. If backlog exceeded 10 min and regulator engagement is active:
   - council-privacy notifies regulator on the engagement channel.
2. If tenants observed `sealed=false` for > 1 h via `evidence_query`:
   - tenancy DPA-bound notification.
3. Postmortem within 5 business days.

## Halt conditions

- Bridge worker repeatedly OOMing despite scale-up → halt; engage axis-foundry-evidence to bug-hunt.
- Substrate signature verification of own outbound emits begins to fail → halt; treat as substrate-side compromise (escalate audit-chain Sev-1).
- Bridge worker hitting Cedar deny → halt; investigate why SPIFFE→bound_microservice no longer matches policy expectations.

## Verification (post-recovery)

- Backlog depth back to < 60 s sustained ≥ 30 min.
- Permanent dead-letter empty.
- All affected packs visible in `evidence_query` with `sealed=true` + audit_event_id.
- Postmortem published.

## References

- `microservices/intelligence/policy/evidence-pack-integrity.md` EPI-10.
- `microservices/intelligence/failure-modes.md` FM-02 + FM-03.
- `microservices/audit-chain/runbooks/audit-chain-restart.md` (substrate restart procedure).
- ADR-0028 (audit-chain Merkle/Ed25519).
