---
doc_class: Reference
shape: Reference
microservice: tenancy
related_adrs: [ADR-0258]
---

# tenancy — CHANGELOG

Versioning: SemVer per ADR-0258. Contracts: `contracts/openapi/tenancy.yaml`,
`contracts/asyncapi/tenant-events.yaml`, `contracts/proto/tenancy.proto`.

## [Unreleased]

### Implemented — W1 capability delivery (2026-08-20)

Ten of the capability's crates were 45-60-line *Wave 15-IMPL-truth-up scaffolds* carrying
declared types and ports with no implementation and **zero tests**; `ports/cli` had 220 lines
and no tests. Each is now a real, tested implementation of its Implementation Plan.

| crate | IP | tests |
|---|---|---|
| `core/cell-assignment` | IP-008 | 66 |
| `core/dsr-cascade` | IP-009 | 64 |
| `core/isolation-policy` | IP-006 + IP-007 | 78 |
| `core/lifecycle-locks` | IP-021 | 94 |
| `core/sub-scope-registry` | IP-016 | 41 |
| `core/reserved-namespace` | IP-017 | 62 |
| `core/kyb-kyc-verifier` | IP-018 | 69 |
| `core/dr-pairing` | IP-019 | 58 |
| `core/per-tenant-quota` | IP-022 | 72 |
| `adapters/data-residency-enforcer` | IP-020 | 64 |
| `ports/cli` | — | 35 |

Capability total: **891 tests passing, 0 failing** (188 before). `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` are clean across all 22 crates.

### Why one crate per IP instead of the IPs' multi-crate plans

Each IP proposed a `oya-tenancy-<bc>-{kernel,domain,usecase,adapter,worker}` fan-out. All of
them are collapsed into the single existing crate as a module tree. Two constraints force it and
neither is negotiable from inside this capability: the founder cap is **≤12 crates per
capability**, and manifest-census headroom is **zero** (PR #2174), so a crate creation must be
paid for by a collapse elsewhere. `Cargo.lock` is a hub path owned by `integ/build`
(`#planes.root_manifests`) and tenancy holds no waiver — so no crate here gained *any*
dependency, not even a path dependency on a sibling tenancy crate, because that rewrites the
lockfile too. Every crate is implemented against `std` alone.

### Defects found and fixed (not merely scaffolding filled in)

Adversarial review — two reviewers per crate on separate lenses, then a fix-or-refute pass —
found real bugs, several of them fail-open:

- **`ports/cli`** shipped an inverted configuration precedence. clap validates `env` eagerly, so
  `OYA_OUTPUT=yaml oya-tenant --output json version` exited 2: the explicit flag that ADR-0167
  makes the highest rung could not be reached, leaving no command-line escape from a stale
  environment variable. An empty `OYA_OUTPUT=` aborted every command outright.
- **`core/sub-scope-registry`** resolved the subject tenant-safely and then delegated to the RAW
  traversal, dropping the tenant key — the cross-tenant read this capability exists to prevent.
- **`core/kyb-kyc-verifier`** keyed screening results by provider alone, so within one vendor
  response a later PEP clear silently overwrote that same vendor's sanctions hit.
- **`core/lifecycle-locks`** made `PendingDeletionGrace` *imply* `DeleteTenant`, so the grace
  period that exists to block deletion also authorized it; the fix exposed a wider defect where
  acquisition conflicts depended on the ORDER two locks arrived in.
- **`core/cell-assignment`** drained `Degraded` cells against its own documented contract, and
  its integrity check used a total-occupancy checksum that cannot detect a tenant delivered to
  the *wrong* cell. Fixes are pinned by mutation testing: 8 mutants applied, 8 killed.
- **`core/per-tenant-quota`** matched pack names byte-exactly, so a regulated ceiling spelled
  `US-HC` silently failed to bind while the decision still claimed pack provenance.
- **`core/dsr-cascade`** never bound the cascade plan to the request, so a waiver holder could
  seal a proof over receipts that were never theirs.
- **`core/reserved-namespace`** split candidate and reserved entry into segments by *different*
  rules, so `o-yatie-support` matched no index and `admin-console` failed to match itself.

### Known gaps — deliberate, and recorded in each crate's `lib.rs` header

- **No Unicode confusable handling** in `reserved-namespace`. The skeleton fold is ASCII-only; a
  Cyrillic or full-width homograph is refused *only* because the syntax rule rejects non-ASCII
  bytes. IP-017's Unicode requirement is **not met**.
- **`dsr-cascade` ships a hand-rolled SHA-256**, pinned to published NIST vectors but unaudited,
  and its proof carries no signature — a certificate whose coverage list and waiver are rewritten
  together still verifies.
- **`isolation-policy` validates JWT claim *shape* and does not verify signatures.**
- **Cedar remains the residency policy authority**; `data-residency-enforcer` mirrors it in Rust
  with nothing automatically binding the two.
- External adapters (Postgres/Citus, Valkey, real KYB/KYC providers), async workers and REST
  surfaces are out of scope behind the sync ports these crates define.

### Corrected

- The Wave-3-B entry below lists a Cedar fragment `data-residency.cedar` among the files added.
  **That file does not exist** anywhere in the tree; the only Cedar in this capability is
  `tenancy/cedar/policies.cedar`. `adapters/data-residency-enforcer` cited the same phantom path
  in its module documentation and now cites what actually exists. The historical entry is left
  standing rather than rewritten, so the record shows the claim and the correction.
- Eleven IP frontmatter `status:` values moved to `in-progress`. Six of them read `planned`,
  which is not in the `specs/ip/canonical-frontmatter-schema.json` enum
  (`pending|in-progress|complete|blocked|deferred|withdrawn`) and was therefore off-schema.
  `in-progress` rather than `complete` is deliberate: the pure cores landed, the adapters did not.

### Added (Wave-3-B gap-fill 2026-05-20)
- ARCHITECTURE.md, README.md, CHANGELOG.md.
- IPs IP-016..IP-026 covering sub-scope-registry / reserved-namespace / KYB-KYC / DR-pairing /
  data-residency-enforcement / lifecycle-locks / per-tenant-quota BCs.
- Cedar fragments: `action-authorization.cedar`, `abuse-defence.cedar`,
  `data-residency.cedar`.
- IaC: edge-waf.yaml, ech-config.yaml, pqc-cert.yaml, openbao-policy.hcl,
  secret-bindings.yaml, kustomize residency overlays (eu / kr / us-healthcare),
  multi-region-failover.tf.
- Dashboards: dr-pairing-state, kyb-kyc-pipeline, quota-utilisation.
- Catalog records for new BCs.
- Capabilities additions: dr-pair-promote, quota-update.
- AUDIT-FINDINGS-2026-05-20.json (new audit pass).

### Changed
- manifest.json scorecards + IP register expanded.

## [0.1.0] — 2026-04
Initial PHASE-01 substrate per ADR-0244.
