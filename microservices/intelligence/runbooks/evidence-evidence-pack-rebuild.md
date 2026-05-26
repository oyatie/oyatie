---
doc_class: Runbook
title: Evidence pack rebuild — late-signal or corrupted-pack recovery
microservice: foundry-evidence
severity: Sev-2 (with Sev-1 escalation path)
status: Accepted
owner_team: axis-foundry-evidence + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-01, FM-06)
  - microservices/intelligence/policy/evidence-pack-integrity.md (EPI-02, EPI-11)
  - microservices/intelligence/incident-response.md
  - microservices/audit-chain/policy/seal-integrity.md (substrate; SI-07)
doc_status: published
---

# Runbook: Evidence pack rebuild

## Purpose

Recovery procedure for two scenarios:
- FM-01: late signal arrived after pack was already assembled, and the late signal is materially significant (e.g., a guardrail decision that was missed in the original assembly).
- FM-06: a pack's content was identified as incorrect (eval-verdict join error, attribution mistake) and a corrected pack must be issued.

Per `policy/evidence-pack-integrity.md` EPI-11, rebuilds NEVER write to historical audit-chain periods. They write a NEW pack at the current period with `supersedes_pack_ref` pointing at the original.

## Trigger

- `oya.foundry_evidence.late_signal.v1` events with `materially_significant=true`.
- Manual escalation from `ops-security` after evidence review.
- Eval-evidence join correctness drill failure.

## Severity

- **Sev-2** for normal late-signal rebuilds.
- **Sev-1** escalation if the original pack is suspected of intentional tampering or if the rebuild affects an active regulator engagement.

## Procedure

### Phase 1: Engage + Halt downstream consumers (≤ 15 min)

1. Declare Sev-2 (or Sev-1 per trigger); open `#inc-<id>` Slack.
2. Engage IC (axis-foundry-evidence on-call) + ops-security SME + council-privacy if regulator-engaged.
3. Halt any in-flight `regulator-export` workflows that scope the original pack:
   ```
   cargo run -p oya-dev-cli -- foundry-evidence regulator-export pause \
     --invocation-id <id> --reason "rebuild-in-flight"
   ```
4. Mark the original pack as `superseded_pending=true` in Postgres via the retention-cascade RPC (Cedar-gated; requires 2-person rule).

### Phase 2: Reconstruct (≤ 1 h)

1. Pull all signals from source µservices for `(tenant_id, invocation_id, attempt_no)`:
   - `oya foundry-runtime invocation get --invocation-id <id>`
   - `oya foundry-eval verdict-at --invocation-id <id>`
   - `oya foundry-guardrails decisions-for --invocation-id <id>`
   - `oya foundry-supervisor autonomy-decision-for --invocation-id <id>`
2. Verify each source signal carries a SPIFFE signature matching its bound µservice.
3. Run `oya foundry-evidence pack-rebuild dry-run --invocation-id <id>` to produce the new pack envelope without emitting.
4. Diff dry-run vs original pack; record the diff in the incident channel.

### Phase 3: 2-person rule sign-off

1. Rebuilder (axis-foundry-evidence on-call) prepares rebuild request envelope: `{invocation_id, supersedes_pack_id, justification, signal_sources, signal_signatures}`.
2. Approver (distinct ops-security or council-privacy principal) reviews + countersigns.
3. Cedar `policy/regulator-export-scope.cedar`-shaped `approver_principal != requester_principal` check enforced at submission.

### Phase 4: Emit + verify (≤ 10 min)

1. Submit rebuild:
   ```
   oya foundry-evidence pack-rebuild submit \
     --invocation-id <id> \
     --approver <approver-spiffe> \
     --justification-file justification.txt
   ```
2. Verify new pack is sealed by audit-chain:
   ```
   oya foundry-evidence evidence query --invocation-id <id> --include-superseded
   ```
   Expect: two packs returned, ordered by emission time; latest has `supersedes_pack_ref=<original_pack_id>`.
3. Verify audit-chain bridge emitted both `foundry.evidence.pack.assembled.v1` (new pack) and `foundry.evidence.pack.superseded.v1` (original) events.

### Phase 5: Notify (≤ 1 h)

1. If a regulator engagement is active for this `(tenant, framework)`:
   - council-privacy notifies regulator engagement lead.
   - Re-issue any export bundle that was already delivered via `runbooks/regulator-export-reissue.md`.
2. If the original pack was returned to tenant via `evidence_query` in the last 90 days:
   - tenancy DPA-bound notification per tenant's preference.
3. Postmortem within 5 business days.

## Halt conditions

- Source signal signature verification fails → escalate to Sev-1; treat as suspected tamper.
- Postgres role grant drift detected (writer has UPDATE) → halt; engage cloud-secrets.
- audit-chain bridge unavailable → escalate to `runbooks/audit-chain-backlog.md`; do not retry until substrate healthy.
- 2-person rule cannot be honoured (insufficient approvers available) → defer rebuild; surface in `governance` µservice for chair escalation.

## Verification (post-rebuild)

- Original pack carries `supersedes_pending=false` + `superseded_at=<ts>` + `superseded_by_pack_ref=<new_pack_id>`.
- New pack visible in `evidence_query` with `supersedes_pack_ref=<original>`.
- audit-chain `pack.superseded.v1` event visible in audit-chain query.
- Postmortem published in `evidence/incidents/foundry-evidence/<inc-id>/`.

## References

- `microservices/intelligence/policy/evidence-pack-integrity.md` EPI-02 + EPI-11.
- `microservices/intelligence/failure-modes.md` FM-01 + FM-06.
- `microservices/audit-chain/policy/seal-integrity.md` SI-07 (no historical-period writes).
- ADR-0024 (eval-evidence integration).
