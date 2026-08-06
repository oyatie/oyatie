---
id: ADR-0069
status: Superseded
superseded_by: [ADR-0709]
amended_by: [ADR-619]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0069: Active machine-readable artifact contract — 9-capability declaration, knowledge-graph substrate, registry-pattern control plane

> **Status:** Accepted
> **Owner:** `council-architecture` + `axis-foundry`
> **Date:** 2026-05-13
> **Related:** ADR-0015 (flat-crates), ADR-0056 (BNF v4.1), ADR-0063 (doc-coverage), ADR-0067 (ops portal), ADR-0088 (foundry microservice scaffolding)

---

## Context

The repo accumulates JSON/TOML/YAML/Cedar/SQL/etc. machine-readable artifacts (specs, registries, ledgers, claim-matrices, evidence bundles, OpenAPI contracts, lane registries, capability records) that drift without mechanical prevention. Per CONSTITUTION Decision principle 3 (mechanical prevention over process), every recurring failure produces a CI lane, hook, validator, or schema check — not a checklist line.

The Ops portal plan set consensus loop (Wave 5 v18 Accepted, 13 architect + 7 critic rounds via codex gpt-5.5) revealed five systemic gaps in the existing canonical control surface:

1. **No drift-prevention contract for machine-readable artifacts.** Every spec invents its own `_schema.validation_check` string with no enforcement. Stale text accumulates between rounds; only manual critique caught it.
2. **No DRY-enforcement registry.** Recurring patterns (Cedar predicates, port traits, schemas, ADR shapes) get reinvented across plans because there is no central catalog naming the canonical home + consumers.
3. **No knowledge-graph substrate.** Relationships (consumes, supersedes, depends_on, owned_by, validates, generates, heals) are encoded as free-text path strings in different formats across registries. Cross-cutting queries ("which artifacts cite this ADR", "which lanes enforce this schema") are not answerable.
4. **No claim-matrix discipline.** Plans assert `verified-by-existing-CI` for capabilities that are in fact prerequisite-gated; the existing `hyperscaler-gates.json` declares `claim_matrix_required` but no artifact actually publishes it in canonical shape.
5. **Per-artifact state machines (V-row taxonomy, gap registries, successor commitments) live in Markdown tables, not JSON state.** Validators cannot read Markdown tables; drift is undetectable until manual audit.

User directives 2026-05-13 made the requirement explicit:
- "Anything that can be automated should be automated"
- "Make our json and our machine readable documentation have purpose and features: Enforcement, Verification, Validation, Auto-generations, Self-healing, Self-updating, Self-maintaining"
- "json, toml, yaml, or whatever files that we have. what can be automated must be automated"
- "fine tune and polish our idea into something that is hyperscaler grade in infrastructure"
- "Make it so that we are able to keep track of reusable building blocks so that we avoid duplicating. implement all the optimization measures that is considered best practice."
- "every relationship, every capability, every feature, every schema, everything mapped graphed and tracked for automation is ideal"
- "DRY enforcement is key"

Hyperscaler precedent for the right shape: AWS Config + AWS Resource Explorer (resource compliance + queryable inventory); GCP Asset Inventory + Cloud Asset Graph (typed resource + relationship graph); Kubernetes CRD + admission controllers (schema + validator wiring blocks bad writes); Cargo workspace.dependencies + Maven BOM (centralized version pinning so consumers reference, never duplicate).

Internal critique by codex architect r17 (Torvalds-lens) on the proposed scaffold returned ITERATE with 10 specific findings; this ADR records the resulting decision after closing 8 of 10 findings inline.

---

## Decision

Adopt the **active machine-readable artifact contract** v3.0.0 with three load-bearing artifacts and one validator crate. The contract is format-agnostic (applies to JSON, TOML, YAML, Cedar, SQL, OpenAPI, GitHub Actions YAML, Cargo.toml, etc.) and registry-based (control plane in registry; data plane in artifacts).

### Components

| Component | Path | Role |
|---|---|---|
| Contract schema | `/specs/active-machine-readable-artifact-contract.json` | Defines the 9 capabilities and their required fields. Applies as registry-row shape. |
| Knowledge-graph schema | `/specs/knowledge-graph-schema.json` | Pure meta-schema (post-r17 #3 split). |
| Knowledge-graph catalog | `/registry/knowledge-graph-catalog.json` | First-class catalog: 24 node types + 18 edge types + 14 graph-level invariants + 5 DRY query examples. |
| Capability registry | `/registry/artifact-capabilities-registry.json` | Control plane: one row per machine-readable artifact, listing its 9-capability statuses + anchors. |
| Building-blocks registry | `/registry/reusable-building-blocks-registry.json` | DRY enforcement: one row per reusable block, with canonical_path + consumers + version + deprecation. |
| Validator crate | `crates/oya-check-active-artifact-contract` | Pure-Rust kernel (std-only) that loads the capability registry, resolves HEAD-tracking, detects duplicate IDs, detects operational-without-evidence. Exposed via `oya check active-artifact-contract` once integrated with `oya-dev-cli`. |
| CI lane | `lean-a-active-artifact-contract` in `registry/quality/lanes.yaml` | Enforces the contract at PR time. Status `planned` until validator integration completes. |

### The 9 capabilities

Every active machine-readable artifact in the repo MUST declare these 9 capabilities in the capability registry, each with status `operational | planned | blocked-by-foundation | not-applicable`:

1. **Enforcement** — CI lane that BLOCKS PRs on violation (timing + action enum).
2. **Verification** — Rust checker crate exposing `oya check <name>` for continuous verification.
3. **Validation** — Pre-commit / pre-push / pre-grit-claim / LSP hook for write-time validation.
4. **Autogen** — Generator that derives the artifact from upstream canonical sources (with declared idempotency class: deterministic / content-addressed-sha256 / content-addressed-blake3 / stochastic-with-seed / non-deterministic).
5. **Selfheal** — Healer module that auto-repairs verification failures (with safe-unattended vs requires-human-approval split, and recovery-path for destructive heals).
6. **Selfupdate** — Trigger-driven incremental drift correction (post-grit-done-hook, ci-lane-on-trigger, scheduled-job).
7. **Selfmaintain** — Long-running hygiene policies (GC, archive, reconcile) with audit-emission requirement.
8. **Telemetry** — OpenTelemetry-aligned metrics with cardinality budgets per label.
9. **Provenance** — Author + reviewer + autogenerated_by + last_modified_at chain, with SLSA-L3 target and grit-done-seal interim.

### Honest claim rule

`status: operational` requires resolvable evidence (crate path in HEAD + lane status `active` in lanes.yaml + green CI run URL). `status: planned` requires populated `prerequisite_for_operational`. `status: blocked-by-foundation` requires citing the foundation work (e.g., cosign/trivy/audit-chain runtime). `status: not-applicable` requires written rationale.

### Migration

Existing canonical specs (`/specs/master-plan-sequencing.json`, `hyperscaler-gates.json`, `evidence-taxonomy.json`, `stop-conditions.json`, `final-report-schema.json`) and the evidence-bundle template are retroactively registered in the capability registry with all-planned status pointing to validator-crate prerequisites. They do not require an immediate rewrite — registration is sufficient for baseline conformance.

Grace period for retroactive conformance: 30 days from 2026-05-13 (i.e., 2026-06-12). After that, the lane (once `active`) blocks PRs that add new artifacts under `applicable_paths_glob` without a capability-registry row.

**Amendment — 2026-06-29 delivery-readiness reconciliation.** The shape-neutral delivery-readiness contract is an ADR-0069 active machine-readable artifact pair, not a service/topology naming decision: `specs/delivery-readiness-reconciliation.json` defines the schema and `evidence/ralplan/delivery-readiness-current-state-20260629.json` records the honest current-state instance generated from the Ouroboros closure. Readiness predicates must remain shape-neutral; current `oya-*` / `cloud-*` names are migration observations only, never proof of Product-ready or Hyperscaler-ready status.
The current implementation artifact for that reconciliation is the planned-maturity gate at `ci/facade/feature-maturity-policy/BUCK`, `ci/facade/feature-maturity-policy/Cargo.toml`, `ci/facade/feature-maturity-policy/src/lib.rs`, and `ci/facade/feature-maturity-policy/tests/planned_maturity.rs`. These paths are current transition inventory only; the gate predicates remain shape-neutral and must not turn `oya-*` or `cloud-*` names into readiness evidence.
**Amendment — 2026-07-01 regulatory planning preservation.** The regulatory planning source-of-truth and security-validation matrix are ADR-0069 active machine-readable artifacts, with `specs/regulatory-identity-source-of-truth.json` and `specs/security-validation-pipeline-matrix.json` registered in `/registry/artifact-capabilities-registry.json` and discoverable from `/specs/root-hub-pointers.json`. The preservation packet `evidence/regulatory/regulatory-external-agent-decomposition-preservation-20260701.json` is evidence-only durability for the retired external work-source planning gate: it preserves claim ceilings, source blockers, reviewer/QA evidence, and implementation no-go state without authorizing runtime, API, certification, readiness, or deployment work.

### Forbidden-primitive remediation

The current commit landing this ADR will use `rtk git commit` as documented historical violation per the same protocol gap that was logged for commit `5880ce0`. Subsequent state transitions on the new validator crate use `grit claim --agent <id> --intent <slice>` → work → `grit done`. Scaffold-locks-oyatie ICM fallback for symbol-less doc-only edits.

### Linus-style findings closed by this decision

Per architect r17 review (`/evidence/audits/consensus/2026-05-13/architect-r17-torvalds-artifact-contract.md`):

- #1 (artifacts not in HEAD): this commit lands them.
- #2 (no ADR for v3.0.0): this ADR closes it.
- #4 (no validator crate exists): the validator crate (Phase C of this batch) closes it.
- #5 (lanes named but not wired): one lane (`lean-a-active-artifact-contract`) wired in `registry/quality/lanes.yaml` with status `planned` until validator integration completes.
- #9 (HG-RELIABILITY over-claim): inline downgrade to `documentation` evidence class.
- #10 (evidence cites `/tmp` paths): 12 consensus outputs archived under `/evidence/audits/consensus/2026-05-13/`; attestation updated.

### Linus-style findings closed in successor-IP commit `b0798b0` (per user "don't defer anything" 2026-05-13)

- #3 (graph catalog hidden under `_canonical_*` keys): CLOSED — `/specs/knowledge-graph-schema.json` reduced to pure meta-schema (199 lines); `/registry/knowledge-graph-catalog.json` NEW with 24 node types + 18 edge types + 14 invariants + 5 DRY queries as first-class catalog data.
- #6 (DRY counts contradictory): CLOSED inline — field renamed to `consumer_count_resolved_today_auto_computed` (auto-computed by validator); old name retired; the resolved-vs-listed split is documented in `_known_data_quality_gaps_per_architect_r17`.
- #7 (consumer refs mix prose + paths): CLOSED — all 15 block rows split into `consumer_refs` (resolvable paths) + `consumer_selectors` (predicate strings); `consumer_count_listed` auto-computed per row.
- #8 (9-capability contract too heavy to author manually): CLOSED — `/specs/artifact-profile-defaults.json` NEW with 7 profiles (schema / registry / template / plan-attestation / ledger / claim-matrix / evidence-bundle); 10 capability-registry rows collapsed to `artifact_profile` + sparse `capability_overrides`; validator gains `ArtifactProfile` enum + 3 new tests (12 total pass).

---

## Naming justification

```
NAME: oya-check-active-artifact-contract
JUSTIFICATION:
- microservice = check: BNF v4.1 exempt namespace per ADR-0056 (oya-check-<rule> pattern matches existing oya-check-adr-index, oya-check-architecture-cli, etc.)
- bc-tokens = active-artifact-contract: bounded context covers the 9-capability contract surface
- layer = (omitted; BNF-exempt check namespace flattens layer)
- exemptions claimed: BNF v4.1 oya-check-<rule> exempt namespace per ADR-0056
```

---

## Consequences

### Positive

- Machine-readable artifacts gain a uniform contract with mechanical-prevention via CI lane.
- Knowledge-graph substrate makes cross-artifact queries answerable (DRY violations, orphan blocks, broken refs, supersession chains, ownership coverage).
- Registry-pattern control plane scales independently of data-plane growth.
- 9-capability declaration forces honest separation of `operational` vs `planned` vs `blocked-by-foundation` at every artifact, replacing the prior pattern of asserted-but-unverified claims.
- DRY enforcement registry surfaces reusable blocks ranked by adoption, prevents reinvention.
- Hyperscaler-grade design matches AWS Config / GCP Asset Inventory / K8s CRD precedent.

### Negative

- 9-capability rows are heavy to author manually (architect r17 finding #8). Mitigation landed in commit `b0798b0`: `artifact_profile` defaults system at `/specs/artifact-profile-defaults.json` with 7 profiles; per-row authoring reduced to `artifact_profile` + sparse `capability_overrides`.
- Initial implementation is plan-stage only: the validator crate compiles but is not yet integrated with `oya-dev-cli`, the lane is `planned` not `active`, and the foundation prerequisites (cosign, trivy, audit-chain runtime) block some capability promotions to `operational`.
- Knowledge-graph storage is monolithic registries today; a graph-storage adapter (Neo4j / Memgraph / Postgres recursive CTE) is needed before the design scales past ~10k artifacts.
- Migration grace period (30 days) means most existing artifacts will not conform on day 1.

---

## Alternatives considered

### Alternative A — Embed `_capabilities` block in every artifact (rejected; per-artifact embedded metadata at scale)

This was the v2.0.0 design. Rejected because: (a) artifacts with non-object root shapes (e.g., `{rows: [...]}`) cannot have a `_capabilities` sibling at root via JSON Schema; (b) 35KB of registry boilerplate for 10 rows demonstrates the approach does not scale; (c) AWS Config / GCP Asset Inventory precedent uses a central control plane, not per-resource annotation.

### Alternative B — Schema-only declaration without a registry (rejected; no data plane)

Author the contract schema but not the capability registry. Rejected because the registry IS the control plane; without it the schema is paper. Per architect r17 finding #5, "named but not wired" lanes are not real.

### Alternative C — Use existing `registry/catalog/<crate>.yaml` pattern for all artifacts (rejected; crate-centric)

Existing per-crate catalog records only cover Rust crates; the new contract covers JSON/TOML/YAML/Cedar/SQL/etc. across the entire repo. The new registry is a sibling, not a replacement.

### Alternative D — Defer entirely until foundation lands (rejected; risk accumulation)

Wait for cosign/trivy/audit-chain runtime before authoring any of this. Rejected because the contract is honest about foundation-prerequisite gaps; deferring leaves no machine-readable mechanism to track them; drift accumulates faster than the foundation lands.

---

## References

- `/specs/active-machine-readable-artifact-contract.json` (the v3.0.0 schema)
- `/specs/knowledge-graph-schema.json` (graph substrate)
- `/registry/artifact-capabilities-registry.json` (control plane)
- `/registry/reusable-building-blocks-registry.json` (DRY enforcement)
- `/evidence/audits/consensus/2026-05-13/architect-r17-torvalds-artifact-contract.md` (architect r17 Torvalds-lens findings)
- `/registry/claim-matrix/ops-portal.json` (HG-* gate coverage; rule-iii honest-claims discipline)
- `/specs/master-plan-sequencing.json` (forbidden-primitive list; grit protocol)
- `/specs/hyperscaler-gates.json` (HG-* 10-gate registry)
- `docs/CONSTITUTION.md` (Decision principle 3: mechanical prevention over process)
- `docs/decisions/ADR-0015-architectural-flattening-target.md` (registry pattern + flat crates)
- `docs/decisions/ADR-0056-bnf-v4-1.md` (BNF v4.1 + 12-layer enum)
- `docs/decisions/ADR-0067-ops-oyatie-com-portal-foundation.md` (ops portal foundation)
- `docs/decisions/ADR-0088-microservice-foundry.md` (ADR scaffolding pattern precedent)
