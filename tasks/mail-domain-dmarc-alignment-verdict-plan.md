# Plan: mail-domain-dmarc-alignment-verdict

**Vertical:** mail  
**Crate:** oya-mail-domain  
**Branch:** feat/task-mail-domain-dmarc-alignment-verdict-2026-05-28  
**Base:** origin/dev  

---

## Objective

Extend `DmarcVerdict` in `crates/oya-mail-domain/src/governance.rs` to compute DMARC actions from *identifier-aligned* pass results (RFC 7489 §3) rather than raw SPF/DKIM pass booleans, and to surface `report_only` accounting for `p=none` monitoring tenants.

---

## Subtasks

### ST1 — Alignment-aware verdict path

**What:** Introduce `spf_aligned`/`dkim_aligned` inputs. Compute `DmarcAction` from ALIGNED pass rather than raw `spf || dkim`. Keep existing `DmarcVerdict::new` signature backward-compatible by treating raw pass as aligned.

**How:**
1. Add `DmarcVerdict::new_aligned(domain_ref, spf_aligned, dkim_aligned, policy, evidence_ref)` constructor that derives action from `(spf_aligned || dkim_aligned, policy)`.
2. Rewrite `DmarcVerdict::new` as a thin wrapper that calls `new_aligned` with the same booleans for both raw and aligned positions (backward-compat: raw pass = aligned pass for legacy callers).
3. Keep `report_only: Classified<bool>` defaulting to `false` in `DmarcVerdict::new` / `new_aligned` for Reject/Quarantine paths (wired up fully in ST2).

**Acceptance:**
- `cargo check -p oya-mail-domain --all-targets` clean.
- New test: `spf=true, dkim=true, spf_aligned=false, dkim_aligned=false, policy=Reject` → `DmarcAction::Reject`.
- Existing `dmarc_fail_quarantine_and_logged` test passes unchanged.

---

### ST2 — Report-only accounting for p=none

**What:** Surface `report_only: Classified<bool>` on `DmarcVerdict`. A `p=none` message with no aligned pass produces `DmarcAction::Accept` but `report_only=true`. An aligned pass always yields `report_only=false`.

**How:**
1. Add `report_only: Classified<bool>` field to `DmarcVerdict` struct.
2. Compute `report_only` inside `new_aligned`: `true` iff `policy == DmarcPolicy::None && !(spf_aligned || dkim_aligned)`.
3. `DmarcVerdict::new` wrapper propagates `report_only` from `new_aligned`.
4. `evidence_ref` retains `DataClass::Audit` classification (no change).

**Acceptance:**
- `cargo nextest run -p oya-mail-domain` passes.
- New test: non-aligned + `p=none` → `DmarcAction::Accept` with `report_only=true`.
- New test: aligned pass (any policy) → `report_only=false`.
- `evidence_ref.data_class == DataClass::Audit` (existing assertion pattern).

---

## Acceptance Summary

| Check | Command |
|-------|---------|
| Compile clean | `cargo check -p oya-mail-domain --all-targets` |
| All tests pass | `cargo nextest run -p oya-mail-domain` |
| No root Cargo.toml edit | `git diff HEAD -- Cargo.toml` empty |
| No new crate | workspace member count unchanged |

---

## Constraints

- Operate ONLY in `crates/oya-mail-domain/src/governance.rs` (and this plan + spec doc).
- No new dependencies; crate depends only on `data-boundary-kernel`.
- No root `Cargo.toml` edit.
- Match existing patterns: `int()` helper, `ne()` guard, `Classified::new(v, DataClass::Audit)` for `evidence_ref`.
