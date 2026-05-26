---
doc_class: Runbook
title: False-positive tenant-relief — FP budget exhaustion / LLM-judge budget exhaustion
microservice: foundry-guardrails
severity: "Sev-3 (tenant-bounded)"
status: Accepted
owner_team: axis-foundry-guardrails + gtm-customer-success
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-08, FM-11)
  - microservices/intelligence/policy/guardrail-enforcement.md (§"Per-Tenant FP Escalation Budget")
doc_status: published
---

# Runbook: False-positive tenant-relief

## Trigger

ONE of:

1. **FP budget exhausted** (FM-08): tenant marked > N blocks/month as FP; budget-exceeded.
2. **LLM-judge budget exhausted** (FM-11): tenant's LLM-judge invocations exceed hourly budget; ambiguous prompts fail-closed.
3. **Tenant operator complaint**: tenant reports excessive blocks via gtm support channel.

## Severity

Sev-3 (tenant-bounded). NOT Sev-2 unless multi-tenant pattern emerges (which would become Sev-2 FM-07).

## Pre-checks

1. Identify the tenant: `tenant_id_hashed`.
2. Pull tenant's recent FP escalations: `oya foundry-guardrails fp-list --tenant <id> --period 30d`.
3. Identify pattern: are the FPs concentrated on a single rule? Multiple rules? Random?
4. Quantify: total FPs vs budget; LLM-judge invocation rate vs budget.

## Steps — FP budget exhaustion (FM-08)

| Step | Action | Time |
|---|---|---|
| 1 | Open low-urgency support ticket `#inc-<id>` | ≤ 1h (business hours) |
| 2 | Review tenant's FP entries; identify offending rule(s) | ≤ 4h |
| 3 | If concentrated on one rule: trigger rule re-review via rule-author dashboard | ≤ 24h |
| 4 | If random / many rules: escalate to gtm-customer-success for tenant onboarding-pattern review (tenant may be exposing edge-case prompts) | days |
| 5 | Per-tenant temporary budget grant (≤ 2× normal budget) for current month if blocking tenant operations: `oya foundry-guardrails fp-budget-extend --tenant <id> --amount <N> --until <date> --reason <rfc>` | ≤ 30 min |
| 6 | Rule-author iterates on shadow→enforce per IP-014 | days |
| 7 | Tenant communications via gtm | parallel |

## Steps — LLM-judge budget exhaustion (FM-11)

| Step | Action | Time |
|---|---|---|
| 1 | Open support ticket | ≤ 1h |
| 2 | Review tenant's recent invocation patterns; identify if prompts are unusually ambiguous OR if attacker pattern (rapid budget consumption) | ≤ 2h |
| 3 | If attacker pattern: engage ops-security; pause LLM-judge for tenant; review for jailbreak escalation | ≤ 30 min |
| 4 | If legitimate: per-tenant budget extension via foundry-providers (foundry-guardrails sibling) | ≤ 30 min |
| 5 | Rule-author tunes ensemble disagreement threshold to reduce LLM-judge invocation rate | days |

## Steps — concerted FP pattern (transition to FM-07)

If 2+ tenants exhausting FP budget on same rule_id within rolling 7d, transition to Sev-2 FM-07 surge handling per `policy-rule-rollback.md`.

## Verification

- Tenant's budget honoured; granted extensions logged.
- Rule-author dashboard reflects.
- Audit-chain seal records budget grant / pause action.
- Tenant comms acknowledgement.

## Post-incident updates

- Postmortem if rule-author iteration revealed gap.
- If recurring on same tenant: gtm review of tenant's use-case; may need DPA-recorded entitlement for legitimate edge-cases.

## References

- `microservices/intelligence/failure-modes.md` FM-08 + FM-11.
- `microservices/intelligence/policy/guardrail-enforcement.md`.
- `microservices/intelligence/incident-response.md` §"FP escalation budget".
