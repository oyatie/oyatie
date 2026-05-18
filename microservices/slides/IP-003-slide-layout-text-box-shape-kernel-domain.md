---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-003-slide-layout-text-box-shape-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
depends_on: [IP-002]
---

# IP-003: slide-layout + text-box + shape + table + equation BCs — kernel + domain

## Intent

Author the slide-structure BCs (layouts, text-box, shape, table, equation) kernel + domain. Pure Rust + deterministic domain.

## ChangeSet boundary

10 crates (kernel + domain pair per BC):
- `oya-slides-slide-layout-{kernel,domain}`
- `oya-slides-text-box-{kernel,domain}`
- `oya-slides-shape-{kernel,domain}`
- `oya-slides-table-{kernel,domain}`
- `oya-slides-equation-{kernel,domain}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-slides-slide-layout-{kernel,domain}/...` | create |
| `src/crates/oya-slides-text-box-{kernel,domain}/...` | create |
| `src/crates/oya-slides-shape-{kernel,domain}/...` | create |
| `src/crates/oya-slides-table-{kernel,domain}/...` | create |
| `src/crates/oya-slides-equation-{kernel,domain}/...` | create |
| catalog entries per crate | create |

## Code Shape

`text-box-kernel/src/entities.rs`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextBox {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub text_box_id: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub slide_id: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub runs: Vec<RichRun>,
    pub bbox: BoundingBox,
    pub animations: Vec<TextAnimation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: String,
    pub font_family: String,
    pub size_pt: f32,
}
```

`equation-kernel/src/entities.rs`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Equation {
    pub equation_id: String,
    pub slide_id: String,
    pub source_format: EquationSourceFormat,  // KaTeX | MathJax-TeX | MathML
    pub source: String,
    pub rendered_svg: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EquationSourceFormat {
    Katex,
    MathJaxTex,
    MathML,
}
```

## Acceptance Gates

```bash
cargo check -p oya-slides-{slide-layout,text-box,shape,table,equation}-{kernel,domain}
cargo nextest run -p oya-slides-text-box-domain --test rich_text_serialization
cargo nextest run -p oya-slides-table-domain --test cell_merge
cargo nextest run -p oya-slides-equation-domain --test katex_parse
oya gate validate layer-correctness --microservice slides
```

## Test Plan

| Test | Verifies |
|---|---|
| Rich-text serialization round-trip | text-box content preserves formatting |
| Cell-merge | table cells merge/split correctly |
| KaTeX parse | equation source-format detection |
| Shape path serialization | freeform path preserves precision |

## Next IP

IP-004.
