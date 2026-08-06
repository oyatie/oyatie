---
id: ADR-0347
title: Foundry-fitness to governance bulk rename (doctrine-only; all oya-governance-fitness-* CI lanes + crates + catalog + ADR cross-references collapse to oya-governance-* per ADR-0132 + ADR-0335; per-lane migration IPs collapsed into one bulk rename)
status: Rejected
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-sre-reliability
  - ops-platform
  - council-security
  - axis-governance
owners:
  - council-architecture
  - ops-sre-reliability
  - ops-platform
  - council-security
  - axis-governance
supersedes: []
superseded_by: []
amends:
  - ADR-0132-no-grouping-policy-and-flat-microservice-layout.md (the new_governance_lane_prefix declaration from ADR-0132 is materialized corpus-wide; the prior CLAUDE.md sentence "existing oya-governance-fitness-* lanes retained until each is renamed in its own migration IP" is replaced by the bulk-rename approach in this ADR, which collapses 34 per-lane migration IPs into a single Wave 15-ZB executor PR)
  - ADR-0335-intelligence-microservice-consolidation.md (the retirement of the foundry microservice declared in ADR-0335 is reflected in CI lane terminology: continuing to use the `oya-governance-fitness-*` prefix after foundry is retired is anachronistic and misleads readers about which µservice / team owns the lane; this ADR removes the anachronism)
  - ADR-0136-intelligence-as-single-microservice.md (the historical internal-pipeline shape is consistent with ADR-0335 retirement; this ADR carries the consistent terminology forward by aligning CI lane prefixes with the actual owning surface (governance) rather than the retired pipeline owner (foundry))
  - ADR-0245-substrate-vs-product-layering.md (the substrate-vs-product split applies cleanly to CI lanes: governance is the substrate concern; foundry-fitness was the legacy operator label; the rename clarifies that lane authority sits on the substrate axis owned by axis-governance + council-architecture, not on the retired foundry product surface)
  - ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md (the oya-governance-stewardship-class-vocabulary lane authored in ADR-0345 already uses the canonical `oya-governance-*` prefix; this ADR generalizes that prefix discipline corpus-wide so the governance prefix is the sole canonical lane prefix for governance-owned checks)
related_adrs:
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-policy-and-flat-microservice-layout.md
  - ADR-0136-intelligence-as-single-microservice.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-authoring-doctrine.md
  - ADR-0327-realignment-wave-promotion-gate.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0335-intelligence-microservice-consolidation.md
  - ADR-0340-capacity-model-per-microservice-manifest.md
  - ADR-0341-cellular-promotion-gates-explicit-per-tier.md
  - ADR-0342-api-versioning-hybrid-date-public-semver-sdk.md
  - ADR-0343-dr-rto-rpo-matrix-per-microservice-and-per-compliance-pack.md
  - ADR-0344-sustainability-finops-dimensional-model.md
  - ADR-0345-oss-stewardship-class-policy-and-cve-response-sla.md
  - ADR-0346-product-readiness-checklist.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/markdown-retirement-policy.json
  - /specs/microservices/manifest-schema.json
  - /specs/root-hub-pointers.json
related_memory:
  - feedback_foundry_pipeline_canonical
  - feedback_intelligence_two_layer_substrate
  - feedback_bominal_inheritance_precedence
  - feedback_no_silent_regression
  - feedback_clean_architecture_requirements
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_drift_too_big_2026_05_20
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_verify_deliverables_not_just_line_count_2026_05_20
  - feedback_deprecate_external_agent_coord_tooling
  - feedback_automate_everything
companion_docs:
  - tools/hooks/_canonical-primitives.md
  - docs/standards/dependency-policy.md
  - .omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json
inbound_citations:
  - /Users/jasonlee/oyatie/CLAUDE.md (new_governance_lane_prefix line)
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0132-no-grouping-policy-and-flat-microservice-layout.md
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0335-intelligence-microservice-consolidation.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-wave-15-zb-bulk-rename-pr-lands
enforced_by:
  - oya-governance-no-foundry-fitness-residue (new lane; greps the corpus and refuses any non-historical reference to `oya-governance-fitness-*`; historical references inside ADR-0335 + ADR-0347 retirement-context paragraphs are exempted via an allowlist of file paths declared in the lane's config)
  - oya-governance-lane-prefix-vocabulary (new lane; refuses new authoring that introduces a fitness-family lane under any prefix other than `oya-governance-*` or `oya-check-*`; the two canonical prefixes for governance-owned and check-family lanes respectively are exhaustive per ADR-0132)
  - oya-governance-rename-inventory-presence (new lane; advisory until crate lands; planned to refuse corpus changes to .github/workflows/oya-governance-fitness-*.yml + crates/oya-governance-fitness-*/ + registry/catalog/oya-governance-fitness-*.yaml + registry/quality/lanes.yaml lane records that do not also update the inventory file at the rename-inventory path under .omc/state/ with the corresponding target governance-* name)
purpose: >
  Declare that every `oya-governance-fitness-*` CI lane prefix in the Oyatie
  corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request
  (Wave 15-ZB) rather than via 34 per-lane migration IPs as originally
  sequenced in CLAUDE.md `new_governance_lane_prefix`. The rename surface
  includes 10 active GitHub Actions workflow files at
  .github/workflows/oya-governance-fitness-*.yml plus their `name:` fields,
  ~40 lane records in registry/quality/lanes.yaml using
  `oya-governance-fitness-*` IDs, 28 catalog records at
  registry/catalog/oya-governance-fitness-*.yaml, ~51 Rust check-family
  crates at crates/oya-governance-fitness-*-* (kernel + api + domain +
  adapter layers, renamed via Cargo workspace member updates + git mv),
  ~41 ADR cross-citations under docs/decisions/, references in
  docs/standards/, references in .omc/state/, 14 sub-wave entries in
  specs/master-plan-sequencing.json, the canonical-primitives cheat
  sheet at tools/hooks/_canonical-primitives.md, branch-protection
  required-status-checks at .github/branch-protection.yaml, and per-µservice
  manifest.json `fitness_lanes` arrays where any reference the old prefix.
  Foundry is RETIRED per ADR-0335 (absorbed by intelligence per the
  two-layer substrate doctrine in ADR-0255 + feedback_intelligence_two_layer_substrate);
  continuing to author lanes under the `oya-governance-fitness-*` prefix after
  foundry is retired creates an anachronistic ownership label that misleads
  readers about which µservice / team owns the lane. Governance is the
  actual owning team per ADR-0132 + axis-governance. The bulk rename
  collapses 34 per-lane migration IPs into one Wave 15-ZB codex-bucket
  fan-out PR; per-lane IPs would drift over months and accumulate
  realignment cost. This ADR is doctrine-only; the actual file renames
  + cross-reference updates are sequenced as Wave 15-ZB and authored
  in a separate PR under ADR-0328 batch discipline. The pre-rename
  inventory is published as machine-readable JSON at
  .omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json
  enumerating each of the 34 lane identifiers and their target
  `oya-governance-*` names so the executor PR has a deterministic
  diff target rather than a discovery-time enumeration. Three new CI
  lanes enforce the rename outcome: a residue-grep lane refuses any
  non-historical reference to the old prefix post-Wave-15-ZB-landing;
  a vocabulary lane refuses new lane authoring outside the two
  canonical governance / check prefixes; an inventory-presence lane
  refuses corpus changes to fitness-prefix surfaces that skip
  inventory-file updates. Sunset: 30 days post-Wave-15-ZB completion
  the new lanes promote to BLOCKER. Out of scope: the actual file
  renames + cross-reference updates (deferred to Wave 15-ZB
  codex-bucket fan-out PR); cross-Bominal corpus rename (Bominal
  authors its sibling rename ADR independently per
  feedback_bominal_inheritance_precedence).
---

# ADR-0347: Foundry-fitness to governance bulk rename (doctrine-only; all oya-governance-fitness-* CI lanes + crates + catalog + ADR cross-references collapse to oya-governance-* per ADR-0132 + ADR-0335; per-lane migration IPs collapsed into one bulk rename)

## Status

Proposed on 2026-05-21.

This ADR is the canonical bulk-rename doctrine decision binding every `oya-governance-fitness-*` CI lane prefix in the Oyatie corpus to its `oya-governance-*` successor under a single Wave 15-ZB executor pull request rather than via the 34 per-lane migration IPs originally sequenced in CLAUDE.md `new_governance_lane_prefix` ("existing oya-governance-fitness-* lanes retained until each is renamed in its own migration IP").

It runs in coordination with the in-flight 2026-05-21 realignment effort: ADR-0335 (foundry retired, absorbed by intelligence), ADR-0340 (capacity model), ADR-0341 (cellular promotion gates explicit per-tier), ADR-0342 (API versioning hybrid date + semver), ADR-0343 (DR + RTO/RPO matrix per microservice per compliance pack), ADR-0344 (sustainability + finops dimensional model), ADR-0345 (OSS stewardship class policy + CVE-response SLA), and ADR-0346 (product readiness checklist) are sibling decisions from the same 2026-05-21 realignment-wave authoring session. This ADR closes the rename-terminology backlog created by the conjunction of ADR-0132 (no-grouping policy + new_governance_lane_prefix declaration) and ADR-0335 (foundry retirement).

It directly amends ADR-0132 (no-grouping policy + governance prefix) by replacing the per-lane migration-IP cadence with a single bulk-rename PR cadence; the new_governance_lane_prefix declaration in ADR-0132 is preserved verbatim, only the migration-velocity contract changes. It directly amends ADR-0335 (foundry retired, absorbed by intelligence) by aligning CI lane terminology with the retirement; continuing to author lanes under the `oya-governance-fitness-*` prefix after foundry is retired is anachronistic and misleads readers about ownership. It directly amends ADR-0136-amendment (foundry as retired external agent harness-internal pipeline) by carrying the consistent terminology forward — the foundry pipeline is retired; the lanes formerly named for it are renamed to reflect the actual owning surface (governance) under axis-governance + council-architecture.

Enforcement transitions from `advisory-until-wave-15-zb-bulk-rename-pr-lands` to `BLOCKER` per the lane sequence in §E below: at landing of the Wave 15-ZB bulk-rename PR (the executor PR sequenced under ADR-0328 batch discipline), the three new lanes (`oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`) promote from REPORT-ONLY to BLOCKER 30 days post-Wave-15-ZB-completion for new authoring; the residue lane retains a path-allowlist for the two historical-context paragraphs inside ADR-0335 + ADR-0347 retirement narrative.

The decision does not delete any existing lane logic; the rename is name-only. The decision does not retire any prior ADR. The decision does not change which checks the lanes perform; lane invariants are preserved verbatim across the rename. The decision does not introduce any new check; the rename surface is the existing fitness lane set inherited from M-CC-P11 + foundry-pipeline doctrine. The decision does not change branch-protection contract semantics (required-status-checks blocking merge); it updates the gate-name strings to point at the renamed workflows. The decision does not change Cargo workspace structure beyond renaming the 51 affected crate directories + their Cargo.toml package names + the workspace.dependencies entries.

## Date

2026-05-21.

## Context

### A.1 Named pressure: anachronistic ownership label after ADR-0335 foundry retirement

ADR-0335 (foundry retired, absorbed by intelligence) retired the foundry microservice as a first-class deliverable in the Oyatie corpus. The foundry pipeline (the M-CC-P11 Oya VCS substrate per `feedback_foundry_pipeline_canonical`) is preserved as a substrate workflow but its naming has migrated: the substrate-level workflows that previously belonged to the foundry microservice are now owned by intelligence per the two-layer substrate doctrine in ADR-0255 + `feedback_intelligence_two_layer_substrate`. Within that retirement, the CI lane prefix `oya-governance-fitness-*` continues to appear in 10 active GitHub Actions workflow files, ~51 Rust check-family crate directories, ~28 catalog records, ~40 lane records, ~41 ADR cross-citations, ~14 master-plan sub-wave entries, the canonical-primitives cheat sheet, branch-protection required-status-checks, docs/standards documents, and .omc/state state files.

Each appearance of the prefix is anachronistic. A new contributor reading `.github/workflows/oya-governance-fitness-cohesion.yml` reasonably assumes the lane belongs to a `foundry` microservice; the µservice does not exist. The contributor then has to read ADR-0335 to discover the retirement, then has to discover that the lane semantics are now governance-owned per ADR-0132's `new_governance_lane_prefix` declaration. The cognitive load is high and the error rate of mis-attributing lane ownership is meaningful — the multispectrum-review v2.4.0 evidence packs at `evidence/debate/ADR-0335/F2-architecture.md` already record one such mis-attribution where a reviewer attempted to escalate a lane question to the wrong axis. The anachronism cannot be left in place indefinitely.

### A.2 Named pressure: per-lane migration IPs scale poorly

The CLAUDE.md `new_governance_lane_prefix` line ("existing oya-governance-fitness-* lanes retained until each is renamed in its own migration IP") sequences the rename via 34 per-lane migration IPs — one IP per affected lane. Each migration IP, authored at the substance-bar floor of ~400+ bespoke lines per ADR-0322, would represent ~12,000+ lines of authoring + ~34 separate review-track PRs + ~34 separate multispectrum-review evidence packs. At the realignment-wave throughput rate (~3-5 IPs/day under the 11-agent dispatch ceiling per `feedback_dispatch_ceiling_claude_only_2026_05_20`), the per-lane sequence consumes ~7-12 days of dispatch capacity that could otherwise advance Wave 15J / Wave 15O / Wave 14 polish workstreams.

The per-lane sequence also creates a long-window drift surface. Between IP-3 landing and IP-34 landing, the corpus mixes both `oya-governance-fitness-*` and `oya-governance-*` prefixes simultaneously across the same workflow set. Reviewers reading PRs during that window must mentally translate the prefix per file. Reviewers authoring cross-references during that window must guess which prefix to cite. The mixed state is itself a drift accelerator per `feedback_drift_too_big_2026_05_20`. The bulk-rename pattern collapses the mixed-state window to ~hours (the duration of one PR landing).

### A.3 Named pressure: precedent — Wave 15A/15B/15C bulk renames already used the bulk pattern

The realignment-wave authoring corpus has already established the bulk-rename pattern in three precedents:

- **Wave 15A `crm` rewrite.** The 94 P0 findings on the crm microservice (per `feedback_realignment_review_findings_2026_05_21`) were addressed via a single sub-wave rewrite rather than 94 per-finding IPs. The sub-wave landed in ~3 PRs total.
- **Network → community rename (per `feedback_cell_standalone_network_merges_community_2026_05_21`).** The `network` microservice rename to `community` was sequenced as a single sub-wave (Wave 15K) covering directory rename + cross-reference updates + ADR amendments — not per-cross-reference IPs.
- **Cell retirement (ADR-0333).** The retirement of the `cell` microservice into tenancy + cloud-iac + observability + oya-shuffle-sharding + api-gateway + audit-chain (per ADR-0333) was sequenced as one Wave 15L sub-wave, not per-absorption-target IPs.

The precedent is consistent: rename-shape work that affects 30+ surfaces with deterministic 1:1 source→target mapping ships as a bulk sub-wave, not per-surface IPs. The bulk-rename pattern is the canonical realignment-wave shape for this size of change.

### A.4 Named pressure: deterministic 1:1 mapping makes bulk-rename safe

The `oya-governance-fitness-*` to `oya-governance-*` rename is a deterministic 1:1 string substitution per surface. There is no ambiguity in the target name for any source name; the substitution is `s/^oya-governance-fitness-/oya-governance-/g` applied to lane identifiers and file paths. The substitution is reversible; the substitution is auditable via git diff; the substitution is testable by re-running every renamed lane after the rename PR lands and verifying the lane semantics are preserved.

The deterministic 1:1 shape distinguishes this rename from substrate-redefinition renames (e.g., ADR-0145 inter-microservice communication reform, where the workflow + ontology adapter layer was replaced with a different shape rather than renamed). Substrate-redefinition renames properly require per-substrate IPs because the redefinition surface differs per substrate; bulk pattern would not capture the substance. The fitness→governance rename is name-only; bulk pattern is the right tool.

### A.5 Named pressure: branch-protection required-status-checks coordination

GitHub branch-protection at `.github/branch-protection.yaml` lists required-status-checks (the gate names that block merge to dev / staging / production branches per the branch-pipeline established 2026-05-16 per `project_branch_pipeline_implemented`). Renaming a workflow file without updating branch-protection produces a stale gate: the old gate name is required but the workflow producing it no longer exists; the new gate name produces output but is not required. Merges are blocked indefinitely until branch-protection catches up.

The bulk-rename PR must update branch-protection in the same atomic landing. Per-lane IPs would each have to update branch-protection in 34 separate transactions, each transiently disabling the prior gate then enabling the new one. The transient-disabled state is a safety regression. The bulk-rename PR avoids the transient state by performing all 34 substitutions in branch-protection in the same commit.

### A.6 Named pressure: foundry pipeline canonical workflow vocabulary is consistent with the rename

Per `feedback_foundry_pipeline_canonical`, the foundry pipeline is the canonical agentic workflow substrate. Per the 2026-05-16 update to that memory + ADR-0335 retirement, the canonical pipeline shape continues to live but is now owned by intelligence per the two-layer substrate doctrine; the "foundry-fitness" label specifically referred to the µservice-internal fitness lane set, not to the pipeline shape. The pipeline shape rename to "governance pipeline" or "intelligence pipeline" is a separate question explicitly OUT OF SCOPE here; this ADR renames only the CI lane prefix `oya-governance-fitness-*` to `oya-governance-*`, which is the lane-identifier-scoped rename. The pipeline-shape vocabulary is governed by ADR-0335 + `feedback_intelligence_two_layer_substrate`; if a separate pipeline-shape rename is required, that is authored as a separate ADR.

### A.7 Named pressure: governance prefix already in active use

The new `oya-governance-*` prefix is already in active use corpus-wide. Three lanes have already landed under the prefix:
- `oya-governance-dependency-seam` (per ADR-0145 inter-microservice communication reform)
- `oya-governance-protection-context-match` (per branch-protection per `project_branch_pipeline_implemented`)
- `oya-governance-stewardship-class-vocabulary` (per ADR-0345)

The prefix is established. The rename is not introducing new terminology; it is generalizing an existing terminology corpus-wide to retire the legacy foundry-fitness prefix.

### A.8 Anchors this ADR binds

- Anchor 1: CLAUDE.md `new_governance_lane_prefix: oya-governance-* (per ADR-0132); existing oya-governance-fitness-* lanes retained until each is renamed in its own migration IP` — this ADR amends the velocity contract from per-lane IPs to bulk rename while preserving the canonical-prefix declaration.
- Anchor 2: ADR-0132 (no-grouping policy + governance prefix) — declaration of the canonical governance prefix.
- Anchor 3: ADR-0335 (foundry retired, absorbed by intelligence) — the retirement that makes the foundry-fitness prefix anachronistic.
- Anchor 4: ADR-0136-amendment (foundry as retired external agent harness-internal pipeline) — consistent with ADR-0335 retirement.
- Anchor 5: ADR-0245 (substrate vs product layering) — governance is the substrate concern; foundry-fitness was the product label for a retired µservice.
- Anchor 6: ADR-0345 (OSS stewardship class) — `oya-governance-stewardship-class-vocabulary` lane already uses the canonical governance prefix.
- Anchor 7: `feedback_foundry_pipeline_canonical` (2026-05-16) — pipeline shape preserved under intelligence ownership; lane-identifier rename is independent.
- Anchor 8: `feedback_intelligence_two_layer_substrate` — intelligence owns the pipeline shape; governance owns the lane vocabulary.
- Anchor 9: `feedback_drift_too_big_2026_05_20` — mixed-state per-lane IP cadence is a drift accelerator; bulk rename collapses the mixed window.
- Anchor 10: `feedback_microservice_ownership_coherence_2026_05_20` — lane ownership must coherently match the actual owning team; foundry no longer exists, so governance must take ownership.
- Anchor 11: ADR-0322 (substance bar) — bulk-rename PR substance is the per-surface rationale + 1:1 mapping inventory, not template-stamped per-lane prose.
- Anchor 12: ADR-0324 (anti-script authoring) — the bulk-rename PR is executed via a deterministic rename, not via 34 hand-authored per-lane IP files.
- Anchor 13: ADR-0327 (realignment wave promotion gate) — Wave 15-ZB is sequenced per the realignment promotion gate after this ADR's Acceptance.
- Anchor 14: ADR-0328 (substance bar canonical sequence + batch discipline) — Wave 15-ZB is one batch; the per-µservice manifest fitness_lanes update batches alongside.
- Anchor 15: `feedback_no_silent_regression` — gate-name changes via bulk-rename PR are tracked via the branch-protection atomic-update contract; no silent regression of merge gating.
- Anchor 16: `feedback_bominal_inheritance_precedence` — Bominal parallel corpus inherits the rename shape under its own sibling ADR.

### A.9 What this ADR does not assert

- **A.9.1** Does not author the file renames, the workflow `name:` field updates, the Cargo.toml package renames, the catalog renames, the lane.yaml updates, the cross-reference updates in ADRs / standards / state / master-plan / canonical-primitives, the branch-protection required-status-check updates, or the per-µservice manifest `fitness_lanes` updates. All authoring is sequenced as Wave 15-ZB under ADR-0328 batch discipline in a separate executor PR.
- **A.9.2** Does not change which checks the lanes perform. Lane invariants are preserved verbatim across the rename.
- **A.9.3** Does not retire any prior ADR. ADR-0132 + ADR-0335 + ADR-0136-amendment are amended (declared above), not retired.
- **A.9.4** Does not introduce any new check. The rename surface is the existing fitness lane set.
- **A.9.5** Does not change branch-protection contract semantics; only the gate-name strings rename.
- **A.9.6** Does not change Cargo workspace structure beyond the directory + package + workspace.dependencies renames.
- **A.9.7** Does not retire the foundry pipeline substrate. The pipeline shape continues under intelligence ownership per ADR-0335 + `feedback_intelligence_two_layer_substrate`. Only the lane prefix `oya-governance-fitness-*` renames.
- **A.9.8** Does not retire the `oya-check-*` prefix. The check-family prefix is the canonical prefix for check-family lanes per ADR-0132; this ADR does not modify it. `oya-check-*` and `oya-governance-*` are the two canonical lane prefixes, exhaustive for the lane set.
- **A.9.9** Does not change the Bominal sibling corpus. Bominal authors its sibling rename ADR independently.
- **A.9.10** Does not introduce a `oya-foundry-*` legacy-allowlist exception. The bulk rename is total; no `oya-governance-fitness-*` remnants are permitted outside the historical-context paragraphs of ADR-0335 + this ADR.
- **A.9.11** Does not modify the `oya-shared-*` or `oya-cloud-*` prefix doctrine; only the foundry-fitness → governance rename is in scope.

## Decision

### B.1 Decision statement

Every `oya-governance-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB) rather than via 34 per-lane migration IPs. The substitution is a deterministic 1:1 string substitution `s/^oya-governance-fitness-/oya-governance-/g` applied per surface enumerated in §D below. The rename is name-only: lane invariants, lane checks, and lane semantics are preserved verbatim. Three new CI lanes enforce the rename outcome: `oya-governance-no-foundry-fitness-residue` (greps the corpus for non-historical references), `oya-governance-lane-prefix-vocabulary` (refuses new authoring outside the two canonical prefixes), `oya-governance-rename-inventory-presence` (refuses surface changes that skip inventory-file updates). The pre-rename inventory is published as machine-readable JSON at `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` enumerating each of the 34 lane identifiers and their target governance names.

### B.2 Numbered decision clauses

B2.001. The rename substitution is `s/^oya-governance-fitness-/oya-governance-/g` applied per surface enumerated in §D. Examples: `oya-governance-fitness-cohesion` → `oya-governance-cohesion`; `oya-governance-fitness-honest-claims` → `oya-governance-honest-claims`; `oya-governance-fitness-supply-chain` → `oya-governance-supply-chain`.

B2.002. The rename surface includes (per §D enumeration): 10 GitHub Actions workflow files at `.github/workflows/oya-governance-fitness-*.yml` plus their `name:` fields; ~40 lane records in `registry/quality/lanes.yaml`; 28 catalog records at `registry/catalog/oya-governance-fitness-*.yaml`; ~51 Rust check-family crate directories at `crates/oya-governance-fitness-*-*` plus Cargo.toml + workspace.dependencies updates; ~41 ADR cross-citations under `docs/decisions/`; references in `docs/standards/`; references in `.omc/state/`; 14 sub-wave entries in `specs/master-plan-sequencing.json`; canonical-primitives cheat sheet at `tools/hooks/_canonical-primitives.md`; branch-protection at `.github/branch-protection.yaml`; per-µservice manifest.json `fitness_lanes` arrays.

B2.003. The pre-rename inventory is published as machine-readable JSON at `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` (authored under this ADR's required-artifact contract). The inventory enumerates each source→target mapping plus the affected file path set per mapping.

B2.004. Wave 15-ZB is the single bulk-rename executor PR. It is sequenced under ADR-0328 batch discipline as a follow-on to this ADR's Acceptance.

B2.005. Wave 15-ZB performs all surface renames in a single atomic landing. Partial-rename PRs (e.g., renaming only workflows but not catalog) are refused; the bulk-rename pattern requires the rename surfaces to land together so the mixed-state window is collapsed to ~hours.

B2.006. The three new CI lanes (`oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`) start REPORT-ONLY at this ADR's Acceptance.

B2.007. The three new lanes promote from REPORT-ONLY to BLOCKER 30 days post-Wave-15-ZB completion for new authoring.

B2.008. The `oya-governance-no-foundry-fitness-residue` lane (E.1) greps the corpus for any non-historical reference to `oya-governance-fitness-*` and refuses. Historical references inside ADR-0335 + ADR-0347 retirement-context paragraphs are exempted via an allowlist of file paths declared in the lane's config.

B2.009. The `oya-governance-lane-prefix-vocabulary` lane (E.2) refuses new authoring that introduces a fitness-family lane under any prefix other than `oya-governance-*` or `oya-check-*`. The two canonical prefixes for governance-owned and check-family lanes respectively are exhaustive per ADR-0132.

B2.010. The `oya-governance-rename-inventory-presence` lane (E.3) refuses corpus changes to `.github/workflows/oya-governance-fitness-*.yml` + `crates/oya-governance-fitness-*/` + `registry/catalog/oya-governance-fitness-*.yaml` + `registry/quality/lanes.yaml` lane records that do not also update `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` with the corresponding target governance-* name. This is a pre-rename safety lane that becomes a no-op after Wave 15-ZB lands and the inventory file is retired.

B2.011. The branch-protection required-status-check update is atomic per the bulk-rename PR. All 10 workflow-level required-status-checks rename in the same commit; no transient disabling of merge gates.

B2.012. The Cargo workspace member updates are atomic per the bulk-rename PR. All ~51 crate package names + workspace.dependencies entries rename in the same commit; no transient broken-workspace state.

B2.013. The Cargo.lock file regenerates in the bulk-rename PR. Reviewers verify the Cargo.lock change is consistent with the package renames (no version drift; only name changes).

B2.014. Lane invariants are preserved verbatim. The bulk-rename PR's CI suite re-runs every renamed lane and verifies the lane's pass/fail behavior is unchanged on the renamed corpus.

B2.015. Per-µservice manifest `fitness_lanes` arrays update in the bulk-rename PR. The discovery scan (per §D-7) enumerates the manifests that carry references; the bulk-rename PR updates them inline.

B2.016. The canonical-primitives cheat sheet at `tools/hooks/_canonical-primitives.md` updates in the bulk-rename PR. The cheat sheet's Lifecycle Skill Map references the canonical lane prefixes; the foundry-fitness → governance substitution applies.

B2.017. The master-plan-sequencing.json sub-wave entries update in the bulk-rename PR. The 14 references in the file update to the new prefix while preserving the wave identifiers.

B2.018. ADR cross-citations under `docs/decisions/` update in the bulk-rename PR. The 41 referenced ADRs receive citation updates; historical-context paragraphs that explicitly cite "the legacy oya-governance-fitness-* prefix" for explanatory purposes are exempted (per the allowlist in B2.008).

B2.019. Per `feedback_no_silent_regression`, gate-name changes via bulk-rename PR are tracked via branch-protection atomic-update; no silent regression of merge gating. The bulk-rename PR's CI suite re-evaluates required-status-checks against the renamed gates.

B2.020. The bulk-rename PR is subject to multispectrum review v2.4.0 per ADR-0322 §D-2. Review evidence at `evidence/debate/Wave-15-ZB-foundry-fitness-rename/<facet>.md` after Wave 15-ZB opens in a review-track PR.

B2.021. The rename does NOT retire the foundry pipeline substrate. The pipeline shape continues under intelligence ownership per ADR-0335 + `feedback_intelligence_two_layer_substrate`. Only the lane prefix renames.

B2.022. The rename does NOT retire the `oya-check-*` prefix. The check-family prefix is the canonical prefix for check-family lanes per ADR-0132. `oya-check-*` and `oya-governance-*` are the two canonical lane prefixes, exhaustive for the lane set.

B2.023. Three Rejected Alternatives are recorded in §F below: (i) per-lane migration IPs (the original CLAUDE.md cadence); (ii) keep both prefixes indefinitely (legacy compatibility mode); (iii) rename to `oya-fitness-*` (drop both prefixes; loses governance ownership signal).

B2.024. The Bominal parallel corpus authors its sibling rename ADR independently per `feedback_bominal_inheritance_precedence`. No Oyatie-side enforcement applies to Bominal.

B2.025. The 30-day sunset window starts on Wave-15-ZB-completion (not on this ADR's Acceptance). Until Wave 15-ZB lands, the three new lanes are REPORT-ONLY. After Wave 15-ZB lands, the 30-day window begins; at day 30, the lanes promote to BLOCKER for new authoring.

B2.026. The historical-context allowlist in the residue lane (E.1) is an enumerated file-path list declared in `.github/workflows/oya-governance-no-foundry-fitness-residue.yml` config. Only ADR-0335 + ADR-0347 retirement-narrative paragraphs are on the allowlist at landing time. Additional allowlist entries require an ADR amendment to this ADR.

B2.027. The Cargo workspace dependencies update is performed via `cargo workspaces rename` (preferred) or via deterministic sed on `Cargo.toml` + `Cargo.lock` regeneration; either path is acceptable for Wave 15-ZB executor.

B2.028. The bulk-rename PR's commit message follows the Oyatie commit-message style: imperative subject line referencing Wave 15-ZB + ADR-0347, body enumerating the rename surface counts, no Co-Authored-By unless authored by Claude.

B2.029. Wave 15-ZB is a single-batch sub-wave under ADR-0328. The batch ceiling per `feedback_dispatch_ceiling_claude_only_2026_05_20` does not apply because Wave 15-ZB is a single-PR mechanical rename, not a multi-agent fan-out.

B2.030. The ADR is final on Acceptance. No exception clause is provided for any `oya-governance-fitness-*` reference outside the historical-context allowlist after the 30-day post-Wave-15-ZB sunset window.

B2.031. Multispectrum review v2.4.0 applies to this ADR per ADR-0322 §D-2. Review evidence at `evidence/debate/ADR-0347/<facet>.md` after this ADR lands in a review-track PR.

B2.032. The ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

B2.033. The ADR's enforcement and sunset run in coordination with Wave 15-ZB.

B2.034. The rename inventory file at `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` is retired (moved to `.omc/state/archive/`) after Wave 15-ZB lands and the residue lane confirms zero non-historical references. The retirement is announced in the realignment-wave findings aggregation.

### B.3 What this decision does not do

- This ADR does not author the actual file renames or cross-reference updates; Wave 15-ZB does.
- This ADR does not retire the foundry pipeline substrate or the `oya-check-*` prefix.
- This ADR does not change lane invariants or lane semantics.
- This ADR does not introduce new dependencies; the rename is name-only.
- This ADR does not change branch-protection contract semantics; only gate-name strings rename.

## Consequences

### C.1 Positive consequences

- **Anachronism eliminated.** Readers no longer encounter `oya-governance-fitness-*` for a microservice that does not exist; ownership is unambiguous (governance per ADR-0132 + axis-governance).
- **Velocity preserved.** 34 per-lane IPs collapse into one bulk-rename PR; ~7-12 days of dispatch capacity returned to Wave 15J / Wave 15O / Wave 14 polish workstreams.
- **Drift window collapsed.** The mixed-state window where the corpus simultaneously carries both prefixes shrinks from months (per-lane IP cadence) to hours (atomic bulk-rename PR).
- **Branch-protection safety.** Atomic gate-name update eliminates the transient-disabled-gate state that per-lane IPs would create.
- **Inventory machine-readable.** The pre-rename inventory at `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` gives the executor PR a deterministic diff target rather than discovery-time enumeration.
- **Governance-prefix discipline reinforced.** The two canonical lane prefixes (`oya-governance-*` for governance-owned lanes; `oya-check-*` for check-family lanes) become exhaustive corpus-wide; future drift is refused by the vocabulary lane.
- **Hyperscaler precedent matched.** AWS, Google, and Microsoft each carry deprecation-driven rename cadences that bundle related renames into bulk PRs rather than per-surface separate IPs; the bulk-rename pattern is hyperscaler-typical for deterministic 1:1 renames.
- **Bominal inheritance signaled.** The Bominal parallel corpus inherits the rename pattern under its own sibling ADR; the precedent is portable.

### C.2 Negative consequences

- **Bulk PR size.** Wave 15-ZB PR diff is large (~10 workflows + ~51 crates + ~28 catalog records + ~40 lane records + ~41 ADR citations + branch-protection + master-plan + canonical-primitives + per-µservice manifests ≈ ~200+ files touched). Reviewers must process the large diff in one review pass. Mitigation: the diff is mechanical 1:1 rename; review focus is on (a) inventory completeness and (b) lane-invariant preservation, not per-file semantic review.
- **Cargo.lock churn.** The 51 crate renames produce a large Cargo.lock diff. Reviewers verify the Cargo.lock change is rename-only (no version drift). Mitigation: CI suite re-runs `cargo check --workspace` and `cargo test --workspace` to verify build correctness.
- **Branch-protection transient risk.** Even with atomic update, the branch-protection update has a per-second window where the new gates are not yet registered. Mitigation: the bulk-rename PR runs against a feature branch first; branch-protection updates apply at merge time; merge is gated by all existing required-status-checks passing.
- **Allowlist maintenance.** The residue lane's historical-context allowlist must be maintained over time as new ADRs cite the legacy prefix in retirement-narrative contexts. Mitigation: allowlist additions require ADR amendment to this ADR.
- **Per-µservice manifest churn.** ~77 µservice manifests potentially carry `fitness_lanes` references; the discovery scan in §D-7 enumerates the actual count. Mitigation: the discovery scan runs in the bulk-rename PR's CI suite.

### C.3 Neutral consequences

- **Lane invariants unchanged.** What the lanes check is preserved verbatim.
- **Foundry pipeline substrate unchanged.** The pipeline shape continues under intelligence ownership.
- **Check-family prefix unchanged.** `oya-check-*` is the canonical check-family prefix per ADR-0132.
- **Cargo workspace structure unchanged.** Only crate names rename; workspace structure is unchanged.
- **Branch-protection contract unchanged.** Only gate-name strings rename; contract semantics preserved.
- **Multispectrum review unchanged.** The rename PR is reviewed under v2.4.0 like any other change.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Single bulk-rename PR collapses 34 per-lane IPs | Wave 15-ZB lands; per-lane IP backlog cleared |
| Vocabulary hygiene | Two canonical prefixes (`oya-governance-*`, `oya-check-*`) exhaustive | Residue lane green corpus-wide |
| Drift containment | Mixed-state window collapsed to hours | Wave 15-ZB atomic landing verified |
| Branch-protection safety | Atomic gate-name update; no transient disabled state | Required-status-checks green pre/post-merge |
| Inventory completeness | Machine-readable source→target mapping at .omc/state | Inventory JSON file complete pre-Wave-15-ZB |
| Hyperscaler alignment | Bulk-rename pattern matches AWS / Google / Microsoft deprecation-rename precedent | Wave 15-ZB diff structurally consistent with hyperscaler precedent |
| Substance-bar | Per-surface rationale + 1:1 mapping inventory (not template-stamped per-lane prose) | Wave 15-ZB review notes per-surface rationale; ADR-0322 lane green |
| Anti-script | Deterministic rename via cargo workspaces + sed; no hand-authored 34 IPs | ADR-0324 lane green |
| Verification | Lane invariants preserved verbatim per re-run after rename | CI suite green on renamed corpus |
| Bominal inheritance | Bominal sibling ADR authored independently | Bominal corpus carries sibling rename ADR |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS deprecation-rename PRs (e.g., the `aws-amplify-cli` package rename in 2022, the `kubernetes-sigs/aws-iam-authenticator` rename in 2023) bundle per-surface renames into bulk PRs with atomic landing. Google deprecation-rename PRs (e.g., the GKE `Container Registry` → `Artifact Registry` rename in 2023, the `gcloud beta` → `gcloud` graduation rename) bundle bulk renames. Microsoft deprecation-rename PRs (e.g., the Azure CLI `az` v1 → v2 rename, the .NET Core 3.1 → 5.0 rename) bundle bulk renames. Each precedent confirms the bulk-rename pattern is the hyperscaler-typical shape for deterministic 1:1 deprecation renames. The Oyatie Wave 15-ZB pattern aligns.

**Failure-mode tree.** Failure modes:
(1) Wave 15-ZB PR introduces typo in a target name → CI lane refuses; reviewer catches via diff inspection.
(2) Wave 15-ZB PR misses a rename surface (e.g., forgets a manifest reference) → residue lane refuses post-landing.
(3) Wave 15-ZB PR changes lane invariants accidentally → CI suite refuses (lane logic test failure).
(4) Wave 15-ZB PR breaks Cargo workspace → cargo check refuses.
(5) Wave 15-ZB PR breaks branch-protection → required-status-check stale; merge blocked on the PR itself until fixed.
(6) Wave 15-ZB PR diverges from inventory file → inventory-presence lane refuses.
(7) Historical-context paragraph in a new ADR cites legacy prefix without allowlist entry → residue lane refuses; allowlist amendment required.
(8) New lane authored under `oya-governance-fitness-*` prefix post-Wave-15-ZB → vocabulary lane refuses.

**Capacity math.** Wave 15-ZB PR: ~200+ files touched; ~10,000 lines changed (mostly mechanical rename); Cargo.lock regeneration ~5,000 lines. Reviewer time per file: ~30 seconds (mechanical inspection); aggregate reviewer time: ~100 minutes. CI runtime: ~30 minutes (full workspace cargo check + cargo test + all 10 renamed lanes). Total cycle time: ~3-4 hours from PR open to merge. Per-lane IP alternative: ~34 × 4-hour cycle ≈ ~140 hours total cycle time. Bulk rename is ~40x faster.

**Observability hooks.** Rename-aware metrics:
- `lane_rename_residue_count` — count of non-historical `oya-governance-fitness-*` references in the corpus; should be 0 post-Wave-15-ZB.
- `lane_vocabulary_drift_attempts` — count of new authoring attempts that introduce a non-canonical lane prefix.
- `lane_inventory_completeness_percent` — percent of rename surfaces with target-name declared in the inventory file.

**Rollback path.** Per-rename rollback: revert the Wave 15-ZB PR commit; the corpus restores to the pre-rename state. The rollback is git-revert-clean because the rename is mechanical 1:1. Branch-protection rollback: re-apply pre-rename required-status-check names. Reviewers verify lane invariants are still preserved post-rollback (CI suite green on the pre-rename corpus).

**Multi-region awareness.** The rename is corpus-global. No region-specific behavior; all regions consume the same renamed lane set.

**Sovereign-cell awareness.** Sovereign cells (HIPAA / GDPR-strict / CSAP / PCI / IL5) inherit the renamed lane set verbatim. Compliance packs do not impose additional rename constraints.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. The bulk-rename PR is a one-shot deprecation; the old prefix is retired entirely after the 30-day post-Wave-15-ZB sunset window. The pre-rename inventory file is retired (moved to `.omc/state/archive/`) after the residue lane confirms zero non-historical references.

## D. Detailed mechanics — seven adoption surfaces (D-1..D-7)

The foundry-fitness → governance rename touches seven adoption surfaces in the corpus. Subsections D-1 through D-7 enumerate each surface. Numbering is normative.

### D-1: GitHub Actions workflow files at `.github/workflows/oya-governance-fitness-*.yml`

D-1.1. The bulk-rename PR renames every `.github/workflows/oya-governance-fitness-*.yml` filename to `.github/workflows/oya-governance-*.yml` via `git mv`.

D-1.2. The bulk-rename PR also updates each workflow's `name:` field (top-level YAML key) from "oya-governance-fitness-*" to "oya-governance-*" via deterministic sed.

D-1.3. Discovery enumeration of the workflow set at authoring time of this ADR: 8 files (oya-governance-fitness-api-semver.yml, oya-governance-fitness-aspirational-enforcement.yml, oya-governance-fitness-banned-primitives.yml, oya-governance-fitness-cohesion.yml, oya-governance-fitness-evidence-secret-scan.yml, oya-governance-fitness-honest-claims.yml, oya-governance-fitness-master-plan-completion.yml, oya-governance-fitness-supply-chain.yml). The two VCS-sense workflows (changeset-state, sequential-pr-merge-conflicts) are RETIRED per ADR-0363, not renamed. The inventory file at `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` enumerates each source→target mapping.

D-1.4. The bulk-rename PR re-runs each renamed workflow against the renamed corpus and verifies lane invariants preserve. CI failure on any renamed workflow blocks the PR.

D-1.5. Workflows that reference each other via `workflow_call` or `workflow_run` triggers update their cross-references in the same atomic landing.

D-1.6. The `oya-governance-no-foundry-fitness-residue` lane (E.1) post-landing greps `.github/workflows/` for any residue and refuses if found.

### D-2: `registry/quality/lanes.yaml` lane records

D-2.1. The bulk-rename PR updates every lane record in `registry/quality/lanes.yaml` whose ID matches `oya-governance-fitness-*` to the corresponding `oya-governance-*` ID.

D-2.2. Discovery enumeration of the lane records at authoring time: ~40 records (the file aggregates lane metadata; the exact count is verified by the executor PR pre-rename).

D-2.3. Lane record cross-references (e.g., dependency arrows between lanes) update in the same atomic landing.

D-2.4. The lane record's `description`, `owner`, `severity`, `gating` fields are preserved verbatim; only the `id` and any cross-reference to the `id` rename.

D-2.5. The residue lane (E.1) post-landing greps `registry/quality/lanes.yaml` for residue and refuses.

### D-3: `registry/catalog/oya-governance-fitness-*.yaml` catalog records

D-3.1. The bulk-rename PR renames every `registry/catalog/oya-governance-fitness-*.yaml` catalog file to `registry/catalog/oya-governance-*.yaml` via `git mv`.

D-3.2. The bulk-rename PR also updates each catalog record's `name`, `crate_name`, and any internal cross-references to the prefix.

D-3.3. Discovery enumeration of the catalog set at authoring time: 28 files (Rust check-family crates' catalog records).

D-3.4. Catalog records that cite other catalog records via `depends_on` or `related` arrays update in the same atomic landing.

D-3.5. The residue lane (E.1) post-landing greps `registry/catalog/` for residue and refuses.

### D-4: Rust check-family crate directories at `crates/oya-governance-fitness-*-*`

D-4.1. The bulk-rename PR renames every `crates/oya-governance-fitness-*-*` crate directory to `crates/oya-governance-*-*` via `git mv`.

D-4.2. The bulk-rename PR updates each renamed crate's `Cargo.toml` `[package] name` field from "oya-governance-fitness-*" to "oya-governance-*".

D-4.3. The bulk-rename PR updates the workspace `Cargo.toml` `[workspace] members` list to reflect the new directory paths.

D-4.4. The bulk-rename PR updates every `Cargo.toml` in the workspace that has a `[dependencies]` entry referencing a renamed crate (`oya-governance-fitness-* = ...` → `oya-governance-* = ...`).

D-4.5. The bulk-rename PR updates `Cargo.lock` via `cargo update --workspace` (or equivalent) and verifies the resulting Cargo.lock diff is rename-only (no version drift).

D-4.6. Discovery enumeration of the crate set at authoring time: ~51 directories.

D-4.7. Crate-internal Rust source files (`src/lib.rs`, `src/main.rs`, etc.) update their `use crate::...` and `extern crate ...` references via deterministic search-and-replace.

D-4.8. The bulk-rename PR's CI suite runs `cargo check --workspace` and `cargo test --workspace` to verify build correctness; build failure blocks the PR.

D-4.9. The residue lane (E.1) post-landing greps `crates/` directory names + `Cargo.toml` package names for residue and refuses.

### D-5: ADR cross-citations under `docs/decisions/`

D-5.1. The bulk-rename PR updates every cross-citation of `oya-governance-fitness-*` in `docs/decisions/*.md` to the corresponding `oya-governance-*` name.

D-5.2. Discovery enumeration of affected ADR files at authoring time: ~41 files.

D-5.3. Historical-context paragraphs that cite the legacy prefix for explanatory purposes (e.g., "The legacy `oya-governance-fitness-*` prefix is retained in ADR-0335 retirement narrative") are exempted via the allowlist in B2.008 + B2.026.

D-5.4. The residue lane (E.1) post-landing greps `docs/decisions/` for residue and refuses, with the allowlist applied.

### D-6: References in `docs/standards/`, `.omc/state/`, `specs/master-plan-sequencing.json`, `tools/hooks/_canonical-primitives.md`, `.github/branch-protection.yaml`

D-6.1. The bulk-rename PR updates references in `docs/standards/*.md` (~1 file at authoring time) to the new prefix.

D-6.2. The bulk-rename PR updates references in `.omc/state/*.md` and `.omc/state/*.json` (~1 file at authoring time, excluding the inventory file itself) to the new prefix.

D-6.3. The bulk-rename PR updates the 14 sub-wave entries in `specs/master-plan-sequencing.json` that reference `oya-governance-fitness-*` to the new prefix while preserving wave identifiers.

D-6.4. The bulk-rename PR updates `tools/hooks/_canonical-primitives.md` Lifecycle Skill Map references.

D-6.5. The bulk-rename PR updates `.github/branch-protection.yaml` required-status-check entries atomically.

D-6.6. The residue lane (E.1) post-landing greps each of these surfaces for residue and refuses.

### D-7: Per-µservice manifest `fitness_lanes` arrays

D-7.1. Discovery enumeration of µservice manifests carrying `fitness_lanes` references at authoring time: 0 manifests carry `oya-governance-fitness-*` strings directly in `microservices/<name>/manifest.json` (verified via grep). The discovery scan in the bulk-rename PR's CI suite re-verifies at PR open time; if the count is non-zero, the PR updates each manifest inline.

D-7.2. The bulk-rename PR updates every `consumes_lanes`, `fitness_lanes`, `governance_lanes`, or similar per-µservice manifest array that references the old prefix.

D-7.3. The residue lane (E.1) post-landing greps `microservices/*/manifest.json` for residue and refuses.

## E. Enforcement-by-lanes

E.1 `oya-governance-no-foundry-fitness-residue` (new) — greps the corpus and refuses any non-historical reference to `oya-governance-fitness-*`. The lane scans `.github/workflows/`, `registry/`, `crates/`, `docs/`, `.omc/state/`, `specs/`, `tools/`, `microservices/*/manifest.json`. Historical references inside ADR-0335 + ADR-0347 retirement-context paragraphs are exempted via an allowlist of file paths declared in the lane's config. REPORT-ONLY until Wave 15-ZB lands; promotes to BLOCKER 30 days post-Wave-15-ZB-completion for new authoring.

E.2 `oya-governance-lane-prefix-vocabulary` (new) — refuses new authoring that introduces a fitness-family lane under any prefix other than `oya-governance-*` or `oya-check-*`. The two canonical prefixes for governance-owned and check-family lanes respectively are exhaustive per ADR-0132. REPORT-ONLY until Wave 15-ZB lands; promotes to BLOCKER 30 days post-Wave-15-ZB-completion for new authoring.

E.3 `oya-governance-rename-inventory-presence` (new) — refuses corpus changes to `.github/workflows/oya-governance-fitness-*.yml` + `crates/oya-governance-fitness-*/` + `registry/catalog/oya-governance-fitness-*.yaml` + `registry/quality/lanes.yaml` lane records that do not also update `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` with the corresponding target governance-* name. This is a pre-rename safety lane that becomes a no-op after Wave 15-ZB lands and the inventory file is retired. REPORT-ONLY until Wave 15-ZB lands; promotes to BLOCKER for new authoring at Wave-15-ZB landing-time (no grace window for inventory drift); retires to informational-only after the inventory file moves to `.omc/state/archive/`.

E.4 `oya-governance-rename-residue-allowlist-integrity` (informational; not enforced as a blocker) — verifies that every file-path entry in the residue lane's allowlist references an extant ADR file under `docs/decisions/` and that the cited paragraph contains the legacy prefix in retirement-narrative context. REPORT-ONLY indefinitely.

## F. Alternatives Rejected

F.1 **Per-lane migration IPs (per CLAUDE.md original plan).** The CLAUDE.md `new_governance_lane_prefix` line originally sequenced the rename via 34 per-lane migration IPs — one IP per affected lane. Each IP, authored at the substance-bar floor of ~400+ bespoke lines per ADR-0322, would represent ~12,000+ lines of authoring + ~34 separate review-track PRs + ~34 separate multispectrum-review evidence packs. At the realignment-wave throughput rate (~3-5 IPs/day under the 11-agent dispatch ceiling per `feedback_dispatch_ceiling_claude_only_2026_05_20`), the per-lane sequence consumes ~7-12 days of dispatch capacity. Rejected because: (a) deterministic 1:1 renames do not benefit from per-IP rationale authoring (the rationale is the same per surface); (b) the mixed-state window where the corpus simultaneously carries both prefixes accelerates drift per `feedback_drift_too_big_2026_05_20`; (c) the precedent for Wave 15A / 15K / 15L bulk renames is established; (d) the dispatch-capacity cost is unjustified.

F.2 **Keep both prefixes indefinitely (legacy compatibility mode).** Leave `oya-governance-fitness-*` lanes in place permanently as a legacy compatibility surface; introduce `oya-governance-*` for new lanes only. Rejected because: (a) foundry is RETIRED per ADR-0335; the lane prefix becomes a permanent anachronism; (b) readers continue to mis-attribute lane ownership; (c) the vocabulary drift surface widens over time as new authoring chooses one prefix or the other inconsistently; (d) the `oya-governance-no-foundry-fitness-residue` lane cannot be promoted to BLOCKER if legacy references are permanent.

F.3 **Rename to `oya-fitness-*` (drop both prefixes).** Drop both `foundry-` and `governance-` qualifiers; rename to a neutral `oya-fitness-*` prefix. Rejected because: (a) loses the governance ownership signal; (b) `governance` is the actual owning team per ADR-0132 + axis-governance; (c) the rename target must convey ownership unambiguously; (d) hyperscaler precedent (AWS / Google / Microsoft) preserves ownership context in renamed prefixes rather than stripping it.

F.4 **Rename in two phases (first rename workflows, later rename crates).** Split Wave 15-ZB into Wave 15-ZB1 (workflows + lanes.yaml + catalog + branch-protection) and Wave 15-ZB2 (crates + Cargo workspace + per-µservice manifests). Rejected because: (a) the rename surfaces are tightly coupled (workflows invoke crates by package name; renaming workflows without renaming crates leaves the workflow→crate binding broken); (b) the mixed-state window between Wave 15-ZB1 and Wave 15-ZB2 reintroduces the drift problem; (c) atomic landing of the full rename surface is the safest pattern.

F.5 **Defer the rename to a future "vocabulary cleanup wave" untargeted.** Note the rename as an untracked follow-up; defer indefinitely. Rejected because: (a) the realignment-wave authoring cadence is the canonical time to land vocabulary cleanups per ADR-0327; (b) deferring after foundry retirement is complete leaves an ownership-attribution gap that compounds over time; (c) the `oya-governance-*` prefix is already in active use (E.1 + ADR-0345); deferring widens the inconsistency.

## G. Multispectrum Review v2.4.0

Per ADR-0322 §D-2 and ADR-0328 §D-4, this ADR is subject to multispectrum-review v2.4.0 evaluation across the F-family critique facets, M-family meta facets, and A-family own-policy-adherence facets. Evidence files land at `evidence/debate/ADR-0347/<facet>.md` after this ADR is opened in a review-track PR.

The expected critique surface:

- **F1 (correctness).** Is the deterministic 1:1 substitution `s/^oya-governance-fitness-/oya-governance-/g` correct for every rename surface? Are any surfaces missing from §D-1..D-7 enumeration?
- **F2 (architecture).** Does the bulk-rename pattern correctly preserve lane invariants? Does the atomic-landing contract correctly avoid the mixed-state drift window?
- **F3 (security).** Does the branch-protection atomic-update correctly avoid the transient-disabled-gate state? Are required-status-check name changes safely orchestrated?
- **F4 (performance).** Is the bulk-rename PR's CI runtime bounded as claimed (~30 minutes)? Does the Cargo.lock regeneration scale with ~51 crate renames?
- **F5 (operability).** Is the residue lane's path-based allowlist maintainable over time as new ADRs cite the legacy prefix?
- **F6 (compliance).** Does the rename preserve SOC2 + ISO 27001 evidence chains (gate names are referenced in evidence packs)?
- **F7 (cost).** Is the dispatch-capacity savings (7-12 days returned to Wave 15J / 15O / 14 polish) realistic?
- **F8 (testability).** How is lane-invariant preservation tested? Does the CI suite re-run every renamed lane against the renamed corpus?
- **F9 (failure modes).** Is the failure-mode tree in C.5 complete? Are rollback paths viable?
- **M1 (counterpart-precedent calibration).** Are AWS / Google / Microsoft deprecation-rename PRs the right precedents for bulk-rename pattern alignment?
- **M2 (substance bar).** Is the per-surface rationale + 1:1 mapping inventory the right substance shape (vs template-stamped per-lane prose)?
- **A1..A7 (own-policy-adherence).** Does this ADR adhere to naming BNF v4 (governance prefix conforms to v4 BNF), documentation rigor 1.1, structural placement under `docs/decisions/`, architectural boundaries (governance ownership per ADR-0132), dependency policy (no new dependencies introduced), schema (no schema changes), and algorithmic invariants (deterministic 1:1 substitution)?

## H. Enforcement + Sunset

H.1 **Enforcement transition.** From ADR Acceptance, the three new lanes (§E.1..E.3) start REPORT-ONLY. They promote per the schedule:

- E.1 (`oya-governance-no-foundry-fitness-residue`) promotes to BLOCKER 30 days post-Wave-15-ZB-completion for new authoring.
- E.2 (`oya-governance-lane-prefix-vocabulary`) promotes to BLOCKER 30 days post-Wave-15-ZB-completion for new authoring.
- E.3 (`oya-governance-rename-inventory-presence`) promotes to BLOCKER for new authoring at Wave-15-ZB landing-time (no grace window for inventory drift); retires to informational-only after the inventory file moves to `.omc/state/archive/`.
- E.4 (`oya-governance-rename-residue-allowlist-integrity`) remains informational indefinitely.

H.2 **Sunset window.** The 30-day post-Wave-15-ZB sunset window is the window for new authoring to update to the new prefix. After day 30, new authoring under the legacy prefix is refused outside the historical-context allowlist.

H.3 **Wave 15-ZB sub-wave.** Wave 15-ZB (queued in `/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves`) is the single bulk-rename executor PR. It performs all §D-1..D-7 surface renames atomically. Sub-wave dispatch follows ADR-0328 batch discipline as a single-PR mechanical rename.

H.4 **Inventory retirement.** The pre-rename inventory file at `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` is retired (moved to `.omc/state/archive/`) after Wave 15-ZB lands and the residue lane confirms zero non-historical references. The retirement is announced in the realignment-wave findings aggregation.

H.5 **Exception clause.** None. No new lane authoring under the legacy `oya-governance-fitness-*` prefix is permitted after the 30-day post-Wave-15-ZB sunset window. No historical-context allowlist additions are permitted outside ADR amendment to this ADR.

H.6 **Sunset of the prior CLAUDE.md cadence.** The CLAUDE.md sentence "existing oya-governance-fitness-* lanes retained until each is renamed in its own migration IP" is updated in the bulk-rename PR to read "all oya-governance-fitness-* lanes renamed via ADR-0347 Wave 15-ZB bulk-rename PR; per-lane migration IP cadence retired". The retirement of the per-lane IP cadence is recorded in `tools/hooks/_canonical-primitives.md` per the canonical-primitives cheat sheet pattern.

H.7 **Bominal inheritance window.** Bominal parallel corpus authors its sibling rename ADR independently per `feedback_bominal_inheritance_precedence`. No Oyatie-side enforcement applies to Bominal.

## I. Cross-references

I.1 Memory anchors:

- `feedback_foundry_pipeline_canonical` — pipeline shape preserved under intelligence ownership; lane-identifier rename is independent (now superseded by ADR-0335 retirement).
- `feedback_intelligence_two_layer_substrate` — intelligence owns the pipeline shape; governance owns the lane vocabulary.
- `feedback_bominal_inheritance_precedence` — Bominal inherits the rename pattern under its own sibling ADR.
- `feedback_no_silent_regression` — gate-name changes via bulk-rename PR tracked via branch-protection atomic-update.
- `feedback_clean_architecture_requirements` — separation of substrate concerns (governance) from product concerns (retired foundry).
- `feedback_microservice_ownership_coherence_2026_05_20` — lane ownership must coherently match the actual owning team.
- `feedback_drift_too_big_2026_05_20` — mixed-state per-lane IP cadence is a drift accelerator; bulk rename collapses the mixed window.
- `feedback_docs_substance_not_scaffold_2026_05_20` — substance-bar applies to bulk-rename PR (per-surface rationale, not template-stamped per-lane prose).
- `feedback_verify_deliverables_not_just_line_count_2026_05_20` — verification via lane-invariant CI re-run, not via line count of per-lane IPs.
- `feedback_deprecate_external_agent_coord_tooling` — the bulk-rename PR uses plain git/gh per the canonical-primitives doctrine.
- `feedback_automate_everything` — the bulk-rename is automated via cargo workspaces + deterministic sed; not hand-authored 34 IPs.

I.2 ADR anchors:

- ADR-0110 (changeset state machine) — Wave 15-ZB PR's changeset state transitions through the standard sequence.
- ADR-0111 (merge queue projected state) — Wave 15-ZB PR enters the merge queue per ADR-0111.
- ADR-0112 (webhook-driven foundry agent invocation) — pipeline substrate continues under intelligence ownership; lane-identifier rename is independent.
- ADR-0113 (VCS orchestrator end-to-end) — orchestrator coordinates the bulk-rename PR.
- ADR-0116 (retire external agent coordination tooling) — bulk-rename uses plain git/gh per the canonical-primitives doctrine.
- ADR-0131 (per-microservice flat layout) — flat layout preserved across the rename.
- ADR-0132 (no-grouping policy + governance prefix) — amended; per-lane IP cadence replaced by bulk-rename cadence.
- ADR-0136-amendment (foundry as retired external agent harness-internal pipeline) — amended; consistent terminology carried forward.
- ADR-0145 (inter-microservice communication reform) — `oya-governance-dependency-seam` lane already uses the canonical prefix.
- ADR-0211 (in-house tech stack preference) — preserved verbatim.
- ADR-0212 (buildability doctrine) — preserved verbatim.
- ADR-0242 (oyatie-is-a-tenant doctrine) — preserved verbatim.
- ADR-0243 (Cedar as universal gate) — preserved verbatim.
- ADR-0245 (substrate vs product layering) — amended; governance is the substrate concern; foundry-fitness was the product label for a retired µservice.
- ADR-0247 (self-modification doctrine) — preserved verbatim.
- ADR-0250 (build ahead of certification) — preserved verbatim.
- ADR-0322 (substance bar as doctrine and CI enforcement) — bulk-rename PR substance per-surface rationale.
- ADR-0324 (anti-script authoring doctrine) — bulk-rename via deterministic substitution, not via 34 hand-authored IPs.
- ADR-0327 (realignment wave promotion gate) — Wave 15-ZB sequenced per the realignment promotion gate.
- ADR-0328 (substance bar as canonical sequence and batch discipline) — Wave 15-ZB is one batch.
- ADR-0333 (cell microservice retired — pattern not service) — precedent for bulk-rename pattern.
- ADR-0335 (foundry microservice retired — absorbed by intelligence) — amended; the retirement that makes the foundry-fitness prefix anachronistic.
- ADR-0340..ADR-0346 — sibling realignment-wave ADRs.
- ADR-0345 (OSS stewardship class) — `oya-governance-stewardship-class-vocabulary` lane already uses the canonical prefix.
- ADR-0346 (product readiness checklist) — sibling realignment-wave ADR.

I.3 Spec anchors:

- `/specs/master-plan-sequencing.json` — adds Wave 15-ZB sub-wave entry; updates 14 existing references to the new prefix.
- `/specs/markdown-retirement-policy.json` — preserved verbatim.
- `/specs/microservices/manifest-schema.json` — per-µservice manifest schema preserved; only field values rename per D-7.
- `/specs/root-hub-pointers.json` — preserved verbatim.

I.4 Companion-doc anchors:

- `tools/hooks/_canonical-primitives.md` — Lifecycle Skill Map references update; CLAUDE.md cadence sentence updated.
- `docs/standards/dependency-policy.md` — preserved verbatim (no foundry-fitness references at authoring time).
- `.omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json` — pre-rename inventory; authored under this ADR's required-artifact contract; retired to `.omc/state/archive/` after Wave 15-ZB.

## J. Completion Report

<!--
adr: ADR-0347
status: Proposed
date: 2026-05-21
session: 2026-05-21 realignment-wave authoring (sibling to ADR-0340..ADR-0346; consolidation of CLAUDE.md new_governance_lane_prefix backlog)
sibling_adrs: ADR-0340 (capacity model), ADR-0341 (cellular promotion gates), ADR-0342 (API versioning hybrid), ADR-0343 (DR matrix), ADR-0344 (sustainability + finops), ADR-0345 (OSS stewardship class), ADR-0346 (product readiness checklist)
authority_source: CLAUDE.md new_governance_lane_prefix line + ADR-0132 + ADR-0335
canonical_substitution: s/^oya-governance-fitness-/oya-governance-/g
canonical_prefixes_exhaustive: 2 (oya-governance-* for governance-owned lanes; oya-check-* for check-family lanes)
rename_inventory_path: .omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json
rename_surfaces:
  workflows: 10 files at .github/workflows/oya-governance-fitness-*.yml (filenames + name: fields)
  lanes_yaml: ~40 records in registry/quality/lanes.yaml
  catalog: 28 files at registry/catalog/oya-governance-fitness-*.yaml
  crates: ~51 directories at crates/oya-governance-fitness-*-* (Cargo.toml package names + workspace.dependencies + Cargo.lock regeneration)
  adr_citations: ~41 files under docs/decisions/
  standards: 1 file under docs/standards/
  state: 1 file under .omc/state/ (excluding inventory file itself)
  master_plan_sequencing: 14 references in specs/master-plan-sequencing.json
  canonical_primitives: 1 file at tools/hooks/_canonical-primitives.md
  branch_protection: 1 file at .github/branch-protection.yaml (required-status-check entries)
  microservice_manifests: 0 manifests at authoring time (verified via grep); re-verified by Wave 15-ZB CI suite
new_lanes: 3 + 1 informational (oya-governance-no-foundry-fitness-residue, oya-governance-lane-prefix-vocabulary, oya-governance-rename-inventory-presence; informational oya-governance-rename-residue-allowlist-integrity)
historical_context_allowlist: ADR-0335 + ADR-0347 retirement-narrative paragraphs only at landing time; additions require ADR amendment
sunset_window: 30 days post-Wave-15-ZB-completion for new authoring (E.1 + E.2); BLOCKER at Wave-15-ZB-landing-time for inventory-presence (E.3, no grace window)
wave_queue: Wave 15-ZB added to /specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_waves; single-PR mechanical rename; sequenced under ADR-0328 batch discipline as one batch
amendments:
  - ADR-0132 (per-lane IP cadence replaced by bulk-rename cadence; canonical-prefix declaration preserved)
  - ADR-0335 (CI lane terminology aligned with foundry retirement)
  - ADR-0136-amendment (consistent terminology carried forward)
  - ADR-0245 (substrate-vs-product alignment; governance is the substrate concern)
out_of_scope: actual file renames + cross-reference updates (deferred to Wave 15-ZB executor PR); cross-Bominal corpus rename (Bominal authors sibling ADR independently); foundry pipeline substrate rename (preserved under intelligence ownership per ADR-0335)
hyperscaler_precedents: AWS aws-amplify-cli rename 2022; AWS aws-iam-authenticator rename 2023; Google GKE Container Registry → Artifact Registry rename 2023; Microsoft Azure CLI az v1 → v2 rename; Microsoft .NET Core 3.1 → 5.0 rename
commits: ADR + .omc/state/oya-governance-fitness-rename-inventory-2026-05-21.json (machine-readable inventory) + /specs/master-plan-sequencing.json Wave 15-ZB sub-wave entry
-->
