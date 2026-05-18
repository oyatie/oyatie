---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-006-termbase-and-glossary-stack
status: pending
execution_unit: ChangeSet
owner: axis-translate
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, tenant-isolation-rls]
---

# IP-006: Termbase + Glossary stack (`oya-translate-termbase-*`)

## Intent

Per-tenant termbase with TBX (ISO 30042) import/export. Per-tenant glossary constraints applied to MT output (FR-05, FR-22). Conflict-detection per FM-60.

## ChangeSet boundary

Crates: `oya-translate-termbase-{kernel, domain, usecase, api, adapter-postgres, rest, worker, sdk, app}`.

## Key Entities

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Term {
    pub id: String,
    pub tenant_id: String,
    pub project_id: Option<String>,
    pub concept_id: String,
    pub source_lang: LanguageTag,
    pub source_value: String,
    pub target_lang: LanguageTag,
    pub target_value: String,
    pub case_sensitive: bool,
    pub do_not_translate: bool,        // freeze through MT
    pub forbid_target: Option<String>, // explicit "do not use this target"
    pub provenance: TermProvenance,    // human-author | tbx-import | tenant-admin
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TermEnforcement {
    pub term_id: String,
    pub action: TermAction,            // EnforceTarget | DoNotTranslate | ForbidTarget
}
```

## TBX Import

Per ISO 30042:2019:
- Parse TBX 3.0 XML via `quick-xml` with DTD/XSL/entity resolution disabled (F-03 in threat-model).
- Schema validation against TBX core schema.
- Concept-id deduplication; merge per per-tenant-per-project conflict policy.
- 2-person review on import (per CI-INV-09 lineage).

## Enforcement Flow (in router-usecase IP-004 step 7)

1. Source segment scanned for source-values per glossary index.
2. For each match: build enforcement list with `(term_id, action, position)`.
3. Engine adapter receives glossary as side-channel context (per-vendor support):
   - Anthropic: system prompt addendum.
   - OpenAI: system prompt addendum.
   - Google Cloud Translation: `glossaryConfig` parameter (native).
   - DeepL: `glossary_id` parameter (native).
   - In-house (foundry-runtime): in-prompt + custom decoder constraint.
4. Post-MT: scan target for target-values; if `forbid_target` violated, re-prompt up to 2 attempts; if still violated, annotate result + emit warning.

## Test Plan

| Test | Verifies |
|---|---|
| `test_tbx_import_schema_valid` | TBX 3.0 conformance |
| `test_tbx_import_rejects_external_entity` | F-03 (XXE) prevented |
| `test_do_not_translate_freezes_source` | `EnforceTarget` |
| `test_forbid_target_triggers_reprompt` | enforcement post-MT |
| `test_glossary_concurrent_conflict_detected` | FM-60 surfaced |
| `test_dsr_erasure_propagates_to_termbase` | DSR cascade |
| `tests/integration/google_glossary_native_use.rs` | Google glossaryConfig honored |
| `tests/integration/deepl_glossary_native_use.rs` | DeepL glossary_id honored |

## Halt Conditions

- TBX import accepts external XML entity.
- Glossary conflict silently overridden.
- Cross-tenant termbase visibility.

## Next IP

[`IP-007-quality-estimation-stack.md`](IP-007-quality-estimation-stack.md)
