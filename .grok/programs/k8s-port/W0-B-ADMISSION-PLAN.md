# W0-B (G002) mechanical Go→Rust port engine — admission plan

**Status:** PLAN ONLY — product engine code is **hard-stopped** until G001 (#1561) is promoted  
**Story:** Ultragoal `G002` / MPV2-0046 / wave `W0-B`  
**Authority:** ADR-0637, ADR-0638; approved ralplan rev 5 (`pending-approval.md` SHA-256 `7010aebc4a1423d5edc2df40548a9945135a509b52fb9a8085080b7ff8e3e888`); handoff `docs/programs/k8s-port/operations/W0-A-20260805-gjc-handoff.md`  
**Program surface:** `.grok/programs/k8s-port/` (this plan + ready-gate)  
**Machine gate:** [evidence/W0-B-ready-gate.json](evidence/W0-B-ready-gate.json)  
**Authored:** 2026-08-05  
**Orchestration:** `.grok` harness + `git` / `gh` / `bd` / `buck2` / `cargo` only. No gjc / omc / omx / hermes. No hand-edit of `*.generated.json`.

---

## 0. G001 gate (hard stop)

| Check | Source of truth | Plan-time state (2026-08-05) |
|---|---|---|
| PR #1561 MERGED into `dev` | `gh pr view 1561 --json state,mergedAt,mergeCommit` | **OPEN** (not MERGED). Local `PROGRAM.json` last requery: ready-for-review, CI green, agent dual-critic APPROVE; waiting oya-ci-required then agent squash-merge (human APPROVE not mandatory). |
| Promoted tip has `oya-ci-required` SUCCESS | post-merge Actions run on squash SHA | **Not available** until merge. |
| Post-merge product-completion packet non-DRAFT | `.grok/programs/k8s-port/evidence/G001-post-merge-packet-DRAFT.md` → rename/fill | **DRAFT only**. |
| Beads `oyatie-7xf` closed with merge SHA + packet path | `bd show oyatie-7xf` | **in_progress** (pre-merge). |
| Ultragoal G001 durable checkpoint | goals/ledger (read-only provenance under `.gjc/.../ultragoal/`) | G001 still `active`; G002 `pending`. |

**Rule:** If any row above is not green, **do not** create `build/port-engine/**`, amend root `Cargo.toml` members, author product rule packs, or open a W0-B PR. Parallelizable work allowed now is **this plan, ready-gate checklist, and G001 closeout only**.

Live re-confirm before any code:

```bash
gh pr view 1561 --json state,mergedAt,mergeCommit,baseRefName,headRefOid,statusCheckRollup,reviewDecision
git fetch origin dev
git rev-parse origin/dev
# after merge: confirm origin/dev tip == mergeCommit.oid and post-merge oya-ci-required SUCCESS
```

---

## 1. W0-B objective (binding)

From Ultragoal G002 / plan §5 + Intent Reconciliation Q2:

1. **Six `oya-port` crates** under `build/port-engine/{ports,core,adapters,facade}/` (re-home of §5.1 table away from `ci/*`).
2. **One** root `Cargo.toml` members-line amendment admitting that tree (ADR-0637 / ADR-0538 exception).
3. **Neutral rule pack v0** under `specs/port-rules/lang/go-rust/**` + `specs/port-rules/idiom/**` with selecting fixtures; rule without fixture **cannot load**.
4. **Pinned out-of-band** `go/packages`+`go/types` SourceModel extractor; two extractions **byte-identical**; licensing admission complete.
5. **Front-end sizing** published (§5.10.5 measurement output).
6. **Six-axis receipt** schema end-to-end: `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest`.

**Non-goals for W0-B (defer):**

| Item | Owner wave |
|---|---|
| Five determinism gates + canary registry wiring into `oya-ci-required` | W0-C / G003 |
| Talos second-corpus proof | W0-D / G004 |
| Expanded trial measurements / ceilings | W0-E / G005 |
| Seam contracts + package→crate map | W0-F / G006 |
| Topology / Q7 / branch-protection readback | W0-G / G007 |
| Threat/fuzz/perf methodology | W0-H / G008 |
| Bulk Kubernetes corpus output under `k8s/` | W1+ (unapproved) |
| Owned Rust Go front end with authority | W2+ (model-equivalence gate) |
| Class-G auto-approval | blocked until ADR-0634 + W0-G |

---

## 2. Target crate inventory (faces fixed)

Intent Reconciliation Q2 re-homes §5.1 paths; faces and responsibilities are unchanged.

| # | Path | Package name (proposed) | Face | Responsibility |
|---|---|---|---|---|
| 1 | `build/port-engine/ports/port-engine-api` | `port-engine-api` | ports | Neutral seams: `SourceModel`, `RulePack`, `TransformPlan`, `TargetIr`, `Renderer`, `Receipt` (+ digests) |
| 2 | `build/port-engine/core/port-engine-kernel` | `port-engine-kernel` | core | Pure transform planning. **Build error** (not lint) on tokens `kube` / `k8s` / `kubernetes` / `apimachinery` |
| 3 | `build/port-engine/core/port-engine-rust-ir` | `port-engine-rust-ir` | core | Rust IR + deterministic renderer (`syn` 2 / `quote`); no clock / RNG / env / path leakage |
| 4 | `build/port-engine/adapters/port-engine-frontend-go` | `port-engine-frontend-go` | adapters | Consumes **SourceModel snapshot only**; never invokes Go in-process / in `verify()` |
| 5 | `build/port-engine/adapters/port-engine-source-pin` | `port-engine-source-pin` | adapters | Pin acquisition + license verification → canonical pin / snapshot digest binding |
| 6 | `build/port-engine/facade/port-engine-app` | `port-engine-app` | facade | Driver: `plan`, `render`, `verify`, `delta`, `receipt` |

**Root membership (single reviewed amendment):**

```toml
# ADR-0637 exception to ADR-0538 glob-only membership.
# Admits face/leaf crates under build/port-engine only; does not create a `build` capability.
"build/port-engine/*/*",
```

Rationale: four shape globs (`*/core/*` …) do not match `build/…`. Homing in `ci/*` was **rejected** by founder reconciliation. `build/` remains zero-capability build-meta (ADR-0637 D1).

**OWNERS (proposed):**

```
# build/port-engine/OWNERS
axis-cloud-platform
council-architecture
```

Per-crate OWNERS optional if root covers; born-accounting still needs catalog rows + ADR anchors per new crate (ADR-0555/0629).

**Capability accounting:** crates are **neutral build infrastructure**, not a `build` product capability and not Kubernetes facts. Catalog / reachability rows must state engine-neutral purpose; k8s corpus facts stay under `k8s` / `specs/k8s-port/**`.

---

## 3. Incremental slices (ordered)

Each slice is a potential PR or tightly stacked commits on one isolated worktree. Later slices may open parallel lanes only where ownership is non-overlapping (see §6).

### Slice 0 — Ready-gate confirmation (no product code)

**Owned paths:** `.grok/programs/k8s-port/**` only (already this plan).

**Acceptance:**

- [ ] All checks in `evidence/W0-B-ready-gate.json` evaluate `true` / present.
- [ ] Worktree branched from **post-G001** `origin/dev` tip (not W0-A feature branch alone).
- [ ] Beads issue for G002/W0-B created or claimed (suggest `bd create` after G001 close).

**Verify:**

```bash
python3 -c 'import json; g=json.load(open(".grok/programs/k8s-port/evidence/W0-B-ready-gate.json")); assert g["may_start_engine_code"] is True'
gh pr view 1561 --json state | grep MERGED
```

**Hard stop:** `may_start_engine_code != true` → stop.

---

### Slice 1 — Empty crate skeletons + members line + OWNERS (fail-closed)

**Owned paths:**

- `Cargo.toml` (members-line only; no opportunistic fmt/lock churn)
- `build/port-engine/OWNERS`
- `build/port-engine/{ports,core,adapters,facade}/port-engine-*/{Cargo.toml,src/lib.rs,BUCK?}`
- `build/port-engine/facade/port-engine-app/src/{lib.rs,main.rs}` (bin optional until driver slice)
- Crate-catalog / reachability / artifact born-accounting rows as required by gates
- Minimal fail-closed unit tests proving crates compile and **do not** claim readiness

**Scaffold rules:**

- `publish = false`, `license = "Apache-2.0"`, workspace edition/version/rust-version.
- Public API: module placeholders + `TODO`/`unimplemented!` only where tests expect fail-closed behavior.
- Kernel scaffold includes a **compile-fail or build-script / procedural check** hook stub for corpus-token ban (full ban lands Slice 2; Slice 1 may ship a test that documents the obligation and fails if tokens appear in kernel sources).
- No Go toolchain dependency in any crate `Cargo.toml`.
- No `k8s/` generated output.
- No `*.generated.json` hand edits; use sanctioned materializers only.

**Acceptance:**

- [ ] `cargo metadata` resolves all six packages via the new members line.
- [ ] `cargo check -p port-engine-api -p port-engine-kernel -p port-engine-rust-ir -p port-engine-frontend-go -p port-engine-source-pin -p port-engine-app` succeeds.
- [ ] Focused tests: at least one test per crate that **fails closed** if a readiness flag is falsely claimed (e.g. `engine_ready()` is `false` until later slices flip via real receipt schema).
- [ ] Born-accounting green for new crates (catalog + OWNERS + ADR anchor).
- [ ] No Kubernetes vocabulary in `port-engine-kernel` sources.

**Verify (local bridge; merge authority remains cloud `oya-ci-required`):**

```bash
cargo check -p port-engine-api -p port-engine-kernel -p port-engine-rust-ir \
  -p port-engine-frontend-go -p port-engine-source-pin -p port-engine-app
cargo test -p port-engine-api -p port-engine-kernel -p port-engine-rust-ir \
  -p port-engine-frontend-go -p port-engine-source-pin -p port-engine-app
# if Buck targets land in-slice:
# buck2 build //build/port-engine/...
git diff --check
# do NOT cargo fmt --all
```

**Hard stops:** members line elsewhere; engine under `ci/`; capability claim for `build`; broad workspace fmt; starting Slice 1 on pre-merge G001 tip.

---

### Slice 2 — Types + six-axis receipt schema (API + kernel seams)

**Owned paths:**

- `build/port-engine/ports/port-engine-api/**` — types for all seams + receipt
- `build/port-engine/core/port-engine-kernel/**` — planning types / pure functions over API types
- Receipt schema JSON (if registry-bound): prefer `specs/port-rules/receipt.schema.json` or crate-local schema with artifact-capability row
- Kernel corpus-token ban **enforced** (build error path demonstrated with planted canary source file outside production compile set, or compile-fail test)

**Acceptance:**

- [ ] Types exist for: `SourceModel`, `RulePack`, `TransformPlan`, `TargetIr`, `Renderer`, `Receipt`.
- [ ] `Receipt` **requires** all six axes; constructing with missing `snapshot_digest` is impossible or RED.
- [ ] `verify()` signature matches ADR-0638 D2 predicate shape (return byte-identical | diff).
- [ ] Kernel token ban demonstrated as **build error** (not clippy-only).
- [ ] Unit tests for digest field presence, stable ordering notes, and fail-closed incomplete receipts.

**Verify:**

```bash
cargo test -p port-engine-api -p port-engine-kernel
# document the exact command that fails closed on a planted kube token in kernel
```

**Hard stops:** Go toolchain in kernel; k8s tokens in kernel; receipt schema with fewer than six axes.

---

### Slice 3 — Source pin adapter + bootstrap extractor admission (out-of-band)

**Owned paths:**

- `build/port-engine/adapters/port-engine-source-pin/**`
- Extractor admission records under program evidence / licensing (extend `specs/k8s-port/licensing.json` bootstrap section state from `not_admitted` → admitted only with full control set)
- Out-of-band extractor tree (recommended: `build/port-engine/tools/bootstrap-go-extractor/` **or** external pinned artifact store referenced by digest — must never be invoked from `verify()`)
- Snapshot artifact store path (content-addressed; not hand-edited regenerable k8s Rust)

**Acceptance:**

- [ ] Pin loader binds `specs/k8s-port/upstream-pin.json` (`v1.36.1`, tag object `5b824a493a…`, peeled `756939600b…`).
- [ ] License verification fails closed on pin/license mismatch (Apache-2.0).
- [ ] Bootstrap extractor dependencies record: source, version, digest, license, SBOM, signature, provenance verification, sandbox policy (`licensing.json`).
- [ ] Two extractions of the same pin produce **byte-identical** SourceModel snapshots (pair committed or CI-reproducible with digests recorded).
- [ ] Producer identity recorded as `bootstrap-go-packages-go-types` per ADR-0638 D3.
- [ ] Front-end sizing note published (schema surface, extractor scope, owned-replacement estimate) as W0-B measurement — not a plan assertion.

**Verify:**

```bash
cargo test -p port-engine-source-pin
# extractor runs OUT OF BAND only; capture snapshot A/B digests
# cmp or sha256 equality of the two snapshot artifacts
```

**Hard stops:** calling Go from `port-engine-app verify()`; admitting extractor without full licensing control set; snapshot without producer identity.

---

### Slice 4 — Frontend-go consumer + snapshot firewall

**Owned paths:**

- `build/port-engine/adapters/port-engine-frontend-go/**`
- Shared snapshot decode in `port-engine-api` if needed

**Acceptance:**

- [ ] Adapter loads snapshot bytes → `SourceModel`; rejects malformed / wrong-digest inputs.
- [ ] Tests prove adapter has **no** `std::process` Go invocation path in library code used by verify.
- [ ] Package-to-producer mapping preserved from snapshot into model metadata.

**Verify:**

```bash
cargo test -p port-engine-frontend-go
# static grep / architecture test: no Command::new("go") in adapter lib used by verify path
```

---

### Slice 5 — Neutral rule pack v0 + selecting fixtures + rust-ir renderer skeleton

**Owned paths:**

- `specs/port-rules/index.json`
- `specs/port-rules/lang/go-rust/**`
- `specs/port-rules/idiom/**`
- Fixtures co-located or under `specs/port-rules/**/fixtures/**`
- `build/port-engine/core/port-engine-rust-ir/**` deterministic render path (minimal)
- Rule loader in kernel or api

**Rule schema (plan §5.3):** stable `id`, `version`, `precondition`, `captures`, `construction`, `precedence` + conflict behavior, `required_diagnostics`, `proof_obligations`, **≥1 selecting fixture**.

**Seed provenance:** rust-skills MIT only with `seed_source`, `seed_license`, `seed_commit` on every seed-derived rule.

**Acceptance:**

- [ ] Registry loads; rule without fixture **cannot load** (unit + integration).
- [ ] 100% of loaded rules have selecting fixtures.
- [ ] Neutral pack contains **zero** Kubernetes-specific rules (corpus rules stay in `specs/k8s-port/rules/**`, still optional/empty at W0-B).
- [ ] `rulepack_digest` = ordered hash of loaded rules; stable across runs.
- [ ] Renderer: stable ordering, normalized formatting; tests forbid clock/RNG/env/path leakage in render inputs.
- [ ] Operations journal entry for first rule-pack introduction (R-DOC).

**Verify:**

```bash
cargo test -p port-engine-kernel -p port-engine-rust-ir
# rule load test: strip fixture → load RED
# planted k8s token in a "neutral" rule path → classification RED
```

**Hard stops:** loading rules without fixtures; Kubernetes tokens in neutral pack; canary registry claims for W0-C gates (may stub index, but do not claim gate liveness).

**Note:** Full canary-region registry for the five determinism gates is **W0-C**. W0-B may add a **minimal** rule-mutation fixture needed to prove loader + fixture coupling only.

---

### Slice 6 — End-to-end six-axis receipts (facade driver)

**Owned paths:**

- `build/port-engine/facade/port-engine-app/**`
- Example/min fixture corpus (neutral, non-k8s) sufficient to plan→render→receipt
- Receipt emission path + golden receipt tests

**Pipeline (ADR/plan):**

```text
pin → [OOB] extractor → SourceModel snapshot (digested)
  → kernel plans under ordered rule pack
  → Rust IR → deterministic renderer
  → output tree + six-axis receipt
```

**Acceptance:**

- [ ] CLI or library entrypoints: `plan`, `render`, `verify`, `delta`, `receipt` (stubs OK only if each has fail-closed tests; prefer thin real wiring).
- [ ] End-to-end test binds all six axes; mutating any one axis yields `verify` RED / diff.
- [ ] Clean re-run with identical six axes → byte-identical receipt + output for the mini fixture.
- [ ] Engine builds under Cargo; Buck targets if repo policy requires new crates on Buck graph.
- [ ] Wave registry / operations journal updated for W0-B run (not `completed=true` until full G002 acceptance).
- [ ] No bulk `k8s/` corpus emission.

**Verify:**

```bash
cargo test -p port-engine-app
cargo run -p port-engine-app -- receipt --help   # or library tests only
# axis-mutation matrix tests: 6 axes × mismatch → RED
```

**G002 story acceptance (rollup):**

- Engine builds in Buck2 + cargo (as landed).
- Kernel token ban demonstrated as build error.
- 100% loaded rules carry selecting fixtures.
- Snapshot pair byte-identical.
- Front-end sizing published.
- Six-axis receipt schema implemented end-to-end.
- §D W0-B row obligations from approved plan satisfied.
- Protected PR path: independent review + `oya-ci-required` + squash merge + post-merge packet for G002.

---

## 4. Parallelization map

| Lane | After | Owns | Conflicts with |
|---|---|---|---|
| **P0** G001 closeout | now | PR #1561 review/merge, packet, beads | Nothing in W0-B code |
| **P1** Slice 1 scaffold | ready-gate true | `build/port-engine/**` + members line + catalog | Any other members-line editor |
| **P2a** Slice 2 types/receipt | P1 | api + kernel | P2b if shared API surface — prefer serialize |
| **P2b** Slice 3 pin/extractor | P1 | source-pin + OOB tool + licensing bootstrap fields | licensing.json single-writer |
| **P2c** Slice 5 rules (draft offline) | ready-gate true for *docs* only; **code load** needs P2a | `specs/port-rules/**` | none with P2b if pure data |
| **P3** Slice 4 frontend-go | P2a + P3 snapshot from P2b | frontend-go | — |
| **P4** Slice 6 e2e | P2a–P3 + P2c loadable rules | port-engine-app | integration order last |

**Concurrency caps (ADR-0637 D3):** up to four isolated worktrees / sixteen agents only with owned-runner capacity evidence. Default: **one W0-B worktree** until Slice 1 lands; then split P2a / P2b / rules-data carefully.

**Integration order:** P1 → (P2a ∥ P2b ∥ rules-data) → P3 → P4.

---

## 5. Worktree / PR procedure (post-G001)

```bash
git fetch origin dev
# confirm origin/dev contains W0-A (ADR-0637/0638, specs/k8s-port, R-DOC)
git worktree add /Users/jasonlee/Developer/oyatie/.worktrees/k8s-port-w0b \
  -b agent/k8s-port-w0b-$(date +%Y%m%d) origin/dev
cd /Users/jasonlee/Developer/oyatie/.worktrees/k8s-port-w0b
# implement Slice 1 only first; SSH-sign; push; PR → dev
```

**Merge path (unchanged):** isolated worktree → SSH-signed push → PR against `dev` → independent APPROVE + threads resolved → no conflict → branch protection → singleton `oya-ci-required` green → squash merge → post-merge packet → beads/ultragoal checkpoint.

**Review model for rules (later slices):** one implementer, two split-context adversarial reviewers, one fixer (ADR-0637 D3). Review object = rules + fixtures, not generated output.

---

## 6. Verify command catalog (cumulative)

| Concern | Command / check |
|---|---|
| Ready-gate | inspect `evidence/W0-B-ready-gate.json` + `gh pr view 1561` |
| Members resolution | `cargo metadata -q` / `cargo check -p port-engine-*` |
| Unit tests | `cargo test -p port-engine-{api,kernel,rust-ir,frontend-go,source-pin,app}` |
| Kernel token ban | planted-token build-error demonstration (document exact recipe in ops journal) |
| Snapshot pair | two OOB extractions; `sha256` equality |
| Rule fixtures | load tests; fixture-strip RED |
| Six-axis | axis-mutation matrix in `port-engine-app` tests |
| R-DOC | `cargo test -p ci-k8s-program-docs` + live gate if program docs touched |
| Canonical JSON | existing `//ci/facade/canonical-json` for new specs |
| Born-accounting | crate-catalog-coverage / reachability as required |
| Diff hygiene | `git diff --check`; package-scoped `cargo fmt -p … -- --check` only |
| Merge authority | cloud `oya-ci-required` only |

---

## 7. Hard stops (always)

1. **No W0-B product code** until G001 promoted (merge + post-merge packet + beads close + durable G001 checkpoint).
2. **No** engine home under `ci/*` to avoid members-line edit.
3. **No** Go toolchain inside `verify()` / producer path.
4. **No** hand-edit regenerable port output (none should exist yet; keep it that way).
5. **No** hand-edit `*.generated.json`.
6. **No** Kubernetes tokens in `port-engine-kernel` or neutral rule pack.
7. **No** W1+ / bulk corpus / class-G auto-approval claims.
8. **No** self-approve / agent merge of protected PRs.
9. **No** gjc / omc / omx / hermes orchestration.
10. **No** broad `cargo fmt --all` absorbing unrelated drift.
11. **No** second root-membership glob beyond the single reviewed `build/port-engine/*/*` line without a new ADR.
12. Canary **denominator** exclusions (CRIT-P5-01) apply when canaries appear; do not count canaries as ported coverage.

---

## 8. Evidence & tracking artifacts

| Artifact | Role |
|---|---|
| This file | Human admission plan |
| `evidence/W0-B-ready-gate.json` | Machine checklist; `may_start_engine_code` |
| `evidence/G001-post-merge-packet-DRAFT.md` | G001 closeout template |
| `PROGRAM.json` | Session program state (`next_story`, hard stops) |
| `docs/programs/k8s-port/operations/*` | R-DOC operations journal for W0-B runs (in-repo, post-start) |
| `docs/programs/k8s-port/wave-registry.rdoc` | W0-B completion flag only after G002 acceptance |
| Beads | G001 `oyatie-7xf`; open G002 issue after G001 close |
| Ultragoal goals/ledger | Provenance only under `.gjc/`; do not drive with gjc CLI |

---

## 9. First slice after ready-gate (explicit)

When — and only when — `may_start_engine_code` is true:

1. Create worktree `k8s-port-w0b` from `origin/dev`.
2. Land **Slice 1 only**: empty six crate skeletons + `"build/port-engine/*/*"` members line + OWNERS + fail-closed readiness tests + born-accounting.
3. Open protected PR; do not stack full rule pack / extractor in the first PR unless capacity and review bandwidth are proven.

Until then: keep G001 lifecycle moving; treat this plan as the parallelizable admission artifact.

---

## 10. Traceability

| Obligation | Plan / ADR | W0-B artifact |
|---|---|---|
| Engine home + members amendment | ADR-0637 D1; Q2 reconciliation | `build/port-engine/*/*` + root `Cargo.toml` |
| Neutral vs corpus policy split | ADR-0637 D1; plan §5.1 | crates vs `specs/port-rules/**` vs `specs/k8s-port/**` |
| Six-axis verify | ADR-0638 D2; plan §5.5 | `Receipt` + `port-engine-app` |
| Snapshot firewall | ADR-0638 D3; plan §5.10 | OOB extractor + frontend-go |
| Bootstrap divergence expiry | DVG-BOOTSTRAP-GO-FRONTEND | licensing + sizing + producer identity |
| Fixture-gated rules | plan §5.3; §D W0-B | `specs/port-rules/index.json` + fixtures |
| Mechanical maintenance | ADR-0637 D2; ADR-0638 D5 | delta/receipt driver (no output patching) |
| Record lanes | ADR-0637 D5 | operations journal entry per W0-B run |
