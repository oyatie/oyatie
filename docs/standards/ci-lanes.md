---
purpose: Oyatie — CI Lanes Catalog
doc_status: published
---

# Oyatie — CI Lanes Catalog

> **Owner:** `axis-foundry` + `ops-sre-reliability`.
> **Companion:** [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), [`standards/code-review.md`](code-review.md), [`standards/testing.md`](testing.md), [ADR-0050 automation-first-pipeline](../decisions/ADR-0050-automation-first-pipeline.md).

## 1. Lane catalog

Every CI gate is a named lane. Lanes are catalog-driven: `registry/quality/lanes.yaml` is the source of truth; this doc is the human-readable mirror.
The registry carries each lane's owner team and `runtime_budget_seconds`; `check_command` values are local/transitional bridge feedback and wired-command catalog data only, not protected-branch merge authority. The `quality-lanes` gate rejects unknown owners, missing budgets, markdown drift, and active commands absent from the canonical wired-command catalog (`oya-governance-gate-catalog-domain`).
Protected-branch authority is the single `oya-ci-required` fan-in plus constituent cloud-ci/Rust gate packets.

### 1.1 Foundation gate catalog (W-Foundation; active lanes block any merge; planned lanes preserve roadmap contract)

| Lane | Purpose | Source ADR |
|---|---|---|
| `oya-governance-authority-cohesion` | authority-chain declarations in AGENTS, README, and MASTERPLAN stay identical and avoid retired prescribed authorities | root-hub-pointers.json / docs/AGENTS.md / ADR-0116 |
| `oya-governance-claim-ceiling` | prevent unshipped stability, security, and supply-chain claims above foundation evidence | ADR-0037 / registry/catalog |
| `oya-governance-codeowners-mirror` | RACI per-surface owner matches CODEOWNERS team ownership | RACI-OWNERSHIP.md |
| `oya-governance-cohesion` | cross-axis contract review-class label | ADR-0011 |
| `oya-governance-data-class` | enforce ADR-0008 data-class annotation | ADR-0008 |
| `oya-governance-doc-catalog` | every consolidated doc has a DOC-CATALOG row | DOC-CATALOG.md |
| `oya-governance-docs` | documentation-system pipeline registry and wiki quickref stay grounded | DOCUMENTATION.md / registry/docs/pipeline.tsv |
| `oya-governance-quality-lanes` | registry/quality/lanes.yaml and this CI-lanes doc mirror stay in sync | standards/ci-lanes.md |
| `oya-governance-honest-claims` | scan authoritative docs/specs/ADRs for deferred active claims and validate ImplementationPlan ChangeSet graph integrity | ADR-0129 / specs/plan-schema.json / specs/masterplan.json |
| `oya-governance-aspirational-enforcement` | block active required enforcement claims that reference missing check crates, workflows, or branch-protection contexts | ADR-0135 / ADR-0133 |
| `oya-governance-banned-primitives` | block banned primitive use inside fenced agent-instruction contracts and the sanitized tracked command-log corpus | specs/master-plan-sequencing.json / F-FORBIDDEN-PRIMITIVES-CI-GUARD |
| `oya-governance-workspace-hygiene` | inventory temp, home, repo-root, build-artifact, and `oyatie-worktrees` residue before pipeline closeout, with explicit temp/build cleanup and owned-root exemptions | specs/workspace-hygiene.json / ADR-0123 |
| `oya-governance-hyperscaler-maturity-claims` | block unsupported hyperscaler maturity claims unless product depth, pipeline, hygiene, UX, safety, guardrail, and competitor evidence are green | specs/hyperscaler-gates.json / ADR-0123 |
| `oya-governance-design-spec-maturity-claims` | allow only the bounded design/spec maturity claim when every microservice has the required implementation-ready design surfaces, while keeping operational maturity blocked | specs/design-spec-maturity-claims.json / ADR-0123 |
| `oya-governance-foundation-bypass` | foundation-bypass expiry monitor | ADR-0040 |
| `oya-governance-glossary-cross-doc-coverage` | every glossary term appears outside GLOSSARY when active | GLOSSARY.md §11 / ADR-0018 |
| `oya-governance-glossary-vocabulary` | retired-vocab hard-fail plus casing/acronym warning baseline ratchet | GLOSSARY.md §11 / ADR-0018 |
| `oya-governance-placeholder-debt` | fail-closed `TODO` / `TBD` registry so placeholder cleanup is tracked outside glossary acronym warnings | AGENTS.md Done-Definition / MISTAKES doctrine |
| `oya-governance-dependency-seam` | fail-closed ADR-0092 dependency rationale coverage, adapter-only imports, fixture-pair coverage, change_class declarations, and online cargo-audit vulnerability checks | ADR-0092 / registry/dependency-rationales.json |
| `oya-governance-plane-class` | catalog plane-class changes require explicit review | ADR-0004 |
| `oya-governance-raci-team-coverage` | every team charter has RACI and CODEOWNERS coverage | RACI-OWNERSHIP.md |
| `oya-governance-readme-doc-coverage` | every root doc has catalog and README discoverability | README.md / DOC-CATALOG.md |
| `oya-governance-runbook-index-resolves` | every RUNBOOKS-INDEX entry is a real file | RUNBOOKS-INDEX.md |
| `cloud-ci-slo-coverage` | every catalog record carries SLO coverage | SLO-CATALOG.md |
| `oya-governance-catalog-records` | every Cargo workspace member has a catalog record | ADR-0015 / registry/catalog |
| `oya-governance-product-index` | product README index and machine-readable product catalog stay in sync | products/README.md |
| `oya-governance-adr-citation` | only-new-pack-citations check | ADR-LEGACY-REGRESSION-MAPPING |
| `oya-governance-brand-residue` | tautological brand transition check | ADR-0017 / MFL-0011 |

### 1.2 Per-PR gates (active and planned; active wiring is registry-enforced)

| Lane | Purpose |
|---|---|
| `cargo-fmt` | `cargo fmt --all -- --check` |
| `cargo-check` | `cargo check --workspace --all-targets --keep-going` |
| `cargo-clippy` | `cargo clippy --workspace --all-targets --keep-going -D warnings` |
| `cargo-nextest` | `cargo nextest run --workspace --no-fail-fast` |
| `cargo-deny` | per ADR-0013 license + advisory check |
| `oya-foundation-demo-smoke` | `oya demo` foundation smoke path exercises tenant, MCP, audit, run, step, outbox, and secret flows |
| `machine-readable-json-parse` | every docs/machine-readable JSON file parses before merge |
| `cargo-machete` | unused-deps |
| `pnpm-typecheck` | TS workspace typecheck |
| `pnpm-test` | TS unit + integration |
| `oya-governance-supply-chain` | Trivy 4-layer + Cosign per ADR-0039 |
| `oya-governance-supply-chain-bootstrap` | source-only supply-chain guard plus RustSec and deny wiring |
| `oya-governance-api-semver` | public-API stability tier per ADR-0037 |
| `oya-governance-cargo-prefix` | every workspace member starts with `oya-` |
| `oya-governance-pre-push` | oya verify command contract maps to the checked local verification bundle (canonical local pre-push entry; retired entry points are recorded in registry/vocabulary/retired.yaml) |
| `oya-governance-loop-recovery-patterns` | pre-push repeat-mistake blocker joins deterministic score cards, loop-recovery patterns, and mistakes-ledger rows without shell hook expansion |
| `oya-governance-master-plan-completion` | status-honesty audit — no phase in specs/masterplan.json#live_implementation_index may be Complete while any child IP is stub/planned/pending/blocked/in-flight/probe-green; every complete IP must be referenced by at least one evidence JSON file |
| `oya-governance-retired-vocabulary` | no live document mentions any retired CLI surface, retired crate, or retired script path (registry/vocabulary/retired.yaml is the canonical record) |
| `oya-governance-protection-context-match` | every required-status-check context in .github/branch-protection.yaml is the `name:` field of some workflow job (prevents silent-bypass where GitHub waits forever for a context no workflow posts) |
| `oya-governance-vacuous-green` | ADR-0221 fixture-backed check that vacuous-green hook detection fails on intentionally empty gate evidence |
| `oya-governance-adr-orphan-citation` | ADR-0221 fixture-backed check that orphan ADR citation detection catches missing decision records |
| `oya-governance-version-pin-source-citation` | ADR-0221 fixture-backed check that OpenAPI and AsyncAPI version pins reject noncanonical versions |
| `oya-governance-buildability-line-count` | ADR-0221 fixture-backed check that buildability line-count guidance distinguishes short docs from substantive docs |
| `oya-governance-high-risk-auto-decision-refusal` | enforces that every µservice declaring a high-risk "refused at Cedar layer" claim in `capabilities/T2-auto.yaml` has a matching `forbid` rule in `policy/tenant-scope.cedar` gating on employment-context (SEC-MAJ-02). |
| `oya-governance-slsa-l3-evidence-grounded` | enforces that every scorecard claiming `slsa_l3: green` cites real `.github/workflows/<file>.yml` files that declare SLSA-relevant primitives (signed provenance, hermetic build, two-party review) (SEC-MAJ-01). |
| `oya-governance-otel-trace-propagation` | validates that per-µservice gRPC client adapters propagate W3C traceparent per ADR-0145 Invariant 2. Runs in DEFERRED (advisory) mode until the strict-mode parser lands per registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-otel-propagation-validator. |
| `oya-governance-ontology-projection-coverage` | blocks canonical-entity-owning µservices that omit concrete `ontology_projections` in their manifest per ADR-0145 Invariant 3. Registry-authority cross-checking remains tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-ontology-projection-validator. |
| `oya-governance-audit-chain-seal-coverage` | validates that µservices with `audit_chain.enabled=true` declare seal events per ADR-0145 Invariant 1. Runs in DEFERRED (advisory) mode until the strict-mode parser lands per registry/placeholder-debt/adr-follow-ups.yaml#adr-0145-audit-chain-seal-validator. |
| `oya-governance-layered-architecture-discipline` | enforces zero feature overlap across the layered hyperscaler architecture — Cilium L3/L4 vs Istio Ambient L7 (ADR-0148); gateway vs mesh (ADR-0182); Cedar vs Kyverno (ADR-0183); Valkey vs Memcached (ADR-0184). Rejects any µservice manifest declaring conflicting layer ownerships or under-claimed mesh tiers. |
| `oya-governance-client-stack-discipline` | enforces ADR-0185 native-per-platform client stack — SvelteKit (Phase 1) → Leptos (Phase 2) on web (sequential not parallel); Swift+SwiftUI on Apple (no KMP klib imports); Kotlin+Compose+KMP on Android (Android-scope only); WinUI 3 on Windows; gtk4-rs + libadwaita on Linux. Rejects React/Vue/Flutter/Electron/Cordova/Tauri/Avalonia/MAUI references. |
| `oya-governance-vendor-lockin-discipline` | enforces ADR-0173 tiered vendor classification — Tier I OWNED with license+steward; Tier II VENDOR-SEAMED with replacement_path, replacement_readiness_gate, seam_adapter_trait, and ≥1 seam_adapter_impls; Tier III FORBIDDEN with refusal rationale + replacement path. |
| `oya-governance-authz-tier-discipline` | enforces ADR-0191 tier boundary — Cedar policy MUST NOT reference edge concerns (ip/asn/geo/country/rate/waf/bot/ddos); Envoy filter config MUST NOT reference origin concerns (oidc principal claims/acr/tenant identity/residency/purpose/data_class). Advisory until per-microservice Cedar/Envoy assets exist. |
| `oya-governance-tenant-cost-labels-coverage` | advisory presence check for mandatory tenant cost labels per ADR-0199 D-1 (oya.io/tenant-id + oya.io/cost-center + oya.io/workload-class + oya.io/regulatory-pack) on every rendered Helm manifest under microservices/*/iac/helm/*/rendered/*.yaml. |
| `oya-governance-backup-retention-discipline` | advisory per-µservice backup retention validation per ADR-0197 D-5 (tier-driven RPO/RTO floor; storage.backup.tier ∈ {app, batch, gpu, regulatory}; storage.backup.retention_days set). |
| `oya-governance-vector-store-discipline` | advisory enforcement of Milvus-canonical vector store tier per ADR-0192 (collection schema, index_type, metric, data_classes). Advisory until per-µservice manifests populate data.vector_store.* namespace. |
| `oya-governance-olap-tier-discipline` | advisory enforcement of ClickHouse-canonical OLAP tier per ADR-0193 (no rogue analytics queries against OLTP postgres). Advisory until per-µservice manifests populate data.olap_client.* namespace. |
| `oya-governance-wasm-runtime-discipline` | advisory enforcement of wasmtime-canonical WASM substrate per ADR-0200 (sandbox classes envoy-filter, workflow-studio-node, or foundry-tool). Promotion target T+60d after ADR-0200 land; flip to blocker. |
| `oya-governance-iac-tier-discipline` | advisory enforcement of OpenTofu-canonical IaC tier per ADR-0202; Terraform path supported only during 90-day migration window. Promotion target T+90d after ADR-0202 land; flip to blocker (forbids new .tf files). |
| `oya-governance-a11y-discipline` | advisory WCAG 2.2 AA coverage per ADR-0207 (axe + pa11y runners declared per client surface in manifest.json#a11y). |
| `oya-governance-i18n-coverage` | advisory Fluent ICU locale coverage per ADR-0206 (default_locale + required_locales + rtl_support + min_coverage_bps). |
| `oya-governance-compliance-evidence-coverage` | advisory compliance evidence collector declaration per ADR-0209 (audit_chain_seal_required + tamper_evidence_algorithm + evidence_collectors[]). |
| `oya-governance-realtime-transport-tier` | advisory real-time transport tier validation per ADR-0208 (sse vs websocket vs grpc-streaming; payload_budget_bytes per tier). |
| `oya-governance-adr-planning-completeness` | ADR-0364 D2 — every planning_impact ADR that declares deliverables has id/description/exit_criteria/verified_by plus a milestone; ADRs without deliverables are advisory until D7 backfill |
| `oya-governance-masterplan-drift` | ADR-0364 D4 — `masterplan.generated.json` (de-committed per ADR-0613) regenerates successfully from the `planning_impact` ADR log (regeneration-success, not committed-byte parity; wraps `gen masterplan --check`). Local-bridge feedback only; required-CI freshness directly regenerates masterplan and product-graph twice and compares each face independently in `gate-generated-artifact-freshness`. |
| `oya-governance-adr-supersession-consistency` | #6b — every ADR supersedes/superseded_by pair is bidirectional; fails on any one-directional link (X supersedes Y but Y does not back-link X, or vice versa) over ADR-to-ADR edges |
| `lean-a1-architecture` | layer-correctness — no dep-direction violations per ADR-0056 §2.2 |
| `lean-a2-bounded-contexts` | microservice-isolation — no cross-µservice deps except via workflow/ontology (v4.1 override) |
| `lean-a3-supply-chain` | supply-chain integrity — Trivy + RustSec + deny per ADR-0039 |
| `lean-a4-semver` | API semver stability tier per ADR-0037 |
| `lean-a5-documentation` | documentation set coverage — every µservice has full canonical + per-pack documentation set per ADR-0063; orphan-scan + masterplan↔workspace registry reconciliation; flips to BLOCKER at M02-P22 |
| `lean-a10-regression` | catch attempted silent regressions of public contracts (manifest.json schema bump without ADR; Cedar policy widening; Protobuf field reuse; fitness lane severity downgrade; µservice removal without retirement ADR; doc frontmatter schema downgrade; event topic schema regression); per feedback_no_silent_regression.md (Linus-style) |
| `quality-statelessness` | no module-level mutable state in application/worker/presentation layers per ADR-0062 |
| `quality-shardability` | all DB designs declare tenant_id partition key + RLS per ADR-0062 |
| `quality-perf-budget` | impl plans include load-test results meeting declared perf targets per ADR-0062 |
| `quality-benchmark` | PRDs include competitive-benchmark section before L4→L5 per ADR-0062 |
| `lean-a-active-artifact-contract` | every machine-readable artifact under applicable_paths_glob conforms to v3.0.0 9-capability contract per ADR-0089; invoked from `oya gate run-all`; emits evidence + graph-edges artifacts on every run |
| `lean-a-cedar-fragment-coverage` | enforces invariants C01..C04 from /registry/cedar-fragments.json — no orphan .cedar files, no dangling cedar_fragments[] references in OpenAPI contracts or bounded-contexts.json, status↔path consistency; invoked from `oya gate run-all` |
| `lean-a-openapi-rest-route-parity` | enforces 1:1 parity between `pub const *_ROUTE` constants in crates/oya-ops-*-rest/src/lib.rs and `paths:` keys in contracts/ops-*.openapi.yaml; default scope ops-only via --crate-prefix/--contract-prefix flags |

### 1.3 Nightly gates

| Lane | Purpose |
|---|---|
| `oya-governance-merge-queue-ref-hygiene` | GC merge-queue-staging-i refs older than 1 hour per ADR-0111 §"Consequences"; transient projected-merge-state staging refs must not accumulate after each scheduler tick. Lane is the post-tick hygiene gate referenced by ADR-0111 wave-C. |
| `foundry-eval-nightly` | per-capability eval set per ADR-0024 |
| `chain-replay-drill` | per-shard audit-chain integrity per ADR-0003 |
| `cross-tenant-access-fuzz` | per-cell isolation per ADR-0009 |
| `oya-governance-vendor-contract-recency` | per VENDOR-PARTNER-LEDGER |
| `oya-governance-mobile-native` | per ADR-0051 |

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
3. If `status: active`, wire `check_command` into the local/transitional `oya gate run-all` bridge catalog (`marketplace/facade/dev-cli/src/commands/gate/run_all.rs::AGGREGATED_VALIDATE_LANES`) so local feedback and wired-command catalog validation stay synchronized; protected-branch merge authority remains `oya-ci-required` plus cloud-ci/Rust gate packets.
4. Run `oya gate validate quality-lanes` as local bridge feedback; do not treat it as protected-branch authority.
5. Open a PR; cite the source ADR in the PR body Verification section.
6. After merge, `oya-governance-cohesion` validates the lane is wired into the per-PR + nightly + release shapes appropriately.

## 4. Sources
ADR-0050 (automation-first pipeline), ADR-0024 (eval harness), ADR-0039 (supply chain), ADR-0013 (license), ADR-0008 (data-class), ADR-0011 (cross-axis contracts), ADR-0017 (brand naming), ADR-0015 (flat crates), ADR-0009 (cell architecture), ADR-0037 (API stability), ADR-0051 (mobile native), [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md), [`standards/code-review.md`](code-review.md), [`standards/testing.md`](testing.md).
