---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: animation-engine-rollback
status: Accepted
severity: Sev-2 (Sev-1 if reduced-motion fallback skipped)
date: 2026-05-17
owner_team: axis-workspace + council-design-system + ops-accessibility
related_artifacts:
  - microservices/slides/decisions/ADR-SLIDES-0004-animation-engine-and-reduced-motion.md
  - microservices/slides/failure-modes.md FM-20
  - microservices/slides/slos/present-mode-transition-latency.openslo.yaml
doc_status: published
---

# Runbook — Animation engine rollback

## When to use

- Present-mode transition frame budget exceeded (p95 > 50ms, target violation).
- Animation BC release introduces visible regression.
- `oya-governance-reduced-motion-fallback-mandatory` lane red.
- Tenant reports animation jank or accessibility-mode failure.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Frame budget violation cluster-wide | Animation engine regression after deploy | step 1 |
| Per-tenant jank | Tenant deck with excessive animations | step 2 |
| Reduced-motion fallback not honored | Animation-BC bug; AC-17 invariant violated | step 3 |
| Per-pack accessibility-default override skipped | Pack overlay miss | step 4 |

## Step 1 — Engine regression

```bash
# Identify recent slides release
helm history slides -n workflow-studio | head -20

# Quick rollback
helm rollback slides <known_good_revision> -n workflow-studio

# Verify SLO recovery
sleep 60
oya vcs --service slides --action slo-status --slo present-mode-transition-latency
```

If rollback restores SLO: open issue with reproducer; freeze new animation-engine releases until root cause.

## Step 2 — Per-tenant jank

```bash
TENANT_ID=<tenant_id>
DECK_ID=<deck_id>

# Inspect deck animation profile
oya vcs --service slides --action describe-deck-animation-profile --deck-id $DECK_ID

# If excessive: recommend simplification + offer accessibility-mode banner
oya vcs --service slides --action recommend-simplify --deck-id $DECK_ID --reason "frame-budget-exceeded"
```

## Step 3 — Reduced-motion fallback failure (Sev-1 if SC 2.3.3 violated)

Per ADR-SLIDES-0004 + WCAG 2.2 SC 2.3.3.

```bash
# Verify lane status
oya gate validate reduced-motion-fallback-mandatory --microservice slides

# If lane red: regression in animations BC
# Roll back + freeze + alarm
helm rollback slides <known_good_revision> -n workflow-studio
oya vcs --service slides --action freeze-capability --capability animations-bc-release
```

Open Sev-1 accessibility incident; legal + DPO notification (WCAG conformance materially impacted).

## Step 4 — Pack overlay miss

```bash
PACK=<pack>

# Verify pack overlay sets reduced_motion_default + color_blind_safe
oya vcs --service slides --action describe-pack-overlay --pack $PACK | jq '.accessibility'

# If overlay drift: re-apply pack overlay
kubectl apply -k iac/kustomize/overlays/pack-$PACK
```

## Re-enable

```bash
# Health verify
oya vcs --service slides --action animation-health
oya gate validate reduced-motion-fallback-mandatory --microservice slides

# Lift any banner
oya vcs --service slides --action announce-animations-restored
```

## Verification

- Present-mode transition p95 ≤ 50ms.
- Reduced-motion fallback test green.
- Audit-chain seal of rollback emitted.

## References

- ADR-SLIDES-0004 (animation engine + reduced-motion).
- WCAG 2.2 SC 2.3.3.
- failure-modes.md FM-20.
