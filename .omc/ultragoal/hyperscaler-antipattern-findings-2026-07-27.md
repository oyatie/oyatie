# Hyperscaler anti-pattern sweep — verified findings (2026-07-27)

29 agents · 2,080,872 subagent tokens · 824 tool calls · every material finding
adversarially re-derived by an independent verifier.

## THE META-FINDING: the linters exist and are disconnected

| Fact | Value |
|---|---|
| `oya-check-*` crates in `libs/` | **72** (30,765 src lines) |
| …reachable from CI (a `ci/facade/` consumer) | **4** — adr-index, brand-residue, license-policy, slo-coverage |
| …whose SOLE dependent is `marketplace/facade/dev-cli` | **64** |
| …with ZERO dependents at all | **4** |
| Repo-wide crates whose only dependent is dev-cli | **74** (42,373 lines) |

`dev-cli` is retirement-marked and carries no merge authority. So ~30k lines of
owned-Rust linting is written, tested, and unreachable. **The answer to "should a
linter catch this" is almost always "one already does, off-line."**

The fix is not writing more checks. It is (a) re-homing the check kernels as
**library predicates** the cloud-ci gate engine invokes, and (b) a **coverage gate**
that REDs when a check kernel has no CI-reachable consumer — otherwise this silently
re-rots.

## CONFIRMED FINDINGS

### 1. Six-way duplicated port trait — HIGH (downgraded from critical)
`ProviderAuthPort`, `AuthToken`, `AuthError`, `AuthMode` each defined **6×**, all at
identical line numbers, in `oya/intelligence/crates/oya-intelligence-adapter-*-kernel`.
All six `lib.rs` are exactly 169 lines; anthropic-vs-openai differ by 6 lines.

- **ADR-0020 forbids this**: it specifies ONE crate `oya-intelligence-adapter-kernel`
  with `fn auth_mode(&self) -> ProviderAuthMode` — the parameterised shape. That crate
  **does not exist**. This is an ADR violation, not a sanctioned choice.
- **Blast radius zero**: the 12 crates are a closed island — no consumer anywhere.
- ⚠ **The obvious remedy is WRONG.** "Delete all 12" would destroy **3,549 LOC of live
  implementation**: `anthropic-subscription-adapter` is 2,315 LOC of real OAuth
  (singleflight refresh, token state machine, 463-line integration test);
  `openai-subscription-adapter` is 1,234 LOC of live key-pooling. Only **4 of 6**
  adapters are the 133-line mocks. Correct remedy: delete 5 kernels + 4 mock adapters
  (1,377 LOC), hoist ONE port definition, repoint the 2 live adapters.
- `AuthMode` has **no canonical home** — it is itself one of the six-way duplicates,
  so there is nothing existing to parameterise against.
- Must reconcile with `oya-intelligence-adapter-domain` which already defines the
  ADR-0020 `ProviderAdapter` trait AND is actually consumed.

### 2. Dead crates — 276 of 926 (29.8%) — HIGH
Zero in-repo consumers, no declared entrypoint. **263,280 of 996,006 src lines (26.4%).**
Claimed 294/291,857 was inflated; 17 are declared buck-label entrypoints in CI policy.

### 3. `os/` is entirely product-unreachable — HIGH
Finding claimed 26 crates / 54,818 lines. Verifier: **understated — all 41 crates /
164,927 lines**. Nothing in `os/` composes into a product path. No ADR sanctions this.

### 4. `libs/` orphan families — MEDIUM
60 of 113 non-check `libs/` crates orphaned (53%): `oya-governance-*` **28 of 39** (72%)
zero-dependent, `oya-shared-*` **24 of 51** (47%). The 72 `oya-check-*` are a separate
defect (see meta-finding) — 63 have exactly one consumer.

### 5. Doc sprawl: unwritten-code placeholders — HIGH
**506 files / 84,697 lines** carry `rust_code_status: not-authored-in-this-wave`.
Families: `hot-split.md` 82, `cold-merge.md` 82, `auto-rebalance.md` 82, `dpia.md` 81,
`IP-WAVE-15-ZD-sharding-automation.md` 80, `IP-ADR-0339-Shared-IaC-Modules.md` 80
(= 487 of 506).

### 6. Superseded ADR-0349 propagated into 776 files — HIGH
**384 `.md` + 269 non-md** (200 yaml, 39 json, 27 rs, 3 toml) reference five ADR-0349
enforcement lanes. **ZERO are implemented.** Plus 356 mangled-rename `.md`.

### 7. Helm chart redundancy — HIGH
**85 charts / 543 files / 306 byte-identical template files / 61 unresolved
`cargoPackage`** — and **42 of the 61 never existed as a package in any commit**. Those
are birth-defective charts, not reorg orphans. Clone families: `service.yaml` 81,
`configmap.yaml` 81, `Chart.yaml` 80, `deployment.yaml` 75, `cedar.yaml` 67.
Total ~688 redundant files in 74 clone groups (~946 in 87 under a fuller normalizer).

### 8. ADR hygiene — MEDIUM
**437 ADRs** (not 439). **67 carry lowercase status spellings** (`accepted` 34,
`proposed` 31, `superseded` 1, `deprecated` 1) that the owned Rust `VALID_STATUSES:
[&str; 6]` in `libs/oya-governance-adr-shape-kernel` rejects — a linter that exists and
does not run. 11 distinct status values; 0 ADRs with no status.

### 9. Charset / filename violations — LOW but trivially fixable
**6 tracked paths** (not 5) contain non-ASCII or shell-hostile characters:
`ADR-SDK-0003-…-tenancy-µservice's-sandbox-.md` (non-ASCII µ + apostrophe + trailing
dash), `ADR-PAS-0007-…-µservice.md`, +4.

### 10. Phantom canonical-verifier citation — MEDIUM
**521 files** assert `./bin/oya verify --ci-required` is "the canonical local pre-push
verifier" without ADR-0346's "provenance only, never merge authority" qualifier —
and **`bin/` does not exist in the repo**.

## REFUTED / MATERIALLY CORRECTED (do not act on these)

| Claim | Verdict |
|---|---|
| Clean-arch layering violations (core→adapters etc.) | **REFUTED** — sanctioned by ADR-0562 §10.9/§10.11 + ADR-0570 + ADR-0627. Severity low. |
| `libs/` → capability-root layer inversion | **REFUTED** — the 30 edges point at `oya/`, the LEGACY root, not capability roots |
| ADR-MS-001 ×8 = duplicate IDs | **REFUTED** — explicitly sanctioned per-service ADR namespacing |
| `cloud/` is "one directory from deletion" | **REFUTED** — 1,307 files across the other 20 dirs |
| 67 zero-crate dirs are post-move shells | **CORRECTED** — only 43 are; 24 never held a `Cargo.toml` |
| 12 twin roots / 1,158 files | **CORRECTED** — 1,209 files; `slos/` already co-moved to 18 capability roots |

## THE LINTER PROGRAM (productize, don't hand-fix)

Each confirmed class → a gate. Ordered by leverage:

1. **check-kernel CI-reachability gate** — RED when an `oya-check-*` kernel has no
   `ci/facade/` consumer. Unlocks the 68 disconnected linters. *Highest leverage: it is
   the gate that makes the other 71 gates real.*
2. **Duplicate-symbol gate** — a `pub trait`/`pub struct` defined N× across sibling
   crates with near-identical bodies. Would have caught #1 at authoring time.
3. **Dead-crate gate** — zero in-repo consumers ∧ no declared entrypoint (#2, #3, #4).
   Needs the entrypoint allowlist the verifier identified (17 buck-label entries).
4. **Per-root scan completeness gate** — every top-level root either enumerated or
   explicitly excluded with a reason. Closes the 12 unscanned roots.
5. **No-new-CLI gate** — forbid `clap`/`structopt`/`argh` as a prod-crate dep; detect
   arg-parsing `fn main` + `[[bin]]` in non-test prod surfaces. Currently ZERO detection;
   `clap` is in 26 prod crates including the gate fleet itself.
6. **Generated-artifact clone gate** — byte-identical template detection for Helm/YAML
   (#7); chart `cargoPackage` must resolve to a live crate.
7. **ADR status normalization** — wire the EXISTING `VALID_STATUSES` kernel into CI (#8).
8. **Charset gate** — tracked paths match `[A-Za-z0-9._/-]+` (#9). Ten-line check.
9. **Superseded-citation gate** — `cross-artifact-agreement` already ships
   `phantom_decision_citation`; extend to superseded-ADR propagation (#6, #10).

Note 7, 8 and 9 are cases where the owned Rust check **already exists** and simply is not
invoked — consistent with the meta-finding.
