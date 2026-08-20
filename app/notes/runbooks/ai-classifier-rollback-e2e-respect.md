---
doc_class: Runbook
title: AI classifier rollback while respecting E2E invariant
microservice: notes
severity: "Sev-1 (E2E invariant breach) / Sev-2 (eval regression)"
status: Accepted
owner_team: council-privacy + axis-notes + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/notes/decisions/ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md
  - microservices/notes/decisions/ADR-NOTES-0001-e2e-encryption-default-personal-tier.md
  - microservices/notes/policy/e2e-personal-tier-default.md
  - microservices/notes/policy/tenant-scope.cedar
  - microservices/notes/capabilities/T0-suggest.yaml
  - microservices/notes/capabilities/T1-assist.yaml
  - microservices/notes/capabilities/T2-auto.yaml
doc_status: published
---

# Runbook: AI classifier rollback while respecting E2E invariant

## When

Three triggers:

1. **Sev-1**: `oya_notes_ai_call_blocked_e2e_total > 0 over 5m` — a code path attempted to invoke AI on a Personal-tier E2E note. Per ADR-NOTES-0005 this is structurally impossible if invariants hold; > 0 is a confirmed invariant breach (or near-miss).
2. **Sev-2**: AI assist canary eval regresses past the threshold (per capability eval set).
3. **Sev-2**: Sustained per-tenant abuse spike — repeated invocations against quota.

## Severity Decision Tree

```
oya_notes_ai_call_blocked_e2e_total > 0?
  YES → Sev-1 (invariant breach)
  NO  → eval regression > 5 % from baseline?
    YES → Sev-2 (rollback to last-known-good)
    NO  → tenant abuse spike?
      YES → Sev-2 (rate-limit + tenant-admin notify)
      NO  → Sev-3 (model latency / cost only)
```

## Sev-1 — E2E Invariant Breach

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page council-privacy + ops-security + axis-notes oncall | observability | t+0 |
| 2 | Disable AI assist tenant-wide: `kubectl patch configmap notes-feature-flags --patch '{"data":{"ai_assist_enabled":"false"}}'` | oncall | t+5m |
| 3 | Freeze deploy: `oya vcs deploy freeze --microservice notes` | oncall | t+5m |
| 4 | Identify code path: was it actually a Personal note? Or was a Professional note misclassified? | oncall + axis-notes | t+30m |
| 5 | If misclassification (Professional misread as Personal due to bug) → no breach; fix classification; re-enable | axis-notes | t+1h |
| 6 | If actual Personal-tier AI call (invariant breach) → forensic capture of code path | ops-security | t+1h |
| 7 | Audit-chain query for any successful Personal-tier AI call (should be NONE; invariant should have blocked) | ops-security | t+2h |
| 8 | If any successful → GDPR Art. 34 notification to affected user within 72h | ops-legal | t+24h |
| 9 | Post-mortem within 5 business days; ADR-NOTES-0005 update if invariant model failed | council-privacy + axis-notes | t+5d |
| 10 | CI lane `oya-check-e2e-ai-refusal` regression test added | axis-notes | t+5d |

## Sev-2 — Eval Regression

| Step | Action | Owner |
|---|---|---|
| 1 | Capability eval set runs nightly; on threshold-cross emits alert | foundry-eval |
| 2 | axis-notes oncall acknowledges; identifies which capability (T0 / T1 / T2) regressed | oncall |
| 3 | Rollback to last-known-good model version via foundry-runtime canary system | axis-foundry-runtime |
| 4 | Re-run eval set on rolled-back model; verify ≥ baseline | foundry-eval |
| 5 | Tenant-admin notification (in-product banner) if user-visible behavior changes | gateway |
| 6 | Root-cause: model upgrade-side bug or prompt-template drift | axis-foundry-runtime + axis-notes |
| 7 | Post-mortem within 5 business days | axis-notes + axis-foundry-runtime |

## Sev-2 — Per-Tenant Abuse Spike

| Step | Action | Owner |
|---|---|---|
| 1 | Acknowledge alert; identify tenant | oncall |
| 2 | Rate-limit auto-engages (per `capabilities/T1-assist.yaml` cost-profile + monthly cap) | gateway |
| 3 | Tenant-admin notification (out-of-band email + in-product banner) | ops-finance |
| 4 | Investigate: legitimate spike (campaign) or abuse (compromised account) | ops-security |
| 5 | If abuse: suspend AI for that tenant pending investigation | ops-security |

## Capability Tier Rollback Procedure

### T1 capability rollback (summarize / tag-suggest / link-suggest)

```bash
oya foundry-runtime rollback \
  --capability cap:notes:t1-assist:v1 \
  --to-version <previous-version> \
  --reason "Sev-2 eval regression on summarize-faithfulness; rollback to last-known-good"
```

### T2 capability rollback (auto-organize)

T2 is disabled at minimum-shippable-tier. If enabled and regressed:

```bash
oya foundry-runtime disable \
  --capability cap:notes:t2-auto:v1 \
  --reason "Sev-2 regression"
```

## Verification After Rollback

- Confirm `oya_notes_ai_call_blocked_e2e_total` returns to 0.
- Confirm canary eval ≥ baseline on rolled-back version.
- Confirm tenant-visible behavior matches pre-regression UX.
- Audit-chain entry `AiCapabilityRolledBack{capability_id, from_version, to_version, reason, principal_ref}`.

## E2E Invariant Verification (After Every Rollback)

Run the regression set that exercises Personal-tier AI calls; each MUST be refused at:

1. Type-system level (compile-time test).
2. Cedar level (runtime test).
3. CI lane level (`oya-check-e2e-ai-refusal`).
4. Runtime metric level (`oya_notes_ai_call_blocked_e2e_total` increments).

## Failure Modes

| Failure | Recovery |
|---|---|
| foundry-runtime rollback fails | manual model-pointer flip via OpenBao; engage axis-foundry-runtime |
| Tenant-admin doesn't notice rate-limit notification | escalate via account-manager + status page |
| CI lane `oya-check-e2e-ai-refusal` itself broken | priority-fix; gate is BLOCKER |

## Metrics

- `oya_notes_ai_call_total{capability, model_version, result}`
- `oya_notes_ai_call_blocked_e2e_total` — expected = 0; alarm Sev-1 at > 0.
- `oya_notes_ai_eval_score{capability, version}` — baseline-relative.
- `oya_notes_ai_rate_limit_engaged_total{tenant_id}` — abuse proxy.

## Pack Overlays

| Pack | Notes |
|---|---|
| pack-eu | EU AI Act Art. 50 — rollback documented in evidence-topic |
| pack-us-healthcare | HIPAA §164.502(b) — AI on PHI requires re-attestation post-rollback |
| pack-kr | KR PIPA Art. 28 — rollback notified to tenant-PIPO |

## References

- ADR-NOTES-0005 (E2E AI refusal invariant).
- ADR-NOTES-0001 (E2E Personal-tier).
- `microservices/notes/policy/e2e-personal-tier-default.md`.
- `microservices/notes/capabilities/T0-suggest.yaml`.
- `microservices/notes/capabilities/T1-assist.yaml`.
- `microservices/notes/capabilities/T2-auto.yaml`.
- EU AI Act Art. 50.
- GDPR Art. 34.
- KR PIPA Art. 28.
