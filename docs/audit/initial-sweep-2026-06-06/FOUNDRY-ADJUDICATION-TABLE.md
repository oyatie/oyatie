# FOUNDRY-ADJUDICATION-TABLE — BATCH-0 per-ident routing (door #1)

> **Status:** PLAN-READY (BATCH 0, NO mutation). Generated 2026-06-07 from a STRICTLY READ-ONLY pass over `/Users/jasonlee/Developer/source` @ branch `cleanup/whole-tree-2026-06-07`.
> **Authority:** [[FOUNDRY-VOCAB-ERADICATION-PLAN]] §2 (sense routing) + §4 BATCH 0 (adjudication deliverable).
> **Enumeration:** `git grep -hoIE 'oya-foundry-[a-z0-9-]+' | sort -u` → **683 distinct identifiers** (plan estimated ~655; drift is grep capturing compound-token prefixes ending in `-`, see note below).
> **Resolution order (most-specific first):** `vcs → fitness → platform → re-home-deferred`.

## Routing legend

| sense | route | target convention | ADR | batch |
|---|---|---|---|---|
| **VCS** | **RETIRED** | delete / supersede-note; *no replacement term* | ADR-0363 / ADR-0116 | Batch 3 |
| **FITNESS** | `oya-governance-*` | suffix-preserving (`oya-foundry-X` → `oya-governance-X`) | ADR-0347 | Batch 4 |
| **PLATFORM** | `oya-intelligence-*` | suffix-preserving (`oya-foundry-X` → `oya-intelligence-X`) | ADR-0335 D-43 | Batch 5 |
| **RE-HOME-DEFERRED** | **HOLD** | founder decides target per-ident; *do NOT guess* | — | Batch 6 |

**ref-classes-touched** legend: BUCK (target names+deps) · Cargo (`members`/`name=`/path-deps/lock) · code (`use`/imports/string-literals `.rs`/`.ts`) · docs (ADR/PRD/`.md` cross-cites) · registry (`registry/**`, catalog `context:`/lane IDs, `.yaml` manifests). `-` = ident not yet materialized in any tracked file beyond a single class (most idents are *planned* crate names living in catalog `context:` + docs, per plan §1c).

## Per-sense totals

| sense | distinct idents | batch | route |
|---|---|---|---|
| **VCS** | **51** | Batch 3 | RETIRED |
| **FITNESS** | **101** | Batch 4 | oya-governance-* |
| **PLATFORM** | **523** | Batch 5 | oya-intelligence-* |
| **RE-HOME-DEFERRED** | **8** | Batch 6 | HOLD |
| **TOTAL** | **683** | — | — |

> **Plan §2 vs measured:** plan estimated VCS-small / FITNESS≈82 / PLATFORM≈250 / re-home≈323 over a ~655 corpus. Measured by deterministic signal-grep over 683 idents: VCS=51, FITNESS=101, PLATFORM=523, RE-HOME-DEFERRED=8. The re-home pile is far SMALLER than the plan's ~323 estimate because the plan counted *prose lines* + un-bucketed tokens, whereas this table classifies *structural `oya-foundry-*` identifiers* — nearly all carry an explicit platform/fitness/vcs signal in their suffix. The ~323 re-home figure in the plan remains the right magnitude for the **prose/persona/user-journey line** disposition (Batch 6), which is NOT identifier-level and is out of scope for this ident table.

## (1) RE-HOME-DEFERRED — the founder per-ident pile (surface this)

These 8 identifiers carry **no** unambiguous vcs/fitness/platform signal in their suffix, or straddle two senses. Per plan §2 hard rule, **no batch may mutate these until the founder confirms a target.** Each row notes WHY it is ambiguous.

| ident | why deferred (evidence) |
|---|---|
| `oya-foundry-cli-` | Bare CLI prefix. SPEC.md:140 shows `oya-foundry-cli-{persona}-*` spanning gate/dev/admin/build/agent/ops/pack/catalog — straddles governance (gate) AND platform (build/agent/ops). Composition-root binary; sense depends on per-persona split. |
| `oya-foundry-dashboard-` | Bare dashboard prefix (M02 visibility-operator-plane). Operator UI — could be intelligence (runtime visibility) or a standalone product surface. No platform/fitness/vcs token. |
| `oya-foundry-e2e-` | Bare E2E test-runtime prefix (standards/testing.md). End-to-end harness spans ALL senses; not ownable to one axis. |
| `oya-foundry-gate-` | Bare "Gate" core primitive (one of Catalog/Lane/Gate/Bypass, ADR-0015/0025). Gate = CI-gate enforcement (governance-leaning) BUT is a foundational platform kernel. Genuinely 2-sense. |
| `oya-foundry-gate-domain` | PRD:190 "Gate rule evaluation" — the domain layer of the Gate primitive; inherits the Gate 2-sense ambiguity. |
| `oya-foundry-gate-kernel` | PRD:189 "Gate primitive (CI gate for cross-axis review, claim-ceiling, etc.)". claim-ceiling is fitness, cross-axis review is governance, but it is THE platform Gate kernel. 2-sense. |
| `oya-foundry-shared-` | Bare shared-crates prefix (ADR-0143 `oya-foundry-shared-*`). Cross-cutting shared lib — sense undefined until contents enumerated. |
| `oya-foundry-supply-app` | Supply-chain attestation (Cosign+Trivy+SBOM, ADR-0025/0039). `supply-chain` is a FITNESS signal BUT this is a release-integrity CI gate — could route to governance OR intelligence (release pipeline). Founder call. |

**RE-HOME-DEFERRED count: 8 identifiers** → all park at Batch 6, founder-adjudicated only.

## (2) Palantir-Foundry carve-out allowlist (DO NOT rename)

Lines matching `/palantir/i` are the external "Palantir Foundry" product reference — **allowed, never renamed** (plan §1a). These get a file-path + line-anchor allowlist in the brand-residue lane config; the *prose* "Palantir Foundry" stays while any co-located `oya-foundry-*` ident still renames per its own sense.

- **Total palantir lines:** 596 (565 non-`.omc` product source + 31 `.omc` bucket) across **246 files** — matches plan's "~596 palantir total".
- **`oya-foundry-*` idents that co-occur with palantir on the same line** (rename the ident, KEEP the "Palantir Foundry" prose): `oya-foundry-eval-*` (docs/products/foundry/PRD.md:883) and `oya-foundry-capability-kernel` (PRD.md:911). Both are PLATFORM-sense; "Palantir" is a competitor-comparison term, not a carve-out of the ident.

### Carve-out file allowlist (path → palantir line count) — top product-source files

| file | palantir lines |
|---|---|
| `docs/user-journeys/j43-healthcare-nurse-patient-handoff/story.md` | 37 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 30 |
| `docs/onboarding/doctrine-bootcamp-2026-05-21.md` | 30 |
| `docs/architecture/hyperscaler-pattern-attribution.md` | 26 |
| `docs/investor/competitive-landscape-and-positioning.md` | 21 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 20 |
| `docs/products/foundry/PRD.md` | 14 |
| `specs/microservices/ontology.json` | 11 |
| `docs/decisions/ADR-0702-identity-authz-live-apex.md` | 10 |
| `oya/ontology/PRD.md` | 10 |
| `docs/investor/ask-and-use-of-funds.md` | 8 |
| `docs/architecture/adr-cross-reference-graph-2026-05-20.md` | 7 |
| `docs/architecture/keystone-bundle-idea-refine-deep-dive.md` | 7 |
| `docs/decisions/ADR-0136-foundry-as-single-microservice.md` | 7 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 7 |
| `docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md` | 6 |
| `docs/prds/ontology.md` | 6 |
| `docs/architecture/adr-corpus-line-audit-2026-05-21.md` | 5 |
| `docs/architecture/keystone-bundle-audit-report.md` | 5 |
| `docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md` | 5 |
| `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` | 5 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 5 |
| `specs/hyperscaler-architecture-invariants.json` | 5 |
| `ADR-INVENTORY.tsv` | 4 |
| `cloud/cell-lifecycle/IPs/IP-ADR-0341-Cellular-Promotion-Gates.md` | 4 |
| `docs/GLOSSARY.md` | 4 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 4 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 4 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 4 |
| `docs/decisions/ADR-0143-foundry-per-bc-release-pointer.md` | 4 |
| `docs/investor/moat-and-defensibility.md` | 4 |
| `cloud/tenancy/ARCHITECTURE.md` | 3 |
| `docs/architecture/keystone-bundle-reading-order.md` | 3 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 3 |
| `docs/decisions/ADR-0704-k8s-port-live-apex.md` | 3 |
| `docs/decisions/ADR-0704-k8s-port-live-apex.md` | 3 |
| `docs/decisions/ADR-0709-general-live-apex.md` | 3 |
| `docs/investor/company-overview-deck.md` | 3 |
| `docs/investor/unit-economics-and-pricing-model.md` | 3 |
| `docs/standards/documentation-rigor.md` | 3 |

*(Full 246-file allowlist — including the 14 `.omc` files — is reproducible via `git grep -ilE palantir`; the executor wires the complete path+line-anchor set into the brand-residue lane config at Batch 7.)*

## (3) Full 683-row adjudication table

Sorted VCS → FITNESS → PLATFORM → RE-HOME-DEFERRED, then alpha. `proposed-target-name` is suffix-preserving.

| # | ident | sense | proposed-target-name | ref-classes-touched | batch |
|---|---|---|---|---|---|
| 1 | `oya-foundry-branch-protection-adapter-bitbucket` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 2 | `oya-foundry-branch-protection-adapter-gitea` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 3 | `oya-foundry-branch-protection-adapter-github` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 4 | `oya-foundry-branch-protection-adapter-gitlab` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 5 | `oya-foundry-branch-protection-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 6 | `oya-foundry-codeowners-mirror-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs | 3 |
| 7 | `oya-foundry-fitness-changeset-state` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 8 | `oya-foundry-fitness-changeset-state-enum-closed` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 9 | `oya-foundry-fitness-changeset-state-monotonicity` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 10 | `oya-foundry-fitness-pr-merge-gate-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 11 | `oya-foundry-fitness-pr-review` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 12 | `oya-foundry-fitness-pr-traceability-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 13 | `oya-foundry-fitness-sequential-pr-merge-conflicts` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 14 | `oya-foundry-pr-review-dispatcher-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 15 | `oya-foundry-pr-traceability-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs | 3 |
| 16 | `oya-foundry-repoctl-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 17 | `oya-foundry-vcs-` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs/registry | 3 |
| 18 | `oya-foundry-vcs-admission` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs | 3 |
| 19 | `oya-foundry-vcs-admission-gate` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs | 3 |
| 20 | `oya-foundry-vcs-admission-gate-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 21 | `oya-foundry-vcs-application` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 22 | `oya-foundry-vcs-artifact-indexer-adapter` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 23 | `oya-foundry-vcs-changeset-state-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 24 | `oya-foundry-vcs-changeset-state-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 25 | `oya-foundry-vcs-cli` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 26 | `oya-foundry-vcs-cli-ratchet` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 27 | `oya-foundry-vcs-gitops-controller-worker` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 28 | `oya-foundry-vcs-merge-queue-` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs | 3 |
| 29 | `oya-foundry-vcs-merge-queue-adapter` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 30 | `oya-foundry-vcs-merge-queue-conflict-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 31 | `oya-foundry-vcs-merge-queue-fix-loop-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 32 | `oya-foundry-vcs-merge-queue-scheduler-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 33 | `oya-foundry-vcs-orchestrator-` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs/registry | 3 |
| 34 | `oya-foundry-vcs-orchestrator-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs/registry | 3 |
| 35 | `oya-foundry-vcs-orchestrator-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 36 | `oya-foundry-vcs-polyglot-indexer-adapters` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 37 | `oya-foundry-vcs-promotion-application` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 38 | `oya-foundry-vcs-promotion-controller` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 39 | `oya-foundry-vcs-provider-execution-gate-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 40 | `oya-foundry-vcs-provider-execution-gate-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 41 | `oya-foundry-vcs-review-domain` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 42 | `oya-foundry-vcs-review-fix-application` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 43 | `oya-foundry-vcs-review-mergequeue` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code/docs | 3 |
| 44 | `oya-foundry-vcs-review-mergequeue-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 45 | `oya-foundry-vcs-rust-indexer-adapter` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 46 | `oya-foundry-vcs-test-standard-domain` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 47 | `oya-foundry-vcs-test-standard-gate` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | code | 3 |
| 48 | `oya-foundry-vcs-webhook-receiver-` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 49 | `oya-foundry-webhook-receiver-app` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 50 | `oya-foundry-webhook-receiver-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 51 | `oya-foundry-write-gate-kernel` | VCS | `RETIRED (delete / supersede-note; no replacement term)` | docs | 3 |
| 52 | `oya-foundry-adr-citation-kernel` | FITNESS | `oya-governance-adr-citation-kernel` | code/docs | 4 |
| 53 | `oya-foundry-adr-index-kernel` | FITNESS | `oya-governance-adr-index-kernel` | code/docs | 4 |
| 54 | `oya-foundry-adr-promoter` | FITNESS | `oya-governance-adr-promoter` | docs | 4 |
| 55 | `oya-foundry-api-semver-kernel` | FITNESS | `oya-governance-api-semver-kernel` | code | 4 |
| 56 | `oya-foundry-architecture-map-` | FITNESS | `oya-governance-architecture-map-` | docs | 4 |
| 57 | `oya-foundry-architecture-map-app` | FITNESS | `oya-governance-architecture-map-app` | docs | 4 |
| 58 | `oya-foundry-architecture-map-kernel` | FITNESS | `oya-governance-architecture-map-kernel` | docs | 4 |
| 59 | `oya-foundry-authority-cohesion-kernel` | FITNESS | `oya-governance-authority-cohesion-kernel` | code/docs | 4 |
| 60 | `oya-foundry-autonomy-ceiling-` | FITNESS | `oya-governance-autonomy-ceiling-` | docs | 4 |
| 61 | `oya-foundry-autonomy-ceiling-app` | FITNESS | `oya-governance-autonomy-ceiling-app` | docs | 4 |
| 62 | `oya-foundry-brand-residue-kernel` | FITNESS | `oya-governance-brand-residue-kernel` | code/docs | 4 |
| 63 | `oya-foundry-cargo-prefix-kernel` | FITNESS | `oya-governance-cargo-prefix-kernel` | code/docs | 4 |
| 64 | `oya-foundry-check-x` | FITNESS | `oya-governance-check-x` | code | 4 |
| 65 | `oya-foundry-claim-ceiling-kernel` | FITNESS | `oya-governance-claim-ceiling-kernel` | code/docs | 4 |
| 66 | `oya-foundry-cohesion-fitness-kernel` | FITNESS | `oya-governance-cohesion-fitness-kernel` | code/docs | 4 |
| 67 | `oya-foundry-cohesion-kernel` | FITNESS | `oya-governance-cohesion-kernel` | docs | 4 |
| 68 | `oya-foundry-constitution-cite-kernel` | FITNESS | `oya-governance-constitution-cite-kernel` | code/docs | 4 |
| 69 | `oya-foundry-data-class-fitness-kernel` | FITNESS | `oya-governance-data-class-fitness-kernel` | code/docs | 4 |
| 70 | `oya-foundry-data-class-kernel` | FITNESS | `oya-governance-data-class-kernel` | docs | 4 |
| 71 | `oya-foundry-doc-catalog-kernel` | FITNESS | `oya-governance-doc-catalog-kernel` | code/docs | 4 |
| 72 | `oya-foundry-documentation-system-kernel` | FITNESS | `oya-governance-documentation-system-kernel` | code/docs | 4 |
| 73 | `oya-foundry-eol-feed` | FITNESS | `oya-governance-eol-feed` | docs | 4 |
| 74 | `oya-foundry-fitness` | FITNESS | `oya-governance-fitness` | code/docs | 4 |
| 75 | `oya-foundry-fitness-` | FITNESS | `oya-governance-fitness-` | code/docs | 4 |
| 76 | `oya-foundry-fitness-adapter-with-no-importer-kernel` | FITNESS | `oya-governance-fitness-adapter-with-no-importer-kernel` | docs | 4 |
| 77 | `oya-foundry-fitness-adr-shape` | FITNESS | `oya-governance-fitness-adr-shape` | code/docs | 4 |
| 78 | `oya-foundry-fitness-adr-shape-kernel` | FITNESS | `oya-governance-fitness-adr-shape-kernel` | docs | 4 |
| 79 | `oya-foundry-fitness-agentic-navigability-kernel` | FITNESS | `oya-governance-fitness-agentic-navigability-kernel` | docs | 4 |
| 80 | `oya-foundry-fitness-api-semver` | FITNESS | `oya-governance-fitness-api-semver` | docs | 4 |
| 81 | `oya-foundry-fitness-architecture-map-freshness-kernel` | FITNESS | `oya-governance-fitness-architecture-map-freshness-kernel` | docs | 4 |
| 82 | `oya-foundry-fitness-aspirational-enforcement` | FITNESS | `oya-governance-fitness-aspirational-enforcement` | docs | 4 |
| 83 | `oya-foundry-fitness-authoritative-tracked` | FITNESS | `oya-governance-fitness-authoritative-tracked` | code/docs | 4 |
| 84 | `oya-foundry-fitness-authoritative-tracked-kernel` | FITNESS | `oya-governance-fitness-authoritative-tracked-kernel` | docs | 4 |
| 85 | `oya-foundry-fitness-banned-primitives` | FITNESS | `oya-governance-fitness-banned-primitives` | code/docs | 4 |
| 86 | `oya-foundry-fitness-banned-primitives-kernel` | FITNESS | `oya-governance-fitness-banned-primitives-kernel` | docs | 4 |
| 87 | `oya-foundry-fitness-bypass-kernel` | FITNESS | `oya-governance-fitness-bypass-kernel` | docs | 4 |
| 88 | `oya-foundry-fitness-cedar-` | FITNESS | `oya-governance-fitness-cedar-` | docs | 4 |
| 89 | `oya-foundry-fitness-cedar-coverage` | FITNESS | `oya-governance-fitness-cedar-coverage` | docs | 4 |
| 90 | `oya-foundry-fitness-cedar-default-deny-coverage` | FITNESS | `oya-governance-fitness-cedar-default-deny-coverage` | docs | 4 |
| 91 | `oya-foundry-fitness-cedar-fragment-signature` | FITNESS | `oya-governance-fitness-cedar-fragment-signature` | docs | 4 |
| 92 | `oya-foundry-fitness-cedar-tenant-fragment-restriction` | FITNESS | `oya-governance-fitness-cedar-tenant-fragment-restriction` | docs | 4 |
| 93 | `oya-foundry-fitness-claim-ceiling-kernel` | FITNESS | `oya-governance-fitness-claim-ceiling-kernel` | docs | 4 |
| 94 | `oya-foundry-fitness-cohesion` | FITNESS | `oya-governance-fitness-cohesion` | docs | 4 |
| 95 | `oya-foundry-fitness-cohesion-fitness-kernel` | FITNESS | `oya-governance-fitness-cohesion-fitness-kernel` | docs | 4 |
| 96 | `oya-foundry-fitness-doc-freshness-kernel` | FITNESS | `oya-governance-fitness-doc-freshness-kernel` | docs | 4 |
| 97 | `oya-foundry-fitness-doc-style-kernel` | FITNESS | `oya-governance-fitness-doc-style-kernel` | docs | 4 |
| 98 | `oya-foundry-fitness-docs` | FITNESS | `oya-governance-fitness-docs` | docs | 4 |
| 99 | `oya-foundry-fitness-evidence-secret-scan` | FITNESS | `oya-governance-fitness-evidence-secret-scan` | docs | 4 |
| 100 | `oya-foundry-fitness-honest-claims` | FITNESS | `oya-governance-fitness-honest-claims` | docs | 4 |
| 101 | `oya-foundry-fitness-image-discipline-kernel` | FITNESS | `oya-governance-fitness-image-discipline-kernel` | docs | 4 |
| 102 | `oya-foundry-fitness-license-policy-kernel` | FITNESS | `oya-governance-fitness-license-policy-kernel` | docs | 4 |
| 103 | `oya-foundry-fitness-lifecycle-kernel` | FITNESS | `oya-governance-fitness-lifecycle-kernel` | docs | 4 |
| 104 | `oya-foundry-fitness-master-plan-completion` | FITNESS | `oya-governance-fitness-master-plan-completion` | docs | 4 |
| 105 | `oya-foundry-fitness-mistakes-ledger-kernel` | FITNESS | `oya-governance-fitness-mistakes-ledger-kernel` | docs | 4 |
| 106 | `oya-foundry-fitness-orphan-detection-kernel` | FITNESS | `oya-governance-fitness-orphan-detection-kernel` | docs | 4 |
| 107 | `oya-foundry-fitness-portfolio-citation` | FITNESS | `oya-governance-fitness-portfolio-citation` | code/docs | 4 |
| 108 | `oya-foundry-fitness-portfolio-citation-kernel` | FITNESS | `oya-governance-fitness-portfolio-citation-kernel` | docs | 4 |
| 109 | `oya-foundry-fitness-pre-push-kernel` | FITNESS | `oya-governance-fitness-pre-push-kernel` | docs | 4 |
| 110 | `oya-foundry-fitness-predictable-naming` | FITNESS | `oya-governance-fitness-predictable-naming` | code/docs | 4 |
| 111 | `oya-foundry-fitness-predictable-naming-kernel` | FITNESS | `oya-governance-fitness-predictable-naming-kernel` | docs | 4 |
| 112 | `oya-foundry-fitness-protection-context-match` | FITNESS | `oya-governance-fitness-protection-context-match` | docs | 4 |
| 113 | `oya-foundry-fitness-provider-coupling-kernel` | FITNESS | `oya-governance-fitness-provider-coupling-kernel` | docs | 4 |
| 114 | `oya-foundry-fitness-purpose-audit` | FITNESS | `oya-governance-fitness-purpose-audit` | code/docs | 4 |
| 115 | `oya-foundry-fitness-purpose-audit-app` | FITNESS | `oya-governance-fitness-purpose-audit-app` | docs | 4 |
| 116 | `oya-foundry-fitness-purpose-kernel` | FITNESS | `oya-governance-fitness-purpose-kernel` | docs | 4 |
| 117 | `oya-foundry-fitness-quality-lane-kernel` | FITNESS | `oya-governance-fitness-quality-lane-kernel` | docs | 4 |
| 118 | `oya-foundry-fitness-rename-inventory-2026-05-21` | FITNESS | `oya-governance-fitness-rename-inventory-2026-05-21` | docs | 4 |
| 119 | `oya-foundry-fitness-sunset-lifecycle-kernel` | FITNESS | `oya-governance-fitness-sunset-lifecycle-kernel` | docs | 4 |
| 120 | `oya-foundry-fitness-supply-chain` | FITNESS | `oya-governance-fitness-supply-chain` | docs | 4 |
| 121 | `oya-foundry-fitness-supply-chain-kernel` | FITNESS | `oya-governance-fitness-supply-chain-kernel` | docs | 4 |
| 122 | `oya-foundry-fitness-tos-policy-kernel` | FITNESS | `oya-governance-fitness-tos-policy-kernel` | docs | 4 |
| 123 | `oya-foundry-fitness-upstream-api-drift-kernel` | FITNESS | `oya-governance-fitness-upstream-api-drift-kernel` | docs | 4 |
| 124 | `oya-foundry-fitness-webhook-delivery-log-monotonic` | FITNESS | `oya-governance-fitness-webhook-delivery-log-monotonic` | docs | 4 |
| 125 | `oya-foundry-glossary-coverage-kernel` | FITNESS | `oya-governance-glossary-coverage-kernel` | code/docs | 4 |
| 126 | `oya-foundry-glossary-localization-kernel` | FITNESS | `oya-governance-glossary-localization-kernel` | docs | 4 |
| 127 | `oya-foundry-glossary-vocabulary-kernel` | FITNESS | `oya-governance-glossary-vocabulary-kernel` | code/docs | 4 |
| 128 | `oya-foundry-guardrails-autonomy-ceiling-gate-adapter-cedar` | FITNESS | `oya-governance-guardrails-autonomy-ceiling-gate-adapter-cedar` | code/registry | 4 |
| 129 | `oya-foundry-guardrails-autonomy-ceiling-gate-kernel` | FITNESS | `oya-governance-guardrails-autonomy-ceiling-gate-kernel` | code/registry | 4 |
| 130 | `oya-foundry-lane-` | FITNESS | `oya-governance-lane-` | docs | 4 |
| 131 | `oya-foundry-lane-app` | FITNESS | `oya-governance-lane-app` | docs | 4 |
| 132 | `oya-foundry-lane-kernel` | FITNESS | `oya-governance-lane-kernel` | docs | 4 |
| 133 | `oya-foundry-lane-planner` | FITNESS | `oya-governance-lane-planner` | docs | 4 |
| 134 | `oya-foundry-license-policy-kernel` | FITNESS | `oya-governance-license-policy-kernel` | code/docs | 4 |
| 135 | `oya-foundry-placeholder-debt-kernel` | FITNESS | `oya-governance-placeholder-debt-kernel` | code/docs | 4 |
| 136 | `oya-foundry-pre-push-kernel` | FITNESS | `oya-governance-pre-push-kernel` | code/docs | 4 |
| 137 | `oya-foundry-quality-lane-kernel` | FITNESS | `oya-governance-quality-lane-kernel` | code/docs | 4 |
| 138 | `oya-foundry-raci-coverage-kernel` | FITNESS | `oya-governance-raci-coverage-kernel` | docs | 4 |
| 139 | `oya-foundry-raci-team-coverage-kernel` | FITNESS | `oya-governance-raci-team-coverage-kernel` | code/docs | 4 |
| 140 | `oya-foundry-readme-coverage-kernel` | FITNESS | `oya-governance-readme-coverage-kernel` | docs | 4 |
| 141 | `oya-foundry-readme-doc-coverage-kernel` | FITNESS | `oya-governance-readme-doc-coverage-kernel` | code/docs | 4 |
| 142 | `oya-foundry-runbook-freshness-kernel` | FITNESS | `oya-governance-runbook-freshness-kernel` | code/docs | 4 |
| 143 | `oya-foundry-runbook-index-kernel` | FITNESS | `oya-governance-runbook-index-kernel` | code/docs | 4 |
| 144 | `oya-foundry-scorecard-` | FITNESS | `oya-governance-scorecard-` | docs | 4 |
| 145 | `oya-foundry-scorecard-app` | FITNESS | `oya-governance-scorecard-app` | docs | 4 |
| 146 | `oya-foundry-scorecard-kernel` | FITNESS | `oya-governance-scorecard-kernel` | docs | 4 |
| 147 | `oya-foundry-slo-coverage-kernel` | FITNESS | `oya-governance-slo-coverage-kernel` | code/docs | 4 |
| 148 | `oya-foundry-supply-chain-kernel` | FITNESS | `oya-governance-supply-chain-kernel` | code/docs | 4 |
| 149 | `oya-foundry-test-quarantine` | FITNESS | `oya-governance-test-quarantine` | docs | 4 |
| 150 | `oya-foundry-typescript-workspace-kernel` | FITNESS | `oya-governance-typescript-workspace-kernel` | code/docs | 4 |
| 151 | `oya-foundry-vendor-contract-recency-kernel` | FITNESS | `oya-governance-vendor-contract-recency-kernel` | code/docs | 4 |
| 152 | `oya-foundry-vendor-recency-kernel` | FITNESS | `oya-governance-vendor-recency-kernel` | docs | 4 |
| 153 | `oya-foundry-account-` | PLATFORM | `oya-intelligence-account-` | code/docs/registry | 5 |
| 154 | `oya-foundry-account-adapter` | PLATFORM | `oya-intelligence-account-adapter` | code/docs/registry | 5 |
| 155 | `oya-foundry-account-adapter-` | PLATFORM | `oya-intelligence-account-adapter-` | code/docs/registry | 5 |
| 156 | `oya-foundry-account-adapter-claude-code` | PLATFORM | `oya-intelligence-account-adapter-claude-code` | code/docs | 5 |
| 157 | `oya-foundry-account-adapter-codex-cli` | PLATFORM | `oya-intelligence-account-adapter-codex-cli` | code/docs | 5 |
| 158 | `oya-foundry-account-adapter-gemini-cli` | PLATFORM | `oya-intelligence-account-adapter-gemini-cli` | code/docs | 5 |
| 159 | `oya-foundry-account-adapter-inmemory` | PLATFORM | `oya-intelligence-account-adapter-inmemory` | docs | 5 |
| 160 | `oya-foundry-account-adapter-openbao` | PLATFORM | `oya-intelligence-account-adapter-openbao` | code/docs/registry | 5 |
| 161 | `oya-foundry-account-app` | PLATFORM | `oya-intelligence-account-app` | code/docs | 5 |
| 162 | `oya-foundry-account-application` | PLATFORM | `oya-intelligence-account-application` | code | 5 |
| 163 | `oya-foundry-account-domain` | PLATFORM | `oya-intelligence-account-domain` | docs | 5 |
| 164 | `oya-foundry-account-helper` | PLATFORM | `oya-intelligence-account-helper` | code | 5 |
| 165 | `oya-foundry-account-management-app` | PLATFORM | `oya-intelligence-account-management-app` | docs | 5 |
| 166 | `oya-foundry-account-runtime` | PLATFORM | `oya-intelligence-account-runtime` | code/docs | 5 |
| 167 | `oya-foundry-account-runtime-app` | PLATFORM | `oya-intelligence-account-runtime-app` | code | 5 |
| 168 | `oya-foundry-account-supervisor-app` | PLATFORM | `oya-intelligence-account-supervisor-app` | code | 5 |
| 169 | `oya-foundry-adapter` | PLATFORM | `oya-intelligence-adapter` | code/docs | 5 |
| 170 | `oya-foundry-adapter-` | PLATFORM | `oya-intelligence-adapter-` | code/docs | 5 |
| 171 | `oya-foundry-adapter-anthropic` | PLATFORM | `oya-intelligence-adapter-anthropic` | code/docs | 5 |
| 172 | `oya-foundry-adapter-anthropic-api` | PLATFORM | `oya-intelligence-adapter-anthropic-api` | code/docs | 5 |
| 173 | `oya-foundry-adapter-anthropic-api-` | PLATFORM | `oya-intelligence-adapter-anthropic-api-` | code/docs | 5 |
| 174 | `oya-foundry-adapter-anthropic-api-adapter` | PLATFORM | `oya-intelligence-adapter-anthropic-api-adapter` | docs | 5 |
| 175 | `oya-foundry-adapter-anthropic-api-kernel` | PLATFORM | `oya-intelligence-adapter-anthropic-api-kernel` | docs | 5 |
| 176 | `oya-foundry-adapter-anthropic-subscription` | PLATFORM | `oya-intelligence-adapter-anthropic-subscription` | docs | 5 |
| 177 | `oya-foundry-adapter-anthropic-subscription-` | PLATFORM | `oya-intelligence-adapter-anthropic-subscription-` | docs | 5 |
| 178 | `oya-foundry-adapter-anthropic-subscription-adapter` | PLATFORM | `oya-intelligence-adapter-anthropic-subscription-adapter` | docs | 5 |
| 179 | `oya-foundry-adapter-claude` | PLATFORM | `oya-intelligence-adapter-claude` | docs | 5 |
| 180 | `oya-foundry-adapter-claude-` | PLATFORM | `oya-intelligence-adapter-claude-` | docs | 5 |
| 181 | `oya-foundry-adapter-claude-api` | PLATFORM | `oya-intelligence-adapter-claude-api` | docs | 5 |
| 182 | `oya-foundry-adapter-claude-subscription` | PLATFORM | `oya-intelligence-adapter-claude-subscription` | docs | 5 |
| 183 | `oya-foundry-adapter-codex` | PLATFORM | `oya-intelligence-adapter-codex` | docs | 5 |
| 184 | `oya-foundry-adapter-codex-` | PLATFORM | `oya-intelligence-adapter-codex-` | docs | 5 |
| 185 | `oya-foundry-adapter-codex-api` | PLATFORM | `oya-intelligence-adapter-codex-api` | docs | 5 |
| 186 | `oya-foundry-adapter-codex-subscription` | PLATFORM | `oya-intelligence-adapter-codex-subscription` | docs | 5 |
| 187 | `oya-foundry-adapter-gemini` | PLATFORM | `oya-intelligence-adapter-gemini` | docs | 5 |
| 188 | `oya-foundry-adapter-gemini-` | PLATFORM | `oya-intelligence-adapter-gemini-` | docs | 5 |
| 189 | `oya-foundry-adapter-gemini-api` | PLATFORM | `oya-intelligence-adapter-gemini-api` | docs | 5 |
| 190 | `oya-foundry-adapter-gemini-api-` | PLATFORM | `oya-intelligence-adapter-gemini-api-` | docs | 5 |
| 191 | `oya-foundry-adapter-gemini-subscription` | PLATFORM | `oya-intelligence-adapter-gemini-subscription` | docs | 5 |
| 192 | `oya-foundry-adapter-gemini-subscription-` | PLATFORM | `oya-intelligence-adapter-gemini-subscription-` | docs | 5 |
| 193 | `oya-foundry-adapter-kernel` | PLATFORM | `oya-intelligence-adapter-kernel` | code/docs | 5 |
| 194 | `oya-foundry-adapter-openai` | PLATFORM | `oya-intelligence-adapter-openai` | docs | 5 |
| 195 | `oya-foundry-adapter-openai-api` | PLATFORM | `oya-intelligence-adapter-openai-api` | docs | 5 |
| 196 | `oya-foundry-adapter-openai-api-` | PLATFORM | `oya-intelligence-adapter-openai-api-` | docs | 5 |
| 197 | `oya-foundry-adapter-openai-subscription` | PLATFORM | `oya-intelligence-adapter-openai-subscription` | docs | 5 |
| 198 | `oya-foundry-adapter-openai-subscription-` | PLATFORM | `oya-intelligence-adapter-openai-subscription-` | docs | 5 |
| 199 | `oya-foundry-adapter-oya-` | PLATFORM | `oya-intelligence-adapter-oya-` | docs | 5 |
| 200 | `oya-foundry-adapter-regional-pack-` | PLATFORM | `oya-intelligence-adapter-regional-pack-` | docs | 5 |
| 201 | `oya-foundry-adapter-router` | PLATFORM | `oya-intelligence-adapter-router` | docs | 5 |
| 202 | `oya-foundry-adapter-session-vault` | PLATFORM | `oya-intelligence-adapter-session-vault` | docs | 5 |
| 203 | `oya-foundry-agent-coordinator-kernel` | PLATFORM | `oya-intelligence-agent-coordinator-kernel` | docs | 5 |
| 204 | `oya-foundry-agent-read-cli` | PLATFORM | `oya-intelligence-agent-read-cli` | docs | 5 |
| 205 | `oya-foundry-agent-runtime` | PLATFORM | `oya-intelligence-agent-runtime` | docs | 5 |
| 206 | `oya-foundry-allow-egress` | PLATFORM | `oya-intelligence-allow-egress` | registry | 5 |
| 207 | `oya-foundry-allow-mesh-ingress` | PLATFORM | `oya-intelligence-allow-mesh-ingress` | registry | 5 |
| 208 | `oya-foundry-anomaly-` | PLATFORM | `oya-intelligence-anomaly-` | docs | 5 |
| 209 | `oya-foundry-anthropic-compat` | PLATFORM | `oya-intelligence-anthropic-compat` | docs | 5 |
| 210 | `oya-foundry-api-rest-adapter` | PLATFORM | `oya-intelligence-api-rest-adapter` | docs | 5 |
| 211 | `oya-foundry-audit-chain-` | PLATFORM | `oya-intelligence-audit-chain-` | code | 5 |
| 212 | `oya-foundry-autonomy-domain` | PLATFORM | `oya-intelligence-autonomy-domain` | docs | 5 |
| 213 | `oya-foundry-backbone-workload-live-app` | PLATFORM | `oya-intelligence-backbone-workload-live-app` | code | 5 |
| 214 | `oya-foundry-budget-kernel` | PLATFORM | `oya-intelligence-budget-kernel` | docs | 5 |
| 215 | `oya-foundry-bypass-` | PLATFORM | `oya-intelligence-bypass-` | code/docs | 5 |
| 216 | `oya-foundry-bypass-app` | PLATFORM | `oya-intelligence-bypass-app` | docs | 5 |
| 217 | `oya-foundry-bypass-kernel` | PLATFORM | `oya-intelligence-bypass-kernel` | code/docs | 5 |
| 218 | `oya-foundry-cache` | PLATFORM | `oya-intelligence-cache` | docs | 5 |
| 219 | `oya-foundry-canary-controller-` | PLATFORM | `oya-intelligence-canary-controller-` | docs | 5 |
| 220 | `oya-foundry-canary-controller-app` | PLATFORM | `oya-intelligence-canary-controller-app` | docs | 5 |
| 221 | `oya-foundry-canary-controller-kernel` | PLATFORM | `oya-intelligence-canary-controller-kernel` | docs | 5 |
| 222 | `oya-foundry-capability` | PLATFORM | `oya-intelligence-capability` | code/docs | 5 |
| 223 | `oya-foundry-capability-` | PLATFORM | `oya-intelligence-capability-` | code/docs | 5 |
| 224 | `oya-foundry-capability-api` | PLATFORM | `oya-intelligence-capability-api` | docs | 5 |
| 225 | `oya-foundry-capability-app` | PLATFORM | `oya-intelligence-capability-app` | docs | 5 |
| 226 | `oya-foundry-capability-author` | PLATFORM | `oya-intelligence-capability-author` | docs | 5 |
| 227 | `oya-foundry-capability-doc-writer` | PLATFORM | `oya-intelligence-capability-doc-writer` | docs | 5 |
| 228 | `oya-foundry-capability-kernel` | PLATFORM | `oya-intelligence-capability-kernel` | code/docs | 5 |
| 229 | `oya-foundry-capability-registry-` | PLATFORM | `oya-intelligence-capability-registry-` | docs | 5 |
| 230 | `oya-foundry-catalog-` | PLATFORM | `oya-intelligence-catalog-` | code/docs | 5 |
| 231 | `oya-foundry-catalog-api` | PLATFORM | `oya-intelligence-catalog-api` | docs | 5 |
| 232 | `oya-foundry-catalog-app` | PLATFORM | `oya-intelligence-catalog-app` | docs | 5 |
| 233 | `oya-foundry-catalog-kernel` | PLATFORM | `oya-intelligence-catalog-kernel` | code/docs | 5 |
| 234 | `oya-foundry-catalog-runtime` | PLATFORM | `oya-intelligence-catalog-runtime` | docs | 5 |
| 235 | `oya-foundry-change-` | PLATFORM | `oya-intelligence-change-` | docs | 5 |
| 236 | `oya-foundry-change-kernel` | PLATFORM | `oya-intelligence-change-kernel` | docs | 5 |
| 237 | `oya-foundry-ci-runner-adapter-1es` | PLATFORM | `oya-intelligence-ci-runner-adapter-1es` | docs | 5 |
| 238 | `oya-foundry-ci-runner-adapter-buildkite` | PLATFORM | `oya-intelligence-ci-runner-adapter-buildkite` | docs | 5 |
| 239 | `oya-foundry-ci-runner-adapter-circleci` | PLATFORM | `oya-intelligence-ci-runner-adapter-circleci` | docs | 5 |
| 240 | `oya-foundry-ci-runner-adapter-github-actions` | PLATFORM | `oya-intelligence-ci-runner-adapter-github-actions` | docs | 5 |
| 241 | `oya-foundry-ci-runner-adapter-gitlab-ci` | PLATFORM | `oya-intelligence-ci-runner-adapter-gitlab-ci` | docs | 5 |
| 242 | `oya-foundry-ci-runner-kernel` | PLATFORM | `oya-intelligence-ci-runner-kernel` | docs | 5 |
| 243 | `oya-foundry-ci-state-store` | PLATFORM | `oya-intelligence-ci-state-store` | docs | 5 |
| 244 | `oya-foundry-ci-worker` | PLATFORM | `oya-intelligence-ci-worker` | docs | 5 |
| 245 | `oya-foundry-claude-account-adapter` | PLATFORM | `oya-intelligence-claude-account-adapter` | docs | 5 |
| 246 | `oya-foundry-cli-dev-runtime` | PLATFORM | `oya-intelligence-cli-dev-runtime` | docs | 5 |
| 247 | `oya-foundry-cloud-mutation-kernel` | PLATFORM | `oya-intelligence-cloud-mutation-kernel` | code/docs | 5 |
| 248 | `oya-foundry-codex-account-adapter` | PLATFORM | `oya-intelligence-codex-account-adapter` | docs | 5 |
| 249 | `oya-foundry-cohort-app` | PLATFORM | `oya-intelligence-cohort-app` | docs | 5 |
| 250 | `oya-foundry-console-tos-wizard` | PLATFORM | `oya-intelligence-console-tos-wizard` | docs | 5 |
| 251 | `oya-foundry-contract-registry-` | PLATFORM | `oya-intelligence-contract-registry-` | docs | 5 |
| 252 | `oya-foundry-control-kernel` | PLATFORM | `oya-intelligence-control-kernel` | docs | 5 |
| 253 | `oya-foundry-cost-budget-kernel` | PLATFORM | `oya-intelligence-cost-budget-kernel` | code/docs | 5 |
| 254 | `oya-foundry-critic-app` | PLATFORM | `oya-intelligence-critic-app` | docs | 5 |
| 255 | `oya-foundry-cron-kernel` | PLATFORM | `oya-intelligence-cron-kernel` | docs | 5 |
| 256 | `oya-foundry-default-deny` | PLATFORM | `oya-intelligence-default-deny` | registry | 5 |
| 257 | `oya-foundry-dev-promoter` | PLATFORM | `oya-intelligence-dev-promoter` | docs | 5 |
| 258 | `oya-foundry-domain-` | PLATFORM | `oya-intelligence-domain-` | docs | 5 |
| 259 | `oya-foundry-eac-app` | PLATFORM | `oya-intelligence-eac-app` | docs | 5 |
| 260 | `oya-foundry-eval` | PLATFORM | `oya-intelligence-eval` | code/docs/registry | 5 |
| 261 | `oya-foundry-eval-` | PLATFORM | `oya-intelligence-eval-` | code/docs/registry | 5 |
| 262 | `oya-foundry-eval-app` | PLATFORM | `oya-intelligence-eval-app` | code/docs/registry | 5 |
| 263 | `oya-foundry-eval-application` | PLATFORM | `oya-intelligence-eval-application` | code/docs/registry | 5 |
| 264 | `oya-foundry-eval-baselines-` | PLATFORM | `oya-intelligence-eval-baselines-` | docs/registry | 5 |
| 265 | `oya-foundry-eval-baselines-eu` | PLATFORM | `oya-intelligence-eval-baselines-eu` | registry | 5 |
| 266 | `oya-foundry-eval-baselines-kr` | PLATFORM | `oya-intelligence-eval-baselines-kr` | registry | 5 |
| 267 | `oya-foundry-eval-determinism-correctness` | PLATFORM | `oya-intelligence-eval-determinism-correctness` | code/registry | 5 |
| 268 | `oya-foundry-eval-domain` | PLATFORM | `oya-intelligence-eval-domain` | docs | 5 |
| 269 | `oya-foundry-eval-eval-runner-adapter` | PLATFORM | `oya-intelligence-eval-eval-runner-adapter` | code/registry | 5 |
| 270 | `oya-foundry-eval-eval-runner-adapter-gpu` | PLATFORM | `oya-intelligence-eval-eval-runner-adapter-gpu` | code/registry | 5 |
| 271 | `oya-foundry-eval-eval-runner-adapter-s3` | PLATFORM | `oya-intelligence-eval-eval-runner-adapter-s3` | code/registry | 5 |
| 272 | `oya-foundry-eval-eval-runner-api` | PLATFORM | `oya-intelligence-eval-eval-runner-api` | code/registry | 5 |
| 273 | `oya-foundry-eval-eval-runner-app` | PLATFORM | `oya-intelligence-eval-eval-runner-app` | code/registry | 5 |
| 274 | `oya-foundry-eval-eval-runner-domain` | PLATFORM | `oya-intelligence-eval-eval-runner-domain` | code/docs/registry | 5 |
| 275 | `oya-foundry-eval-eval-runner-kernel` | PLATFORM | `oya-intelligence-eval-eval-runner-kernel` | code/docs/registry | 5 |
| 276 | `oya-foundry-eval-eval-runner-rest` | PLATFORM | `oya-intelligence-eval-eval-runner-rest` | code/registry | 5 |
| 277 | `oya-foundry-eval-eval-runner-sdk` | PLATFORM | `oya-intelligence-eval-eval-runner-sdk` | code/registry | 5 |
| 278 | `oya-foundry-eval-eval-runner-usecase` | PLATFORM | `oya-intelligence-eval-eval-runner-usecase` | code/registry | 5 |
| 279 | `oya-foundry-eval-eval-runner-worker` | PLATFORM | `oya-intelligence-eval-eval-runner-worker` | code/registry | 5 |
| 280 | `oya-foundry-eval-eval-set-registry-rest` | PLATFORM | `oya-intelligence-eval-eval-set-registry-rest` | docs | 5 |
| 281 | `oya-foundry-eval-golden-set-curator` | PLATFORM | `oya-intelligence-eval-golden-set-curator` | docs | 5 |
| 282 | `oya-foundry-eval-kek-eu` | PLATFORM | `oya-intelligence-eval-kek-eu` | registry | 5 |
| 283 | `oya-foundry-eval-kek-kr` | PLATFORM | `oya-intelligence-eval-kek-kr` | registry | 5 |
| 284 | `oya-foundry-eval-kernel` | PLATFORM | `oya-intelligence-eval-kernel` | code/docs | 5 |
| 285 | `oya-foundry-eval-multispectrum-review-runner` | PLATFORM | `oya-intelligence-eval-multispectrum-review-runner` | docs | 5 |
| 286 | `oya-foundry-eval-parity-analyzer-` | PLATFORM | `oya-intelligence-eval-parity-analyzer-` | code/docs/registry | 5 |
| 287 | `oya-foundry-eval-parity-analyzer-adapter-clickhouse` | PLATFORM | `oya-intelligence-eval-parity-analyzer-adapter-clickhouse` | code/registry | 5 |
| 288 | `oya-foundry-eval-parity-analyzer-api` | PLATFORM | `oya-intelligence-eval-parity-analyzer-api` | registry | 5 |
| 289 | `oya-foundry-eval-parity-analyzer-kernel` | PLATFORM | `oya-intelligence-eval-parity-analyzer-kernel` | registry | 5 |
| 290 | `oya-foundry-eval-parity-analyzer-rest` | PLATFORM | `oya-intelligence-eval-parity-analyzer-rest` | docs | 5 |
| 291 | `oya-foundry-eval-replay-engine-rest` | PLATFORM | `oya-intelligence-eval-replay-engine-rest` | docs | 5 |
| 292 | `oya-foundry-eval-replay-eu` | PLATFORM | `oya-intelligence-eval-replay-eu` | registry | 5 |
| 293 | `oya-foundry-eval-replay-kr` | PLATFORM | `oya-intelligence-eval-replay-kr` | registry | 5 |
| 294 | `oya-foundry-eval-run-latency` | PLATFORM | `oya-intelligence-eval-run-latency` | code/registry | 5 |
| 295 | `oya-foundry-eval-runner` | PLATFORM | `oya-intelligence-eval-runner` | docs | 5 |
| 296 | `oya-foundry-eval-usecase` | PLATFORM | `oya-intelligence-eval-usecase` | docs | 5 |
| 297 | `oya-foundry-eventing-protocols-kernel` | PLATFORM | `oya-intelligence-eventing-protocols-kernel` | docs | 5 |
| 298 | `oya-foundry-evidence` | PLATFORM | `oya-intelligence-evidence` | code/docs/registry | 5 |
| 299 | `oya-foundry-evidence-` | PLATFORM | `oya-intelligence-evidence-` | code/docs/registry | 5 |
| 300 | `oya-foundry-evidence-adapter-file` | PLATFORM | `oya-intelligence-evidence-adapter-file` | code/docs | 5 |
| 301 | `oya-foundry-evidence-app` | PLATFORM | `oya-intelligence-evidence-app` | docs | 5 |
| 302 | `oya-foundry-evidence-capability-invocation-recorder-api` | PLATFORM | `oya-intelligence-evidence-capability-invocation-recorder-api` | registry | 5 |
| 303 | `oya-foundry-evidence-capability-invocation-recorder-domain` | PLATFORM | `oya-intelligence-evidence-capability-invocation-recorder-domain` | registry | 5 |
| 304 | `oya-foundry-evidence-capability-invocation-recorder-kernel` | PLATFORM | `oya-intelligence-evidence-capability-invocation-recorder-kernel` | code/docs/registry | 5 |
| 305 | `oya-foundry-evidence-chain-integrity-correctness` | PLATFORM | `oya-intelligence-evidence-chain-integrity-correctness` | code/registry | 5 |
| 306 | `oya-foundry-evidence-emit-latency` | PLATFORM | `oya-intelligence-evidence-emit-latency` | code/registry | 5 |
| 307 | `oya-foundry-evidence-evidence-pack-builder-adapter` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-adapter` | code/docs/registry | 5 |
| 308 | `oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-adapter-audit-chain-bridge` | code/docs/registry | 5 |
| 309 | `oya-foundry-evidence-evidence-pack-builder-adapter-postgres` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-adapter-postgres` | code/registry | 5 |
| 310 | `oya-foundry-evidence-evidence-pack-builder-adapter-s3` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-adapter-s3` | code/registry | 5 |
| 311 | `oya-foundry-evidence-evidence-pack-builder-api` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-api` | code/docs/registry | 5 |
| 312 | `oya-foundry-evidence-evidence-pack-builder-app` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-app` | code/registry | 5 |
| 313 | `oya-foundry-evidence-evidence-pack-builder-domain` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-domain` | code/docs/registry | 5 |
| 314 | `oya-foundry-evidence-evidence-pack-builder-kernel` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-kernel` | registry | 5 |
| 315 | `oya-foundry-evidence-evidence-pack-builder-rest` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-rest` | code/registry | 5 |
| 316 | `oya-foundry-evidence-evidence-pack-builder-usecase` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-usecase` | code/docs/registry | 5 |
| 317 | `oya-foundry-evidence-evidence-pack-builder-worker` | PLATFORM | `oya-intelligence-evidence-evidence-pack-builder-worker` | code/registry | 5 |
| 318 | `oya-foundry-evidence-evidence-query-api` | PLATFORM | `oya-intelligence-evidence-evidence-query-api` | registry | 5 |
| 319 | `oya-foundry-evidence-evidence-query-domain` | PLATFORM | `oya-intelligence-evidence-evidence-query-domain` | registry | 5 |
| 320 | `oya-foundry-evidence-file` | PLATFORM | `oya-intelligence-evidence-file` | docs | 5 |
| 321 | `oya-foundry-evidence-kernel` | PLATFORM | `oya-intelligence-evidence-kernel` | code/docs | 5 |
| 322 | `oya-foundry-evidence-nats` | PLATFORM | `oya-intelligence-evidence-nats` | registry | 5 |
| 323 | `oya-foundry-evidence-pack-kernel` | PLATFORM | `oya-intelligence-evidence-pack-kernel` | docs | 5 |
| 324 | `oya-foundry-evidence-regulator-export` | PLATFORM | `oya-intelligence-evidence-regulator-export` | docs/registry | 5 |
| 325 | `oya-foundry-evidence-regulator-export-api` | PLATFORM | `oya-intelligence-evidence-regulator-export-api` | registry | 5 |
| 326 | `oya-foundry-evidence-regulator-export-framework-profiles` | PLATFORM | `oya-intelligence-evidence-regulator-export-framework-profiles` | docs | 5 |
| 327 | `oya-foundry-evidence-rest-v1` | PLATFORM | `oya-intelligence-evidence-rest-v1` | registry | 5 |
| 328 | `oya-foundry-evidence-sdk` | PLATFORM | `oya-intelligence-evidence-sdk` | code/docs/registry | 5 |
| 329 | `oya-foundry-evidence-sdk-rust` | PLATFORM | `oya-intelligence-evidence-sdk-rust` | registry | 5 |
| 330 | `oya-foundry-foundation-bypass-ledger-app` | PLATFORM | `oya-intelligence-foundation-bypass-ledger-app` | docs | 5 |
| 331 | `oya-foundry-gate-catalog-domain` | PLATFORM | `oya-intelligence-gate-catalog-domain` | docs | 5 |
| 332 | `oya-foundry-gemini-account-adapter` | PLATFORM | `oya-intelligence-gemini-account-adapter` | docs | 5 |
| 333 | `oya-foundry-glossary-extractor` | PLATFORM | `oya-intelligence-glossary-extractor` | docs | 5 |
| 334 | `oya-foundry-goal-kernel` | PLATFORM | `oya-intelligence-goal-kernel` | docs | 5 |
| 335 | `oya-foundry-grit-cli` | PLATFORM | `oya-intelligence-grit-cli` | docs | 5 |
| 336 | `oya-foundry-guardrails` | PLATFORM | `oya-intelligence-guardrails` | code/docs/registry | 5 |
| 337 | `oya-foundry-guardrails-` | PLATFORM | `oya-intelligence-guardrails-` | code/docs/registry | 5 |
| 338 | `oya-foundry-guardrails-ai-slop-detector-kernel` | PLATFORM | `oya-intelligence-guardrails-ai-slop-detector-kernel` | code/registry | 5 |
| 339 | `oya-foundry-guardrails-autonomy-tier-gate-adapter-cedar` | PLATFORM | `oya-intelligence-guardrails-autonomy-tier-gate-adapter-cedar` | code | 5 |
| 340 | `oya-foundry-guardrails-autonomy-tier-gate-kernel` | PLATFORM | `oya-intelligence-guardrails-autonomy-tier-gate-kernel` | code/docs | 5 |
| 341 | `oya-foundry-guardrails-cedar-engine` | PLATFORM | `oya-intelligence-guardrails-cedar-engine` | docs/registry | 5 |
| 342 | `oya-foundry-guardrails-classifier-model-serving` | PLATFORM | `oya-intelligence-guardrails-classifier-model-serving` | registry | 5 |
| 343 | `oya-foundry-guardrails-content-safety-rule-engine-adapter-postgres` | PLATFORM | `oya-intelligence-guardrails-content-safety-rule-engine-adapter-postgres` | code/registry | 5 |
| 344 | `oya-foundry-guardrails-content-safety-rule-engine-kernel` | PLATFORM | `oya-intelligence-guardrails-content-safety-rule-engine-kernel` | code/docs/registry | 5 |
| 345 | `oya-foundry-guardrails-domain` | PLATFORM | `oya-intelligence-guardrails-domain` | docs | 5 |
| 346 | `oya-foundry-guardrails-jailbreak-detector-adapter-classifier-model` | PLATFORM | `oya-intelligence-guardrails-jailbreak-detector-adapter-classifier-model` | code/registry | 5 |
| 347 | `oya-foundry-guardrails-jailbreak-detector-kernel` | PLATFORM | `oya-intelligence-guardrails-jailbreak-detector-kernel` | code/registry | 5 |
| 348 | `oya-foundry-guardrails-kernel` | PLATFORM | `oya-intelligence-guardrails-kernel` | docs | 5 |
| 349 | `oya-foundry-guardrails-output-validator-kernel` | PLATFORM | `oya-intelligence-guardrails-output-validator-kernel` | code/docs/registry | 5 |
| 350 | `oya-foundry-guardrails-pii-redaction` | PLATFORM | `oya-intelligence-guardrails-pii-redaction` | docs | 5 |
| 351 | `oya-foundry-guardrails-policy-eval-latency` | PLATFORM | `oya-intelligence-guardrails-policy-eval-latency` | code/registry | 5 |
| 352 | `oya-foundry-guardrails-postgres` | PLATFORM | `oya-intelligence-guardrails-postgres` | registry | 5 |
| 353 | `oya-foundry-guardrails-prompt-classifier-adapter-classifier-model` | PLATFORM | `oya-intelligence-guardrails-prompt-classifier-adapter-classifier-model` | code | 5 |
| 354 | `oya-foundry-guardrails-prompt-classifier-app` | PLATFORM | `oya-intelligence-guardrails-prompt-classifier-app` | code/registry | 5 |
| 355 | `oya-foundry-guardrails-prompt-classifier-kernel` | PLATFORM | `oya-intelligence-guardrails-prompt-classifier-kernel` | code/docs/registry | 5 |
| 356 | `oya-foundry-guardrails-prompt-classifier-rest` | PLATFORM | `oya-intelligence-guardrails-prompt-classifier-rest` | code/registry | 5 |
| 357 | `oya-foundry-guardrails-prompt-injection-detector` | PLATFORM | `oya-intelligence-guardrails-prompt-injection-detector` | docs | 5 |
| 358 | `oya-foundry-guardrails-rule-store` | PLATFORM | `oya-intelligence-guardrails-rule-store` | docs | 5 |
| 359 | `oya-foundry-guardrails-shadow-mode-fp-budget` | PLATFORM | `oya-intelligence-guardrails-shadow-mode-fp-budget` | code/registry | 5 |
| 360 | `oya-foundry-guardrails-toxic-content-classifier` | PLATFORM | `oya-intelligence-guardrails-toxic-content-classifier` | docs | 5 |
| 361 | `oya-foundry-icm-cli` | PLATFORM | `oya-intelligence-icm-cli` | docs | 5 |
| 362 | `oya-foundry-incident-grader` | PLATFORM | `oya-intelligence-incident-grader` | docs | 5 |
| 363 | `oya-foundry-jsonl-supervisor-adapter` | PLATFORM | `oya-intelligence-jsonl-supervisor-adapter` | docs | 5 |
| 364 | `oya-foundry-key-rotation-worker` | PLATFORM | `oya-intelligence-key-rotation-worker` | registry | 5 |
| 365 | `oya-foundry-lease-kernel` | PLATFORM | `oya-intelligence-lease-kernel` | docs | 5 |
| 366 | `oya-foundry-marketplace` | PLATFORM | `oya-intelligence-marketplace` | docs | 5 |
| 367 | `oya-foundry-marketplace-app` | PLATFORM | `oya-intelligence-marketplace-app` | docs | 5 |
| 368 | `oya-foundry-mcp-adapter` | PLATFORM | `oya-intelligence-mcp-adapter` | docs | 5 |
| 369 | `oya-foundry-mcp-gateway` | PLATFORM | `oya-intelligence-mcp-gateway` | code/docs | 5 |
| 370 | `oya-foundry-mcp-gateway-kernel` | PLATFORM | `oya-intelligence-mcp-gateway-kernel` | code/docs | 5 |
| 371 | `oya-foundry-mcp-server` | PLATFORM | `oya-intelligence-mcp-server` | docs | 5 |
| 372 | `oya-foundry-mcp-server-` | PLATFORM | `oya-intelligence-mcp-server-` | docs | 5 |
| 373 | `oya-foundry-mcp-server-app` | PLATFORM | `oya-intelligence-mcp-server-app` | docs | 5 |
| 374 | `oya-foundry-memory-adapter` | PLATFORM | `oya-intelligence-memory-adapter` | docs | 5 |
| 375 | `oya-foundry-memory-kernel` | PLATFORM | `oya-intelligence-memory-kernel` | docs | 5 |
| 376 | `oya-foundry-meta-api` | PLATFORM | `oya-intelligence-meta-api` | docs | 5 |
| 377 | `oya-foundry-meta-rest` | PLATFORM | `oya-intelligence-meta-rest` | registry | 5 |
| 378 | `oya-foundry-metadata-augment-cli` | PLATFORM | `oya-intelligence-metadata-augment-cli` | registry | 5 |
| 379 | `oya-foundry-milvus` | PLATFORM | `oya-intelligence-milvus` | registry | 5 |
| 380 | `oya-foundry-milvus-slo` | PLATFORM | `oya-intelligence-milvus-slo` | registry | 5 |
| 381 | `oya-foundry-mobile-native-kernel` | PLATFORM | `oya-intelligence-mobile-native-kernel` | code/docs | 5 |
| 382 | `oya-foundry-model-cutover` | PLATFORM | `oya-intelligence-model-cutover` | docs | 5 |
| 383 | `oya-foundry-model-data-pipeline-` | PLATFORM | `oya-intelligence-model-data-pipeline-` | docs | 5 |
| 384 | `oya-foundry-model-eval-` | PLATFORM | `oya-intelligence-model-eval-` | docs | 5 |
| 385 | `oya-foundry-model-finetune-` | PLATFORM | `oya-intelligence-model-finetune-` | docs | 5 |
| 386 | `oya-foundry-model-kernel` | PLATFORM | `oya-intelligence-model-kernel` | docs | 5 |
| 387 | `oya-foundry-model-lora` | PLATFORM | `oya-intelligence-model-lora` | docs | 5 |
| 388 | `oya-foundry-model-redteam-` | PLATFORM | `oya-intelligence-model-redteam-` | docs | 5 |
| 389 | `oya-foundry-model-registry-` | PLATFORM | `oya-intelligence-model-registry-` | docs | 5 |
| 390 | `oya-foundry-model-serve-` | PLATFORM | `oya-intelligence-model-serve-` | docs | 5 |
| 391 | `oya-foundry-model-serve-app` | PLATFORM | `oya-intelligence-model-serve-app` | docs | 5 |
| 392 | `oya-foundry-model-speech-` | PLATFORM | `oya-intelligence-model-speech-` | docs | 5 |
| 393 | `oya-foundry-model-speech-kernel` | PLATFORM | `oya-intelligence-model-speech-kernel` | docs | 5 |
| 394 | `oya-foundry-model-speech-stt-app` | PLATFORM | `oya-intelligence-model-speech-stt-app` | docs | 5 |
| 395 | `oya-foundry-model-speech-tts-app` | PLATFORM | `oya-intelligence-model-speech-tts-app` | docs | 5 |
| 396 | `oya-foundry-model-speech-voice-biometric-app` | PLATFORM | `oya-intelligence-model-speech-voice-biometric-app` | docs | 5 |
| 397 | `oya-foundry-model-speech-wake-word-app` | PLATFORM | `oya-intelligence-model-speech-wake-word-app` | docs | 5 |
| 398 | `oya-foundry-model-train-` | PLATFORM | `oya-intelligence-model-train-` | docs | 5 |
| 399 | `oya-foundry-model-train-app` | PLATFORM | `oya-intelligence-model-train-app` | docs | 5 |
| 400 | `oya-foundry-model-vision-` | PLATFORM | `oya-intelligence-model-vision-` | docs | 5 |
| 401 | `oya-foundry-model-vision-classification-app` | PLATFORM | `oya-intelligence-model-vision-classification-app` | docs | 5 |
| 402 | `oya-foundry-model-vision-detection-app` | PLATFORM | `oya-intelligence-model-vision-detection-app` | docs | 5 |
| 403 | `oya-foundry-model-vision-facial-recognition-app` | PLATFORM | `oya-intelligence-model-vision-facial-recognition-app` | docs | 5 |
| 404 | `oya-foundry-model-vision-kernel` | PLATFORM | `oya-intelligence-model-vision-kernel` | docs | 5 |
| 405 | `oya-foundry-model-vision-ocr-app` | PLATFORM | `oya-intelligence-model-vision-ocr-app` | docs | 5 |
| 406 | `oya-foundry-model-vision-scene-anomaly-app` | PLATFORM | `oya-intelligence-model-vision-scene-anomaly-app` | docs | 5 |
| 407 | `oya-foundry-model-vision-video-analytics-app` | PLATFORM | `oya-intelligence-model-vision-video-analytics-app` | docs | 5 |
| 408 | `oya-foundry-new-feature-kernel` | PLATFORM | `oya-intelligence-new-feature-kernel` | docs | 5 |
| 409 | `oya-foundry-openai-compat` | PLATFORM | `oya-intelligence-openai-compat` | docs | 5 |
| 410 | `oya-foundry-openapi-kernel` | PLATFORM | `oya-intelligence-openapi-kernel` | code/docs | 5 |
| 411 | `oya-foundry-persistence-domain` | PLATFORM | `oya-intelligence-persistence-domain` | docs | 5 |
| 412 | `oya-foundry-plan-kernel` | PLATFORM | `oya-intelligence-plan-kernel` | docs | 5 |
| 413 | `oya-foundry-planner-app` | PLATFORM | `oya-intelligence-planner-app` | docs | 5 |
| 414 | `oya-foundry-plugin-runtime-kernel` | PLATFORM | `oya-intelligence-plugin-runtime-kernel` | docs | 5 |
| 415 | `oya-foundry-plugin-substrate-app` | PLATFORM | `oya-intelligence-plugin-substrate-app` | docs | 5 |
| 416 | `oya-foundry-pod-kernel` | PLATFORM | `oya-intelligence-pod-kernel` | docs | 5 |
| 417 | `oya-foundry-policy` | PLATFORM | `oya-intelligence-policy` | code/docs/registry | 5 |
| 418 | `oya-foundry-policy-app` | PLATFORM | `oya-intelligence-policy-app` | docs | 5 |
| 419 | `oya-foundry-policy-binding-api` | PLATFORM | `oya-intelligence-policy-binding-api` | docs | 5 |
| 420 | `oya-foundry-policy-domain` | PLATFORM | `oya-intelligence-policy-domain` | docs | 5 |
| 421 | `oya-foundry-policy-engine-cedar` | PLATFORM | `oya-intelligence-policy-engine-cedar` | docs | 5 |
| 422 | `oya-foundry-policy-evaluator-cedar-domain` | PLATFORM | `oya-intelligence-policy-evaluator-cedar-domain` | docs | 5 |
| 423 | `oya-foundry-policy-evaluator-domain` | PLATFORM | `oya-intelligence-policy-evaluator-domain` | docs | 5 |
| 424 | `oya-foundry-policy-fuzz` | PLATFORM | `oya-intelligence-policy-fuzz` | docs | 5 |
| 425 | `oya-foundry-policy-kernel` | PLATFORM | `oya-intelligence-policy-kernel` | code/docs | 5 |
| 426 | `oya-foundry-policy-rest` | PLATFORM | `oya-intelligence-policy-rest` | docs/registry | 5 |
| 427 | `oya-foundry-primary` | PLATFORM | `oya-intelligence-primary` | docs | 5 |
| 428 | `oya-foundry-prod-promoter` | PLATFORM | `oya-intelligence-prod-promoter` | docs | 5 |
| 429 | `oya-foundry-proto-kernel` | PLATFORM | `oya-intelligence-proto-kernel` | docs | 5 |
| 430 | `oya-foundry-provider-` | PLATFORM | `oya-intelligence-provider-` | docs | 5 |
| 431 | `oya-foundry-provider-anthropic-adapter` | PLATFORM | `oya-intelligence-provider-anthropic-adapter` | docs | 5 |
| 432 | `oya-foundry-provider-app` | PLATFORM | `oya-intelligence-provider-app` | docs | 5 |
| 433 | `oya-foundry-provider-azure-openai-adapter` | PLATFORM | `oya-intelligence-provider-azure-openai-adapter` | docs | 5 |
| 434 | `oya-foundry-provider-bedrock-adapter` | PLATFORM | `oya-intelligence-provider-bedrock-adapter` | docs | 5 |
| 435 | `oya-foundry-provider-credential-store-adapter` | PLATFORM | `oya-intelligence-provider-credential-store-adapter` | docs | 5 |
| 436 | `oya-foundry-provider-domain` | PLATFORM | `oya-intelligence-provider-domain` | docs | 5 |
| 437 | `oya-foundry-provider-google-adapter` | PLATFORM | `oya-intelligence-provider-google-adapter` | docs | 5 |
| 438 | `oya-foundry-provider-kernel` | PLATFORM | `oya-intelligence-provider-kernel` | docs | 5 |
| 439 | `oya-foundry-provider-openai-adapter` | PLATFORM | `oya-intelligence-provider-openai-adapter` | docs | 5 |
| 440 | `oya-foundry-provider-runtime` | PLATFORM | `oya-intelligence-provider-runtime` | docs | 5 |
| 441 | `oya-foundry-provider-sglang-adapter` | PLATFORM | `oya-intelligence-provider-sglang-adapter` | docs | 5 |
| 442 | `oya-foundry-provider-tensorrt-adapter` | PLATFORM | `oya-intelligence-provider-tensorrt-adapter` | docs | 5 |
| 443 | `oya-foundry-provider-vllm-adapter` | PLATFORM | `oya-intelligence-provider-vllm-adapter` | docs | 5 |
| 444 | `oya-foundry-providers` | PLATFORM | `oya-intelligence-providers` | code/docs/registry | 5 |
| 445 | `oya-foundry-providers-` | PLATFORM | `oya-intelligence-providers-` | code/docs/registry | 5 |
| 446 | `oya-foundry-providers-adapter-` | PLATFORM | `oya-intelligence-providers-adapter-` | code/docs/registry | 5 |
| 447 | `oya-foundry-providers-adapter-anthropic-api` | PLATFORM | `oya-intelligence-providers-adapter-anthropic-api` | code/registry | 5 |
| 448 | `oya-foundry-providers-adapter-anthropic-api-` | PLATFORM | `oya-intelligence-providers-adapter-anthropic-api-` | code | 5 |
| 449 | `oya-foundry-providers-adapter-anthropic-subscription` | PLATFORM | `oya-intelligence-providers-adapter-anthropic-subscription` | code/registry | 5 |
| 450 | `oya-foundry-providers-adapter-gemini-api` | PLATFORM | `oya-intelligence-providers-adapter-gemini-api` | code/registry | 5 |
| 451 | `oya-foundry-providers-adapter-gemini-subscription` | PLATFORM | `oya-intelligence-providers-adapter-gemini-subscription` | code/registry | 5 |
| 452 | `oya-foundry-providers-adapter-in-house` | PLATFORM | `oya-intelligence-providers-adapter-in-house` | code/registry | 5 |
| 453 | `oya-foundry-providers-adapter-openai-api` | PLATFORM | `oya-intelligence-providers-adapter-openai-api` | code/registry | 5 |
| 454 | `oya-foundry-providers-adapter-openai-subscription` | PLATFORM | `oya-intelligence-providers-adapter-openai-subscription` | code/registry | 5 |
| 455 | `oya-foundry-providers-adapter-openbao` | PLATFORM | `oya-intelligence-providers-adapter-openbao` | code/registry | 5 |
| 456 | `oya-foundry-providers-availability-anthropic` | PLATFORM | `oya-intelligence-providers-availability-anthropic` | code/registry | 5 |
| 457 | `oya-foundry-providers-availability-google` | PLATFORM | `oya-intelligence-providers-availability-google` | code/registry | 5 |
| 458 | `oya-foundry-providers-availability-openai` | PLATFORM | `oya-intelligence-providers-availability-openai` | code/registry | 5 |
| 459 | `oya-foundry-providers-circuit-breaker-correctness` | PLATFORM | `oya-intelligence-providers-circuit-breaker-correctness` | code/registry | 5 |
| 460 | `oya-foundry-providers-events` | PLATFORM | `oya-intelligence-providers-events` | registry | 5 |
| 461 | `oya-foundry-providers-openbao-agent-config` | PLATFORM | `oya-intelligence-providers-openbao-agent-config` | registry | 5 |
| 462 | `oya-foundry-providers-postgres` | PLATFORM | `oya-intelligence-providers-postgres` | registry | 5 |
| 463 | `oya-foundry-providers-postgres-backups` | PLATFORM | `oya-intelligence-providers-postgres-backups` | registry | 5 |
| 464 | `oya-foundry-providers-redis` | PLATFORM | `oya-intelligence-providers-redis` | registry | 5 |
| 465 | `oya-foundry-providers-router` | PLATFORM | `oya-intelligence-providers-router` | code/docs/registry | 5 |
| 466 | `oya-foundry-providers-router-` | PLATFORM | `oya-intelligence-providers-router-` | code/docs/registry | 5 |
| 467 | `oya-foundry-providers-router-adapter` | PLATFORM | `oya-intelligence-providers-router-adapter` | code/docs/registry | 5 |
| 468 | `oya-foundry-providers-router-api` | PLATFORM | `oya-intelligence-providers-router-api` | code/registry | 5 |
| 469 | `oya-foundry-providers-router-app` | PLATFORM | `oya-intelligence-providers-router-app` | code/registry | 5 |
| 470 | `oya-foundry-providers-router-domain` | PLATFORM | `oya-intelligence-providers-router-domain` | code/docs/registry | 5 |
| 471 | `oya-foundry-providers-router-kernel` | PLATFORM | `oya-intelligence-providers-router-kernel` | code/docs/registry | 5 |
| 472 | `oya-foundry-providers-router-rest` | PLATFORM | `oya-intelligence-providers-router-rest` | code/registry | 5 |
| 473 | `oya-foundry-providers-router-sdk` | PLATFORM | `oya-intelligence-providers-router-sdk` | code/registry | 5 |
| 474 | `oya-foundry-providers-router-signing-` | PLATFORM | `oya-intelligence-providers-router-signing-` | code | 5 |
| 475 | `oya-foundry-providers-router-usecase` | PLATFORM | `oya-intelligence-providers-router-usecase` | code/registry | 5 |
| 476 | `oya-foundry-providers-router-worker` | PLATFORM | `oya-intelligence-providers-router-worker` | code/registry | 5 |
| 477 | `oya-foundry-rag` | PLATFORM | `oya-intelligence-rag` | docs/registry | 5 |
| 478 | `oya-foundry-rag-app` | PLATFORM | `oya-intelligence-rag-app` | docs | 5 |
| 479 | `oya-foundry-rag-endpoint-` | PLATFORM | `oya-intelligence-rag-endpoint-` | docs/registry | 5 |
| 480 | `oya-foundry-rag-endpoint-api` | PLATFORM | `oya-intelligence-rag-endpoint-api` | docs | 5 |
| 481 | `oya-foundry-rag-kernel` | PLATFORM | `oya-intelligence-rag-kernel` | docs | 5 |
| 482 | `oya-foundry-rag-rest` | PLATFORM | `oya-intelligence-rag-rest` | docs/registry | 5 |
| 483 | `oya-foundry-rag-worker` | PLATFORM | `oya-intelligence-rag-worker` | docs | 5 |
| 484 | `oya-foundry-registry` | PLATFORM | `oya-intelligence-registry` | docs/registry | 5 |
| 485 | `oya-foundry-registry-` | PLATFORM | `oya-intelligence-registry-` | docs/registry | 5 |
| 486 | `oya-foundry-registry-app` | PLATFORM | `oya-intelligence-registry-app` | docs | 5 |
| 487 | `oya-foundry-registry-kernel` | PLATFORM | `oya-intelligence-registry-kernel` | docs | 5 |
| 488 | `oya-foundry-registry-rest` | PLATFORM | `oya-intelligence-registry-rest` | registry | 5 |
| 489 | `oya-foundry-release` | PLATFORM | `oya-intelligence-release` | code/docs | 5 |
| 490 | `oya-foundry-release-app` | PLATFORM | `oya-intelligence-release-app` | docs | 5 |
| 491 | `oya-foundry-release-evidence-pack-kernel` | PLATFORM | `oya-intelligence-release-evidence-pack-kernel` | code/docs | 5 |
| 492 | `oya-foundry-release-pack-kernel` | PLATFORM | `oya-intelligence-release-pack-kernel` | docs | 5 |
| 493 | `oya-foundry-replay-app` | PLATFORM | `oya-intelligence-replay-app` | docs | 5 |
| 494 | `oya-foundry-rest` | PLATFORM | `oya-intelligence-rest` | docs | 5 |
| 495 | `oya-foundry-review-app` | PLATFORM | `oya-intelligence-review-app` | code | 5 |
| 496 | `oya-foundry-risk-tracker` | PLATFORM | `oya-intelligence-risk-tracker` | docs | 5 |
| 497 | `oya-foundry-robotics-control-` | PLATFORM | `oya-intelligence-robotics-control-` | docs | 5 |
| 498 | `oya-foundry-robotics-control-app` | PLATFORM | `oya-intelligence-robotics-control-app` | docs | 5 |
| 499 | `oya-foundry-robotics-control-kernel` | PLATFORM | `oya-intelligence-robotics-control-kernel` | docs | 5 |
| 500 | `oya-foundry-robotics-control-runtime` | PLATFORM | `oya-intelligence-robotics-control-runtime` | docs | 5 |
| 501 | `oya-foundry-route-policy-kernel` | PLATFORM | `oya-intelligence-route-policy-kernel` | docs | 5 |
| 502 | `oya-foundry-router` | PLATFORM | `oya-intelligence-router` | docs | 5 |
| 503 | `oya-foundry-run-` | PLATFORM | `oya-intelligence-run-` | code/docs | 5 |
| 504 | `oya-foundry-run-adapter-file` | PLATFORM | `oya-intelligence-run-adapter-file` | code/docs | 5 |
| 505 | `oya-foundry-run-kernel` | PLATFORM | `oya-intelligence-run-kernel` | code/docs | 5 |
| 506 | `oya-foundry-run-worker` | PLATFORM | `oya-intelligence-run-worker` | docs | 5 |
| 507 | `oya-foundry-runbook-extractor` | PLATFORM | `oya-intelligence-runbook-extractor` | docs | 5 |
| 508 | `oya-foundry-runtime` | PLATFORM | `oya-intelligence-runtime` | code/docs/registry | 5 |
| 509 | `oya-foundry-runtime-` | PLATFORM | `oya-intelligence-runtime-` | code/docs/registry | 5 |
| 510 | `oya-foundry-runtime-autonomy` | PLATFORM | `oya-intelligence-runtime-autonomy` | registry | 5 |
| 511 | `oya-foundry-runtime-capability-executor-` | PLATFORM | `oya-intelligence-runtime-capability-executor-` | code/docs/registry | 5 |
| 512 | `oya-foundry-runtime-capability-executor-adapter` | PLATFORM | `oya-intelligence-runtime-capability-executor-adapter` | code/registry | 5 |
| 513 | `oya-foundry-runtime-capability-executor-api` | PLATFORM | `oya-intelligence-runtime-capability-executor-api` | code/registry | 5 |
| 514 | `oya-foundry-runtime-capability-executor-app` | PLATFORM | `oya-intelligence-runtime-capability-executor-app` | code/docs/registry | 5 |
| 515 | `oya-foundry-runtime-capability-executor-domain` | PLATFORM | `oya-intelligence-runtime-capability-executor-domain` | code/registry | 5 |
| 516 | `oya-foundry-runtime-capability-executor-kernel` | PLATFORM | `oya-intelligence-runtime-capability-executor-kernel` | code/docs/registry | 5 |
| 517 | `oya-foundry-runtime-capability-executor-rest` | PLATFORM | `oya-intelligence-runtime-capability-executor-rest` | code/registry | 5 |
| 518 | `oya-foundry-runtime-capability-executor-sdk` | PLATFORM | `oya-intelligence-runtime-capability-executor-sdk` | code/registry | 5 |
| 519 | `oya-foundry-runtime-capability-executor-usecase` | PLATFORM | `oya-intelligence-runtime-capability-executor-usecase` | code/registry | 5 |
| 520 | `oya-foundry-runtime-capability-registry-cache` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache` | code/docs/registry | 5 |
| 521 | `oya-foundry-runtime-capability-registry-cache-adapter` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-adapter` | code/registry | 5 |
| 522 | `oya-foundry-runtime-capability-registry-cache-adapter-postgres` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-adapter-postgres` | code/registry | 5 |
| 523 | `oya-foundry-runtime-capability-registry-cache-api` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-api` | code/registry | 5 |
| 524 | `oya-foundry-runtime-capability-registry-cache-app` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-app` | code/docs/registry | 5 |
| 525 | `oya-foundry-runtime-capability-registry-cache-kernel` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-kernel` | code/registry | 5 |
| 526 | `oya-foundry-runtime-capability-registry-cache-usecase` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-usecase` | code/registry | 5 |
| 527 | `oya-foundry-runtime-capability-registry-cache-worker` | PLATFORM | `oya-intelligence-runtime-capability-registry-cache-worker` | code/registry | 5 |
| 528 | `oya-foundry-runtime-invocation-orchestrator` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator` | code/docs/registry | 5 |
| 529 | `oya-foundry-runtime-invocation-orchestrator-adapter` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-adapter` | code/registry | 5 |
| 530 | `oya-foundry-runtime-invocation-orchestrator-api` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-api` | code/registry | 5 |
| 531 | `oya-foundry-runtime-invocation-orchestrator-app` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-app` | code/registry | 5 |
| 532 | `oya-foundry-runtime-invocation-orchestrator-domain` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-domain` | code/docs/registry | 5 |
| 533 | `oya-foundry-runtime-invocation-orchestrator-kernel` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-kernel` | code/registry | 5 |
| 534 | `oya-foundry-runtime-invocation-orchestrator-usecase` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-usecase` | code/registry | 5 |
| 535 | `oya-foundry-runtime-invocation-orchestrator-worker` | PLATFORM | `oya-intelligence-runtime-invocation-orchestrator-worker` | code/registry | 5 |
| 536 | `oya-foundry-runtime-load-latency` | PLATFORM | `oya-intelligence-runtime-load-latency` | code | 5 |
| 537 | `oya-foundry-runtime-policy` | PLATFORM | `oya-intelligence-runtime-policy` | docs | 5 |
| 538 | `oya-foundry-runtime-policy-` | PLATFORM | `oya-intelligence-runtime-policy-` | docs | 5 |
| 539 | `oya-foundry-runtime-policy-app` | PLATFORM | `oya-intelligence-runtime-policy-app` | docs | 5 |
| 540 | `oya-foundry-runtime-pool` | PLATFORM | `oya-intelligence-runtime-pool` | registry | 5 |
| 541 | `oya-foundry-runtime-postgres` | PLATFORM | `oya-intelligence-runtime-postgres` | docs/registry | 5 |
| 542 | `oya-foundry-runtime-postgres-replica-` | PLATFORM | `oya-intelligence-runtime-postgres-replica-` | docs | 5 |
| 543 | `oya-foundry-runtime-rag` | PLATFORM | `oya-intelligence-runtime-rag` | docs | 5 |
| 544 | `oya-foundry-runtime-redis` | PLATFORM | `oya-intelligence-runtime-redis` | docs/registry | 5 |
| 545 | `oya-foundry-runtime-redis-` | PLATFORM | `oya-intelligence-runtime-redis-` | docs | 5 |
| 546 | `oya-foundry-runtime-rest` | PLATFORM | `oya-intelligence-runtime-rest` | docs | 5 |
| 547 | `oya-foundry-runtime-runtime-pool` | PLATFORM | `oya-intelligence-runtime-runtime-pool` | code/docs/registry | 5 |
| 548 | `oya-foundry-runtime-runtime-pool-adapter` | PLATFORM | `oya-intelligence-runtime-runtime-pool-adapter` | code/registry | 5 |
| 549 | `oya-foundry-runtime-runtime-pool-api` | PLATFORM | `oya-intelligence-runtime-runtime-pool-api` | code/registry | 5 |
| 550 | `oya-foundry-runtime-runtime-pool-app` | PLATFORM | `oya-intelligence-runtime-runtime-pool-app` | code/registry | 5 |
| 551 | `oya-foundry-runtime-runtime-pool-kernel` | PLATFORM | `oya-intelligence-runtime-runtime-pool-kernel` | code/registry | 5 |
| 552 | `oya-foundry-runtime-runtime-pool-usecase` | PLATFORM | `oya-intelligence-runtime-runtime-pool-usecase` | code/registry | 5 |
| 553 | `oya-foundry-runtime-runtime-pool-worker` | PLATFORM | `oya-intelligence-runtime-runtime-pool-worker` | code/docs/registry | 5 |
| 554 | `oya-foundry-runtime-session-state` | PLATFORM | `oya-intelligence-runtime-session-state` | code/registry | 5 |
| 555 | `oya-foundry-runtime-session-state-` | PLATFORM | `oya-intelligence-runtime-session-state-` | code/registry | 5 |
| 556 | `oya-foundry-runtime-session-state-adapter` | PLATFORM | `oya-intelligence-runtime-session-state-adapter` | code/registry | 5 |
| 557 | `oya-foundry-runtime-session-state-adapter-postgres` | PLATFORM | `oya-intelligence-runtime-session-state-adapter-postgres` | code/registry | 5 |
| 558 | `oya-foundry-runtime-session-state-adapter-redis` | PLATFORM | `oya-intelligence-runtime-session-state-adapter-redis` | code/registry | 5 |
| 559 | `oya-foundry-runtime-session-state-api` | PLATFORM | `oya-intelligence-runtime-session-state-api` | code/registry | 5 |
| 560 | `oya-foundry-runtime-session-state-app` | PLATFORM | `oya-intelligence-runtime-session-state-app` | code/registry | 5 |
| 561 | `oya-foundry-runtime-session-state-domain` | PLATFORM | `oya-intelligence-runtime-session-state-domain` | code/registry | 5 |
| 562 | `oya-foundry-runtime-session-state-kernel` | PLATFORM | `oya-intelligence-runtime-session-state-kernel` | code/registry | 5 |
| 563 | `oya-foundry-runtime-session-state-sdk` | PLATFORM | `oya-intelligence-runtime-session-state-sdk` | code/registry | 5 |
| 564 | `oya-foundry-runtime-session-state-usecase` | PLATFORM | `oya-intelligence-runtime-session-state-usecase` | code/registry | 5 |
| 565 | `oya-foundry-rustdoc-fixer` | PLATFORM | `oya-intelligence-rustdoc-fixer` | docs | 5 |
| 566 | `oya-foundry-sandbox` | PLATFORM | `oya-intelligence-sandbox` | docs | 5 |
| 567 | `oya-foundry-sandbox-app` | PLATFORM | `oya-intelligence-sandbox-app` | docs | 5 |
| 568 | `oya-foundry-sandbox-escape-detector` | PLATFORM | `oya-intelligence-sandbox-escape-detector` | docs | 5 |
| 569 | `oya-foundry-sandbox-firecracker-app` | PLATFORM | `oya-intelligence-sandbox-firecracker-app` | docs | 5 |
| 570 | `oya-foundry-sandbox-kernel` | PLATFORM | `oya-intelligence-sandbox-kernel` | docs | 5 |
| 571 | `oya-foundry-sandbox-wasm-app` | PLATFORM | `oya-intelligence-sandbox-wasm-app` | docs | 5 |
| 572 | `oya-foundry-sdk-gen-` | PLATFORM | `oya-intelligence-sdk-gen-` | docs | 5 |
| 573 | `oya-foundry-secret-app` | PLATFORM | `oya-intelligence-secret-app` | docs | 5 |
| 574 | `oya-foundry-session-vault` | PLATFORM | `oya-intelligence-session-vault` | docs | 5 |
| 575 | `oya-foundry-settings-template` | PLATFORM | `oya-intelligence-settings-template` | code/docs | 5 |
| 576 | `oya-foundry-settings-template-adapter` | PLATFORM | `oya-intelligence-settings-template-adapter` | docs | 5 |
| 577 | `oya-foundry-settings-template-adapter-fs` | PLATFORM | `oya-intelligence-settings-template-adapter-fs` | docs | 5 |
| 578 | `oya-foundry-settings-template-kernel` | PLATFORM | `oya-intelligence-settings-template-kernel` | docs | 5 |
| 579 | `oya-foundry-shadow-diff-adapter-cedar` | PLATFORM | `oya-intelligence-shadow-diff-adapter-cedar` | docs | 5 |
| 580 | `oya-foundry-shadow-diff-adapter-event` | PLATFORM | `oya-intelligence-shadow-diff-adapter-event` | docs | 5 |
| 581 | `oya-foundry-shadow-diff-adapter-grpc` | PLATFORM | `oya-intelligence-shadow-diff-adapter-grpc` | docs | 5 |
| 582 | `oya-foundry-shadow-diff-adapter-http` | PLATFORM | `oya-intelligence-shadow-diff-adapter-http` | docs | 5 |
| 583 | `oya-foundry-shadow-diff-kernel` | PLATFORM | `oya-intelligence-shadow-diff-kernel` | docs | 5 |
| 584 | `oya-foundry-spec-interview-app` | PLATFORM | `oya-intelligence-spec-interview-app` | docs | 5 |
| 585 | `oya-foundry-staging-promoter` | PLATFORM | `oya-intelligence-staging-promoter` | docs | 5 |
| 586 | `oya-foundry-step-adapter-file` | PLATFORM | `oya-intelligence-step-adapter-file` | code/docs | 5 |
| 587 | `oya-foundry-step-kernel` | PLATFORM | `oya-intelligence-step-kernel` | code/docs | 5 |
| 588 | `oya-foundry-subagent-app` | PLATFORM | `oya-intelligence-subagent-app` | code | 5 |
| 589 | `oya-foundry-subagent-runtime-` | PLATFORM | `oya-intelligence-subagent-runtime-` | code | 5 |
| 590 | `oya-foundry-supervisor` | PLATFORM | `oya-intelligence-supervisor` | BUCK/Cargo/code/docs/registry | 5 |
| 591 | `oya-foundry-supervisor-` | PLATFORM | `oya-intelligence-supervisor-` | code/docs/registry | 5 |
| 592 | `oya-foundry-supervisor-adapter-claude-code` | PLATFORM | `oya-intelligence-supervisor-adapter-claude-code` | docs | 5 |
| 593 | `oya-foundry-supervisor-adapter-codex-cli` | PLATFORM | `oya-intelligence-supervisor-adapter-codex-cli` | docs | 5 |
| 594 | `oya-foundry-supervisor-adapter-cron-tokio` | PLATFORM | `oya-intelligence-supervisor-adapter-cron-tokio` | docs | 5 |
| 595 | `oya-foundry-supervisor-adapter-gemini-cli` | PLATFORM | `oya-intelligence-supervisor-adapter-gemini-cli` | docs | 5 |
| 596 | `oya-foundry-supervisor-adapter-jsonl` | PLATFORM | `oya-intelligence-supervisor-adapter-jsonl` | docs | 5 |
| 597 | `oya-foundry-supervisor-adapter-webhook-hyper` | PLATFORM | `oya-intelligence-supervisor-adapter-webhook-hyper` | docs | 5 |
| 598 | `oya-foundry-supervisor-admission-webhook` | PLATFORM | `oya-intelligence-supervisor-admission-webhook` | registry | 5 |
| 599 | `oya-foundry-supervisor-agent-fleet-lifecycle-adapter` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-adapter` | code/registry | 5 |
| 600 | `oya-foundry-supervisor-agent-fleet-lifecycle-adapter-k8s-operator` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-adapter-k8s-operator` | code/registry | 5 |
| 601 | `oya-foundry-supervisor-agent-fleet-lifecycle-adapter-postgres` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-adapter-postgres` | code/registry | 5 |
| 602 | `oya-foundry-supervisor-agent-fleet-lifecycle-api` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-api` | code/registry | 5 |
| 603 | `oya-foundry-supervisor-agent-fleet-lifecycle-app` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-app` | code/registry | 5 |
| 604 | `oya-foundry-supervisor-agent-fleet-lifecycle-domain` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-domain` | code/docs/registry | 5 |
| 605 | `oya-foundry-supervisor-agent-fleet-lifecycle-kernel` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-kernel` | code/registry | 5 |
| 606 | `oya-foundry-supervisor-agent-fleet-lifecycle-rest` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-rest` | code/registry | 5 |
| 607 | `oya-foundry-supervisor-agent-fleet-lifecycle-sdk` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-sdk` | code/registry | 5 |
| 608 | `oya-foundry-supervisor-agent-fleet-lifecycle-usecase` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-usecase` | code/registry | 5 |
| 609 | `oya-foundry-supervisor-agent-fleet-lifecycle-worker` | PLATFORM | `oya-intelligence-supervisor-agent-fleet-lifecycle-worker` | code/registry | 5 |
| 610 | `oya-foundry-supervisor-app` | PLATFORM | `oya-intelligence-supervisor-app` | docs | 5 |
| 611 | `oya-foundry-supervisor-autonomy-policy-enforcement-adapter` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-adapter` | code/registry | 5 |
| 612 | `oya-foundry-supervisor-autonomy-policy-enforcement-api` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-api` | code/registry | 5 |
| 613 | `oya-foundry-supervisor-autonomy-policy-enforcement-app` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-app` | code/registry | 5 |
| 614 | `oya-foundry-supervisor-autonomy-policy-enforcement-domain` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-domain` | code/registry | 5 |
| 615 | `oya-foundry-supervisor-autonomy-policy-enforcement-kernel` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-kernel` | code/docs/registry | 5 |
| 616 | `oya-foundry-supervisor-autonomy-policy-enforcement-rest` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-rest` | code/registry | 5 |
| 617 | `oya-foundry-supervisor-autonomy-policy-enforcement-sdk` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-sdk` | code/registry | 5 |
| 618 | `oya-foundry-supervisor-autonomy-policy-enforcement-usecase` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-usecase` | code/registry | 5 |
| 619 | `oya-foundry-supervisor-autonomy-policy-enforcement-worker` | PLATFORM | `oya-intelligence-supervisor-autonomy-policy-enforcement-worker` | docs | 5 |
| 620 | `oya-foundry-supervisor-bench` | PLATFORM | `oya-intelligence-supervisor-bench` | docs | 5 |
| 621 | `oya-foundry-supervisor-capability-deployment-` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-` | code/registry | 5 |
| 622 | `oya-foundry-supervisor-capability-deployment-adapter` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-adapter` | code/registry | 5 |
| 623 | `oya-foundry-supervisor-capability-deployment-adapter-postgres` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-adapter-postgres` | code/registry | 5 |
| 624 | `oya-foundry-supervisor-capability-deployment-api` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-api` | code/registry | 5 |
| 625 | `oya-foundry-supervisor-capability-deployment-app` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-app` | code/registry | 5 |
| 626 | `oya-foundry-supervisor-capability-deployment-domain` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-domain` | code/registry | 5 |
| 627 | `oya-foundry-supervisor-capability-deployment-kernel` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-kernel` | code/registry | 5 |
| 628 | `oya-foundry-supervisor-capability-deployment-rest` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-rest` | code/registry | 5 |
| 629 | `oya-foundry-supervisor-capability-deployment-sdk` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-sdk` | code/registry | 5 |
| 630 | `oya-foundry-supervisor-capability-deployment-usecase` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-usecase` | code/registry | 5 |
| 631 | `oya-foundry-supervisor-capability-deployment-worker` | PLATFORM | `oya-intelligence-supervisor-capability-deployment-worker` | code/registry | 5 |
| 632 | `oya-foundry-supervisor-conformance` | PLATFORM | `oya-intelligence-supervisor-conformance` | code/docs/registry | 5 |
| 633 | `oya-foundry-supervisor-controller` | PLATFORM | `oya-intelligence-supervisor-controller` | registry | 5 |
| 634 | `oya-foundry-supervisor-domain` | PLATFORM | `oya-intelligence-supervisor-domain` | docs | 5 |
| 635 | `oya-foundry-supervisor-kernel` | PLATFORM | `oya-intelligence-supervisor-kernel` | docs | 5 |
| 636 | `oya-foundry-supervisor-kill-switch-circuit-breaker-adapter` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-adapter` | code/registry | 5 |
| 637 | `oya-foundry-supervisor-kill-switch-circuit-breaker-adapter-k8s-operator` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-adapter-k8s-operator` | code/registry | 5 |
| 638 | `oya-foundry-supervisor-kill-switch-circuit-breaker-api` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-api` | code/registry | 5 |
| 639 | `oya-foundry-supervisor-kill-switch-circuit-breaker-app` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-app` | code/registry | 5 |
| 640 | `oya-foundry-supervisor-kill-switch-circuit-breaker-domain` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-domain` | code/docs/registry | 5 |
| 641 | `oya-foundry-supervisor-kill-switch-circuit-breaker-kernel` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-kernel` | code/registry | 5 |
| 642 | `oya-foundry-supervisor-kill-switch-circuit-breaker-rest` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-rest` | code/registry | 5 |
| 643 | `oya-foundry-supervisor-kill-switch-circuit-breaker-sdk` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-sdk` | code/registry | 5 |
| 644 | `oya-foundry-supervisor-kill-switch-circuit-breaker-usecase` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-usecase` | code/registry | 5 |
| 645 | `oya-foundry-supervisor-kill-switch-circuit-breaker-worker` | PLATFORM | `oya-intelligence-supervisor-kill-switch-circuit-breaker-worker` | code/registry | 5 |
| 646 | `oya-foundry-supervisor-kms` | PLATFORM | `oya-intelligence-supervisor-kms` | registry | 5 |
| 647 | `oya-foundry-supervisor-kustomize-bootstrap` | PLATFORM | `oya-intelligence-supervisor-kustomize-bootstrap` | registry | 5 |
| 648 | `oya-foundry-supervisor-live-smoke` | PLATFORM | `oya-intelligence-supervisor-live-smoke` | docs | 5 |
| 649 | `oya-foundry-supervisor-pack-kr-wal` | PLATFORM | `oya-intelligence-supervisor-pack-kr-wal` | registry | 5 |
| 650 | `oya-foundry-supervisor-postgres` | PLATFORM | `oya-intelligence-supervisor-postgres` | registry | 5 |
| 651 | `oya-foundry-supervisor-redis` | PLATFORM | `oya-intelligence-supervisor-redis` | registry | 5 |
| 652 | `oya-foundry-supervisor-runtime` | PLATFORM | `oya-intelligence-supervisor-runtime` | docs | 5 |
| 653 | `oya-foundry-supervisor-supervision-event-bus-adapter` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-adapter` | code/registry | 5 |
| 654 | `oya-foundry-supervisor-supervision-event-bus-api` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-api` | code/registry | 5 |
| 655 | `oya-foundry-supervisor-supervision-event-bus-app` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-app` | code/registry | 5 |
| 656 | `oya-foundry-supervisor-supervision-event-bus-kernel` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-kernel` | code/registry | 5 |
| 657 | `oya-foundry-supervisor-supervision-event-bus-sdk` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-sdk` | code/registry | 5 |
| 658 | `oya-foundry-supervisor-supervision-event-bus-usecase` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-usecase` | code/registry | 5 |
| 659 | `oya-foundry-supervisor-supervision-event-bus-worker` | PLATFORM | `oya-intelligence-supervisor-supervision-event-bus-worker` | code/registry | 5 |
| 660 | `oya-foundry-task-supervisor` | PLATFORM | `oya-intelligence-task-supervisor` | docs | 5 |
| 661 | `oya-foundry-telemetry-app` | PLATFORM | `oya-intelligence-telemetry-app` | docs | 5 |
| 662 | `oya-foundry-timescaledb-extension` | PLATFORM | `oya-intelligence-timescaledb-extension` | registry | 5 |
| 663 | `oya-foundry-tool-runner` | PLATFORM | `oya-intelligence-tool-runner` | code | 5 |
| 664 | `oya-foundry-trace` | PLATFORM | `oya-intelligence-trace` | docs | 5 |
| 665 | `oya-foundry-trace-` | PLATFORM | `oya-intelligence-trace-` | docs | 5 |
| 666 | `oya-foundry-translation-drafter` | PLATFORM | `oya-intelligence-translation-drafter` | docs | 5 |
| 667 | `oya-foundry-trigger-dsl` | PLATFORM | `oya-intelligence-trigger-dsl` | code/docs | 5 |
| 668 | `oya-foundry-trigger-dsl-` | PLATFORM | `oya-intelligence-trigger-dsl-` | code/docs | 5 |
| 669 | `oya-foundry-trigger-dsl-domain` | PLATFORM | `oya-intelligence-trigger-dsl-domain` | docs | 5 |
| 670 | `oya-foundry-trigger-dsl-kernel` | PLATFORM | `oya-intelligence-trigger-dsl-kernel` | code/docs | 5 |
| 671 | `oya-foundry-trigger-dsl-runtime` | PLATFORM | `oya-intelligence-trigger-dsl-runtime` | code/docs | 5 |
| 672 | `oya-foundry-usage-window-kernel` | PLATFORM | `oya-intelligence-usage-window-kernel` | docs | 5 |
| 673 | `oya-foundry-ux-profile-adapter` | PLATFORM | `oya-intelligence-ux-profile-adapter` | docs | 5 |
| 674 | `oya-foundry-verifier-app` | PLATFORM | `oya-intelligence-verifier-app` | docs | 5 |
| 675 | `oya-foundry-workflow-engine` | PLATFORM | `oya-intelligence-workflow-engine` | docs | 5 |
| 676 | `oya-foundry-cli-` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 677 | `oya-foundry-dashboard-` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 678 | `oya-foundry-e2e-` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 679 | `oya-foundry-gate-` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 680 | `oya-foundry-gate-domain` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 681 | `oya-foundry-gate-kernel` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 682 | `oya-foundry-shared-` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |
| 683 | `oya-foundry-supply-app` | RE-HOME-DEFERRED | `HOLD — founder per-ident target` | docs | 6 |

## Methodology + caveats (verify-each-step)

- **Enumeration command (exact):** `git grep -hoIE 'oya-foundry-[a-z0-9-]+' | sort -u` on `cleanup/whole-tree-2026-06-07`. Build artifacts (`buck-out/`, `target/`) are git-untracked → excluded automatically. `.omc/` IS included in enumeration (idents there are real) but its *line scrub* is deferred to Batch 7.
- **683 vs ~655 drift:** grep captures compound-token prefixes ending in `-` (e.g. `oya-foundry-cli-`, `oya-foundry-provider-`, `oya-foundry-eval-`) AS WELL AS their full forms (`oya-foundry-cli-dev-runtime`). These prefix-stubs are not separate crates; the executor MUST de-dupe prefix-stubs against their longest completion before counting renames. Net distinct *crate-grade* idents ≈ 655 once stubs collapse.
- **Signal classification is deterministic but suffix-only:** a token is routed by the first matching signal in vcs→fitness→platform order. Per plan §2 hard rule, the executor re-confirms each row against full-line context before mutating; this table is the adjudication INPUT, not a license to batch-rename.
- **NO source mutation occurred.** Read-only pass. All counts are live from the tree on the date above.

