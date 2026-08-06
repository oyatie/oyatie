---
id: ADR-0097
title: "Rename oya-intelligence-account-adapter-{claude-code,codex-cli,gemini-cli} → oya-foundry-{claude,codex,gemini}-account-adapter"
status: Superseded
doc_status: published
owner: council-architecture
date: 2026-05-15
owner_phase: M02-P06
deciders:
  - Architect (v4 BNF consensus)
  - Critic (v6 PRE-1..PRE-6 mandate sweep)
supersedes: []
superseded_by: []
related:
  - ADR-0056  # 12-layer enum + BNF v4.1 (layer token MUST be last)
  - ADR-0096  # supervisor language: Rust (cites these adapter crates)
supersession_note: "3-crate cosmetic foundry rename; subsumed by the foundry sweep. Archived per D-DISPOSITIONS-RATIFIED: ARCHIVE-5."
---

# ADR-0097: Rename `oya-intelligence-account-adapter-*` — Layer Token Must Be Last

## Status

Accepted (v6 pre-scaffold amendment; source: F-SETTINGS-ADAPTER-RENAME-BNF-CONFORMANT-1, CONV-8, `M2` + `F1`).

## Context

M02-P06 Wave 3 introduces three CLI-driver crates, one per AI provider account:

| Current name (non-conformant) | v4 BNF parse result |
|---|---|
| `oya-intelligence-account-adapter-claude-code` | layer = `code`? — NOT a recognised layer value |
| `oya-intelligence-account-adapter-codex-cli` | layer = `cli` BUT `adapter-codex` is not a BC token; `adapter` is mid-name |
| `oya-intelligence-account-adapter-gemini-cli` | same problem |

ADR-0056 BNF v4.1 parser rule:

> LAST token MUST be a layer value (one of 12 canonical).

In all three names `adapter` is not the last token; `code`, `cli` (incorrectly positioned), or another suffix follows it. This violates the closed 12-value layer enum rule.

This ADR records the rename decision. **The actual crate rename is a prerequisite task separate from M02-P06 implementation** and executes before any Wave 3 grit units open against these crates.

## Decision

Rename the three provider account-adapter crates so that `adapter` is the final segment:

| Before | After |
|---|---|
| `oya-intelligence-account-adapter-claude-code` | `oya-intelligence-claude-account-adapter` |
| `oya-intelligence-account-adapter-codex-cli` | `oya-intelligence-codex-account-adapter` |
| `oya-intelligence-account-adapter-gemini-cli` | `oya-intelligence-gemini-account-adapter` |

**BNF v4.1 parse of the new names:**

```
oya-intelligence-claude-account-adapter
    microservice = foundry
    bc-tokens    = claude-account
    layer        = adapter   ✓ (last token, recognised layer value)

oya-intelligence-codex-account-adapter
    microservice = foundry
    bc-tokens    = codex-account
    layer        = adapter   ✓

oya-intelligence-gemini-account-adapter
    microservice = foundry
    bc-tokens    = gemini-account
    layer        = adapter   ✓
```

## Decision Drivers

1. **ADR-0056 §Parser rule** — last segment must be one of the 12 closed layer values. Current names violate this by placing provider-qualifier tokens (`claude-code`, `codex-cli`, `gemini-cli`) after `adapter`.

2. **Sibling precedent** — `oya-intelligence-jsonl-supervisor-adapter` follows the pattern correctly:
   `microservice=foundry`, `bc-tokens=jsonl-supervisor`, `layer=adapter`. Provider account
   adapters must follow the same shape: `microservice=foundry`, `bc-tokens=<provider>-account`,
   `layer=adapter`.

3. **oya-check-architecture LEAN-A1 enforcement** — The `lib-name-parity` and `layer-correctness`
   CI lanes derive the expected layer from the last crate-name segment. Non-conformant names
   cause false CI failures or silently bypass the layer-correctness check.

4. **v6 amendment PRE-1 analogy** — PRE-1 renames `oya-intelligence-settings-template-adapter-fs` →
   `oya-intelligence-settings-template-adapter` for the identical reason (layer token must be last).
   Account-adapter crates have the same structural defect.

## Alternatives Considered

### Alt A — Keep existing names, add BNF exemption

**Pros:** Zero rename cost.

**Cons:** BNF exemption requires a new ADR to extend the 12-value layer enum or carve out a special
rule for provider-qualified adapters. Creates precedent for name drift. `oya-check-architecture`
either breaks or must be patched to special-case these names.

**Verdict: REJECTED** — BNF extension cost > rename cost; drift precedent unacceptable.

### Alt B — Drop provider qualifier entirely (`oya-intelligence-account-adapter`)

**Shape:** Single crate routing across all three providers.

**Pros:** Simpler surface.

**Cons:** Three distinct `SessionDriver` implementations with different retry, stop-hook, and
idempotency semantics (v4 §A.3, Wave 3a-3c matrix) cannot safely live in one crate without
feature-flag complexity. The Wave 3 split was chosen precisely to isolate per-provider divergence.

**Verdict: REJECTED** — Per-provider isolation is a design constraint, not an implementation
convenience. The split must be preserved.

### Alt C (chosen) — Rename with `<provider>-account` as BC tokens

BC tokens `claude-account`, `codex-account`, `gemini-account` identify the provider-qualified
account sub-domain within the foundry µservice. This is idiomatic BNF v4.1 BC usage (multiple
concepts at the same layer get distinct BC tokens). Layer = `adapter` is last. Sibling precedent
confirmed.

## Consequences

### Positive
- All three crate names pass `oya-check-architecture --layer-correctness` and
  `--lib-name-parity` without special-casing.
- `SessionDriver` trait implementations remain in separate crates (per-provider isolation
  preserved).
- Wave 3 grit units can open with conformant crate names on day-1.

### Negative / Trade-offs
- `Cargo.toml` path and `[package] name` updates required across workspace + any existing
  references in `docs/`, `registry/`, `templates/`.
- `lib.rs` `#![crate_name]` attribute and `use` imports must be updated to snake_case equivalents:
  `oya_foundry_claude_account_adapter`, `oya_foundry_codex_account_adapter`,
  `oya_foundry_gemini_account_adapter`.
- The rename itself must precede Wave 3a grit claim (`grit claim --agent worker-3a …`) or
  Wave 3a will open against a non-existent path.

### Not in scope
- The rename execution is a separate prerequisite task tracked independently. This ADR records
  the decision only.
- No `Cargo.lock` churn beyond path/name fields (no dependency version changes).

## Follow-ups

1. **Prerequisite task** — Execute the three-crate rename before Wave 3a grit unit opens.
   Update `Cargo.toml`, `[lib] name`, all `docs/` + `registry/` + `templates/` references.
2. **oya-check-architecture** — Verify LEAN-A1 passes on all three renamed crates after rename.
3. **ADR-0096 §Alternatives** — Update prose references from old to new names in a companion PR.

## References

- ADR-0056 §Parser rule — 12-layer enum, BNF v4.1
- ADR-0056 §Examples — `oya-intelligence-jsonl-supervisor-adapter` sibling precedent
- v6 amendments PRE-1 — identical fix applied to `oya-intelligence-settings-template-adapter-fs`
- `feedback_naming_justification.md` — every new name must carry one-line BNF + layer conformance justification
