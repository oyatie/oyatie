# #117 — Clean-arch port-placement gate: forbid storage-port traits defined in adapter crates — 2026-06-22 (dev 2bc9520a5)

Productizes the defect class #116 exemplified (billing `AccountingJournalStoragePort` was defined in an adapter crate). Founder doctrine: "impossible to ship anti-patterns through enforcement"; "construction > reaction"; ports belong in core (proven: tenancy `TenantLifecycleStore`, SCIM `UserStore`/`GroupStore` both in core). No existing gate catches "port trait DEFINED in an */adapters/* crate" (face-direction/tier-acyclicity check dep EDGES; kernel-purity checks *-kernel containment) → NEW gate, mirroring kernel-purity's STRUCTURE.

## Template to mirror EXACTLY
`cloud/cloud-ci/gates/oya-cloud-ci-kernel-purity-app/` — read its: `src/lib.rs` (policy-driven scan + baseline + enforce-no-regression), `kernel-purity-policy.json` (policy-as-data), the frozen baseline file, BUCK, Cargo.toml, the gate-registration meta-test hook (firewall `gate_registration.rs`), and how it's wired into `.github/workflows/oya-ci-required.yml` + the gate catalog. Replicate that shape for the new gate.

## The gate (new crate `cloud/cloud-ci/gates/oya-cloud-ci-port-placement-app` or similar canonical name)
- **Predicate (HERMETIC, pure Rust, no shell/net/clock/rand):** scan repo crates; flag a `pub trait <Name>` whose DEFINING crate path contains `/adapters/` AND whose name matches a storage/repository/port heuristic (suffixes: `Store`, `Repository`, `StoragePort`, `Port` — refine by scanning what actually exists so the heuristic is sound, not noisy). The rule: such a port trait belongs in a `core`/`ports`/`kernel` crate, not an adapter. Reuse the shared BUCK/AST/text parsing kernel if one exists (the repo consolidated BUCK/AST parsing into a shared kernel — check libs/ for it; do NOT hand-roll a parser if a shared one exists).
- **UNIVERSAL (policy-as-data):** the suffixes + the layer-dir names (`adapters` forbidden; `core`/`ports`/`kernel` allowed) + any allowlist live in a `*-policy.json`, NOT hardcoded — so the gate is a neutral engine runnable on any repo.
- **born-advisory + enforce-no-regression:** a frozen baseline of EXISTING violations (after #116, billing is clean — scan the whole repo and baseline whatever remains; do NOT assume zero). New violations vs the frozen baseline → RED. Existing baselined ones → allowed (ratchet-down only). Include an allowlist mechanism for genuine false positives (an adapter-internal trait legitimately named *Store).
- **AUTOMATED:** emit a precise remediation per violation (which trait in which adapter crate → move to which core/ports crate). Full auto-move = a noted follow-up, not this slice (a trait-relocation codemod is non-trivial); flag-with-precise-fix is acceptable for v1 but SAY SO.
- **gate-registration:** wire into the firewall gate-registration meta-test (the gate must be registered or the meta-test fails) + the gate catalog + `.github/workflows/oya-ci-required.yml` (born-blocking, like the other §2.5 gates). Mirror exactly how kernel-purity is registered.

## born-accounting (NEW gate crate → full accounting)
DOGFOOD `register_crate` (the #105 scaffold: `cloud/cloud-ci/gates/oya-cloud-ci-register-crate-app`) to onboard the new gate crate end-to-end (OWNERS + ADR governed-surfaces + capability mapping + catalog + workspace glob + reachability + faces settle). This both saves the ~4-round born-accounting grind AND validates register_crate on a real new crate (founder loves the dogfooding). If register_crate can't handle some piece, do it manually + note the gap (feeds #106/#107). Reference an EXISTING layering ADR (ADR-0280 DAG / ADR-0245 tiers / the ports-adapters doctrine ADR) for the governed-surfaces justification — do NOT mint a new ADR unless required; if the gate needs a policy/decision record, a short ADR is acceptable (mirror how kernel-purity's ADR-0547 documents its gate).

## Tests (RED/GREEN, DB-free)
- Pure-predicate unit tests: a trait named `FooStore` in a `*/adapters/*` crate → flagged; the SAME trait in a `*/core/*` or `*/ports/*` crate → clean; a non-port-named trait in an adapter → clean; an allowlisted one → clean; baseline-frozen existing → not-RED; a NEW one beyond baseline → RED.
- The gate-registration meta-test passes (gate is registered).

## Done-bar
- buck2 build + test the gate + the affected cone. The gate, run on the CURRENT corpus, is GREEN (billing fixed by #116; everything else baselined). 
- Regen Cargo.lock + materialize faces + firewall GO-LIVE + freshness + affected-set + target-parity + the new gate's own lane + face-settle --verify. git status clean.
- 7-property check (memory pipeline-four-property-bar): UNIVERSAL (policy-as-data) + PRODUCTIZED (engine+policy+baseline+catalog) + HERMETIC (pure Rust, no shell/net) + AUTOMATED (precise remediation; full codemod = noted follow-up) + CLOUD-NATIVE/API (it's a CI gate, fine) + RIGHT-TOOL + LATEST. Self-audit against this before done.
- Fresh worktree off origin/dev; never touch canonical checkout; buck2 for build/test; trailer EXACTLY `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`. After green: STOP, do NOT self-approve — orchestrator runs adversarial review (verify: predicate sound + not noisy, baseline captures all existing, block-new actually fails on a planted violation, born-accounting complete, gate-registration wired).

## Scope discipline
This is ONE gate. Do NOT also build the full auto-move codemod (follow-up). Do NOT retrofit other capabilities' ports (the gate baselines them; their migration is separate). Keep it to: the gate + its baseline + registration + born-accounting.
