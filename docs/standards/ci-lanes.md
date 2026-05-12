# Oyatie — CI Lanes Catalog

> **Owner:** `axis-foundry` + `ops-sre-reliability`.
> **Companion:** [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), [`standards/code-review.md`](code-review.md), [`standards/testing.md`](testing.md), [ADR-0050 automation-first-pipeline](../decisions/ADR-0050-automation-first-pipeline.md).

## 1. Lane catalog

Every CI gate is a named lane. Lanes are catalog-driven: `registry/quality/lanes.yaml` is the source of truth; this doc is the human-readable mirror.
The registry carries each lane's owner team and `runtime_budget_seconds`; the `quality-lanes` gate rejects unknown owners, missing budgets, markdown drift, and active commands absent from `scripts/check.sh`.

### 1.1 Foundation gate catalog (W-Foundation; active lanes block any merge; planned lanes preserve roadmap contract)

| Lane | Purpose | Source ADR |
|---|---|---|
| `oya-foundry-fitness-authority-cohesion` | authority-chain declarations in CONSTITUTION, AGENTS, and README stay identical | CONSTITUTION.md |
| `oya-foundry-fitness-claim-ceiling` | prevent unshipped stability, security, and supply-chain claims above foundation evidence | ADR-0037 / registry/catalog |
| `oya-foundry-fitness-codeowners-mirror` | RACI per-surface owner matches CODEOWNERS team ownership | RACI-OWNERSHIP.md |
| `oya-foundry-fitness-cohesion` | cross-axis contract review-class label | ADR-0011 |
| `oya-foundry-fitness-constitution-cite-coverage` | tier-one docs cite constitutional authority at heading level | CONSTITUTION.md |
| `oya-foundry-fitness-data-class` | enforce ADR-0008 data-class annotation | ADR-0008 |
| `oya-foundry-fitness-doc-catalog` | every consolidated doc has a DOC-CATALOG row | DOC-CATALOG.md |
| `oya-foundry-fitness-docs` | documentation-system pipeline registry and wiki quickref stay grounded | DOCUMENTATION.md / registry/docs/pipeline.tsv |
| `oya-foundry-fitness-quality-lanes` | registry/quality/lanes.yaml and this CI-lanes doc mirror stay in sync | standards/ci-lanes.md |
| `oya-foundry-fitness-foundation-bypass` | foundation-bypass expiry monitor | ADR-0040 |
| `oya-foundry-fitness-glossary-cross-doc-coverage` | every glossary term appears outside GLOSSARY when active | GLOSSARY.md §11 / ADR-0018 |
| `oya-foundry-fitness-glossary-vocabulary` | retired-vocab hard-fail plus casing/acronym warning baseline ratchet | GLOSSARY.md §11 / ADR-0018 |
| `oya-foundry-fitness-placeholder-debt` | fail-closed `TODO` / `TBD` registry so placeholder cleanup is tracked outside glossary acronym warnings | AGENTS.md Done-Definition / MISTAKES doctrine |
| `oya-foundry-fitness-license` | enforce ADR-0013 license posture | ADR-0013 |
| `oya-foundry-fitness-plane-class` | catalog plane-class changes require explicit review | ADR-0004 |
| `oya-foundry-fitness-raci-team-coverage` | every team charter has RACI and CODEOWNERS coverage | RACI-OWNERSHIP.md |
| `oya-foundry-fitness-readme-doc-coverage` | every root doc has catalog and README discoverability | README.md / DOC-CATALOG.md |
| `oya-foundry-fitness-runbook-index-resolves` | every RUNBOOKS-INDEX entry is a real file | RUNBOOKS-INDEX.md |
| `oya-foundry-fitness-slo-coverage` | every catalog record carries SLO coverage | SLO-CATALOG.md |
| `oya-foundry-fitness-catalog-records` | every Cargo workspace member has a catalog record | ADR-0015 / registry/catalog |
| `oya-foundry-fitness-flat-crates` | per-PR flat-crates path, legacy-tree, and role-boundary check | ADR-0015 |
| `oya-foundry-fitness-product-index` | product README index and machine-readable product catalog stay in sync | products/README.md |
| `oya-foundry-fitness-adr-citation` | only-new-pack-citations check | ADR-LEGACY-REGRESSION-MAPPING |
| `oya-foundry-fitness-brand-residue` | tautological brand transition check | ADR-0017 / MFL-0011 |

### 1.2 Per-PR gates (active and planned; active wiring is registry-enforced)

| Lane | Purpose |
|---|---|
| `cargo-fmt` | `cargo fmt --all -- --check` |
| `cargo-check` | `cargo check --workspace --all-targets --all-features` |
| `cargo-clippy` | `cargo clippy --workspace --all-features --all-targets -D warnings` |
| `cargo-nextest` | `cargo nextest run --workspace --all-features --no-fail-fast` |
| `cargo-deny` | per ADR-0013 license + advisory check |
| `oya-foundation-demo-smoke` | `oya demo` foundation smoke path exercises tenant, MCP, audit, run, step, outbox, and secret flows |
| `machine-readable-json-parse` | every docs/machine-readable JSON file parses before merge |
| `cargo-machete` | unused-deps |
| `pnpm-typecheck` | TS workspace typecheck |
| `pnpm-test` | TS unit + integration |
| `oya-foundry-fitness-supply-chain` | Trivy 4-layer + Cosign per ADR-0039 |
| `oya-foundry-fitness-supply-chain-bootstrap` | source-only supply-chain guard plus RustSec and deny wiring |
| `traceability-validator` | PR template carries the 5 mandatory traceability H2 sections |
| `oya-foundry-fitness-api-semver` | public-API stability tier per ADR-0037 |
| `oya-foundry-fitness-cargo-prefix` | every workspace member starts with `oya-` |
| `oya-foundry-fitness-pre-push` | repoctl pre-push command contract maps to the checked local verification bundle |

### 1.3 Nightly gates

| Lane | Purpose |
|---|---|
| `foundry-eval-nightly` | per-capability eval set per ADR-0024 |
| `chain-replay-drill` | per-shard audit-chain integrity per ADR-0003 |
| `cross-tenant-access-fuzz` | per-cell isolation per ADR-0009 |
| `oya-foundry-fitness-vendor-contract-recency` | per VENDOR-PARTNER-LEDGER |
| `oya-foundry-fitness-mobile-native` | per ADR-0051 |

### 1.4 Per-release gates

| Lane | Purpose |
|---|---|
| `release-supply-chain` | Cosign keyless + Rekor + SBOM per ADR-0039 |
| `release-evidence-pack` | per-regulator evidence regen per COMPLIANCE-MATRIX |
| `release-runbook-freshness` | freshness SLA per RUNBOOKS-INDEX §3 |

`release-supply-chain` runs in `pre-release` phase during local/all-lane checks:
an explicit empty-scope manifest may pass before a release candidate, but a tag
release runs `--phase release` and requires per-artifact evidence records.

## 2. Lane discipline

Per ADR-0050:
- Every lane has a runtime budget (per-lane wall-clock cap).
- Every lane is owned by a team (per RACI-OWNERSHIP.md).
- Lanes that fail produce an evidence record in the audit chain.
- Lanes that exceed budget auto-open an issue against the owning team.
- Adding / removing lanes requires `crew-adr-promotion` review + a CHANGELOG row.

## 3. Adding a new lane

1. Add or update the lane record in `registry/quality/lanes.yaml`.
2. Mirror the lane row in this document under the matching stage table.
3. If `status: active`, wire `check_command` into `scripts/check.sh`.
4. Run `oya gate validate quality-lanes`.
5. Open a PR; cite the source ADR in the PR body Verification section.
6. After merge, `oya-foundry-fitness-cohesion` validates the lane is wired into the per-PR + nightly + release shapes appropriately.

## 4. Sources
ADR-0050 (automation-first pipeline), ADR-0024 (eval harness), ADR-0039 (supply chain), ADR-0013 (license), ADR-0008 (data-class), ADR-0011 (cross-axis contracts), ADR-0017 (brand naming), ADR-0015 (flat crates), ADR-0009 (cell architecture), ADR-0037 (API stability), ADR-0051 (mobile native), [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), [`standards/code-review.md`](code-review.md), [`standards/testing.md`](testing.md).
