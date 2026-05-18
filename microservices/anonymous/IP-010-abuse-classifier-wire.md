---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-010-abuse-classifier-wire
status: pending
execution_unit: ChangeSet
owner: axis-anonymous + axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-test, oya-governance-ai-assessed-label-present]
---

# IP-010: Abuse-classifier wire (foundry-runtime T1 adapter)

## Intent

Wire content-moderation BC to foundry-runtime classifier per ADR-ANON-0005. Every classifier verdict carries `ai_assessed_label` (EU AI Act Art. 50) + appeal_link (EU DSA Art. 14) + statement_of_reasons (EU DSA Art. 17).

## ChangeSet

- content-moderation kernel + domain + usecase + adapter-foundry-runtime + rest + worker + sdk
- T1-assist + T2-auto capability tier wiring
- NCMEC CyberTipline trigger for CSAM-suspect verdicts (PRD FR-27)

## Acceptance

- Classifier verdict roundtrip < 200ms p95
- ai_assessed_label presence lint passes (LEAN lane)
- Appeal-flow test passes
- NCMEC reporting test passes
- Classifier rollback runbook tested
