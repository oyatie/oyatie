---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-009-animations-transitions-reduced-motion
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + council-design-system + ops-accessibility
acceptance_lanes: [cargo-check, cargo-nextest, reduced-motion-fallback-mandatory, flashing-policy]
depends_on: [IP-003]
---

# IP-009: animations + transitions + reduced-motion fallback

## Intent

Author the animation engine + transition engine + reduced-motion 4-layer policy resolver per ADR-SLIDES-0004 + WCAG 2.2 SC 2.3.3 + 2.3.1.

## ChangeSet boundary

~12 crates across animations + transitions + accessibility BCs.

## Concrete File Targets

`src/crates/oya-slides-animations-...`, `oya-slides-transitions-...`, `oya-slides-accessibility-...`

## Code Shape

`accessibility-domain/src/reduced_motion.rs`:

```rust
pub struct ReducedMotionPolicy { ... }

impl ReducedMotionPolicy {
    pub fn resolve(
        &self,
        pack: &PackPolicy,
        ua_pref: UaPreference,
        deck_override: DeckOverride,
        audience_override: AudienceOverride,
    ) -> ResolvedMotionPolicy {
        // Layer 4 (most specific) wins
        if audience_override == AudienceOverride::ReducedOn {
            return ResolvedMotionPolicy::Reduced;
        }
        if deck_override == DeckOverride::ReducedOn {
            return ResolvedMotionPolicy::Reduced;
        }
        if ua_pref == UaPreference::Reduce {
            return ResolvedMotionPolicy::Reduced;
        }
        if pack.reduced_motion_default {
            return ResolvedMotionPolicy::Reduced;
        }
        ResolvedMotionPolicy::Full
    }
}
```

`animations-domain/src/flashing_policy.rs`:

```rust
pub fn validate_no_excessive_flashing(timeline: &AnimationTimeline) -> Result<(), FlashingPolicyError> {
    let luminance_peaks_per_sec = compute_peaks(timeline);
    if luminance_peaks_per_sec > 3 {
        return Err(FlashingPolicyError::ExcessiveFlashing);  // WCAG 2.3.1
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-accessibility-domain --test reduced_motion_resolve
cargo nextest run -p oya-slides-animations-domain --test flashing_policy
cargo nextest run -p oya-slides-transitions-domain --test transition_replacements
oya gate validate reduced-motion-fallback-mandatory --microservice slides
oya gate validate flashing-policy --microservice slides
```

## Halt Conditions

- Reduced-motion fallback test fails — STOP. AC-17 invariant.
- Flashing-policy test fails — STOP. WCAG 2.3.1.

## Next IP

IP-010.
