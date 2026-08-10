---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-008
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0297
companion_docs:
  - flags/runbooks/killswitch-engaged.md
  - flags/runbooks/flag-mutation-cascade.md
  - microservices/feature-flags/incident-response.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Runbook: Accessibility Flag Violation

## A. Trigger conditions

- A feature flag controlling an accessible UI component is causing WCAG 2.2 AA regression.
- An experiment flag is routing assistive-tech users (screen-reader, keyboard-only, switch-access) to a variant that fails accessibility requirements.
- A flag is hiding accessible alternative content (e.g., hiding audio CAPTCHA alternative when abuse-defence flag is active).
- A11y-CI lane (`axe-pa11y-runner`) fails on a feature controlled by a flag.
- User report: assistive-technology user cannot complete a critical workflow due to a flag variant.

## B. Pre-checks (≤3 minutes)

1. Identify the flag:
   ```bash
   oya flags list --tenant <tenant_id> --tag a11y
   # A11y-sensitive flags are tagged during creation
   ```
2. Check if this is an experiment flag routing assistive-tech users to inaccessible variant:
   ```bash
   oya experiments list --tenant <tenant_id> --state active --flag-key <flag_key>
   ```
3. Run axe/pa11y on the affected surface with the flag in both variants:
   ```bash
   # Requires iac/helm/axe-pa11y-runner
   oya a11y scan --url <affected_url> --flag-context '<flag_key>=on'
   oya a11y scan --url <affected_url> --flag-context '<flag_key>=off'
   ```
4. Check WCAG 2.2 AA violations in scan output (critical = Level A/AA; advisory = Level AAA).

## C. Procedure

### Step 1 — Immediate remediation: kill-switch or rollback (≤5 minutes)

If WCAG Level A or AA violation confirmed:
- If experiment: follow `runbooks/experiment-rollback.md` to roll back to control variant.
- If release flag: engage kill-switch per `runbooks/killswitch-engaged.md`.

```bash
# Kill-switch if critical a11y regression (roll back to accessible default)
oya flags kill-switch engage <flag_key> \
  --tenant <tenant_id> \
  --reason "a11y regression: WCAG 2.2 AA violation in on-variant" \
  --step-up-token $STEP_UP_TOKEN
```

### Step 2 — Identify affected users (≤15 minutes)

```bash
# Check if assistive-tech users are disproportionately affected
oya audit query --event-class FlagEvaluated \
  --flag-key <flag_key> \
  --tenant <tenant_id> \
  --filter "context.accessibility_profile != null"
```

If assistive-tech users were routed to the inaccessible variant: trigger adverse-action notification per WCAG accessibility policy + EU Accessibility Act.

### Step 3 — Fix and re-test (≤2 days)

1. Fix the UI component in the flag variant.
2. Re-run axe/pa11y on both variants.
3. Add a11y-specific targeting rule: assistive-tech users (`context.accessibility_profile != null`) MUST receive the accessible variant.
   ```bash
   oya flags update <flag_key> \
     --tenant <tenant_id> \
     --targeting-rules '[
       {"variant": "on", "cedar_predicate": "!(context.accessibility_profile != \"\")"},
       {"variant": "accessible-on", "cedar_predicate": "context.accessibility_profile != \"\""}
     ]'
   ```
4. Re-enable flag after both variants pass axe/pa11y.

## D. Verification

- `oya a11y scan` returns 0 Level A/AA violations on both variants.
- Assistive-tech users (`accessibility_profile != null`) routed to accessible variant.
- A11y-CI lane (`axe-pa11y-runner`) green.

## E. Rollback

- Kill-switch remains engaged until fix is verified.
- Disengagement: `oya flags kill-switch disengage <flag_key>` (step-up Class B).

## F. Post-incident

- Root cause: was a11y testing missing from the feature's CI? Add `axe-pa11y-runner` to the PR pipeline for all consumer µservices using this flag.
- Experiment design: future experiments on UI components MUST include a11y slice analysis (does variant break a11y for any user segment?).
- Per §3.2.3 UX floor: challenge (CAPTCHA, step-up) presented to assistive-tech users MUST have audio alternative + keyboard navigation. If abuse-defence triggered the a11y violation, review `policy/abuse-defence.cedar` UX floor compliance.

## G. References

- `docs/standards/a11y-canonical.md` — WCAG 2.2 AA requirements.
- `docs/standards/wcag-2-2-aa-checklist.md` — full checklist.
- `iac/helm/axe-pa11y-runner/` — CI a11y scanner.
- `runbooks/killswitch-engaged.md` — kill-switch procedure.
- ADR-0297 §UX-floor — abuse-defence UX-floor invariants including a11y.
