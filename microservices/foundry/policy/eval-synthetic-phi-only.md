---
doc_class: Policy
title: Synthetic-PHI-Only Policy (foundry-eval eval-sets)
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-foundry
deciders: council-privacy, ops-security, axis-foundry
related_adrs: [ADR-0024, ADR-0131]
related_artifacts:
  - microservices/foundry/threat-model.md
  - microservices/foundry/dpia.md
  - microservices/foundry/policy/tenant-scope.cedar
review_cadence: quarterly + on every new pack-us-healthcare capability
doc_status: published
---

# Synthetic-PHI-Only Policy (foundry-eval eval-sets)

## Purpose

Live Protected Health Information (PHI) under HIPAA never enters foundry-eval eval-set authoring. Eval-set baseline inputs and case prompts use synthetic-PHI fixtures only. This policy is BLOCKER on all `microservices/foundry/eval-sets/**` content under the `oya-check-synthetic-phi-only` LEAN lane.

## Scope

Applies to:
- Every eval-set manifest at `eval-sets/<capability>/v<n>.evalset.yaml`.
- Every baseline-output object referenced from an eval-set.
- Every eval-case input prompt.
- Pack-us-healthcare capabilities specifically.

Does NOT apply to:
- Replay traces (which may contain unredacted PHI from source µservices; handled separately via OTel redactor + per-subject DEK envelope).
- Production foundry-runtime invocations (handled by foundry-runtime's sandbox + data-class boundary).

## Policy

### P-1: Live-PHI excluded by construction

Every eval-case input must carry `data_class` annotation. Cases tagged `PHI` must additionally carry `phi_origin: synthetic` with a synthetic-fixture provenance reference. Cases tagged `PHI` with `phi_origin != synthetic` are refused at registry-load time.

### P-2: Synthetic-fixture provenance

Synthetic-PHI fixtures are sourced from approved generators:
- **HHS de-identification expert-determination dataset** (when HHS published; reference recorded).
- **MIMIC-IV-Demo** (synthetic-only subset; license recorded).
- **Synthea** (open-source synthetic patient generator; commit SHA recorded).
- **Internal oyatie synthetic generator** (per `tools/synthetic-phi-generator/`; output reviewed by council-privacy).

Each fixture file carries a header block:

```yaml
fixture:
  name: <fixture-name>
  generator: <generator-id>
  generated_at: <unix-ts>
  generator_version: <semver>
  hhs_expert_determination_recorded_at: <unix-ts>   # for HHS-eligible fixtures
  council_privacy_review_recorded_at: <unix-ts>     # for internal-generator
  provenance_doi_or_commit: <doi-or-sha>
```

### P-3: LEAN-check enforcement

`oya-check-synthetic-phi-only` LEAN lane validates:
1. Every eval-case with `data_class=PHI` carries `phi_origin: synthetic`.
2. Every referenced fixture file carries the provenance header (P-2).
3. No fixture exceeds 1y staleness without re-review.
4. Cross-fixture re-use limited to ≤ 10 cases per fixture (to prevent over-fit).

### P-4: HHS expert-determination path

For fixtures derived from real-PHI via HHS §164.514(b)(1) expert-determination de-identification:
- Expert credential + determination date recorded.
- Re-identification risk assessment ≤ 0.04 (per HHS guidance) recorded.
- Annual re-review required.

### P-5: Pack-us-healthcare capability gating

Pack-us-healthcare capabilities must:
1. Declare `requires_phi_evaluation: true` in capability manifest.
2. Reference at least one synthetic-PHI fixture in eval-set.
3. Pass adversarial cohort tests against PHI exfiltration patterns.
4. Be reviewed by council-privacy + HIPAA Compliance Officer before publish.

### P-6: Live-PHI exfiltration prevention

If a capability's published eval-set ever attempts to load a `PHI` case with `phi_origin != synthetic`:
- Registry refuses load + emits `EVT-FOUNDRY-EVAL-LIVE-PHI-ATTEMPT`.
- Sev-1 security incident triggered.
- council-privacy + ops-security paged immediately.

## Verification

- `oya-check-synthetic-phi-only` LEAN lane: exit 0 in every PR touching `eval-sets/**`.
- Quarterly council-privacy review of approved synthetic-PHI fixtures.
- Annual external auditor review (HIPAA-engagement scope).
- Pen-test: attempt to push a live-PHI eval-case; must be refused at three layers (PR review, LEAN check, registry-load).

## References

- ADR-0024 (eval-set authoring; adversarial cohort).
- HIPAA 45 CFR §164.502, §164.514, §164.316.
- HHS de-identification guidance (safe-harbor + expert-determination methods).
- threat-model.md (R-14 in dpia.md; T-A-01 contamination).
- dpia.md §"Decisions" decision 1.
