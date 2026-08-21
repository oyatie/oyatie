---
doc_class: Specification
shape: Specification
length_cap: 400
microservice: policy
related_adrs:
  - ADR-0615
  - ADR-0711
  - ADR-0717
inbound_citations:
  - policy/README.md
---

# Promotion — what this capability still owes, and what blocks it

This capability was delivered under a hard scope bound: **`policy/**` only**, which is also its
ADR-0711 D-9 envelope (`specs/integ-branch-envelopes.json#roots.policy` → `envelope_globs:
["policy/**"]`). Every edit below is outside that envelope. The envelope law is fail-closed —
*"a unit may touch paths outside envelope(R) only when an explicit adjunct claim is recorded for this
wave. Claims are fail-closed: absent claim = refuse"* — and no adjunct claim exists for `integ/policy`.
So these are **not** work that was skipped; they are work this branch may not perform.

Two of them (§1) fire on the mere existence of the `policy/` directory, because
`ci/adapters/scan-root-derivation` resolves a root by `repo_root.join(name).is_dir()`. They are
mechanically forced by landing *any* file here — including this one.

## §1 Forced by the directory existing — 5 edits

Whoever lands `policy/` owes these in the same change. The gate policies say so themselves:
*"Sibling roots base/ and policy/ remain absent and remain declared; whoever lands their first crate
owes the same one-line retirement."*

| File | Edit |
|---|---|
| `ci/facade/scan-root-liveness/scan-root-liveness-policy.json` | delete `"ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json::/scan_roots::policy"` from `forward_declarations` |
| `ci/facade/scan-root-liveness/scan-root-liveness-policy.json` | delete `"ci/facade/module-membership/capability-membership-policy.json::/scan_roots::policy"` from `forward_declarations` |
| `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json` | remove `"policy"` from `_provenance.pending_roots` (leaving `["base"]`) |
| `ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json` | remove `"policy"` from `_pending_scan_roots` (leaving `["base"]`) |
| `governance/capability-registry.json` (**HUB**) | set `capabilities[policy].absorbs_current_dirs` to `["policy"]` |

Verified present on `origin/dev@7f8a5a075`. Both `pending_roots` sets are frozen **two-sided** by
tests named `pending_roots_equal_the_frozen_set_exactly`, and `forward_declarations_are_all_still_absent`
panics rather than reporting — so these are hard failures, not advisories.

The registry row is the one that was NOT predicted. It was filed under §2 as a crates-only blocker
and is in fact forced by the directory existing, because `walked_roots_are_exactly_the_registry_derived_set`
compares the registry's materialization record against the tree:

> capability `policy`: the registry's materialization record and the tree disagree — either the
> registry row is stale or the directory was deleted without retiring it

`governance/capability-registry.json` is a **hub** (`specs/integ-branch-envelopes.json#hubs`,
`sole_owner_per_wave: true`), so it needs a waiver row, not merely an adjunct claim.

### Measured, not predicted

Both locally (`cargo test -p ...`, after `oya-cloud-ci-materialize-generated-faces`) and in CI on
this head, the failure set is exactly **6 tests across 3 crates**:

| crate | failing tests |
|---|---|
| `ci-caller-supplied-authorization` | `pending_roots_equal_the_frozen_set_exactly`, `walked_roots_are_exactly_the_registry_derived_set` |
| `ci-embedded-asset-hermeticity` | `pending_roots_equal_the_frozen_set_exactly`, `walked_roots_are_exactly_the_registry_derived_set` |
| `ci-scan-root-liveness` | `forward_declarations_are_all_still_absent`, `live_corpus_is_green_against_the_frozen_policy` |

`ci-repo-root-hygiene` and `ci-module-membership` **pass** — the corpus budget and top-level-dir
membership are satisfied, which is the design of §3 working. A local `ci-slo-coverage` failure
(`run producer binary: NotFound`) is a local-environment artifact, not a finding: CI builds that
producer binary and `ci-slo-coverage` passes there.

Already satisfied, needing no edit: `policy` is present in module-membership `allowed_top_level_dirs`
and `scan_roots`, and in repo-root-hygiene `allowed_root_dirs`. Ownership resolves through
`policy/OWNERS`, and reachability through the envelope prefix (`envelope_glob_to_prefix` maps
`policy/**` → `policy/`), so total-accounting's `unowned`/`unreachable`/`unjustified` are all clear
with no registry edit.

## §2 Blocked: no Rust crate can land in this envelope

The capability's C0 face — the port specified in `CONTRACT.md` — needs a crate. Five independent
blockers stop one landing from `integ/policy`, and **none is inside `policy/**`**:

1. **`Cargo.lock` is a hub** owned solely by `integ/build`
   (`specs/integ-branch-envelopes.json#hubs`, and `adjunct_claims.rules`: *"Cargo.lock edits are owned
   solely by integ/build … other integs MUST use an unexpired hubs.active_waivers row"*). There is no
   waiver for `integ/policy`. Any new workspace member writes the lock, and CI runs `--locked`.
2. **`governance/capability-registry.json` is a hub.** Its `absorbs_current_dirs` edit is already
   forced by §1 above. It is *additionally* required for crates: `module-membership` resolves a
   crate's home only from that mapping — there is no implicit "directory name == capability" rule —
   so every `policy/core/*` crate is a `MEM-NEW-UNMAPPED-CRATE` regression until `"policy"` is there.
3. **`registry/catalog/policy-<leaf>.yaml`, one per crate.** `crate-catalog-coverage` blocks every new
   uncatalogued crate. The row is a `.yaml`, which §3 independently forbids.
4. **`ci/facade/slo-coverage/tests/slo_coverage.rs:163`, `SLO_CATALOG_CENSUS = 743`** — an equality pin
   against a live `registry/catalog/*.yaml` count of exactly 743. N new rows require re-freezing it to
   `743 + N` in the same PR.
5. **`ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json`** — needs `policy`
   in `capability_roots` and `policy/*/*` in `crate_root_globs`; the former is also cascaded by
   `cross-artifact-agreement`'s `registry_derived_policy_desync` once §2.2 lands.

Working in our favour and needing no edit: the root `Cargo.toml` `members` globs (`*/core/*`,
`*/ports/*`, `*/adapters/*`, `*/facade/*`) admit `policy/{core,ports,adapters,facade}/<leaf>` by
construction (ADR-0538).

## §3 Blocked: no `.json` or `.yaml` file can land anywhere

ADR-0717's corpus-budget ratchet freezes nine classes shrink-only, and **two of them are repo-wide**:
`json_files` and `yaml_files` use the empty prefix, so they count every tracked file with that suffix
anywhere in the tree. Measured on `origin/dev@7f8a5a075`:

| class | ceiling | live | headroom |
|---|---|---|---|
| `json_files` | 1287 | 1287 | **0** |
| `yaml_files` | 3025 | 3025 | **0** |
| (the other seven classes) | — | — | **0** |

Every class sits exactly at its ceiling. `evaluate_corpus_budget` is `observed > frozen` → born-blocking.
The two remedies — one-in-one-out, or a `reviewed_raises` entry in
`ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json` — are both outside `policy/**`, and
each class permits only **one** sanctioned raise whose `from`/`to` must match exactly or it fails closed.

This is why the capability ships **no `manifest.json`, no `capabilities/*.yaml`, no
`observability/slos/*.openslo.yaml`, no `dashboards/*.json`, and no `scorecards/overrides.json`**,
despite all five being present in every sibling capability root. `.md`, `.cedar`, `.cedarschema`,
`.toml`, `BUCK` and `OWNERS` are counted by no class, which is exactly what this capability is built from.

Note on the OpenSLO gap specifically: doctrine (ADR-0706, `CLAUDE.md#observability_substrate`) places
per-capability OpenSLO at `<capability>/observability/slos/`, but no gate compels it for `policy` —
`policy` is absent from `governed_service_roots` in
`ci/facade/service-tier-metadata/tier-field-coverage-policy.json`. The *enforced* SLO declaration is
the `slo:` scalar on the catalog row, which is §2.3. So the missing SLO is a doctrinal debt with a
named unblock, not a silently dropped obligation.

## §4 Not attempted: the tier ruling

`governance/capability-registry.json` deliberately leaves `policy` with **no declared `tier` or
`substrate_dag_position.stratum`**; the registry states an undeclared capability is non-baselineable
and needs a founder/architecture ruling. Nothing here invents one. `specs/substrate-dependency-dag.json`
already declares both faces (`policy.authoring.cp` plane G, `policy.local-pdp` plane C0), so the DAG
needs no edit either way.

## §5 Finding for other capabilities — bare `forbid` in 64 fragments

Recorded in `policy/cedar/README.md` with an executable demonstration. 64 of 448 tracked `.cedar`
fragments open with `forbid (principal, action, resource);`, which denies the entire `PolicySet` it
lands in. It is a **latent hazard, not a live defect** — no loader concatenates those fragments today.
Fixing them is outside this envelope and is not attempted here. Owners: `audit` (35),
`oya/global-trade` (10), `gateway` (6), `marketplace` (6), `oya/slides` (4), `iam` (2), `flags` (1).

## Suggested promotion order

1. A hub wave on `integ/specs` lands §1's four strikes plus the registry and envelope entries (§2.2),
   and records an adjunct claim + `Cargo.lock` waiver for `integ/policy`.
2. `integ/build` or the waivered `integ/policy` lands the first crate with its catalog row, the
   `SLO_CATALOG_CENSUS` re-freeze, the tier-dependency roots, and the two corpus raises.
3. `policy/ports/policy-snapshot-store` implements `CONTRACT.md`; `policy/adapters/policy-cedar-conformance`
   takes the harness in `cedar/CONFORMANCE.md` verbatim as its test body.
