---
id: ADR-0549
title: "oya-buck-syntax-kernel: one sound BUCK/Starlark parsing oracle + fixer self-validation harness"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-11
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0083, ADR-0131, ADR-0515, ADR-0544, ADR-0545, ADR-0547]
amends: [ADR-0545, ADR-0547]
related: [ADR-0132, ADR-0363, ADR-0516, ADR-0540, ADR-0546, ADR-0548]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0549: oya-buck-syntax-kernel — one sound BUCK/Starlark parsing oracle + fixer self-validation harness

## Status

**Proposed - 2026-06-11 (authored for founder sign-off; door: two-way — consumers can pin or
revert per-gate without unwinding the substrate).**

## Context

Three-plus gate/tool crates carried private, divergent text-heuristic BUCK parsers
(`oya-cloud-ci-embedded-asset-hermeticity-app`, `oya-cloud-ci-kernel-purity-app`,
`tools/oya-buck-test-wiring-app`, `oya-cloud-ci-accounting-registry-app`'s target-parity
producer). Hostile review beat each of them, repeatedly and in the same ways:

- **comment-blind `call_block`** — `rust_library(` or `third-party//:x` inside a comment or
  string was treated as a real target/dep (PR #690/#691 review classes; ADR-0545 "Known
  Limitations and Destination");
- **first-occurrence name binding** — fixers located a target block by the first occurrence of
  `"name"` as a substring, mis-binding when the literal appeared earlier in another field
  (ADR-0545 residual risk);
- **paren-in-string / paren-in-comment depth miscounts** — a stray `)` ended a block early and
  hid every dep below it from the detect lane (#691 H5; #693 LOW-X2 backslash-escaped quote);
- **backslash-newline continuation hiding deps** — a Starlark string continued across
  `\` + newline cooks to a single joined value; the line-bounded strippers reset string state at
  the newline and saw a truncated token (`third-party//:k` + `ube` → dep `k`, `kube` invisible)
  (#693 LOW-2 follow-up; ledgered as FRIC-1781230000);
- **corrupting fixers** — comma-placement heuristics produced missing-comma/double-comma BUCK
  corruption (FRIC-1781190000), which forced ADR-0545's comment-bearing-block refusal guard and
  ADR-0547 D6's round-3 descope of BUCK `--fix` to refusal-only (FRIC-1781200001).

Repo doctrine (enforcement-layering; ADR-0548 pipeline-as-product): structural impossibility
beats per-lane discipline. A parser fix in one consumer silently leaves the others wrong; the
duplication itself is the defect (FRIC-1781131000-buck-syntax-kernel, queued-shared-kernel since
ADR-0545; FRIC-1781200001 since ADR-0547).

## Decision

Extract **`libs/oya-buck-syntax-kernel`**: the single shared, SOUND lexer/parser for the
Starlark subset the gates consume, plus span-accurate safe-edit primitives and the fixer
self-validation harness. Migrate the two cloud-ci gate consumers onto it.

### D1 — Sound parsing core (bespoke rowan-style; W2 doctrine)

A hand-rolled lexer + recursive-descent parser, std-only, with byte-exact spans on every node:

- **Lexer**: comments are trivia (never depth/state-bearing); strings are cooked with full
  escape state (`\\` pairs, `\"`, backslash-newline continuation, raw/triple-quoted forms);
  newlines separate statements only at bracket depth zero (Python implicit line joining);
  multi-char operators (`==` etc.) are single tokens so kwarg detection is unambiguous.
- **Parser**: models `IDENT = expr`, `IDENT[expr] = expr`, top-level calls, string/int/ident
  literals, `+` chains, list/dict literals, dict comprehensions, and nested calls. Anything
  outside the subset becomes an exact-span `Opaque` node — **fail-honest**: never
  misinterpreted, never silently dropped. Structurally undelimitable input (unterminated
  string/call, double comma) is a hard `ParseError` — **fail-closed**.
- **Eval**: static resolution of string vars, glob vars, concat chains, dict literal /
  comprehension destination values, `VAR["k"] = v` assembly, and **target binding by the actual
  `name` kwarg** (retiring first-occurrence substring binding), plus the proven buck2-style
  glob matcher.
- **Edit**: span-based primitives (`insert_kwarg`, `insert_dict_entry`, `remove_list_element`,
  `replace_span`) that read comma positions from PARSED spans — the missing/double-comma
  corruption class is impossible by construction; unprovable edits are refused.

Precedent (REFERENCE ONLY, per the W2 founder ruling: bespoke rowan-style is the house default;
tree-sitter is research-only, no dependency): rust-analyzer's rowan lossless-span design and
tree-sitter's error-tolerant node model informed the span-fidelity + opaque-node shape. No
third-party parser dependency exists or is planned.

### D2 — R0 pack-shape (pure kernel)

No filesystem, no policy, no repo specifics, no third-party deps (std only). Inputs are strings
(+ caller-supplied file lists for glob expansion); outputs are parsed structures, evaluated
values, edited strings, or refusals. All I/O and all policy live in consumers. The crate name
matches `*-kernel`, so the kernel-purity gate itself scans it from birth.

### D3 — Fixer self-validation harness (the write-through guard)

Every fixer must route rewrites through `harness::guarded_rewrite`:

1. **Reparse**: the candidate content is reparsed with this kernel; a structurally corrupt
   rewrite is refused before any caller-visible success.
2. **Semantic hook**: a CALLER-SUPPLIED validation closure runs over the parsed candidate
   (e.g. "target still present", "injected value visible via the real detector parse path",
   "no collateral dep removed", "no dangling feature refs") and refuses on `Err`.
3. **Refusal returns the pre-image**: the only sound outcome of a failed validation is keeping
   or restoring the original bytes.
4. **`PreImageRegistry`**: FIRST pre-image per path key wins (a file edited twice rolls back to
   its ORIGINAL content — the #693 LOW-X3 class), with deterministic path-ordered rollback.

Corruption-refusal fixtures pin the historical vectors: missing comma, double comma from
comment-blind heuristics, dangling feature refs (via the semantic hook), unterminated blocks.

### D4 — Consumer migration

| Consumer | Private code dropped | Replaced by | Behavior delta |
| --- | --- | --- | --- |
| `oya-cloud-ci-embedded-asset-hermeticity-app` | `call_block`, `field_value_expr`, `find_top_level(_keyword)`, `quoted_strings`, `top_level_string_vars`/`glob_vars` scanners, `unquote_concat`, `mapped_dict_values`, `eval_value_expr`, `split_top_level`, `brace_block`, `find_var_assignment`, local `glob_match`, comment-guard refusal (`block_has_out_of_string_comment`) | kernel parse/eval/edit + `guarded_rewrite`; public fns (`parse_buck_targets`, `resolve_mapped_var`, `glob_match`, `apply_remediation`) keep signatures | detect lane: none on the live corpus (baseline set-equality gate test green, baseline file untouched). `--fix`: comment-bearing blocks are now EDITED soundly instead of refused (RED→GREEN fixtures `apply_remediation_trailing_comment_block_is_edited_soundly`, `apply_remediation_comment_block_with_missing_comma_is_edited_soundly`); comprehension-dict insertion is refused up front instead of post-hoc. |
| `oya-cloud-ci-kernel-purity-app` | `find_block_end`, `strip_starlark_comment_and_strings`, raw-block `extract_thirdparty_tokens`, refusal-only `remove_buck_dep_line` stub, ad-hoc pre-image map | kernel parse + `call_strings` extraction; sound `remove_buck_dep_line` via `remove_list_element` + `guarded_rewrite`; shared `PreImageRegistry` in `apply_fixes_with_validator` | detect lane: none on the live corpus (differential probe: 865/865 BUCK files parse, dep-set extraction identical to legacy on every file) — and the FRIC-1781230000 continuation gap CLOSES (RED fixture `backslash_newline_continuation_does_not_hide_dep`). `--fix`: the ADR-0547 D6 round-3 BUCK refusal-only descope is closed — a mechanically dead dep (all five D6 bounds hold) now also drops its `rust_library` BUCK edge, guarded by the harness; unsound shapes (vars, `select`, unterminated blocks) still refuse with the file byte-identical. |

Where the prior gates' detect lanes were comment-blind in the OVER-reporting direction (a dep
mention in a comment counted), the kernel fixes that too; the live corpus carries no such shape
(differential-probe proven), so reported findings are identical before/after.

**Fail-closed posture for unmodeled content**: target enumeration walks EVERY parsed call
expression in the document (top-level statements, assignment-wrapped `X = rust_library(...)`,
index-assignment values, calls nested in any expression) via `BuckDoc::visit_calls`; every span
the parser could NOT fully model is raw-scanned from any `rust_library(` occurrence — a
`rust_library` call with an `Opaque` argument (its exact span), a `Stmt::Opaque` statement
(including modeled statements with TRAILING unmodeled tokens, which the parser demotes to
`Stmt::Opaque` rather than silently truncating), an `Assign`/`IndexAssign` whose value or key
contains expression-level `Opaque` content (the postfix-index / unmodeled-primary-ternary /
discarded-comprehension-iter wrappers the hostile re-review produced), and an unparseable BUCK
file in full. A wrapper therefore lands in exactly one of two buckets: parsed (enumerated) or
unmodeled (raw-scanned) — never silently outside both.
For the born-blocking DETECTOR an over-approximation can only ADD findings, never hide one
(the same posture the pre-kernel EOF fallback took). On the hermeticity side, a target call
carrying a positional opaque tail (the ternary `srcs = [...] if c else [...]` shape) is demoted
to `unparseable` — a visible skip, never a silent narrowing to one branch. The REMOVER and all
edit paths are independently guarded and refuse unsound input outright.

### D5 — Deferred consumers (audited honestly)

- `tools/oya-buck-test-wiring-app` — carries its own balanced-paren BUCK scanner; CLI surface is
  retirement-marked (`cli_surface_policy`), so migration is deferred until its successor lane
  decides retire-vs-migrate. The non-duplication lint (below) tracks it.
- `oya-cloud-ci-accounting-registry-app` (target-parity producer) — `has_rust_test_target` is a
  `contains("rust_test(")` substring probe (comment-blind in the over-counting direction only;
  benign for its parity purpose). Migration is mechanical once that crate takes the kernel dep;
  deferred to keep this change reviewable.
- Future: a `workspace-glob-coverage`-style non-duplication lint ("no new private BUCK parser
  outside libs/oya-buck-syntax-kernel") is the ratchet that keeps the oracle single; queued in
  the friction ledger under FRIC-1781131000-buck-syntax-kernel's closure evidence.

### D6 — Verification contract

- Kernel unit suite ports ALL parser fixtures from both gates (H5, H6, LOW-X2, MED-4 indented
  close, multibyte/em-dash, cedar comprehension shapes, value-not-key membership) plus the new
  vectors (continuation cooking, double-comma hard error, decoy-name binding, harness refusals).
- Identical-findings proof on the live corpus, three independent legs:
  1. differential probe over every BUCK file in the repo (legacy extraction vs kernel
     extraction, byte-for-byte dep-set equality; 865/865 identical, 0 parse failures);
  2. the hermeticity gate's live-corpus baseline SET-EQUALITY test passes with the baseline
     file untouched;
  3. both gate binaries run at base and at head produce identical stdout + exit codes.
- `buck2 test //cloud/cloud-ci/...` + `//libs/oya-buck-syntax-kernel:...-unittest` green.
- Follow-up parser hardening evidence:
  `evidence/multispectrum/buck-syntax-parser-depth-cap-20260625-1782429922.json`.

## Consequences

- One parsing oracle: a parser fix lands once and every consumer inherits it; drift between
  gate parsers is structurally impossible for the migrated consumers.
- The fixer corruption class (FRIC-1781190000) is closed at the substrate: every rewrite is
  reparse-validated with first-pre-image rollback, not heuristically trusted.
- BUCK `--fix` for kernel-purity is live again (FRIC-1781200001 closed); the hermeticity fixer
  no longer refuses comment-bearing blocks (ADR-0545 limitation retired).
- The kernel is itself governed: `*-kernel` naming puts it inside the kernel-purity scan, and
  its std-only contract keeps it cutover-stable (ADR-0510 indifferent).
- Residual risk: the modeled subset is intentionally small; new BUCK macro shapes surface as
  `Opaque` (detect lanes over-approximate, fixers refuse) rather than silent misreads — the
  honest failure mode ADR-0548 D7 ("soundness defines possible") prescribes.

## References

- FRIC-1781131000-buck-syntax-kernel (queued-shared-kernel → closed by this ADR's crate +
  migration); FRIC-1781200001 (BUCK --fix descope → closed); FRIC-1781230000 (backslash-newline
  continuation detect gap → closed, RED fixture).
- ADR-0545 "Known Limitations and Destination"; ADR-0547 D6 (five Cargo sound bounds, round-3
  BUCK descope); ADR-0548 pipeline-as-product doctrine (D7 cites this ADR as the shared-harness
  destination).
- rust-analyzer rowan / tree-sitter design notes — REFERENCE ONLY per the W2 founder ruling
  (bespoke rowan-style default; tree-sitter research-only; no dependency).
- Founder doctrine: proven patterns, Rust reimplementation; enforcement layering (structural
  impossibility > hooks); automation maximalism (manual-twice = write the automation).
