# Review PR #670 — Round 3 (fresh-context reviewer of record)

PR: jason931225/oyatie #670 · branch `agent/g011-test-wiring-gen` · head `a83c7b7bbf2ccfd79f0e3f5abf508c7a8ad926d3` · base `dev` (`16f2e3b54`)
Rubric: `.omc/ultragoal/RUBRIC-torvalds-review.md` (Torvalds lens §1–5 + hyperscaler §6 + owned-architecture §7)
History: r1 BLOCK ×3 · r2 BLOCK ×1 (10 new total-accounting/unjustified keys)

## VERDICT: **APPROVE**

The r2 blocker (10 new `cloud-ci-total-accounting/unjustified` baseline keys laundered into the frozen face) is fully resolved. The both-ways baseline key-set diff is now **0 added / 29 removed** — net debt reduction, no laundering. The two authority-surface edits (root-hub-pointers.json, ADR-0540) are **legitimate satisfaction of the accounting contract via the producer's documented mechanisms**, not reachability/justification gaming. All five gate/tool tests pass; settle protocol, signatures, and append-only audit chain hold; all r1/r2 code findings remain fixed.

---

## Findings

No blocking findings. Items below are informational/LOW.

1. **`tools/oya-buck-test-wiring-app/src/main.rs:1464` — LOW, confidence high (carried, non-blocking).** `resolve_owners` ignores `repo_root` and parses only OWNERS *existence*, not OWNERS *content* (`let _ = repo_root; // ... A-STRUCT follow-on`). So the OWNERS team name (`cloud-ci-platform`) is never validated against an owner registry — any single-line file satisfies the `unowned` advisory. This is pre-existing producer behavior identical to the two OWNERS files already in `dev`; it is not introduced or worsened by this PR. `unowned` is `advisory-until-infra` (non-blocking) so it cannot gate-launder. Noted as residual design debt, not a blocker for #670.

2. **`tools/oya-buck-test-wiring-app/src/main.rs:1582` — LOW, confidence high (carried).** Justification is a substring/token match of a tracked path inside an ADR body. An ADR can therefore "justify" any file simply by naming its path. Here ADR-0540 genuinely governs the generator (it IS the target-parity gate ADR and this IS its local-bridge generator), so the justification is semantically real — but the mechanism is text-trust, the same heuristic flagged in r1/r2 residual risk. Pre-existing; not worsened.

---

## Critical scrutiny: are the two authority-surface edits legitimate, or gaming?

**Conclusion: legitimate satisfaction, sibling-parity precedent. NOT gaming.**

The accounting producer derives three facts per file (`src/lib.rs:80-85`, binary `src/main.rs`):
- **owner** ← nearest up-tree `OWNERS` file (`resolve_owners`, main.rs:1449)
- **justification** ← first ADR whose body tokenizes to the tracked path (`resolve_justifications`, main.rs:1547; tokenizer splits on whitespace/`"` `` ` `` `()[],;`, trims `.:#*`, keeps tracked paths ≥4 chars containing `/`)
- **reachability** ← path is a substring of masterplan / root-hub-pointers / doc-catalog, OR lives under a cargo-member dir (`resolve_reachability`, main.rs:1494)

Reconstructed registry rows at HEAD (from `accounting-registry.generated.json`):

| file | owner | justification | reachable_from | verdict |
|---|---|---|---|---|
| `tools/oya-buck-test-wiring-app/Cargo.toml` | `OWNERS:tools/oya-buck-test-wiring-app` | `ADR-0540` | `cargo-members` | KEEP |
| `…/src/lib.rs` | same | `ADR-0540` | `cargo-members` | KEEP |
| `…/BUCK` | same | `ADR-0540` | `cargo-members` | KEEP |
| `…/OWNERS` | same | `ADR-0540` | `cargo-members` | ARCHIVE (advisory) |
| `evidence/multispectrum/g011-…-1781107105.json` | `OWNERS:evidence/multispectrum` | `ADR-0540` | `root-hub` | ARCHIVE (advisory) |

Why this is satisfaction, not gaming:
- **Tool crate files reach via `cargo-members` intrinsically** — they ARE a workspace member; reachability would hold with or without any authority edit. The root-hub edit is needed ONLY for the evidence JSON (not a crate). So the root-hub edit is the minimal correct surface, not a blanket token-stuff.
- **Justification = ADR-0540 for all of them**, the ADR that actually commissions this generator. The ADR-0540 amendment lists the exact governed file paths in a normal "files owned by this ADR" table/list — the same shape an ADR uses to claim any surface. The tokenizer reads them through the documented path. This is how every ADR-justified file in the repo is justified.
- **OWNERS precedent is established in `dev`**: `cloud/cloud-ci/gates/oya-cloud-ci-slo-coverage-app/OWNERS` and `cloud/cloud-kernel/OWNERS` are single-team-name files. The two new OWNERS files (`cloud-ci-platform`, one line each) are byte-for-byte the same convention.
- **Net debt reduction, not neutral**: the new `evidence/multispectrum/OWNERS` also resolves 9 PRE-EXISTING `unowned` advisory keys (other multispectrum evidence files) — those are the 9 of the 29 removed keys beyond the 20 wirings.
- **None of the new tool/evidence paths appear in ANY baseline bucket at HEAD** (verified programmatically) — they are fully accounted, freezing zero new debt.

The substring-reachability and token-justification heuristics are weak by design (LOW findings 1-2 above), but they are the EXISTING contract sibling files satisfy; this PR conforms to it rather than inventing a bypass. The mechanism is sound AND the precedent is the sibling pattern, not a workaround.

---

## Commands run + exact output lines

**1. Both-ways baseline key-set diff (own python over `gates[*][code].keys`):**
```
DEV keys: 55160 HEAD keys: 55131
ADDED (in HEAD not dev): 0
REMOVED (in dev not HEAD): 29
```
Per-bucket: `cloud-ci-target-parity/member_test_code_without_rust_test_target: +0 -20` (libs/oya-check-a11y-discipline … data-class); `cloud-ci-total-accounting/unowned: +0 -9` (pre-existing multispectrum evidence now owned). **Zero `unjustified`/`unreachable`/any-bucket additions** — r2 blocker resolved.

**2. Four gate tests + tool unittest (buck2, lane worktree):**
```
accounting-registry-app-unittest : test result: ok. 6 passed; 0 failed   · Tests finished: Pass 1. Fail 0.
registry-drift-gate              : test result: ok. 2 passed; 0 failed (62.02s) · Tests finished: Pass 1. Fail 0.
target-parity-app-gate           : test result: ok. 1 passed; 0 failed   · Tests finished: Pass 1. Fail 0.
total-accounting-app-unittest    : test result: ok. 5 passed; 0 failed   · Tests finished: Pass 1. Fail 0.
oya-buck-test-wiring-app-unittest: test result: ok. 6 passed; 0 failed   · Tests finished: Pass 1. Fail 0.
```
`registry-drift-gate` green (regenerate + byte-compare) ⇒ all three generated faces are producer-mechanical, NOT hand-edited — the decisive check that the authority edits flow correctly through the producer.

**3. Settle protocol on r3 commits:**
```
ba6ed1e72 (content) : M ADR-0540 · A evidence/multispectrum/OWNERS · M root-hub-pointers.json · A tools/.../OWNERS   [NO *.generated.json]  G SHA256:5grGNU…RZ8E8
a83c7b7bb (settle)  : M accounting-registry.generated.json · M gate-baseline.generated.json · M scm-facts.generated.json  [faces only, LAST]  G SHA256:5grGNU…RZ8E8
```
Content-first, faces-only-last. Both SSH-signed ED25519, same key as r1/r2. (Local principal trust unmapped — `.git/omx-local/allowed_signers` not present in this worktree; same caveat as r1/r2.)

**4. r1/r2 findings status on this head:**
- r1 #1 (doctest): `Cargo.toml:19 doctest = false`; `manifest_missing_lib_doctest_false` = 25 keys, tool absent. FIXED.
- r1 #2 (--check abort): `buck2 run … -- --check` emits 6 `diagnostic code=unsupported_non_library_buck …` lines, `608 rust_test wiring candidates remain`, `TOOL_EXIT=1` (fail-closed), no `parse rust_library` abort. FIXED.
- r1 #3 (golden fixtures): 5 fixtures present; `assert_eq!` ×8, `include_str!` ×8 (byte-equality). FIXED.
- r2 #1 (total-accounting laundering): 0 added keys. FIXED.
- Production safety: `#![forbid(unsafe_code)]` in main.rs & lib.rs; all `unwrap()/expect()` are inside `mod tests` (line 712+, under `#[cfg(test)]`). CLEAN.

**5. Integrity sweep:**
- `git diff --check origin/dev...HEAD` ⇒ CLEAN.
- `evidence/audit-chain.jsonl` ⇒ +1 line, strictly append-only.
- Batch selection: 20 wired `libs/` == first-20 sorted `libs/` target-parity debt keys from dev (verified `True`) — no cherry-picking to dodge hard cases.
- `target_parity.rs` test debt assertion 634→614 with `Verdict::Red` preserved — gate still fails-closed; count tracks real remediation, not a weakening.

---

## Residual risk

Justification and reachability rest on **text-substring/token trust** (`resolve_justifications` token match, `resolve_reachability` `.contains(path)`). A future change could "justify" or "reach" a file merely by naming its path in any ADR or in root-hub-pointers, without that ADR semantically governing the file — the producer cannot distinguish real governance from a path mention. For THIS PR the governance is real (ADR-0540 commissions the generator), but the contract itself is spoofable. Hardening would require structured `affected_surfaces` front-matter parsing rather than free-text token scanning. Most likely production failure even if merged: the generator's text-based BUCK/Rust-test detection misclassifies an uncommon future BUCK shape (already surfaced as `unsupported_non_library_buck` diagnostics for 6 `tools/` members), silently skipping a member that should be wired — coverage gap, not a false-green.
