---
doc_class: Runbook
title: Jailbreak escalation — Sev-1 (always) post-mortem chain
microservice: foundry-guardrails
severity: "Sev-1 (always for confirmed jailbreak success)"
status: Accepted
owner_team: axis-foundry-guardrails + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-06)
  - microservices/intelligence/incident-response.md (§"Jailbreak-Success Specific Protocol")
  - microservices/intelligence/threat-model.md (T-T-04, T-T-05, T-I-04)
doc_status: published
---

# Runbook: Jailbreak escalation

## Trigger

ONE of:

1. **Automated detection**: post-output validator catches unsafe content that the pre-invocation classifier passed (post-hoc detection of false-negative).
2. **Tenant report**: tenant operator marks decision as `false_negative_severe` via FP escalation API.
3. **Red-team finding**: internal red-team or external pen-test identifies a passing-prompt that produces unsafe output.
4. **External report**: security researcher / bug bounty / responsible disclosure.

## Severity

**Sev-1 always**. The safety floor failed; aggregate trust posture demands maximum response regardless of single-tenant scope.

## Pre-checks

1. Confirm the jailbreak: re-run the offending prompt against current ensemble; verify reproducibility.
2. Capture invariants: prompt hash; provider output hash; affected tenant_id_hashed; affected capability_id; classifier model versions; ensemble verdict at original invocation time; cedar bundle SHA.
3. Determine blast-radius: how many invocations from this tenant used the same prompt pattern? Query foundry-evidence.

## Steps

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-<id>`; engage axis-foundry-guardrails IC + ops-security IC jointly | ≤ 5 min |
| 2 | Auto-allocate incident_id; auto-generate post-mortem template at `evidence/postmortems/<year>/<incident-id>.md` | ≤ 2 min |
| 3 | Freeze the offending capability for the affected tenant via Cedar emergency policy: `oya foundry-guardrails freeze --tenant <id> --capability <id> --reason <rfc>` | ≤ 5 min |
| 4 | If pattern suggests widespread risk (e.g., ensemble caught NONE of a class of obfuscation), freeze the capability cluster-wide; emits Sev-1 + per-tenant audit-chain seal | ≤ 5 min |
| 5 | Pin the failing prompt to red-team fixture catalogue (`tests/jailbreak/baseline_fixtures.rs`); will run on every classifier rollout | ≤ 1h |
| 6 | Engage PrivacyLead (council-privacy chair) to determine data-subject impact: was unsafe content delivered? Was PII / PHI exposed? | ≤ 30 min |
| 7 | If data-subject impact confirmed: begin per-pack regulatory notification chain (per `incident-response.md` §"Regulatory Notifications") | per per-pack timeline (GDPR 72h, KR PIPA 72h, HIPAA 60d ≥ 500, EU AI Act 15d serious) |
| 8 | Classifier retraining: data team retrains affected model on new fixture + adjacent perturbations; shadow→enforce per IP-014 | days-to-weeks |
| 9 | Annual red-team cadence updated to test the new pattern class explicitly | next quarter |
| 10 | Post-mortem published within 5 business days; action items tracked | ≤ 5 BD |

## Rollback (of the freeze — if mis-attribution)

If the "jailbreak" was actually a legitimate prompt (false alarm on the post-hoc detector):
1. Lift the freeze: `oya foundry-guardrails unfreeze --tenant <id> --capability <id> --reason <rfc>`.
2. Audit-chain emit `Sev1JailbreakRetraction`.
3. Treat as FM-07 (false-positive surge) for the post-hoc detector; retune.

## Verification

After completion:
- Affected capability un-frozen (or root-cause confirmed + retraining underway).
- Red-team fixture catalogue updated.
- Post-mortem published.
- Action items tracked.
- Tenant comms + regulatory notification (if applicable) completed within timeline.
- Classifier retraining shadow-deployed.
- `foundry_guardrails_sev1_jailbreak_total` decremented by manual incident-resolution mark.

## Post-incident updates

- Postmortem to `evidence/postmortems/<year>/<incident-id>.md`.
- Action items: typically include "why didn't the ensemble catch this?" + "what classifier capability gap is revealed?" + "what red-team fixture do we add?".
- Threat-model re-review trigger (per `threat-model.md` §"Re-review Triggers").
- DPIA re-review trigger.

## References

- `microservices/intelligence/failure-modes.md` FM-06.
- `microservices/intelligence/incident-response.md` §"Jailbreak-Success Specific Protocol".
- `microservices/intelligence/threat-model.md` T-T-04 + T-T-05 + T-I-04.
- `tests/jailbreak/baseline_fixtures.rs`.
- MITRE ATLAS — `atlas.mitre.org`.
- OWASP LLM Top 10 (2025) LLM01 Prompt Injection.
