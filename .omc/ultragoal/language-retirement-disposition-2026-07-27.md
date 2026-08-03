# Language retirement — disposition ledger (2026-07-27)

**Founder directives this encodes:**
- *"all js (unless there is a good reason to keep it), all python, typescript, sveltekit, shellscript are retirement targets"*
- *"we don't want anything cli shaped. genuinely api based cloud native"*
- *"reorg also isn't a simple move … may require refactor, rewrite, delete, or additional work with or without codemod"*

## Census

46 tracked files in retirement-target languages at session start → **38 remain** after commit `fb5bc3176`.

| Ext | Count (start) | Now |
|---|---|---|
| `.sh` | 19 | 19 |
| `.py` | 13 | 12 |
| `.js` | 7 | 7 |
| `.ts` | 3 | 0 |
| `.tsx` | 1 | 0 |
| `.svelte` | 3 | 0 |

## LANDED — commit `fb5bc3176`

Deleted 8 files (896 lines), added `oya` to `rust-first-automation-policy.json` scan roots.
Gate verified: `//ci/facade/automation-language-policy:...-gate` **45 passed, 0 failed**.

Resolution note: the non-Rust exception baseline is **shrink-only against an immutable
merge-base ceiling**, so a newly-surfaced file CANNOT be admitted by adding an allowlist
row. Deletion was the only green path — and the correct one.

## THE FOUR LEGITIMATE DESTINATIONS (non-CLI)

A `.sh` → Rust-**binary** conversion is WRONG. It trades shell-CLI for Rust-CLI; both
violate all-CLI-retirement. Every MIGRATE resolves to one of:

1. **Library predicate** consumed by the cloud-ci gate engine
2. **Reconciler / operator + CRD** (declarative state, GitOps)
3. **buck2 `rust_test` target** as merge authority
4. **Corpus L4 `Inv` + `GateDecl` nodes**

In-repo template: `ci/facade/layer-dependency-acyclicity/BUCK` — `rust_library` kernel +
`rust_test` as MERGE AUTHORITY, with the `rust_binary` explicitly commented
*"LOCAL BRIDGE feedback only … never merge authority."*

## DELETE (11) — proven dead

| File | Evidence |
|---|---|
| `docs/audit/initial-sweep-2026-06-06/**` (7 `.js`) | Hardcode `/Users/jasonlee/Developer/linux/…` (absent). No `package.json`/lockfile tracked anywhere; no node/npm/bun in Makefile or CI. Outputs already committed as `.md` siblings. Origin `c0bc8984c` (PR #635), never modified. **⚠ GOVERNANCE FLAG: `docs/audit/` is a declared provenance-archive class (`oya-ci.toml path_excludes`, FRIC-010). Removal needs a deliberate archive-scope ruling, not a mechanical sweep.** |
| `tools/anchor-sweep/inject_anchors.py` (732 ln) | Hardcodes `/Users/jasonlee/oyatie/microservices` (absent); `microservices/` no longer exists. One-shot Wave-3-C sweep ALREADY executed — injected anchors visible in `cloud/cloud-k8s/ARCHITECTURE.md:3`. Zero execution sites. |
| `tools/hooks/stale-tool-suggester.sh` | Advisory-echoes the same surface `local-authority-enforcer.sh` already **hard-blocks (exit 2) on the same matcher**. Zero enforcement value. |
| `tools/hooks/adr-orphan-detect.sh` | Superseded by born-blocking Rust `phantom_decision_citation` in `ci/facade/cross-artifact-agreement` (`src/lib.rs:485`). |
| `tools/hooks/stop-did-you-forget-suggester.sh` | 3 checks: (1) a nag with no verdict, (2) same check as `phantom_decision_citation`, (3) substring grep for "fail" in a YAML — a false-positive generator. |
| `tools/hooks/pre-dispatch-guide.sh` | No policy content. |

**Coupled-edit requirement:** each hook deletion is atomic across **3 files** — the `.sh`
plus its rows in BOTH `.claude/settings.json` AND `.codex/hooks.json`. Otherwise
`ci/facade/hook-wiring` REDs `wired_hook_missing_file` (`src/lib.rs:190-206`). Also drop
the matching row from `rust-first-automation-policy.json` `exceptions[]` **and** the
shrink-only baseline, or `..._baseline_stale` fires.

## MIGRATE (9) — destinations CORRECTED to non-CLI

| File | Workflow proposed | Corrected |
|---|---|---|
| `infra/ci/install-buck2.sh` (190 ln, 13 CI call sites) | Rust binary ❌ | ~4-line fetch residue, or **zero** via digest-pinned runner image |
| `tools/hooks/no-cargo-enforcer.sh` | new `rust_binary` ❌ | library predicate + gate test; patterns as policy-as-data |
| `tools/hooks/local-authority-enforcer.sh` | same new binary ❌ | collapse with `stale-tool-suggester` into ONE retired-authority predicate |
| `tools/hooks/main-checkout-guard.sh` | — | already correct shape; blocker is binary provisioning into gitignored `tools/hooks/bin/` |
| `scripts/ci/regen-third-party.sh` | CLI subcommand ❌ | reindeer/buckify glue; `//tools:buckify` not shipped |
| `infra/seaweedfs/tests/*` | fold into `operator-secret-rbac` ✅ | correct — encodes live security invariants (no inline S3 creds, PodSecurity, NetworkPolicy:8333); no Rust equivalent exists |
| `specs/fixtures/calendar-prd/calendar_prd_replay_check.py` | contract-slice ✅ | correct — 16 fixture JSONs migrate unchanged as DATA |
| `cloud/cloud-k8s/tests/test_runtime_substrate_validation.py` | contract-slice ✅ | correct — 6 tests pass live; policy JSON already adjudicated it migrate-not-keep |

## KEEP: **zero**

The single KEEP claim (`install-buck2.sh`, bootstrap) was **refuted**. GitHub Actions
`run:` and the repo's own `Command::new(path)` test both accept ANY executable — no
shell-only interface exists. The policy row cited as blessing it reads
`status: temporary_legacy_bridge` with a replacement clause. ADR-0523's authorized shape
is a ≤3-line exec shim; the file is 190 lines of pure policy. `reqwest`+`sha2` already
workspace-pinned. Precedent: `buck2-affected-gate.sh` already retired to Rust.

## CLASS DEFECTS FOUND (productize, don't hand-fix)

1. **12 top-level roots unscanned.** `rust-first-automation-policy.json` enumerates 34
   roots and omits `oya`, `tasks`, `contracts`, `docs`, `evidence`, `packs`, `plan`,
   `registry`, `specs`, `templates`, `benchmarks`, `toolchains`. Confirmed live:
   `specs/fixtures/calendar-prd/*.py` escapes the ratchet entirely. **Fix = per-root
   completeness gate**, not another hand-added root.

2. **The no-CLI loophole is open.** `rust-first-automation-hygiene` checks file extensions
   and inline shell only — zero clap/arg-parsing detection. A `.sh` retired into a Rust CLI
   passes clean. `clap` is a workspace dep in **26 production crates**, and the cloud-ci
   gate fleet itself ships as clap binaries — the largest instance of the anti-pattern.

3. **Vacuous gates.** `libs/oya-check-i18n-coverage` reads `clients/i18n/source.ftl` but
   the only `.ftl` lives at `clients/web-sveltekit/packages/i18n-source/` — path mismatch,
   and its only caller is the retired `dev-cli`. `libs/oya-check-client-stack-discipline`
   has zero callers outside its own tests.

## DEFERRED — needs a governance ruling

- **3 `.ftl` files** at `oya/workflow-studio/clients/web-sveltekit/packages/i18n-source/`
  are the repo's ONLY i18n corpus, governed by ADR-0206 (Fluent/ICU). They need a
  DESTINATION, not a deletion. Classic reorg-is-not-a-move case.
- **ADR-0185 + ADR-0204 still Accepted** while mandating SvelteKit as "the sole web stack,
  Phase 1 active". ADR-0393 (founder-confirmed 2026-06-01, all-Leptos) superseded ADR-0372
  but never propagated. Contradiction PRE-DATES this work. `cross-artifact-agreement`
  enforces ADR index-projection parity, so this is its own governed edit.
- **`docs/audit/` provenance-archive scope** — see DELETE table flag.
