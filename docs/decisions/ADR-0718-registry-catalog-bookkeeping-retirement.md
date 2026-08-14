---
doc_status: published
id: ADR-0718
title: "Registry/catalog bookkeeping retirement: gates without a corpus retire with their bookkeeping, and survivors re-point at machine-native identity"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-14
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0555, ADR-0544]
amended_by: []
depends_on: [ADR-0716]
related: [ADR-0515, ADR-0551, ADR-0562, ADR-0563, ADR-0717]
milestone: W0
deliverables:
  - id: ADR-0718-D1
    description: "Delete the 17 gate crates whose enforcement corpus was the hand-maintained registry bookkeeping: crate-catalog-coverage, crate-registration, module-membership, corpus-index-coverage, service-catalog-parity, service-tier-metadata, lifecycle-status, feature-maturity-policy, planning-projection, parity-claim-evidence, stale-artifact-detection, topology-manifest-contract, contract-slice-conformance, package-manifest-hygiene, scan-root-liveness, artifact-accountability, and action-item-accounting — together with registry/catalog (775 rows), registry/milestone-audit, registry/design-spec-maturity, the friction-ledger merge driver, its .gitattributes registration, and the born-accounting pre-push check mode."
    exit_criteria: "No ci/facade directory, gate disposition row, oya-ci.toml enabled block, workflow step, or producer face remains for the retired gates; cargo check --workspace --all-targets is clean; the gate-registration, kernel, and self-conformance suites are green."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D2
    description: "Retire the schema + policy sections whose only consumer was a retired gate: [manifest] (ManifestConfig), [catalog_liveness] (CatalogLivenessConfig), and [slo_coverage].catalog_record_globs (SloCoverageConfig) leave the closed schema, the bundled defaults, oya-ci.toml, and the config reference in the same change; the disposition table drops the four retired gate rows and the topology-manifest-contract stub."
    exit_criteria: "oya-ci.toml [[gates.enabled]] and the bundled disposition converge exactly (11 gates); oya-ci-config-kernel tests green; the canonical-json gate has no malformed governed file."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D3
    description: "Re-point the retained slo-coverage gate from the catalog mirror to the canonical OpenSLO corpus: one row per tracked *.openslo.yaml envelope, keyed by repo-relative path with metadata.name as the declaration; census codes renamed (slo_empty_corpus, slo_census_drop_unattributed, slo_census_growth_unattributed); the frozen census pin moves 773 -> 727 with a recorded attribution."
    exit_criteria: "slo-coverage unit tests green; the duplicate-basename producer test keys rows by path; the live census test passes in CI against the materialized face."
    verified_by: "oya-ci-required"
  - id: ADR-0718-D4
    description: "Migrate the crate identity facet the deleted catalog carried into machine-native identity: the eight tools/oya-governance-*-app crates carry capability: fitness-* in [package.metadata.oya-ci], and the outside-the-fleet discriminator in the gate-registration suite reads the manifest facet instead of the catalog row."
    exit_criteria: "gate_registration outside-fleet tests green against the manifest facet; no consumer reads registry/catalog anywhere in the tree."
    verified_by: "oya-ci-required"
---

# ADR-0718: Registry/catalog bookkeeping retirement

## Status

**Accepted** (founder directive 2026-08-14, anti-friction wave 4: "registries, catalogs, related
CI … reorg/rewrite/refactor/remove … purge"). Amends ADR-0555 (the accounting admission doctrine
loses its gates but keeps its producer-side registration semantics) and ADR-0544 (the friction
ledger gate retires with the ledger driver).

## Context

Wave 2 cleaned the doc/evidence corpora and wave 3 froze their budgets, but the CI fleet still
carried a parallel class of hand-maintained bookkeeping: a 775-row crate catalog, milestone and
design-spec maturity registries, per-gate baseline/policy JSON, and seventeen gate crates whose
entire enforcement corpus was those hand-maintained rows. The rows duplicated facts that live
elsewhere by construction — package identity lives in each crate's `Cargo.toml`, SLO declarations
live in the canonical `*.openslo.yaml` corpus, workspace membership lives in the root manifest —
so the gates were enforcing the accuracy of their own mirror. Hyperscaler practice is the
inverse: durable properties are enforced from the machine-native source, and a gate without a
corpus retires instead of decaying into a false-green mirror guard.

## Decision

1. **A gate whose only corpus is hand-maintained bookkeeping retires with the bookkeeping.**
   Seventeen gate crates, `registry/catalog/`, `registry/milestone-audit/`,
   `registry/design-spec-maturity/`, the friction-ledger merge driver + its `.gitattributes`
   registration, and the born-accounting `--check-paths`/`--check-diff` pre-push mode are
   deleted outright — no tombstones, no stub policies.
2. **Retired gates leave no orphan schema.** `[manifest]`, `[catalog_liveness]`, and
   `[slo_coverage]` sections, their config structs, the disposition rows, the enabled-gate
   blocks, the producer face collection, and the workflow wiring are removed in the same change
   so the closed schema and the gate catalog describe exactly the live fleet (11 gates).
3. **Retained gates re-point at machine-native identity.** `slo-coverage` enumerates the
   tracked `*.openslo.yaml` corpus (one row per envelope, keyed by path, `metadata.name` as the
   declaration) and re-freezes its census pin 773 -> 727 with attribution. The `fitness-*`
   crate facet moves from catalog rows into `[package.metadata.oya-ci]` and the
   outside-the-fleet discriminator reads the manifest.
4. **Producer-side registration semantics survive the gates.** The registry face still
   computes owner/justification/reachability per tracked path and still feeds registry-drift
   and the baseline ratchet; the `--fix-owners`/`--fix-reachability` local bridges remain the
   transitional registration tools.
5. **No silent enumeration collapse.** The retained gates keep their fail-closed census and
   frozen-empty dispositions; every removal in this change carries its number moves (census
   pin, disposition rows, gate count) in the same PR.

## The rule (northstar, five-field)

- **achieves:** enforcement reads the machine-native source of truth instead of guarding a
  hand-maintained mirror, and a gate whose corpus is deleted cannot silently turn into a
  vacuous pass.
- **origin:** the 775-row catalog mirrored package identity and per-crate SLOs that already
  lived in `Cargo.toml` manifests and `*.openslo.yaml` files; gates built on the mirror were
  enforcing the mirror, and their bookkeeping was the friction wave 4 targets.
- **rule:** when registry/catalog/plan bookkeeping is removed, its consuming gates retire in
  the same change; a retained gate MUST be re-pointed at a machine-native corpus with a
  fail-closed census (equality pin) so an enumeration collapse is RED, never a quiet pass.
- **ensure:** the gate-registration suite pins the fleet/disposition convergence exactly; the
  slo-coverage census pin is equality-checked in CI; `cargo check --workspace --all-targets`
  fails on any orphan reference to a deleted crate.
- **overturn_when:** a recorded challenge shows a retired gate's corpus was actually
  load-bearing with no machine-native replacement, AND a re-created gate ships with a
  reviewed producer + frozen baseline in the same change.

## Consequences

- Positive: ~890 files and every hand-maintained mirror they guarded are gone; the fleet is 11
  gates with machine-native inputs; the closed schema, disposition table, gate catalog, and
  config reference describe exactly the live system.
- Negative: registration of ownership/reachability loses its admission-gate backstop and now
  rides the registry face + ratchet alone (accepted: the face is still byte-diffed and
  ratcheted); slo-coverage loses per-crate tier scalars the catalog mirrored (accepted: the
  openslo corpus is the durable record and the tier string was never validated against an
  enum).
- The two pre-existing cargo-merge-path breaks the wave exposed — the shell-frontend `ssr`
  feature gating for its live-server test and the `serde_json`/`futures` dev-deps — are fixed
  in the same change so the ADR-0716 merge path is green end to end.
