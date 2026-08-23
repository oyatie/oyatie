---
doc_class: PipelineSpec
shape: pipeline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Auto-derive `docs/GLOSSARY.md` from term-definition blocks in source files
  (`/// glossary: <term>` rustdoc, plus `<!-- glossary: <term> -->` in markdown).
  Retired terms enforced via the `governance-glossary` lane. No hand-edits
  permitted after generation; the canonical vocabulary lives at source, not in a
  hand-curated wordlist.
planned_enforcement_ref: governance-glossary
extends_crates:
  - governance-glossary-coverage-kernel
  - governance-glossary-vocabulary-kernel
companion_docs:
  - INDEX.md
  - ../../docs/GLOSSARY.md
doc_status: published
---

# Pipeline: GLOSSARY.md auto-derivation

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

Glossaries that are hand-maintained drift the moment the next feature lands. This pipeline inverts the convention: term definitions live next to the construct that defines them, are extracted from source on every build, and `docs/GLOSSARY.md` is a generated view. The extant `governance-glossary-coverage-kernel` validates coverage; this pipeline plus `governance-glossary-vocabulary-kernel` enforce the rendering and retirement.

## 2. Source-side annotation grammar

In Rust source (preferred when the term has a kernel struct):

```rust
/// glossary: TenantContext
/// definition: The cross-axis identity envelope binding a request to a tenant,
/// region, persona-tier, and capability-token. Authoritative under ADR-0002.
/// see-also: TenantId, PersonaTier, RegionCode
pub struct TenantContext { /* ... */ }
```

In Markdown source (for concepts without a kernel struct):

```markdown
<!-- glossary: AutonomyCeiling
definition: The upper bound on action breadth + sensitivity an agent run is
authorized to perform; enforced jointly by Cedar policy + agent-runtime gates.
see-also: PersonaTier, CapabilityToken
-->
```

## 3. Inputs

- All `///`/`<!-- -->` glossary blocks across `crates/**/src/**/*.rs` and `docs/**/*.md`.
- A retirement file `docs/glossary-retirements.md` listing terms removed from active vocabulary with `retired_at:` and `replacement:`.

## 4. Outputs

- `docs/GLOSSARY.md` — single-file rendered glossary alphabetical by term; each entry includes definition, see-also cross-links, source-of-definition link.
- `docs/machine-readable/glossary.json` — same data machine-readable.
- mdbook chapter `docs/site/src/reference/glossary.md` mirroring the markdown.

## 5. Trigger matrix

| Event | Action |
|---|---|
| Per-PR | Regenerate; PR fails if generated output differs from committed. |
| Nightly | Sweep for orphan term references (`docs/**/*.md` cites a term not in glossary). |
| On `glossary-retirements.md` edit | Validate retirement entry has `retired_at:` + `replacement:` + ADR citation. |

## 6. Validation gates (`governance-glossary`)

1. **Block well-formedness.** Every glossary block has `definition:` non-empty (BLOCKER).
2. **Term uniqueness.** No two source locations declare the same term with conflicting definitions (BLOCKER; resolved by ADR or by consolidating into one source).
3. **Retired-term enforcement.** A term listed in `glossary-retirements.md` referenced anywhere in `docs/**/*.md` after `retired_at` → HIGH; CI suggests the `replacement:` term.
4. **see-also cycle/missing.** Every `see-also:` target resolves to another glossary term (HIGH on missing).
5. **Coverage floor.** Per the extant `governance-glossary-coverage-kernel`, terms cited in Tier-1 docs MUST resolve in glossary (BLOCKER).
6. **Generated drift.** Committed `docs/GLOSSARY.md` differs from regenerated artifact (BLOCKER).

## 7. Cross-axis vocabulary harmonization

`governance-glossary-vocabulary-kernel` enforces canonical naming across axes (e.g. "Tenant" not "Customer" in cloud axis; "Workspace" not "Org" in workspace axis). This pipeline's validation gate 2 (Term uniqueness) inherits the vocabulary rules.

## 8. Out-of-scope

- Localized glossaries (KR + EN + JP variants are separate; tracked under `governance-glossary-localization-kernel`, future).
- Term-of-art capitalization style (covered by `docs/standards/doc-style.md`).
- External-vendor terminology (vendor glossaries linked, not inlined).
