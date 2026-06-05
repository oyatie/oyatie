---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-002-presentation-slide-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + council-design-system
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
depends_on: []
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: presentation + slide BCs — kernel + domain

## Intent

Author the foundational `presentation` BC (deck container) + `slide` BC (individual slide) kernel + domain layers. Pure Rust; zero I/O at kernel; deterministic domain.

## ChangeSet boundary

10 crates:
- `oya-slides-presentation-{kernel,domain}`
- `oya-slides-slide-{kernel,domain}`
- Plus catalog records under `microservices/slides/catalog/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-slides-presentation-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `src/crates/oya-slides-presentation-domain/{Cargo.toml,src/lib.rs,src/lifecycle.rs,tests/lifecycle.rs}` | create |
| `src/crates/oya-slides-slide-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `src/crates/oya-slides-slide-domain/{Cargo.toml,src/lib.rs,src/ordering.rs,tests/ordering.rs}` | create |
| `microservices/slides/catalog/oya-slides-presentation-{kernel,domain}.yaml` | create |
| `microservices/slides/catalog/oya-slides-slide-{kernel,domain}.yaml` | create |

## Code Shape

`presentation-kernel/src/entities.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deck {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub deck_id: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub tenant_id: String,
    #[data_class(INTERNAL_ONLY)]
    pub pack: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub title: String,
    #[data_class(AUDIT)]
    pub owner_oidc_sub: String,
    #[data_class(AUDIT)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)]
    pub version_sha: String,
    pub parent_version_sha: Option<String>,
    pub theme_id: Option<String>,
    pub slide_count: u32,
    pub share_link_enabled: bool,
    pub contains_special_category: bool,
    pub broadcast_link_public: bool,
}

pub trait DeckRepository {
    fn create(&self, deck: &Deck) -> Result<(), DeckError>;
    fn get(&self, tenant_id: &str, deck_id: &str) -> Result<Deck, DeckError>;
    fn update(&self, deck: &Deck) -> Result<(), DeckError>;
    fn delete(&self, tenant_id: &str, deck_id: &str) -> Result<(), DeckError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-slides-presentation-kernel -p oya-slides-presentation-domain \
  -p oya-slides-slide-kernel -p oya-slides-slide-domain
cargo nextest run -p oya-slides-presentation-domain --test lifecycle
cargo nextest run -p oya-slides-slide-domain --test ordering
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice slides
```

## Test Plan

| Test | Verifies |
|---|---|
| `lifecycle::test_deck_create_then_update_emits_version_increment` | version_sha monotonic |
| `lifecycle::test_data_class_annotations_present` | every field annotated |
| `ordering::test_reorder_preserves_count` | reorder ops preserve slide count |

## Halt Conditions

- Data-class lane red — STOP.

## Next IP

IP-003.
