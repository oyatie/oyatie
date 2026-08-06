---
id: ADR-0546
title: "Canonical-JSON determinism gate"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-11
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0083, ADR-0132, ADR-0363, ADR-0515, ADR-0539, ADR-0544]
amends: []
related: [ADR-0017, ADR-0083, ADR-0131, ADR-0132, ADR-0363, ADR-0515, ADR-0538, ADR-0539, ADR-0540, ADR-0544]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0546: Canonical-JSON determinism gate

## Status

**Proposed - 2026-06-11 (authored for founder sign-off; door: one-way — the canonical-form
parameters are a one-way commitment).**

## Context

Machine-readable JSON is a load-bearing governed surface across the repo: `specs/root-hub-pointers.json`
is the authoritative agent entry surface, `specs/masterplan.json` and `specs/master-plan-sequencing.json`
drive sequencing, and dozens of cloud-ci gates read JSON policies and faces. These files are
content-addressed and cross-referenced; their BYTES, not just their logical content, matter for diffs,
merge conflicts, and the cross-artifact-agreement gate's byte comparisons.

The motivating friction is FRIC-1781130000 (a real defect this session): a lane silently re-encoded
`specs/root-hub-pointers.json` from escaped-unicode (`\uXXXX`, `ensure_ascii=true`) to literal UTF-8
(`→ µ § —`), producing ~30 lines of churn on content unrelated to the lane's one intentional pointer
addition. Some tool/editor in the lane round-tripped the file through a JSON serializer with different
`ensure_ascii`/indent/key-order settings. This is a **hermetic-output failure**: the same logical
content serialized to different bytes. Non-deterministic serialization pollutes diffs, risks merge
conflicts, and defeats cross-artifact agreement. The interim fix restored the dev bytes by hand — a
manual patch, not a forcing function. Founder automation-maximalism doctrine: manual-twice = write the
automation; deterministic output is part of "hermetic output for any repo" (founder R0).

Hyperscaler precedent (founder doctrine: proven patterns, Rust reimplementation). Deterministic,
gated source serialization is a universal paved-road capability at scale:

- **`gofmt` / `cargo fmt --check`** — a single canonical source form, machine-enforced in CI, no
  per-directory style flags. "Gofmt's style is no one's favorite, yet gofmt is everyone's favorite."
- **Bazel `buildifier`** — canonical BUILD/`.bzl` formatting enforced as a presubmit; the same
  binary checks and fixes (`--mode=check` vs `--mode=fix`).
- **`prettier --check`** — deterministic re-serialization compared against committed bytes; CI fails
  on drift, the developer runs the writer to fix.
- **RFC 8785 JSON Canonicalization Scheme (JCS)** — §3.2.2.2 mandates minimal string escaping
  (literal UTF-8, not `\uXXXX`) as the canonical form. We adopt the *property* (one canonical byte
  form per logical content) while keeping a pretty-printed, human-diffable layout rather than JCS's
  whitespace-free single line, because these files are hand-read governance artifacts, not wire-format
  payloads. The divergence is deliberate and recorded here.

We reimplement this Rust-native as a cloud-ci gate (the merge-admission product per ADR-0515), not a
separate formatter service or a shell hook (ADR-0363 retires external coordination tooling; the
founder enforcement-layering doctrine puts gates + branch protection as canonical and hooks as a
safety net only).

## Decision

Add `ci/facade/canonical-json` as a pure cloud-ci determinism gate.

NAME: oya-cloud-ci-canonical-json-app
JUSTIFICATION:
- microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515.
- bc-tokens = canonical-json: the bounded concern is deterministic JSON serialization.
- layer = app: the crate is an executable CI gate surface with a pure canonicalizer kernel.
- single-concern + flat per ADR-0132; exemptions claimed: none.

**The canonical form (the one-way-door parameters).** A tracked `*.json` under a governed root is
canonical iff its committed bytes equal the canonical re-serialization with:

| param | value | justification |
|---|---|---|
| `ensure_ascii` | **false** (literal UTF-8) | Consistent with the faces serializer `accounting-registry::to_canonical_json` (18 310 literal-non-ASCII lines, 0 escaped) and RFC 8785 JCS minimal escaping. The FRIC defect was *silent ungated churn*, not literalness — the fix is to PICK a form and ENFORCE it, not to forbid literal bytes. |
| `sort_keys` | **false** (source order) | The defect class is rewrite nondeterminism, not key-order ambiguity. Sorting would churn 1452 repo JSON files (measured) and destroy intentional ordering on the agent entry surface; gofmt/prettier/buildifier preserve semantic order. The faces serializer keeps its OWN sorted form and is excluded (single-owner). |
| `indent_width` | **2** | Matches the faces serializer and 223/224 specs files. |
| `trailing_newline` | **true**, `newline` = LF, `utf8_bom` = false | POSIX text convention; matches the faces serializer and gofmt/prettier; one canonical byte form per (content, key order). |

These parameters are a one-way commitment: changing `ensure_ascii` or `indent_width` later would
re-churn the entire governed corpus. They are settled here and pinned as DATA in the policy.

**Consistency with the faces serializer (the critical constraint).** The `*.generated.json` faces are
produced by the materialize/settle tooling (`accounting-registry::to_canonical_json`) with its own
serializer, and the freshness/registry-drift gates compare committed face bytes to producer stdout
VERBATIM. This gate's canonical form is therefore chosen *consistent* with that serializer (literal
UTF-8, 2-space, trailing newline) — but the faces are **excluded** from this gate's scope
(`*.generated.json` suffix exclusion), because they are owned byte-verbatim by freshness. Single-owner
principle: this gate never double-gates a face, and never fights the settle tooling. Likewise
`specs/fixtures/*` are excluded: they are INPUTS owned byte-/parse-verbatim by 5 other gate test
suites (firewall, cross-artifact-agreement, staleness-reaper, total-accounting, automation-ratchet);
reformatting them is a cross-gate-ownership violation.

**Self-contained lexical canonicalizer (NOT `serde_json::to_string_pretty`).** The canonical bytes are
produced by a hand-written JSON lexer → CST → formatter with ZERO `serde_json` in the canonical path.
`serde_json`'s `preserve_order` and `arbitrary_precision` features are reindeer-unioned ON
workspace-wide under buck2; routing canonical bytes through `serde_json` would make the gate's output
depend on a build-system feature union it does not control — the exact silent-byte-drift class it
polices. Number lexemes round-trip verbatim (no precision/format rewrite), object key order is
preserved natively, duplicate keys are detected during CST construction, and a depth bound returns an
error rather than a stack abort (no-panic doctrine, ADR-0083). `serde_json` is used only for policy
parsing and in tests — never to produce canonical bytes.

**Born pack-shaped (founder R0).** The governed roots, the canonical-form parameters, and the
exclusions are DATA in `canonical-json-policy.json`. The Rust kernel hardcodes no repo path nor any
oyatie string; another repo adopts the gate by repointing `governed_roots` and settling its own
`canonical_form`. The kernel fixes only the canonical-form ALGORITHM, not any path.

**Kernel contract.** `canonicalize(bytes, form) -> Result<String, CanonError>` is the pure core,
shared by the gate (check) and the fixer (write) — check == fix by construction (`cargo fmt --check`
precedent). `collect_observed(root, policy) -> Observed` performs the only I/O (read-only filesystem
walk of the governed roots). `evaluate_keyed(policy, observed) -> BTreeSet<Finding>` is pure and
unit-testable without a filesystem; one finding per offending file, keyed by repo-relative path.
`evaluate` is the bare-code Green/Red projection. Violation codes: `json_not_canonical` (the FRIC
drift class; losslessly fixable), `json_parse_error` (invalid JSON, lone surrogates, NaN/Infinity,
leading-zero numbers, non-UTF-8), `json_duplicate_key` (canonical form undefined — the fixer refuses
rather than silently drop a member).

**Automation is the default; enforcement is the backstop (founder directive 2026-06-11).** "Gate
should prioritize automation where possible; automation should be the default, enforcement is the
extra layer." Canonicalization is mechanically derivable, so the canonical answer to a
`json_not_canonical` finding is to RUN the auto-remediator, never to hand-edit bytes. The deliverable
is therefore *detector + auto-remediator + blocking backstop*, mirroring the face-settle precedent
(`oya-cloud-ci-face-settle --settle --commit` is the documented default remediation; the freshness
gate is the backstop). The gate binary's `--fix` mode rewrites every non-canonical governed file to
canonical form in one pass
(`buck2 run //ci/facade/canonical-json:oya-cloud-ci-canonical-json-bin -- --fix`).
It REFUSES parse/duplicate-key defects (human-judgment residue, not drift). The blocking gate's
failure output prints this EXACT command (the `AUTO_FIX_COMMAND` constant, asserted by a fixture so a
typo'd target can never ship). Per the founder CLI-retirement directive, the binary is LOCAL BRIDGE
feedback only; merge authority is the gate's buck2 `rust_test` behind `oya-ci-required`, never the
binary.

**Shared-serializer consistency (the explicit one-way-door, founder directive 2026-06-11 point 2).**
`--fix` and the face-settle tool MUST NOT fight each other in a rewrite loop. We guarantee this two
ways: (a) the `*.generated.json` faces are EXCLUDED from this gate's scope, so `--fix` never touches a
file the settle tool owns; and (b) the two forms agree on the FORMATTING axes they share — literal
UTF-8 (`ensure_ascii=false`), 2-space indent, trailing newline — so neither rewrites the other's
whitespace/encoding. The one axis they differ on is key order: `accounting-registry::to_canonical_json`
SORTS keys (recursive `BTreeMap`), while this gate preserves source order (`sort_keys=false`). That is
*not* a conflict, because sorted output is itself a fixed point of an order-preserving canonicalizer:
re-running this gate over an already-sorted file leaves the (already-sorted) order untouched. The
correct framing is therefore the fixed-point property, not byte-identity of the two forms: the settle
tool's output is a FIXED POINT of this gate's `--fix`. A regression fixture
(`settle_style_canonical_file_fix_is_a_no_op`) asserts exactly this — a file already in the settle byte
form is a `--fix` no-op — so the no-rewrite-loop guarantee is tested, not assumed.

**Ratchet (zero baseline, born-blocking).** A dry-run of the fixer over the governed `specs/` root
(excluding faces + fixtures) found exactly 7 non-canonical files — all escaped-unicode and/or
non-2-space drift, including the FRIC-1781130000 exemplar `specs/root-hub-pointers.json`. Under the
30-file fix-in-PR threshold, this PR fixes all 7 in place with the fixer (verified: parsed JSON
identical before/after — only bytes change), so the live corpus is GREEN at a ZERO baseline. All
three codes are born-blocking-empty: any NEW non-canonical governed json fails closed. This
intentionally re-applies the literal form that FRIC-1781130000's interim fix had reverted — drift in
EITHER direction is henceforth gated.

**Scope + expansion.** The initial governed root is `specs/`. Expansion to `docs/`, `oya/`, and
`cloud/` dashboards is a follow-up IP per governed root (a fixer run + a one-line policy-data edit),
shrink-only — never a single mass-reformat PR (that churn would itself be the anti-pattern).

## Consequences

- A whole class of invisible serialization churn becomes impossible to ship under the governed roots;
  the next lane that round-trips `specs/root-hub-pointers.json` through a different serializer is
  born-blocked, with a one-command fix.
- The gate is repo-agnostic and pack-shaped: a non-oyatie repo enables it by editing one policy file.
- The canonical form is now load-bearing: changing `ensure_ascii`/`indent_width` is a one-way door
  requiring a corpus-wide re-fix; it is settled in this ADR + the policy DATA.
- Trade-off: the gate's lexical canonicalizer is bespoke rather than `serde_json::to_string_pretty`.
  Justified above (the serde feature union is the very drift class being policed); the implementation
  is ~400 lines, fully RED/GREEN tested, with an idempotence property under both `ensure_ascii`
  branches.
- Residual UNGATED drift classes (explicit): because the gate preserves key order verbatim
  (`sort_keys=false`) and number lexemes verbatim, a rewrite that reorders object keys or respells a
  number lexeme (e.g. `1E9` ↔ `1e9`) still passes this gate — those two axes are caught only by
  diff review, not by this determinism check.

## Dogfood closure

FRIC-1781130000 is dispositioned terminal (`fixed-in-PR`) with evidence citing this gate
(`gate-id: cloud-ci-canonical-json`), ADR-0546, and the PR. The friction-accounting gate
(ADR-0544) stays green on the updated ledger.

## Verification

buck2-only. `oya-cloud-ci-canonical-json-app-unittest` (the RED/GREEN canonicalizer fixture suite
— the test target owns the count, not this prose:
escaped↔literal, indent/minify/CRLF/BOM drift, parse errors, duplicate keys, lone surrogates,
truncated/non-ASCII `\u` boundary safety, NaN/Infinity, leading-zero numbers, depth bound,
number-lexeme verbatim, idempotence under both `ensure_ascii` forms, `newline`/`utf8_bom` live-DATA
honoring, case-insensitive `.json` matching, fixer fixed-point + duplicate-key refusal + RED→fix→GREEN
auto-remediation + settle-style no-op, exclusions). `oya-cloud-ci-canonical-json-app-gate` (4 tests:
live-corpus born-blocking GREEN at zero baseline over the real `specs/` corpus + exclusions +
gate-id + FRIC exemplar). The firewall `gate_registration` meta-test confirms the gate is registered
in `oya-ci-required.yml`. Full `buck2 test //cloud/cloud-ci/...` is GREEN (40 targets) after the
faces settle.
