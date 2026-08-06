---
id: ADR-0545
title: "Embedded-asset hermeticity gate"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-11
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amended_by: [ADR-0549]
depends_on: [ADR-0083, ADR-0132, ADR-0363, ADR-0515, ADR-0540, ADR-0544]
amends: []
related: [ADR-0017, ADR-0131, ADR-0132, ADR-0363, ADR-0510, ADR-0515, ADR-0538, ADR-0539, ADR-0540, ADR-0544]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0545: Embedded-asset hermeticity gate

## Status

**Proposed - 2026-06-11 (authored for founder sign-off; door: one-way).**

## Context

A Rust crate can embed a file at compile time with `include_str!`/`include_bytes!`. The macro
resolves its string-literal argument **relative to the including source file**, and rustc reads that
path from wherever the file sits in the build sandbox. Under buck2 the sandbox layout is the target's
`__srcs` tree: plain/glob `srcs` land at their package-relative short paths, and `mapped_srcs` dict
VALUES land verbatim. If the asset is declared somewhere in the BUCK target but mapped to the WRONG
sandbox location, rustc cannot read it at the include-relative path and the build fails — and buck2's
`failure_filter` reports the downstream `missing rmeta` artifact, not the upstream rustc
`couldn't read …`, so diagnosis is expensive.

This is FRIC-1781131000: the cloud-intelligence cedar-adapter's
`include_str!("../policy/cloud-intelligence.cedar")` was mapped to the wrong sandbox path, so the
crate never built hermetically (masked as a missing-rmeta / ring buildscript failure under cold
ordering). The founder's pipeline-as-product directive (R0) requires that whole defect classes become
unshippable: anti-patterns must be structurally impossible to merge.

This is Bazel/Buck **hermetic-action missing-input** enforcement. Bazel's sandbox fails an action
when a declared input is absent at the path the action reads; strict-deps / missing-input detection
are the same family. We adopt that proven production methodology and reimplement it Rust-native
(founder doctrine: proven patterns, Rust reimplementation), diverging from a full buck2 `aquery` only
in that the gate does a conservative STATIC parse so it runs in presubmit WITHOUT the cold build it is
meant to make unnecessary. Where it cannot statically resolve a site it fails SAFE (a surfaced,
baselined skip), never fail-open (silent) nor fail-closed (false RED).

## Empirical evidence (decisive — collected during ralplan consensus planning)

`buck2 build //oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-authz-cedar-adapter:oya-ci-webhook-gateway-authz-cedar-adapter`
**failed today** before this change:

```
error: couldn't read `buck-out/.../__srcs/src/../../../policy/ci-webhook-gateway.cedar`: No such file or directory
  --> src/lib.rs:36 include_str!("../../../policy/ci-webhook-gateway.cedar")
Missing required input file LPPMD/liboya_ci_webhook_gateway_authz_cedar_adapter-*.rmeta
```

This is a live SECOND instance of FRIC-1781131000: the webhook adapter was "fixed" cargo-only in an
earlier commit, was never buck2-buildable, and was absent from the `oya-ci-required` matrix. The exact
`missing rmeta` mask reproduced. The gate's static analysis independently predicts this failure. The
error also proves the sandbox model: with a SHORT `crate_root = "src/lib.rs"` the include base is
`__srcs/src/`, so `../../../policy/…` ESCAPES the tree; with a ROOT-prefixed `crate_root` (the cedar
comprehension form) the base is `__srcs/<pkg>/src/`, so the include resolves to the mapped VALUE.

## Decision

Ship a standalone, born-blocking, pack-shaped cloud-ci gate
`cloud-ci-embedded-asset-hermeticity` (crate
`ci/facade/embedded-asset-hermeticity`) that mirrors the ADR-0544
gate family (pure kernel + policy DATA + reviewed shrink-only baseline + a `*-gate` rust_test
self-test). The kernel contract (the **tree-namespace rule**):

- **D(T)** = { package-relative short paths of every plain/glob/list src of target `T` } ∪
  { every `mapped_srcs` dict VALUE, verbatim }. Globs are expanded against the on-disk tree.
- **dest(F)** = the including file `F`'s mapped VALUE if `F` is itself a mapped source of `T`, else
  `F`'s package-relative short path. (A `{src: ROOT + "/" + src for src in SRCS}` comprehension maps
  `src/lib.rs` to `ROOT/src/lib.rs`; a plain glob leaves it at `src/lib.rs`.)
- **R** = lexical `normalize(join(dirname(dest(F)), L))` — `.`/`..` collapse, NO filesystem access.
- A site is hermetic iff **some covering target** `T` has `R ∈ D(T)`. Non-membership — including an
  escaped `..` path, which can never be a sandbox-tree member — is the blocking
  `embedded_asset_unmapped_include`. Membership is checked against srcs short-paths and `mapped_srcs`
  VALUES, **never** `mapped_srcs` KEYS (KEY-matching would PASS the original defect and false-RED
  cedar — explicitly refuted in consensus, anti-KEY regression test mandated) and **never** the
  on-disk repo layout (the cedar asset lives outside its crate and is mapped IN).

A file is commonly compiled by several targets (a `rust_library`, its `rust_test`, a `rust_binary`),
each with its own `__srcs` tree; because the gate cannot statically resolve `#[cfg(test)]` gating, an
include is hermetic if mapped by ANY covering target (no false RED for a `tests/` fixture include that
lives only in the rust_test's tree).

Sites the conservative lexical BUCK parser cannot fully resolve are surfaced as non-blocking `skip_*`
codes (`skip_non_literal_argument`, `skip_absolute_literal`, `skip_build_output_path`,
`skip_no_owning_target`, `skip_buck_unparseable`) — counted, baselined shrink-only with reviewed
ceilings, never verdict-flipping. `Report.violations` filters by membership in `VIOLATION_CODES`
(`embedded_asset_unmapped_include`, `embedded_asset_policy_gate_id_mismatch`), the single source of
truth — the `skip_` prefix is documentation, not the filter.

The include scanner is context-aware (skips comments, string/char/raw-string literals, UTF-8 safe) so
it never treats macro text inside a comment or string (e.g. this crate's own `const MACROS`) as a real
invocation.

**Prerequisite (deliverable, same PR):** the webhook cedar-adapter BUCK is rewritten in the cedar
comprehension pattern so its include resolves to its sandbox destination; `buck2 build` then succeeds.
The include literal is deliberately unchanged — editing it would break the cargo build path.

### Automation-default (`--fix` auto-remediator)

Founder directive 2026-06-11: *"gate should prioritize automation where possible; automation should be
the default; enforcement is the extra layer."* The deliverable is therefore detector +
AUTO-REMEDIATOR + blocking backstop, not a detector alone (the face-settle precedent: `--settle
--commit` is the default path, the freshness gate is the backstop). The gate crate ships a
`--check`/`--fix` binary
(`oya-cloud-ci-embedded-asset-hermeticity-fixer`):

- `--fix` derives and applies the corrected mapping for each unmapped asset — the default developer /
  agent path. Two mechanically-derivable transforms: a narrow `mapped_srcs[key]=value` add/replace
  when the target already places the file at the deep path, and the proven **cedar comprehension
  rewrite** of the whole `rust_library` when a short `crate_root` makes a cross-package `../` include
  escape. Both are proven buildable end-to-end: applied to the webhook adapter, the `--fix` output
  produced `BUILD SUCCEEDED` and a clean `--check`.
- The blocking `*-gate` rust_test is the backstop for anything `--fix` cannot SAFELY derive (binaries,
  multi-`..` ambiguity, unmodelled BUCK), and its failure detail prints the exact `--fix` command.

Everything repo-specific (scan roots, the rust/embedded extension sets, build-output dirs) is DATA in
`embedded-asset-hermeticity-policy.json`; the Rust kernel hardcodes no oyatie path and runs on any
repo by repointing the policy.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `ci/facade/embedded-asset-hermeticity/` | create gate crate + policy + baseline + fixer binary | `oya-cloud-ci-embedded-asset-hermeticity-app` | app |
| `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-authz-cedar-adapter/BUCK` | prerequisite hermeticity fix (FRIC-1781131000 second instance) | - | - |
| `.github/workflows/oya-ci-required.yml` | add one gate matrix line | - | - |
| `docs/oya-ci/gate-catalog.md` | document gate, key shape, codes, auto-remediator | - | - |

The gate-crate files owned by this ADR are:
`ci/facade/embedded-asset-hermeticity/BUCK`,
`.../Cargo.toml`, `.../embedded-asset-hermeticity-policy.json`,
`.../embedded-asset-hermeticity-baseline.json`, `.../src/lib.rs`, `.../src/main.rs` (the fixer),
`.../tests/embedded_asset_hermeticity.rs`.

### Integration via Workflow + Ontology

Not applicable. This ADR changes repository admission checks only; it does not emit or consume
Workflow events nor write Ontology objects.

### Positive

- A crate can no longer reach the merge queue with an embedded asset mapped to the wrong sandbox
  location; the FRIC-1781131000 defect class is born-blocking.
- The gate is its own remediator: the default agent path is `--fix`, not a hand edit, so the friction
  the gate polices is auto-converted rather than merely reported (automation-maximalism doctrine).
- The repo became measurably more hermetic: the gate caught a live second instance (the webhook
  adapter) during design, which was fixed as a prerequisite.
- Born pack-shaped: a different repo adopts the gate by repointing the policy.
- The blocking code is born-frozen-empty; conservative skips are baselined shrink-only — never silent,
  never a false RED.

### Negative

- The minimal lexical BUCK parser is the THIRD text-heuristic BUCK parser in the repo
  (oya-buck-test-wiring-app, accounting-registry, this gate); consolidation into the planned
  `oya-buck-syntax-kernel` is a named follow-up (friction row), not done here (smallest-viable-change;
  the shared kernel does not yet exist). Until then a BUCK construct the parser cannot model is a
  surfaced `skip_buck_unparseable`, not a false verdict.
- 21 build-output ELF includes (the bare-metal cloud-kernel arch adapters) and 2 `concat!`/`env!`
  non-literals and 10 unbound `tests/` includes are frozen shrink-only skips until each is wired /
  literalized; they are visible in the baseline.
- The `--fix` comprehension rewrite is conservative — it only rewrites a single `rust_library` with a
  glob srcs + short `src/lib.rs|main.rs` crate_root; anything else is reported manual (the backstop).
  This is deliberate: a half-correct BUCK rewrite is worse than blocking.
- **Crate-prefixed path model:** internally the gate uses crate-prefixed destinations rather than
  buck2's exact short paths; the relabeling is membership-preserving (both `dest(F)` and `D(T)` use
  the same convention) and was validated against the live corpus + the buck2 build, but a true escape
  ABOVE the crate root is detected by non-membership rather than by an explicit escape check.
- **Undeclared buck2 input:** the live-repo test walks to the real repo root to read the tree (the
  established gate-test-family convention), so a warm-cache `buck2 test //cloud/cloud-ci/...` can serve
  a stale verdict after a source edit. Merge authority is unaffected — the `oya-ci-required` matrix
  leg runs on a fresh runner — but this shares the declared-input friction ADR-0544 already names.

### Operational

- Buck2 is the binding local verification surface; the fixer binary runs via `buck2 run`.
- The baseline + ceilings are reviewed, hand-shrunk artifacts; never regenerated to absorb new debt
  (FRIC-1781112000).

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` | Affected | App crate + binary depend inward on serde_json only. |
| `cross-product-refusal` | Not affected | No product boundary is introduced. |
| `port-location` | Not affected | No new port traits. |
| `layer-correctness` | Affected | New gate declares the `app` layer in its BNF name. |
| `composition-root-only` | Affected | The fixer binary is a thin CLI shell; verdict + remediation logic live in the pure kernel. |
| `sdk-kernel-only` | Not affected | No SDK kernel boundary change. |

## Alternatives Considered

**Alternative 1 - On-disk-layout oracle (check the asset exists at the include-relative repo path)**
- Description: resolve the include against the repo filesystem and assert the file exists there.
- Cons: the original defect is INVISIBLE to it — the cedar asset exists on disk (in a sibling
  package); the bug is the sandbox MAPPING, not the file's existence.
- Reason rejected: it would pass the exact defect FRIC-1781131000 describes.

**Alternative 2 - Match against `mapped_srcs` KEYS**
- Description: treat the dict KEY (`//pkg:name` label) as the destination set.
- Cons: passes the original defect (the key was right, the value wrong) and false-REDs the correct
  cedar adapter (whose key is a label, not the resolved path).
- Reason rejected: refuted in consensus by arithmetic on all three live mapped_srcs cases; the
  anti-KEY regression test guards it.

**Alternative 3 - Detector only; leave fixing to humans**
- Description: ship the blocking gate without `--fix`.
- Cons: violates the founder automation-default directive — a mechanically-derivable fix that the gate
  can only block leaves the deliverable incomplete.
- Reason rejected: the corrected mapping IS mechanically derivable (proven buildable); the gate ships
  the auto-remediator as the default path and remains the backstop.

**Alternative 4 - Build the shared `oya-buck-syntax-kernel` now and consume it**
- Description: extract the BUCK parser as a shared kernel in this lane.
- Cons: scope creep; the kernel does not yet exist and three consumers would need migrating.
- Reason rejected: smallest-viable-change — ship a minimal in-crate parser and file the consolidation
  as a friction row (extraction at the second consumer, per SHARED-KERNEL-CANDIDATES doctrine).

## Verification

- `buck2 build //ci/facade/embedded-asset-hermeticity/...`
- `buck2 test //ci/facade/embedded-asset-hermeticity:oya-cloud-ci-embedded-asset-hermeticity-app-unittest`
- `buck2 test //ci/facade/embedded-asset-hermeticity:oya-cloud-ci-embedded-asset-hermeticity-app-gate`
- `buck2 build //oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-authz-cedar-adapter:oya-ci-webhook-gateway-authz-cedar-adapter` (prerequisite fix; was failing, now `BUILD SUCCEEDED`)
- E2E auto-remediation: break the webhook BUCK, `buck2 run …:oya-cloud-ci-embedded-asset-hermeticity-fixer -- --fix`, then `buck2 build` the adapter → `BUILD SUCCEEDED` + `--check` clean.

## Known Limitations and Destination

> **Amended by ADR-0549 (2026-06-11):** the destination shipped. `libs/oya-buck-syntax-kernel`
> is the shared sound parser + fixer self-validation harness, and this gate's detect and `--fix`
> lanes now ride it: the comment-guard refusal, the first-occurrence name binding, and the
> bare-var/`POLICY_REL` notes below are RETIRED as current behavior (see ADR-0549 D4 for the
> migration table). The text below is preserved as the historical record of the pre-kernel
> scope boundaries.

This ADR records the in-crate implementation's known scope boundaries honestly. The destination for
all of them is `oya-buck-syntax-kernel` (FRIC-1781131000-buck-syntax-kernel, task #10): a shared,
Starlark-aware BUCK parser that makes these limitations impossible by construction.

### --fix: comment-bearing target blocks reported manual

`apply_remediation` refuses to edit any target block that contains an out-of-string `#` character.
A comment before `)` such as `deps = [],  # note` makes the comma-placement heuristic unreliable;
rather than risk emitting a double-comma (corrupt BUCK, buck2 parse error), the fixer classifies
the block as manual. The reviewer-cited probe (`deps = [],  # trailing comment`) is correctly
refused with an actionable message. **Destination**: the shared parser handles comments natively,
removing this exception entirely.

### --fix: first-occurrence target binding

`apply_remediation` locates the target block by `out.find("\"<name>\"")` — the first occurrence.
A BUCK file with two targets sharing a name prefix (e.g. `"svc"` and `"svc-ffi"`) could mis-bind
if the shorter name appears as a substring of the longer one before it. In practice buck2 requires
unique names within a file, so duplicates are impossible; substring prefix collisions are the
residual risk. The same pattern is replicated in `validate_remediation_output`. **Destination**:
shared parser provides unambiguous target location by parse-tree position.

### --fix: bare-var `mapped_srcs` (IDENT form) reported manual

When `mapped_srcs = SOME_VAR` references a top-level variable, `apply_remediation` cannot inject
an entry into the variable's assembly site (which may be spread across multiple assignment lines).
These sites are surfaced as `[manual]` with an actionable message. **Destination**: the shared
parser models variable assignment sites and can inject at the correct location.

### --fix: `POLICY_REL` is hardcoded in the binary

`main.rs` hardcodes `POLICY_REL` to the crate's path within the oyatie repo. A different repo
adopting the gate would need to pass `--policy`. This is a CLI surface limitation, not a kernel
limitation (the kernel is fully pack-shaped). **Destination**: the binary should default to
`--policy` auto-discovery via the policy filename in the same directory as the binary.

### Fixer self-validation: `validate_remediation_output` parse scope

The round-trip validation calls `parse_buck_targets` with an empty `crate_files` slice (no glob
expansion). This is sufficient to verify structural parse success and mapped_dest value presence
for inline dict entries. It does not validate comprehension-form mapped_srcs (glob expansion
requires the filesystem). ComprehensionRewrite output is validated by `validate_remediation_output`
via the target-findable check only; the mapped value is verified structurally by `rewrite_to_comprehension`'s own format correctness. **Destination**: shared parser with filesystem-backed
glob expansion for full round-trip fidelity.

## References

- FRIC-1781131000: cedar-adapter include mapped to the wrong sandbox path → non-hermetic build masked
  as missing-rmeta.
- FRIC-1781190000: auto-remediator corruption class; guard v1 (findability-only) was shallow; guard
  v2 (this PR) uses parse_buck_targets round-trip + comment-guard refusal. Correction event:
  FRIC-1781190000-guard-v2 in the friction ledger.
- Founder directive 2026-06-11: automation should be the default; enforcement is the extra layer.
- Founder pipeline-as-product R0 directive: anti-patterns must be structurally unshippable.
- Bazel sandbox strict-deps / missing-input detection; Buck2 hermetic action inputs (the production
  precedent reimplemented Rust-native).
- FRIC-1781112000: same-PR baseline regeneration launders new debt; freeze the ratchet against a
  reviewed, non-regenerable reference.
- ADR-0544: friction-ledger closed-loop accounting meta-gate (the gate-family pattern mirrored here).
- ADR-0540: target-parity gate (shrink-only baseline + reviewed ceiling precedent).
- ADR-0515: cloud-ci required status context as merge authority.
