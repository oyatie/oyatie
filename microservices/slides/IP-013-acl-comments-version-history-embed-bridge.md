---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-013-acl-comments-version-history-embed-bridge
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, cedar-preview-required, per-slide-acl-no-deck-bypass, named-block-no-bypass]
depends_on: [IP-005]
---

# IP-013: acl + comments + version-history + embed-bridge

## Intent

Author the per-slide ACL (ADR-SLIDES-0007) + comments + version-history + embed-bridge (docs quotes + forms polls + drive assets) BCs.

## ChangeSet boundary

~25 crates across 4 BCs.

## Concrete File Targets

`src/crates/oya-slides-acl-...`, `oya-slides-comments-...`, `oya-slides-version-history-...`, `oya-slides-embed-bridge-...`

## Code Shape

`acl-domain/src/per_slide_refinement.rs`:

```rust
pub fn evaluate_per_slide(
    principal: &Principal,
    action: &Action,
    slide: &Slide,
    deck_acl: &DeckAcl,
) -> Decision {
    // Deck-level grant is necessary but not sufficient
    let deck_decision = evaluate_deck_level(principal, action, deck_acl);
    if deck_decision == Decision::Deny {
        return Decision::Deny;
    }
    // Per-slide deny overrides deck allow
    if slide.per_slide_acl.has_deny(principal) {
        return Decision::Deny;
    }
    // Named-block deny overrides slide allow
    for block in &slide.named_blocks {
        if block.has_deny(principal) {
            return Decision::Deny;
        }
    }
    deck_decision
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-acl-domain --test per_slide_deny_overrides_deck_allow
cargo nextest run -p oya-slides-acl-domain --test named_block_deny_overrides_slide_allow
cargo nextest run -p oya-slides-acl-domain --test cedar_preview
cargo nextest run -p oya-slides-version-history-domain --test restore_replays_cedar
cargo nextest run -p oya-slides-embed-bridge-domain --test docs_quote_bind
cargo nextest run -p oya-slides-embed-bridge-domain --test forms_poll_bind
oya gate validate cedar-preview-required --microservice slides
oya gate validate per-slide-acl-no-deck-bypass --microservice slides
oya gate validate named-block-no-bypass --microservice slides
```

## Halt Conditions

- Per-slide ACL bypass test fails — STOP. AC-08 + ADR-SLIDES-0007 invariant.
- Version-history restore Cedar replay fails — STOP.

## Next IP

IP-014.
