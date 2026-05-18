---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-008-themes-templates-master-slide-editor
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + council-design-system
acceptance_lanes: [cargo-check, cargo-nextest, signature-verify, layer-correctness]
depends_on: [IP-003]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: themes + templates + master-slide-editor + slide-sorter + layout-engine

## Intent

Author the design-system BCs: themes (signed gallery + custom), templates (signed gallery + tenant custom), master-slide-editor, slide-sorter, layout-engine (auto-align + auto-distribute + smart-arrange).

## ChangeSet boundary

~24 crates spanning kernel + domain + usecase + api + adapter + adapter-s3 + adapter-leptos-wasm + sdk across 5 BCs.

## Concrete File Targets

`src/crates/oya-slides-themes-...`, `oya-slides-templates-...`, `oya-slides-master-slide-editor-...`, `oya-slides-slide-sorter-...`, `oya-slides-layout-engine-...`

## Code Shape

`themes-domain/src/signature.rs`:

```rust
pub fn verify_theme_signature(theme: &Theme, pubkey: &Ed25519PublicKey) -> Result<(), SignatureError> {
    let signed_bytes = theme.canonical_bytes();
    ed25519_dalek::verify(pubkey, &signed_bytes, &theme.signature)?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-themes-domain --test signature_verify
cargo nextest run -p oya-slides-templates-domain --test gallery_list
cargo nextest run -p oya-slides-layout-engine-domain --test auto_align
cargo nextest run -p oya-slides-master-slide-editor-domain --test layout_cascade
oya gate validate signature-verify --microservice slides
```

## Test Plan

| Test | Verifies |
|---|---|
| Theme signature verify | Ed25519 verification + revocation list check |
| Tampered theme refused | tampered bytes rejected |
| Layout-engine auto-align | element alignment math correct |
| Master slide cascade | edits to master cascade to derived slides |

## Next IP

IP-009.
