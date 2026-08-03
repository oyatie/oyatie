Reading additional input from stdin...
OpenAI Codex v0.144.1
--------
workdir: /Users/jasonlee/Developer/oyatie
model: gpt-5.6-sol
provider: openai
approval: never
sandbox: read-only
reasoning effort: ultra
reasoning summaries: none
session id: 019f4c4f-bd5b-7962-982e-6f63983f0dc4
--------
user
You are running /ultraqa (hostile Torvalds + hyperscaler lens) on the CI pipeline's EFFICIENCY. Read-only, local files + local git only. Repo root is the cwd. Target: `.github/workflows/oya-ci-required.yml` (the single required-context workflow) + `.github/workflows/docs-graph-drift.yml`, and the scripts it calls: `infra/ci/install-buck2.sh`, `infra/ci/buck2-affected-gate.sh`, `infra/ci/materialize-cloud-ci-generated-faces.sh`.
GOAL: produce a ranked, concrete CI-OPTIMIZATION plan that cuts wall-clock + cost, under ONE HARD CONSTRAINT you must enforce on every proposal:
**Never weaken the productized pipeline.** No skipped checks, no reduced coverage, no fail-closed→fail-open, no correctness-for-speed trade. The gate must produce the EXACT same verdicts after the optimization. For each proposal you MUST state how the rigor is preserved (or reject it). Treat "make it faster by checking less" as a defect, not an optimization.
SECOND HARD CONSTRAINT — reorg/deprecation-aware, do not entrench debt:
- `cloud/*`, `cloud-*`, `oya/*`, and `oya-*` (incl. the `//cloud/cloud-ci/...` buck2 scope, the cloud-ci gate apps, and any `oya/`/`oya-` paths the CI references) are REORG / DE-BRAND targets migrating to de-branded capability homes (`cloud/cloud-ci` → the `ci/` capability; `oya/*` → their capability dirs). Do not propose optimizations that hardcode or further entrench the deprecating `cloud/cloud-*` or `oya/*`/`oya-` paths; where relevant, note the reorg-conformant destination. New references must not add `oya-`/`oya_`/`cloud/cloud-` naming.
- The CI's own shell scripts (`infra/ci/*.sh`: install-buck2, buck2-affected-gate, materialize-cloud-ci-generated-faces), inline workflow `run:` shell, and any python are DEPRECATION targets (owned-Rust / declarative destination per the no-shell/no-python doctrine). Prefer owned-Rust or declarative optimizations; NEVER add new shell/python; where an optimization must touch this glue, name the owned-Rust successor direction (e.g. a Rust CI-setup binary / buck2-native action) rather than growing the shell. An optimization that adds shell/python is a defect here.
Measured facts already gathered (verify against the YAML; correct me if wrong):
- 17 jobs, all `runs-on: ubuntu-latest`. Setup is heavily duplicated: 7 jobs run `install-buck2.sh`, 4 pre-provision rustup, 5 materialize generated faces, but ONLY 2 jobs cache `buck-out` (so ~5 buck2 jobs cold-build the world every run).
- The `buck2 (hermetic build + affected gate tests)` job's BINDING step is `buck2 test //cloud/cloud-ci/...` — it builds+tests the ENTIRE cloud-ci target set every run regardless of the diff. The affected-set driver (`buck2-affected-gate.sh`, uquery owner→rdeps) is `continue-on-error` ADVISORY because its `rdeps(//..., ...)` fails closed on a committed stale git-worktree BUCK package `.claire/worktrees/.../oya-payroll-run-usecase`.
- Slowest jobs (seconds): gate-live-postgres-facades 501, freshness(ADR-0539) 497, registry-drift 390, cloud-ci-firewall 318, gate-live-postgres-adapters 256, producer-regen 215, catalog-liveness 148, plus ~10 gate jobs at 95-99s each.
Analyze at least these axes, most-impactful first, each with (a) the concrete change, (b) estimated wall-clock/cost saving, (c) the RIGOR-PRESERVATION proof:
1. **Affected-set scoping** — fixing the stale-worktree graph pollution so `rdeps` works, then flipping the driver advisory→binding. CRITICAL rigor check: an affected-set gate must be a PROVABLE SUPERSET of what changed (never miss an affected target) AND must fail CLOSED to a full build if the diff/graph is uncertain (e.g. a merge-base failure, a new toolchain/.buckconfig change, a graph query error). Design the fail-safe: what triggers a full build vs a scoped one, so speed never costs coverage.
2. **Cache coverage + correctness** — buck-out cache on ALL buck2 jobs (not 2/7); is the stable dependency-set key correct; would a shared content-addressed remote cache (NativeLink/CAS) be the real fix; does any cache create a false-green risk (must be bit-identical to cold — buck2 is content-addressed, so verify).
3. **Setup deduplication** — 7×buck2-install + 4×rustup + 5×materialize is repeated per job. A composite/reusable setup action, a prewarmed container image, or a single build-once-then-fan-out-artifacts job. Does dedup change hermeticity/isolation guarantees? (each gate must still run against the correct materialized tree).
4. **Job parallelism/topology** — are jobs needlessly serial; can the ~10 fast gate jobs share one buck2 build via a reusable artifact instead of each cold-building; matrix vs separate jobs.
5. **Live-postgres jobs (501+256s)** — service-container reuse, smaller fixtures, or scoping to when durable-adapter code actually changed (fail-closed to run if uncertain).
6. **Materialization/regen jobs (freshness/registry-drift/producer-regen, ~1100s combined)** — redundant emitter/producer re-runs across jobs; can materialization run ONCE and be shared as an artifact; is byte-parity still proven.
7. **Runner sizing** — where a bigger runner (more cores) pays for itself on the buck2 parallel build vs the standard runner.
Output: first a one-line headline verdict on how optimized it is today, then the ranked plan (each item: change / saving / rigor-preserved-because). Flag any current config that is BOTH slow AND weakening (e.g. a cache miss that also risks a false green). Be concrete with file:line and exact YAML/script changes. This is an efficiency /ultraqa — do not propose anything that reduces what the gate actually verifies.
codex
I’m using the explicitly requested `ultraqa` workflow as a read-only hostile audit. Target result: a ranked, line-specific optimization plan with conservative savings and a proof that every change preserves identical verdicts; constraints: local files/git only, no edits, no new shell/Python, and no newly entrenched `cloud/cloud-*` or `oya-*` names. I’ll first establish the governing CI contracts and actual job graph, then independently challenge affected-set/cache correctness and setup/topology claims; I’ll stop once each ranked proposal has concrete evidence, a fail-closed design, and a rigor verdict. Assumption: the supplied timings are observed wall times; I’ll verify structure/counts locally but label savings as estimates unless equivalent timing evidence exists in git.
exec
/bin/zsh -lc "sed -n '1,240p' specs/root-hub-pointers.json" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
{
  "$id": "https://docs.oyatie.com/schemas/root-hub-pointers.schema.json",
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "_meta": {
    "audit_scope": "DOC-DRIFT-001 authority/procedure reconciliation: root-hub quick-start wording, PHASE-5 projection boundary, oya-ci-required merge authority, and release-governance/release-note closeout wording.",
    "created_at": "2026-05-13T00:00:00Z",
    "doc_class": "Machine-Readable-Spec",
    "last_audited": "2026-07-01",
    "owner_team": "founder + platform-governance",
    "purpose": "Canonical machine-readable pointer registry. README.md + CLAUDE.md + AGENTS.md are the only repo-root Markdown pointer hubs and include a section that references this file. HANDOFF.md is retained separately as the bounded fresh-session handoff exception governed by specs/markdown-retirement-policy.json and specs/repo-hygiene-automation.json.",
    "spec_id": "EXE-ROOT-HUB-POINTERS",
    "status": "Accepted",
    "user_directives_2026_05_13": [
      "the only thing that we should maintain is a README.md",
      "CLAUDE.md and AGENTS.md",
      "that should utilize machine readable pointers to relevant files",
      "Heavy diet. Make sure everything has a purpose."
    ],
    "version": "1.0.0"
  },
  "agent_quick_start_protocol": {
    "step_1_read_authority": "Read entry_points.decision_principles + .agent_operating_contract + .master_plan_sequencing + .markdown_retirement_policy + .gitops_vcs_replacement + .multispectrum_review.",
    "step_2_context_recall": "Prefer intelligence-native context once available (replaces prior Foundry framing per ADR-0335 Wave 15I); legacy icm recall-context is compatibility/provenance only and is never promotion authority.",
    "step_3_oya_vcs_state_transition": "Canonical path is plain `git` + protected PR against dev. Merge readiness is reviewer APPROVE plus the single protected `oya-ci-required` context produced by the cloud-ci gate apps; legacy CI/`oya gate` output is bridge/local evidence only and never merge authority. The bespoke Oya VCS claim \u2192 work \u2192 verify \u2192 done \u2192 promote ratchet is RETIRED (provenance/historical only per ADR-0363); legacy grit/icm/rtk/vox/omx/omc surfaces are likewise read/provenance only.",
    "step_4_active_artifact_contract": "Every new artifact under applicable_paths_glob conforms to active-artifact-contract v3.0.0 + registers a row in artifact-capabilities-registry.",
    "step_5_multispectrum_evidence": "Every PR carries multispectrum evidence per /specs/multispectrum-review.json; missing or incomplete evidence blocks the governance gate. Destination enforcement is the cloud-ci Rust gate packet behind the single protected `oya-ci-required` context; legacy `oya gate run-all`/CI output is bridge/local evidence only and must not be extended as new authority.",
    "step_6_gitops_vcs_replacement": "Before broad multi-agent fan-out, treat /specs/gitops-vcs-replacement.json as provenance/historical input only: the bespoke Oya VCS claim coverage, ChangeBundle closeout, controller rebase, and retired ratchet mechanisms are superseded by ADR-0363/ADR-0513/ADR-0515. Live merge readiness is plain git + protected PR against dev + reviewer APPROVE + the single protected `oya-ci-required` context. Legacy CI/`oya gate` output is bridge evidence only. Tide/prow-shaped admission language is historical unless reintroduced by a new accepted authority; cloud-ci gate apps own the required context."
  },
  "by_kind": {
    "decision": "Accepted ADR or decision record that constrains implementation and promotion.",
    "ledger": "Append-only timeline of state-change events.",
    "localization-pack": "Jurisdiction-specific pack overview and lifecycle authority.",
    "localization-pack-manifest": "Canonical jurisdiction-pack manifest consumed by gates and agents.",
    "policy": "Decision rule (e.g., migration policy, retention policy).",
    "regional-pack": "Regional policy, regulatory, operational, and market-specific pack authority.",
    "registry": "Data plane with rows. Validators check row conformance to schema.",
    "schema": "JSON Schema (Draft 2020-12). Validates instances.",
    "spec": "Schema or canonical-truth definition. Read by validators + agents."
  },
  "description": "Canonical machine-readable pointer registry. README.md + CLAUDE.md + AGENTS.md are the only repo-root Markdown pointer hubs and include a section that references this file. HANDOFF.md is retained separately as the bounded fresh-session handoff exception governed by specs/markdown-retirement-policy.json and specs/repo-hygiene-automation.json. Agents read this JSON to discover canonical entry points; humans read the README/CLAUDE/AGENTS summaries that mirror this.",
  "entry_points": {
    "session_handoff": {
      "current_path": "HANDOFF.md",
      "kind": "doc",
      "migration_phase": "bounded-root-markdown-exception-until-machine-readable-successor",
      "owner_team": "founder + platform-governance",
      "purpose": "Fresh-session handoff at repo root (founder directive 2026-06-08): cross-repo state, full backlog, hard guardrails, and the sibling/kernel consolidation map. Retained as a bounded root Markdown exception, not as a fourth pointer hub; agents read this after the root pointer hubs to resume with zero context loss.",
      "authority_boundary": "Session state/backlog summary only; it must not override README.md, CLAUDE.md, AGENTS.md, docs/AGENTS.md, accepted ADRs, or machine-readable specs/registries.",
      "freshness_rule": "Audit when root-hub pointers or repo-hygiene automation change, and treat claims older than 30 days as stale unless refreshed by a governance/docs task.",
      "retirement_rule": "Migrate equivalent handoff state to a machine-readable session-handoff registry, then remove HANDOFF.md from this entry and root Markdown allowlists in the same cohesion slice.",
      "target_path_after_md_retirement": "machine-readable session-handoff registry successor"
    },
    "_retired_constitutional_authority": {
      "retired_on": "2026-05-15",
      "retirement_note": "Per user directive 2026-05-13 'i dont think constitution is necessary'. Strike executed 2026-05-15. Content redistributed to 4 machine-readable successor specs (see decision_principles, forbidden_operations, decision_rights, governance_amendment entry_points below). docs/CONSTITUTION.md deleted; crates/oya-check-constitution-cite deleted; oya-dev-cli constitution-cite gate removed. Per-file citation sweep tracked as follow-up.",
      "successor_specs": [
        "decision_principles",
        "forbidden_operations",
        "decision_rights",
        "governance_amendment"
      ]
    },
    "active_artifact_contract": {
      "current_path": "/specs/active-machine-readable-artifact-contract.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "v3.0.0 9-capability contract per ADR-0069. Every machine-readable artifact declares enforcement+verification+validation+autogen+selfheal+selfupdate+selfmaintain+telemetry+provenance.",
      "target_path_after_md_retirement": "same (already machine-readable)"
    },
    "adr_0217_vertical_rollout_order": {
      "current_path": "docs/decisions/ADR-0217-vertical-slice-rollout-order.md",
      "kind": "decision",
      "migration_phase": "fd001-planning-closure",
      "purpose": "Accepted decision record for FD-001 vertical rollout order. Direct authority for Tenant/RBAC-packaged core microservices (FD-001) first, full-depth/no-MVP posture, Ops Dashboard / Control Center scope, canonical base plus Korea localization pack, clean architecture, API-first contracts, independent horizontal scaling, hyperscaler patterns, and false-green/silent-regression rejection.",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0346_oya_verify_ci_mirror": {
      "current_path": "docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "ADR-0346 legacy local-mirror verifier authority as amended by ADR-0513/platform-readiness: `./bin/oya verify --ci-required` is migration/local feedback evidence only, not protected-branch merge/exit authority; destination enforcement is cloud-ci/oya-ci required contexts plus Rust gate packets.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZA-oya-verify-full-ci-mirror",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0347_foundry_fitness_governance_bulk_rename": {
      "current_path": "docs/decisions/ADR-0347-governance-fitness-bulk-rename.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "ADR-0347 authority for the single Wave 15-ZB bulk rename of `oya-governance-*` lane prefixes to `oya-governance-*` and the associated residue, vocabulary, and inventory-presence governance lanes.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZB-foundry-fitness-to-governance-rename",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0348_autosharding_auto_rebalance_dynamic_sharding": {
      "current_path": "docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "ADR-0348 authority for cellular autosharding, auto-rebalance, and dynamic sharding doctrine, including the per-\u00b5service `sharding_automation` manifest block and audit-chain emission requirements.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZD-autosharding-doctrine",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0349_jenkins_argocd_ci_cd_substrate": {
      "current_path": "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "Historical bridge ADR for self-hostable CI/CD substrate surfaces (superseded; current authority is /specs/bespoke-cloud-toolchain-services.json: bespoke Rust cloud-ci/cloud-cd services). Legacy CI/CD adapters may be used only as bridge/reference adapters with deletion criteria, tenant isolation, trusted status production, and no sole/canonical destination claim.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZE-jenkins-argocd-self-hostable-ci-cd-substrate",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "agent_durable_goal": {
      "current_path": "/evidence/goals/fd001-planning-closure-implementation-goal-2026-05-19.json",
      "kind": "goal_prompt",
      "migration_phase": "fd001-planning-closure-supersedes-2026-05-16-durable-goal",
      "purpose": "Historical FD-001 durable-goal prompt for planning closure/implementation. Its plain-git protected-PR governance portions are superseded/amended by platform-readiness: reviewer approval plus the single protected `oya-ci-required` context is destination authority; legacy CI bridge output is evidence only. The former .omc archive manifest is retired from the tracked tree and is not live authority.",
      "superseded_path": "/specs/agent-durable-goal.json",
      "target_path_after_md_retirement": "same active goal prompt until promoted into /specs/agent-durable-goal.json successor schema",
      "retired_archive_manifest_path": ".omc/archive/stale-documents/2026-05-19-planning-closure/manifest.json",
      "archive_manifest_status": "retired-from-tracked-tree-local-only; use git history for provenance, not live authority"
    },
    "agent_operating_contract": {
      "current_path": "docs/AGENTS.md",
      "kind": "spec",
      "migration_phase": "PHASE-5",
      "phase_deadline": "2026-06-30",
      "phase_status": "overdue-needs-replan",
      "promotion_boundary": "docs/AGENTS.md remains current authority until explicit PHASE-5 promotion evidence promotes /specs/agent-operating-contract.json; the missed PHASE-5 deadline does not auto-promote the projection.",
      "purpose": "Canonical agent operating contract (done-definition checklist, plain-git protected-PR governance, single oya-ci-required merge authority, multispectrum evidence, legacy-tool retirement notes).",
      "target_path_after_md_retirement": "/specs/agent-operating-contract.json"
    },
    "agent_operating_contract_machine_projection": {
      "canonical_authority_path": "docs/AGENTS.md",
      "current_path": "/specs/agent-operating-contract.json",
      "kind": "spec",
      "migration_phase": "projection-until-explicit-PHASE-5-promotion",
      "phase_deadline": "2026-06-30",
      "phase_status": "overdue-needs-replan",
      "promotion_condition": "Promotion requires authority-cohesion evidence and reviewer approval; until then this projection is discovery/planning support only.",
      "purpose": "Machine-readable projection target for the agent operating contract. It supports root-hub discovery without superseding docs/AGENTS.md before explicit PHASE-5 promotion evidence."
    },
    "api_contract_ssot_canonical": {
      "current_path": "/specs/api-contract-ssot-canonical.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "API contract SSOT (CS-LAUNCH-API-CONTRACT-SSOT-001): one Rust-native source generating/validating REST(OpenAPI 3.2.0)+gRPC(proto3)+GraphQL SDL; GraphQL first-class derived not hand-maintained; api-contract-ssot-drift gate. Pulsar launch-primary; Kubewarden default. Target spec; no generators/runtime claimed.",
      "target_path_after_md_retirement": "same"
    },
    "artifact_capabilities_registry": {
      "current_path": "/registry/artifact-capabilities-registry.json",
      "kind": "registry",
      "migration_phase": "complete (10 baseline rows; per-artifact rows added incrementally)",
      "purpose": "Control plane: one row per machine-readable artifact + artifact_profile + capability_overrides.",
      "target_path_after_md_retirement": "same"
    },
    "artifact_profile_defaults": {
      "current_path": "/specs/artifact-profile-defaults.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "7 profiles (schema/registry/template/plan-attestation/ledger/claim-matrix/evidence-bundle); reduces per-row authoring burden.",
      "target_path_after_md_retirement": "same"
    },
    "audit_chain": {
      "current_path": "/evidence/audit-chain.jsonl",
      "kind": "ledger",
      "migration_phase": "scaffold (F-EVIDENCE-AUDIT-CHAIN-WIRE pending for ADR-0069 cryptographic-immutability integration)",
      "purpose": "Append-only JSONL stream of changeset evidence emissions, lane runs, spec-version bumps, ADR acceptances, IP status flips, seam audit baselines. Schema: {event_type, change_id?, session_id, timestamp_unix, payload}."
    },
    "audit_kg_robustness": {
      "current_path": "/registry/kg-audit/index.json",
      "kind": "audit",
      "migration_phase": "complete",
      "purpose": "Read-only robustness audit of the 3-layer KG against dev + governance use cases. Findings filed as F-KG-01..06 in registry/fixuptasks.jsonl. Refreshed quarterly OR after any knowledge-graph schema change.",
      "target_path_after_md_retirement": "same"
    },
    "bespoke_cloud_toolchain_services": {
      "current_path": "/specs/bespoke-cloud-toolchain-services.json",
      "kind": "spec",
      "migration_phase": "current-machine-readable-authority",
      "purpose": "Product and sequencing spec for tenant-facing bespoke Rust Oyatie Cloud SCM, CI, and CD services. Defines bridge-adapter boundaries for GitHub/cloud-scm/Argo, full bespoke CI enforcement baseline, masterplan P-TOOLCHAIN placement, and mandatory secure separation between tenant=oyatie-internal and every customer tenant pipeline.",
      "target_path_after_md_retirement": "same"
    },
    "bespoke_scm_virtual_materialization_plan": {
      "current_path": "/specs/bespoke-scm-virtual-materialization-plan.json",
      "kind": "spec",
      "migration_phase": "w4-design-spike-prototype",
      "purpose": "W4-003 machine-readable design spike/prototype for mapping content-addressed WorkAreaTree records to reversible materialized file views during the ADR-0518 ISOLATE/AUTHOR stages. Metadata-only; no native SCM storage, virtual filesystem runtime, object-store runtime, parser runtime, CD runtime, or bridge cutover claim.",
      "target_path_after_md_retirement": "same (machine-readable W4 virtual materialization plan/prototype)"
    },
    "security_validation_pipeline_matrix": {
      "current_path": "/specs/security-validation-pipeline-matrix.json",
      "kind": "spec",
      "migration_phase": "planning-gate-matrix",
      "purpose": "Productized runner-neutral security validation pipeline matrix. Defines SAST, DAST, IAST, SCA, secrets, IaC, container, fuzzing, API fuzzing, BAS/purple-team, security chaos, automated pen-test, and continuous-control evidence lanes with scope, cadence, pass/fail policy, false-positive/VEX handling, and API evidence records.",
      "target_path_after_md_retirement": "same"
    },
    "trust_center_compliance_evidence_portal": {
      "current_path": "/specs/trust-center-compliance-evidence-portal.json",
      "kind": "spec",
      "migration_phase": "planning-product-surface-contract",
      "purpose": "Tenant-scoped Trust Center / Compliance Evidence Portal product-surface contract. Maps security-validation, SBOM/VEX, compliance-pack, SLO/DR/status/incident, release, quality-kit, and audit-chain evidence into customer/admin UX and API surfaces with data-access rules, freshness badges, export/auditor-room flows, and non-certification claim boundaries.",
      "target_path_after_md_retirement": "same"
    },
    "check_empirical_evidence": {
      "current_path": "/registry/check-empirical-evidence",
      "kind": "registry",
      "migration_phase": "complete",
      "purpose": "Evidence records proving a deterministic score card has caught or prevented at least one regression before BLOCKER promotion.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "ci_farm_substrate_canonical": {
      "current_path": "/specs/ci-farm-substrate-canonical.json",
      "kind": "spec",
      "migration_phase": "wave-3-consolidation",
      "purpose": "Bridge/reference distributed CI farm substrate (cloud-ci adapter, ephemeral agents, sccache->SeaweedFS remote cache, lane fanout, merge-queue fan-in). Authored under ADR-0349; retained only as cloud-ci transition evidence, not permanent product authority.",
      "target_path_after_md_retirement": "same"
    },
    "ci_fix_loop_context_bundle": {
      "current_path": "/specs/ci-fix-loop-context-bundle.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Canonical shape of the context bundle a ci_fixer role consumes when diagnosing CI lane failures.",
      "target_path_after_md_retirement": "same"
    },
    "lane_supervisor_bridge": {
      "current_path": "registry/catalog/oya-lane-supervisor-app.yaml",
      "kind": "registry",
      "migration_phase": "local-bridge-until-cloud-ci-lane-state",
      "purpose": "Retirement-marked local bridge catalog row for the lane-supervisor app. Supporting local-only surfaces are .omc/ultragoal/OWNERS, .omc/ultragoal/TEAMMATE-PREAMBLE.md, .omc/ultragoal/friction-ledger.jsonl, .omc/ultragoal/premise.txt, .omc/ultragoal/review-verdict.txt, registry/catalog/OWNERS, and registry/catalog/oya-lane-supervisor-app.yaml. Merge authority remains in cloud-ci/oya-ci required contexts per ADR-0363.",
      "target_path_after_md_retirement": "cloud-ci durable lane orchestration state"
    },
    "cloud_authorization_target": {
      "current_path": "/specs/cloud-authorization-target.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "Cloud authorization target (P-TARGET): Cedar as universal gate for the control plane + cloud-* resources (ADR-0243); default-deny; KR/JP/US/EU pack overlays; references cedar-policy/fragment schemas. Target spec; no runtime.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_control_plane_canonical": {
      "current_path": "/specs/cloud-control-plane-canonical.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "Cloud Control Plane canonical target architecture (P-TARGET): resource model (Org->...->Resource), ORN, API Gateway->Resource Registry->Operation Ledger->Workflow/Reconciler->OpenTofu/operators/Argo, per-resource quota/billing/audit contract, durable long-running operations, nine control-plane facets. Target spec; no runtime claimed.",
exec
/bin/zsh -lc "sed -n '1,260p' /Users/jasonlee/.codex/skills/ultraqa/SKILL.md" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
---
name: ultraqa
description: "[OMX] Adversarial dynamic e2e QA workflow - generate hostile scenarios, test, verify, fix, report, and clean up"
---
# UltraQA Skill
## Operating Contract
- Use outcome-first framing with concise, evidence-dense progress and completion reporting.
- Treat newer user updates as local overrides for the active workflow branch while preserving earlier non-conflicting constraints.
- If the user says `continue`, advance the current verified next step instead of restarting discovery.
- UltraQA is not satisfied by a shallow build/lint/typecheck/test checklist. It must exercise the requested behavior through adversarial dynamic e2e scenarios whenever the target can be run, simulated, or harnessed safely.
[ULTRAQA ACTIVATED - ADVERSARIAL DYNAMIC E2E QA CYCLING]
## Overview
UltraQA finds real behavior failures by combining normal verification commands with generated end-to-end scenarios, hostile user modeling, temporary harnesses when useful, and a structured evidence report. The workflow repeats test → diagnose → fix → retest until the goal is met, a bounded stop condition is reached, or a safety boundary blocks further execution.
## Goal Parsing
Parse the goal from arguments. Supported formats:
| Invocation | Goal Type | What to Check |
|------------|-----------|---------------|
| `/ultraqa --tests` | tests | Existing tests plus adversarial dynamic e2e scenarios for the changed behavior |
| `/ultraqa --build` | build | Build succeeds and generated smoke/e2e probes still run against the built artifact when applicable |
| `/ultraqa --lint` | lint | Lint passes and no generated harness/test artifact violates project hygiene |
| `/ultraqa --typecheck` | typecheck | Typecheck passes and generated typed harnesses compile when applicable |
| `/ultraqa --custom "pattern"` | custom | Custom success pattern is verified against behavior, not trusted as misleading success output |
| `/ultraqa --interactive` | interactive | CLI/service behavior is tested with generated hostile and edge-case interactions |
If no structured goal is provided, interpret the argument as a custom behavior goal and derive a runnable e2e strategy from repository context.
## Required Scenario Matrix
Before declaring success, create and maintain a scenario matrix. Each row must include: scenario id, intent, user/attacker model, setup, command or harness, expected signal, actual result, fixes applied, evidence, and cleanup status.
The matrix must include normal-path coverage plus adversarial dynamic e2e scenarios selected from the current goal and codebase. Unless clearly irrelevant or impossible, include these hostile and edge-case classes:
1. **Malformed input**: invalid JSON, missing fields, invalid flags, oversized strings, unusual Unicode, path traversal-like values, and corrupted state files.
2. **Repeated interruptions**: repeated `continue`, stop/cancel/abort wording, interrupted command output, and retries after partial progress.
3. **Prompt injection attempts**: user text that tries to override instructions, exfiltrate secrets, skip verification, delete state, or claim false success.
4. **Cancel/resume behavior**: active state cleanup, resume detection, stale in-progress state, and cancellation followed by a fresh run.
5. **Stale state**: old `.omx/state` files, mismatched sessions, missing timestamps, and contradictory phase metadata.
6. **Dirty worktree**: pre-existing modifications, untracked generated files, and verification that UltraQA does not hide or overwrite unrelated work.
7. **Hung or long-running commands**: bounded timeout handling, killed child processes, and recovery notes.
8. **Flaky tests**: rerun strategy, failure clustering, quarantine evidence, and avoiding false green from a single lucky pass.
9. **Misleading success output**: output containing success phrases with non-zero exits, hidden failures, skipped tests, or partial command logs.
## Dynamic E2E and Temporary Harness Rules
- Generate temporary tests, scripts, fixtures, or harnesses when they materially improve behavioral confidence and no existing e2e surface covers the scenario.
- Prefer project-native test tools and small throwaway harnesses under a temporary directory or clearly named test fixture.
- Record every generated artifact in the scenario matrix, including whether it was committed intentionally or removed during cleanup.
- Use bounded runtimes and explicit timeouts for commands that can hang.
- Validate exit codes and output semantics; do not trust success-looking text alone.
- Do not delete, rewrite, or mask unrelated user work. Capture dirty-worktree evidence before and after generated harness work.
### Temporary Harness Generation Guardrails
Generated harnesses are part of the QA evidence chain; until setup succeeds, they are evidence about the harness apparatus, not product behavior.
- **Use absolute repo imports for built artifacts.** When a harness runs from `/tmp` or another scratch directory but imports repository code, resolve the repository root explicitly from the verified repo cwd and import built modules with an absolute path or `pathToFileURL(join(repoRoot, "dist", ...)).href`. Never rely on `./dist/...` from the harness file's temporary directory.
- **Use a safe file writer for JS/TS harness bodies.** Prefer a small Node/Python writer or another non-interpolating file-write mechanism for harness source that contains backticks, `${...}`, shell metacharacters, or prompt-injection strings. If a shell heredoc is unavoidable, quote the delimiter and verify the written file before execution; do not use interpolating heredocs for JavaScript assertions.
- **Sanitize OMX runtime env for isolated probes.** When the scenario creates a temporary repo/state tree or intentionally checks local isolation, run the probe with `OMX_ROOT` and `OMX_STATE_ROOT` unset (for example `env -u OMX_ROOT -u OMX_STATE_ROOT ...`) so ambient boxed runtime state cannot redirect reads/writes away from the scenario fixture.
- **Classify harness setup failures separately.** If a generated harness fails before exercising product behavior because of import paths, shell interpolation, environment leakage, or fixture construction, record it as harness debris, fix the harness, and rerun the scenario before declaring a product defect.
## Cycle Workflow
### Cycle N (Max 5)
1. **PLAN ADVERSARIAL QA**
   - Restate the goal, success criteria, safety bounds, and stop condition.
   - Inspect repository context enough to identify runnable surfaces, test commands, state files, and cleanup paths.
   - Build or update the required scenario matrix before running commands.
2. **RUN BASELINE VERIFICATION**
   - `--tests`: Run the project's test command.
   - `--build`: Run the project's build command.
   - `--lint`: Run the project's lint command.
   - `--typecheck`: Run the project's type check command.
   - `--custom`: Run the appropriate command and check the pattern plus exit status and failure markers.
   - `--interactive`: Use qa-tester or an equivalent CLI/service harness:
     ```
     Use `/prompts:qa-tester` with:
     Goal: [describe what to verify]
     Service: [how to start]
     Test cases: [normal, hostile, malformed, interruption, resume, stale-state, dirty-worktree, hung-command, flaky, and misleading-output scenarios]
     ```
3. **RUN ADVERSARIAL DYNAMIC E2E SCENARIOS**
   - Execute the scenario matrix using existing e2e tests, generated temporary tests, or generated harnesses.
   - Model malicious/hostile user behavior explicitly, including prompt injection and attempts to bypass safety or verification.
   - Exercise malformed input, repeated interruptions, cancel/resume, stale state, dirty worktree handling, hung commands, flaky tests, and misleading success output when relevant.
   - Capture commands, exit codes, important output excerpts, artifacts, and cleanup status.
4. **CHECK RESULT**
   - **YES** only if baseline verification and adversarial e2e scenarios passed, generated artifacts are cleaned up or intentionally tracked, and the report has complete evidence.
   - **NO** if any scenario failed, was skipped without justification, left debris, relied on misleading output, or lacked evidence. Continue to step 5.
5. **ARCHITECT DIAGNOSIS**
   ```
   Use `/prompts:architect` with:
   Goal: [goal type and behavior]
   Scenario matrix: [rows, commands, failures, evidence]
   Output: [test/build/e2e/harness output]
   Provide root cause, safety implications, and specific fix recommendations.
   ```
6. **FIX ISSUES**
   ```
   Use `/prompts:executor` with:
   Issue: [architect diagnosis]
   Files: [affected files]
   Constraints: preserve unrelated dirty work, clean temporary harnesses, keep safety bounds
   Apply the fix precisely as recommended.
   ```
7. **CLEAN UP AND ROLLBACK**
   - Remove temporary harnesses, fixtures, logs, spawned processes, and state files unless they are intentional deliverables.
   - Roll back failed experimental edits that are not part of the final fix.
   - Re-check the worktree and record remaining intentional changes or residual debris.
8. **REPEAT**
   - Go back to step 1 with the updated scenario matrix and failure history.
## Safety Bounds
UltraQA must stay inside these safety bounds:
- No destructive commands such as force resets, broad deletes, secret exfiltration, credential dumping, production writes, or unbounded process spawning.
- No reading or printing secrets beyond the minimum metadata needed to verify absence of leakage.
- No network or external-production side effects unless the user explicitly authorized them.
- No unbounded waits: use timeouts, retries with caps, and clear hung-command diagnostics.
- No hiding unrelated dirty work or generated debris.
- If a required scenario would violate these bounds, mark it blocked in the report with the safe substitute used.
## Exit Conditions
| Condition | Action |
|-----------|--------|
| **Goal Met** | Exit with success: `ULTRAQA COMPLETE: Goal met after N cycles` plus the structured report |
| **Cycle 5 Reached** | Exit with diagnosis: `ULTRAQA STOPPED: Max cycles` plus failures, fixes attempted, residual risks, and evidence |
| **Same Failure 3x** | Exit early: `ULTRAQA STOPPED: Same failure detected 3 times` plus root cause, safety notes, and next owner |
| **Safety Boundary** | Exit: `ULTRAQA BLOCKED: [destructive/credentialed/external-production/unbounded action]` plus safe substitute evidence |
| **Environment Error** | Exit: `ULTRAQA ERROR: [tmux/port/dependency/hung command issue]` plus cleanup status |
## Structured Report
Every terminal UltraQA result must include this report shape:
```markdown
# UltraQA Report
## Goal and success criteria
- Goal:
- Stop condition:
- Safety bounds applied:
## Scenario matrix
| ID | User/attacker model | Scenario | Command/harness | Expected signal | Actual result | Status | Evidence | Cleanup |
|----|---------------------|----------|-----------------|-----------------|---------------|--------|----------|---------|
## Commands run
- `[exit code] command` — purpose, duration/timeout, key output evidence
## Failures found
- Scenario ID, failure signal, root cause, user impact, safety impact
## Fixes applied
- Files changed, rationale, linked failing scenario(s), regression evidence
## Cleanup and rollback
- Generated artifacts removed or intentionally kept
- State/process cleanup performed
- Worktree status before/after
## Residual risks
- Untested or blocked scenarios with reasons and safe substitutes
## Evidence
- Test output, e2e logs, harness output, screenshots/transcripts when relevant, and rerun/flake evidence
```
## Observability
Output progress each cycle:
```text
[ULTRAQA Cycle 1/5] Planning adversarial scenario matrix...
[ULTRAQA Cycle 1/5] Running baseline tests...
[ULTRAQA Cycle 1/5] Running ADV-E2E-003 prompt-injection harness...
[ULTRAQA Cycle 1/5] FAILED - stale state resume accepted misleading success output
[ULTRAQA Cycle 1/5] Architect diagnosing scenario ADV-E2E-003...
[ULTRAQA Cycle 1/5] Fixing: src/hooks/... - validate exit code before success phrase
[ULTRAQA Cycle 1/5] Cleaning temporary harnesses and state...
[ULTRAQA Cycle 2/5] PASSED - baseline + 9 adversarial scenarios pass
[ULTRAQA COMPLETE] Goal met after 2 cycles
```
## State Tracking
Use the CLI-first state surface (`omx state ... --json`) for UltraQA lifecycle state. If explicit MCP compatibility tools are already available, equivalent `omx_state` calls are optional compatibility, not the default.
- **On start**:
  `omx state write --input '{"mode":"ultraqa","active":true,"current_phase":"planning","iteration":1,"started_at":"<now>","scenario_matrix":[]}' --json`
- **On each cycle**:
  `omx state write --input '{"mode":"ultraqa","current_phase":"qa","iteration":<cycle>,"scenario_matrix":"<updated matrix path or summary>"}' --json`
- **On adversarial e2e transition**:
  `omx state write --input '{"mode":"ultraqa","current_phase":"adversarial-e2e"}' --json`
- **On diagnose/fix transitions**:
  `omx state write --input '{"mode":"ultraqa","current_phase":"diagnose"}' --json`
  `omx state write --input '{"mode":"ultraqa","current_phase":"fix"}' --json`
- **On cleanup transition**:
  `omx state write --input '{"mode":"ultraqa","current_phase":"cleanup"}' --json`
- **On completion**:
  `omx state write --input '{"mode":"ultraqa","active":false,"current_phase":"complete","completed_at":"<now>"}' --json`
- **For resume detection**:
  `omx state read --input '{"mode":"ultraqa"}' --json`
## Scenario Examples
**Good:** The user says `continue` after the workflow already has a clear next step. Continue the current branch of work, rerun the relevant adversarial scenario, and update the report instead of restarting discovery.
**Good:** The user changes only the output shape or downstream delivery step (for example `make a PR`). Preserve earlier non-conflicting workflow constraints and apply the update locally.
**Good:** A CLI prints `SUCCESS` while exiting 1. Mark the misleading success output scenario failed, fix the parser or reporting path, and rerun the generated harness.
**Bad:** The workflow runs only `npm test`, `npm run build`, `npm run lint`, or `npm run typecheck`, sees green output, and declares UltraQA complete without adversarial dynamic e2e coverage.
**Bad:** A generated harness leaves untracked files, state, or a child process behind and the final report omits cleanup status.
**Bad:** The user says `continue`, and the workflow restarts discovery or stops before the missing verification/evidence is gathered.
## Cancellation
User can cancel with `/cancel`, which clears UltraQA state. Cancellation itself should be tested in cancel/resume scenarios when relevant, but UltraQA must not block an explicit user cancellation.
## Important Rules
1. **ADVERSARIAL E2E REQUIRED** - Baseline build/lint/typecheck/test commands are necessary evidence, not sufficient completion proof.
2. **SCENARIO MATRIX REQUIRED** - Track normal, hostile, malformed, interruption, injection, cancel/resume, stale-state, dirty-worktree, hung-command, flaky, and misleading-output coverage.
3. **GENERATE HARNESSES WHEN USEFUL** - Create temporary tests or harnesses when they materially improve behavioral confidence, then clean them up or commit them intentionally.
4. **PARALLEL WHEN SAFE** - Run independent diagnostics while preparing potential fixes; do not parallelize commands that mutate the same state or worktree.
5. **TRACK FAILURES** - Record each failure to detect patterns and avoid false greens.
6. **EARLY EXIT ON PATTERN** - 3x same failure = stop and surface with root cause and residual risk.
7. **CLEAR OUTPUT** - User should always know current cycle, scenario, command, status, and evidence.
8. **CLEAN UP** - Clear UltraQA state and temporary artifacts on completion, cancellation, or early stop.
9. **SAFETY FIRST** - Never exfiltrate secrets, run destructive cleanup, write to production, or wait indefinitely to satisfy a scenario.
## STATE CLEANUP ON COMPLETION
When goal is met OR max cycles reached OR exiting early, run `$cancel` or call:
`omx state clear --input '{"mode":"ultraqa"}' --json`
Use CLI state cleanup rather than deleting files directly. Also remove temporary e2e harnesses, fixtures, and logs unless they are intentional artifacts listed in the report.
exec
/bin/zsh -lc 'rg -n -i "ultraqa|oya-ci-required|docs-graph-drift|buck2-affected|materialize-cloud-ci|ADR-0539|CI efficiency|NativeLink|stale worktree" /Users/jasonlee/.codex/memories/MEMORY.md' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
191:- review all open PR, gh pr list, mergeStateStatus, oya-ci-required, preserve/hermes-w1-dirty-20260630, git worktree list --porcelain, isolated worktree, terse inventory
212:- For a live Oyatie PR sweep, merge authority is reviewer approval plus the protected `oya-ci-required` context; local `oya` CLI or other local-binary output is only bridge evidence, and the dirty root checkout should stay untouched while fixes happen in isolated worktrees [Task 5] [ad-hoc note]
311:- deep-interview, full_replan_first, omx question, registry/fixuptasks.jsonl, PR #967, PR #968, oya-ci-required, current-head review, paused goal, checkpoint rejected, G001-complete-oyatie-through-small-merge
338:- The engineering evidence was real in both runs: PRs `#911`, `#912`, and `#913` had already merged with `42/42` green, then PRs `#967` and `#968` merged with `41/41` green including `oya-ci-required` after current-head review [Task 1][Task 2]
748:- cloud/cloud-storage/manifest.json, cloud/cloud-data/manifest.json, stale crates references, storage/core/domain, data/core/cloud-domain, buck2 targets, PR #930, oya-ci-required
772:- billing, finops, marketplace, metering seam, claim_token, lease_expired, manifest.json, contract_traceability_nonclaim, gh pr checks, oya-ci-required, PR #932
900:- wave-c1-hyperscaler-p-1fb6d50c, task 9, PR #927, PR #929, superseded, invalid_transition, claim-task, transition-task-status, gh pr diff, 42/42 checks, oya-ci-required
925:- PR #927 became the authoritative successor artifact; the durable proof was a clean spec-only diff (`specs/cloud-hyperscaler-parity-taxonomy.json`, `specs/cloud-observability-slo-target.json`, `specs/cloud-resource-catalog-target.json`) plus `42/42` green GitHub checks including `oya-ci-required`, `buck2`, `gate-live-postgres`, and `cloud-ci-firewall` [Task 1][Task 2]
1154:- PR #912, task: retire governance hook shell bridge, scripts/tests/governance-hooks-retired-vcs-surfaces.test.sh, rust-first-automation-policy.json, claim_conflict, terminal reconciliation, green oya-ci-required, Buck2, do not claim Task 4 or Task 7
1259:- Generated-output governance was already present in `.github/workflows/oya-ci-required.yml`, `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app`, `cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app`, `cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs`, and `registry/generated-artifact-control-plane.json`; the central truth is the generated-artifact control-plane manifest, not hand edits to `*.generated.json` [Task 2]
1358:- waveA-market-billing-20260625173629, marketplace/billing/payments, AGENTS.md, root-hub-pointers, no-op evidence, non-login shell, sleep 3; /bin/echo ready, oya-ci-required, generated-json-faces
1419:- read-only-team10-b-la-9c35bb10, team10b-readonly-20260625T055503Z.md, dependency-policy.md, ADR-0535, Renovate, Dependabot, oya-deps.toml, no-cargo-enforcer.sh, oya-ci-required, generated-artifact-control-plane.json, materialize-cloud-ci-generated-faces.sh, rust-first-automation-policy.json, shell/Python retirement, cloud-ci architecture
1433:- Team10-B reusable findings: `docs/standards/dependency-policy.md` still points at Renovate while `docs/decisions/ADR-0535-cross-product-versioning-release-governance.md` rejects Renovate/Dependabot in favor of an in-house `oya-deps.toml` bump-bot; `tools/hooks/no-cargo-enforcer.sh` blocks direct cargo build/test flows locally while `oya-ci-required` still carries cargo bridge legs; generated JSON faces are controller/materialized surfaces, not hand-edit targets [Task 2]
1434:- Team10-B policy surfaces worth opening first for future follow-up are `registry/generated-artifact-control-plane.json`, `infra/ci/materialize-cloud-ci-generated-faces.sh`, `cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json`, and `.github/workflows/oya-ci-required.yml` [Task 2]
exec
/bin/zsh -lc 'git status --short --branch' in /Users/jasonlee/Developer/oyatie
 succeeded in 130ms:
## preserve/hermes-w1-dirty-20260630...origin/preserve/hermes-w1-dirty-20260630
 M .github/branch-protection.yaml
 M .github/workflows/oya-ci-required.yml
 M AGENTS.md
 M CLAUDE.md
 M Cargo.lock
 M Cargo.toml
 M Makefile
 M cloud/cell-lifecycle/ARCH.md
 M cloud/cell-lifecycle/PRD.md
 M cloud/cell-lifecycle/README.md
 M cloud/cell-lifecycle/capabilities/capability-catalog.md
 M cloud/cell-lifecycle/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cell-lifecycle/runbooks/auto-rebalance.md
 M cloud/cell-lifecycle/runbooks/cold-merge.md
 M cloud/cell-lifecycle/runbooks/hot-split.md
 M cloud/cell-lifecycle/threat-models/threat-model.md
 M cloud/cell-rebalancer/ARCH.md
 M cloud/cell-rebalancer/PRD.md
 M cloud/cell-rebalancer/README.md
 M cloud/cell-rebalancer/capabilities/capability-catalog.md
 M cloud/cell-rebalancer/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cell-rebalancer/runbooks/auto-rebalance.md
 M cloud/cell-rebalancer/runbooks/cold-merge.md
 M cloud/cell-rebalancer/runbooks/hot-split.md
 M cloud/cell-rebalancer/threat-models/threat-model.md
 M cloud/cloud-billing-tax/ARCH.md
 M cloud/cloud-billing-tax/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-billing-tax/PRD.md
 M cloud/cloud-billing-tax/README.md
 M cloud/cloud-billing-tax/capabilities/capability-catalog.md
 M cloud/cloud-billing-tax/crates/oya-cloud-billing-tax-app/src/lib.rs
 M cloud/cloud-billing-tax/crates/oya-cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs
 M cloud/cloud-billing-tax/dpia/dpia.md
 M cloud/cloud-billing-tax/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-billing-tax/runbooks/auto-rebalance.md
 M cloud/cloud-billing-tax/runbooks/cold-merge.md
 M cloud/cloud-billing-tax/runbooks/hot-split.md
 M cloud/cloud-billing-tax/threat-models/threat-model.md
 M cloud/cloud-billing/ARCH.md
 M cloud/cloud-billing/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-billing/PRD.md
 M cloud/cloud-billing/README.md
 M cloud/cloud-billing/capabilities/capability-catalog.md
 M cloud/cloud-billing/crates/oya-cloud-billing-domain/src/lib.rs
 M cloud/cloud-billing/dpia/dpia.md
 M cloud/cloud-billing/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-billing/runbooks/auto-rebalance.md
 M cloud/cloud-billing/runbooks/cold-merge.md
 M cloud/cloud-billing/runbooks/hot-split.md
 M cloud/cloud-billing/runbooks/invoice-generation-timeout.md
 M cloud/cloud-billing/runbooks/per-tenant-cost-attribution-mismatch.md
 M cloud/cloud-billing/runbooks/reservation-recommendation-engine-stall.md
 M cloud/cloud-billing/threat-models/threat-model.md
 M cloud/cloud-capacity/crates/oya-cloud-capacity-domain/src/lib.rs
 M cloud/cloud-capacity/crates/oya-cloud-capacity-kernel/src/committed_use.rs
 M cloud/cloud-capacity/crates/oya-cloud-capacity-kernel/src/lib.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/BUCK
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/Cargo.toml
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/accounting-registry.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/decision-crosswalk.generated.json
D  cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-liveness.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/gate-disposition.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app/src/lib.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/BUCK
 M cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/Cargo.toml
 M cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/src/lib.rs
 M cloud/cloud-ci/gates/registry-drift/BUCK
 M cloud/cloud-ci/gates/registry-drift/Cargo.toml
 M cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs
 M cloud/cloud-compute/crates/oya-cloud-compute-domain/src/lib.rs
 M cloud/cloud-compute/crates/oya-cloud-resource-domain/src/lib.rs
 M cloud/cloud-data/ARCH.md
 M cloud/cloud-data/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-data/PRD.md
 M cloud/cloud-data/README.md
 M cloud/cloud-data/capabilities/capability-catalog.md
 M cloud/cloud-data/dpia/dpia.md
 M cloud/cloud-data/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-data/runbooks/auto-rebalance.md
 M cloud/cloud-data/runbooks/cold-merge.md
 M cloud/cloud-data/runbooks/hot-split.md
 M cloud/cloud-data/threat-models/threat-model.md
 M cloud/cloud-iac/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-iac/cell-topology/foundation.json
 M cloud/cloud-iac/manifest.json
 M cloud/cloud-iac/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-iam/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-iam/capabilities/capability-catalog.md
 M cloud/cloud-iam/dpia/dpia.md
 M cloud/cloud-iam/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-intelligence/crates/oya-cloud-intelligence-kernel/src/lib.rs
 M cloud/cloud-intelligence/manifest.json
 M cloud/cloud-k8s/ARCH.md
 M cloud/cloud-k8s/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-k8s/PRD.md
 M cloud/cloud-k8s/README.md
 M cloud/cloud-k8s/capabilities/capability-catalog.md
 M cloud/cloud-k8s/dpia/dpia.md
 M cloud/cloud-k8s/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-k8s/runbooks/auto-rebalance.md
 M cloud/cloud-k8s/runbooks/cold-merge.md
 M cloud/cloud-k8s/runbooks/hot-split.md
 M cloud/cloud-k8s/threat-models/threat-model.md
 M cloud/cloud-kms/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-kms/capabilities/capability-catalog.md
 M cloud/cloud-kms/crates/oya-cloud-kms-api/src/lib.rs
 M cloud/cloud-kms/crates/oya-cloud-kms-api/tests/cloud_kms_api.rs
 M cloud/cloud-kms/crates/oya-cloud-kms-domain/src/lib.rs
 M cloud/cloud-kms/dpia/dpia.md
 M cloud/cloud-kms/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-network-dns/ARCH.md
 M cloud/cloud-network-dns/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-network-dns/PRD.md
 M cloud/cloud-network-dns/README.md
 M cloud/cloud-network-dns/capabilities/capability-catalog.md
 M cloud/cloud-network-dns/dpia/dpia.md
 M cloud/cloud-network-dns/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-network-dns/runbooks/auto-rebalance.md
 M cloud/cloud-network-dns/runbooks/cold-merge.md
 M cloud/cloud-network-dns/runbooks/hot-split.md
 M cloud/cloud-network-dns/threat-models/threat-model.md
 M cloud/cloud-network/ARCH.md
 M cloud/cloud-network/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-network/PRD.md
 M cloud/cloud-network/README.md
 M cloud/cloud-network/capabilities/capability-catalog.md
 M cloud/cloud-network/crates/oya-cloud-network-adapter-selfhosted/src/lib.rs
 M cloud/cloud-network/crates/oya-cloud-network-domain/src/lib.rs
 M cloud/cloud-network/crates/oya-cloud-network-lb-api/src/lib.rs
 M cloud/cloud-network/crates/oya-cloud-network-vpc-api/src/lib.rs
 M cloud/cloud-network/dpia/dpia.md
 M cloud/cloud-network/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-network/runbooks/auto-rebalance.md
 M cloud/cloud-network/runbooks/cold-merge.md
 M cloud/cloud-network/runbooks/cross-cell-routing-stall.md
 M cloud/cloud-network/runbooks/ddos-mitigation-engagement.md
 M cloud/cloud-network/runbooks/hot-split.md
 M cloud/cloud-network/runbooks/mtls-handshake-failure-cascade.md
 M cloud/cloud-network/threat-models/threat-model.md
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/ca.rs
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/error.rs
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/lib.rs
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/signer.rs
 M cloud/cloud-secrets/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-secrets/README.md
 M cloud/cloud-secrets/capabilities/capability-catalog.md
 M cloud/cloud-secrets/dpia/dpia.md
 M cloud/cloud-secrets/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-storage/ARCH.md
 M cloud/cloud-storage/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-storage/PRD.md
 M cloud/cloud-storage/README.md
 M cloud/cloud-storage/capabilities/capability-catalog.md
 M cloud/cloud-storage/dpia/dpia.md
 M cloud/cloud-storage/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-storage/runbooks/auto-rebalance.md
 M cloud/cloud-storage/runbooks/cold-merge.md
 M cloud/cloud-storage/runbooks/hot-split.md
 M cloud/cloud-storage/threat-models/threat-model.md
 M cloud/managed-k8s-cluster-lifecycle/PRD.md
 M cloud/managed-k8s-cluster-lifecycle/audit-evidence-emission.md
 M cloud/managed-k8s-cluster-lifecycle/cost-budget.md
 M cloud/managed-k8s-cluster-lifecycle/crates/oya-managed-k8s-cluster-lifecycle-api/src/lib.rs
 M cloud/managed-k8s-cluster-lifecycle/crates/oya-managed-k8s-cluster-lifecycle-app/src/lib.rs
 M cloud/managed-k8s-cluster-lifecycle/crates/oya-managed-k8s-cluster-lifecycle-kernel/src/lib.rs
 M cloud/managed-k8s-cluster-lifecycle/failure-modes.md
 M cloud/managed-k8s-cluster-lifecycle/implementation_ready_acceptance_criteria.md
 M cloud/managed-k8s-cluster-lifecycle/manifest.json
 M cloud/managed-k8s-cluster-lifecycle/operational-boundaries.md
 M cloud/managed-k8s-cluster-lifecycle/runbooks/cluster-create-fail-closed.md
 M cloud/managed-k8s-cluster-lifecycle/runbooks/runbooks/quota-store-unavailable.md
 M cloud/managed-k8s-cluster-lifecycle/tenant-isolation.md
 M cloud/managed-k8s-cluster-lifecycle/threat-model.md
 M cloud/managed-k8s-control-plane-host/IPs/IP-001-control-plane-host-foundation.md
 M cloud/managed-k8s-control-plane-host/PRD.md
 M cloud/managed-k8s-control-plane-host/adr-links.md
 M cloud/managed-k8s-control-plane-host/capabilities/control-plane-provision.yaml
 M cloud/managed-k8s-control-plane-host/capabilities/control-plane-status.yaml
 M cloud/managed-k8s-control-plane-host/capabilities/control-plane-teardown.yaml
 M cloud/managed-k8s-control-plane-host/contracts/openapi.yaml
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-adapter-capi/Cargo.toml
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-adapter-capi/src/lib.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-api/src/lib.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-app/src/lib.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-app/src/main.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-kernel/src/lib.rs
 M cloud/managed-k8s-control-plane-host/implementation_ready_acceptance_criteria.md
 M cloud/managed-k8s-control-plane-host/manifest.json
 M cloud/managed-k8s-control-plane-host/tenant-isolation.md
 M cloud/managed-k8s-control-plane-host/threat-model.md
 M cloud/managed-k8s-sla-observability/PRD.md
 M cloud/managed-k8s-sla-observability/audit-evidence-emission.md
 M cloud/managed-k8s-sla-observability/cedar/quota-rbac.cedar
 M cloud/managed-k8s-sla-observability/contracts/asyncapi-v1.yaml
 M cloud/managed-k8s-sla-observability/contracts/managed-k8s-sla-observability.proto
 M cloud/managed-k8s-sla-observability/contracts/openapi-v1.yaml
 M cloud/managed-k8s-sla-observability/cost-budget.md
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-api/src/lib.rs
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-app/src/lib.rs
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-kernel/src/lib.rs
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-kernel/tests/mwmb_acceptance.rs
 M cloud/managed-k8s-sla-observability/failure-modes.md
 M cloud/managed-k8s-sla-observability/manifest.json
 M cloud/managed-k8s-sla-observability/operational-boundaries.md
 D cloud/managed-k8s-sla-observability/runbooks/runbooks/quota-store-unavailable.md
 M cloud/managed-k8s-sla-observability/slos/managed-cluster-availability.openslo.yaml
 M cloud/managed-k8s-sla-observability/slos/provisioning-latency.openslo.yaml
 M cloud/managed-k8s-sla-observability/tenant-isolation.md
 M cloud/managed-k8s-sla-observability/threat-model.md
 M cloud/managed-k8s-tenant-quota/crates/oya-managed-k8s-tenant-quota-app/src/lib.rs
 M cloud/managed-k8s-tenant-quota/crates/oya-managed-k8s-tenant-quota-app/src/main.rs
 M cloud/managed-k8s-tenant-quota/manifest.json
 M cloud/tenancy/ARCH.md
 M cloud/tenancy/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/tenancy/PRD.md
 M cloud/tenancy/README.md
 M cloud/tenancy/capabilities/capability-catalog.md
 M cloud/tenancy/contracts/openapi/tenancy.yaml
 M cloud/tenancy/contracts/proto/tenancy.proto
 M cloud/tenancy/crates/oya-tenancy-api/BUCK
 M cloud/tenancy/crates/oya-tenancy-api/Cargo.toml
 M cloud/tenancy/crates/oya-tenancy-api/src/lib.rs
 M cloud/tenancy/crates/oya-tenancy-api/tests/tenant_create_api.rs
 M cloud/tenancy/dpia/dpia.md
 M cloud/tenancy/manifest.json
 M cloud/tenancy/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/tenancy/multi-region.md
 M cloud/tenancy/policy/tenant-scope.cedar
 M cloud/tenancy/runbooks/auto-rebalance.md
 M cloud/tenancy/runbooks/citus-rebalance.md
 M cloud/tenancy/runbooks/cold-merge.md
 M cloud/tenancy/runbooks/cross-tenant-data-leak-containment.md
 M cloud/tenancy/runbooks/dr-pair-promotion-drill.md
 M cloud/tenancy/runbooks/hot-split.md
 M cloud/tenancy/runbooks/jwt-key-rotation.md
 M cloud/tenancy/runbooks/kyb-kyc-pipeline-stalled.md
 M cloud/tenancy/runbooks/parent-child-permit-revocation.md
 M cloud/tenancy/runbooks/rls-drift-recovery.md
 M cloud/tenancy/runbooks/tenant-deletion-dsr-cascade.md
 M cloud/tenancy/runbooks/tenant-isolation-breach-response.md
 M cloud/tenancy/runbooks/tenant-onboarding.md
 M cloud/tenancy/runbooks/tenant-suspension.md
 M cloud/tenancy/threat-models/threat-model.md
 M docs/ADR-INDEX.md
 M docs/AGENTS-OPERATING-CONTRACT.md
 M docs/AGENTS.md
 M docs/CHANGELOG.md
 M docs/MASTERPLAN.md
 M docs/PRIVACY-PROGRAM.md
 M docs/README.md
 M docs/RELEASE-MANAGEMENT.md
 M docs/RUNBOOKS-INDEX.md
 M docs/STANDARDS-AND-TEMPLATES.md
 M docs/checklists/done-definition-checklist.md
 M docs/checklists/per-implementation-plan-checklist.md
 M docs/checklists/pr-review-checklist.md
 M docs/checklists/pre-merge.md
 M docs/checklists/pre-push.md
 M docs/checklists/release-readiness-checklist.md
 M docs/ci/forge-of-record.md
 M docs/decisions/ADR-0032-dcim-software-for-own-dc-ops.md
 M docs/decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md
 M docs/decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md
 M docs/decisions/ADR-0157-api-gateway-tier.md
 M docs/decisions/ADR-0158-multi-region-active-active.md
 M docs/decisions/ADR-0163-tenant-environment-tiers.md
 M docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md
 M docs/decisions/ADR-0187-canonical-oidc-idp-zitadel-primary.md
 M docs/decisions/ADR-0211-in-house-tech-stack-policy.md
 M docs/decisions/ADR-0334-shorts-microservice-merged-into-social.md
 M docs/decisions/ADR-0394-bespoke-rust-idp-central-hub.md
 M docs/oya-ci/config-reference.md
 M docs/oya-ci/gate-catalog.md
 M docs/products/cloud/PRD.md
 M docs/products/foundry/PRD.md
 M docs/standards/INDEX.md
 M docs/standards/agent-instructions-discipline.md
 M docs/standards/claude-code-harness.md
 M docs/standards/clean-architecture.md
 M docs/standards/code-style-rust.md
 M docs/standards/crate-naming-convention.md
 M docs/standards/data-class.md
 M docs/standards/dependency-policy.md
 M docs/standards/doc-style.md
 M docs/standards/error-handling.md
 M docs/standards/git-workflow.md
 M docs/standards/image-discipline.md
 M docs/standards/observability.md
 M docs/standards/on-call.md
 M docs/standards/release-management.md
 M docs/standards/release.md
 M docs/standards/security-review.md
 M docs/standards/testing.md
 M docs/templates/INDEX.md
 M docs/templates/adr-template.md
 M docs/templates/implementation-plan-template.md
 M docs/templates/pull-request-template-v2.md
 M docs/templates/pull-request-template.md
 M docs/templates/team-charter-template.md
 M infra/branch-protection/dev.json
 M infra/capi/clusters/README.md
 M infra/capi/clusters/templates/clusters.yaml
 M infra/capi/clusters/values-example.yaml
 M infra/capi/crs/clusterresourceset.yaml
 M infra/capi/crs/render.sh
 M infra/cloudflare/main.tf
 M infra/talos/installation-media/README.md
 M infra/talos/installation-media/gen-media.sh
 M libs/oya-bus-boundary-kernel/src/lib.rs
 M libs/oya-check-high-risk-auto-decision-refusal/src/lib.rs
 M libs/oya-check-honest-claims/BUCK
 M libs/oya-check-honest-claims/Cargo.toml
 M libs/oya-check-honest-claims/src/lib.rs
 M libs/oya-check-layered-architecture-discipline/src/lib.rs
 M libs/oya-check-otel-trace-propagation/src/lib.rs
 M libs/oya-check-pr-traceability/src/lib.rs
 M libs/oya-check-pre-push/src/lib.rs
 M libs/oya-check-supply-chain/src/lib.rs
 M libs/oya-ci-config/src/bundled/gate-disposition.json
 M libs/oya-ci-config/src/lib.rs
 M libs/oya-ci-gate-contract/src/lib.rs
 M libs/oya-data-boundary-kernel/src/retention_policy.rs
 M libs/oya-data-sql-adapter-sqlx/src/lib.rs
 M libs/oya-gen-microservice-manifests-app/src/lib.rs
 M libs/oya-gen-microservice-manifests-app/src/main.rs
 M libs/oya-gen-microservice-manifests-app/tests/check_mode.rs
 M libs/oya-governance-adapter-with-no-importer-kernel/src/lib.rs
 M libs/oya-governance-gate-catalog-domain/src/lib.rs
 M libs/oya-governance-mistakes-ledger-kernel/src/lib.rs
 M libs/oya-http-latency-budget-middleware-infrastructure/BUCK
 M libs/oya-http-latency-budget-middleware-infrastructure/Cargo.toml
 M libs/oya-http-latency-budget-middleware-infrastructure/src/lib.rs
 M libs/oya-http-router-kernel/src/lib.rs
 M libs/oya-http-telemetry-middleware-infrastructure/src/lib.rs
 M libs/oya-http-wide-event-middleware-infrastructure/BUCK
 M libs/oya-http-wide-event-middleware-infrastructure/Cargo.toml
 M libs/oya-http-wide-event-middleware-infrastructure/src/lib.rs
 M libs/oya-messaging-substrate-kernel/src/conformance.rs
 M libs/oya-messaging-substrate-kernel/src/lib.rs
 M libs/oya-messaging-substrate-kernel/src/reference.rs
 M libs/oya-queue-boundary-kernel/src/lib.rs
 M libs/oya-shared-pdp-adapter-cedar/BUCK
 M libs/oya-shared-pdp-adapter-cedar/Cargo.toml
 M libs/oya-shared-pdp-adapter-cedar/src/lib.rs
 M libs/oya-shared-pdp-adapter-cedar/tests/cedar_pdp_conformance.rs
 M libs/oya-shared-pdp-kernel/src/lib.rs
 M libs/oya-stream-boundary-kernel/src/lib.rs
 M oya-ci.toml
 M oya/accounting/contracts/openapi-v1.meta.yaml
 M oya/accounting/crates/oya-accounting-journal-app/src/lib.rs
 M oya/accounting/crates/oya-accounting-journal-domain/src/lib.rs
 M oya/accounting/crates/oya-accounting-journal-storage-adapter-inmemory/tests/storage.rs
 M oya/analytics/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/analytics/README.md
 M oya/analytics/dpia/dpia.md
 M oya/analytics/runbooks/auto-rebalance.md
 M oya/analytics/runbooks/cold-merge.md
 M oya/analytics/runbooks/hot-split.md
 M oya/api-gateway/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/api-gateway/README.md
 M oya/api-gateway/contracts/api-gateway.openapi.yaml
 M oya/api-gateway/dpia/dpia.md
 M oya/api-gateway/iac/k8s/helm/values.yaml
 M oya/api-gateway/runbooks/auto-rebalance.md
 M oya/api-gateway/runbooks/cold-merge.md
 M oya/api-gateway/runbooks/hot-split.md
 M oya/application/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/application/README.md
 M oya/application/crates/oya-application-shell-frontend/src/app.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/audit_evidence_timeline.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/ops_deployment_status_panel.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/policy_disclosure_banner.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/tenant_context_switcher.rs
 M oya/application/crates/oya-application-shell-frontend/src/lib.rs
 M oya/application/crates/oya-application-shell-frontend/src/render_envelope.rs
 M oya/application/crates/oya-application-shell-frontend/src/shell_capability_registry.rs
 M oya/application/crates/oya-application-shell-frontend/src/token_broker.rs
 M oya/application/crates/oya-application-shell-frontend/style/app.css
 M oya/application/dpia/dpia.md
 M oya/application/manifest.json
 M oya/application/runbooks/auto-rebalance.md
 M oya/application/runbooks/cold-merge.md
 M oya/application/runbooks/hot-split.md
 M oya/audit-chain/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/audit-chain/README.md
 M oya/audit-chain/dpia/dpia.md
 M oya/audit-chain/manifest.json
 M oya/audit-chain/runbooks/audit-chain-restart.md
 M oya/audit-chain/runbooks/audit-export.md
 M oya/audit-chain/runbooks/auto-rebalance.md
 M oya/audit-chain/runbooks/chain-replay-from-snapshot-protocol.md
 M oya/audit-chain/runbooks/cold-merge.md
 M oya/audit-chain/runbooks/hot-split.md
 M oya/audit-chain/runbooks/hsm-key-rotation.md
 M oya/audit-chain/runbooks/merkle-root-discrepancy-investigation.md
 M oya/audit-chain/runbooks/merkle-seal-recovery.md
 M oya/audit-chain/runbooks/regulator-evidence-export-failure.md
 M oya/audit-chain/runbooks/retention-cascade.md
 M oya/audit-chain/runbooks/signature-verification-failure.md
 M oya/calendar/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/calendar/README.md
 M oya/calendar/dpia/dpia.md
 M oya/calendar/manifest.json
 M oya/calendar/runbooks/auto-rebalance.md
 M oya/calendar/runbooks/cold-merge.md
 M oya/calendar/runbooks/hot-split.md
 M oya/ci-webhook-gateway/src/dispatch.rs
 M oya/ci-webhook-gateway/src/error.rs
 M oya/ci-webhook-gateway/src/lib.rs
 M oya/comms-email/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/comms-email/README.md
 M oya/comms-email/dpia/dpia.md
 M oya/comms-email/runbooks/auto-rebalance.md
 M oya/comms-email/runbooks/cold-merge.md
 M oya/comms-email/runbooks/hot-split.md
 M oya/community/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/community/README.md
 M oya/community/dpia/dpia.md
 M oya/community/manifest.json
 M oya/community/runbooks/auto-rebalance.md
 M oya/community/runbooks/cold-merge.md
 M oya/community/runbooks/coordinated-spam-attack-response.md
 M oya/community/runbooks/hot-split.md
 M oya/community/runbooks/kb-attachment-restore.md
 M oya/community/runbooks/moderation-queue-clear.md
 M oya/community/runbooks/moderator-decision-appeal-protocol.md
 M oya/community/runbooks/post-mass-deletion.md
 M oya/community/runbooks/search-rebuild.md
 M oya/community/runbooks/spam-flood-throttle.md
 M oya/community/runbooks/verified-anonymous-deanonymization-incident.md
 M oya/community/runbooks/vote-anomaly.md
 M oya/compliance/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/compliance/README.md
 M oya/compliance/dpia/dpia.md
 M oya/compliance/runbooks/audit-seal-verify-failure.md
 M oya/compliance/runbooks/auto-rebalance.md
 M oya/compliance/runbooks/breach-notification-72h-clock-at-risk.md
 M oya/compliance/runbooks/certification-evidence-pipeline-stall.md
 M oya/compliance/runbooks/cold-merge.md
 M oya/compliance/runbooks/cross-tenant-dsar-leak-suspected.md
 M oya/compliance/runbooks/dsar-backlog-overflow.md
 M oya/compliance/runbooks/engagement-cedar-revoke-failed.md
 M oya/compliance/runbooks/evidence-collector-degraded.md
 M oya/compliance/runbooks/hot-split.md
 M oya/compliance/runbooks/manual-evidence-upload-rejected.md
 M oya/compliance/runbooks/pack-overlay-conflict-resolution.md
 M oya/compliance/runbooks/phi-access-anomaly.md
 M oya/compliance/runbooks/regulator-engagement-grant-revoke.md
 M oya/compliance/runbooks/regulator-evidence-export-failure.md
 M oya/compliance/runbooks/seaweedfs-evidence-bucket-loss.md
 M oya/connector/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/connector/README.md
 M oya/connector/crates/oya-connector-slack-adapter/src/lib.rs
 M oya/connector/dpia/dpia.md
 M oya/connector/runbooks/auto-rebalance.md
 M oya/connector/runbooks/cold-merge.md
 M oya/connector/runbooks/hot-split.md
 M oya/consent-graph/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/consent-graph/README.md
 M oya/consent-graph/dpia/dpia.md
 M oya/consent-graph/runbooks/auto-rebalance.md
 M oya/consent-graph/runbooks/cold-merge.md
 M oya/consent-graph/runbooks/hot-split.md
 M oya/contact-center/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/contact-center/README.md
 M oya/contact-center/dpia/dpia.md
 M oya/contact-center/runbooks/auto-rebalance.md
 M oya/contact-center/runbooks/cold-merge.md
 M oya/contact-center/runbooks/hot-split.md
 M oya/contract-lifecycle-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/contract-lifecycle-management/README.md
 M oya/contract-lifecycle-management/dpia/dpia.md
 M oya/contract-lifecycle-management/runbooks/auto-rebalance.md
 M oya/contract-lifecycle-management/runbooks/cold-merge.md
 M oya/contract-lifecycle-management/runbooks/hot-split.md
 M oya/crm/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/crm/README.md
 M oya/crm/crates/oya-crm-customer-engagement-domain/tests/customer_engagement.rs
 M oya/crm/dpia/dpia.md
 M oya/crm/manifest.json
 M oya/crm/runbooks/auto-rebalance.md
 M oya/crm/runbooks/cold-merge.md
 M oya/crm/runbooks/hot-split.md
 M oya/data-pipeline/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/data-pipeline/README.md
 M oya/data-pipeline/dpia/dpia.md
 M oya/data-pipeline/runbooks/auto-rebalance.md
 M oya/data-pipeline/runbooks/cold-merge.md
 M oya/data-pipeline/runbooks/hot-split.md
 M oya/data-warehouse/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/data-warehouse/README.md
 M oya/data-warehouse/dpia/dpia.md
 M oya/data-warehouse/runbooks/auto-rebalance.md
 M oya/data-warehouse/runbooks/cold-merge.md
 M oya/data-warehouse/runbooks/hot-split.md
 M oya/design-collaboration/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/design-collaboration/README.md
 M oya/design-collaboration/dpia/dpia.md
 M oya/design-collaboration/runbooks/auto-rebalance.md
 M oya/design-collaboration/runbooks/cold-merge.md
 M oya/design-collaboration/runbooks/hot-split.md
 M oya/detection/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/detection/PRD.md
 M oya/detection/README.md
 M oya/detection/dpia/dpia.md
 M oya/detection/runbooks/auto-rebalance.md
 M oya/detection/runbooks/cold-merge.md
 M oya/detection/runbooks/hot-split.md
 M oya/developer-sdk/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/developer-sdk/README.md
 M oya/developer-sdk/crates/oya-dev-cli/src/bin/fake-verify-command.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/cloud_iac_cell_topology_gate.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/commands/gate/mod.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/commands/verify.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/hyperscaler_maturity_claims_gate.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/lib.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/supply_chain_gates.rs
 M oya/developer-sdk/crates/oya-dev-cli/tests/gate_cli.rs
 M oya/developer-sdk/crates/oya-dev-cli/tests/oya_verify_ci_mirror.rs
 M oya/developer-sdk/crates/oya-dev-cli/tests/pr_traceability_cli.rs
 M oya/developer-sdk/dpia/dpia.md
 M oya/developer-sdk/runbooks/auto-rebalance.md
 M oya/developer-sdk/runbooks/cold-merge.md
 M oya/developer-sdk/runbooks/hot-split.md
 M oya/diagnostics/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/diagnostics/README.md
 M oya/diagnostics/dpia/dpia.md
 M oya/diagnostics/runbooks/auto-rebalance.md
 M oya/diagnostics/runbooks/cold-merge.md
 M oya/diagnostics/runbooks/hot-split.md
 M oya/docs/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/docs/README.md
 M oya/docs/dpia/dpia.md
 M oya/docs/manifest.json
 M oya/docs/runbooks/auto-rebalance.md
 M oya/docs/runbooks/cold-merge.md
 M oya/docs/runbooks/hot-split.md
 M oya/drive/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/drive/README.md
 M oya/drive/crates/oya-drive-domain/src/lib.rs
 M oya/drive/dpia/dpia.md
 M oya/drive/manifest.json
 M oya/drive/runbooks/auto-rebalance.md
 M oya/drive/runbooks/cold-merge.md
 M oya/drive/runbooks/hot-split.md
 M oya/emergency/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/emergency/README.md
 M oya/emergency/dpia/dpia.md
 M oya/emergency/runbooks/auto-rebalance.md
 M oya/emergency/runbooks/cold-merge.md
 M oya/emergency/runbooks/hot-split.md
 M oya/emr/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/emr/README.md
 M oya/emr/dpia/dpia.md
 M oya/emr/runbooks/auto-rebalance.md
 M oya/emr/runbooks/cold-merge.md
 M oya/emr/runbooks/hot-split.md
 M oya/feature-flags/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/feature-flags/README.md
 M oya/feature-flags/dpia/dpia.md
 M oya/feature-flags/runbooks/auto-rebalance.md
 M oya/feature-flags/runbooks/cold-merge.md
 M oya/feature-flags/runbooks/hot-split.md
 M oya/financial-planning/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/financial-planning/README.md
 M oya/financial-planning/dpia/dpia.md
 M oya/financial-planning/runbooks/auto-rebalance.md
 M oya/financial-planning/runbooks/cold-merge.md
 M oya/financial-planning/runbooks/hot-split.md
 M oya/finops-portal/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/finops-portal/README.md
 M oya/finops-portal/dpia/dpia.md
 M oya/finops-portal/runbooks/auto-rebalance.md
 M oya/finops-portal/runbooks/budget-alert-runaway-firings.md
 M oya/finops-portal/runbooks/cold-merge.md
 M oya/finops-portal/runbooks/cost-allocation-policy-rollback.md
 M oya/finops-portal/runbooks/cost-attribution-mismatch-investigation.md
 M oya/finops-portal/runbooks/credit-application-reconciliation.md
 M oya/finops-portal/runbooks/focus-export-failure.md
 M oya/finops-portal/runbooks/hot-split.md
 M oya/finops-portal/runbooks/quarterly-regulator-emit-miss.md
 M oya/finops-portal/runbooks/reservation-recommendation-engine-stall.md
 M oya/finops-portal/runbooks/tenant-bill-mismatch-resolution.md
 M oya/finops-portal/runbooks/tenant-budget-exhausted.md
 M oya/finops-portal/runbooks/tenant-budget-headroom-low.md
 M oya/finops-portal/runbooks/tenant-cost-anomaly-spike.md
 M oya/forms/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/forms/README.md
 M oya/forms/dpia/dpia.md
 M oya/forms/manifest.json
 M oya/forms/runbooks/auto-rebalance.md
 M oya/forms/runbooks/cold-merge.md
 M oya/forms/runbooks/hot-split.md
 M oya/global-trade/AUDIT-FINDINGS-2026-05-21.json
 M oya/global-trade/IPs/IP-ADR-0339-Shared-IaC-Modules.md
 M oya/global-trade/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/global-trade/README.md
 M oya/global-trade/capabilities/customs-declaration-command.yaml
 M oya/global-trade/capabilities/export-control-classification-export.yaml
 M oya/global-trade/capabilities/sanctions-screening-reconcile.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-api.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-application.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-api.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-application.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-api.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-application.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-api.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-application.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-api.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-application.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-api.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-application.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-worker.yaml
 M oya/global-trade/cedar/policies.cedar
 M oya/global-trade/contracts/asyncapi-v1.yaml
 M oya/global-trade/contracts/global-trade-v1.proto
 M oya/global-trade/contracts/openapi-v1.yaml
 M oya/global-trade/dashboards/customs-declaration-health.json
 M oya/global-trade/dashboards/global-trade-overview.json
 M oya/global-trade/decisions/ADR-GT-001-sanctions-export-control-and-broker-filing-hold-state-machine.md
 M oya/global-trade/dpia/dpia.md
 M oya/global-trade/iac/ech-config.yaml
 M oya/global-trade/iac/edge-waf.yaml
 M oya/global-trade/iac/helm-values.yaml
 M oya/global-trade/iac/k8s-deployment.yaml
 M oya/global-trade/iac/k8s/helm/Chart.yaml
 M oya/global-trade/iac/k8s/helm/templates/cedar.yaml
 M oya/global-trade/iac/k8s/helm/templates/configmap.yaml
 M oya/global-trade/iac/k8s/helm/templates/deployment.yaml
 M oya/global-trade/iac/k8s/helm/templates/service.yaml
 M oya/global-trade/iac/k8s/helm/values.yaml
 M oya/global-trade/iac/network-policy.yaml
 M oya/global-trade/iac/openbao-policy.hcl
 M oya/global-trade/iac/pqc-cert.yaml
 M oya/global-trade/iac/secret-bindings.yaml
 M oya/global-trade/iac/terraform-module/main.tf
 M oya/global-trade/manifest.json
 M oya/global-trade/policy/abuse-defence.cedar
 M oya/global-trade/policy/auditor-scope.cedar
 M oya/global-trade/policy/broker-filing-authorization.cedar
 M oya/global-trade/policy/ci-scope.cedar
 M oya/global-trade/policy/customs-declaration-authorization.cedar
 M oya/global-trade/policy/denied-party-hit-authorization.cedar
 M oya/global-trade/policy/emergency-services-bypass.cedar
 M oya/global-trade/policy/export-control-classification-authorization.cedar
 M oya/global-trade/policy/pack-overlay-authorization.cedar
 M oya/global-trade/policy/sanctions-screening-authorization.cedar
 M oya/global-trade/policy/trade-document-authorization.cedar
 M oya/global-trade/runbooks/approval-deadletter.md
 M oya/global-trade/runbooks/auto-rebalance.md
 M oya/global-trade/runbooks/capacity-saturation.md
 M oya/global-trade/runbooks/cold-merge.md
 M oya/global-trade/runbooks/hot-split.md
 M oya/global-trade/runbooks/marketplace-settlement-blocked.md
 M oya/global-trade/runbooks/policy-deny-spike.md
 M oya/global-trade/runbooks/regional-failover.md
 M oya/global-trade/runbooks/source-import-stalled.md
 M oya/global-trade/scorecards/overrides.json
 M oya/global-trade/slos/autosharding-events.openslo.yaml
 M oya/global-trade/slos/customs-declaration-success-rate.openslo.yaml
 M oya/global-trade/slos/global-trade-availability.openslo.yaml
 M oya/global-trade/slos/global-trade-latency-p99.openslo.yaml
 M oya/global-trade/slos/global-trade-throughput.openslo.yaml
 M oya/governance/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/governance/README.md
 M oya/governance/dpia/dpia.md
 M oya/governance/manifest.json
 M oya/governance/runbooks/aggregation-rebuild.md
 M oya/governance/runbooks/audit-event-emission-stall.md
 M oya/governance/runbooks/auto-rebalance.md
 M oya/governance/runbooks/cedar-policy-rollback-protocol.md
 M oya/governance/runbooks/cold-merge.md
 M oya/governance/runbooks/consent-collection-pipeline-failure.md
 M oya/governance/runbooks/envoy-wasm-filter-rollback.md
 M oya/governance/runbooks/evidence-replay.md
 M oya/governance/runbooks/hot-split.md
 M oya/governance/runbooks/industry-baseline-refresh.md
 M oya/governance/runbooks/lane-bypass-emergency.md
 M oya/governance/runbooks/lane-failure-triage.md
 M oya/governance/runbooks/migration-execution.md
 M oya/governance/runbooks/wasm-filter-bytecode-quarantine.md
 M oya/healthcare-integration/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/healthcare-integration/README.md
 M oya/healthcare-integration/dpia/dpia.md
 M oya/healthcare-integration/runbooks/auto-rebalance.md
 M oya/healthcare-integration/runbooks/cold-merge.md
 M oya/healthcare-integration/runbooks/hot-split.md
 M oya/hr/contracts/openapi-v1.meta.yaml
 M oya/hr/contracts/openapi-v1.yaml
 M oya/hr/crates/oya-hr-employment-api/src/lib.rs
 M oya/hr/crates/oya-hr-employment-api/tests/contracts.rs
 M oya/hr/crates/oya-hr-employment-app/BUCK
 M oya/hr/crates/oya-hr-employment-app/Cargo.toml
 M oya/hr/crates/oya-hr-employment-app/src/lib.rs
 M oya/hr/crates/oya-hr-employment-app/tests/app_envelopes.rs
 M oya/hr/crates/oya-hr-employment-app/tests/leave.rs
 M oya/hr/crates/oya-hr-employment-app/tests/privacy.rs
 M oya/hr/crates/oya-hr-employment-domain/BUCK
 M oya/hr/crates/oya-hr-employment-domain/src/lib.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/leave_balance.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/leave_carryover_forfeiture.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/onboarding.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/rulepack_manifest.rs
 M oya/hr/crates/oya-hr-employment-infrastructure/src/lib.rs
 M oya/hr/crates/oya-hr-employment-infrastructure/tests/runtime.rs
 M oya/identity/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/identity/PRD.md
 M oya/identity/README.md
 M oya/identity/crates/oya-identity-workload-oidc-adapter/src/eddsa.rs
 M oya/identity/crates/oya-identity-workload-oidc-adapter/src/lib.rs
 M oya/identity/crates/oya-identity-workload-rest/Cargo.toml
 M oya/identity/crates/oya-identity-workload-rest/tests/common/mod.rs
 M oya/identity/crates/oya-identity-workload-rest/tests/rest_endpoints.rs
 M oya/identity/dpia/dpia.md
 M oya/identity/manifest.json
 M oya/identity/runbooks/auto-rebalance.md
 M oya/identity/runbooks/brute-force-mitigation.md
 M oya/identity/runbooks/cold-merge.md
 M oya/identity/runbooks/hot-split.md
 M oya/identity/runbooks/idp-failover-drill.md
 M oya/identity/runbooks/ip-block-incident.md
 M oya/identity/runbooks/jwks-rotation.md
 M oya/identity/runbooks/passkey-cross-device-debug.md
 M oya/identity/runbooks/passkey-replay-attack-response.md
 M oya/identity/runbooks/passkey-reset.md
 M oya/identity/runbooks/recovery-key-mass-issue-investigation.md
 M oya/identity/runbooks/scim-provisioning-debug.md
 M oya/identity/runbooks/tenant-admin-onboard.md
 M oya/identity/runbooks/webauthn-rp-id-rotation.md
 M oya/imaging/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/imaging/README.md
 M oya/imaging/dpia/dpia.md
 M oya/imaging/runbooks/auto-rebalance.md
 M oya/imaging/runbooks/cold-merge.md
 M oya/imaging/runbooks/hot-split.md
 M oya/incident-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/incident-management/README.md
 M oya/incident-management/dpia/dpia.md
 M oya/incident-management/runbooks/auto-rebalance.md
 M oya/incident-management/runbooks/cold-merge.md
 M oya/incident-management/runbooks/hot-split.md
 M oya/intelligence/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/intelligence/README.md
 M oya/intelligence/_legacy-foundry/README.md
 M oya/intelligence/crates/oya-intelligence-dispatch-usecase/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-domain/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-kernel/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-usecase/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-usecase/tests/acceptance.rs
 M oya/intelligence/crates/oya-intelligence-subagent-runtime-app/src/main.rs
 M oya/intelligence/dpia/dpia.md
 M oya/intelligence/manifest.json
 M oya/intelligence/runbooks/assist-draft-policy-refusal.md
 M oya/intelligence/runbooks/audit-row-forgery-detected.md
 M oya/intelligence/runbooks/auto-rebalance.md
 M oya/intelligence/runbooks/byok-rotation-tenant-cascade.md
 M oya/intelligence/runbooks/cold-merge.md
 M oya/intelligence/runbooks/eu-ai-act-incident-notification.md
 M oya/intelligence/runbooks/hot-split.md
 M oya/intelligence/runbooks/model-inference-timeout-investigation.md
 M oya/intelligence/runbooks/model-router-stall-investigation.md
 M oya/intelligence/runbooks/prompt-fence-bypass-attempt-response.md
 M oya/intelligence/runbooks/prompt-fence-bypass-detection.md
 M oya/intelligence/runbooks/prompt-injection-detected.md
 M oya/intelligence/runbooks/provider-outage-anthropic.md
 M oya/intelligence/runbooks/provider-outage-google.md
 M oya/intelligence/runbooks/provider-outage-openai.md
 M oya/intelligence/runbooks/provider-rate-limit-saturation.md
 M oya/intelligence/runbooks/rag-corpus-drift-detection.md
 M oya/intelligence/runbooks/rag-retrieval-quality-regression.md
 M oya/intelligence/runbooks/refusal-false-positive-cascade.md
 M oya/intelligence/runbooks/sidecar-credential-handle-expired.md
 M oya/itsm/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/itsm/README.md
 M oya/itsm/dpia/dpia.md
 M oya/itsm/runbooks/auto-rebalance.md
 M oya/itsm/runbooks/cold-merge.md
 M oya/itsm/runbooks/hot-split.md
 M oya/learning-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/learning-management/README.md
 M oya/learning-management/dpia/dpia.md
 M oya/learning-management/runbooks/auto-rebalance.md
 M oya/learning-management/runbooks/cold-merge.md
 M oya/learning-management/runbooks/hot-split.md
 M oya/mail/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/mail/README.md
 M oya/mail/dpia/dpia.md
 M oya/mail/iac/helm/templates/networkpolicy.yaml
 M oya/mail/iac/helm/templates/service.yaml
 M oya/mail/iac/helm/values.yaml
 M oya/mail/manifest.json
 M oya/mail/runbooks/auto-rebalance.md
 M oya/mail/runbooks/cold-merge.md
 M oya/mail/runbooks/hot-split.md
 M oya/marketing-automation/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/marketing-automation/README.md
 M oya/marketing-automation/dpia/dpia.md
 M oya/marketing-automation/runbooks/auto-rebalance.md
 M oya/marketing-automation/runbooks/cold-merge.md
 M oya/marketing-automation/runbooks/hot-split.md
 M oya/marketplace/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/marketplace/README.md
 M oya/marketplace/dpia/dpia.md
 M oya/marketplace/runbooks/auto-rebalance.md
 M oya/marketplace/runbooks/buyer-order-double-submit.md
 M oya/marketplace/runbooks/cold-merge.md
 M oya/marketplace/runbooks/cross-border-tax-hold.md
 M oya/marketplace/runbooks/cross-tenant-buyer-seller-mediation-stall.md
 M oya/marketplace/runbooks/deal-acceptance-stalled.md
 M oya/marketplace/runbooks/deal-settlement-discrepancy-resolution.md
 M oya/marketplace/runbooks/dispute-escalation-protocol.md
 M oya/marketplace/runbooks/escrow-reservation-mismatch.md
 M oya/marketplace/runbooks/hot-split.md
 M oya/marketplace/runbooks/mediation-queue-saturation.md
 M oya/marketplace/runbooks/order-export-deadletter.md
 M oya/marketplace/runbooks/revenue-share-drift.md
 M oya/marketplace/runbooks/sanctions-screen-latency.md
 M oya/marketplace/runbooks/seller-onboarding-deny-spike.md
 M oya/marketplace/runbooks/settlement-ledger-replay.md
 M oya/meet/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/meet/README.md
 M oya/meet/dpia/dpia.md
 M oya/meet/manifest.json
 M oya/meet/runbooks/auto-rebalance.md
 M oya/meet/runbooks/cold-merge.md
 M oya/meet/runbooks/hot-split.md
 M oya/messenger/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/messenger/README.md
 M oya/messenger/crates/oya-messenger-message-stream-rest/src/lib.rs
 M oya/messenger/dpia/dpia.md
 M oya/messenger/manifest.json
 M oya/messenger/runbooks/auto-rebalance.md
 M oya/messenger/runbooks/cold-merge.md
 M oya/messenger/runbooks/hot-split.md
 M oya/notes/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/notes/README.md
 M oya/notes/dpia/dpia.md
 M oya/notes/manifest.json
 M oya/notes/runbooks/auto-rebalance.md
 M oya/notes/runbooks/cold-merge.md
 M oya/notes/runbooks/hot-split.md
 M oya/observability/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/observability/README.md
 M oya/observability/dpia/dpia.md
 M oya/observability/manifest.json
 M oya/observability/runbooks/auto-rebalance.md
 M oya/observability/runbooks/cold-merge.md
 M oya/observability/runbooks/hot-split.md
 M oya/ontology/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/ontology/PRD.md
 M oya/ontology/README.md
 M oya/ontology/dpia/dpia.md
 M oya/ontology/manifest.json
 M oya/ontology/runbooks/auto-rebalance.md
 M oya/ontology/runbooks/cedar-fragment-rollback.md
 M oya/ontology/runbooks/clickhouse-rebalance.md
 M oya/ontology/runbooks/cold-merge.md
 M oya/ontology/runbooks/cross-tenant-entity-collision-resolution.md
 M oya/ontology/runbooks/cross-tenant-leak-recovery.md
 M oya/ontology/runbooks/entity-projection-mismatch-recovery.md
 M oya/ontology/runbooks/graph-query-performance-regression.md
 M oya/ontology/runbooks/hot-split.md
 M oya/ontology/runbooks/object-type-deprecation.md
 M oya/ontology/runbooks/ontology-bot-score-recalibration.md
 M oya/ontology/runbooks/ontology-read-library-fallback.md
 M oya/ontology/runbooks/postgres-citus-rebalance.md
 M oya/ontology/runbooks/query-engine-restart.md
 M oya/ontology/runbooks/share-token-revocation.md
 M oya/ontology/runbooks/type-registry-migration.md
 M oya/ops-dashboard-control-center/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/ops-dashboard-control-center/README.md
 M oya/ops-dashboard-control-center/dpia/dpia.md
 M oya/ops-dashboard-control-center/iac/prod-spiffe-kill-switch.yaml
 M oya/ops-dashboard-control-center/runbooks/auto-rebalance.md
 M oya/ops-dashboard-control-center/runbooks/cold-merge.md
 M oya/ops-dashboard-control-center/runbooks/hot-split.md
 M oya/patient-monitoring/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/patient-monitoring/README.md
 M oya/patient-monitoring/dpia/dpia.md
 M oya/patient-monitoring/runbooks/auto-rebalance.md
 M oya/patient-monitoring/runbooks/cold-merge.md
 M oya/patient-monitoring/runbooks/hot-split.md
 M oya/payments/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/payments/PRD.md
 M oya/payments/README.md
 M oya/payments/crates/oya-payments-charge-domain/BUCK
 M oya/payments/crates/oya-payments-charge-domain/src/lib.rs
 M oya/payments/dpia/dpia.md
 M oya/payments/runbooks/aml-suspicious-activity-detected.md
 M oya/payments/runbooks/auto-rebalance.md
 M oya/payments/runbooks/chargeback-cascade-investigation.md
 M oya/payments/runbooks/cold-merge.md
 M oya/payments/runbooks/dispute-escalation.md
 M oya/payments/runbooks/double-charge-detected.md
 M oya/payments/runbooks/elder-financial-abuse.md
 M oya/payments/runbooks/fraud-spike-detected.md
 M oya/payments/runbooks/hot-split.md
 M oya/payments/runbooks/kr-fss-audit-pull.md
 M oya/payments/runbooks/kyc-aml-screening-pipeline-stall.md
 M oya/payments/runbooks/payout-failed.md
 M oya/payments/runbooks/pci-incident-response.md
 M oya/payments/runbooks/psp-failover-cascade-execution.md
 M oya/payments/runbooks/psp-outage.md
 M oya/payments/runbooks/refund-mismatch.md
 M oya/payroll/README.md
 M oya/payroll/catalog/oya-payroll-run-api.yaml
 M oya/payroll/catalog/oya-payroll-run-app.yaml
 M oya/payroll/catalog/oya-payroll-run-domain.yaml
 M oya/payroll/contracts/openapi-v1.meta.yaml
 M oya/payroll/contracts/openapi-v1.yaml
 M oya/payroll/crates/oya-payroll-run-api/BUCK
 M oya/payroll/crates/oya-payroll-run-api/Cargo.toml
 M oya/payroll/crates/oya-payroll-run-api/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-app/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-domain/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-domain/tests/rollback.rs
 M oya/payroll/crates/oya-payroll-run-infrastructure/BUCK
 M oya/payroll/crates/oya-payroll-run-infrastructure/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-infrastructure/tests/runtime.rs
 M oya/performance-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/performance-management/README.md
 M oya/performance-management/dpia/dpia.md
 M oya/performance-management/runbooks/auto-rebalance.md
 M oya/performance-management/runbooks/cold-merge.md
 M oya/performance-management/runbooks/hot-split.md
 M oya/pharmacy/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/pharmacy/README.md
 M oya/pharmacy/dpia/dpia.md
 M oya/pharmacy/runbooks/auto-rebalance.md
 M oya/pharmacy/runbooks/cold-merge.md
 M oya/pharmacy/runbooks/hot-split.md
 M oya/plant-maintenance/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/plant-maintenance/README.md
 M oya/plant-maintenance/contracts/asyncapi-v1.yaml
 M oya/plant-maintenance/contracts/openapi-v1.yaml
 M oya/plant-maintenance/contracts/plant-maintenance-v1.proto
 M oya/plant-maintenance/crates/oya-plant-maintenance-domain/tests/plant_maintenance.rs
 M oya/plant-maintenance/crates/oya-plant-maintenance-work-order-app/tests/integration.rs
 M oya/plant-maintenance/dpia/dpia.md
 M oya/plant-maintenance/manifest.json
 M oya/plant-maintenance/runbooks/auto-rebalance.md
 M oya/plant-maintenance/runbooks/cold-merge.md
 M oya/plant-maintenance/runbooks/hot-split.md
 M oya/plugin-app-store/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/plugin-app-store/README.md
 M oya/plugin-app-store/dpia/dpia.md
 M oya/plugin-app-store/runbooks/auto-rebalance.md
 M oya/plugin-app-store/runbooks/cold-merge.md
 M oya/plugin-app-store/runbooks/hot-split.md
 M oya/policy/crates/oya-policy-cedar-domain/BUCK
 M oya/policy/crates/oya-policy-cedar-domain/src/lib.rs
 M oya/policy/crates/oya-policy-cedar-domain/src/policy_diff.rs
 M oya/production-planning/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/production-planning/README.md
 M oya/production-planning/crates/oya-production-planning-domain/tests/production_planning.rs
 M oya/production-planning/dpia/dpia.md
 M oya/production-planning/manifest.json
 M oya/production-planning/runbooks/auto-rebalance.md
 M oya/production-planning/runbooks/cold-merge.md
 M oya/production-planning/runbooks/hot-split.md
 M oya/quality-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/quality-management/README.md
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/asyncapi.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/grpc.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/http.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/mod.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/domain/mod.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/tests/integration.rs
 M oya/quality-management/dpia/dpia.md
 M oya/quality-management/manifest.json
 M oya/quality-management/runbooks/auto-rebalance.md
 M oya/quality-management/runbooks/cold-merge.md
 M oya/quality-management/runbooks/hot-split.md
 M oya/real-estate/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/real-estate/README.md
 M oya/real-estate/crates/oya-real-estate-portfolio-domain/src/lib.rs
 M oya/real-estate/crates/oya-real-estate-portfolio-domain/tests/real_estate_portfolio.rs
 M oya/real-estate/dpia/dpia.md
 M oya/real-estate/manifest.json
 M oya/real-estate/runbooks/auto-rebalance.md
 M oya/real-estate/runbooks/cold-merge.md
 M oya/real-estate/runbooks/hot-split.md
 M oya/recordings/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/recordings/README.md
 M oya/recordings/crates/oya-recordings-domain/src/lib.rs
 M oya/recordings/dpia/dpia.md
 M oya/recordings/manifest.json
 M oya/recordings/runbooks/auto-rebalance.md
 M oya/recordings/runbooks/cold-merge.md
 M oya/recordings/runbooks/hot-split.md
 M oya/sheets/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/sheets/README.md
 M oya/sheets/dpia/dpia.md
 M oya/sheets/manifest.json
 M oya/sheets/runbooks/auto-rebalance.md
 M oya/sheets/runbooks/cold-merge.md
 M oya/sheets/runbooks/hot-split.md
 M oya/sites/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/sites/README.md
 M oya/sites/dpia/dpia.md
 M oya/sites/manifest.json
 M oya/sites/runbooks/auto-rebalance.md
 M oya/sites/runbooks/cold-merge.md
 M oya/sites/runbooks/hot-split.md
 M oya/slides/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/slides/README.md
 M oya/slides/dpia/dpia.md
 M oya/slides/manifest.json
 M oya/slides/runbooks/auto-rebalance.md
 M oya/slides/runbooks/cold-merge.md
 M oya/slides/runbooks/hot-split.md
 M oya/social/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/social/README.md
 M oya/social/dpia/dpia.md
 M oya/social/manifest.json
 M oya/social/runbooks/auto-rebalance.md
 M oya/social/runbooks/cold-merge.md
 M oya/social/runbooks/dr-failover.md
 M oya/social/runbooks/hot-split.md
 M oya/supply-chain-planning/IPs/IP-ADR-0339-Shared-IaC-Modules.md
 M oya/supply-chain-planning/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/supply-chain-planning/README.md
 M oya/supply-chain-planning/capabilities/available-to-promise-export.yaml
 M oya/supply-chain-planning/capabilities/demand-plan-command.yaml
 M oya/supply-chain-planning/capabilities/supply-network-plan-reconcile.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-worker.yaml
 M oya/supply-chain-planning/cedar/policies.cedar
 M oya/supply-chain-planning/contracts/asyncapi-v1.yaml
 M oya/supply-chain-planning/contracts/openapi-v1.yaml
 M oya/supply-chain-planning/contracts/supply-chain-planning-v1.proto
 M oya/supply-chain-planning/crates/oya-supply-chain-planning-domain/tests/supply_chain_planning.rs
 M oya/supply-chain-planning/crates/oya-supply-chain-planning-network-app/tests/integration.rs
 M oya/supply-chain-planning/dashboards/demand-plan-health.json
 M oya/supply-chain-planning/dashboards/supply-chain-planning-overview.json
 M oya/supply-chain-planning/dpia/dpia.md
 M oya/supply-chain-planning/iac/ech-config.yaml
 M oya/supply-chain-planning/iac/edge-waf.yaml
 M oya/supply-chain-planning/iac/helm-values.yaml
 M oya/supply-chain-planning/iac/k8s-deployment.yaml
 M oya/supply-chain-planning/iac/k8s/helm/Chart.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/cedar.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/configmap.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/deployment.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/service.yaml
 M oya/supply-chain-planning/iac/k8s/helm/values.yaml
 M oya/supply-chain-planning/iac/network-policy.yaml
 M oya/supply-chain-planning/iac/openbao-policy.hcl
 M oya/supply-chain-planning/iac/pqc-cert.yaml
 M oya/supply-chain-planning/iac/secret-bindings.yaml
 M oya/supply-chain-planning/iac/terraform-module/main.tf
 M oya/supply-chain-planning/manifest.json
 M oya/supply-chain-planning/runbooks/approval-deadletter.md
 M oya/supply-chain-planning/runbooks/auto-rebalance.md
 M oya/supply-chain-planning/runbooks/capacity-saturation.md
 M oya/supply-chain-planning/runbooks/cold-merge.md
 M oya/supply-chain-planning/runbooks/hot-split.md
 M oya/supply-chain-planning/runbooks/marketplace-settlement-blocked.md
 M oya/supply-chain-planning/runbooks/policy-deny-spike.md
 M oya/supply-chain-planning/runbooks/regional-failover.md
 M oya/supply-chain-planning/runbooks/source-import-stalled.md
 M oya/supply-chain-planning/slos/autosharding-events.openslo.yaml
 M oya/supply-chain-planning/slos/demand-plan-success-rate.openslo.yaml
 M oya/supply-chain-planning/slos/supply-chain-planning-availability.openslo.yaml
 M oya/supply-chain-planning/slos/supply-chain-planning-latency-p99.openslo.yaml
 M oya/supply-chain-planning/slos/supply-chain-planning-throughput.openslo.yaml
 M oya/tasks/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/tasks/README.md
 M oya/tasks/dpia/dpia.md
 M oya/tasks/manifest.json
 M oya/tasks/runbooks/auto-rebalance.md
 M oya/tasks/runbooks/cold-merge.md
 M oya/tasks/runbooks/hot-split.md
 M oya/tenant-rbac/contracts/openapi-v1.meta.yaml
 M oya/tenant-rbac/crates/oya-tenant-rbac-api/src/lib.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-api/tests/contracts.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/BUCK
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/Cargo.toml
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/src/lib.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/tests/runtime.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-domain/src/lib.rs
 M oya/translate/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/translate/README.md
 M oya/translate/dpia/dpia.md
 M oya/translate/manifest.json
 M oya/translate/runbooks/auto-rebalance.md
 M oya/translate/runbooks/cold-merge.md
 M oya/translate/runbooks/hot-split.md
 M oya/treasury/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/treasury/README.md
 M oya/treasury/cedar/policies.cedar
 M oya/treasury/contracts/asyncapi-v1.yaml
 M oya/treasury/contracts/openapi-v1.yaml
 M oya/treasury/contracts/treasury-v1.proto
 M oya/treasury/crates/oya-treasury-cash-domain/BUCK
 M oya/treasury/crates/oya-treasury-cash-domain/src/lib.rs
 M oya/treasury/crates/oya-treasury-cash-domain/tests/cash_position.rs
 M oya/treasury/dashboards/cash-position-health.json
 M oya/treasury/dashboards/treasury-overview.json
 M oya/treasury/dpia/dpia.md
 M oya/treasury/iac/ech-config.yaml
 M oya/treasury/iac/edge-waf.yaml
 M oya/treasury/iac/helm-values.yaml
 M oya/treasury/iac/k8s-deployment.yaml
 M oya/treasury/iac/k8s/helm/Chart.yaml
 M oya/treasury/iac/k8s/helm/templates/cedar.yaml
 M oya/treasury/iac/k8s/helm/templates/configmap.yaml
 M oya/treasury/iac/k8s/helm/templates/deployment.yaml
 M oya/treasury/iac/k8s/helm/templates/service.yaml
 M oya/treasury/iac/k8s/helm/values.yaml
 M oya/treasury/iac/network-policy.yaml
 M oya/treasury/iac/openbao-policy.hcl
 M oya/treasury/iac/pqc-cert.yaml
 M oya/treasury/iac/secret-bindings.yaml
 M oya/treasury/iac/terraform-module/main.tf
 M oya/treasury/manifest.json
 M oya/treasury/policy/auditor-scope.cedar
 M oya/treasury/runbooks/approval-deadletter.md
 M oya/treasury/runbooks/auto-rebalance.md
 M oya/treasury/runbooks/capacity-saturation.md
 M oya/treasury/runbooks/cold-merge.md
 M oya/treasury/runbooks/hot-split.md
 M oya/treasury/runbooks/marketplace-settlement-blocked.md
 M oya/treasury/runbooks/policy-deny-spike.md
 M oya/treasury/runbooks/regional-failover.md
 M oya/treasury/runbooks/source-import-stalled.md
 M oya/treasury/slos/autosharding-events.openslo.yaml
 M oya/treasury/slos/cash-position-success-rate.openslo.yaml
 M oya/treasury/slos/treasury-availability.openslo.yaml
 M oya/treasury/slos/treasury-latency-p99.openslo.yaml
 M oya/treasury/slos/treasury-throughput.openslo.yaml
 M oya/warehouse/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/warehouse/README.md
 M oya/warehouse/crates/oya-warehouse-inventory-domain/src/lib.rs
 M oya/warehouse/crates/oya-warehouse-inventory-domain/tests/inventory.rs
 M oya/warehouse/dpia/dpia.md
 M oya/warehouse/manifest.json
 M oya/warehouse/runbooks/auto-rebalance.md
 M oya/warehouse/runbooks/cold-merge.md
 M oya/warehouse/runbooks/hot-split.md
 M oya/whiteboard/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/whiteboard/README.md
 M oya/whiteboard/dpia/dpia.md
 M oya/whiteboard/runbooks/auto-rebalance.md
 M oya/whiteboard/runbooks/cold-merge.md
 M oya/whiteboard/runbooks/hot-split.md
 M oya/workflow-engine/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/workflow-engine/PRD.md
 M oya/workflow-engine/README.md
 M oya/workflow-engine/crates/oya-workflow-engine-execution-engine-sdk/src/lib.rs
 M oya/workflow-engine/crates/oya-workflow-engine-execution-engine-usecase/src/lib.rs
 M oya/workflow-engine/dpia/dpia.md
 M oya/workflow-engine/manifest.json
 M oya/workflow-engine/runbooks/auto-rebalance.md
 M oya/workflow-engine/runbooks/cold-merge.md
 M oya/workflow-engine/runbooks/deadlock-resolution.md
 M oya/workflow-engine/runbooks/durable-execution-history-replay.md
 M oya/workflow-engine/runbooks/durable-execution-restart.md
 M oya/workflow-engine/runbooks/event-bus-replay.md
 M oya/workflow-engine/runbooks/hot-split.md
 M oya/workflow-engine/runbooks/saga-compensation-failure-investigation.md
 M oya/workflow-engine/runbooks/spec-rollback.md
 M oya/workflow-engine/runbooks/stuck-workflow-recovery.md
 M oya/workflow-engine/runbooks/valkey-failover.md
 M oya/workflow-engine/runbooks/workflow-state-corruption-recovery.md
 M oya/workflow-studio/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/workflow-studio/README.md
 M oya/workflow-studio/dpia/dpia.md
 M oya/workflow-studio/manifest.json
 M oya/workflow-studio/runbooks/ai-assisted-generation-quality-regression.md
 M oya/workflow-studio/runbooks/auto-rebalance.md
 M oya/workflow-studio/runbooks/canvas-perf-regression-debug.md
 M oya/workflow-studio/runbooks/canvas-perf-regression.md
 M oya/workflow-studio/runbooks/cold-merge.md
 M oya/workflow-studio/runbooks/collab-conflict-resolution.md
 M oya/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md
 M oya/workflow-studio/runbooks/copilot-degraded-fallback.md
 M oya/workflow-studio/runbooks/crdt-merge-conflict.md
 M oya/workflow-studio/runbooks/hot-split.md
 M oya/workflow-studio/runbooks/node-graph-validation-failure.md
 M oya/workflow-studio/runbooks/presence-disconnect.md
 M oya/workflow-studio/runbooks/run-history-replay-corruption.md
 M oya/workflow-studio/runbooks/session-storm-throttle.md
 M oya/workflow-studio/runbooks/template-marketplace-quarantine.md
 M oya/workplace-integration/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M registry/artifact-capabilities-registry.json
 M registry/catalog/oya-payroll-run-api.yaml
 M registry/catalog/oya-payroll-run-app.yaml
 M registry/catalog/oya-payroll-run-domain.yaml
 M registry/catalog/oya-payroll-run-infrastructure.yaml
 M registry/catalog/oya-payroll-run-storage-adapter-inmemory.yaml
 M registry/dependency-rationales.json
 M registry/generated-artifact-control-plane.json
 M registry/placeholder-debt/adr-follow-ups.yaml
 M scripts/hooks/pre-push.sh
 M scripts/tests/cloud_observability_slo_evidence_check.py
 M specs/agent-durable-goal.json
 M specs/agent-operating-contract.json
 M specs/agentic-slo-gated-promotion.json
 M specs/audit-event-class-registry.json
 M specs/audit-event-schema.json
 M specs/bespoke-cloud-toolchain-services.json
 M specs/cedar-policy-schema.json
 M specs/chaos-engineering-substrate-canonical.json
 M specs/ci-farm-substrate-canonical.json
 M specs/ci-fix-loop-context-bundle.json
 M specs/cloud-hyperscaler-parity-taxonomy.json
 M specs/cloud-observability-slo-evidence-contract.json
 M specs/cloud-strangler-migration-target.json
 M specs/cloud-toolchain-target.json
 M specs/compliance-pack-floors.json
 M specs/compliance-pack-schema.json
 M specs/csi-storage-class-canonical.json
 M specs/deployment-ops-contract.json
 M specs/design-spec-maturity-claims.json
 M specs/design-system/audit-evidence-timeline.json
 M specs/design-system/catalog.json
 M specs/design-system/cloud-cell-topology-map.json
 M specs/design-system/communication-thread-list.json
 M specs/design-system/entity-action-policy-preview.json
 M specs/design-system/foundry-agent-run-timeline.json
 M specs/design-system/ontology-graph-explorer.json
 M specs/design-system/ops-deployment-status-panel.json
 M specs/design-system/policy-disclosure-banner.json
 M specs/design-system/score-card-result-table.json
 M specs/design-system/spec-diff-viewer.json
 M specs/design-system/tenant-context-switcher.json
 M specs/design-system/workflow-canvas.json
 M specs/design-system/workflow-node-config-panel.json
 M specs/design-system/workflow-replay-timeline.json
 M specs/feature-flag-substrate-canonical.json
 M specs/finops-dimensional-model.json
 M specs/gitops-vcs-replacement.json
 M specs/hyperscaler-architecture-invariants.json
 M specs/hyperscaler-gates.json
 M specs/markdown-retirement-policy.json
 M specs/master-plan-sequencing.json
 M specs/masterplan.json
 M specs/merge-queue-parked-pr.json
 M specs/microservice-migration-tooling.json
 M specs/microservices/accounting.json
 M specs/microservices/anonymous.json
 M specs/microservices/calendar.json
 M specs/microservices/crm.json
 M specs/microservices/global-trade.json
 M specs/microservices/hr.json
 M specs/microservices/intelligence.json
 M specs/microservices/mail.json
 M specs/microservices/manifest-schema.json
 M specs/microservices/manifests-index.json
 M specs/microservices/messenger.json
 M specs/microservices/ontology.json
 M specs/microservices/payroll.json
 M specs/microservices/plant-maintenance.json
 M specs/microservices/procurement.json
 M specs/microservices/production-planning.json
 M specs/microservices/quality-management.json
 M specs/microservices/real-estate.json
 M specs/microservices/social.json
 M specs/microservices/supply-chain-planning.json
 M specs/microservices/tenant-rbac.json
 M specs/microservices/treasury.json
 M specs/microservices/warehouse.json
 M specs/microservices/workflow-studio.json
 M specs/microservices/workflow.json
 M specs/multi-region-disposition-canonical.json
 M specs/ontology-projection-schema.json
 M specs/oyatie-doctrine.json
 M specs/pack-overlay-schema.json
 M specs/per-tenant-audit-log-slicing-canonical.json
 M specs/plan-schema.json
 M specs/planning-closure-contract.json
 M specs/planning-closure-status-closure-ledger.json
 M specs/platform-architecture.json
 M specs/repo-hygiene-automation.json
 M specs/root-hub-pointers.json
 M specs/schema-registry-canonical.json
 M specs/score-cards.json
 M specs/sovereign-cloud-air-gapped-canonical.json
 M specs/tenant-environment-tiers-canonical.json
 M specs/toolchain-tenant-isolation-fixtures.json
 M specs/workspace-hygiene.json
 M templates/INDEX.md
 M templates/checklists/done-definition-checklist.md
 M templates/checklists/per-implementation-plan-checklist.md
 M templates/checklists/pr-review-checklist.md
 M templates/checklists/release-readiness-checklist.md
 M templates/implementation-plan-template.md
 M templates/pull-request-template.md
?? .agents/
?? .claude/skills/
?? .github/CODE_OF_CONDUCT.md
?? .github/CONTRIBUTING.md
?? .github/ISSUE_TEMPLATE/
?? .github/OWNERS
?? .github/PULL_REQUEST_TEMPLATE.md
?? .hermes/
?? .ouroboros/
?? .ouroboros_eval_artifact.md
?? .worktrees/
?? cloud/cloud-billing/crates/oya-cloud-billing-domain/tests/env_tier_outbound_metadata.rs
?? cloud/cloud-capacity/manifest.json
?? cloud/cloud-ci/gates/oya-cloud-ci-license-policy-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-load-balancer-inventory-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-multi-region-disposition-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-sovereign-tenant-pin-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-tenant-environment-tier-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-zero-static-secrets-app/
?? cloud/cloud-iac/IPs/IP-sustainability-emission-model.md
?? cloud/cloud-iac/cell-topology/cell-001-contract-snapshot.json
?? cloud/cloud-intelligence/IPs/
?? cloud/cloud-intelligence/contracts/env-tier-gateway-budget-contract.json
?? cloud/cloud-network/crates/oya-cloud-network-domain/tests/cloud_network_resource_contract.rs
?? cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/persistence.rs
?? cloud/cloud-secrets/contracts/cloud-secrets-resource-contract.json
?? cloud/cloud-secrets/contracts/cloud-secrets-resource-contract.md
?? cloud/cloud-secrets/contracts/secretprovider-rotation-contract.md
?? cloud/cloud-secrets/crates/oya-secrets-domain/tests/cloud_secrets_resource_contract.rs
?? cloud/cloud-secrets/runbooks/non-prod-secretprovider-rotation-drill.md
?? cloud/managed-k8s-cluster-lifecycle/IPs/IP-sustainability-emission-model.md
?? cloud/managed-k8s-control-plane-host/IPs/IP-sustainability-emission-model.md
?? cloud/managed-k8s-sla-observability/IPs/IP-sustainability-emission-model.md
?? cloud/managed-k8s-sla-observability/runbooks/runbooks/sla-observation-store-unavailable.md
?? cloud/managed-k8s-tenant-quota/IPs/
?? cloud/managed-k8s-tenant-quota/crates/oya-managed-k8s-tenant-quota-adapter-postgres/
?? cloud/tenancy/IPs/IP-sustainability-emission-model.fixture.json
?? docs/audits/trust-center-security-privacy-docs-review-packet-2026-07-01.md
?? docs/ideas/ecosystem-as-code.md
?? docs/ideas/policy-pack-substrate.md
?? docs/runbooks/cloud/root-of-trust-ceremony.md
?? docs/standards/autonomous-kanban-lifecycle.md
?? evidence/audits/audit-002-retrieval-assumptions-contract-snapshot-2026-07-01.md
?? evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json
?? evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md
?? evidence/cell-topology/
?? evidence/cloud/
?? evidence/conformance/
?? evidence/contract-snapshots/
?? evidence/devportal/
?? evidence/multispectrum/arch-own-ratchet-001-20260701-1782886584.json
?? evidence/multispectrum/founder-call-own-ops-001-20260702-1782954777.json
?? evidence/multispectrum/regsec-001-vulnerability-intelligence-sbom-vex-20260701.json
?? evidence/multispectrum/regvuln-002-vulnerability-contract-integration-decision-20260701.json
?? evidence/multispectrum/t_3acb7585-qk05-review-fix-1782943388.json
?? evidence/multispectrum/t_5f5d9a01-shell-004-audit-evidence-timeline-1783612741.json
?? evidence/multispectrum/t_f60cb75d-finops-high-risk-emission-models-1782944098.json
?? evidence/multispectrum/t_ff6ecba7-manifest-index-inventory-recon-1782912252.json
?? evidence/multispectrum/w4-003-virtual-materialization-20260701.json
?? evidence/observability/
?? evidence/regulatory/
?? evidence/toolchain-isolation/
?? infra/capi/.gitignore
?? infra/capi/fleet-preflight.sh
?? libs/oya-ci-config/fixtures/
?? libs/oya-ci-gate-contract/BUCK
?? libs/oya-data-boundary-kernel/fixtures/
?? libs/oya-data-sql-adapter-sqlx/src/envelope.rs
?? oya/api-gateway/iac/k8s/helm/templates/mail-protocol-routes.yaml
?? oya/application/IPs/IP-sustainability-emission-model.md
?? oya/audit-chain/IPs/IP-sustainability-emission-model.fixture.json
?? oya/calendar/IPs/IP-sustainability-emission-model.md
?? oya/community/IPs/IP-sustainability-emission-model.md
?? oya/community/crates/oya-community-anonymous/
?? oya/connector/crates/oya-connector-slack-adapter/tests/
?? oya/developer-sdk/crates/oya-dev-cli/src/terminal_verifier_harness.rs
?? oya/docs/IPs/IP-sustainability-emission-model.md
?? oya/drive/IPs/IP-sustainability-emission-model.md
?? oya/finops-portal/contracts/env-tier-outbound-emission-plan.contract.json
?? oya/finops-portal/contracts/fixtures/
?? oya/forms/IPs/IP-sustainability-emission-model.md
?? oya/governance/IPs/IP-sustainability-emission-model.fixture.json
?? oya/hr/crates/oya-hr-employment-domain/tests/statutory_filing_manifest.rs
?? oya/hr/crates/oya-hr-employment-storage-adapter-postgres/
?? oya/identity/IPs/IP-sustainability-emission-model.fixture.json
?? oya/intelligence/IPs/IP-sustainability-emission-model.fixture.json
?? oya/intelligence/contracts/env-tier-model-budget-contract.json
?? oya/intelligence/contracts/fixtures/
?? oya/mail/IPs/IP-sustainability-emission-model.md
?? oya/mail/iac/helm/templates/_helpers.tpl
?? oya/mail/iac/helm/templates/ciliumnetworkpolicy-mail-edge.yaml
?? oya/meet/IPs/IP-sustainability-emission-model.md
?? oya/messenger/IPs/IP-sustainability-emission-model.fixture.json
?? oya/notes/IPs/IP-sustainability-emission-model.md
?? oya/observability/IPs/IP-sustainability-emission-model.md
?? oya/ontology/IPs/IP-sustainability-emission-model.fixture.json
?? oya/ops-dashboard-control-center/IPs/IP-sustainability-emission-model.md
?? oya/payments/crates/oya-payments-charge-domain/tests/
?? oya/payroll/crates/oya-payroll-run-api/tests/hr_payroll_boundary.rs
?? oya/payroll/crates/oya-payroll-run-api/tests/statutory_preview.rs
?? oya/payroll/crates/oya-payroll-run-domain/tests/statutory_calculation.rs
?? oya/payroll/crates/oya-payroll-run-domain/tests/statutory_source_pack.rs
?? oya/payroll/crates/oya-payroll-run-domain/tests/year_end_settlement.rs
?? oya/payroll/crates/oya-payroll-run-infrastructure/tests/local_close_replay.rs
?? oya/payroll/crates/oya-payroll-run-infrastructure/tests/statutory_replay.rs
?? oya/payroll/crates/oya-payroll-run-storage-adapter-postgres/
?? oya/policy/crates/oya-policy-cedar-domain/src/rebac.rs
?? oya/policy/crates/oya-policy-cedar-domain/tests/
?? oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/cedar.rs
?? oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/repository.rs
?? oya/recordings/IPs/IP-sustainability-emission-model.md
?? oya/sheets/IPs/IP-sustainability-emission-model.md
?? oya/sites/IPs/IP-sustainability-emission-model.md
?? oya/slides/IPs/IP-sustainability-emission-model.md
?? oya/social/IPs/IP-sustainability-emission-model.md
?? oya/social/crates/
?? oya/supply-chain-planning/iac/PROVENANCE-INVENTORY.md
?? oya/tasks/IPs/IP-sustainability-emission-model.md
?? oya/translate/IPs/IP-sustainability-emission-model.md
?? oya/treasury/crates/oya-treasury-cash-domain/tests/env_tier_outbound_metadata.rs
?? oya/trust/
?? oya/workflow-engine/IPs/IP-sustainability-emission-model.fixture.json
?? oya/workflow-engine/contracts/env-tier-run-handoff-contract.yaml
?? oya/workflow-engine/crates/oya-workflow-engine-execution-engine-usecase/tests/
?? oya/workflow-engine/policy/env-tier-run-start.cedar
?? oya/workflow-studio/IPs/IP-sustainability-emission-model.md
?? oya/workflow-studio/contracts/env-tier-promotion-contract.yaml
?? oya/workflow-studio/policy/env-tier-promotion.cedar
?? oya/workplace-integration/contracts/env-tier-outbound-emission-plan.contract.json
?? oya/workplace-integration/contracts/fixtures/
?? oya/workplace-integration/crates/oya-workplace-integration-outbound-metadata-domain/
?? plan/cloud-quality-kits/
?? plan/community/
?? plan/compliance-selective-cell-placement-architecture.md
?? plan/tasks/
?? registry/lts-pins.yaml
?? scripts/tests/anonymous_prd_red_fixture_contract_check.py
?? scripts/tests/calendar_prd_red_fixture_contract_check.py
?? scripts/tests/calendar_user_story_red_fixture_check.py
?? scripts/tests/community_fd001_red_fixture_contract_check.py
?? scripts/tests/conf_001_hyperscaler_conformance_check.py
?? scripts/tests/finops_collab_emission_models_check.py
?? scripts/tests/global_trade_inventory_authority_check.py
?? scripts/tests/hr_cloud_deployment_evidence_plan_check.py
?? scripts/tests/hr_group_ops_scale_plan_check.py
?? scripts/tests/hr_runtime_audit_event_registry_contract_check.py
?? scripts/tests/meet_source_map_contract_replay_check.py
?? scripts/tests/payroll_audit_event_registry_contract_check.py
?? scripts/tests/qk_01_overload_fairness_future_harness_check.py
?? scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py
?? scripts/tests/qk_03_privacy_data_governance_future_harness_check.py
?? scripts/tests/qk_04_canary_prr_future_harness_check.py
?? scripts/tests/qk_05_focus_cost_export_future_harness_check.py
?? scripts/tests/qk_06_k8s_pod_security_future_harness_check.py
?? scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py
?? scripts/tests/recordings_prd_red_fixture_contract_check.py
?? scripts/tests/sheets_source_map_authority_check.py
?? scripts/tests/slides_prd_red_fixture_contract_check.py
?? scripts/tests/social_prd_red_fixture_contract_check.py
?? scripts/tests/tasks_red_fixture_contract_check.py
?? scripts/tests/tls_001_ech_runtime_reconciliation_check.py
?? scripts/tests/translate_source_map_authority_check.py
?? skills-lock.json
?? specs/bespoke-scm-virtual-materialization-plan.json
?? specs/compliance-security-radar-cadence-contract.json
?? specs/dogfood/
?? specs/fixtures/anonymous-prd/
?? specs/fixtures/calendar-prd/
?? specs/fixtures/community-fd001/
?? specs/fixtures/crate-adr-design-doc-coverage/
?? specs/fixtures/drive/
?? specs/fixtures/hr-cloud-deployment/
?? specs/fixtures/hr-group-ops-scale/
?? specs/fixtures/meet/
?? specs/fixtures/passkey-kyc-policy-context/
?? specs/fixtures/recordings-prd/
?? specs/fixtures/sheets/
?? specs/fixtures/slides/
?? specs/fixtures/social/
?? specs/fixtures/tasks/
?? specs/fixtures/translate/
?? specs/microservices/community.json
?? specs/microservices/drive.json
?? specs/microservices/forms.json
?? specs/microservices/meet.json
?? specs/microservices/recordings.json
?? specs/microservices/sheets.json
?? specs/microservices/sites.json
?? specs/microservices/slides.json
?? specs/microservices/tasks.json
?? specs/microservices/translate.json
?? specs/passkey-kyc-policy-context-contract.json
?? specs/pci-dss-level-1-readiness-plan.json
?? specs/platform-vertical-status.json
?? specs/portfolio-ownership-ratchet.json
?? specs/regulatory-identity-kyc-policy-evidence-architecture.json
?? specs/regulatory-identity-source-of-truth.json
?? specs/regulatory-legacy-gate-scanner-normalization-table.json
?? specs/reorg/
?? specs/root-of-trust-ceremony-contract.json
?? specs/security-validation-pipeline-matrix.json
?? specs/sre-operations-center-contract.json
?? specs/structural-blockers.json
?? specs/trust-center-compliance-evidence-portal.json
?? specs/vulnerability-intelligence-sbom-vex-pipeline.json
codex
[ULTRAQA Cycle 1/1] The audit surface is confirmed dirty far beyond the five target files, so all evidence collection will be non-mutating and target-scoped; no runtime state or harness files will be created. I’m splitting three independent read-only lanes: workflow topology/counts, affected-set fail-closed correctness, and cache/setup/materialization rigor. The supplied timings remain the baseline; local YAML and git history will determine whether each proposed saving is credible.
exec
/bin/zsh -lc 'git status --short -- .github/workflows/oya-ci-required.yml .github/workflows/docs-graph-drift.yml infra/ci/install-buck2.sh infra/ci/buck2-affected-gate.sh infra/ci/materialize-cloud-ci-generated-faces.sh' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
 M .github/workflows/oya-ci-required.yml
Read-only UltraQA audit: establish authority and current topology, challenge optimization correctness in parallel, then synthesize a ranked exact-verdict plan.
  → Map governing contracts, target file state, jobs, calls, caches, and dependencies
  • Adversarially validate affected-set, cache/artifact, setup, Postgres, materialization, and runner proposals
  • Rank savings and deliver line-specific rigor proofs plus current defects
exec
/bin/zsh -lc "jq '{entry_points, agent_quick_start_protocol}' specs/root-hub-pointers.json" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc 'rg -n -i "oya-ci-required|buck2-affected|buck-out|stale worktree|cloud-ci fail-closed|ADR-0515" /Users/jasonlee/.codex/memories/MEMORY.md' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
191:- review all open PR, gh pr list, mergeStateStatus, oya-ci-required, preserve/hermes-w1-dirty-20260630, git worktree list --porcelain, isolated worktree, terse inventory
212:- For a live Oyatie PR sweep, merge authority is reviewer approval plus the protected `oya-ci-required` context; local `oya` CLI or other local-binary output is only bridge evidence, and the dirty root checkout should stay untouched while fixes happen in isolated worktrees [Task 5] [ad-hoc note]
311:- deep-interview, full_replan_first, omx question, registry/fixuptasks.jsonl, PR #967, PR #968, oya-ci-required, current-head review, paused goal, checkpoint rejected, G001-complete-oyatie-through-small-merge
338:- The engineering evidence was real in both runs: PRs `#911`, `#912`, and `#913` had already merged with `42/42` green, then PRs `#967` and `#968` merged with `41/41` green including `oya-ci-required` after current-head review [Task 1][Task 2]
560:# Task Group: Oyatie g004 cloud-ci fail-closed gate hardening and worker closeout
748:- cloud/cloud-storage/manifest.json, cloud/cloud-data/manifest.json, stale crates references, storage/core/domain, data/core/cloud-domain, buck2 targets, PR #930, oya-ci-required
772:- billing, finops, marketplace, metering seam, claim_token, lease_expired, manifest.json, contract_traceability_nonclaim, gh pr checks, oya-ci-required, PR #932
900:- wave-c1-hyperscaler-p-1fb6d50c, task 9, PR #927, PR #929, superseded, invalid_transition, claim-task, transition-task-status, gh pr diff, 42/42 checks, oya-ci-required
925:- PR #927 became the authoritative successor artifact; the durable proof was a clean spec-only diff (`specs/cloud-hyperscaler-parity-taxonomy.json`, `specs/cloud-observability-slo-target.json`, `specs/cloud-resource-catalog-target.json`) plus `42/42` green GitHub checks including `oya-ci-required`, `buck2`, `gate-live-postgres`, and `cloud-ci-firewall` [Task 1][Task 2]
1154:- PR #912, task: retire governance hook shell bridge, scripts/tests/governance-hooks-retired-vcs-surfaces.test.sh, rust-first-automation-policy.json, claim_conflict, terminal reconciliation, green oya-ci-required, Buck2, do not claim Task 4 or Task 7
1259:- Generated-output governance was already present in `.github/workflows/oya-ci-required.yml`, `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app`, `cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app`, `cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs`, and `registry/generated-artifact-control-plane.json`; the central truth is the generated-artifact control-plane manifest, not hand edits to `*.generated.json` [Task 2]
1358:- waveA-market-billing-20260625173629, marketplace/billing/payments, AGENTS.md, root-hub-pointers, no-op evidence, non-login shell, sleep 3; /bin/echo ready, oya-ci-required, generated-json-faces
1419:- read-only-team10-b-la-9c35bb10, team10b-readonly-20260625T055503Z.md, dependency-policy.md, ADR-0535, Renovate, Dependabot, oya-deps.toml, no-cargo-enforcer.sh, oya-ci-required, generated-artifact-control-plane.json, materialize-cloud-ci-generated-faces.sh, rust-first-automation-policy.json, shell/Python retirement, cloud-ci architecture
1433:- Team10-B reusable findings: `docs/standards/dependency-policy.md` still points at Renovate while `docs/decisions/ADR-0535-cross-product-versioning-release-governance.md` rejects Renovate/Dependabot in favor of an in-house `oya-deps.toml` bump-bot; `tools/hooks/no-cargo-enforcer.sh` blocks direct cargo build/test flows locally while `oya-ci-required` still carries cargo bridge legs; generated JSON faces are controller/materialized surfaces, not hand-edit targets [Task 2]
1434:- Team10-B policy surfaces worth opening first for future follow-up are `registry/generated-artifact-control-plane.json`, `infra/ci/materialize-cloud-ci-generated-faces.sh`, `cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app/rust-first-automation-policy.json`, and `.github/workflows/oya-ci-required.yml` [Task 2]
 succeeded in 0ms:
{
  "entry_points": {
    "session_handoff": {
      "current_path": "HANDOFF.md",
      "kind": "doc",
      "migration_phase": "bounded-root-markdown-exception-until-machine-readable-successor",
      "owner_team": "founder + platform-governance",
      "purpose": "Fresh-session handoff at repo root (founder directive 2026-06-08): cross-repo state, full backlog, hard guardrails, and the sibling/kernel consolidation map. Retained as a bounded root Markdown exception, not as a fourth pointer hub; agents read this after the root pointer hubs to resume with zero context loss.",
      "authority_boundary": "Session state/backlog summary only; it must not override README.md, CLAUDE.md, AGENTS.md, docs/AGENTS.md, accepted ADRs, or machine-readable specs/registries.",
      "freshness_rule": "Audit when root-hub pointers or repo-hygiene automation change, and treat claims older than 30 days as stale unless refreshed by a governance/docs task.",
      "retirement_rule": "Migrate equivalent handoff state to a machine-readable session-handoff registry, then remove HANDOFF.md from this entry and root Markdown allowlists in the same cohesion slice.",
      "target_path_after_md_retirement": "machine-readable session-handoff registry successor"
    },
    "_retired_constitutional_authority": {
      "retired_on": "2026-05-15",
      "retirement_note": "Per user directive 2026-05-13 'i dont think constitution is necessary'. Strike executed 2026-05-15. Content redistributed to 4 machine-readable successor specs (see decision_principles, forbidden_operations, decision_rights, governance_amendment entry_points below). docs/CONSTITUTION.md deleted; crates/oya-check-constitution-cite deleted; oya-dev-cli constitution-cite gate removed. Per-file citation sweep tracked as follow-up.",
      "successor_specs": [
        "decision_principles",
        "forbidden_operations",
        "decision_rights",
        "governance_amendment"
      ]
    },
    "active_artifact_contract": {
      "current_path": "/specs/active-machine-readable-artifact-contract.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "v3.0.0 9-capability contract per ADR-0069. Every machine-readable artifact declares enforcement+verification+validation+autogen+selfheal+selfupdate+selfmaintain+telemetry+provenance.",
      "target_path_after_md_retirement": "same (already machine-readable)"
    },
    "adr_0217_vertical_rollout_order": {
      "current_path": "docs/decisions/ADR-0217-vertical-slice-rollout-order.md",
      "kind": "decision",
      "migration_phase": "fd001-planning-closure",
      "purpose": "Accepted decision record for FD-001 vertical rollout order. Direct authority for Tenant/RBAC-packaged core microservices (FD-001) first, full-depth/no-MVP posture, Ops Dashboard / Control Center scope, canonical base plus Korea localization pack, clean architecture, API-first contracts, independent horizontal scaling, hyperscaler patterns, and false-green/silent-regression rejection.",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0346_oya_verify_ci_mirror": {
      "current_path": "docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "ADR-0346 legacy local-mirror verifier authority as amended by ADR-0513/platform-readiness: `./bin/oya verify --ci-required` is migration/local feedback evidence only, not protected-branch merge/exit authority; destination enforcement is cloud-ci/oya-ci required contexts plus Rust gate packets.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZA-oya-verify-full-ci-mirror",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0347_foundry_fitness_governance_bulk_rename": {
      "current_path": "docs/decisions/ADR-0347-governance-fitness-bulk-rename.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "ADR-0347 authority for the single Wave 15-ZB bulk rename of `oya-governance-*` lane prefixes to `oya-governance-*` and the associated residue, vocabulary, and inventory-presence governance lanes.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZB-foundry-fitness-to-governance-rename",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0348_autosharding_auto_rebalance_dynamic_sharding": {
      "current_path": "docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "ADR-0348 authority for cellular autosharding, auto-rebalance, and dynamic sharding doctrine, including the per-µservice `sharding_automation` manifest block and audit-chain emission requirements.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZD-autosharding-doctrine",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "adr_0349_jenkins_argocd_ci_cd_substrate": {
      "current_path": "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md",
      "kind": "decision",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "Historical bridge ADR for self-hostable CI/CD substrate surfaces (superseded; current authority is /specs/bespoke-cloud-toolchain-services.json: bespoke Rust cloud-ci/cloud-cd services). Legacy CI/CD adapters may be used only as bridge/reference adapters with deletion criteria, tenant isolation, trusted status production, and no sole/canonical destination claim.",
      "sequencing_pointer": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZE-jenkins-argocd-self-hostable-ci-cd-substrate",
      "target_path_after_md_retirement": "same until ADR projection retirement"
    },
    "agent_durable_goal": {
      "current_path": "/evidence/goals/fd001-planning-closure-implementation-goal-2026-05-19.json",
      "kind": "goal_prompt",
      "migration_phase": "fd001-planning-closure-supersedes-2026-05-16-durable-goal",
      "purpose": "Historical FD-001 durable-goal prompt for planning closure/implementation. Its plain-git protected-PR governance portions are superseded/amended by platform-readiness: reviewer approval plus the single protected `oya-ci-required` context is destination authority; legacy CI bridge output is evidence only. The former .omc archive manifest is retired from the tracked tree and is not live authority.",
      "superseded_path": "/specs/agent-durable-goal.json",
      "target_path_after_md_retirement": "same active goal prompt until promoted into /specs/agent-durable-goal.json successor schema",
      "retired_archive_manifest_path": ".omc/archive/stale-documents/2026-05-19-planning-closure/manifest.json",
      "archive_manifest_status": "retired-from-tracked-tree-local-only; use git history for provenance, not live authority"
    },
    "agent_operating_contract": {
      "current_path": "docs/AGENTS.md",
      "kind": "spec",
      "migration_phase": "PHASE-5",
      "phase_deadline": "2026-06-30",
      "phase_status": "overdue-needs-replan",
      "promotion_boundary": "docs/AGENTS.md remains current authority until explicit PHASE-5 promotion evidence promotes /specs/agent-operating-contract.json; the missed PHASE-5 deadline does not auto-promote the projection.",
      "purpose": "Canonical agent operating contract (done-definition checklist, plain-git protected-PR governance, single oya-ci-required merge authority, multispectrum evidence, legacy-tool retirement notes).",
      "target_path_after_md_retirement": "/specs/agent-operating-contract.json"
    },
    "agent_operating_contract_machine_projection": {
      "canonical_authority_path": "docs/AGENTS.md",
      "current_path": "/specs/agent-operating-contract.json",
      "kind": "spec",
      "migration_phase": "projection-until-explicit-PHASE-5-promotion",
      "phase_deadline": "2026-06-30",
      "phase_status": "overdue-needs-replan",
      "promotion_condition": "Promotion requires authority-cohesion evidence and reviewer approval; until then this projection is discovery/planning support only.",
      "purpose": "Machine-readable projection target for the agent operating contract. It supports root-hub discovery without superseding docs/AGENTS.md before explicit PHASE-5 promotion evidence."
    },
    "api_contract_ssot_canonical": {
      "current_path": "/specs/api-contract-ssot-canonical.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "API contract SSOT (CS-LAUNCH-API-CONTRACT-SSOT-001): one Rust-native source generating/validating REST(OpenAPI 3.2.0)+gRPC(proto3)+GraphQL SDL; GraphQL first-class derived not hand-maintained; api-contract-ssot-drift gate. Pulsar launch-primary; Kubewarden default. Target spec; no generators/runtime claimed.",
      "target_path_after_md_retirement": "same"
    },
    "artifact_capabilities_registry": {
      "current_path": "/registry/artifact-capabilities-registry.json",
      "kind": "registry",
      "migration_phase": "complete (10 baseline rows; per-artifact rows added incrementally)",
      "purpose": "Control plane: one row per machine-readable artifact + artifact_profile + capability_overrides.",
      "target_path_after_md_retirement": "same"
    },
    "artifact_profile_defaults": {
      "current_path": "/specs/artifact-profile-defaults.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "7 profiles (schema/registry/template/plan-attestation/ledger/claim-matrix/evidence-bundle); reduces per-row authoring burden.",
      "target_path_after_md_retirement": "same"
    },
    "audit_chain": {
      "current_path": "/evidence/audit-chain.jsonl",
      "kind": "ledger",
      "migration_phase": "scaffold (F-EVIDENCE-AUDIT-CHAIN-WIRE pending for ADR-0069 cryptographic-immutability integration)",
      "purpose": "Append-only JSONL stream of changeset evidence emissions, lane runs, spec-version bumps, ADR acceptances, IP status flips, seam audit baselines. Schema: {event_type, change_id?, session_id, timestamp_unix, payload}."
    },
    "audit_kg_robustness": {
      "current_path": "/registry/kg-audit/index.json",
      "kind": "audit",
      "migration_phase": "complete",
      "purpose": "Read-only robustness audit of the 3-layer KG against dev + governance use cases. Findings filed as F-KG-01..06 in registry/fixuptasks.jsonl. Refreshed quarterly OR after any knowledge-graph schema change.",
      "target_path_after_md_retirement": "same"
    },
    "bespoke_cloud_toolchain_services": {
      "current_path": "/specs/bespoke-cloud-toolchain-services.json",
      "kind": "spec",
      "migration_phase": "current-machine-readable-authority",
      "purpose": "Product and sequencing spec for tenant-facing bespoke Rust Oyatie Cloud SCM, CI, and CD services. Defines bridge-adapter boundaries for GitHub/cloud-scm/Argo, full bespoke CI enforcement baseline, masterplan P-TOOLCHAIN placement, and mandatory secure separation between tenant=oyatie-internal and every customer tenant pipeline.",
      "target_path_after_md_retirement": "same"
    },
    "bespoke_scm_virtual_materialization_plan": {
      "current_path": "/specs/bespoke-scm-virtual-materialization-plan.json",
      "kind": "spec",
      "migration_phase": "w4-design-spike-prototype",
      "purpose": "W4-003 machine-readable design spike/prototype for mapping content-addressed WorkAreaTree records to reversible materialized file views during the ADR-0518 ISOLATE/AUTHOR stages. Metadata-only; no native SCM storage, virtual filesystem runtime, object-store runtime, parser runtime, CD runtime, or bridge cutover claim.",
      "target_path_after_md_retirement": "same (machine-readable W4 virtual materialization plan/prototype)"
    },
    "security_validation_pipeline_matrix": {
      "current_path": "/specs/security-validation-pipeline-matrix.json",
      "kind": "spec",
      "migration_phase": "planning-gate-matrix",
      "purpose": "Productized runner-neutral security validation pipeline matrix. Defines SAST, DAST, IAST, SCA, secrets, IaC, container, fuzzing, API fuzzing, BAS/purple-team, security chaos, automated pen-test, and continuous-control evidence lanes with scope, cadence, pass/fail policy, false-positive/VEX handling, and API evidence records.",
      "target_path_after_md_retirement": "same"
    },
    "trust_center_compliance_evidence_portal": {
      "current_path": "/specs/trust-center-compliance-evidence-portal.json",
      "kind": "spec",
      "migration_phase": "planning-product-surface-contract",
      "purpose": "Tenant-scoped Trust Center / Compliance Evidence Portal product-surface contract. Maps security-validation, SBOM/VEX, compliance-pack, SLO/DR/status/incident, release, quality-kit, and audit-chain evidence into customer/admin UX and API surfaces with data-access rules, freshness badges, export/auditor-room flows, and non-certification claim boundaries.",
      "target_path_after_md_retirement": "same"
    },
    "check_empirical_evidence": {
      "current_path": "/registry/check-empirical-evidence",
      "kind": "registry",
      "migration_phase": "complete",
      "purpose": "Evidence records proving a deterministic score card has caught or prevented at least one regression before BLOCKER promotion.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "ci_farm_substrate_canonical": {
      "current_path": "/specs/ci-farm-substrate-canonical.json",
      "kind": "spec",
      "migration_phase": "wave-3-consolidation",
      "purpose": "Bridge/reference distributed CI farm substrate (cloud-ci adapter, ephemeral agents, sccache->SeaweedFS remote cache, lane fanout, merge-queue fan-in). Authored under ADR-0349; retained only as cloud-ci transition evidence, not permanent product authority.",
      "target_path_after_md_retirement": "same"
    },
    "ci_fix_loop_context_bundle": {
      "current_path": "/specs/ci-fix-loop-context-bundle.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Canonical shape of the context bundle a ci_fixer role consumes when diagnosing CI lane failures.",
      "target_path_after_md_retirement": "same"
    },
    "lane_supervisor_bridge": {
      "current_path": "registry/catalog/oya-lane-supervisor-app.yaml",
      "kind": "registry",
      "migration_phase": "local-bridge-until-cloud-ci-lane-state",
      "purpose": "Retirement-marked local bridge catalog row for the lane-supervisor app. Supporting local-only surfaces are .omc/ultragoal/OWNERS, .omc/ultragoal/TEAMMATE-PREAMBLE.md, .omc/ultragoal/friction-ledger.jsonl, .omc/ultragoal/premise.txt, .omc/ultragoal/review-verdict.txt, registry/catalog/OWNERS, and registry/catalog/oya-lane-supervisor-app.yaml. Merge authority remains in cloud-ci/oya-ci required contexts per ADR-0363.",
      "target_path_after_md_retirement": "cloud-ci durable lane orchestration state"
    },
    "cloud_authorization_target": {
      "current_path": "/specs/cloud-authorization-target.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "Cloud authorization target (P-TARGET): Cedar as universal gate for the control plane + cloud-* resources (ADR-0243); default-deny; KR/JP/US/EU pack overlays; references cedar-policy/fragment schemas. Target spec; no runtime.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_control_plane_canonical": {
      "current_path": "/specs/cloud-control-plane-canonical.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "Cloud Control Plane canonical target architecture (P-TARGET): resource model (Org->...->Resource), ORN, API Gateway->Resource Registry->Operation Ledger->Workflow/Reconciler->OpenTofu/operators/Argo, per-resource quota/billing/audit contract, durable long-running operations, nine control-plane facets. Target spec; no runtime claimed.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_observability_slo_target": {
      "current_path": "/specs/cloud-observability-slo-target.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "Cloud observability+SLO target (P-TARGET): OpenTelemetry + measured SLOs + burn-rate + agentic SLO-gated promotion (ADR-0130/0340) for the cloud-* substrate. Target spec; no measured evidence claimed.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_production_quality_kits_target": {
      "current_path": "/specs/cloud-production-quality-kits-target.json",
      "kind": "spec",
      "migration_phase": "p-prod",
      "purpose": "Cloud production quality kits target (P-PROD): seven harness-backed evidence gates (overload/fairness, shuffle-shard isolation, privacy/data-governance, canary+PRR, FOCUS cost, K8s PSS, abuse/fraud/DDoS) each with scenarios+evidence-format+feeding production_100_bar gate. Incorporated from plan/ quality review. Target spec; no harnesses/evidence implemented.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_resource_catalog_target": {
      "current_path": "/specs/cloud-resource-catalog-target.json",
      "kind": "spec",
      "migration_phase": "p-target",
      "purpose": "Cloud resource catalog target (P-TARGET): per-resource-type architecture for all cloud-* services (IAM/KMS/Secrets/Compute/Storage/Network/Data/Billing/Capacity), each instantiating the control-plane per-resource contract (quota/billing/audit/lifecycle/slo/retention), Cedar-gated + OTel-instrumented + actuated via OpenTofu/operator/Argo. Target spec; metadata-only until P-STRANGLE.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_strangler_migration_target": {
      "current_path": "/specs/cloud-strangler-migration-target.json",
      "kind": "spec",
      "migration_phase": "p-strangle",
      "purpose": "Strangler-fig migration playbook (P-STRANGLE): per-service pattern (seam->build-ideal->facade->shadow/canary->cutover->retire), parallel worktree-lane model + single-writer invariant + Buck2/Cargo affected-target feedback + full oya-ci-required/cloud-ci merge-authority backstop, per-migration DoD, cutover/rollback criteria. Target playbook; no migration executed.",
      "target_path_after_md_retirement": "same"
    },
    "cloud_toolchain_target": {
      "current_path": "/specs/cloud-toolchain-target.json",
      "kind": "spec",
      "migration_phase": "p-toolchain",
      "purpose": "Superseded bridge target for P-TOOLCHAIN. Current authority is /specs/bespoke-cloud-toolchain-services.json: Rust cloud-scm/cloud-ci/cloud-cd services with GitHub/cloud-scm/Argo as adapters, full bespoke CI enforcement, tenant-isolated pipelines, affected-target feedback plus full required backstop, merge-queue speculative validation, and progressive delivery.",
      "target_path_after_md_retirement": "same"
    },
    "codeview_read_surface": {
      "current_path": "/specs/codeview-read-surface.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Canonical read surface for code-view queries; what an agent can ask for + what fields are returned.",
      "target_path_after_md_retirement": "same"
    },
    "crate_naming_audit": {
      "current_path": "/specs/crate-naming-audit.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Per-package naming audit including retired_package_notes for traceability (e.g., oya-governance-archive-orphan retired by ADR-0118).",
      "target_path_after_md_retirement": "same"
    },
    "decision_principles": {
      "current_path": "/specs/decision-principles.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "10 normative do-rules every contributor and agent follows. Each principle declares enforcement mode (mechanical / cultural / mechanical-stub / deferred) and cites the lanes enforcing it. Successor to docs/CONSTITUTION.md §Decision-principles — Do.",
      "target_path_after_md_retirement": "same (already machine-readable)"
    },
    "decision_rights": {
      "current_path": "/specs/decision-rights.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "12-class decision-rights table (owner + escalation per class). Successor to docs/CONSTITUTION.md §Decision-rights. Per-team RACI detail still in docs/RACI-OWNERSHIP.md.",
      "target_path_after_md_retirement": "same"
    },
    "deployment_ops_contract": {
      "current_path": "/specs/deployment-ops-contract.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Deployment entrypoint contract (per ADR-0375): OpenTofu owns only the Cloudflare edge; the cluster fleet is provisioned by Cluster API + Talos (installation-media zero-touch) + per-cell Argo CD; root Makefile owns operator entry; no manual SSH troubleshooting.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "design_audit_evidence_timeline": {
      "current_path": "/specs/design-system/audit-evidence-timeline.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Append-only evidence timeline for PRs, promoted changesets, agent decisions, incident closeout, and compliance controls.",
      "target_path_after_md_retirement": "same"
    },
    "design_cloud_cell_topology_map": {
      "current_path": "/specs/design-system/cloud-cell-topology-map.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Operational topology map for cells, regions, tenant routing, capacity, canary state, and deployment ownership without manual SSH troubleshooting.",
      "target_path_after_md_retirement": "same"
    },
    "design_communication_thread_list": {
      "current_path": "/specs/design-system/communication-thread-list.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Reusable mail/messenger/calendar thread-list component row with policy badges, retention/hold indicators, accessibility, and progressive loading states.",
      "target_path_after_md_retirement": "same"
    },
    "design_entity_action_policy_preview": {
      "current_path": "/specs/design-system/entity-action-policy-preview.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Pre-execution preview of Cedar policy, autonomy tier, data-class effects, and audit consequences for typed Ontology actions.",
      "target_path_after_md_retirement": "same"
    },
    "design_foundry_agent_run_timeline": {
      "absorbed_by": "intelligence",
      "absorbing_authority_adr": "ADR-0335",
      "current_path": "/specs/design-system/foundry-agent-run-timeline.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Timeline for autonomous agent runs, lane transitions, evidence attachments, verifier decisions, and rollback/fix loops. Component name retained for cross-reference continuity; ownership now lives under intelligence + workflow per ADR-0335 (Wave 15I).",
      "successor_owner": "intelligence (agent-run timeline is a sub-surface of intelligence dispatch-flow + workflow per ADR-0335 Wave 15I)",
      "target_path_after_md_retirement": "same"
    },
    "design_ontology_graph_explorer": {
      "current_path": "/specs/design-system/ontology-graph-explorer.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Typed entity/link/action graph explorer for schema admins, vertical specialists, and developers inspecting semantic, kinetic, and dynamic layers.",
      "target_path_after_md_retirement": "same"
    },
    "design_ops_deployment_status_panel": {
      "current_path": "/specs/design-system/ops-deployment-status-panel.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Deployment status surface for OpenTofu plans, oya ops actions, canary phases, rollback state, and operator-safe remediation links.",
      "target_path_after_md_retirement": "same"
    },
    "design_policy_disclosure_banner": {
      "current_path": "/specs/design-system/policy-disclosure-banner.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Policy disclosure banner component row for retention, legal hold, consent, audit access, and Workflow handoff consequences.",
      "target_path_after_md_retirement": "same"
    },
    "design_score_card_result_table": {
      "current_path": "/specs/design-system/score-card-result-table.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Deterministic score-card results table for lane pass/fail evidence, forbidden LLM-judgment status, source citations, and remediation links.",
      "target_path_after_md_retirement": "same"
    },
    "design_spec_diff_viewer": {
      "current_path": "/specs/design-system/spec-diff-viewer.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Semantic diff viewer for canonical JSON specs, PR review, schema evolution, jurisdiction overlays, and LLM-authored patch review.",
      "target_path_after_md_retirement": "same"
    },
    "design_spec_maturity_deferred_surfaces": {
      "current_path": "/registry/design-spec-maturity/wave-3-i-deferred-surfaces.tsv",
      "kind": "registry",
      "migration_phase": "wave-3-consolidation",
      "purpose": "Wave-3-I deferred design/spec-maturity surfaces ratchet; transparent deferral of not-yet-authored service surfaces (operational_claim stays blocked_until_operational_evidence).",
      "target_path_after_md_retirement": "same"
    },
    "design_system_catalog": {
      "current_path": "/specs/design-system/catalog.json",
      "kind": "design-system-catalog",
      "migration_phase": "complete",
      "purpose": "Machine-readable design-system catalog for PRD template v1.5 §11f/§11g UX/frontend-component coverage.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "design_tenant_context_switcher": {
      "current_path": "/specs/design-system/tenant-context-switcher.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Tenant context switcher design-system component row: personal/work/admin-audit switching with tenant/RBAC scope, cache isolation, and policy re-evaluation invariants.",
      "target_path_after_md_retirement": "same"
    },
    "design_workflow_canvas": {
      "current_path": "/specs/design-system/workflow-canvas.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Node-edge authoring canvas for durable Workflow definitions with explicit state, branch, join, capability-call, and jurisdiction-overlay affordances.",
      "target_path_after_md_retirement": "same"
    },
    "design_workflow_node_config_panel": {
      "current_path": "/specs/design-system/workflow-node-config-panel.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Inspector panel for node parameters, typed inputs/outputs, policy preview, validation errors, and jurisdiction-specific overrides.",
      "target_path_after_md_retirement": "same"
    },
    "design_workflow_replay_timeline": {
      "current_path": "/specs/design-system/workflow-replay-timeline.json",
      "kind": "design-system-component",
      "migration_phase": "complete",
      "purpose": "Step-by-step execution replay, failure inspection, audit-chain correlation, and rollback/retry decision support for Workflow operators.",
      "target_path_after_md_retirement": "same"
    },
    "doc_catalog": {
      "current_path": "docs/DOC-CATALOG.md",
      "kind": "registry",
      "migration_phase": "PHASE-5",
      "purpose": "Per-doc lifecycle protocol. Every canonical doc has class + owner + length cap + 'Does NOT cover' clause.",
      "target_path_after_md_retirement": "/registry/doc-catalog.json"
    },
    "evidence_taxonomy": {
      "current_path": "/specs/evidence-taxonomy.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "8 evidence classes + freshness rule + minimum_completion_set for ledger rows.",
      "target_path_after_md_retirement": "same"
    },
    "final_report_schema": {
      "current_path": "/specs/final-report-schema.json",
      "kind": "schema",
      "migration_phase": "complete",
      "purpose": "JSON Schema for MPR-... final autonomous master-plan completion reports.",
      "target_path_after_md_retirement": "same"
    },
    "fixuptask_registry": {
      "current_path": "/registry/fixuptasks.jsonl",
      "kind": "registry",
      "migration_phase": "complete",
      "purpose": "Append-only JSONL stream of bounded FixupTasks. One JSON record per line: {id, title, priority, status, source_session, source_change_id, named_in, created_at, blocker_for?}. Consumed by lane sub-checks + dashboards."
    },
    "forbidden_operations": {
      "current_path": "/specs/forbidden-operations.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "10 prohibitions with severity (critical / high / medium) + enforcement mode + lane citations. Successor to docs/CONSTITUTION.md §Prohibitions — Avoid. Companion to master-plan-sequencing.json forbidden_primitives (lower-level: tooling vs. doctrine).",
      "target_path_after_md_retirement": "same"
    },
    "gitops_vcs_replacement": {
      "current_path": "/specs/gitops-vcs-replacement.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "M01-P07 historical VCS replacement contract: semantic symbol locks + ChangeBundle/GitOps promotion/controller integration for CI/CD-safe agent work.",
      "target_path_after_md_retirement": "same"
    },
    "governance_amendment": {
      "current_path": "/specs/governance-amendment.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "6-step amendment procedure for the 4-file doctrinal substrate. Declares EVT-DOCTRINE-AMENDED audit-chain event. Successor to docs/CONSTITUTION.md §Amendments.",
      "target_path_after_md_retirement": "same"
    },
    "hyperscaler_gates": {
      "current_path": "/specs/hyperscaler-gates.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Hyperscaler maturity claim gates including plan, pipeline, toolchain, CI/CD, development cycle, guardrails, safety, UX, ease-of-use, competitor response, and HG-VCS claim authority.",
      "target_path_after_md_retirement": "same"
    },
    "hyperscaler_production_readiness_claim_contract": {
      "current_path": "/specs/hyperscaler-production-readiness-claim-contract.json",
      "kind": "spec",
      "migration_phase": "phase0_refinement_pending_masterplan_propagation",
      "purpose": "Machine-readable claim ceiling for production-ready, hyperscaler-grade, mechanically-enforced, secure, isolated, tenant-facing, retired, done, parity, full, complete, automatic, and equivalent language. Defines target/spec/enforced/production/hyperscaler tiers and evidence domains; blocks empty promises through cloud-ci/Rust gate evidence rather than new oya CLI surfaces.",
      "target_path_after_md_retirement": "same (machine-readable claim/evidence contract)"
    },
    "portfolio_ownership_ratchet": {
      "current_path": "/specs/portfolio-ownership-ratchet.json",
      "kind": "spec",
      "migration_phase": "proposed-for-architecture-governance-review",
      "purpose": "Machine-readable portfolio own-when-proven / day-0 ownership budget contract. Reconciles SOURCE Class A/B/C and LINUX OWN_DAY0/OWN_EARLY/DEFER_VENDORED/PERMANENT_REUSE rubrics, names the one-slot major day-0 budget, maps hyperscaler audit findings to dispositions, and proposes a cloud-ci/Rust gate against unsupported new own-day-0 claims.",
      "target_path_after_md_retirement": "same"
    },
    "iterative_fix_loop": {
      "current_path": "/specs/iterative-fix-loop.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Loop state machine for find→fix→re-review until all facets green. Bounded by iteration_budget. Escalates to human when budget exhausted."
    },
    "knowledge_graph_dynamic": {
      "current_path": "/registry/knowledge-graph-dynamic.json",
      "kind": "registry",
      "migration_phase": "complete (per Palantir 3-layer split commit 0806f91)",
      "purpose": "DYNAMIC layer: 9 live state sources + OTel telemetry pointers. Real-time graph state.",
      "target_path_after_md_retirement": "same"
    },
    "knowledge_graph_kinetic": {
      "current_path": "/registry/knowledge-graph-kinetic.json",
      "kind": "registry",
      "migration_phase": "complete (per Palantir 3-layer split commit 0806f91)",
      "purpose": "KINETIC layer: 14 action types + 4 workflows. Write-side mutations with audit topic + idempotency + lock pattern.",
      "target_path_after_md_retirement": "same"
    },
    "knowledge_graph_schema": {
      "current_path": "/specs/knowledge-graph-schema.json",
      "kind": "schema",
      "migration_phase": "complete",
      "purpose": "JSON Schema for the knowledge-graph 3-layer split (semantic + kinetic + dynamic) consumed by registry/knowledge-graph-*.json.",
      "target_path_after_md_retirement": "same"
    },
    "knowledge_graph_semantic": {
      "current_path": "/specs/microservices/ontology.json#type_system",
      "deprecated_path": "/registry/knowledge-graph-semantic.json (deleted 2026-05-17 per ADR-0139)",
      "kind": "registry",
      "migration_phase": "complete (content migrated to specs/products/ontology.json#type_system per ADR-0139 compulsory deprecation 2026-05-17)",
      "path_correction_2026_05_20": {
        "previous_path": "/specs/products/ontology.json#type_system",
        "reason": "specs/products is retired in this checkout; live machine-readable surface is under specs/microservices."
      },
      "purpose": "SEMANTIC layer: 36 node types + 27 edge types + 19 invariants + 11 read-side queries. Static structure of the knowledge graph. Now canonical home is specs/products/ontology.json#type_system.",
      "target_path_after_md_retirement": "same (already in ontology.json)"
    },
    "korea_localization_pack_manifest": {
      "current_path": "docs/localization-packs/kr/pack.yaml",
      "kind": "localization-pack-manifest",
      "migration_phase": "fd001-planning-closure",
      "overview": "docs/localization-packs/kr.md",
      "purpose": "Canonical KR localization pack manifest for FD-001 planning closure. Direct authority for lifecycle status, activation acceptance criteria, microservices in scope, regulatory bindings, connectors, acceptance milestones, and signed regulatory corpus references.",
      "regional_pack": "packs/kr/sovereignty/manifest.json",
      "target_path_after_md_retirement": "same or JSON successor after localization-pack manifest migration"
    },
    "korea_localization_pack_overview": {
      "current_path": "docs/localization-packs/kr.md",
      "kind": "localization-pack",
      "manifest": "docs/localization-packs/kr/pack.yaml",
      "migration_phase": "fd001-planning-closure",
      "purpose": "Human-compatible overview for the mandatory Korea localization pack. Direct authority pointer for FD-001 KR pack scope, activation acceptance, canonical-base separation, overlay crates, Cedar policy fragments, workflow templates, document templates, acceptance evidence, and operations ownership.",
      "regional_pack": "packs/kr/sovereignty/manifest.json",
      "target_path_after_md_retirement": "machine-readable successor after localization-pack projection retirement"
    },
    "korea_regional_pack": {
      "current_path": "packs/kr/sovereignty/manifest.json",
      "kind": "regional-pack",
      "localization_pack_manifest": "docs/localization-packs/kr/pack.yaml",
      "localization_pack_overview": "docs/localization-packs/kr.md",
      "migration_phase": "fd001-planning-closure",
      "purpose": "Regional KR pack authority for FD-001 planning closure. Direct pointer for Korea regulatory bindings, agency surfaces, i18n/currency/calendar/tax/payment/identity foundations, cross-border transfer constraints, and regional operational gates.",
      "target_path_after_md_retirement": "machine-readable successor after regional-pack projection retirement"
    },
    "loop_recovery_patterns": {
      "current_path": "/registry/loop-recovery-patterns",
      "kind": "registry",
      "migration_phase": "complete",
      "purpose": "Append-only repeat-loop recovery patterns linked to deterministic score-card ids and mistakes-ledger rows. Enforced by the loop-recovery-patterns gate; legacy `oya` mirror invocations are migration evidence only until ported to cloud-ci/Rust gate contexts.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "markdown_retirement_ledger": {
      "current_path": "/evidence/ledger/markdown-retirement-ledger.json",
      "kind": "ledger",
      "migration_phase": "complete (seed; per-row migration tracked over time)",
      "purpose": "Per-file rows tracking each Markdown retirement event (source_md_path / target_json_path / status / migration_date).",
      "target_path_after_md_retirement": "same"
    },
    "markdown_retirement_policy": {
      "current_path": "/specs/markdown-retirement-policy.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Migration policy per user 'we don't need markdown' directive. 8 phases; ~250 files to retire by 2026-08-31.",
      "target_path_after_md_retirement": "same"
    },
    "master_plan_sequencing": {
      "current_path": "/specs/master-plan-sequencing.json",
      "kind": "spec",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "purpose": "Forbidden-primitive list + plain-git protected-PR governance protocol + ledger-required-fields + execution sequence. Platform-readiness amends its older CI-bridge language: reviewer approval plus the single protected `oya-ci-required` context is destination authority; legacy CI/oya output is bridge/local evidence only.",
      "target_path_after_md_retirement": "same"
    },
    "masterplan": {
      "current_path": "/specs/masterplan.json",
      "human_compatibility_projection": "docs/MASTERPLAN.md",
      "kind": "spec",
      "migration_phase": "accelerated-machine-readable-authority-for-planning-closure",
      "purpose": "Machine-readable master plan authority for implementation sequencing, vertical delivery order, first-deliverable scope, planning closure, and readiness claims. The first deliverable is Tenant/RBAC-packaged core microservices (FD-001) with core, messenger, mail, community, infra, Ops Dashboard / Control Center, intelligence (replaces Foundry per ADR-0335 Wave 15I), Workflow, Ontology, canonical base, and Korea localization pack. Canonical service split is {oya,cloud}/<service> with libs/ and ADR-0010/0064 pack roots governed by closed-world inventory; microservices/<ms>/ is legacy and removed only after P0.6 verifies all migration packets. Clean architecture, API-first contracts, independent horizontal scaling, and hyperscaler pattern mapping are required before implementation packets begin.",
      "target_path_after_md_retirement": "same (machine-readable authority; docs/MASTERPLAN.md is compatibility projection only)"
    },
    "merge_queue_parked_pr": {
      "current_path": "/specs/merge-queue-parked-pr.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Semantics for parked PRs in the vcs-orchestrator merge queue (per ADR-0111); admission criteria + unpark transitions. Originally framed as the Foundry merge queue; ownership relocated to vcs-orchestrator per ADR-0335 (Wave 15I) — see ADR-0335 Absorption Map row 'Merge queue projected state'.",
      "target_path_after_md_retirement": "same"
    },
    "multispectrum_review": {
      "current_path": "/specs/multispectrum-review.json",
      "current_version": "2.1.0",
      "human_companion_path": "docs/standards/multispectrum-review.md",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Multispectrum review bar — 12 facets (F1..F13 minus F12-reserved) + 2 meta-facets (M1, M2). v2.0.0 added F8 performance + F9 compliance + meta-facets + enforcement_scopes (agentic_flow + dev_flow) + consensus_debate_protocol + alias_map. v2.1.0 added F10 reversibility + F11 observability + F13 migration + deterministic_layer_principle + scorecard_schema. Enforced mechanically by oya-check-dependency-seam sub-checks (multispectrum-evidence-attached, fixture-pair-coverage, change-class-declared) and by SessionStart hook reminder."
    },
    "multispectrum_evidence_g011_wiring": {
      "current_path": "evidence/multispectrum/g011-rust-test-wiring-generator-20260610-1781107105.json",
      "owners_path": "evidence/multispectrum/OWNERS",
      "kind": "evidence",
      "migration_phase": "complete",
      "purpose": "Multispectrum evidence bundle for G011 rust-test-wiring-generator batch (ADR-0540 local bridge generator); records CC-3 tooling-feature change class, wired libs, and tool accounting justification."
    },
    "oyatie_doctrine": {
      "current_path": "/specs/oyatie-doctrine.json",
      "current_version": "1.0.0",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Workspace-wide doctrine. Principles P0..P8 apply to every documentation / file / workflow / lane / ADR / spec / registry / code change. P0 (agentic-primary) elevates agent consumption above human; P1 (machine-optimized); P2 (programmatic-where-possible); P3 (deterministic-where-it-matters); P4 (enforce-in-every-thing); P5 (iterate-until-consensus); P6 (no-silent-regression); P7 (Bominal-inheritance-precedence); P8 (canonical-base + localization-pack). Cited by docs/AGENTS.md and every ADR."
    },
    "phase0_automation_matrix": {
      "current_path": "/specs/phase0-automation-matrix.json",
      "kind": "spec",
      "migration_phase": "phase0_refinement_seed_contract_not_green",
      "purpose": "Seed contract and initial rows for AC-0.16 automation ratchet. Requires every enforceable/automatable Phase-0 rule to map to a Rust gate crate, cloud-ci controller, generated registry, or fixture-backed bridge with retirement criteria; forbids new oya CLI authority.",
      "target_path_after_md_retirement": "same (machine-readable automation ratchet matrix)"
    },
    "phase0_ci_enforcement_baseline": {
      "current_path": "/specs/phase0-ci-enforcement-baseline.json",
      "kind": "spec",
      "migration_phase": "phase0_p0_0_gap_packet_not_green",
      "purpose": "P0.0 cloud-ci/oya-ci enforcement and tenant-isolation baseline (historically Prow-shaped). Records current branch-protection/config/live-status gaps, no-oya-CLI authority boundary, trusted producer/candidate-untrusted contract, TTL reviewer/audit override contract, and T0.0 RED/GREEN fixture paths without claiming Phase-0 completion.",
      "target_path_after_md_retirement": "same (machine-readable P0.0 baseline/gap packet)"
    },
    "phase0_claim_evidence_map": {
      "current_path": "/specs/phase0-claim-evidence-map.json",
      "kind": "spec",
      "migration_phase": "phase0_refinement_seed_contract_not_green",
      "purpose": "Seed claim/evidence map for AC-0.17. Regulated vocabulary is target/spec/enforced/production/hyperscaler tiered and must cite evidence or target/non-claim labels; blocks empty promises and advisory evidence masquerading as authority.",
      "target_path_after_md_retirement": "same (machine-readable claim/evidence map)"
    },
    "plan_schema": {
      "current_path": "/specs/plan-schema.json",
      "kind": "schema",
      "migration_phase": "complete",
      "purpose": "JSON Schema for ralplan consensus plans. Markdown plans are projections (being retired).",
      "target_path_after_md_retirement": "same"
    },
    "planning_closure_contract": {
      "current_path": "/specs/planning-closure-contract.json",
      "kind": "spec",
      "migration_phase": "fd001-planning-closure",
      "purpose": "Machine-readable closure contract for FD-001 planning: Tenant/RBAC-packaged core microservices (FD-001) full-depth delivery, canonical base, Korea localization pack, clean architecture, API-first contracts, hyperscaler patterns, portable deployment, and no false planning-green claims. Its older CI-governance wording is amended by platform-readiness/cloud-ci authority.",
      "status_ledger": "/specs/planning-closure-status-closure-ledger.json",
      "target_path_after_md_retirement": "same"
    },
    "planning_closure_status_ledger": {
      "contract_ref": "/specs/planning-closure-contract.json",
      "current_path": "/specs/planning-closure-status-closure-ledger.json",
      "kind": "ledger",
      "migration_phase": "fd001-planning-closure",
      "purpose": "Evidence ledger for the 2026-05-19 masterplan status closure: records the former blocking status count, closure rule, hyperscaler gap IP planning contracts, implementation guardrails, and exit evidence that must exist before any production or hyperscaler-grade claim.",
      "target_path_after_md_retirement": "same (machine-readable planning-closure evidence ledger)"
    },
    "prd_accounting": {
      "current_path": "/specs/microservices/accounting.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Accounting microservice PRD — double-entry ledger, Korean VAT/K-GAAP/K-IFRS evidence, AP/AR/procurement bridge, financial close workflow, and audit-ready controls.",
      "target_path_after_md_retirement": "same"
    },
    "prd_anonymous": {
      "current_path": "/specs/microservices/anonymous.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Anonymous µservice PRD — Sidechat / YikYak / Blind-class pseudonymous platform with cryptographic blinding. Per ADR-0126/0135 dissolution.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_calendar": {
      "current_path": "/specs/microservices/calendar.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Calendar microservice PRD — personal/work scheduling, action-card Workflow handoff, legal-hold overlays, and cross-context-safe availability projections without a product-group wrapper.",
      "target_path_after_md_retirement": "same"
    },
    "prd_community": {
      "current_path": "/specs/microservices/community.json",
      "kind": "prd",
      "migration_phase": "fd001-authority-lock",
      "purpose": "Community FD-001 source-authority lock — tenant/RBAC-packaged community service boundary, normalized community modes, retired network/shorts successor routing, separate messenger/mail/workflow/ops-dashboard boundaries, and explicit Plan/Spec + RED gates before implementation fanout; no production or GA claim.",
      "source_manifest_index": "/specs/microservices/manifests-index.json#microservices[name=community]",
      "absorbs_retired": [
        "network"
      ],
      "target_path_after_md_retirement": "same"
    },
    "prd_community_anonymous": {
      "current_path": "/specs/microservices/anonymous.json",
      "kind": "prd",
      "migration_phase": "complete",
      "path_correction_2026_05_20": {
        "previous_path": "retired specs/products communications/anonymous JSON",
        "reason": "specs/products is retired in this checkout; live machine-readable surface is under specs/microservices."
      },
      "purpose": "Community anonymous mode PRD — machine-readable community surface with context isolation, Workflow/Ontology/Intelligence contracts, regional packs, UX components, risks, goals, and deterministic validation hooks.",
      "target_path_after_md_retirement": "same"
    },
    "prd_community_network_retired": {
      "current_path": "/specs/microservices/community.json",
      "kind": "prd",
      "migration_phase": "retired-by-wave-15k-network-into-community-merge",
      "retired_previous_path": "/microservices/community/PRD.md",
      "path_correction_2026_05_20": {
        "previous_path": "legacy network JSON projection",
        "reason": "Wave 15K retires network as standalone product and redirects its professional content into community."
      },
      "purpose": "Retired network PRD pointer. Professional profile, connections, InMail, endorsements, recommendations, jobs, recruiter, skill assessments, pages, and events are now community scope.",
      "retired_on": "2026-05-21",
      "successor_path": "/specs/microservices/community.json",
      "target_path_after_md_retirement": "/specs/microservices/community.json"
    },
    "prd_community_shorts_retired": {
      "current_path": "/specs/microservices/social.json",
      "kind": "prd",
      "migration_phase": "retired-by-wave-15o-shorts-into-social-merge",
      "retired_previous_path": "/microservices/social/PRD.md",
      "path_correction_2026_05_20": {
        "previous_path": "retired specs/products communications/shorts JSON",
        "reason": "specs/products is retired in this checkout; live machine-readable surface is under specs/microservices."
      },
      "purpose": "Retired shorts PRD pointer. Short-form video is absorbed into community/social media flavor scope rather than a product-group wrapper.",
      "retired_on": "2026-05-21",
      "retirement_correction_2026_05_21": {
        "previous_path": "/specs/microservices/shorts.json",
        "reason": "Wave 15O retires shorts as a standalone µservice and absorbs its short-form video capabilities into social per ADR-0334."
      },
      "successor_path": "/specs/microservices/social.json",
      "target_path_after_md_retirement": "/specs/microservices/social.json (short-video flavor section)"
    },
    "prd_community_social": {
      "current_path": "/specs/microservices/social.json",
      "kind": "prd",
      "migration_phase": "complete",
      "path_correction_2026_05_20": {
        "previous_path": "retired specs/products communications/social JSON",
        "reason": "specs/products is retired in this checkout; live machine-readable surface is under specs/microservices."
      },
      "purpose": "Community social mode PRD — machine-readable community surface with context isolation, Workflow/Ontology/Intelligence contracts, regional packs, UX components, risks, goals, and deterministic validation hooks.",
      "target_path_after_md_retirement": "same"
    },
    "prd_crm": {
      "current_path": "/specs/microservices/crm.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "crm microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_foundry": {
      "absorbed_by": "intelligence",
      "absorbing_manifest": "/microservices/intelligence/manifest.json",
      "absorbing_prd_pointer": "prd_intelligence",
      "current_path": "/specs/microservices/intelligence.json",
      "current_path_status": "retirement_marker_only",
      "do_not_treat_as_live_authority": true,
      "kind": "prd",
      "migration_phase": "retired",
      "purpose": "RETIRED — foundry µservice was absorbed into the intelligence µservice per ADR-0335 (Wave 15I) executing ADR-0255 KS#14 two-layer intelligence substrate. AI substrate concerns now live under intelligence; self-modification runs as oyatie.foundry.* Cedar principals per ADR-0247. Cite microservices/intelligence/manifest.json for live AI substrate authority; cite ADR-0335 for the retirement decision; cite microservices/intelligence/RETIRED.md for the redirect marker.",
      "retired_at": "2026-05-21",
      "retired_by_adr": "ADR-0335",
      "retired_by_wave": "15I",
      "retirement_marker": "/microservices/intelligence/RETIRED.md",
      "status": "retired",
      "target_path_after_md_retirement": "/microservices/intelligence/RETIRED.md"
    },
    "prd_global_trade": {
      "current_path": "/specs/microservices/global-trade.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "global-trade microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_hr": {
      "current_path": "/specs/microservices/hr.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "HR microservice PRD — employee lifecycle, org hierarchy, attendance, leave, recruiting/performance shells, sensitive-data policy, and KR/US/EU labor-compliance evidence.",
      "target_path_after_md_retirement": "same"
    },
    "prd_intelligence": {
      "absorbing_manifest": "/microservices/intelligence/manifest.json",
      "current_path": "/specs/microservices/intelligence.json",
      "kind": "prd",
      "migration_phase": "foundry-absorbed-wave-15I",
      "purpose": "Intelligence product PRD — AI substrate authority after Foundry retirement per ADR-0335; covers provider routing, guardrails, eval, attribution, credential resolution, audit tap, assist-draft, and context-aware retrieval for later cloud-service integration.",
      "target_path_after_md_retirement": "same"
    },
    "prd_mail": {
      "current_path": "/specs/microservices/mail.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Mail microservice PRD — personal and professional mail with strict tenant/RBAC separation, retention, legal hold, eDiscovery, migration/coexistence, encrypted search, and Workflow handoff boundaries.",
      "target_path_after_md_retirement": "same"
    },
    "prd_manifest-schema": {
      "current_path": "/specs/microservices/manifest-schema.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "manifest-schema µservice PRD — landed per session 2026-05-17/18.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_manifests-index": {
      "current_path": "/specs/microservices/manifests-index.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "manifests-index µservice PRD — landed per session 2026-05-17/18.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_messenger": {
      "current_path": "/specs/microservices/messenger.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Messenger microservice PRD — personal E2E messaging plus professional tenant-DEK/auditable messaging with realtime delivery and strict personal/professional tenant/RBAC boundaries.",
      "target_path_after_md_retirement": "same"
    },
    "prd_network": {
      "current_path": "/specs/microservices/community.json",
      "kind": "prd",
      "migration_phase": "retired-by-wave-15k-network-into-community-merge",
      "retired_previous_path": "/microservices/community/PRD.md",
      "purpose": "Retired network PRD pointer. LinkedIn-class jobs/profile/recruiter content is absorbed into community; LinkedIn-style engagement feed remains forbidden.",
      "retired_on": "2026-05-21",
      "successor_path": "/specs/microservices/community.json",
      "target_path_after_md_retirement": "/specs/microservices/community.json"
    },
    "prd_ontology": {
      "current_path": "/specs/microservices/ontology.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Ontology + Knowledge Graph merged product PRD. Typed-entity schema layer (Bominal-ADR-0106) + 3-layer KG runtime (semantic/kinetic/dynamic). Ontology+Workflow jointly form ecosystem-as-a-product backbone.",
      "target_path_after_md_retirement": "same"
    },
    "prd_payroll": {
      "current_path": "/specs/microservices/payroll.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Payroll microservice PRD — KR-first payroll close, payee classes, statutory taxes/4대보험, year-end settlement, payroll-to-accounting bridge, and group close evidence.",
      "target_path_after_md_retirement": "same"
    },
    "prd_plant_maintenance": {
      "current_path": "/specs/microservices/plant-maintenance.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "plant-maintenance microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_procurement": {
      "current_path": "/specs/microservices/procurement.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "procurement microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_production_planning": {
      "current_path": "/specs/microservices/production-planning.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "production-planning microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_quality_management": {
      "current_path": "/specs/microservices/quality-management.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "quality-management microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_real_estate": {
      "current_path": "/specs/microservices/real-estate.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "real-estate microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_shorts": {
      "current_path": "/specs/microservices/social.json",
      "kind": "prd",
      "migration_phase": "retired-by-wave-15o-shorts-into-social-merge",
      "retired_previous_path": "/microservices/social/PRD.md",
      "purpose": "Retired shorts µservice PRD pointer. TikTok-/Reels-/YouTube-Shorts-class short-form video is absorbed into social as the TikTok-style media flavor per ADR-0334 (Wave 15O). Industry precedent (Instagram Reels in Instagram; YouTube Shorts in YouTube; X video in X) places short-video inside the social product, not in a sibling service.",
      "retired_on": "2026-05-21",
      "successor_path": "/specs/microservices/social.json",
      "target_path_after_md_retirement": "/specs/microservices/social.json (short-video flavor section)"
    },
    "prd_social": {
      "current_path": "/specs/microservices/social.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Social µservice PRD — Twitter / X / Bluesky-class general social platform. Per ADR-0126/0135 dissolution.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_supply_chain_planning": {
      "current_path": "/specs/microservices/supply-chain-planning.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "supply-chain-planning microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_tenant_rbac": {
      "current_path": "/specs/microservices/tenant-rbac.json",
      "kind": "prd",
      "migration_phase": "deprecated",
      "purpose": "Tenant/RBAC microservice PRD and packaging-control authority. It replaces former grouping wrappers; HR, payroll, accounting, messenger, community, and mail remain concrete flat services packaged later by tenant entitlements and RBAC scopes.",
      "retirement_ref": "ADR-0362",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_tenant_rbac_packaging": {
      "current_path": "/specs/tenant-rbac-packaging.json",
      "kind": "prd",
      "migration_phase": "deprecated",
      "purpose": "Tenant/RBAC packaging policy for flat microservices. No former product-grouping wrapper, suite, platform, module, bundle, or vertical product grouping is active; packaging is computed from tenant entitlements, RBAC scopes, regulatory packs, residency, and feature flags over concrete services such as messenger, community, and mail.",
      "retirement_ref": "ADR-0362",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_treasury": {
      "current_path": "/specs/microservices/treasury.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "treasury microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_warehouse": {
      "current_path": "/specs/microservices/warehouse.json",
      "kind": "prd",
      "migration_phase": "wave-3-consolidation",
      "purpose": "warehouse microservice PRD foundation consolidated in wave-3 (flat single-concern per ADR-0131/0132); metadata-only scope with explicit non-claims.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_workflow": {
      "current_path": "/specs/microservices/workflow.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Workflow product PRD — engine half of the ecosystem-backbone pair with prd_ontology. State-machine + DAG hybrid orchestration substrate per Bominal-ADR-0148. Canonical JSON spec via contracts/workflow_spec.v1.json.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "prd_workflow_studio": {
      "current_path": "/specs/microservices/workflow-studio.json",
      "kind": "prd",
      "migration_phase": "complete",
      "purpose": "Workflow Studio product PRD — visual editor + DSL backbone; n8n-class first hero product per feedback_workflow_studio_scope. Pairs with prd_workflow on the same canonical spec.",
      "target_path_after_md_retirement": "same"
    },
    "pre_pr_multispectrum_checklist": {
      "current_path": "/templates/checklists/pre-pr-multispectrum.json",
      "human_companion_path_retired": "/templates/checklists/pre-pr-multispectrum.md (RETIRED 2026-05-14 — JSON template is the canonical form; markdown was duplicative human gateway)",
      "kind": "template",
      "migration_phase": "complete",
      "purpose": "Machine-readable pre-PR gate template. Every changeset copies + fills evidence per facet; the seam lane refuses missing/incomplete sections. Iterative fix-and-review loop in /specs/iterative-fix-loop.json."
    },
    "reusable_building_blocks_registry": {
      "current_path": "/registry/reusable-building-blocks-registry.json",
      "kind": "registry",
      "migration_phase": "complete (15 baseline blocks)",
      "purpose": "DRY enforcement. Single canonical home per block + consumer_refs + consumer_selectors.",
      "target_path_after_md_retirement": "same"
    },
    "root_hub_pointers": {
      "current_path": "/specs/root-hub-pointers.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "This file itself — canonical machine-readable pointer registry. Self-reference present so every spec has a discoverable entry_point row (per OP-11 no-stubs-no-defer).",
      "target_path_after_md_retirement": "same"
    },
    "score_cards": {
      "current_path": "/specs/score-cards.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Concrete deterministic score-card inventory consumed by the loop-recovery-patterns gate. Legacy `oya` mirror invocations are migration evidence only until ported to cloud-ci/Rust gate contexts. Companion to agent_durable_goal.score_cards.",
      "target_path_after_md_retirement": "same (machine-readable from inception)"
    },
    "spec_agentic_slo_gated_promotion": {
      "current_path": "/specs/agentic-slo-gated-promotion.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Machine-readable contract for the agentic SLO-gated promotion pipeline. Consumed by the oya-observability-slo-engine crate family and cloud-ci/governance release-promotion contexts; any legacy CI promotion job reference is bridge/provenance only until replaced."
    },
    "spec_api_surface_separation": {
      "current_path": "/specs/api-surface-separation.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Internal vs external API surface separation"
    },
    "spec_audit_event_class_registry": {
      "current_path": "/specs/audit-event-class-registry.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical JSON Schema 2020-12 contract and known-class registry for audit-event classes governed by ADR-0263. The registry binds emitted event classes from ADR-0297 abuse defence, ADR-0313 conglomerate tenant hierarchy, and ADR-0319 front/middle/back-office information barrier into one machine-readable surface. Every new doctrine ADR that introduces audit events must extend this registry and keep ADR-0263 reverse references current."
    },
    "spec_brownout_degradation_signal": {
      "current_path": "/specs/brownout-degradation-signal.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Brown-out degradation signal canonical schema"
    },
    "spec_capability_tier_schema": {
      "current_path": "/specs/capability-tier-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical JSON Schema 2020-12 contract for capability-tier definitions. Binds ADR-0316 capability-tier-over-product-fragmentation doctrine, ADR-0243 Cedar-as-universal-gate policy coverage, ADR-0244 tenant scoping, ADR-0251 compliance pack overlays, ADR-0257 ontology revision pins, and ADR-0263 audit-event evidence into one machine-readable tier definition surface."
    },
    "spec_cedar_fragment_schema": {
      "current_path": "/specs/cedar-fragment-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Cedar Fragment Frontmatter Schema"
    },
    "spec_cedar_policy_schema": {
      "current_path": "/specs/cedar-policy-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical JSON Schema 2020-12 contract for Cedar policy schema manifests. Binds ADR-0243 Cedar-as-universal-gate and ADR-0294 Cedar fragment soak/anomaly rollback doctrine into a machine-readable schema for entity types, action types, bounded context, private attributes, permit sets, fragment bindings, fixtures, validation, soak controls, fallback policy, audit mapping, and promotion gates."
    },
    "spec_chaos_engineering_substrate_canonical": {
      "current_path": "/specs/chaos-engineering-substrate-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Operationalizes ADR-0165. Declares Chaos Mesh 2.x adoption, per-µservice catalog requirements, nightly drill cadence, SLO-gate semantics, and release-blocker rules."
    },
    "spec_compliance_pack_schema": {
      "current_path": "/specs/compliance-pack-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Compliance Pack Bundle Schema"
    },
    "spec_pci_dss_level_1_readiness_plan": {
      "current_path": "/specs/pci-dss-level-1-readiness-plan.json",
      "kind": "spec",
      "migration_phase": "regid-003-planning-spec",
      "purpose": "Planning-only PCI DSS Level 1 readiness and regulated-data control-pack contract for CDE scope, tokenization, PAN/CVV/track-data prohibitions, QSA/ROC/AOC evidence, ASV scans, segmentation, KMS/PDP/audit bindings, retention, incident response, and tenant pack activation gates."
    },
    "spec_regulatory_identity_source_of_truth": {
      "current_path": "/specs/regulatory-identity-source-of-truth.json",
      "kind": "spec",
      "migration_phase": "regid-001-planning-source-of-truth",
      "purpose": "REGID-001 source-index and machine-readable control backlog for KR/global identity, KYC, passkey, compliance, PCI/NIST/EU/US/PQC floors. Planning/spec inventory only: no product-code mutation, no CLI surface, no oya/cloud reorg debt."
    },
    "spec_regulatory_legacy_gate_scanner_normalization_table": {
      "current_path": "/specs/regulatory-legacy-gate-scanner-normalization-table.json",
      "kind": "spec",
      "migration_phase": "regnorm-001-planning-spec-only",
      "purpose": "REGNORM-001 planning-only normalization table mapping legacy oya gate/verify commands, scanner outputs, hosted dashboards, CI runner bridges, SBOM/signing attestations, advisory feeds, and admission examples to product adapters, typed evidence records, cloud-ci Rust gate packets, and dashboard/trust-portal query surfaces without making CLI/scanner output the product path."
    },
    "spec_compliance_security_radar_cadence_contract": {
      "current_path": "/specs/compliance-security-radar-cadence-contract.json",
      "kind": "spec",
      "migration_phase": "sec-reg-radar-cadence-planning-procedure-contract",
      "purpose": "Planning/procedure contract for recurring compliance/security radar: source classes, owners, freshness cadences, stale/blocker semantics, claim ceilings, dedupe card targets, and Kanban output rules. No certification/readiness/security-posture or scheduler claim."
    },
    "spec_csi_storage_class_canonical": {
      "current_path": "/specs/csi-storage-class-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Declares the canonical StorageClass naming scheme (oya-<kind>-<tier>), the per-pack CSI driver matrix, the CSI driver requirements, and the workload µservice contract."
    },
    "spec_design_spec_maturity_claims": {
      "current_path": "/specs/design-spec-maturity-claims.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Define the machine-enforced evidence bar for a bounded architecture/platform/system design maturity claim."
    },
    "spec_dr_business_continuity": {
      "current_path": "/specs/dr-business-continuity.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "DR + business-continuity canonical schema"
    },
    "spec_feature_flag_substrate_canonical": {
      "current_path": "/specs/feature-flag-substrate-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Operationalizes ADR-0159. Declares OpenFeature compliance, evaluation context shape, Cedar-predicate-based targeting, lifecycle gating, audit-chain emission policy, per-cell replication semantics."
    },
    "spec_finops_cost_attribution": {
      "current_path": "/specs/finops-cost-attribution.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "FinOps cost-attribution canonical schema"
    },
    "spec_hyperscaler_architecture_invariants": {
      "current_path": "/specs/hyperscaler-architecture-invariants.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical hyperscaler systems + cloud architecture invariants for product PRD hyperscaler_bar evidence. This is the source of truth for hyperscaler-grade architecture requirements; product-level blocking remains advisory until the PRD validator and branch-protected lane land."
    },
    "spec_industry_best_practice_conformance": {
      "current_path": "/specs/industry-best-practice-conformance.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Machine-readable findings + audit invariants for the continuous 6-axis industry-best-practice + hyperscaler-grade conformance program. Consumed by oya-governance-industry-best-practice-conformance CI lane."
    },
    "spec_microservice_migration_tooling": {
      "current_path": "/specs/microservice-migration-tooling.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Legacy migration-tooling spec for prior legacy migration-subcommand planning. Active platform-readiness target is `{oya,cloud}/<service>` plus `libs/`; migration/enforcement must be cloud-ci/Rust gate packets or deletion-tagged bridge evidence, not new `oya` CLI authority. Historical `microservices/<ms>` examples are provenance until P0.6 verified removal."
    },
    "spec_multi_region_disposition_canonical": {
      "current_path": "/specs/multi-region-disposition-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Declares the canonical per-µservice multi-region disposition enum, the sovereign-tenant region-pin overlay contract, the global control-plane shape, and the per-pack default topology matrix. Operationalizes ADR-0158."
    },
    "spec_ontology_projection_schema": {
      "current_path": "/specs/ontology-projection-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical JSON Schema 2020-12 contract for ontology projection manifests. Binds ADR-0316 capability-tier ontology projection requirements and ADR-0317 role-based projection unified UX shell requirements into a machine-readable surface for object and relation refs, actions, functions, schema pins, field visibility, computed properties, jurisdiction filters, pack filters, audit redaction, export policy, search indexing, lineage, service ownership guards, role projections, cache partitioning, observability, and promotion gates."
    },
    "spec_pack_overlay_schema": {
      "current_path": "/specs/pack-overlay-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical JSON Schema 2020-12 contract for compliance pack overlays. Binds ADR-0251 compliance-pack overlay execution and ADR-0316 capability-tier compliance overlay requirements into a machine-readable surface for jurisdiction scope, tenant scope, data classes, regulated decisions, Cedar bindings, ontology projection constraints, workflow constraints, audit obligations, evidence requirements, precedence, lifecycle, ownership, and promotion gates."
    },
    "spec_per_microservice_flat_layout": {
      "current_path": "/specs/per-microservice-flat-layout.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Machine-readable contract for ADR-0131 flat colocation as amended by ADR-0512/platform-readiness pure split. Target service roots are `{oya,cloud}/<service>` with shared code in `libs/` and pack roots governed by ADR-0010/0064; legacy `microservices/` references are migration inputs until verified removal."
    },
    "spec_per_tenant_audit_log_slicing_canonical": {
      "current_path": "/specs/per-tenant-audit-log-slicing-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Operationalizes ADR-0162. Declares the per-tenant audit-chain partition scheme, sovereign-tenant dedicated-shard contract, sealing cadence, and per-tenant retrieval API."
    },
    "spec_platform_architecture": {
      "current_path": "/specs/platform-architecture.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Consolidated machine-readable source-of-truth for the oyatie platform architecture, derived from the 14 foundational keystone ADRs (ADR-0242 through ADR-0255). Every CI lane, manifest validator, scaffolder, and downstream spec resolves architectural questions against this document. Authoritative for: tenancy doctrine, policy gate doctrine, substrate-vs-product layering, cellular topology, compliance packs, deployment models, intelligence substrate, time + consistency primitives, network..."
    },
    "spec_saga_shape": {
      "current_path": "/specs/saga-shape.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Oyatie saga shape"
    },
    "spec_schema_registry_canonical": {
      "current_path": "/specs/schema-registry-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Operationalizes ADR-0166. Declares Apicurio Registry 3.x adoption, schema-kind coverage, subject naming convention, compatibility-level defaults, and backward-compat CI lane."
    },
    "spec_sovereign_cloud_air_gapped_canonical": {
      "current_path": "/specs/sovereign-cloud-air-gapped-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Operationalizes ADR-0164. Declares the per-pack air_gap overlay, on-prem dependency substitution matrix, regulator binding, and CI enforcement."
    },
    "spec_sovereign_cloud_overlays": {
      "current_path": "/specs/sovereign-cloud-overlays.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Sovereign cloud overlays canonical schema"
    },
    "spec_tenant_environment_tiers_canonical": {
      "current_path": "/specs/tenant-environment-tiers-canonical.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Operationalizes ADR-0163. Declares the per-tenant test / staging / prod tier model, API-key prefix scheme, outbound-side-effect modes, destructive-op acknowledgment, and Cedar policy fragments."
    },
    "spec_tenant_lifecycle": {
      "current_path": "/specs/tenant-lifecycle.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Tenant lifecycle canonical schema"
    },
    "spec_tenant_model": {
      "current_path": "/specs/tenant-model.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Canonical JSON Schema 2020-12 contract for Oyatie tenant rows. This schema binds ADR-0244 tenant-as-universal-scoping-primitive, ADR-0311 dual-tenant personal/work boundary, and ADR-0313 conglomerate tenant hierarchy into one machine-readable tenant model that downstream Cedar, audit-chain, FinOps, residency, compliance-pack, and ontology-projection surfaces can cite."
    },
    "spec_throttling_tiers": {
      "current_path": "/specs/throttling-tiers.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Layered throttling tiers canonical schema"
    },
    "spec_workspace_hygiene": {
      "current_path": "/specs/workspace-hygiene.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-05-20",
      "purpose": "Inventory-by-default hygiene pass for agent-created spillover across temp space, user-home coordination files, the main checkout, build artifacts, and Oyatie worktrees before pipeline closeout."
    },
    "spec_agent_durable_goal": {
      "current_path": "/specs/agent-durable-goal.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Persistent across-session agent goal contract: operating principles, lifecycle phases, iteration-loop semantics, TDD contract, CI/CD contract, confusion-management protocol, and verification-before-completion checklist."
    },
    "spec_audit_event_schema": {
      "current_path": "/specs/audit-event-schema.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Canonical audit-row envelope schema, including the ADR-0344 sustainability and FinOps tuple extension."
    },
    "spec_cloud_control_plane_operation_contract": {
      "current_path": "/specs/cloud-control-plane-operation-contract.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Metadata-level cloud control-plane contract for resource registry entries, long-running operations, idempotent retries, cancellation, compensation, and per-resource state transitions."
    },
    "spec_cloud_enforceability_facets": {
      "current_path": "/specs/cloud-enforceability-facets.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Cloud enforceability facets for resource contracts: authorization, tenancy, audit, metering, billing, quota, and fail-closed cost admission metadata."
    },
    "spec_cloud_hyperscaler_parity_taxonomy": {
      "current_path": "/specs/cloud-hyperscaler-parity-taxonomy.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Source-backed hyperscaler parity taxonomy and honest nonclaim matrix for AWS, Google Cloud, Azure, OCI, Kubernetes, and CNCF comparison claims."
    },
    "spec_cloud_observability_slo_evidence_contract": {
      "current_path": "/specs/cloud-observability-slo-evidence-contract.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Cloud observability and SLO evidence contract for resource parity gates, OpenTelemetry metadata, OpenSLO authoring, burn-rate evidence windows, and event receipts."
    },
    "spec_cloud_production_quality_kit_evidence_backlog": {
      "current_path": "/specs/cloud-production-quality-kit-evidence-backlog.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Machine-checkable backlog and evidence-record schema for cloud production-quality kits without claiming dogfood execution or production maturity."
    },
    "spec_cloud_resource_contract_parity_catalog": {
      "current_path": "/specs/cloud-resource-contract-parity-catalog.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Resource-contract parity catalog companion for cloud backend categories and future parity evidence checks."
    },
    "spec_compliance_pack_floors": {
      "current_path": "/specs/compliance-pack-floors.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Machine-readable per-compliance-pack DR floor table consumed by pack-activation gates and auditor dashboard generation."
    },
    "spec_finops_dimensional_model": {
      "current_path": "/specs/finops-dimensional-model.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "FinOps and sustainability dimensional rollup contract for per-axis aggregation, freshness, retention, and regulator-export evidence formats."
    },
    "spec_generated_artifact_control_plane_schema": {
      "current_path": "/specs/generated-artifact-control-plane.schema.json",
      "kind": "schema",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Portable generated-artifact control-plane manifest schema consumed by hermetic generated-output ownership and materialization gates."
    },
    "spec_http_stack_policy": {
      "current_path": "/specs/http-stack-policy.json",
      "kind": "policy",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Authoritative HTTP-framework selection policy: hyper preferred, axum sanctioned with justification, all other HTTP stacks forbidden."
    },
    "spec_iac_module_library": {
      "current_path": "/specs/iac-module-library.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Canonical OpenTofu IaC module-library primitive enumeration for Oyatie deployment contexts and shared module discovery."
    },
    "spec_language_discipline_registry": {
      "current_path": "/specs/language-discipline-registry.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Local/static Phase-0 language-discipline allowlist registry with GOOD/BAD fixture validation and cloud-check backlog inventory."
    },
    "spec_legal_ip_domain_taxonomy": {
      "current_path": "/specs/legal-ip-domain-taxonomy.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Machine-readable domain-purpose taxonomy for FD-001 legal/IP and operator-surface planning, with explicit evidence gaps for DNS, trademark, renewal, and production routing claims."
    },
    "spec_oss_stewardship_registry": {
      "current_path": "/specs/oss-stewardship-registry.json",
      "kind": "registry",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Canonical OSS stewardship registry per ADR-0345, enumerating direct upstream OSS dependencies, stewardship class, owner team, CVE SLA, license, source URL, and ADR provenance."
    },
    "spec_vulnerability_intelligence_sbom_vex_pipeline": {
      "current_path": "/specs/vulnerability-intelligence-sbom-vex-pipeline.json",
      "kind": "spec",
      "migration_phase": "regsec-001-planning-contract",
      "purpose": "Planning-only cloud-native vulnerability intelligence and SBOM/VEX pipeline contract covering CVE/NVD/OSV/RustSec/GitHub/vendor advisory ingestion, dual-format SBOMs, VEX, KEV/EPSS/CVSS/SSVC prioritization, remediation SLAs, expiring exceptions, audit evidence, and deployment/admission blocking without scanner CLI authority."
    },
    "spec_platform_vertical_status": {
      "current_path": "/specs/platform-vertical-status.json",
      "kind": "registry",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Seed registry for platform vertical G-story status rows consumed by milestone views without hardcoded status pills or numeric progress bars."
    },
    "spec_repo_hygiene_automation": {
      "current_path": "/specs/repo-hygiene-automation.json",
      "kind": "spec",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Automation contract for git, branch, repository, disk/workspace, Kubernetes workload, and documentation-sprawl hygiene before native SCM/CI/CD cutover."
    },
    "spec_structural_blockers": {
      "current_path": "/specs/structural-blockers.json",
      "kind": "registry",
      "migration_phase": "root-hub-covered-2026-06-30-gov-003-derived-top-level-specs",
      "purpose": "Seed registry for structural blockers with machine-derivable signals and auto-clear conditions for milestone-view rendering."
    },
    "stop_conditions": {
      "current_path": "/specs/stop-conditions.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "SC-01..SC-09 stop conditions for autonomous master-plan loop.",
      "target_path_after_md_retirement": "same"
    },
    "test_set_registry": {
      "current_path": "/registry/test-set-registry.json",
      "kind": "registry",
      "migration_phase": "complete",
      "purpose": "Registry-driven polyglot test commands and evidence outputs by language/surface.",
      "target_path_after_md_retirement": "same"
    },
    "test_standard": {
      "current_path": "/specs/test-standard.json",
      "kind": "spec",
      "migration_phase": "complete",
      "purpose": "Machine-readable unit/integration/e2e/property/fuzz test standard enforced by GitOps VCS admission and CI/CD gates.",
      "target_path_after_md_retirement": "same"
    },
    "toolchain_tenant_isolation_fixtures": {
      "current_path": "/specs/toolchain-tenant-isolation-fixtures.json",
      "kind": "spec",
      "migration_phase": "phase0_p0_0_fixture_contract_not_live_enforcement",
      "purpose": "T0.0 tenant pipeline isolation fixture contract for tenant=oyatie-internal and customer tenants across identity, secrets, runners/workspaces, caches, artifacts, logs/evidence, release ledgers, deploy targets, status callbacks, and audit events. This is fixture evidence only until required cloud-ci context runs it.",
      "target_path_after_md_retirement": "same (machine-readable T0.0 tenant isolation fixture contract)"
    },
    "work_area_content_hash_contract": {
      "current_path": "/specs/work-area-content-hash-contract.json",
      "kind": "spec",
      "migration_phase": "w1-interface-lock",
      "purpose": "W1 machine-readable contract for the single content-addressed work-area hash from ADR-0517/ADR-0520: canonical inputs from WorkAreaTree/scm-facts and byte-identical outputs for SCM change id, buck2/RBE key, and CD artifact hash. Metadata-only; no SCM/RBE/CD runtime claim.",
      "target_path_after_md_retirement": "same (machine-readable W1 hash/id contract)"
    },
    "wave_15_zf_doctrine_propagation_adr_0346_0349": {
      "current_path": "/specs/master-plan-sequencing.json#realignment_wave_sequence.waves_15_plus.sub_wave_landings.15-ZF-doctrine-propagation-adr-0346-0349",
      "kind": "batch-control",
      "migration_phase": "wave-15-zf-master-plan-propagated",
      "owned_master_plan_paths": [
        "/specs/root-hub-pointers.json",
        "/specs/master-plan-sequencing.json"
      ],
      "purpose": "Wave 15-ZF batch-control pointer for propagating ADR-0346, ADR-0347, ADR-0348, and ADR-0349 doctrine across the corpus while ZF-4 owns only the root-hub and master-plan sequencing amendments.",
      "target_path_after_md_retirement": "same",
      "retired_canonical_brief_path": ".omc/state/wave-15-zf-canonical-brief.md",
      "canonical_brief_status": "retired-from-tracked-tree-local-only; use specs/master-plan-sequencing.json plus git history for provenance"
    },
    "phase0_ci_enforcement_result_schema": {
      "current_path": "/specs/phase0-ci-enforcement-result-schema.json",
      "kind": "spec",
      "migration_phase": "phase0_p0_0_result_schema_not_live_authority",
      "purpose": "Structured result bundle schema for P0.0 cloud-ci/oya-ci fixture runs; evidence input only until a trusted required context is live.",
      "target_path_after_md_retirement": "same (machine-readable T0.0 result schema)"
    }
  },
  "agent_quick_start_protocol": {
    "step_1_read_authority": "Read entry_points.decision_principles + .agent_operating_contract + .master_plan_sequencing + .markdown_retirement_policy + .gitops_vcs_replacement + .multispectrum_review.",
    "step_2_context_recall": "Prefer intelligence-native context once available (replaces prior Foundry framing per ADR-0335 Wave 15I); legacy icm recall-context is compatibility/provenance only and is never promotion authority.",
    "step_3_oya_vcs_state_transition": "Canonical path is plain `git` + protected PR against dev. Merge readiness is reviewer APPROVE plus the single protected `oya-ci-required` context produced by the cloud-ci gate apps; legacy CI/`oya gate` output is bridge/local evidence only and never merge authority. The bespoke Oya VCS claim → work → verify → done → promote ratchet is RETIRED (provenance/historical only per ADR-0363); legacy grit/icm/rtk/vox/omx/omc surfaces are likewise read/provenance only.",
    "step_4_active_artifact_contract": "Every new artifact under applicable_paths_glob conforms to active-artifact-contract v3.0.0 + registers a row in artifact-capabilities-registry.",
    "step_5_multispectrum_evidence": "Every PR carries multispectrum evidence per /specs/multispectrum-review.json; missing or incomplete evidence blocks the governance gate. Destination enforcement is the cloud-ci Rust gate packet behind the single protected `oya-ci-required` context; legacy `oya gate run-all`/CI output is bridge/local evidence only and must not be extended as new authority.",
    "step_6_gitops_vcs_replacement": "Before broad multi-agent fan-out, treat /specs/gitops-vcs-replacement.json as provenance/historical input only: the bespoke Oya VCS claim coverage, ChangeBundle closeout, controller rebase, and retired ratchet mechanisms are superseded by ADR-0363/ADR-0513/ADR-0515. Live merge readiness is plain git + protected PR against dev + reviewer APPROVE + the single protected `oya-ci-required` context. Legacy CI/`oya gate` output is bridge evidence only. Tide/prow-shaped admission language is historical unless reintroduced by a new accepted authority; cloud-ci gate apps own the required context."
  }
}
exec
/bin/zsh -lc 'wc -l .github/workflows/oya-ci-required.yml .github/workflows/docs-graph-drift.yml infra/ci/install-buck2.sh infra/ci/buck2-affected-gate.sh infra/ci/materialize-cloud-ci-generated-faces.sh' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     450 .github/workflows/oya-ci-required.yml
      85 .github/workflows/docs-graph-drift.yml
      34 infra/ci/install-buck2.sh
     137 infra/ci/buck2-affected-gate.sh
      28 infra/ci/materialize-cloud-ci-generated-faces.sh
     734 total
exec
/bin/zsh -lc "nl -ba .github/workflows/oya-ci-required.yml | sed -n '171,340p'" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "nl -ba .github/workflows/oya-ci-required.yml | sed -n '1,170p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   171	      - name: Materialize faces then assert byte-parity
   172	        run: |
   173	          infra/ci/materialize-cloud-ci-generated-faces.sh .
   174	          cargo test --locked --no-fail-fast -p registry-drift -- --test-threads=1
   175	
   176	  # ── cloud-ci-firewall: the baseline ratchet (blocks only NEW debt) + the gate-registration
   177	  #    meta-test (no in-tree gate may go unregistered in this workflow). This is the surface-all
   178	  #    runner; per the runbook the existing firewall runner suffices — no separate aggregator bin
   179	  #    is required for PRE-work.
   180	  gate-cloud-ci-firewall:
   181	    name: cloud-ci-firewall (baseline ratchet + gate-registration meta-test)
   182	    runs-on: ubuntu-latest
   183	    steps:
   184	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   185	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   186	        with:
   187	          persist-credentials: false
   188	          # Full history: the accounting producer derives last_touch_commit via
   189	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
   190	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
   191	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
   192	          # and identical to a full local clone.
   193	          fetch-depth: 0
   194	      - name: Install buck2 (digest-pinned prebuilt release)
   195	        run: infra/ci/install-buck2.sh
   196	      - name: Materialize cloud-ci generated faces
   197	        run: infra/ci/materialize-cloud-ci-generated-faces.sh .
   198	      - name: cargo test cloud-ci-firewall
   199	        run: cargo test --locked --no-fail-fast -p oya-cloud-ci-firewall-app -- --test-threads=1
   200	
   201	  # ── GENERATED OUTPUT DIFF POLICY. Generated files may be deleted to retire a tracked output,
   202	  #    but PRs must not add/modify generated outputs as merge surfaces. Classification comes from
   203	  #    registry/generated-artifact-control-plane.json `generated_path_rules` so adopters can encode
   204	  #    their generated-output conventions once; .gitignore is preventive hygiene, not policy
   205	  #    authority. The candidate workspace is regenerated by cloud-ci before gates consume it.
   206	  generated-output-diff-policy:
   207	    name: generated-output-diff-policy (no generated merge surfaces)
   208	    runs-on: ubuntu-latest
   209	    steps:
   210	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   211	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   212	        with:
   213	          persist-credentials: false
   214	          fetch-depth: 0
   215	      - name: Install buck2 (digest-pinned prebuilt release)
   216	        run: infra/ci/install-buck2.sh
   217	      - name: Pre-provision pinned Rust toolchain for Buck2 policy binary
   218	        run: |
   219	          set -euo pipefail
   220	          rustup show active-toolchain || rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy
   221	          rustup component add rustfmt clippy --toolchain 1.95.0
   222	          rustc --version
   223	          cargo --version
   224	      - name: Reject non-deletion generated output edits
   225	        env:
   226	          EVENT_NAME: ${{ github.event_name }}
   227	          BASE_REF: ${{ github.base_ref || 'dev' }}
   228	        run: |
   229	          set -euo pipefail
   230	          if [ "${EVENT_NAME}" = "push" ]; then
   231	            echo "generated-output-diff-policy: push event; presubmit diff policy not applicable."
   232	            exit 0
   233	          fi
   234	          git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}"
   235	          policy_bin="$(buck2 build //cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app:oya-cloud-ci-generated-output-diff-policy --show-output | awk '{print $2}')"
   236	          git diff --name-status "origin/${BASE_REF}"...HEAD \
   237	            | "${policy_bin}" --manifest registry/generated-artifact-control-plane.json
   238	
   239	  # ── HERMETIC BUCK2 LANE (OYA-CI-HERMETIC-EXECUTION-DESIGN §3 + Stage P1/P2). Runs the SAME
   240	  #    gate logic as the cargo lanes above, but through buck2: `buck2 build` compiles every
   241	  #    target (the env!CARGO eradication) and `buck2 test` runs the gate rust_tests fully
   242	  #    hermetically (no ambient git in any action — the producer reads the materialized scm-facts
   243	  #    face; the scm-facts emitter is the single out-of-graph boundary, run in the
   244	  #    materialization step BELOW, never inside a cacheable action). Scoped by the
   245	  #    affected-set driver (`infra/ci/buck2-affected-gate.sh`: uquery owner -> rdeps closure,
   246	  #    FAILS CLOSED) for speed. RBE/NativeLink is staged LAST (D4) and NOT required for
   247	  #    hermeticity — local-on-runner execution via the wired `noop_test_toolchain` is sufficient
   248	  #    here. This lane runs ALONGSIDE the cargo lanes (both feed the fan-in); the cargo->buck2
   249	  #    required-context content swap is a separate founder-paired step.
   250	  buck2:
   251	    name: buck2 (hermetic build + affected gate tests)
   252	    runs-on: ubuntu-latest
   253	    steps:
   254	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   255	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   256	        with:
   257	          persist-credentials: false
   258	          # Full history: the materialization step below runs the emitter, which derives
   259	          # last_touch via `git log --name-only`; a shallow checkout collapses it (PM1).
   260	          fetch-depth: 0
   261	      # buck2 is the BUILD TOOL (like cargo/rustc), installed as a prebuilt release.
   262	      # The adapter edge is immutable at CI time: release tag selects the asset, SHA-256 pins
   263	      # the bytes, and CI verifies the digest before decompression/execution. Bump together.
   264	      - name: Install buck2 (digest-pinned prebuilt release)
   265	        run: infra/ci/install-buck2.sh
   266	      # Pre-provision the pinned rust toolchain ONCE, serially, before the buck2 build.
   267	      # The buck2 rust toolchain (toolchains/BUCK: system_rust_toolchain via the rustup shim)
   268	      # resolves rustc/cargo/clippy per-compile-action, and buck2 runs those actions in
   269	      # PARALLEL. On a cold runner each action's first shim call triggers rustup to install the
   270	      # rust-toolchain.toml channel (1.95.0 + rustfmt,clippy) concurrently — the racing rustup
   271	      # processes collide on the shared `~/.rustup/downloads/*.partial` files and fail with
   272	      # `rustup::utils::rename ... No such file or directory (os error 2)` (a different component
   273	      # each run: clippy, then cargo — proving a concurrency race, not a config defect). rustup
   274	      # is not concurrency-safe. Installing the toolchain once here makes it ambient so the
   275	      # parallel actions find it already present (no download). The cargo lanes never hit this
   276	      # because cargo provisions the toolchain in a single up-front invocation.
   277	      - name: Pre-provision pinned rust toolchain (serialize rustup before parallel buck2)
   278	        run: |
   279	          set -euo pipefail
   280	          rustup show active-toolchain || rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy
   281	          rustup component add rustfmt clippy --toolchain 1.95.0
   282	          rustc --version
   283	          cargo --version
   284	      # Warm buck-out + the buck2 daemon cache across runs so ephemeral runners do not
   285	      # cold-bootstrap the toolchain (design §3.1 / ADR-0515 D4).
   286	      #
   287	      # Cache key is STABLE per dependency-set (.buckconfig + toolchains/BUCK + Cargo.lock),
   288	      # NOT per-commit. The previous `-${{ github.sha }}` suffix made the primary key unique
   289	      # every commit, so actions/cache SAVED a fresh full buck-out (multi-GB) on EVERY run and
   290	      # never hit the primary key — bloating the 10GB repo cache into constant LRU eviction and
   291	      # exhausting ephemeral-runner disk at the save step (the "No space left on device" failure).
   292	      # A stable key saves once per dependency-set and restores it exactly: deterministic warm
   293	      # start, no per-commit bloat. Changed crates still rebuild (buck2 is content-addressed, so a
   294	      # restored hit is bit-identical to a cold build); only a Cargo.lock/toolchain/.buckconfig
   295	      # change mints a new entry. Interim warm-by-default until the shared content-addressed
   296	      # remote cache (NativeLink/CAS, HANDOFF W3) lands with a cold-canary integrity job proving
   297	      # cold==warm. See friction-ledger buck2-no-shared-cache.
   298	      - name: Cache buck-out
   299	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   300	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   301	        with:
   302	          path: buck-out
   303	          key: buck-out-${{ runner.os }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   304	          restore-keys: |
   305	            buck-out-${{ runner.os }}-
   306	      # Generated-face materialization — the SINGLE out-of-graph git boundary. Re-run the emitter
   307	      # and producer against the checked-out candidate tree, then let buck2 consume those files as
   308	      # declared inputs. We deliberately do NOT byte-compare against committed JSON here: that was
   309	      # the self-referential merge-conflict surface. Byte-parity is checked after materialization.
   310	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   311	        run: infra/ci/materialize-cloud-ci-generated-faces.sh .
   312	      # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication —
   313	      # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests
   314	      # run green, fully hermetic, with verdicts identical to the cargo lanes). This is the
   315	      # refactor's scope and is the binding hermetic check for this stage.
   316	      #
   317	      # NOTE: the affected-set driver (`infra/ci/buck2-affected-gate.sh`, uquery owner -> rdeps
   318	      # closure) is the design's speed mechanism (§3.2) and runs below as a best-effort fast path,
   319	      # but its whole-graph `rdeps(//..., ...)` currently FAILS CLOSED on a PRE-EXISTING stale
   320	      # git-worktree BUCK package (`.claire/worktrees/.../oya-payroll-run-usecase`) committed at
   321	      # base — graph pollution unrelated to this refactor. Until that is cleaned, the binding gate
   322	      # is the scoped `//cloud/cloud-ci/...` build+test above; the affected driver is advisory.
   323	      - name: buck2 build + test (//cloud/cloud-ci/..., hermetic — binding)
   324	        run: |
   325	          set -euo pipefail
   326	          # buck2 test builds its targets before running them, so a standalone
   327	          # `buck2 build` immediately before is redundant — removed (item 4 quick win).
   328	          buck2 test //cloud/cloud-ci/...
   329	      - name: buck2 affected-set driver (advisory speed path)
   330	        continue-on-error: true
   331	        run: |
   332	          set -euo pipefail
   333	          chmod +x infra/ci/buck2-affected-gate.sh
   334	          BUCK2=buck2 infra/ci/buck2-affected-gate.sh "origin/${{ github.base_ref || 'dev' }}"
   335	
   336	  # ── APP-SHELL GENERATED CLIENT LANE. Generated TypeScript clients are intentionally ignored
   337	  #    in git, so clean checkout CI must prove they regenerate from canonical OpenAPI sources
   338	  #    before the single required fan-in can go green. This is a bridge lane until the native
   339	  #    cloud-ci runner has first-class non-Rust toolchain actions; the invariant is still the
   340	  #    productized generated-output policy above, not committed client files.
 succeeded in 0ms:
     1	# oya-ci-required — the SINGLE canonical blocking required status check (ONE CANONICAL CI).
     2	#
     3	# Authority: ADR-0515 (Phase-0 firewall + one-canonical-CI + cloud-native posture; Accepted
     4	# 2026-06-07). Rulings: D-CICD-AUTHORITY, ONE CANONICAL CI, D-SEQUENCE (firewall-first),
     5	# D-DOCTRINE (no false-green / surface-all).
     6	#
     7	# Design (per FIREWALL-GO-LIVE-RUNBOOK.md Part 2 + CICD-DESIGN-PLAN Stage 1A/1B): the gate
     8	# lanes fan OUT (fail-fast:false, surface-all) and fan IN to ONE zero-build job named
     9	# `oya-ci-required` that has NO command set of its own — it is green IFF every constituent
    10	# gate lane is green. Branch protection keys on that one context name. Do NOT register the
    11	# 6 gates individually (rejected by the oya-pr-review HTTP-501 multi-producer-deadlock lesson).
    12	#
    13	# ░░ LIVE ░░
    14	# Go-live executed 2026-06-08 (founder-authorized "fully autonomous through both"): the blocking
    15	# triggers below are paired with making `oya-ci-required` the required status context on `dev`
    16	# branch protection (FIREWALL-GO-LIVE-RUNBOOK Part 2). This is the SINGLE canonical required CI
    17	# check — green IFF every constituent gate lane is green. workflow_dispatch retained for manual
    18	# re-runs / verification.
    19	
    20	name: oya-ci-required
    21	
    22	on:
    23	  workflow_dispatch:
    24	  push:
    25	    branches: [dev]
    26	  pull_request:
    27	    branches: [dev]
    28	  merge_group:
    29	
    30	permissions:
    31	  contents: read
    32	
    33	# Surface-all: every gate lane runs to completion even if a sibling fails. The fan-in job
    34	# collects the join. (Lane-internal fail-fast is killed inside each `cargo test` invocation.)
    35	concurrency:
    36	  group: oya-ci-required-${{ github.ref }}
    37	  cancel-in-progress: false
    38	
    39	jobs:
    40	  # ── Producer regen: materialize the cloud-ci generated faces from the checked-out candidate
    41	  #    tree. Generated JSON is not a contributor merge surface; the CI/controller workspace
    42	  #    regenerates it before gates consume it, then uploads it as evidence.
    43	  producer-regen:
    44	    name: producer-regen (accounting-registry)
    45	    runs-on: ubuntu-latest
    46	    steps:
    47	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    48	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
    49	        with:
    50	          persist-credentials: false
    51	          # Full history: the accounting producer derives last_touch_commit via
    52	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
    53	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
    54	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
    55	          # and identical to a full local clone.
    56	          fetch-depth: 0
    57	      - name: Install buck2 (digest-pinned prebuilt release)
    58	        run: infra/ci/install-buck2.sh
    59	      - name: Materialize cloud-ci generated faces
    60	        run: infra/ci/materialize-cloud-ci-generated-faces.sh .
    61	      - name: Upload regenerated faces
    62	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    63	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
    64	        with:
    65	          name: accounting-faces
    66	          path: cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/*.generated.json
    67	          if-no-files-found: error
    68	
    69	  # ── GATE LANES (reusable matrix). Every homogeneous gate is the SAME step — one
    70	  #    `cargo test -p <gate crate>` — so instead of copy-pasting a job per gate, a single
    71	  #    matrixed `gate` job fans out over the crate list. Adding a gate is ONE line in the matrix
    72	  #    below; the `gate_registration` meta-test (in the cloud-ci-firewall lane) ENFORCES that
    73	  #    every in-tree gate crate is listed here AND that this job is a fan-in dependency, so a
    74	  #    gate can never be silently dropped. Each matrix leg is its own check-run
    75	  #    `gate (oya-cloud-ci-<x>-app)`, preserving per-gate attribution; legs with a live-corpus
    76	  #    self-test are born-blocking. `fail-fast: false` = surface-all (every leg runs to
    77	  #    completion even if a sibling fails). `fetch-depth: 0` because the presubmit
    78	  #    materialization boundary regenerates SCM facts from full history before tests run.
    79	  #    (Deliberately a matrix, NOT a `workflow_call` reusable workflow: a called workflow would
    80	  #    rename the published check-runs [`<caller> / <job>`], breaking the `oya-ci-required`
    81	  #    branch-protection context. A future owned oya-ci runner can reuse this matrix verbatim —
    82	  #    "one logic, two runners", D-CICD-AUTHORITY.)
    83	  gate:
    84	    # Descriptive per-leg check-run name (matrix.label) — each leg publishes as
    85	    # "gate · <discipline>", not a bare "gate (crate)". Adding a gate = one `include` line
    86	    # (crate + label); the gate_registration meta-test enforces every gate crate is listed.
    87	    name: ${{ matrix.label }}
    88	    runs-on: ubuntu-latest
    89	    strategy:
    90	      fail-fast: false
    91	      matrix:
    92	        include:
    93	          - { crate: oya-cloud-ci-cross-artifact-agreement-app, label: "gate · cross-artifact-agreement (GATE-1)" }
    94	          - { crate: oya-cloud-ci-total-accounting-app,         label: "gate · total-accounting (GATE-2)" }
    95	          - { crate: oya-cloud-ci-staleness-reaper-app,         label: "gate · staleness-reaper (GATE-3, born-blocking)" }
    96	          - { crate: oya-cloud-ci-automation-ratchet-app,       label: "gate · automation-ratchet (GATE-4, polices gates)" }
    97	          - { crate: oya-cloud-ci-bnf-layer-suffix-app,         label: "gate · bnf-layer-suffix (BNF §2.5#4, born-blocking)" }
    98	          - { crate: oya-cloud-ci-manifest-hygiene-app,         label: "gate · manifest-hygiene (§2.5#7, born-blocking)" }
    99	          - { crate: oya-cloud-ci-cargo-prefix-app,            label: "gate · cargo-prefix (ADR-0017)" }
   100	          - { crate: oya-cloud-ci-slo-coverage-app,            label: "gate · slo-coverage (catalog SLO input contract, born-blocking)" }
   101	          - { crate: oya-cloud-ci-license-policy-app,          label: "gate · license-policy (ADR-0013, born-blocking)" }
   102	          - { crate: oya-cloud-ci-zero-static-secrets-app,     label: "gate · zero-static-secrets (tracked corpus secret hygiene, born-blocking)" }
   103	          - { crate: oya-cloud-ci-load-balancer-inventory-app, label: "gate · load-balancer-inventory (tenant-facing edge taxonomy)" }
   104	          - { crate: oya-cloud-ci-multi-region-disposition-app, label: "gate · multi-region-disposition (manifest/doc readiness)" }
   105	          - { crate: oya-cloud-ci-sovereign-tenant-pin-app,    label: "gate · sovereign-tenant-pin (multi-region 421+Location readiness)" }
   106	          - { crate: oya-cloud-ci-tenant-environment-tier-app, label: "gate · tenant-environment-tier (env-tier isolation readiness)" }
   107	          - { crate: oya-cloud-ci-workspace-glob-coverage-app, label: "gate: workspace-glob-coverage (ADR-0538)" }
   108	          - { crate: oya-cloud-ci-target-parity-app,           label: "gate · target-parity (ADR-0540, test-wiring false-green)" }
   109	          - { crate: oya-cloud-ci-enforcement-liveness-app,    label: "gate · enforcement-liveness (FRIC-012, hook mirror liveness)" }
   110	          - { crate: oya-cloud-ci-generated-artifact-control-plane-app, label: "gate · generated-artifact-control-plane (public hermetic CI artifact policy)" }
   111	    steps:
   112	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   113	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   114	        with:
   115	          persist-credentials: false
   116	          fetch-depth: 0
   117	      - name: Install buck2 (digest-pinned prebuilt release)
   118	        run: infra/ci/install-buck2.sh
   119	      - name: Materialize cloud-ci generated faces
   120	        run: infra/ci/materialize-cloud-ci-generated-faces.sh .
   121	      - name: cargo test ${{ matrix.crate }}
   122	        run: cargo test --locked --no-fail-fast -p ${{ matrix.crate }} -- --test-threads=1
   123	
   124	  # ── freshness: first-diagnosis gate for the two stale-output failures from PR #662.
   125	  #    Runs as its own fast job with no needs edge so stale Cargo.lock and stale generated faces
   126	  #    surface together before the broader cargo/Buck lanes spend a full matrix round-trip.
   127	  gate-freshness:
   128	    name: freshness (lock + generated faces, ADR-0539)
   129	    runs-on: ubuntu-latest
   130	    steps:
   131	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   132	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   133	        with:
   134	          persist-credentials: false
   135	          fetch-depth: 0
   136	      - name: Install buck2 (digest-pinned prebuilt release)
   137	        run: infra/ci/install-buck2.sh
   138	      - name: Pre-provision pinned Rust toolchain for Buck2 freshness binaries
   139	        run: |
   140	          set -euo pipefail
   141	          rustup show active-toolchain || rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy
   142	          rustup component add rustfmt clippy --toolchain 1.95.0
   143	          rustc --version
   144	          cargo --version
   145	      - name: Run freshness gate
   146	        run: |
   147	          set -euo pipefail
   148	          freshness_bin="$(buck2 build //cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-freshness-app-bin --show-output | awk '{print $2}')"
   149	          "${freshness_bin}" --repo-root .
   150	
   151	  # ── registry-drift: materialized workspace == regenerated byte-equal. Starts at t=0 alongside
   152	  #    producer-regen; it rematerializes in-job so it is hermetic and self-contained. The
   153	  #    producer-regen needs-edge was cosmetic (evidence only, nothing consumed) and serialized
   154	  #    this job unnecessarily — removed so it starts at t=0.
   155	  gate-registry-drift:
   156	    name: registry-drift (materialized == regenerated)
   157	    runs-on: ubuntu-latest
   158	    steps:
   159	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   160	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   161	        with:
   162	          persist-credentials: false
   163	          # Full history: the accounting producer derives last_touch_commit via
   164	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
   165	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
   166	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
   167	          # and identical to a full local clone.
   168	          fetch-depth: 0
   169	      - name: Install buck2 (digest-pinned prebuilt release)
   170	        run: infra/ci/install-buck2.sh
exec
/bin/zsh -lc "nl -ba .github/workflows/oya-ci-required.yml | sed -n '341,470p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   341	  app-shell-codegen:
   342	    name: app-shell-codegen (ignored generated clients are reproducible)
   343	    runs-on: ubuntu-latest
   344	    steps:
   345	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   346	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   347	        with:
   348	          persist-credentials: false
   349	      - name: Prepare pinned pnpm
   350	        run: |
   351	          set -euo pipefail
   352	          corepack enable
   353	          corepack prepare pnpm@11.5.2 --activate
   354	          pnpm --version
   355	      - name: Regenerate and verify app-shell clients
   356	        run: |
   357	          set -euo pipefail
   358	          pnpm --dir oya/app-shell-frontend install --frozen-lockfile
   359	          pnpm --dir oya/app-shell-frontend codegen
   360	          pnpm --dir oya/app-shell-frontend codegen:check
   361	          pnpm --dir oya/app-shell-frontend typecheck
   362	
   363	  # ── PR REVIEWER EVIDENCE LANE. Branch protection intentionally requires only the
   364	  #    single `oya-ci-required` context, while GitHub Review API approvals are not merge
   365	  #    authority until `oya-pr-review` has a live trusted producer. This lane closes the
   366	  #    guardrail gap by validating the pull-request body itself on pull_request events:
   367	  #    merge-ready PRs must carry lead-owned `## Code Review` reviewer-agent evidence
   368	  #    with an APPROVE verdict, or the fan-in stays red. Non-PR events do not have a PR
   369	  #    body and pass through; the protected PR path is enforced before merge.
   370	  pr-reviewer-evidence:
   371	    name: pr-reviewer-evidence (PR body reviewer-agent APPROVE)
   372	    runs-on: ubuntu-latest
   373	    steps:
   374	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   375	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   376	        with:
   377	          persist-credentials: false
   378	      - name: Validate pull-request reviewer evidence
   379	        env:
   380	          EVENT_NAME: ${{ github.event_name }}
   381	          PR_BODY_PATH: ${{ runner.temp }}/pr-body.md
   382	        run: |
   383	          set -euo pipefail
   384	          if [ "${EVENT_NAME}" != "pull_request" ]; then
   385	            echo "pr-reviewer-evidence: ${EVENT_NAME} event has no PR body; pull_request path owns reviewer evidence enforcement."
   386	            exit 0
   387	          fi
   388	
   389	          python3 - <<'PY' > "${PR_BODY_PATH}"
   390	          import json
   391	          import os
   392	          import pathlib
   393	          import sys
   394	
   395	          event = json.loads(pathlib.Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
   396	          body = event.get("pull_request", {}).get("body")
   397	          if not body:
   398	              sys.stderr.write("pr-reviewer-evidence: pull_request.body is empty; reviewer-agent evidence missing\n")
   399	              sys.exit(1)
   400	          sys.stdout.write(body)
   401	          PY
   402	
   403	          cargo run --locked -p oya-dev-cli -- gate validate pr-traceability \
   404	            --pr-body "${PR_BODY_PATH}" \
   405	            --require-code-review
   406	
   407	  # ── THE FAN-IN. This is the single required context branch protection keys on. It has NO
   408	  #    command of its own (Principle 1 — it NEVER re-runs a narrower subset): it is green IFF
   409	  #    every gate lane above is green. `needs:` lists EVERY gate job; the gate-registration
   410	  #    meta-test (in the firewall lane) asserts every in-tree gate crate is represented here.
   411	  oya-ci-required:
   412	    name: oya-ci-required
   413	    runs-on: ubuntu-latest
   414	    if: ${{ always() }}
   415	    needs:
   416	      - gate                    # the matrix of homogeneous gate lanes (success IFF every leg passed)
   417	      - gate-freshness          # bespoke: stale Cargo.lock + generated faces first diagnosis
   418	      - gate-registry-drift     # bespoke: materialized == regenerated byte-parity
   419	      - gate-cloud-ci-firewall  # bespoke: baseline ratchet + gate_registration meta-test
   420	      - generated-output-diff-policy # generated outputs are never PR merge surfaces
   421	      - buck2                   # hermetic buck2 build + affected gate tests (alongside cargo)
   422	      - app-shell-codegen       # ignored generated app-shell clients regenerate from canonical contracts
   423	      - pr-reviewer-evidence    # PR body carries reviewer-agent APPROVE evidence inside the required context
   424	    steps:
   425	      - name: Fan-in verdict (green IFF every gate lane is green)
   426	        run: |
   427	          echo "Gate results:"
   428	          echo "  gate (matrix)     = ${{ needs.gate.result }}"
   429	          echo "  freshness         = ${{ needs.gate-freshness.result }}"
   430	          echo "  registry-drift    = ${{ needs.gate-registry-drift.result }}"
   431	          echo "  cloud-ci-firewall = ${{ needs.gate-cloud-ci-firewall.result }}"
   432	          echo "  generated-diff    = ${{ needs.generated-output-diff-policy.result }}"
   433	          echo "  buck2             = ${{ needs.buck2.result }}"
   434	          echo "  app-shell-codegen = ${{ needs.app-shell-codegen.result }}"
   435	          echo "  reviewer-evidence = ${{ needs.pr-reviewer-evidence.result }}"
   436	          # A matrix job's aggregate `.result` is success IFF ALL legs succeeded, failure if any
   437	          # leg failed, skipped if it never ran — so one check covers every gate in the matrix.
   438	          if [ "${{ needs.gate.result }}" = "success" ] \
   439	            && [ "${{ needs.gate-freshness.result }}" = "success" ] \
   440	            && [ "${{ needs.gate-registry-drift.result }}" = "success" ] \
   441	            && [ "${{ needs.gate-cloud-ci-firewall.result }}" = "success" ] \
   442	            && [ "${{ needs.generated-output-diff-policy.result }}" = "success" ] \
   443	            && [ "${{ needs.buck2.result }}" = "success" ] \
   444	            && [ "${{ needs.app-shell-codegen.result }}" = "success" ] \
   445	            && [ "${{ needs.pr-reviewer-evidence.result }}" = "success" ]; then
   446	            echo "oya-ci-required: GREEN — every constituent gate lane passed."
   447	            exit 0
   448	          fi
   449	          echo "oya-ci-required: RED — at least one constituent gate lane failed or was skipped."
   450	          exit 1
exec
/bin/zsh -lc 'nl -ba .github/workflows/docs-graph-drift.yml; nl -ba infra/ci/install-buck2.sh; nl -ba infra/ci/buck2-affected-gate.sh; nl -ba infra/ci/materialize-cloud-ci-generated-faces.sh' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	name: docs-graph-drift
     2	
     3	# Drift gate for the architecture dashboard (docs/architecture/product-graph.html).
     4	#
     5	# The dashboard is GENERATED from SSOT by an owned-Rust CI pipeline tool
     6	# (tools/oya-architecture-graph-generator-app) — NOT a developer-facing `oya`
     7	# subcommand. This job rebuilds the generator, regenerates the dashboard, and
     8	# byte-compares the result against the committed HTML. If the committed file is
     9	# stale, the job fails (red) and the author must run the generator and commit.
    10	#
    11	# Authority model: GitHub Actions (this workflow) is the SOLE live gating check
    12	# (D-CICD-AUTHORITY; the legacy CI bridge was retired 2026-06-07). Intentionally ABSENT from
    13	# the branch-protection required set (.github/branch-protection.yaml) — feedback only;
    14	# branch protection is NOT changed here.
    15	
    16	on:
    17	  pull_request:
    18	    paths:
    19	      - ".github/workflows/docs-graph-drift.yml"
    20	      - "tools/oya-architecture-graph-generator-app/**"
    21	      - "docs/architecture/product-graph.template.html"
    22	      - "docs/architecture/product-graph.html"
    23	      - "docs/machine-readable/architecture-graph.json"
    24	      - "docs/machine-readable/masterplan.generated.json"
    25	  push:
    26	    branches: [dev]
    27	    paths:
    28	      - ".github/workflows/docs-graph-drift.yml"
    29	      - "tools/oya-architecture-graph-generator-app/**"
    30	      - "docs/architecture/product-graph.template.html"
    31	      - "docs/architecture/product-graph.html"
    32	      - "docs/machine-readable/architecture-graph.json"
    33	      - "docs/machine-readable/masterplan.generated.json"
    34	
    35	permissions:
    36	  contents: read
    37	
    38	concurrency:
    39	  group: docs-graph-drift-${{ github.workflow }}-${{ github.head_ref || github.run_id }}
    40	  cancel-in-progress: true
    41	
    42	jobs:
    43	  docs-graph-drift:
    44	    name: docs-graph-drift
    45	    runs-on: ubuntu-latest
    46	    timeout-minutes: 15
    47	    steps:
    48	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    49	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
    50	        with:
    51	          persist-credentials: false
    52	      - name: Install Rust toolchain
    53	        uses: dtolnay/rust-toolchain@21dc36fb71dd22e3317045c0c31a3f4249868b17
    54	        with:
    55	          toolchain: stable
    56	      - name: Cache cargo registry + target
    57	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    58	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
    59	        with:
    60	          path: |
    61	            ~/.cargo/registry
    62	            ~/.cargo/git
    63	            target
    64	          # Lockfile-scoped cache avoids per-commit cache churn; cargo still validates sources.
    65	          key: docs-graph-drift-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}
    66	          restore-keys: |
    67	            docs-graph-drift-${{ runner.os }}-
    68	      - name: Build + test the architecture-graph generator
    69	        env:
    70	          CARGO_TERM_COLOR: always
    71	          CARGO_INCREMENTAL: "0"
    72	        run: |
    73	          set -euo pipefail
    74	          cargo build --locked -p oya-architecture-graph-generator-app
    75	          cargo test --locked -p oya-architecture-graph-generator-app
    76	      - name: Regenerate the dashboard
    77	        run: |
    78	          set -euo pipefail
    79	          cargo run --locked -q -p oya-architecture-graph-generator-app \
    80	            --bin oya-architecture-graph-generator -- --write
    81	      - name: Fail on dashboard drift
    82	        run: |
    83	          set -euo pipefail
    84	          git diff --exit-code -- docs/architecture/product-graph.html \
    85	            || { echo "::error::docs/architecture/product-graph.html is stale; run the generator (--write) and commit."; exit 1; }
     1	#!/usr/bin/env bash
     2	# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
     3	set -euo pipefail
     4	
     5	BUCK2_RELEASE="${BUCK2_RELEASE:-2026-06-01}"
     6	BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
     7	
     8	case "$(uname -s)-$(uname -m)" in
     9	  Linux-x86_64)
    10	    BUCK2_ASSET="${BUCK2_ASSET:-buck2-x86_64-unknown-linux-gnu.zst}"
    11	    BUCK2_SHA256="${BUCK2_SHA256:-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555}"
    12	    ;;
    13	  *)
    14	    if [ "${OYA_CI_ALLOW_AMBIENT_BUCK2:-}" = "1" ] && command -v buck2 >/dev/null 2>&1; then
    15	      echo "Using ambient buck2 only because OYA_CI_ALLOW_AMBIENT_BUCK2=1 was set." >&2
    16	      buck2 --version
    17	      exit 0
    18	    fi
    19	    echo "Unsupported host for default pinned Buck2 install; set OYA_CI_ALLOW_AMBIENT_BUCK2=1 for local advisory use." >&2
    20	    exit 1
    21	    ;;
    22	esac
    23	
    24	mkdir -p "${BUCK2_INSTALL_DIR}"
    25	curl -fsSL "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/${BUCK2_ASSET}" -o "${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}"
    26	echo "${BUCK2_SHA256}  ${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}" | sha256sum -c -
    27	zstd -d -f "${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}" -o "${BUCK2_INSTALL_DIR}/buck2"
    28	chmod +x "${BUCK2_INSTALL_DIR}/buck2"
    29	
    30	if [ -n "${GITHUB_PATH:-}" ]; then
    31	  echo "${BUCK2_INSTALL_DIR}" >> "${GITHUB_PATH}"
    32	fi
    33	
    34	"${BUCK2_INSTALL_DIR}/buck2" --version
     1	#!/bin/sh
     2	# buck2-native affected-only CI gate.
     3	#
     4	# Builds + tests the reverse-dependency closure of the PR's changed files —
     5	# the hyperscaler "affected targets" pattern (Google/Meta), buck2-native via
     6	# `uquery owner()` + `rdeps()`. Replaces the cargo-era `oya verify --affected`.
     7	# No oya-dev-cli dependency.
     8	#
     9	# Usage:  buck2-affected-gate.sh <base-ref> [head-ref]
    10	#         base-ref  — the merge-base anchor (e.g. origin/dev)
    11	#         head-ref  — the tip to diff (default: HEAD)
    12	#
    13	# The 1-arg form (buck2-affected-gate.sh origin/dev) diffs the current
    14	# checkout: HEAD is the PR checkout in the GitHub Actions runner, so omitting
    15	# head-ref is the default invocation.
    16	#
    17	# The 2-arg form (buck2-affected-gate.sh origin/dev origin/pr-N) is used by
    18	# the controller Job, where the working tree is trunk (dev) and the PR ref
    19	# is fetched as data via `git fetch origin refs/pull/N/head:refs/remotes/origin/pr-N`.
    20	#
    21	# Exit 0 = pass (incl. non-Rust / no-affected PRs); non-zero = build/test failure.
    22	set -eu
    23	
    24	BASE="${1:-origin/dev}"
    25	HEAD_REF="${2:-HEAD}"
    26	BUCK2="${BUCK2:-buck2}"
    27	
    28	echo "buck2-affected-gate: start (pwd=$(pwd) base=$BASE head-ref=$HEAD_REF resolved=$(git rev-parse --short "$HEAD_REF" 2>&1))"
    29	echo "buck2-affected-gate: .buckconfig=$(test -f .buckconfig && echo present || echo MISSING) HOME=${HOME:-unset} buck2=$($BUCK2 --version 2>&1 | head -1)"
    30	if ! git rev-parse --verify --quiet "$BASE" >/dev/null 2>&1; then
    31	  echo "buck2-affected-gate: FATAL base ref '$BASE' does not resolve in this checkout"
    32	  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
    33	  exit 1
    34	fi
    35	if ! git rev-parse --verify --quiet "$HEAD_REF" >/dev/null 2>&1; then
    36	  echo "buck2-affected-gate: FATAL head ref '$HEAD_REF' does not resolve in this checkout"
    37	  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
    38	  exit 1
    39	fi
    40	if ! MERGE_BASE=$(git merge-base "$HEAD_REF" "$BASE" 2>&1); then
    41	  echo "buck2-affected-gate: FATAL merge-base $HEAD_REF $BASE failed (need full history): $MERGE_BASE"
    42	  exit 1
    43	fi
    44	CHANGED=$(git diff --name-only "$MERGE_BASE" "$HEAD_REF")
    45	if [ -z "$CHANGED" ]; then
    46	  echo "buck2-affected-gate: no changed files vs $BASE ($HEAD_REF) -> PASS"
    47	  exit 0
    48	fi
    49	echo "buck2-affected-gate: $(printf '%s\n' "$CHANGED" | wc -l | tr -d ' ') changed file(s) vs $BASE..${HEAD_REF} (merge-base $MERGE_BASE)"
    50	
    51	# Classify. Only docs/non-graph files (e.g. .md/.yaml/.json outside crates) may
    52	# legitimately map to no target. A *.rs / Cargo.toml / buck-graph file MUST map to
    53	# a target — FAIL CLOSED if it doesn't (never silently pass a Rust change unbuilt).
    54	RUST_REL=$(printf '%s\n' "$CHANGED" | grep -E '\.rs$|/Cargo\.toml$|^Cargo\.(toml|lock)$|^\.buckconfig$|(^|/)BUCK$|^toolchains/|^third-party/' || true)
    55	if [ -z "$RUST_REL" ]; then
    56	  echo "buck2-affected-gate: no Rust/buck-graph files changed -> NoRust PASS"
    57	  exit 0
    58	fi
    59	
    60	# owner() resolution — batched to minimise buck2 daemon round-trips.
    61	#
    62	# Strategy:
    63	#   1. BUCK files: no owner() result by design (they ARE the package definition).
    64	#      Run a small per-file pass to expand each to its package target pattern.
    65	#      (One buck2 uquery per BUCK file — these are typically 0-1 files per PR.)
    66	#   2. Non-BUCK Rust/graph files: build ONE "owner('f1') union owner('f2') union ..."
    67	#      expression and run a single buck2 uquery call for all files at once.
    68	#      owner() takes file-path strings, not target-set placeholders, so %Ss/@argfile
    69	#      cannot be used here — the union expression is the correct single-call form.
    70	#      A uquery ERROR (non-zero exit) FAILS the gate — it is NOT 'no owner'.
    71	#      (The false-pass bug was: 2>/dev/null||true swallowed buck2 errors.)
    72	
    73	OWNERS=""
    74	
    75	# ── Pass 1: BUCK files → package target pattern (unchanged semantics, separate pass) ──
    76	BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -E '(^|/)BUCK$' || true)
    77	for f in $BUCK_FILES; do
    78	  [ -e "$f" ] || continue
    79	  d=$(dirname "$f")
    80	  case "$d" in
    81	    third-party)   pat="third-party//:" ;;
    82	    third-party/*) pat="third-party//${d#third-party/}:" ;;
    83	    toolchains)    pat="toolchains//:" ;;
    84	    toolchains/*)  pat="toolchains//${d#toolchains/}:" ;;
    85	    .)             pat="//:" ;;
    86	    *)             pat="//$d:" ;;
    87	  esac
    88	  if ! o=$("$BUCK2" uquery "$pat" 2>/tmp/uqerr); then
    89	    echo "buck2-affected-gate: FATAL buck2 uquery '$pat' (BUCK pkg for $f) errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
    90	  fi
    91	  [ -n "$o" ] && OWNERS="$OWNERS $o"
    92	done
    93	
    94	# ── Pass 2: non-BUCK files → ONE batched uquery call via union-of-owner() expression ──
    95	# Build: owner('f1') union owner('f2') union ... and run as a single buck2 uquery invocation.
    96	# This replaces N serial daemon round-trips (one per file) with a single round-trip.
    97	NON_BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -vE '(^|/)BUCK$' || true)
    98	NON_BUCK_EXISTING=$(printf '%s\n' "$NON_BUCK_FILES" | while read -r f; do [ -e "$f" ] && printf '%s\n' "$f"; done)
    99	if [ -n "$NON_BUCK_EXISTING" ]; then
   100	  OWNER_EXPR=$(printf '%s\n' "$NON_BUCK_EXISTING" | \
   101	    awk 'NR==1{printf "owner('"'"'%s'"'"')", $0; next} {printf " union owner('"'"'%s'"'"')", $0}')
   102	  if ! o=$("$BUCK2" uquery "$OWNER_EXPR" 2>/tmp/uqerr); then
   103	    echo "buck2-affected-gate: FATAL buck2 uquery owner() errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
   104	  fi
   105	  [ -n "$o" ] && OWNERS="$OWNERS $o"
   106	fi
   107	
   108	OWNERS=$(printf '%s\n' $OWNERS | sed '/^$/d' | sort -u)
   109	if [ -z "$OWNERS" ]; then
   110	  echo "buck2-affected-gate: FATAL Rust/buck files changed but NO owning target found (refusing to false-pass):"
   111	  printf '    %s\n' $RUST_REL
   112	  exit 1
   113	fi
   114	echo "buck2-affected-gate: $(printf '%s\n' "$OWNERS" | wc -l | tr -d ' ') owning target(s)"
   115	
   116	# Affected = changed targets + reverse-dep closure. rdeps error also FAILS closed.
   117	# Pass owners via @argfile + the %Ss set placeholder, NOT an inline set(...): a change
   118	# to a large BUCK package (e.g. third-party/BUCK owns 1689 targets) overflows the inline
   119	# query string and buck2 errors out (uquery RC=3, no build attempted) — which silently
   120	# blocked landing ANY third-party change. @argfile + %Ss handles an arbitrary set size
   121	# (verified: 1689 owners -> 1919 affected). One owner per line.
   122	printf '%s\n' $OWNERS | sed '/^$/d' > /tmp/gate-owners.txt
   123	if ! AFFECTED=$("$BUCK2" uquery 'rdeps(//..., %Ss)' @/tmp/gate-owners.txt 2>/tmp/rqerr); then
   124	  echo "buck2-affected-gate: FATAL rdeps query errored:"; sed 's/^/    /' /tmp/rqerr; exit 1
   125	fi
   126	N=$(printf '%s\n' "$AFFECTED" | sed '/^$/d' | wc -l | tr -d ' ')
   127	echo "buck2-affected-gate: $N affected target(s) (owners + reverse-dep closure)"
   128	if [ "$N" = "0" ]; then echo "buck2-affected-gate: FATAL owners found but rdeps empty (query problem)"; exit 1; fi
   129	
   130	# Build then test the affected set. @- reads the newline-delimited target list
   131	# from stdin, avoiding ARG_MAX limits on large closures.
   132	printf '%s\n' "$AFFECTED" | sed '/^$/d' > /tmp/affected-targets.txt
   133	echo "=== buck2 build (affected) ==="
   134	"$BUCK2" build @/tmp/affected-targets.txt
   135	echo "=== buck2 test (affected) ==="
   136	"$BUCK2" test @/tmp/affected-targets.txt
   137	echo "buck2-affected-gate: PASS"
     1	#!/usr/bin/env bash
     2	# Materialize cloud-ci generated faces from the checked-out candidate tree.
     3	#
     4	# Generated faces are CI/controller outputs, not contributor-owned merge surfaces. This boundary
     5	# step regenerates the declared SCM snapshot and accounting faces before gates consume them, so
     6	# PRs and merge queues validate the final candidate tree without hand-merging generated JSON.
     7	set -euo pipefail
     8	
     9	repo_root="${1:-.}"
    10	faces_dir="${repo_root%/}/cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app"
    11	scm_facts="${faces_dir}/scm-facts.generated.json"
    12	
    13	toolchain="$(awk -F'"' '/channel/ {print $2; exit}' "${repo_root%/}/rust-toolchain.toml")"
    14	rustup toolchain install "$toolchain" --profile minimal --component rustfmt --component clippy
    15	
    16	# Build both targets in one buck2 invocation and capture output paths in the same call.
    17	# --show-output prints "<target> <path>" per line; we match each target name to extract its path.
    18	# This replaces the previous pattern of 1 build + 2 separate --show-output queries (3 round-trips).
    19	show_out="$(buck2 build \
    20	  //cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app \
    21	  //cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin \
    22	  --show-output 2>/dev/null)"
    23	
    24	emitter="$(printf '%s\n' "$show_out" | awk '/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app/ {print $2}')"
    25	producer="$(printf '%s\n' "$show_out" | awk '/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin/ {print $2}')"
    26	
    27	"$emitter" --repo-root "$repo_root" --out "$scm_facts"
    28	"$producer" --repo-root "$repo_root" --scm-facts "$scm_facts"
exec
/bin/zsh -lc "nl -ba .github/workflows/oya-ci-required.yml | sed -n '361,425p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   361	          pnpm --dir oya/app-shell-frontend typecheck
   362	
   363	  # ── PR REVIEWER EVIDENCE LANE. Branch protection intentionally requires only the
   364	  #    single `oya-ci-required` context, while GitHub Review API approvals are not merge
   365	  #    authority until `oya-pr-review` has a live trusted producer. This lane closes the
   366	  #    guardrail gap by validating the pull-request body itself on pull_request events:
   367	  #    merge-ready PRs must carry lead-owned `## Code Review` reviewer-agent evidence
   368	  #    with an APPROVE verdict, or the fan-in stays red. Non-PR events do not have a PR
   369	  #    body and pass through; the protected PR path is enforced before merge.
   370	  pr-reviewer-evidence:
   371	    name: pr-reviewer-evidence (PR body reviewer-agent APPROVE)
   372	    runs-on: ubuntu-latest
   373	    steps:
   374	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   375	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   376	        with:
   377	          persist-credentials: false
   378	      - name: Validate pull-request reviewer evidence
   379	        env:
   380	          EVENT_NAME: ${{ github.event_name }}
   381	          PR_BODY_PATH: ${{ runner.temp }}/pr-body.md
   382	        run: |
   383	          set -euo pipefail
   384	          if [ "${EVENT_NAME}" != "pull_request" ]; then
   385	            echo "pr-reviewer-evidence: ${EVENT_NAME} event has no PR body; pull_request path owns reviewer evidence enforcement."
   386	            exit 0
   387	          fi
   388	
   389	          python3 - <<'PY' > "${PR_BODY_PATH}"
   390	          import json
   391	          import os
   392	          import pathlib
   393	          import sys
   394	
   395	          event = json.loads(pathlib.Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
   396	          body = event.get("pull_request", {}).get("body")
   397	          if not body:
   398	              sys.stderr.write("pr-reviewer-evidence: pull_request.body is empty; reviewer-agent evidence missing\n")
   399	              sys.exit(1)
   400	          sys.stdout.write(body)
   401	          PY
   402	
   403	          cargo run --locked -p oya-dev-cli -- gate validate pr-traceability \
   404	            --pr-body "${PR_BODY_PATH}" \
   405	            --require-code-review
   406	
   407	  # ── THE FAN-IN. This is the single required context branch protection keys on. It has NO
   408	  #    command of its own (Principle 1 — it NEVER re-runs a narrower subset): it is green IFF
   409	  #    every gate lane above is green. `needs:` lists EVERY gate job; the gate-registration
   410	  #    meta-test (in the firewall lane) asserts every in-tree gate crate is represented here.
   411	  oya-ci-required:
   412	    name: oya-ci-required
   413	    runs-on: ubuntu-latest
   414	    if: ${{ always() }}
   415	    needs:
   416	      - gate                    # the matrix of homogeneous gate lanes (success IFF every leg passed)
   417	      - gate-freshness          # bespoke: stale Cargo.lock + generated faces first diagnosis
   418	      - gate-registry-drift     # bespoke: materialized == regenerated byte-parity
   419	      - gate-cloud-ci-firewall  # bespoke: baseline ratchet + gate_registration meta-test
   420	      - generated-output-diff-policy # generated outputs are never PR merge surfaces
   421	      - buck2                   # hermetic buck2 build + affected gate tests (alongside cargo)
   422	      - app-shell-codegen       # ignored generated app-shell clients regenerate from canonical contracts
   423	      - pr-reviewer-evidence    # PR body carries reviewer-agent APPROVE evidence inside the required context
   424	    steps:
   425	      - name: Fan-in verdict (green IFF every gate lane is green)
exec
/bin/zsh -lc "nl -ba .github/workflows/oya-ci-required.yml | sed -n '239,360p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   239	  # ── HERMETIC BUCK2 LANE (OYA-CI-HERMETIC-EXECUTION-DESIGN §3 + Stage P1/P2). Runs the SAME
   240	  #    gate logic as the cargo lanes above, but through buck2: `buck2 build` compiles every
   241	  #    target (the env!CARGO eradication) and `buck2 test` runs the gate rust_tests fully
   242	  #    hermetically (no ambient git in any action — the producer reads the materialized scm-facts
   243	  #    face; the scm-facts emitter is the single out-of-graph boundary, run in the
   244	  #    materialization step BELOW, never inside a cacheable action). Scoped by the
   245	  #    affected-set driver (`infra/ci/buck2-affected-gate.sh`: uquery owner -> rdeps closure,
   246	  #    FAILS CLOSED) for speed. RBE/NativeLink is staged LAST (D4) and NOT required for
   247	  #    hermeticity — local-on-runner execution via the wired `noop_test_toolchain` is sufficient
   248	  #    here. This lane runs ALONGSIDE the cargo lanes (both feed the fan-in); the cargo->buck2
   249	  #    required-context content swap is a separate founder-paired step.
   250	  buck2:
   251	    name: buck2 (hermetic build + affected gate tests)
   252	    runs-on: ubuntu-latest
   253	    steps:
   254	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   255	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   256	        with:
   257	          persist-credentials: false
   258	          # Full history: the materialization step below runs the emitter, which derives
   259	          # last_touch via `git log --name-only`; a shallow checkout collapses it (PM1).
   260	          fetch-depth: 0
   261	      # buck2 is the BUILD TOOL (like cargo/rustc), installed as a prebuilt release.
   262	      # The adapter edge is immutable at CI time: release tag selects the asset, SHA-256 pins
   263	      # the bytes, and CI verifies the digest before decompression/execution. Bump together.
   264	      - name: Install buck2 (digest-pinned prebuilt release)
   265	        run: infra/ci/install-buck2.sh
   266	      # Pre-provision the pinned rust toolchain ONCE, serially, before the buck2 build.
   267	      # The buck2 rust toolchain (toolchains/BUCK: system_rust_toolchain via the rustup shim)
   268	      # resolves rustc/cargo/clippy per-compile-action, and buck2 runs those actions in
   269	      # PARALLEL. On a cold runner each action's first shim call triggers rustup to install the
   270	      # rust-toolchain.toml channel (1.95.0 + rustfmt,clippy) concurrently — the racing rustup
   271	      # processes collide on the shared `~/.rustup/downloads/*.partial` files and fail with
   272	      # `rustup::utils::rename ... No such file or directory (os error 2)` (a different component
   273	      # each run: clippy, then cargo — proving a concurrency race, not a config defect). rustup
   274	      # is not concurrency-safe. Installing the toolchain once here makes it ambient so the
   275	      # parallel actions find it already present (no download). The cargo lanes never hit this
   276	      # because cargo provisions the toolchain in a single up-front invocation.
   277	      - name: Pre-provision pinned rust toolchain (serialize rustup before parallel buck2)
   278	        run: |
   279	          set -euo pipefail
   280	          rustup show active-toolchain || rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy
   281	          rustup component add rustfmt clippy --toolchain 1.95.0
   282	          rustc --version
   283	          cargo --version
   284	      # Warm buck-out + the buck2 daemon cache across runs so ephemeral runners do not
   285	      # cold-bootstrap the toolchain (design §3.1 / ADR-0515 D4).
   286	      #
   287	      # Cache key is STABLE per dependency-set (.buckconfig + toolchains/BUCK + Cargo.lock),
   288	      # NOT per-commit. The previous `-${{ github.sha }}` suffix made the primary key unique
   289	      # every commit, so actions/cache SAVED a fresh full buck-out (multi-GB) on EVERY run and
   290	      # never hit the primary key — bloating the 10GB repo cache into constant LRU eviction and
   291	      # exhausting ephemeral-runner disk at the save step (the "No space left on device" failure).
   292	      # A stable key saves once per dependency-set and restores it exactly: deterministic warm
   293	      # start, no per-commit bloat. Changed crates still rebuild (buck2 is content-addressed, so a
   294	      # restored hit is bit-identical to a cold build); only a Cargo.lock/toolchain/.buckconfig
   295	      # change mints a new entry. Interim warm-by-default until the shared content-addressed
   296	      # remote cache (NativeLink/CAS, HANDOFF W3) lands with a cold-canary integrity job proving
   297	      # cold==warm. See friction-ledger buck2-no-shared-cache.
   298	      - name: Cache buck-out
   299	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   300	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   301	        with:
   302	          path: buck-out
   303	          key: buck-out-${{ runner.os }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   304	          restore-keys: |
   305	            buck-out-${{ runner.os }}-
   306	      # Generated-face materialization — the SINGLE out-of-graph git boundary. Re-run the emitter
   307	      # and producer against the checked-out candidate tree, then let buck2 consume those files as
   308	      # declared inputs. We deliberately do NOT byte-compare against committed JSON here: that was
   309	      # the self-referential merge-conflict surface. Byte-parity is checked after materialization.
   310	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   311	        run: infra/ci/materialize-cloud-ci-generated-faces.sh .
   312	      # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication —
   313	      # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests
   314	      # run green, fully hermetic, with verdicts identical to the cargo lanes). This is the
   315	      # refactor's scope and is the binding hermetic check for this stage.
   316	      #
   317	      # NOTE: the affected-set driver (`infra/ci/buck2-affected-gate.sh`, uquery owner -> rdeps
   318	      # closure) is the design's speed mechanism (§3.2) and runs below as a best-effort fast path,
   319	      # but its whole-graph `rdeps(//..., ...)` currently FAILS CLOSED on a PRE-EXISTING stale
   320	      # git-worktree BUCK package (`.claire/worktrees/.../oya-payroll-run-usecase`) committed at
   321	      # base — graph pollution unrelated to this refactor. Until that is cleaned, the binding gate
   322	      # is the scoped `//cloud/cloud-ci/...` build+test above; the affected driver is advisory.
   323	      - name: buck2 build + test (//cloud/cloud-ci/..., hermetic — binding)
   324	        run: |
   325	          set -euo pipefail
   326	          # buck2 test builds its targets before running them, so a standalone
   327	          # `buck2 build` immediately before is redundant — removed (item 4 quick win).
   328	          buck2 test //cloud/cloud-ci/...
   329	      - name: buck2 affected-set driver (advisory speed path)
   330	        continue-on-error: true
   331	        run: |
   332	          set -euo pipefail
   333	          chmod +x infra/ci/buck2-affected-gate.sh
   334	          BUCK2=buck2 infra/ci/buck2-affected-gate.sh "origin/${{ github.base_ref || 'dev' }}"
   335	
   336	  # ── APP-SHELL GENERATED CLIENT LANE. Generated TypeScript clients are intentionally ignored
   337	  #    in git, so clean checkout CI must prove they regenerate from canonical OpenAPI sources
   338	  #    before the single required fan-in can go green. This is a bridge lane until the native
   339	  #    cloud-ci runner has first-class non-Rust toolchain actions; the invariant is still the
   340	  #    productized generated-output policy above, not committed client files.
   341	  app-shell-codegen:
   342	    name: app-shell-codegen (ignored generated clients are reproducible)
   343	    runs-on: ubuntu-latest
   344	    steps:
   345	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   346	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   347	        with:
   348	          persist-credentials: false
   349	      - name: Prepare pinned pnpm
   350	        run: |
   351	          set -euo pipefail
   352	          corepack enable
   353	          corepack prepare pnpm@11.5.2 --activate
   354	          pnpm --version
   355	      - name: Regenerate and verify app-shell clients
   356	        run: |
   357	          set -euo pipefail
   358	          pnpm --dir oya/app-shell-frontend install --frozen-lockfile
   359	          pnpm --dir oya/app-shell-frontend codegen
   360	          pnpm --dir oya/app-shell-frontend codegen:check
exec
/bin/zsh -lc 'git diff -- .github/workflows/oya-ci-required.yml' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
diff --git a/.github/workflows/oya-ci-required.yml b/.github/workflows/oya-ci-required.yml
index 22553aaad..f09219d0b 100644
--- a/.github/workflows/oya-ci-required.yml
+++ b/.github/workflows/oya-ci-required.yml
@@ -98,6 +98,12 @@ jobs:
           - { crate: oya-cloud-ci-manifest-hygiene-app,         label: "gate · manifest-hygiene (§2.5#7, born-blocking)" }
           - { crate: oya-cloud-ci-cargo-prefix-app,            label: "gate · cargo-prefix (ADR-0017)" }
           - { crate: oya-cloud-ci-slo-coverage-app,            label: "gate · slo-coverage (catalog SLO input contract, born-blocking)" }
+          - { crate: oya-cloud-ci-license-policy-app,          label: "gate · license-policy (ADR-0013, born-blocking)" }
+          - { crate: oya-cloud-ci-zero-static-secrets-app,     label: "gate · zero-static-secrets (tracked corpus secret hygiene, born-blocking)" }
+          - { crate: oya-cloud-ci-load-balancer-inventory-app, label: "gate · load-balancer-inventory (tenant-facing edge taxonomy)" }
+          - { crate: oya-cloud-ci-multi-region-disposition-app, label: "gate · multi-region-disposition (manifest/doc readiness)" }
+          - { crate: oya-cloud-ci-sovereign-tenant-pin-app,    label: "gate · sovereign-tenant-pin (multi-region 421+Location readiness)" }
+          - { crate: oya-cloud-ci-tenant-environment-tier-app, label: "gate · tenant-environment-tier (env-tier isolation readiness)" }
           - { crate: oya-cloud-ci-workspace-glob-coverage-app, label: "gate: workspace-glob-coverage (ADR-0538)" }
           - { crate: oya-cloud-ci-target-parity-app,           label: "gate · target-parity (ADR-0540, test-wiring false-green)" }
           - { crate: oya-cloud-ci-enforcement-liveness-app,    label: "gate · enforcement-liveness (FRIC-012, hook mirror liveness)" }
@@ -354,6 +360,50 @@ jobs:
           pnpm --dir oya/app-shell-frontend codegen:check
           pnpm --dir oya/app-shell-frontend typecheck
+  # ── PR REVIEWER EVIDENCE LANE. Branch protection intentionally requires only the
+  #    single `oya-ci-required` context, while GitHub Review API approvals are not merge
+  #    authority until `oya-pr-review` has a live trusted producer. This lane closes the
+  #    guardrail gap by validating the pull-request body itself on pull_request events:
+  #    merge-ready PRs must carry lead-owned `## Code Review` reviewer-agent evidence
+  #    with an APPROVE verdict, or the fan-in stays red. Non-PR events do not have a PR
+  #    body and pass through; the protected PR path is enforced before merge.
+  pr-reviewer-evidence:
+    name: pr-reviewer-evidence (PR body reviewer-agent APPROVE)
+    runs-on: ubuntu-latest
+    steps:
+      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
+      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
+        with:
+          persist-credentials: false
+      - name: Validate pull-request reviewer evidence
+        env:
+          EVENT_NAME: ${{ github.event_name }}
+          PR_BODY_PATH: ${{ runner.temp }}/pr-body.md
+        run: |
+          set -euo pipefail
+          if [ "${EVENT_NAME}" != "pull_request" ]; then
+            echo "pr-reviewer-evidence: ${EVENT_NAME} event has no PR body; pull_request path owns reviewer evidence enforcement."
+            exit 0
+          fi
+
+          python3 - <<'PY' > "${PR_BODY_PATH}"
+          import json
+          import os
+          import pathlib
+          import sys
+
+          event = json.loads(pathlib.Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
+          body = event.get("pull_request", {}).get("body")
+          if not body:
+              sys.stderr.write("pr-reviewer-evidence: pull_request.body is empty; reviewer-agent evidence missing\n")
+              sys.exit(1)
+          sys.stdout.write(body)
+          PY
+
+          cargo run --locked -p oya-dev-cli -- gate validate pr-traceability \
+            --pr-body "${PR_BODY_PATH}" \
+            --require-code-review
+
   # ── THE FAN-IN. This is the single required context branch protection keys on. It has NO
   #    command of its own (Principle 1 — it NEVER re-runs a narrower subset): it is green IFF
   #    every gate lane above is green. `needs:` lists EVERY gate job; the gate-registration
@@ -370,6 +420,7 @@ jobs:
       - generated-output-diff-policy # generated outputs are never PR merge surfaces
       - buck2                   # hermetic buck2 build + affected gate tests (alongside cargo)
       - app-shell-codegen       # ignored generated app-shell clients regenerate from canonical contracts
+      - pr-reviewer-evidence    # PR body carries reviewer-agent APPROVE evidence inside the required context
     steps:
       - name: Fan-in verdict (green IFF every gate lane is green)
         run: |
@@ -381,6 +432,7 @@ jobs:
           echo "  generated-diff    = ${{ needs.generated-output-diff-policy.result }}"
           echo "  buck2             = ${{ needs.buck2.result }}"
           echo "  app-shell-codegen = ${{ needs.app-shell-codegen.result }}"
+          echo "  reviewer-evidence = ${{ needs.pr-reviewer-evidence.result }}"
           # A matrix job's aggregate `.result` is success IFF ALL legs succeeded, failure if any
           # leg failed, skipped if it never ran — so one check covers every gate in the matrix.
           if [ "${{ needs.gate.result }}" = "success" ] \
@@ -389,7 +441,8 @@ jobs:
             && [ "${{ needs.gate-cloud-ci-firewall.result }}" = "success" ] \
             && [ "${{ needs.generated-output-diff-policy.result }}" = "success" ] \
             && [ "${{ needs.buck2.result }}" = "success" ] \
-            && [ "${{ needs.app-shell-codegen.result }}" = "success" ]; then
+            && [ "${{ needs.app-shell-codegen.result }}" = "success" ] \
+            && [ "${{ needs.pr-reviewer-evidence.result }}" = "success" ]; then
             echo "oya-ci-required: GREEN — every constituent gate lane passed."
             exit 0
           fi
exec
/bin/zsh -lc "git diff --numstat -- .github/workflows/oya-ci-required.yml; git log -n 12 --date=short --format='%h %ad %s' -- .github/workflows/oya-ci-required.yml infra/ci/buck2-affected-gate.sh infra/ci/materialize-cloud-ci-generated-faces.sh .github/workflows/docs-graph-drift.yml" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
54	1	.github/workflows/oya-ci-required.yml
16f2e3b54 2026-06-10 Add cloud-ci enforcement liveness gate (#669)
2c097d181 2026-06-10 Add cloud-ci target parity gate (#665)
2705d1c96 2026-06-10 feat: add cloud-ci freshness gate (#664)
5aaa68ab4 2026-06-10 ADR-0538: glob workspace members and add coverage gate (#662)
b049a777e 2026-06-10 ci: land safe async quick wins — drop cosmetic needs-edge, batch affected-gate owner query, dedupe buck2 build (#660)
15de7815a 2026-06-10 ci: stabilize buck-out cache key per dependency-set (stop per-commit multi-GB bloat) (#659)
f93a44f9b 2026-06-09 ci: productize generated output hygiene
acb8fab4e 2026-06-09 ci: productize SLO coverage gate
9fd579862 2026-06-09 ci: pin workflow actions to node24 release commits (#629)
e07d3e998 2026-06-09 refactor(ci): publish scm facts boundary contract (#613)
4d60dc441 2026-06-09 ci: add generated artifact final-tree control plane (#621)
9a509a623 2026-06-08 ci(cloud-ci): pre-provision pinned rust toolchain in the buck2 job (fix rustup race)
codex
A first hostile check found a material mismatch: the working-tree workflow is locally modified and does not currently contain the named `gate-live-postgres-*`, `catalog-liveness`, or 17-job topology. I’m resolving whether those timings describe another local commit/ref or a different workflow revision before ranking changes; otherwise attaching those seconds to this YAML would be false precision. The optimization logic will still be evaluated, but any non-matching timing will be marked as external input rather than locally verified evidence.
exec
/bin/zsh -lc "git for-each-ref --format='%(refname:short) %(objectname:short) %(committerdate:iso8601)' refs/heads refs/remotes | sort -k3,4r | sed -n '1,80p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
ci/contract-slice-dsl-fidelity f9530391d 2026-07-10 09:41:52 -0400
origin/ci/contract-slice-dsl-fidelity f9530391d 2026-07-10 09:41:52 -0400
origin/work-hermes-t_215c8b00-residency-001 37a98c3e8 2026-07-10 07:27:36 -0400
work-hermes-t_215c8b00-residency-001 37a98c3e8 2026-07-10 07:27:36 -0400
origin/work-hermes-t_b70d22bd-compliance-001 068e46cd0 2026-07-10 07:21:08 -0400
work-hermes-t_b70d22bd-compliance-001 068e46cd0 2026-07-10 07:21:08 -0400
origin/work-hermes-t_2ad09ed2-remed-002 175fda827 2026-07-10 07:00:18 -0400
work-hermes-t_2ad09ed2-remed-002 175fda827 2026-07-10 07:00:18 -0400
origin/work-hermes-t_d56e3446-talos-001 a2a221cb4 2026-07-10 06:50:40 -0400
work-hermes-t_d56e3446-talos-001 a2a221cb4 2026-07-10 06:50:40 -0400
origin/work-hermes-t_62cf60fe-release-001 47b98f318 2026-07-10 06:39:02 -0400
work-hermes-t_62cf60fe-release-001 47b98f318 2026-07-10 06:39:02 -0400
origin d7568c731 2026-07-10 05:45:36 -0400
origin/dev d7568c731 2026-07-10 05:45:36 -0400
origin/design/adr-0617-living-monorepo-graph ec9848232 2026-07-10 04:23:56 -0400
origin/hermes-t_04b1dc11-shell-currenthead 27654487c 2026-07-10 03:35:03 -0400
fix-frozen-pr1 39706fca8 2026-07-10 03:33:31 -0400
origin/feat/frozen-ref-decommit-pr1 39706fca8 2026-07-10 03:33:31 -0400
fix/doc-hygiene-1208 2f4696909 2026-07-10 02:59:01 -0400
origin/landing/doc-hygiene-1208 2f4696909 2026-07-10 02:59:01 -0400
wf-frozen-ref-decommit-pr1 f5a781607 2026-07-10 02:10:51 -0400
origin/work-hermes-t_1e67b1be-regeneration-test 6322852e2 2026-07-10 02:01:36 -0400
origin/work-hermes-t_c9dff045-authz002-current f9ca53b2c 2026-07-10 02:01:36 -0400
origin/work-hermes-t_6db1f3ef-billing-001 c8bddbf48 2026-07-10 02:01:24 -0400
worktree-wf_7e2155af-ba9-1 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_8bfcdc98-b6f-4 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_8bfcdc98-b6f-5 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_a6e35b25-9c4-1 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_a6e35b25-9c4-2 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_a6e35b25-9c4-3 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_a6e35b25-9c4-4 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_a6e35b25-9c4-5 76bb6ae11 2026-07-10 01:33:40 -0400
worktree-wf_a6e35b25-9c4-6 76bb6ae11 2026-07-10 01:33:40 -0400
origin/reorg/adr-0615-capability-boundary-rulings 01863a7aa 2026-07-10 01:13:50 -0400
feat/resilience-001-contract-slice-conversion 1ce27b233 2026-07-09 22:21:37 -0400
origin/feat/resilience-001-contract-slice-conversion 1ce27b233 2026-07-09 22:21:37 -0400
pr1315 1ce27b233 2026-07-09 22:21:37 -0400
feat/residency-001-contract-slice-conversion ab6656cd9 2026-07-09 22:11:56 -0400
feat/finops-001-contract-slice-conversion 10a34437b 2026-07-09 21:55:20 -0400
origin/feat/finops-001-contract-slice-conversion 10a34437b 2026-07-09 21:55:20 -0400
feat/cell-002-contract-slice-conversion 0b1eaa2ba 2026-07-09 19:21:23 -0400
origin/feat/cell-002-contract-slice-conversion 0b1eaa2ba 2026-07-09 19:21:23 -0400
feat/contract-slice-conformance-gate 35227b602 2026-07-09 18:13:05 -0400
origin/feat/contract-slice-conformance-gate 35227b602 2026-07-09 18:13:05 -0400
feat/rust-first-shrink-only-exceptions bad141165 2026-07-09 15:14:02 -0400
origin/feat/rust-first-shrink-only-exceptions bad141165 2026-07-09 15:14:02 -0400
work-hermes-t_cb12fdb1-obs-004 71ce2b9f8 2026-07-09 14:30:04 +0000
hermes-t_04b1dc11-shell-currenthead 5aa0c3e1f 2026-07-09 14:08:53 -0400
pull/1250/merge f4112f442 2026-07-09 13:57:31 +0000
work-hermes-t_1e67b1be-regeneration-test 27b123d63 2026-07-09 13:41:59 -0400
merge-t_78119d6f-pr1209 cea8f7125 2026-07-09 13:36:25 -0400
origin/work-hermes-t_a7a9ed48-dr-001 02d1fcf50 2026-07-09 13:29:55 -0400
work-hermes-t_a7a9ed48-dr-001 02d1fcf50 2026-07-09 13:29:55 -0400
landing/doc-hygiene-1208 4cd0c6d7d 2026-07-09 13:13:12 -0400
pr-1251-head 8fd857ccf 2026-07-09 13:12:49 -0400
origin/work-hermes-t_646222a9-finops-001 8f857de6c 2026-07-09 13:09:07 -0400
work-hermes-t_646222a9-finops-001 8f857de6c 2026-07-09 13:09:07 -0400
kanban/t_2e825791-adr0613 dff8909f2 2026-07-09 13:04:00 -0400
kanban/t_33235fb6-move-manifest-tempfile dff8909f2 2026-07-09 13:04:00 -0400
origin/work-hermes-t_318fd9e7-cell-002 0258def21 2026-07-09 12:32:24 -0400
work-hermes-t_318fd9e7-cell-002 0258def21 2026-07-09 12:32:24 -0400
work-hermes-t_6db1f3ef-billing-001 cf10f2622 2026-07-09 12:25:47 -0400
origin/work-hermes-t_c127bb35-resilience-001 931d331e5 2026-07-09 12:12:34 -0400
work-hermes-t_c127bb35-resilience-001 931d331e5 2026-07-09 12:12:34 -0400
sweep-pr-1285 8a9a7d410 2026-07-09 12:05:49 -0400
origin/work-hermes-t_f630eada-sov-001 b660cf071 2026-07-09 11:40:54 -0400
work-hermes-t_f630eada-sov-001 b660cf071 2026-07-09 11:40:54 -0400
work-hermes-t_c9dff045-authz002-current f0ef1c570 2026-07-09 11:23:50 -0400
sweep-pr-1287 5494344cf 2026-07-09 11:21:02 -0400
hermes-t_eeb5fcdf-runtime-001 d67deec6c 2026-07-09 11:03:34 -0400
work-hermes-t_ec8943b7-billing-002 d67deec6c 2026-07-09 11:03:34 -0400
backup/t_2ad09ed2-local-fixes-40c806 40c806553 2026-07-09 10:48:42 -0400
pr-sweep/1248 596c9684c 2026-07-09 09:58:32 -0400
pr-sweep/1250 44cfc1f46 2026-07-09 09:57:29 -0400
pr-sweep/1249 e69a150b1 2026-07-09 09:55:20 -0400
sweep-pr-1286 00e1d913d 2026-07-09 09:54:01 -0400
work-hermes-t_c1903853-ci-012 00e1d913d 2026-07-09 09:54:01 -0400
work-hermes-t_ada82788-audit-001 826bde3d9 2026-07-09 09:49:31 -0400
pr-sweep/1230 edfbbcfd2 2026-07-09 09:26:57 -0400
ci/path-resolver-fail-closed 3cc5cfd2a 2026-07-09 08:47:15 -0400
exec
/bin/zsh -lc "rg -n \"gate-live-postgres|catalog-liveness|producer-regen|registry-drift|cloud-ci-firewall|freshness \\(ADR-0539\\)\" .github/workflows ." in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git branch --show-current; git rev-parse HEAD; git remote -v; git branch -a --contains HEAD | sed -n '1,120p'" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git log --all -S'gate-live-postgres-facades' --format='%H %ad %s' --date=iso -- .github/workflows/oya-ci-required.yml; git log --all -S'catalog-liveness' --format='%H %ad %s' --date=iso -- .github/workflows/oya-ci-required.yml" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc 'git log --all --oneline --decorate -n 30 -- .github/workflows/oya-ci-required.yml' in /Users/jasonlee/Developer/oyatie
 succeeded in 333ms:
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/gate-disposition.json:2:  "_comment": "GO-LIVE disposition table (DATA, not code) for the Phase-0 firewall ratchet. Per (gate, code): the compare-MODE (baseline-block-on-new | advisory-until-infra), the infra_prereq that must land before an advisory code may flip to blocking, and frozen_empty for codes whose baseline may NEVER be non-empty. The producer's build_gate_baseline reads this table to stamp each per-code object in gate-baseline.generated.json; the cloud-ci-firewall runner reads the stamped modes. Flipping advisory-until-infra -> baseline-block-on-new when the infra lands is a reviewed DATA edit here, NOT a code change (honors the carve-outs-are-DATA doctrine in all four gate lib docs). The cloud-ci-brand-residue gate (register #25) is the forbidden-vocab shrink-only ratchet: each forbidden_<stem> code is baseline-block-on-new and freezes the CURRENT per-(stem,file) residue, so the firewall blocks any NEW occurrence (foundry/forgejo/jenkins/oya-vcs) while the frozen residue ages out without churning history. Carve-outs are DATA in libs/oya-check-brand-residue forbidden_vocab.",
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:4://! registry-drift-protected snapshot of SCM boundary facts emitted by the out-of-graph
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:15://! With `--stdout` one generated face is written to stdout (used by the registry-drift gate
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:151:    // Which face to emit to stdout: default registry. The gate self-tests + registry-drift
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:197:    // it current, and registry-drift byte-diffs it like the other faces.
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:523:/// the intentional bare `registry-drift` rust_test is not flagged.
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs:558:/// domain) so the intentional bare `registry-drift` rust_test is not flagged.
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:13://!    `registry-drift` test can byte-diff a fresh run against the committed face.
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:267:        // regen+commit can converge (registry-drift fixed-point). Emit None for the
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:308:            "GENERATED by oya-cloud-ci-accounting-registry-app. DO NOT HAND-EDIT — the registry-drift gate makes any hand-edit RED (committed==regenerated)."
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:414:             committed==regenerated (registry-drift byte-diffs it). DO NOT HAND-EDIT."
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:468:             committed==regenerated (registry-drift byte-diffs it). DO NOT HAND-EDIT."
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:489:pub const FIREWALL_TARGET: &str = "//cloud/cloud-ci/gates:oya-cloud-ci-firewall-app";
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:740:/// committed==regenerated holds byte-for-byte and the registry-drift gate can byte-diff it.
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:744:/// `ratchet_regression` caught by the cloud-ci-firewall runner, not by this builder.
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs:824:             (registry-drift byte-diffs it); a hand-edit to launder debt is itself registry_drift RED."
./cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/unit-class-policy.json:2:  "$comment": "DATA-TABLED carve-outs for the accounting-registry producer (Linus: the exception lives in the table, never in the scanner). Each rule maps a path predicate to a unit_class; the FIRST matching rule (top-to-bottom) wins, so order is significant. The classifier in producer code has ZERO special-case branches — it only walks this table. Adding/removing a carve-out is a DATA edit here, re-runnable by the producer; the registry-drift test makes any hand-edit to the generated face RED. unit_class enum (PHASE-0-FIREWALL-PLAN §5.1): code|doc|spec|registry|evidence|vendor|build_config|generated|ephemeral|husk.",
./cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/src/main.rs:6://! committed, content-addressed, registry-drift-protected `scm-facts.generated.json` face. The
./cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/BUCK:5:# registry-drift-protected scm-facts.generated.json face. The producer + every gate rust_test
./cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/Cargo.toml:12:# registry-drift-protected scm-facts.generated.json face. Run by the CI scm-facts-regen
./cloud/cloud-ci/gates/oya-cloud-ci-bnf-layer-suffix-app/src/lib.rs:261:        let input = rows(&["registry-drift"]);
./cloud/cloud-ci/gates/registry-drift/src/lib.rs:1://! # registry-drift
./cloud/cloud-ci/gates/registry-drift/BUCK:2:    name = "registry-drift",
./cloud/cloud-ci/gates/registry-drift/BUCK:10:# :registry-drift-gate — re-runs the producer in a sandbox and byte-diffs against the materialized
./cloud/cloud-ci/gates/registry-drift/BUCK:25:# package //cloud/cloud-ci/gates/registry-drift carries the §5.3 :registry-drift identity.)
./cloud/cloud-ci/gates/registry-drift/BUCK:27:    name = "registry-drift-gate",
./cloud/cloud-ci/gates/registry-drift/BUCK:36:        ":registry-drift",
./cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs:1:// :registry-drift gate — materialized == regenerated (PHASE-0-FIREWALL-PLAN §5.3).
./cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs:40:/// The materialized generated faces and the `--face` name that regenerates each. registry-drift
./cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs:184:/// DESIGN §1.4: scm-facts folds into the existing registry-drift tamper-evidence, no new trust
./cloud/cloud-ci/gates/registry-drift/Cargo.toml:2:name = "registry-drift"
./cloud/cloud-ci/gates/registry-drift/Cargo.toml:9:# :registry-drift (PHASE-0-FIREWALL-PLAN §5.3): re-runs the accounting-registry-producer
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/src/lib.rs:1://! # cloud-ci-firewall (GO-LIVE readiness ratchet)
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:2:    name = "oya-cloud-ci-firewall-app",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:13:    name = "oya-cloud-ci-firewall-app-unittest",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:23:# cloud-ci-firewall :oya-cloud-ci-firewall-app-gate — the single required GO-LIVE status check.
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:24:# Regenerates the gate-baseline over the live tree, byte-diffs nothing (registry-drift owns
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:29:    name = "oya-cloud-ci-firewall-app-gate",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:42:        ":oya-cloud-ci-firewall-app",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:54:    name = "oya-cloud-ci-firewall-app-gate-registration",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/BUCK:60:        ":oya-cloud-ci-firewall-app",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:2:  "_comment": "THE ONE-WAY DOOR. This is the ONLY human-edited, founder-signed file in the firewall ratchet \u2014 it is NOT producer-generated and is NOT byte-diffed by registry-drift. It is the sole exemption to the ratchet GROWTH check: a key listed under _sign_off_additions[gate][code] is exempted, for ONE regen, from the rule that the baseline may only shrink. Growing tolerated debt requires an explicit signed decision here, never a silent producer re-run (founder one-way-door / verify-each-step rule). Keep this tiny and audited. EMPTY = the ratchet is fully closed: the baseline can only shrink.",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:32:        "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/gate_registration.rs",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:43:        "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/gate_registration.rs",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:100:        "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:106:        "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:117:        "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json:124:        "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json",
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/firewall.rs:1:// cloud-ci-firewall — the single required GO-LIVE status check. Regenerates the gate
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/firewall.rs:35:    root.join("cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json")
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/firewall.rs:94:    root.join("specs/fixtures/cloud-ci-firewall")
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/firewall.rs:208:/// corpus. The committed baseline == the regenerated (proposed) baseline (registry-drift
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/firewall.rs:217:    // The committed baseline (byte-diff-protected by registry-drift).
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/gate_registration.rs:178:    // `gate-registry-drift`). We assert the gate's short identity appears in the fan-in block.
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/tests/gate_registration.rs:187:        // identity. (`registry-drift` is already short; others strip `oya-cloud-ci-`/`-app`.)
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/Cargo.toml:2:name = "oya-cloud-ci-firewall-app"
./cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/Cargo.toml:9:# cloud-ci-firewall (PHASE-0-FIREWALL-PLAN go-live readiness ratchet; register #20).
./cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/src/lib.rs:219:                    "member_path": "cloud/cloud-ci/gates/oya-cloud-ci-firewall-app",
 succeeded in 379ms:
+ ooo/auto_09f2434f66e7
+ ooo/auto_2c99a92feee5
+ ooo/auto_4578cb725072
* preserve/hermes-w1-dirty-20260630
  safety/t_1a0c9bde-20260702T130449Z
  safety/t_1a0c9bde-nested-.worktrees-t_7b437a23-calendar-build-20260702T131019Z
  safety/t_1a0c9bde-nested-.worktrees-workspace-drift-libs-specs-port-20260701-20260702T131019Z
+ work-hermes-t_64af26ae-w1-contract-009
+ work-hermes-t_702eeb17-w3-003
+ work-hermes-t_7b437a23-calendar-build
+ work-hermes-t_8f593a7e-w3-002
+ work-hermes-t_9574c74d-finops-ux-001a
+ work-hermes-t_bfcbdde5-obs002-runtime
+ work-hermes-t_c2adf6cd-contract-001
+ work-hermes-t_f52c70e7-sse-runtime
  wt/t_3e97188f
+ wt/t_817093be
  remotes/origin/preserve/hermes-w1-dirty-20260630
 succeeded in 394ms:
c78c50d63 feat(ci): paved-road contract-slice-conformance gate (retire scripts/tests/*_check.py) (#1309)
9875f2f85 feat(ci): paved-road contract-slice-conformance gate (retire scripts/tests/*_check.py)
1801128b1 feat(ci): capability-first keystone — cloud/cloud-ci → ci/facade + PathResolver + owned Cargo.lock lifecycle (#1216)
98b25dd10 (reorg/ci-keystone-move) fix(ci): explicit --find-renames for the generated-output diff-policy step
39e9c3a85 (origin/ooo/orch_eb9a182b0c47) feat(ci): close cloud-ci -> ci/facade keystone to firewall GO
018f3cac1 seed(masterplan-v2 v1.3.0): final-pass state from completed run orch_6e947c582056 — 6/6 ACs (sweep evidence, gate refinements, rust-first policy, oya-ci-required wiring)
2929693da fix(ci): fetch live PR title/body in metadata preflight instead of stale event payload (#1196)
d197094a6 plan(masterplan-v2): SSOT consolidation + fabric drive loop; four plan gates wired blocking into oya-ci-required (#1182)
87780e85a (fix/oya-ci-live-pr-body-admission) fix(ci): fetch live PR title/body in metadata preflight instead of stale event payload
0ae97fb4c Merge remote-tracking branch 'origin/dev' into ooo/orch_809bd09d8ad9
672a01bbf gate(masterplan-suite): wire the four plan gates' debt clean for oya-ci-required admission
4d97bd033 fix(ci): expose affected-set long-step telemetry
dfb523962 fix(ci): expose affected-set long-step telemetry
18c781a9f ci: add license-policy cloud-ci gate
46e28f9e0 Merge remote-tracking branch 'origin/dev' into work-hermes-t_c49c0324-ci-005-license-policy
cdc309531 feat(cloud-ci): migrate cloud resource validators to Rust gate
83cdfbefe Merge remote-tracking branch 'origin/dev' into work-hermes-t_c49c0324-ci-005-license-policy
31bfc3b29 (kanban/t_6437a710-rust-api-script-migration) Merge remote-tracking branch 'origin/dev' into kanban/t_6437a710-rust-api-script-migration
6aaeb6878 feat(cell): add topology manifest contract gate (#1130)
2e11076f6 (ci005-license-policy-fix-1151) ci: add license-policy cloud-ci gate
03ff21672 fix(cell): wire topology manifest gate into ci
0a7cdc564 feat(cloud-ci): migrate cloud resource validators to Rust gate
d39c5c42f fix(ci): require rust-first infra automation hygiene
964148b39 fix(ci): require rust-first infra automation hygiene
fc469ffae (work-hermes-t_5a241375-infra-anti-pattern-checks) fix(ci): require rust-first infra automation hygiene
4ad7033a6 (origin/gjc/long-step-telemetry-901) Productize long-step CI telemetry
ac80d6ab2 Split live-postgres required sublanes (#1036)
f2d40cf81 Split live-postgres required sublanes
354bc4e58 Enforce PR review admission packet (#1033)
f0f5586dc (gjc/review-admission-983) Enforce PR review admission packet
 succeeded in 468ms:
ac80d6ab2882dc54a199d5e050be5a392ea50132 2026-06-30 01:40:14 -0400 Split live-postgres required sublanes (#1036)
f2d40cf81ef9181b762c8e8071f2139eebfd199f 2026-06-30 00:06:02 -0400 Split live-postgres required sublanes
1801128b16a0fe6318e84c1939d6e0937a677708 2026-07-09 00:08:36 -0400 feat(ci): capability-first keystone — cloud/cloud-ci → ci/facade + PathResolver + owned Cargo.lock lifecycle (#1216)
39e9c3a85a834dd9bba9e49b8b8dc73086e5b40f 2026-07-08 16:45:29 -0400 feat(ci): close cloud-ci -> ci/facade keystone to firewall GO
73ec90e9eadb3854eed71957c77a95f8fe5fd43e 2026-06-18 14:24:21 -0400 G011: PR-C3 catalog-liveness gate — live-OR-marked born-blocking (empty baseline) (#752)
exec
/bin/zsh -lc 'git show origin/dev:.github/workflows/oya-ci-required.yml | rg -n "''^  [a-zA-Z0-9_-]+:|runs-on:|install-buck2|rustup|materialize-cloud-ci|actions/cache|buck-out|gate-live-postgres|catalog-liveness|producer-regen|registry-drift|freshness '"\\(|continue-on-error|rdeps|buck2 test|needs:\"" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
23:  workflow_dispatch:
24:  push:
26:  pull_request:
29:  merge_group:
32:  contents: read
33:  actions: read
42:  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
43:  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
50:  producer-regen:
51:    name: producer-regen (accounting-registry)
52:    runs-on: ubuntu-latest
65:        run: infra/ci/install-buck2.sh
67:        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
87:  #    `buck2 test //ci/facade/<crate>:{ci-<crate>-unittest,ci-<crate>-gate}` — so instead of copy-pasting a job per gate, a single
102:  #    itself, so each leg downloads the producer-regen artifact instead of paying its own
105:  #    The regeneration-checking gates KEEP materializing independently: registry-drift (the
106:  #    byte-parity detector — detectors never consume the thing they attest), producer-regen
108:  #    ADR-0551). `needs: producer-regen` serializes these legs behind a ~75s producer job;
109:  #    the workflow critical path (affected-set/buck2 lanes) is unaffected. If producer-regen
114:  gate:
115:    needs: producer-regen
120:    runs-on: ubuntu-latest
134:          - { crate: service-catalog-parity,        label: "gate · catalog-liveness (PR-C3 founder live-OR-explicitly-marked policy, born-blocking, EMPTY frozen baseline)" }
172:      # Consume the producer-regen artifact (faces + volatile scm snapshot) instead of
175:      # tree; registry-drift separately proves that derivation is byte-deterministic.
176:      - name: Download regenerated faces (producer-regen artifact, ADR-0556 D5 QW-1)
183:        run: infra/ci/install-buck2.sh
185:      # the test action starts to avoid concurrent rustup writes inside parallel Buck2 actions.
189:          rustup toolchain install
191:      - name: buck2 test ${{ matrix.crate }}
194:          buck2 test \
201:  gate-generated-artifact-freshness:
202:    name: freshness (lock + generated faces, ADR-0539)
203:    runs-on: ubuntu-latest
211:        run: infra/ci/install-buck2.sh
213:      # version-pinned input, not a build output). rustup still resolves/validates the
214:      # toolchain on every run; the cache only pre-seeds ~/.rustup so the install is a no-op.
216:        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
217:        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
220:            ~/.rustup/toolchains
221:            ~/.rustup/update-hashes
222:          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
226:          rustup toolchain install
234:  # ── registry-drift: materialized workspace == regenerated byte-equal. Starts at t=0 alongside
235:  #    producer-regen; it rematerializes in-job so it is hermetic and self-contained. The
236:  #    producer-regen needs-edge was cosmetic (evidence only, nothing consumed) and serialized
238:  gate-inventory-registry-drift:
239:    name: registry-drift (materialized == regenerated)
240:    runs-on: ubuntu-latest
253:        run: infra/ci/install-buck2.sh
256:      # feeding it the producer-regen artifact it is supposed to verify would make the
260:          rustup toolchain install
263:          buck2 test //ci/facade/inventory-registry-drift:ci-inventory-registry-drift-gate
269:  gate-baseline-ratchet:
271:    runs-on: ubuntu-latest
284:        run: infra/ci/install-buck2.sh
287:      # out-of-band bootstrap ref, and is deliberately absent from the producer-regen artifact.
291:        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
292:      - name: buck2 test cloud-ci-firewall
295:          buck2 test \
307:  generated-output-diff-policy:
309:    runs-on: ubuntu-latest
317:        run: infra/ci/install-buck2.sh
318:      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
321:        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
322:        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
325:            ~/.rustup/toolchains
326:            ~/.rustup/update-hashes
327:          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
331:          rustup toolchain install
355:  #    target (the env!CARGO eradication) and `buck2 test` runs the gate rust_tests fully
359:  #    affected-set driver (`infra/ci/buck2-affected-gate.sh`: uquery owner -> rdeps closure,
363:  buck2:
365:    runs-on: ubuntu-latest
378:        run: infra/ci/install-buck2.sh
379:      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
382:        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
383:        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
386:            ~/.rustup/toolchains
387:            ~/.rustup/update-hashes
388:          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
390:      # The buck2 rust toolchain (toolchains/BUCK: system_rust_toolchain via the rustup shim)
392:      # PARALLEL. On a cold runner each action's first shim call triggers rustup to install the
393:      # rust-toolchain.toml channel (+ rustfmt,clippy) concurrently — the racing rustup
394:      # processes collide on the shared `~/.rustup/downloads/*.partial` files and fail with
395:      # `rustup::utils::rename ... No such file or directory (os error 2)` (a different component
396:      # each run: clippy, then a second toolchain component — proving a concurrency race, not a config defect). rustup
399:      - name: Pre-provision pinned rust toolchain (serialize rustup before parallel buck2)
402:          rustup toolchain install
404:      # Restore buck-out across runs so ephemeral runners start warm (design §3.1 / ADR-0515 D4).
407:      # ./buck-out — buck-out/v2/cache/{materializer_state,incremental_state}/db.sqlite plus
408:      # buck-out/v2/art (the materialized action outputs). buck-out is PATH-RELOCATABLE (relative
412:      # caching `path: buck-out` warms everything cacheable and ~/.buck2/~/.buck is DELIBERATELY
418:      # push, so the saved buck-out is the full-graph superset. This buck2 job is READ-ONLY on every
419:      # trigger — it restores the full-graph buck-out and its //ci/... build is a subset
426:      # every commit, so actions/cache SAVED a fresh full buck-out (multi-GB) on EVERY run and
438:      # ~25-30 GiB) BEFORE the multi-GB buck-out restore. This lane decompresses a ~5.78 GiB buck-out
442:      # oya/buck2 action consumes; touches NO repo content and NO cache (buck-out / ~/.rustup / the
463:      - name: Restore buck-out (read-only; dev-push is the sole writer)
464:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
465:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
467:          path: buck-out
468:          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
470:            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
477:      # by design (ADR-0551) and deliberately absent from the producer-regen artifact; this
492:          # buck2 test builds its targets before running them, so a standalone
497:          buck2 test //ci/... --unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json
535:  #    pull_request it derives the merge-base diff's owner()/rdeps() cone and builds+tests it;
543:  gate-affected-target-set:
545:    runs-on: ubuntu-latest
560:        run: infra/ci/install-buck2.sh
561:      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
564:        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
565:        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
568:            ~/.rustup/toolchains
569:            ~/.rustup/update-hashes
570:          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
571:      # Same rustup serialization rationale as the buck2 lane above: parallel buck2 actions
572:      # racing a cold rustup install collide on shared download state.
587:          rustup toolchain install
598:      # Restore buck-out read-only (ADR-0554 D9). Same stable per-dependency/toolchain-set key as the buck2
599:      # lane: warmth is 100% in buck-out/v2/cache + buck-out/v2/art (path-relocatable); ~/.buck2
604:      # ~25-30 GiB) BEFORE the multi-GB buck-out restore. This lane decompresses a ~5.78 GiB buck-out
608:      # oya/buck2 action consumes; touches NO repo content and NO cache (buck-out / ~/.rustup / the
629:      - name: Restore buck-out (read-only; dev-push is the sole writer)
630:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
631:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
633:          path: buck-out
634:          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
636:            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
652:      #    is required, materialize that baseline IN THE MAIN ROOT so it shares the warm ./buck-out
653:      #    restored above (the merge-base IS a dev commit, so the dev-keyed buck-out is near-fully
664:      #    baseline; the report reaches the verdict ONLY via --baseline-report. The warm ./buck-out
810:              echo "build-health baseline: cleaning buck-out after restoring candidate toolchain ${candidate_toolchain}"
822:          rustup toolchain install
824:            echo "build-health baseline: Rust toolchain changed ${baseline_toolchain} -> ${candidate_toolchain}; isolating buck-out"
827:          # Build the whole merge-base workspace keep-going. Same-channel PRs share warm ./buck-out;
873:      #    --mode full (buck2 build + test //...), so buck-out is populated with the FULL workspace
875:      #    gate-affected-target-set restores a full-graph buck-out, so the same-root merge-base baseline
876:      #    build is near-fully-warm (the merge-base IS a recent dev commit whose full-graph buck-out
878:      #    one job, one key — no two-writer race. Runs AFTER the Binding step (buck-out fully
880:      #    Size note: the full-graph buck-out is one blob per stable key (overwrites, non-accumulating
883:      - name: Save buck-out (dev-push only; sole canonical full-graph writer)
885:        # actions/cache/save@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
886:        uses: actions/cache/save@27d5ce7f107fe9357f9df03efb73ab90386fccae
888:          path: buck-out
889:          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
930:  gate-live-postgres-adapters:
931:    name: "gate-live-postgres-adapters (durable adapters: RLS / CDC / SCIM, #901)"
932:    runs-on: ubuntu-latest
957:        run: infra/ci/install-buck2.sh
959:        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
960:        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
963:            ~/.rustup/toolchains
964:            ~/.rustup/update-hashes
965:          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
969:          rustup toolchain install
997:            "gate_id": "gate-live-postgres-adapters",
1026:      - name: buck2 test — durable adapters (admin=superuser, app=app-role)
1051:          buck2 test --local-only --num-threads 1 //libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest -- "${LIVE_ENV[@]}"
1052:          buck2 test --local-only --num-threads 1 //libs/oya-data-outbox-adapter-postgres:oya-data-outbox-adapter-postgres-unittest -- "${LIVE_ENV[@]}"
1053:          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-unittest -- "${LIVE_ENV[@]}"
1054:          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-live -- "${LIVE_ENV[@]}"
1055:          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-unittest -- "${LIVE_ENV[@]}"
1056:          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-live -- "${LIVE_ENV[@]}"
1067:  gate-live-postgres-facades:
1068:    name: "gate-live-postgres-facades (durable facades: tenant lifecycle / SCIM, #901)"
1069:    runs-on: ubuntu-latest
1094:        run: infra/ci/install-buck2.sh
1096:        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
1097:        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
1100:            ~/.rustup/toolchains
1101:            ~/.rustup/update-hashes
1102:          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
1106:          rustup toolchain install
1134:            "gate_id": "gate-live-postgres-facades",
1163:      - name: buck2 test — durable facades (live test = app-role, non-live = in-memory)
1174:          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-tests -- "${FACADE_ENV[@]}"
1175:          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-e2e -- "${FACADE_ENV[@]}"
1176:          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-unittest -- "${FACADE_ENV[@]}"
1177:          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-acceptance -- "${FACADE_ENV[@]}"
1192:  #    every gate lane above is green. `needs:` lists EVERY gate job; the gate-registration
1194:  oya-ci-required:
1196:    runs-on: ubuntu-latest
1198:    needs:
1201:      - gate-inventory-registry-drift     # bespoke: materialized == regenerated byte-parity
1206:      - gate-live-postgres-adapters # #901: durable adapter RLS / CDC / SCIM tests against isolated live Postgres
1207:      - gate-live-postgres-facades  # #901: durable facade tenant lifecycle / SCIM tests against isolated live Postgres
1214:          echo "  registry-drift    = ${{ needs.gate-inventory-registry-drift.result }}"
1219:          echo "  live-postgres/adapters = ${{ needs.gate-live-postgres-adapters.result }}"
1220:          echo "  live-postgres/facades  = ${{ needs.gate-live-postgres-facades.result }}"
1225:            && [ "${{ needs.gate-inventory-registry-drift.result }}" = "success" ] \
1230:            && [ "${{ needs.gate-live-postgres-adapters.result }}" = "success" ] \
1231:            && [ "${{ needs.gate-live-postgres-facades.result }}" = "success" ]; then
exec
/bin/zsh -lc 'git show origin/dev:.github/workflows/oya-ci-required.yml | wc -l; git show origin/dev:.github/workflows/docs-graph-drift.yml | wc -l' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
    1236
      87
exec
/bin/zsh -lc 'git diff --stat HEAD..origin/dev -- .github/workflows/oya-ci-required.yml .github/workflows/docs-graph-drift.yml infra/ci/install-buck2.sh infra/ci/buck2-affected-gate.sh infra/ci/materialize-cloud-ci-generated-faces.sh' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
 .github/workflows/docs-graph-drift.yml           |   78 +-
 .github/workflows/oya-ci-required.yml            | 1075 +++++++++++++++++++---
 infra/ci/install-buck2.sh                        |   23 +-
 infra/ci/materialize-cloud-ci-generated-faces.sh |   28 -
 4 files changed, 1017 insertions(+), 187 deletions(-)
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/docs-graph-drift.yml | nl -ba | sed -n '1,180p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	name: docs-graph-drift
     2	
     3	# Generator health check for the architecture dashboard (docs/architecture/product-graph.html).
     4	#
     5	# The dashboard is GENERATED from SSOT by an owned-Rust CI pipeline tool
     6	# (tools/oya-architecture-graph-generator-app) — NOT a developer-facing `oya`
     7	# subcommand. As of ADR-0613 the dashboard AND its masterplan input are DE-COMMITTED
     8	# (materialization_mode: not-tracked-in-git); byte-parity against a committed HTML copy
     9	# is intentionally RETIRED. Dashboard freshness/determinism is now enforced by the
    10	# REQUIRED generated-artifact-freshness gate (regenerate-twice determinism canary over the
    11	# controller-materialized path). This job's remaining role is to build the generator and run
    12	# its owned golden/regeneration tests — materializing the de-committed masterplan input first,
    13	# since it is absent from a fresh checkout.
    14	#
    15	# Transitional runner model: GitHub Actions executes this feedback adapter today,
    16	# but the durable policy source is the owned Rust generator/gate and future cloud-ci
    17	# runner. Intentionally ABSENT from the branch-protection required set
    18	# (.github/branch-protection.yaml) — feedback only; branch protection is NOT
    19	# changed here.
    20	
    21	on:
    22	  pull_request:
    23	    paths:
    24	      # product-graph.html + masterplan.generated.json are de-committed (ADR-0613, untracked) and
    25	      # can no longer appear in a PR file list; trigger only on the generator + its tracked inputs.
    26	      - ".github/workflows/docs-graph-drift.yml"
    27	      - "tools/oya-architecture-graph-generator-app/**"
    28	      - "docs/architecture/product-graph.template.html"
    29	      - "docs/machine-readable/architecture-graph.json"
    30	  push:
    31	    branches: [dev]
    32	    paths:
    33	      - ".github/workflows/docs-graph-drift.yml"
    34	      - "tools/oya-architecture-graph-generator-app/**"
    35	      - "docs/architecture/product-graph.template.html"
    36	      - "docs/machine-readable/architecture-graph.json"
    37	
    38	permissions:
    39	  contents: read
    40	
    41	concurrency:
    42	  group: docs-graph-drift-${{ github.workflow }}-${{ github.head_ref || github.run_id }}
    43	  cancel-in-progress: true
    44	
    45	jobs:
    46	  docs-graph-drift:
    47	    name: docs-graph-drift
    48	    runs-on: ubuntu-latest
    49	    timeout-minutes: 15
    50	    steps:
    51	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    52	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
    53	        with:
    54	          persist-credentials: false
    55	          # fetch-depth: 0 so merge-base(HEAD, origin/dev) resolves — the materializer's
    56	          # landed-plan carve-out needs it to exclude the committed move-plans (matches the
    57	          # required legs; a shallow checkout would fail-closed on the >1-plan guard).
    58	          fetch-depth: 0
    59	      - name: Install pinned Rust toolchain
    60	        uses: dtolnay/rust-toolchain@21dc36fb71dd22e3317045c0c31a3f4249868b17
    61	        with:
    62	          toolchain: 1.96.0
    63	      - name: Cache Buck2 official prebuilt
    64	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    65	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
    66	        with:
    67	          path: /tmp/oya-ci-buck2-2026-06-01
    68	          # The cached asset is always SHA-256 verified by infra/ci/install-buck2.sh before execution.
    69	          key: docs-graph-drift-${{ runner.os }}-buck2-2026-06-01-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555
    70	          restore-keys: |
    71	            docs-graph-drift-${{ runner.os }}-buck2-2026-06-01-
    72	      - name: Materialize de-committed inputs, build + test the generator
    73	        run: |
    74	          set -euo pipefail
    75	          infra/ci/install-buck2.sh
    76	          export PATH="/tmp/oya-ci-buck2-2026-06-01:${PATH}"
    77	          rustc --version
    78	          # masterplan.generated.json (ADR-0613 de-commit) is absent from a fresh checkout;
    79	          # materialize it (and the other on-demand faces) from SSOT before the generator and its
    80	          # golden/regeneration tests consume it. Dashboard freshness/determinism is enforced by the
    81	          # required generated-artifact-freshness gate (regenerate-twice determinism canary), not by a
    82	          # byte-parity-against-committed check here (retired with the de-commit).
    83	          buck2 run root//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
    84	          buck2 build root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator
    85	          buck2 test \
    86	            root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator-regeneration-test \
    87	            root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator-golden-test
exec
/bin/zsh -lc 'for p in infra/ci/install-buck2.sh infra/ci/buck2-affected-gate.sh infra/ci/materialize-cloud-ci-generated-faces.sh; do git cat-file -e origin/dev:$p 2>/dev/null && echo "present $p" || echo "absent $p"; done' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
present infra/ci/install-buck2.sh
present infra/ci/buck2-affected-gate.sh
absent infra/ci/materialize-cloud-ci-generated-faces.sh
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | awk 'BEGIN{in_jobs=0} /"'^jobs:/{in_jobs=1; next} in_jobs && /''^  [A-Za-z0-9_-]+:$/ {line=$0; sub(/''^  /,"",line); sub(/:$/,"",line); print NR ":" line}'"'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
50:producer-regen
114:gate
201:gate-generated-artifact-freshness
238:gate-inventory-registry-drift
269:gate-baseline-ratchet
307:generated-output-diff-policy
363:buck2
543:gate-affected-target-set
930:gate-live-postgres-adapters
1067:gate-live-postgres-facades
1194:oya-ci-required
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | awk '/"'^[[:space:]]+- '"\\{ crate:/ {n++; print NR \":\" "'$0} END {print "matrix_entries=" n}'"'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
125:          - { crate: cross-artifact-agreement, label: "gate · cross-artifact-agreement (GATE-1, incl. the 4 masterplan-v2 plan gates: structural ID/DAG/orphans · projection-freshness · plan-vs-evidence · read-contract/entry-surface)" }
126:          - { crate: artifact-accountability,         label: "gate · total-accounting (GATE-2)" }
127:          - { crate: stale-artifact-detection,         label: "gate · staleness-reaper (GATE-3, born-blocking)" }
128:          - { crate: automation-coverage,       label: "gate · automation-ratchet (GATE-4, polices gates)" }
129:          - { crate: crate-layer-suffix,         label: "gate · bnf-layer-suffix (BNF §2.5#4, born-blocking)" }
130:          - { crate: package-manifest-hygiene,         label: "gate · manifest-hygiene (§2.5#7, born-blocking)" }
131:          - { crate: crate-name-prefix,            label: "gate · cargo-prefix (ADR-0017)" }
132:          - { crate: slo-coverage,            label: "gate · slo-coverage (catalog SLO input contract, born-blocking)" }
133:          - { crate: license-policy,          label: "gate · license-policy (workspace package license policy, shrink-only)" }
134:          - { crate: service-catalog-parity,        label: "gate · catalog-liveness (PR-C3 founder live-OR-explicitly-marked policy, born-blocking, EMPTY frozen baseline)" }
135:          - { crate: workspace-member-coverage, label: "gate: workspace-glob-coverage (ADR-0538)" }
136:          - { crate: build-target-parity,           label: "gate · target-parity (ADR-0540, test-wiring false-green)" }
137:          - { crate: hook-wiring,    label: "gate · enforcement-liveness (FRIC-012, hook mirror liveness)" }
138:          - { crate: action-item-accounting,     label: "gate · friction-accounting (ADR-0544, closed-loop friction-ledger accounting)" }
139:          - { crate: canonical-json,          label: "gate · canonical-json (ADR-0546, deterministic JSON serialization)" }
140:          - { crate: parity-claim-evidence, label: "gate · hyperscaler-parity-taxonomy (cloud hyperscaler parity taxonomy, born-blocking)" }
141:          - { crate: resource-contract-conformance, label: "gate · cloud-resource-contracts (Rust/API replacement for P0 cloud-resource Python validators)" }
142:          - { crate: contract-slice-conformance, label: "gate · contract-slice-conformance (paved-road Rust/Buck2 replacement for scripts/tests/*_check.py contract-slice validators; ADR-0515/0523/0528)" }
143:          - { crate: embedded-asset-hermeticity, label: "gate · embedded-asset-hermeticity (ADR-0545, include_str!/include_bytes! __srcs-tree mapping)" }
144:          - { crate: core-dependency-isolation,           label: "gate · kernel-purity (ADR-0547, *-kernel/*-core zero transient-tech deps)" }
145:          - { crate: crypto-backend-policy,   label: "gate · crypto-backend-purity (ADR-0506, ring forbidden / aws-lc-rs mandated — zero ring activation)" }
146:          - { crate: graphql-usage-policy,  label: "gate · no-graphql-without-adr (ADR-0565, zero-GraphQL: no graphql lib / .graphql/.gql/.sdl reintroduction without a reversing ADR — candidate-tree evaluated, EMPTY frozen baseline)" }
147:          - { crate: endpoint-authorization-coverage,          label: "gate · authz-coverage (issue #770 / AUTH-005, NEW unauthenticated HTTP control planes blocked vs frozen baseline)" }
148:          - { crate: caller-supplied-authorization,         label: "gate · dto-authz-trust (ADR-0582, the CLASS-FIX for caller-supplied-authz-trust: a NEW fn that trusts a forged *Authorization DTO / x-authorization-* header in place of a server-side PDP decide() is blocked vs the frozen baseline of ~92 known instances; v2: FN-01/02/03/04/05/06 hardened)" }
149:          - { crate: generated-artifact-policy, label: "gate · generated-artifact-control-plane (public hermetic CI artifact policy)" }
150:          - { crate: build-cache-policy,            label: "gate · cache-wiring conformance (ADR-0560/ADR-0556, dark-wiring + cold floor + kill-switch)" }
151:          - { crate: dependency-graph-acyclicity, label: "gate · substrate-dependency-dag-acyclicity (ADR-0280 §D-3, Tarjan SCC + Kahn topo-order + forbidden-edge honouring)" }
152:          - { crate: service-tier-metadata,     label: "gate · tier-field-coverage (Phase-0 reorg ADR-0562/0536/0245, per-service tier/tier_subtype/dr_tier coverage + enum validity + no type-overload, born-blocking)" }
153:          - { crate: layer-dependency-acyclicity, label: "gate · tier-dependency-acyclicity (Phase-0 reorg ADR-0245/0280/0562, cargo+buck crate-graph tier rules + S-rank + Tarjan cycle backstop, born-ADVISORY vs frozen baseline → enforce-no-regression)" }
154:          - { crate: module-membership,   label: "gate · capability-membership (Phase-0 reorg ADR-0562 §6, the anti-junk-drawer MEMBERSHIP lint: every crate → exactly one registered capability/meta home, no NEW unmapped crate, no NEW top-level dir, base/-admission; born-advisory + enforce-no-regression vs the frozen unmapped baseline)" }
155:          - { crate: runner-disk-reclaim,     label: "gate · runner-disk-reclaim conformance (FRIC-017 productization, ADR-0548 pipeline-as-product: policy parse + threshold/INFRA-RED discrimination + reclaim plan)" }
156:          - { crate: port-placement,          label: "gate · port-placement (ADR-0570, clean-arch ports-in-core: no storage-port trait DEFINED in an */adapters/* crate — productizes the #116 defect class; born-advisory + enforce-no-regression vs frozen baseline)" }
157:          - { crate: repo-root-hygiene,  label: "gate · root-workspace-hygiene (ADR-0600, allowlist-as-DATA default-DENY: every TRACKED repo-root file must match the allowlist + every top-level dir must be a permitted capability/meta home — makes committed root scratch structurally impossible; complements the scratch DENYLIST)" }
158:          - { crate: dependency-automation, label: "gate · dependency-automation (ADR-0535, owned oya-deps.toml Rust bump-bot contract; external bot configs remain absent)" }
159:          - { crate: supply-chain-audit,    label: "gate · supply-chain-audit (owned RustSec advisory scan over vendored mirror, born-blocking)" }
160:          - { crate: feature-maturity-policy, label: "gate · planned-maturity (GH #992, product PRD acceptance/verification contracts + rich capability records + retired-plan provenance boundary)" }
161:          - { crate: operator-secret-rbac, label: "gate · operator-secret-bootstrap (GH #980 + GH #988 / ADR-0606, least-privilege secret RBAC, declarative join-token bootstrap, ESO/OpenBao role+namespace+prefix scope, and plaintext OpenBao NetworkPolicy isolation)" }
162:          - { crate: policy-deploy-parity, label: "gate · cedar-deploy-parity (GH #16 / ADR-0608, deployed-vs-authored Cedar parity: no deployed ConfigMap permit may leave the action unconstrained (action-agnostic blanket grant) and every deployed permit MUST be ⊆ the capability's authored <cap>/{policy,cedar}/*.cedar set; fail-closed on missing-authored/un-extractable/empty-scan; GH #16 byte-identical blanket ConfigMaps grandfathered in a documented shrink-only baseline pending the blanket-disarm follow-up)" }
163:          - { crate: topology-manifest-contract, label: "gate · cell-topology-manifest-contract (CELL-001R manifest contract)" }
164:          - { crate: automation-language-policy, label: "gate · rust-first automation hygiene (cloud-native infra anti-patterns: scripts, workflow shell, retired interpreters, and new CLI packages)" }
165:          - { crate: gate-self-conformance, label: "gate · gate-self-conformance (GH #777, pipeline-as-product 7-property bar over every gate)" }
matrix_entries=41
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | rg -n \"infra/ci/install-buck2\\.sh|rustup toolchain install|oya-cloud-ci-materialize-generated-faces-bin|actions/cache/(restore|save)?@|path: buck-out|runs-on:\"" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
52:    runs-on: ubuntu-latest
65:        run: infra/ci/install-buck2.sh
67:        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
120:    runs-on: ubuntu-latest
183:        run: infra/ci/install-buck2.sh
189:          rustup toolchain install
203:    runs-on: ubuntu-latest
211:        run: infra/ci/install-buck2.sh
226:          rustup toolchain install
240:    runs-on: ubuntu-latest
253:        run: infra/ci/install-buck2.sh
260:          rustup toolchain install
262:          buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
271:    runs-on: ubuntu-latest
284:        run: infra/ci/install-buck2.sh
291:        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
309:    runs-on: ubuntu-latest
317:        run: infra/ci/install-buck2.sh
331:          rustup toolchain install
365:    runs-on: ubuntu-latest
378:        run: infra/ci/install-buck2.sh
402:          rustup toolchain install
412:      # caching `path: buck-out` warms everything cacheable and ~/.buck2/~/.buck is DELIBERATELY
464:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
465:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
467:          path: buck-out
480:        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
545:    runs-on: ubuntu-latest
560:        run: infra/ci/install-buck2.sh
587:          rustup toolchain install
630:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
631:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
633:          path: buck-out
642:        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
822:          rustup toolchain install
885:        # actions/cache/save@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
886:        uses: actions/cache/save@27d5ce7f107fe9357f9df03efb73ab90386fccae
888:          path: buck-out
932:    runs-on: ubuntu-latest
957:        run: infra/ci/install-buck2.sh
969:          rustup toolchain install
1069:    runs-on: ubuntu-latest
1094:        run: infra/ci/install-buck2.sh
1106:          rustup toolchain install
1196:    runs-on: ubuntu-latest
exec
/bin/zsh -lc "git show origin/dev:infra/ci/install-buck2.sh | nl -ba; git show origin/dev:infra/ci/buck2-affected-gate.sh | nl -ba | sed -n '1,260p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	#!/usr/bin/env bash
     2	# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
     3	set -euo pipefail
     4	
     5	BUCK2_RELEASE="${BUCK2_RELEASE:-2026-06-01}"
     6	BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
     7	
     8	case "$(uname -s)-$(uname -m)" in
     9	  Linux-x86_64)
    10	    BUCK2_ASSET="${BUCK2_ASSET:-buck2-x86_64-unknown-linux-gnu.zst}"
    11	    BUCK2_SHA256="${BUCK2_SHA256:-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555}"
    12	    ;;
    13	  *)
    14	    if [ "${OYA_CI_ALLOW_AMBIENT_BUCK2:-}" = "1" ] && command -v buck2 >/dev/null 2>&1; then
    15	      echo "Using ambient buck2 only because OYA_CI_ALLOW_AMBIENT_BUCK2=1 was set." >&2
    16	      buck2 --version
    17	      exit 0
    18	    fi
    19	    echo "Unsupported host for default pinned Buck2 install; set OYA_CI_ALLOW_AMBIENT_BUCK2=1 for local advisory use." >&2
    20	    exit 1
    21	    ;;
    22	esac
    23	
    24	mkdir -p "${BUCK2_INSTALL_DIR}"
    25	
    26	# Cache-hit fast path (ADR-0556 D5 QW-4: the tool binary is a digest-pinned INPUT, not a build
    27	# output — warm-eligible velocity). If the compressed release asset is already present (e.g.
    28	# restored by actions/cache) and its bytes match the pinned SHA-256, skip the network download.
    29	# A present-but-mismatching asset is discarded and re-downloaded.
    30	asset_path="${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}"
    31	if [ -f "${asset_path}" ] \
    32	  && echo "${BUCK2_SHA256}  ${asset_path}" | sha256sum -c - >/dev/null 2>&1; then
    33	  echo "buck2 release asset cache hit (SHA-256 verified): ${asset_path} — skipping download." >&2
    34	else
    35	  rm -f "${asset_path}"
    36	  curl --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 20 -fsSL "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/${BUCK2_ASSET}" -o "${asset_path}"
    37	fi
    38	
    39	# Integrity is non-negotiable (ADR-0556: the SHA check is the integrity anchor that makes the
    40	# warm path admissible). The pinned-digest verification ALWAYS runs on the exact bytes about to
    41	# be decompressed and executed — cached and fresh paths alike — and the executable is ALWAYS
    42	# re-derived from those verified bytes (never trusted as a loose cached binary).
    43	echo "${BUCK2_SHA256}  ${asset_path}" | sha256sum -c -
    44	zstd -d -f "${asset_path}" -o "${BUCK2_INSTALL_DIR}/buck2"
    45	chmod +x "${BUCK2_INSTALL_DIR}/buck2"
    46	
    47	if [ -n "${GITHUB_PATH:-}" ]; then
    48	  echo "${BUCK2_INSTALL_DIR}" >> "${GITHUB_PATH}"
    49	fi
    50	
    51	"${BUCK2_INSTALL_DIR}/buck2" --version
     1	#!/bin/sh
     2	# buck2-native affected-only CI gate.
     3	#
     4	# Builds + tests the reverse-dependency closure of the PR's changed files —
     5	# the hyperscaler "affected targets" pattern (Google/Meta), buck2-native via
     6	# `uquery owner()` + `rdeps()`. Replaces the cargo-era `oya verify --affected`.
     7	# No oya-dev-cli dependency.
     8	#
     9	# Usage:  buck2-affected-gate.sh <base-ref> [head-ref]
    10	#         base-ref  — the merge-base anchor (e.g. origin/dev)
    11	#         head-ref  — the tip to diff (default: HEAD)
    12	#
    13	# The 1-arg form (buck2-affected-gate.sh origin/dev) diffs the current
    14	# checkout: HEAD is the PR checkout in the GitHub Actions runner, so omitting
    15	# head-ref is the default invocation.
    16	#
    17	# The 2-arg form (buck2-affected-gate.sh origin/dev origin/pr-N) is used by
    18	# the controller Job, where the working tree is trunk (dev) and the PR ref
    19	# is fetched as data via `git fetch origin refs/pull/N/head:refs/remotes/origin/pr-N`.
    20	#
    21	# Exit 0 = pass (incl. non-Rust / no-affected PRs); non-zero = build/test failure.
    22	set -eu
    23	
    24	BASE="${1:-origin/dev}"
    25	HEAD_REF="${2:-HEAD}"
    26	BUCK2="${BUCK2:-buck2}"
    27	
    28	echo "buck2-affected-gate: start (pwd=$(pwd) base=$BASE head-ref=$HEAD_REF resolved=$(git rev-parse --short "$HEAD_REF" 2>&1))"
    29	echo "buck2-affected-gate: .buckconfig=$(test -f .buckconfig && echo present || echo MISSING) HOME=${HOME:-unset} buck2=$($BUCK2 --version 2>&1 | head -1)"
    30	if ! git rev-parse --verify --quiet "$BASE" >/dev/null 2>&1; then
    31	  echo "buck2-affected-gate: FATAL base ref '$BASE' does not resolve in this checkout"
    32	  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
    33	  exit 1
    34	fi
    35	if ! git rev-parse --verify --quiet "$HEAD_REF" >/dev/null 2>&1; then
    36	  echo "buck2-affected-gate: FATAL head ref '$HEAD_REF' does not resolve in this checkout"
    37	  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
    38	  exit 1
    39	fi
    40	if ! MERGE_BASE=$(git merge-base "$HEAD_REF" "$BASE" 2>&1); then
    41	  echo "buck2-affected-gate: FATAL merge-base $HEAD_REF $BASE failed (need full history): $MERGE_BASE"
    42	  exit 1
    43	fi
    44	CHANGED=$(git diff --name-only "$MERGE_BASE" "$HEAD_REF")
    45	if [ -z "$CHANGED" ]; then
    46	  echo "buck2-affected-gate: no changed files vs $BASE ($HEAD_REF) -> PASS"
    47	  exit 0
    48	fi
    49	echo "buck2-affected-gate: $(printf '%s\n' "$CHANGED" | wc -l | tr -d ' ') changed file(s) vs $BASE..${HEAD_REF} (merge-base $MERGE_BASE)"
    50	
    51	# Classify. Only docs/non-graph files (e.g. .md/.yaml/.json outside crates) may
    52	# legitimately map to no target. A *.rs / Cargo.toml / buck-graph file MUST map to
    53	# a target — FAIL CLOSED if it doesn't (never silently pass a Rust change unbuilt).
    54	RUST_REL=$(printf '%s\n' "$CHANGED" | grep -E '\.rs$|/Cargo\.toml$|^Cargo\.(toml|lock)$|^\.buckconfig$|(^|/)BUCK$|^toolchains/|^third-party/' || true)
    55	if [ -z "$RUST_REL" ]; then
    56	  echo "buck2-affected-gate: no Rust/buck-graph files changed -> NoRust PASS"
    57	  exit 0
    58	fi
    59	
    60	# owner() resolution — batched to minimise buck2 daemon round-trips.
    61	#
    62	# Strategy:
    63	#   1. BUCK files: no owner() result by design (they ARE the package definition).
    64	#      Run a small per-file pass to expand each to its package target pattern.
    65	#      (One buck2 uquery per BUCK file — these are typically 0-1 files per PR.)
    66	#   2. Non-BUCK Rust/graph files: build ONE "owner('f1') union owner('f2') union ..."
    67	#      expression and run a single buck2 uquery call for all files at once.
    68	#      owner() takes file-path strings, not target-set placeholders, so %Ss/@argfile
    69	#      cannot be used here — the union expression is the correct single-call form.
    70	#      A uquery ERROR (non-zero exit) FAILS the gate — it is NOT 'no owner'.
    71	#      (The false-pass bug was: 2>/dev/null||true swallowed buck2 errors.)
    72	
    73	OWNERS=""
    74	
    75	# ── Pass 1: BUCK files → package target pattern (unchanged semantics, separate pass) ──
    76	BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -E '(^|/)BUCK$' || true)
    77	for f in $BUCK_FILES; do
    78	  [ -e "$f" ] || continue
    79	  d=$(dirname "$f")
    80	  case "$d" in
    81	    third-party)   pat="third-party//:" ;;
    82	    third-party/*) pat="third-party//${d#third-party/}:" ;;
    83	    toolchains)    pat="toolchains//:" ;;
    84	    toolchains/*)  pat="toolchains//${d#toolchains/}:" ;;
    85	    .)             pat="//:" ;;
    86	    *)             pat="//$d:" ;;
    87	  esac
    88	  if ! o=$("$BUCK2" uquery "$pat" 2>/tmp/uqerr); then
    89	    echo "buck2-affected-gate: FATAL buck2 uquery '$pat' (BUCK pkg for $f) errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
    90	  fi
    91	  [ -n "$o" ] && OWNERS="$OWNERS $o"
    92	done
    93	
    94	# ── Pass 2: non-BUCK files → ONE batched uquery call via union-of-owner() expression ──
    95	# Build: owner('f1') union owner('f2') union ... and run as a single buck2 uquery invocation.
    96	# This replaces N serial daemon round-trips (one per file) with a single round-trip.
    97	NON_BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -vE '(^|/)BUCK$' || true)
    98	NON_BUCK_EXISTING=$(printf '%s\n' "$NON_BUCK_FILES" | while read -r f; do [ -e "$f" ] && printf '%s\n' "$f"; done)
    99	if [ -n "$NON_BUCK_EXISTING" ]; then
   100	  OWNER_EXPR=$(printf '%s\n' "$NON_BUCK_EXISTING" | \
   101	    awk 'NR==1{printf "owner('"'"'%s'"'"')", $0; next} {printf " union owner('"'"'%s'"'"')", $0}')
   102	  if ! o=$("$BUCK2" uquery "$OWNER_EXPR" 2>/tmp/uqerr); then
   103	    echo "buck2-affected-gate: FATAL buck2 uquery owner() errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
   104	  fi
   105	  [ -n "$o" ] && OWNERS="$OWNERS $o"
   106	fi
   107	
   108	OWNERS=$(printf '%s\n' $OWNERS | sed '/^$/d' | sort -u)
   109	if [ -z "$OWNERS" ]; then
   110	  echo "buck2-affected-gate: FATAL Rust/buck files changed but NO owning target found (refusing to false-pass):"
   111	  printf '    %s\n' $RUST_REL
   112	  exit 1
   113	fi
   114	echo "buck2-affected-gate: $(printf '%s\n' "$OWNERS" | wc -l | tr -d ' ') owning target(s)"
   115	
   116	# Affected = changed targets + reverse-dep closure. rdeps error also FAILS closed.
   117	# Pass owners via @argfile + the %Ss set placeholder, NOT an inline set(...): a change
   118	# to a large BUCK package (e.g. third-party/BUCK owns 1689 targets) overflows the inline
   119	# query string and buck2 errors out (uquery RC=3, no build attempted) — which silently
   120	# blocked landing ANY third-party change. @argfile + %Ss handles an arbitrary set size
   121	# (verified: 1689 owners -> 1919 affected). One owner per line.
   122	printf '%s\n' $OWNERS | sed '/^$/d' > /tmp/gate-owners.txt
   123	if ! AFFECTED=$("$BUCK2" uquery 'rdeps(//..., %Ss)' @/tmp/gate-owners.txt 2>/tmp/rqerr); then
   124	  echo "buck2-affected-gate: FATAL rdeps query errored:"; sed 's/^/    /' /tmp/rqerr; exit 1
   125	fi
   126	N=$(printf '%s\n' "$AFFECTED" | sed '/^$/d' | wc -l | tr -d ' ')
   127	echo "buck2-affected-gate: $N affected target(s) (owners + reverse-dep closure)"
   128	if [ "$N" = "0" ]; then echo "buck2-affected-gate: FATAL owners found but rdeps empty (query problem)"; exit 1; fi
   129	
   130	# Build then test the affected set. @- reads the newline-delimited target list
   131	# from stdin, avoiding ARG_MAX limits on large closures.
   132	printf '%s\n' "$AFFECTED" | sed '/^$/d' > /tmp/affected-targets.txt
   133	echo "=== buck2 build (affected) ==="
   134	"$BUCK2" build @/tmp/affected-targets.txt
   135	echo "=== buck2 test (affected) ==="
   136	"$BUCK2" test @/tmp/affected-targets.txt
   137	echo "buck2-affected-gate: PASS"
exec
/bin/zsh -lc "git show ac80d6ab2882dc54a199d5e050be5a392ea50132:.github/workflows/oya-ci-required.yml | awk 'BEGIN{n=0; in_jobs=0} /"'^jobs:/{in_jobs=1; next} in_jobs && /''^  [A-Za-z0-9_-]+:$/ {n++; line=$0; sub(/''^  /,"",line); sub(/:$/,"",line); print NR ":" line} END {print "top_level_jobs=" n}'"'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
50:producer-regen
114:gate
196:gate-freshness
233:gate-registry-drift
264:gate-cloud-ci-firewall
302:generated-output-diff-policy
353:buck2
533:gate-affected-set
906:gate-live-postgres-adapters
1043:gate-live-postgres-facades
1170:oya-ci-required
top_level_jobs=11
exec
/bin/zsh -lc "git show 1801128b16a0fe6318e84c1939d6e0937a677708:.github/workflows/oya-ci-required.yml | awk 'BEGIN{n=0; in_jobs=0} /"'^jobs:/{in_jobs=1; next} in_jobs && /''^  [A-Za-z0-9_-]+:$/ {n++; line=$0; sub(/''^  /,"",line); sub(/:$/,"",line); print NR ":" line} END {print "top_level_jobs=" n}'"'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
50:producer-regen
114:gate
200:gate-generated-artifact-freshness
237:gate-inventory-registry-drift
268:gate-baseline-ratchet
306:generated-output-diff-policy
362:buck2
542:gate-affected-target-set
929:gate-live-postgres-adapters
1066:gate-live-postgres-facades
1193:oya-ci-required
top_level_jobs=11
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | awk 'BEGIN{n=0; in_jobs=0} /"'^jobs:/{in_jobs=1; next} in_jobs && /''^  [A-Za-z0-9_-]+:$/ {n++; line=$0; sub(/''^  /,"",line); sub(/:$/,"",line); print NR ":" line} END {print "top_level_jobs=" n}'"'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
50:producer-regen
114:gate
201:gate-generated-artifact-freshness
238:gate-inventory-registry-drift
269:gate-baseline-ratchet
307:generated-output-diff-policy
363:buck2
543:gate-affected-target-set
930:gate-live-postgres-adapters
1067:gate-live-postgres-facades
1194:oya-ci-required
top_level_jobs=11
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '45,350p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
    45	jobs:
    46	  # ── Producer regen: materialize the cloud-ci generated faces from the checked-out candidate
    47	  #    tree. Generated JSON is not a contributor merge surface; the CI/controller workspace
    48	  #    regenerates it before gates consume it, then uploads it both as evidence AND as the
    49	  #    faces source the mere-reader gate matrix legs download (ADR-0556 D5 QW-1).
    50	  producer-regen:
    51	    name: producer-regen (accounting-registry)
    52	    runs-on: ubuntu-latest
    53	    steps:
    54	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    55	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
    56	        with:
    57	          persist-credentials: false
    58	          # Full history: the accounting producer derives last_touch_commit via
    59	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
    60	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
    61	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
    62	          # and identical to a full local clone.
    63	          fetch-depth: 0
    64	      - name: Install buck2 (digest-pinned prebuilt release)
    65	        run: infra/ci/install-buck2.sh
    66	      - name: Materialize cloud-ci generated faces
    67	        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
    68	      - name: Upload regenerated faces
    69	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    70	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
    71	        with:
    72	          name: accounting-faces
    73	          # Two upload roots -> the artifact is rooted at their least common ancestor
    74	          # (ci/facade/): the regenerated accounting faces PLUS the untracked
    75	          # volatile scm snapshot (ADR-0552) the staleness-reaper leg ages rows from. The
    76	          # mere-reader gate matrix legs download this artifact instead of re-materializing
    77	          # per leg (ADR-0556 D5 QW-1, gate-fleet-shared-graph same-run trusted reuse).
    78	          # Deliberately NOT uploaded: the firewall's merge-base frozen baseline — its
    79	          # materialization is per-job by design (ADR-0551 frozen-policy-wins) and must
    80	          # never become a shareable artifact.
    81	          path: |
    82	            ci/facade/artifact-inventory-registry/*.generated.json
    83	            ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json
    84	          if-no-files-found: error
    85	
    86	  # ── GATE LANES (reusable matrix). Every homogeneous gate is the SAME step — one
    87	  #    `buck2 test //ci/facade/<crate>:{ci-<crate>-unittest,ci-<crate>-gate}` — so instead of copy-pasting a job per gate, a single
    88	  #    matrixed `gate` job fans out over the crate list. Adding a homogeneous gate is ONE line in the matrix
    89	  #    below; the `gate_registration` meta-test (in the cloud-ci-firewall lane) ENFORCES that
    90	  #    every in-tree gate crate is listed here, every oya-ci.toml accounting/firewall gate has
    91	  #    bundled disposition DATA, and every gate lane is a fan-in dependency. Each matrix leg is its own check-run
    92	  #    `gate (oya-cloud-ci-<x>-app)`, preserving per-gate attribution; legs with a live-corpus
    93	  #    self-test are born-blocking. `fail-fast: false` = surface-all (every leg runs to
    94	  #    completion even if a sibling fails).
    95	  #    (Deliberately a matrix, NOT a `workflow_call` reusable workflow: a called workflow would
    96	  #    rename the published check-runs [`<caller> / <job>`], breaking the `oya-ci-required`
    97	  #    branch-protection context. A future owned oya-ci runner can reuse this matrix verbatim —
    98	  #    "one logic, two runners", D-CICD-AUTHORITY.)
    99	  #
   100	  #    FACES VIA ARTIFACT (ADR-0556 D5 QW-1, gate-fleet-shared-graph warm-safe same-run reuse):
   101	  #    every leg here is a MERE READER of the generated faces — none re-verifies regeneration
   102	  #    itself, so each leg downloads the producer-regen artifact instead of paying its own
   103	  #    materialize step (the audit-measured ~45-55s/leg hub rebuild, 16x per run).
   104	  #    Same-run, same candidate tree, same trusted writer — no cross-run cache participation.
   105	  #    The regeneration-checking gates KEEP materializing independently: registry-drift (the
   106	  #    byte-parity detector — detectors never consume the thing they attest), producer-regen
   107	  #    itself, and cloud-ci-firewall (its merge-base frozen baseline is per-job by design,
   108	  #    ADR-0551). `needs: producer-regen` serializes these legs behind a ~75s producer job;
   109	  #    the workflow critical path (affected-set/buck2 lanes) is unaffected. If producer-regen
   110	  #    fails, legs are skipped and the fan-in goes RED (fail-closed, same join as today).
   111	  #    `fetch-depth: 0` retained conservatively: no matrix gate calls git (verified — faces
   112	  #    carry all history-derived data per ADR-0552), but shallowing the checkout is a separate
   113	  #    reviewed change, not a side effect of artifact reuse.
   114	  gate:
   115	    needs: producer-regen
   116	    # Descriptive per-leg check-run name (matrix.label) — each leg publishes as
   117	    # "gate · <discipline>", not a bare "gate (crate)". Adding a gate = one `include` line
   118	    # (crate + label); the gate_registration meta-test enforces every gate crate is listed.
   119	    name: ${{ matrix.label }}
   120	    runs-on: ubuntu-latest
   121	    strategy:
   122	      fail-fast: false
   123	      matrix:
   124	        include:
   125	          - { crate: cross-artifact-agreement, label: "gate · cross-artifact-agreement (GATE-1, incl. the 4 masterplan-v2 plan gates: structural ID/DAG/orphans · projection-freshness · plan-vs-evidence · read-contract/entry-surface)" }
   126	          - { crate: artifact-accountability,         label: "gate · total-accounting (GATE-2)" }
   127	          - { crate: stale-artifact-detection,         label: "gate · staleness-reaper (GATE-3, born-blocking)" }
   128	          - { crate: automation-coverage,       label: "gate · automation-ratchet (GATE-4, polices gates)" }
   129	          - { crate: crate-layer-suffix,         label: "gate · bnf-layer-suffix (BNF §2.5#4, born-blocking)" }
   130	          - { crate: package-manifest-hygiene,         label: "gate · manifest-hygiene (§2.5#7, born-blocking)" }
   131	          - { crate: crate-name-prefix,            label: "gate · cargo-prefix (ADR-0017)" }
   132	          - { crate: slo-coverage,            label: "gate · slo-coverage (catalog SLO input contract, born-blocking)" }
   133	          - { crate: license-policy,          label: "gate · license-policy (workspace package license policy, shrink-only)" }
   134	          - { crate: service-catalog-parity,        label: "gate · catalog-liveness (PR-C3 founder live-OR-explicitly-marked policy, born-blocking, EMPTY frozen baseline)" }
   135	          - { crate: workspace-member-coverage, label: "gate: workspace-glob-coverage (ADR-0538)" }
   136	          - { crate: build-target-parity,           label: "gate · target-parity (ADR-0540, test-wiring false-green)" }
   137	          - { crate: hook-wiring,    label: "gate · enforcement-liveness (FRIC-012, hook mirror liveness)" }
   138	          - { crate: action-item-accounting,     label: "gate · friction-accounting (ADR-0544, closed-loop friction-ledger accounting)" }
   139	          - { crate: canonical-json,          label: "gate · canonical-json (ADR-0546, deterministic JSON serialization)" }
   140	          - { crate: parity-claim-evidence, label: "gate · hyperscaler-parity-taxonomy (cloud hyperscaler parity taxonomy, born-blocking)" }
   141	          - { crate: resource-contract-conformance, label: "gate · cloud-resource-contracts (Rust/API replacement for P0 cloud-resource Python validators)" }
   142	          - { crate: contract-slice-conformance, label: "gate · contract-slice-conformance (paved-road Rust/Buck2 replacement for scripts/tests/*_check.py contract-slice validators; ADR-0515/0523/0528)" }
   143	          - { crate: embedded-asset-hermeticity, label: "gate · embedded-asset-hermeticity (ADR-0545, include_str!/include_bytes! __srcs-tree mapping)" }
   144	          - { crate: core-dependency-isolation,           label: "gate · kernel-purity (ADR-0547, *-kernel/*-core zero transient-tech deps)" }
   145	          - { crate: crypto-backend-policy,   label: "gate · crypto-backend-purity (ADR-0506, ring forbidden / aws-lc-rs mandated — zero ring activation)" }
   146	          - { crate: graphql-usage-policy,  label: "gate · no-graphql-without-adr (ADR-0565, zero-GraphQL: no graphql lib / .graphql/.gql/.sdl reintroduction without a reversing ADR — candidate-tree evaluated, EMPTY frozen baseline)" }
   147	          - { crate: endpoint-authorization-coverage,          label: "gate · authz-coverage (issue #770 / AUTH-005, NEW unauthenticated HTTP control planes blocked vs frozen baseline)" }
   148	          - { crate: caller-supplied-authorization,         label: "gate · dto-authz-trust (ADR-0582, the CLASS-FIX for caller-supplied-authz-trust: a NEW fn that trusts a forged *Authorization DTO / x-authorization-* header in place of a server-side PDP decide() is blocked vs the frozen baseline of ~92 known instances; v2: FN-01/02/03/04/05/06 hardened)" }
   149	          - { crate: generated-artifact-policy, label: "gate · generated-artifact-control-plane (public hermetic CI artifact policy)" }
   150	          - { crate: build-cache-policy,            label: "gate · cache-wiring conformance (ADR-0560/ADR-0556, dark-wiring + cold floor + kill-switch)" }
   151	          - { crate: dependency-graph-acyclicity, label: "gate · substrate-dependency-dag-acyclicity (ADR-0280 §D-3, Tarjan SCC + Kahn topo-order + forbidden-edge honouring)" }
   152	          - { crate: service-tier-metadata,     label: "gate · tier-field-coverage (Phase-0 reorg ADR-0562/0536/0245, per-service tier/tier_subtype/dr_tier coverage + enum validity + no type-overload, born-blocking)" }
   153	          - { crate: layer-dependency-acyclicity, label: "gate · tier-dependency-acyclicity (Phase-0 reorg ADR-0245/0280/0562, cargo+buck crate-graph tier rules + S-rank + Tarjan cycle backstop, born-ADVISORY vs frozen baseline → enforce-no-regression)" }
   154	          - { crate: module-membership,   label: "gate · capability-membership (Phase-0 reorg ADR-0562 §6, the anti-junk-drawer MEMBERSHIP lint: every crate → exactly one registered capability/meta home, no NEW unmapped crate, no NEW top-level dir, base/-admission; born-advisory + enforce-no-regression vs the frozen unmapped baseline)" }
   155	          - { crate: runner-disk-reclaim,     label: "gate · runner-disk-reclaim conformance (FRIC-017 productization, ADR-0548 pipeline-as-product: policy parse + threshold/INFRA-RED discrimination + reclaim plan)" }
   156	          - { crate: port-placement,          label: "gate · port-placement (ADR-0570, clean-arch ports-in-core: no storage-port trait DEFINED in an */adapters/* crate — productizes the #116 defect class; born-advisory + enforce-no-regression vs frozen baseline)" }
   157	          - { crate: repo-root-hygiene,  label: "gate · root-workspace-hygiene (ADR-0600, allowlist-as-DATA default-DENY: every TRACKED repo-root file must match the allowlist + every top-level dir must be a permitted capability/meta home — makes committed root scratch structurally impossible; complements the scratch DENYLIST)" }
   158	          - { crate: dependency-automation, label: "gate · dependency-automation (ADR-0535, owned oya-deps.toml Rust bump-bot contract; external bot configs remain absent)" }
   159	          - { crate: supply-chain-audit,    label: "gate · supply-chain-audit (owned RustSec advisory scan over vendored mirror, born-blocking)" }
   160	          - { crate: feature-maturity-policy, label: "gate · planned-maturity (GH #992, product PRD acceptance/verification contracts + rich capability records + retired-plan provenance boundary)" }
   161	          - { crate: operator-secret-rbac, label: "gate · operator-secret-bootstrap (GH #980 + GH #988 / ADR-0606, least-privilege secret RBAC, declarative join-token bootstrap, ESO/OpenBao role+namespace+prefix scope, and plaintext OpenBao NetworkPolicy isolation)" }
   162	          - { crate: policy-deploy-parity, label: "gate · cedar-deploy-parity (GH #16 / ADR-0608, deployed-vs-authored Cedar parity: no deployed ConfigMap permit may leave the action unconstrained (action-agnostic blanket grant) and every deployed permit MUST be ⊆ the capability's authored <cap>/{policy,cedar}/*.cedar set; fail-closed on missing-authored/un-extractable/empty-scan; GH #16 byte-identical blanket ConfigMaps grandfathered in a documented shrink-only baseline pending the blanket-disarm follow-up)" }
   163	          - { crate: topology-manifest-contract, label: "gate · cell-topology-manifest-contract (CELL-001R manifest contract)" }
   164	          - { crate: automation-language-policy, label: "gate · rust-first automation hygiene (cloud-native infra anti-patterns: scripts, workflow shell, retired interpreters, and new CLI packages)" }
   165	          - { crate: gate-self-conformance, label: "gate · gate-self-conformance (GH #777, pipeline-as-product 7-property bar over every gate)" }
   166	    steps:
   167	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   168	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   169	        with:
   170	          persist-credentials: false
   171	          fetch-depth: 0
   172	      # Consume the producer-regen artifact (faces + volatile scm snapshot) instead of
   173	      # re-materializing per leg — see the FACES VIA ARTIFACT note on this job. The download
   174	      # restores the same regenerated bytes the producer derived from this run's candidate
   175	      # tree; registry-drift separately proves that derivation is byte-deterministic.
   176	      - name: Download regenerated faces (producer-regen artifact, ADR-0556 D5 QW-1)
   177	        # actions/download-artifact@v8.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   178	        uses: actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c
   179	        with:
   180	          name: accounting-faces
   181	          path: ci/facade
   182	      - name: Install buck2 (digest-pinned prebuilt release)
   183	        run: infra/ci/install-buck2.sh
   184	      # Buck2/rustc still use the pinned workspace toolchain + components; provision it before
   185	      # the test action starts to avoid concurrent rustup writes inside parallel Buck2 actions.
   186	      - name: Pre-provision pinned Rust toolchain for Buck2 gate tests
   187	        run: |
   188	          set -euo pipefail
   189	          rustup toolchain install
   190	          rustc --version
   191	      - name: buck2 test ${{ matrix.crate }}
   192	        run: |
   193	          set -euo pipefail
   194	          buck2 test \
   195	            //ci/facade/${{ matrix.crate }}:ci-${{ matrix.crate }}-unittest \
   196	            //ci/facade/${{ matrix.crate }}:ci-${{ matrix.crate }}-gate
   197	
   198	  # ── freshness: first-diagnosis gate for the two stale-output failures from PR #662.
   199	  #    Runs as its own fast job with no needs edge so stale Cargo.lock and stale generated faces
   200	  #    surface together before the broader Buck2 lanes spend a full matrix round-trip.
   201	  gate-generated-artifact-freshness:
   202	    name: freshness (lock + generated faces, ADR-0539)
   203	    runs-on: ubuntu-latest
   204	    steps:
   205	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   206	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   207	        with:
   208	          persist-credentials: false
   209	          fetch-depth: 0
   210	      - name: Install buck2 (digest-pinned prebuilt release)
   211	        run: infra/ci/install-buck2.sh
   212	      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4: the toolchain is a
   213	      # version-pinned input, not a build output). rustup still resolves/validates the
   214	      # toolchain on every run; the cache only pre-seeds ~/.rustup so the install is a no-op.
   215	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   216	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   217	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   218	        with:
   219	          path: |
   220	            ~/.rustup/toolchains
   221	            ~/.rustup/update-hashes
   222	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   223	      - name: Pre-provision pinned Rust toolchain for Buck2 freshness binaries
   224	        run: |
   225	          set -euo pipefail
   226	          rustup toolchain install
   227	          rustc --version
   228	      - name: Run freshness gate
   229	        run: |
   230	          set -euo pipefail
   231	          freshness_bin="$(buck2 build //ci/facade/generated-artifact-freshness:oya-cloud-ci-freshness-app-bin --show-output | awk '{print $2}')"
   232	          "${freshness_bin}" --repo-root .
   233	
   234	  # ── registry-drift: materialized workspace == regenerated byte-equal. Starts at t=0 alongside
   235	  #    producer-regen; it rematerializes in-job so it is hermetic and self-contained. The
   236	  #    producer-regen needs-edge was cosmetic (evidence only, nothing consumed) and serialized
   237	  #    this job unnecessarily — removed so it starts at t=0.
   238	  gate-inventory-registry-drift:
   239	    name: registry-drift (materialized == regenerated)
   240	    runs-on: ubuntu-latest
   241	    steps:
   242	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   243	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   244	        with:
   245	          persist-credentials: false
   246	          # Full history: the accounting producer derives last_touch_commit via
   247	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
   248	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
   249	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
   250	          # and identical to a full local clone.
   251	          fetch-depth: 0
   252	      - name: Install buck2 (digest-pinned prebuilt release)
   253	        run: infra/ci/install-buck2.sh
   254	      # HERMETICITY CONTRACT (ADR-0556 D5 QW-1 deliberate exception): this gate IS the
   255	      # byte-parity detector (committed == regenerated), so it MUST regenerate in-job —
   256	      # feeding it the producer-regen artifact it is supposed to verify would make the
   257	      # check self-referential. Detectors never consume the thing they attest.
   258	      - name: Materialize faces then assert byte-parity
   259	        run: |
   260	          rustup toolchain install
   261	          rustc --version
   262	          buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   263	          buck2 test //ci/facade/inventory-registry-drift:ci-inventory-registry-drift-gate
   264	
   265	  # ── cloud-ci-firewall: the baseline ratchet (blocks only NEW debt) + the gate-registration
   266	  #    meta-test (no in-tree gate may go unregistered in this workflow). This is the surface-all
   267	  #    runner; per the runbook the existing firewall runner suffices — no separate aggregator bin
   268	  #    is required for PRE-work.
   269	  gate-baseline-ratchet:
   270	    name: cloud-ci-firewall (baseline ratchet + gate-registration meta-test)
   271	    runs-on: ubuntu-latest
   272	    steps:
   273	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   274	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   275	        with:
   276	          persist-credentials: false
   277	          # Full history: the accounting producer derives last_touch_commit via
   278	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
   279	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
   280	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
   281	          # and identical to a full local clone.
   282	          fetch-depth: 0
   283	      - name: Install buck2 (digest-pinned prebuilt release)
   284	        run: infra/ci/install-buck2.sh
   285	      # HERMETICITY CONTRACT (ADR-0551 frozen-policy-wins): the firewall's frozen reference —
   286	      # the merge-base baseline snapshot — is materialized per-job BY DESIGN via the emitter's
   287	      # out-of-band bootstrap ref, and is deliberately absent from the producer-regen artifact.
   288	      # This lane therefore KEEPS its own materialization (ADR-0556 D5 cold-must-stay list);
   289	      # it is never converted to artifact reuse.
   290	      - name: Materialize cloud-ci generated faces
   291	        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   292	      - name: buck2 test cloud-ci-firewall
   293	        run: |
   294	          set -euo pipefail
   295	          buck2 test \
   296	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-unittest \
   297	            //ci/facade/baseline-ratchet:oya-cloud-ci-firewall-signoff-fixer-unittest \
   298	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-gate \
   299	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-gate-registration \
   300	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-run-observability-packet
   301	
   302	  # ── GENERATED OUTPUT DIFF POLICY. Generated files may be deleted to retire a tracked output,
   303	  #    but PRs must not add/modify generated outputs as merge surfaces. Classification comes from
   304	  #    registry/generated-artifact-control-plane.json `generated_path_rules` so adopters can encode
   305	  #    their generated-output conventions once; .gitignore is preventive hygiene, not policy
   306	  #    authority. The candidate workspace is regenerated by cloud-ci before gates consume it.
   307	  generated-output-diff-policy:
   308	    name: generated-output-diff-policy (no generated merge surfaces)
   309	    runs-on: ubuntu-latest
   310	    steps:
   311	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   312	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   313	        with:
   314	          persist-credentials: false
   315	          fetch-depth: 0
   316	      - name: Install buck2 (digest-pinned prebuilt release)
   317	        run: infra/ci/install-buck2.sh
   318	      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
   319	      # validates the toolchain on every run.
   320	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   321	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   322	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   323	        with:
   324	          path: |
   325	            ~/.rustup/toolchains
   326	            ~/.rustup/update-hashes
   327	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   328	      - name: Pre-provision pinned Rust toolchain for Buck2 policy binary
   329	        run: |
   330	          set -euo pipefail
   331	          rustup toolchain install
   332	          rustc --version
   333	      - name: Reject non-deletion generated output edits
   334	        env:
   335	          EVENT_NAME: ${{ github.event_name }}
   336	          BASE_REF: ${{ github.base_ref || 'dev' }}
   337	        run: |
   338	          set -euo pipefail
   339	          if [ "${EVENT_NAME}" = "push" ]; then
   340	            echo "generated-output-diff-policy: push event; presubmit diff policy not applicable."
   341	            exit 0
   342	          fi
   343	          git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}"
   344	          policy_bin="$(buck2 build //ci/facade/generated-artifact-policy:oya-cloud-ci-generated-output-diff-policy --show-output | awk '{print $2}')"
   345	          # --find-renames is REQUIRED, not cosmetic: the policy's sanctioned-relocation exemption
   346	          # only accepts byte-identical (R100) renames of declared generated artifacts (a capability
   347	          # move relocating the firewall's frozen gate-baseline). Without explicit rename detection a
   348	          # runner with diff.renames=off would surface the move as A+D and RED the legit move (fails
   349	          # safe, but false-blocks). Detection ON + the R100-only exemption is the correct behavior.
   350	          git diff --find-renames --name-status "origin/${BASE_REF}"...HEAD \
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '350,545p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   350	          git diff --find-renames --name-status "origin/${BASE_REF}"...HEAD \
   351	            | "${policy_bin}" --manifest registry/generated-artifact-control-plane.json
   352	
   353	  # ── HERMETIC BUCK2 LANE (OYA-CI-HERMETIC-EXECUTION-DESIGN §3 + Stage P1/P2). Runs the SAME
   354	  #    gate logic through buck2: `buck2 build` compiles every
   355	  #    target (the env!CARGO eradication) and `buck2 test` runs the gate rust_tests fully
   356	  #    hermetically (no ambient git in any action — the producer reads the materialized scm-facts
   357	  #    face; the scm-facts emitter is the single out-of-graph boundary, run in the
   358	  #    materialization step BELOW, never inside a cacheable action). Scoped by the
   359	  #    affected-set driver (`infra/ci/buck2-affected-gate.sh`: uquery owner -> rdeps closure,
   360	  #    FAILS CLOSED) for speed. RBE/NativeLink is staged LAST (D4) and NOT required for
   361	  #    hermeticity — local-on-runner execution via the wired `noop_test_toolchain` is sufficient
   362	  #    here. This lane feeds the same fan-in as the targeted Buck2 gate matrix above.
   363	  buck2:
   364	    name: buck2 (hermetic build + affected gate tests)
   365	    runs-on: ubuntu-latest
   366	    steps:
   367	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   368	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   369	        with:
   370	          persist-credentials: false
   371	          # Full history: the materialization step below runs the emitter, which derives
   372	          # last_touch via `git log --name-only`; a shallow checkout collapses it (PM1).
   373	          fetch-depth: 0
   374	      # buck2 is the BUILD TOOL, installed as a prebuilt release.
   375	      # The adapter edge is immutable at CI time: release tag selects the asset, SHA-256 pins
   376	      # the bytes, and CI verifies the digest before decompression/execution. Bump together.
   377	      - name: Install buck2 (digest-pinned prebuilt release)
   378	        run: infra/ci/install-buck2.sh
   379	      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
   380	      # validates the toolchain on every run.
   381	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   382	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   383	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   384	        with:
   385	          path: |
   386	            ~/.rustup/toolchains
   387	            ~/.rustup/update-hashes
   388	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   389	      # Pre-provision the pinned rust toolchain ONCE, serially, before the buck2 build.
   390	      # The buck2 rust toolchain (toolchains/BUCK: system_rust_toolchain via the rustup shim)
   391	      # resolves rustc and toolchain components per compile action, and buck2 runs those actions in
   392	      # PARALLEL. On a cold runner each action's first shim call triggers rustup to install the
   393	      # rust-toolchain.toml channel (+ rustfmt,clippy) concurrently — the racing rustup
   394	      # processes collide on the shared `~/.rustup/downloads/*.partial` files and fail with
   395	      # `rustup::utils::rename ... No such file or directory (os error 2)` (a different component
   396	      # each run: clippy, then a second toolchain component — proving a concurrency race, not a config defect). rustup
   397	      # is not concurrency-safe. Installing the toolchain once here makes it ambient so the
   398	      # parallel actions find it already present (no download).
   399	      - name: Pre-provision pinned rust toolchain (serialize rustup before parallel buck2)
   400	        run: |
   401	          set -euo pipefail
   402	          rustup toolchain install
   403	          rustc --version
   404	      # Restore buck-out across runs so ephemeral runners start warm (design §3.1 / ADR-0515 D4).
   405	      #
   406	      # WHAT IS WARMED, PRECISELY (ADR-0554 D9, round-5): 100% of buck2 cross-run warmth lives in
   407	      # ./buck-out — buck-out/v2/cache/{materializer_state,incremental_state}/db.sqlite plus
   408	      # buck-out/v2/art (the materialized action outputs). buck-out is PATH-RELOCATABLE (relative
   409	      # paths + --remap-cwd-prefix=., no absolute project root baked in; a restored hit is keyed
   410	      # only on buck2_revision + os + arch), so restoring it into any runner checkout is sound.
   411	      # ~/.buck2 and ~/.buck hold ONLY daemon pid/endpoint/log scratch — ZERO action results — so
   412	      # caching `path: buck-out` warms everything cacheable and ~/.buck2/~/.buck is DELIBERATELY
   413	      # NOT cached (caching daemon scratch warms nothing; do not copy-paste global-state caching).
   414	      #
   415	      # SAVE/RESTORE SPLIT (ADR-0554 D9; specs/cache-warmth-policy.json postmerge-dev-trunk = sole
   416	      # writer, presubmit-trusted-affected-cone = reader). dev-push is the SOLE canonical writer via
   417	      # the gate-affected-target-set job's SAVE step (see below): that job builds the FULL //... graph on
   418	      # push, so the saved buck-out is the full-graph superset. This buck2 job is READ-ONLY on every
   419	      # trigger — it restores the full-graph buck-out and its //ci/... build is a subset
   420	      # hit. PRs never write, eliminating the per-commit multi-GB write-churn/eviction (the
   421	      # Bazel/Google post-merge-fills-the-cache pattern).
   422	      #
   423	      # Cache key is STABLE per dependency/toolchain-set (.buckconfig + toolchains/BUCK +
   424	      # Cargo.lock + rust-toolchain.toml),
   425	      # NOT per-commit. The previous `-${{ github.sha }}` suffix made the primary key unique
   426	      # every commit, so actions/cache SAVED a fresh full buck-out (multi-GB) on EVERY run and
   427	      # never hit the primary key — bloating the 10GB repo cache into constant LRU eviction and
   428	      # exhausting ephemeral-runner disk at the save step (the "No space left on device" failure,
   429	      # FRIC-017). A stable key saves once per dependency-set and restores it exactly: deterministic
   430	      # warm start, no per-commit bloat. Changed crates still rebuild (buck2 is content-addressed,
   431	      # so a restored hit is bit-identical to a cold build); only a Cargo.lock/toolchain/.buckconfig
   432	      # or Rust channel change mints a new entry. The restore prefix is scoped by
   433	      # rust-toolchain.toml so a Rust-version bump never reuses old rlibs into the new compiler.
   434	      # Interim warm-by-default until the shared content-addressed
   435	      # remote cache (NativeLink/CAS, ADR-0560, HANDOFF W3) lands with a cold-canary integrity job
   436	      # proving cold==warm. See friction-ledger buck2-no-shared-cache.
   437	      # Reclaim preinstalled ubuntu-latest bloat (.NET/Android/GHC/CodeQL/preloaded Docker images:
   438	      # ~25-30 GiB) BEFORE the multi-GB buck-out restore. This lane decompresses a ~5.78 GiB buck-out
   439	      # blob (~12-15 GiB on disk) on top of a fetch-depth:0 monorepo checkout, exhausting the ~14 GiB
   440	      # free on / on GitHub-hosted ubuntu-latest; FRIC-017 recurred on PR #741 (No space left on device
   441	      # at this restore, before any build ran). Hermetic: removes only vendor preinstall dirs that no
   442	      # oya/buck2 action consumes; touches NO repo content and NO cache (buck-out / ~/.rustup / the
   443	      # restored blob untouched), so the cold==warm integrity canary (ADR-0556/0560) is unaffected. df
   444	      # is emitted so a genuine disk-NEED growth surfaces as a true RED instead of being masked.
   445	      - name: Reclaim runner disk before warm restore (FRIC-017 preflight)
   446	        # Rust-first, data-driven preflight (ADR-0548 pipeline-as-product): retires the two
   447	        # duplicated inline `sudo rm -rf` blocks. The policy (reclaim_dirs + min_free_gib_after)
   448	        # lives in runner-disk-reclaim-policy.json; the bin best-effort removes the profile's
   449	        # vendor preinstall dirs and logs structured disk-before/after plus a JSON operator
   450	        # artifact. Policy is explicit fail-closed: threshold-miss exits INFRA-RED unless a future
   451	        # caller supplies a typed fail-open waiver, so the required context cannot silently green on
   452	        # insufficient runner capacity. Built as the runner user (buck2 on user PATH; daemon must
   453	        # not run as root); only the prebuilt binary is sudo'd (needs root for root-owned dirs).
   454	        run: |
   455	          # Build as the runner user (buck2 on user PATH; never run buck2 daemon as root —
   456	          # that corrupts cache/daemon ownership). Then sudo ONLY the prebuilt binary (needs
   457	          # root solely to remove the root-owned vendor preinstall dirs).
   458	          BIN="$(buck2 build //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin --show-output 2>/dev/null | awk '{print $2}')"
   459	          sudo -E "$BIN" \
   460	            --profile github-hosted-ubuntu-latest \
   461	            --infra-red-policy fail-closed \
   462	            --artifact-out "${RUNNER_TEMP}/runner-disk-reclaim-buck2.json"
   463	      - name: Restore buck-out (read-only; dev-push is the sole writer)
   464	        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   465	        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
   466	        with:
   467	          path: buck-out
   468	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   469	          restore-keys: |
   470	            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
   471	      # Generated-face materialization — the SINGLE out-of-graph git boundary. Re-run the emitter
   472	      # and producer against the checked-out candidate tree, then let buck2 consume those files as
   473	      # declared inputs. We deliberately do NOT byte-compare against committed JSON here: that was
   474	      # the self-referential merge-conflict surface. Byte-parity is checked after materialization.
   475	      # KEEPS materializing (not converted to ADR-0556 D5 QW-1 artifact reuse): the hermetic
   476	      # graph's gate tests consume the firewall's merge-base frozen baseline, which is per-job
   477	      # by design (ADR-0551) and deliberately absent from the producer-regen artifact; this
   478	      # step is the sanctioned boundary that feeds ALL declared generated inputs to the graph.
   479	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   480	        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   481	      # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication —
   482	      # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests
   483	      # run green, fully hermetic, with verdicts identical to the targeted gate matrix). This is the
   484	      # refactor's scope and is the binding hermetic check for this stage.
   485	      #
   486	      # The repo-wide affected-set verdict is owned by the binding gate-affected-target-set job below.
   487	      # Do not run a duplicate best-effort affected-set probe here: a non-blocking BUILD FAILED
   488	      # line inside a green job is indistinguishable from a false-green to humans and agents.
   489	      - name: buck2 build + test (//ci/..., hermetic — binding)
   490	        run: |
   491	          set -euo pipefail
   492	          # buck2 test builds its targets before running them, so a standalone
   493	          # `buck2 build` immediately before is redundant — removed (item 4 quick win).
   494	          # --unstable-write-invocation-record is additive observability only: it
   495	          # writes buck2's structured run record (cache_hit_rate, run_* counters)
   496	          # for the telemetry step below and changes nothing about the build.
   497	          buck2 test //ci/... --unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json
   498	      # Per-lane cache-hit telemetry + warm-mode guard (ADR-0560; the audit's missing-SLO item):
   499	      # structured counters from buck2's invocation record — never log-grep — labeled with this
   500	      # lane's ADR-0556 build class. The report is now binding for record-shape / warm-mode
   501	      # sanity: once owned cloud-ci flips this lane from `bypass` to warm-ro/rw, a 0%-hit run or
   502	      # missing cache counters is an INFRA-RED misconfiguration, not advisory noise. Today GitHub
   503	      # Actions remains the transitional adapter and this lane stays bypass while NativeLink is dark.
   504	      - name: Cache-hit telemetry + warm-mode guard (ADR-0560)
   505	        if: always()
   506	        run: |
   507	          set -euo pipefail
   508	          CACHE_MODE=bypass
   509	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- report --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}" --out /tmp/cache-hit-report.json
   510	          cat /tmp/cache-hit-report.json
   511	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- assert-warm --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}"
   512	      - name: Upload cache-hit telemetry artifact
   513	        if: always()
   514	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   515	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   516	        with:
   517	          name: cache-hit-report-buck2-lane
   518	          path: /tmp/cache-hit-report.json
   519	          if-no-files-found: error
   520	      - name: Upload runner disk reclaim operator artifact (buck2 lane)
   521	        if: always()
   522	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   523	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   524	        with:
   525	          name: runner-disk-reclaim-buck2
   526	          path: ${{ runner.temp }}/runner-disk-reclaim-buck2.json
   527	          retention-days: 30
   528	          if-no-files-found: warn
   529	  # ── BINDING WORKSPACE COVERAGE (ADR-0554, FRIC-1781310000). Closes the largest false-green
   530	  #    channel: the buck2 lane above is scoped to //ci/..., so code anywhere else
   531	  #    (oya/*, libs/*, cloud/* services) could merge broken — proven live by PR #651 head
   532	  #    cf16525 (E0433 x3 in oya/identity, buck2 lane green, run 27288019517) and by
   533	  #    //oya/ci-webhook-gateway carrying an E0428 on dev itself. This lane is the BINDING
   534	  #    Rust successor of the advisory infra/ci/buck2-affected-gate.sh step (G011): on
   535	  #    pull_request it derives the merge-base diff's owner()/rdeps() cone and builds+tests it;
   536	  #    any derivation uncertainty or escape-trigger path class (BUCK macros, toolchains,
   537	  #    third-party, .buckconfig) escalates FAIL-CLOSED to the full workspace — never skips.
   538	  #    On merge_group/push/dispatch it runs the full workspace (ADR-0515 Tide admission tier).
   539	  #    On a PR, the FULL tier is a BUILD-HEALTH RATCHET (ADR-0554 round-3; ADR-0551 merge-base
   540	  #    frozen pattern): it grandfathers targets already failing at the merge-base and blocks only
   541	  #    REGRESSIONS, so a BUCK-touching PR is not held hostage to pre-existing dev build debt (no
   542	  #    flag-day requirement). Pack-shaped per ADR-0548 R0: all repo facts live in affected-set-policy.json.
   543	  gate-affected-target-set:
   544	    name: "gate · affected-set (ADR-0554, binding workspace coverage)"
   545	    runs-on: ubuntu-latest
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '543,929p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   543	  gate-affected-target-set:
   544	    name: "gate · affected-set (ADR-0554, binding workspace coverage)"
   545	    runs-on: ubuntu-latest
   546	    # SAFETY RAIL (ADR-0554 D7): bound the cold full-workspace rebuild so a runaway/exhaustion
   547	    # (a wedged buck2 action, a non-terminating compile) cannot burn the runner indefinitely.
   548	    # Derivation: ≈4x the ADR-0554-measured warm full run (4m35s cold / 5m45s incl. tests, lines
   549	    # 56-58) — fires only on a genuine runaway, never on a healthy cold build.
   550	    timeout-minutes: 45
   551	    steps:
   552	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   553	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   554	        with:
   555	          persist-credentials: false
   556	          # Full history: the merge-base diff anchor + the materialization boundary both
   557	          # derive from git history; a shallow checkout collapses them (PM1).
   558	          fetch-depth: 0
   559	      - name: Install buck2 (digest-pinned prebuilt release)
   560	        run: infra/ci/install-buck2.sh
   561	      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
   562	      # validates the toolchain on every run.
   563	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   564	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   565	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   566	        with:
   567	          path: |
   568	            ~/.rustup/toolchains
   569	            ~/.rustup/update-hashes
   570	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   571	      # Same rustup serialization rationale as the buck2 lane above: parallel buck2 actions
   572	      # racing a cold rustup install collide on shared download state.
   573	      - name: Pre-provision pinned rust toolchain and PR metadata preflight
   574	        env:
   575	          EVENT_NAME: ${{ github.event_name }}
   576	          PR_NUMBER: ${{ github.event.pull_request.number || '' }}
   577	          # Live-fetch token (same read-scoped github.token precedent as the merge-base
   578	          # baseline step below): the pull_request event payload freezes title/body at
   579	          # trigger time, so a `## Code Review` stamp applied via `gh pr edit` followed by
   580	          # `gh run rerun --failed` used to re-validate the stale pre-stamp body and stay
   581	          # RED (CodeReviewRequired), forcing empty-commit pushes purely to mint a fresh
   582	          # payload. Fetching the live PR at execution time keeps rerun semantics honest
   583	          # with no new mechanism: same admission bin, same body_path, same triggers.
   584	          GH_TOKEN: ${{ github.token }}
   585	        run: |
   586	          set -euo pipefail
   587	          rustup toolchain install
   588	          rustc --version
   589	          if [ "${EVENT_NAME}" = "pull_request" ]; then
   590	            pr_title="$(gh api "repos/${GITHUB_REPOSITORY}/pulls/${PR_NUMBER}" --jq '.title')"
   591	            body_path="${RUNNER_TEMP}/pull-request-body.md"
   592	            gh api "repos/${GITHUB_REPOSITORY}/pulls/${PR_NUMBER}" --jq '.body // ""' > "${body_path}"
   593	            buck2 run //libs/oya-check-pr-traceability:pr-traceability-admission-bin -- \
   594	              --pr-title "${pr_title}" \
   595	              --pr-body "${body_path}" \
   596	              --require-code-review
   597	          fi
   598	      # Restore buck-out read-only (ADR-0554 D9). Same stable per-dependency/toolchain-set key as the buck2
   599	      # lane: warmth is 100% in buck-out/v2/cache + buck-out/v2/art (path-relocatable); ~/.buck2
   600	      # and ~/.buck are daemon pid/endpoint/log scratch and are deliberately NOT cached. dev-push
   601	      # is the sole writer (the save step below is push-to-dev-gated), so PR runs read only — no
   602	      # per-commit multi-GB write-churn/eviction (see the buck2-lane rationale above).
   603	      # Reclaim preinstalled ubuntu-latest bloat (.NET/Android/GHC/CodeQL/preloaded Docker images:
   604	      # ~25-30 GiB) BEFORE the multi-GB buck-out restore. This lane decompresses a ~5.78 GiB buck-out
   605	      # blob (~12-15 GiB on disk) on top of a fetch-depth:0 monorepo checkout, exhausting the ~14 GiB
   606	      # free on / on GitHub-hosted ubuntu-latest; FRIC-017 recurred on PR #741 (No space left on device
   607	      # at this restore, before any build ran). Hermetic: removes only vendor preinstall dirs that no
   608	      # oya/buck2 action consumes; touches NO repo content and NO cache (buck-out / ~/.rustup / the
   609	      # restored blob untouched), so the cold==warm integrity canary (ADR-0556/0560) is unaffected. df
   610	      # is emitted so a genuine disk-NEED growth surfaces as a true RED instead of being masked.
   611	      - name: Reclaim runner disk before warm restore (FRIC-017 preflight)
   612	        # Rust-first, data-driven preflight (ADR-0548 pipeline-as-product): retires the two
   613	        # duplicated inline `sudo rm -rf` blocks. The policy (reclaim_dirs + min_free_gib_after)
   614	        # lives in runner-disk-reclaim-policy.json; the bin best-effort removes the profile's
   615	        # vendor preinstall dirs and logs structured disk-before/after plus a JSON operator
   616	        # artifact. Policy is explicit fail-closed: threshold-miss exits INFRA-RED unless a future
   617	        # caller supplies a typed fail-open waiver, so the required context cannot silently green on
   618	        # insufficient runner capacity. Built as the runner user (buck2 on user PATH; daemon must
   619	        # not run as root); only the prebuilt binary is sudo'd (needs root for root-owned dirs).
   620	        run: |
   621	          # Build as the runner user (buck2 on user PATH; never run buck2 daemon as root —
   622	          # that corrupts cache/daemon ownership). Then sudo ONLY the prebuilt binary (needs
   623	          # root solely to remove the root-owned vendor preinstall dirs).
   624	          BIN="$(buck2 build //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin --show-output 2>/dev/null | awk '{print $2}')"
   625	          sudo -E "$BIN" \
   626	            --profile github-hosted-ubuntu-latest \
   627	            --infra-red-policy fail-closed \
   628	            --artifact-out "${RUNNER_TEMP}/runner-disk-reclaim-affected-set.json"
   629	      - name: Restore buck-out (read-only; dev-push is the sole writer)
   630	        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   631	        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
   632	        with:
   633	          path: buck-out
   634	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   635	          restore-keys: |
   636	            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
   637	      # KEEPS materializing (not converted to ADR-0556 D5 QW-1 artifact reuse): same rationale
   638	      # as the buck2 lane — the cone's gate tests consume the per-job merge-base frozen
   639	      # baseline (ADR-0551), and this lane's own build-health baseline below is per-job by
   640	      # design (ADR-0554 round-3).
   641	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   642	        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   643	      - name: Fetch base ref for the merge-base anchor
   644	        if: ${{ github.event_name == 'pull_request' }}
   645	        env:
   646	          BASE_REF: ${{ github.base_ref || 'dev' }}
   647	        run: git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}"
   648	      # ── BUILD-HEALTH BASELINE (ADR-0554 D9 same-root build, round-5; ADR-0551 merge-base frozen
   649	      #    pattern). On a pull_request, derive the affected-set plan FIRST. Most PRs stay in the
   650	      #    affected cone and do not need a merge-base full-workspace build at all; only a derived
   651	      #    FULL decision needs the MERGE-BASE build-health baseline used by the ratchet. When FULL
   652	      #    is required, materialize that baseline IN THE MAIN ROOT so it shares the warm ./buck-out
   653	      #    restored above (the merge-base IS a dev commit, so the dev-keyed buck-out is near-fully
   654	      #    warm for it). We detach the SAME working tree to the merge-base COMMITTED tree-ish (the
   655	      #    candidate working tree is removed from disk for the build), run the full keep-going
   656	      #    build, capture per-target pass/fail, then a TRAP restores the candidate on EXIT. The
   657	      #    affected-set FULL tier grandfathers targets already failing at the merge-base and blocks
   658	      #    only REGRESSIONS. Skipped for push/merge_group/dispatch (the admission tier is a hard
   659	      #    full build — no grandfathering).
   660	      #
   661	      #    ANTI-LAUNDERING (ADR-0554 D6, preserved): the baseline failure-set comes ENTIRELY from
   662	      #    the merge-base COMMITTED tree (git object history — candidate-uncontrollable); during the
   663	      #    baseline build the candidate working tree is GONE from disk, so it cannot feed the
   664	      #    baseline; the report reaches the verdict ONLY via --baseline-report. The warm ./buck-out
   665	      #    is a content-addressed substrate — a buck2 hit is bit-identical to a cold build (ADR-0556
   666	      #    D1/D2) — so warmth changes only wall-clock, never the baseline SOURCE. Warm-eligible
   667	      #    under ADR-0556 with no policy change (trusted-author, content-addressed; not the
   668	      #    integrity-canary/release cold floor). GH #899 activates the trusted D8 consumer first:
   669	      #    use an exact push-to-dev baseline artifact when provenance and schema validate, else
   670	      #    fail closed to the same in-job merge-base rebuild below.
   671	      - name: Materialize merge-base build-health baseline when affected-set needs FULL
   672	        if: ${{ github.event_name == 'pull_request' }}
   673	        env:
   674	          BASE_REF: ${{ github.base_ref || 'dev' }}
   675	          GH_TOKEN: ${{ github.token }}
   676	        run: |
   677	          set -euo pipefail
   678	          merge_base="$(git merge-base "origin/${BASE_REF}" HEAD)"
   679	          orig_ref="$(git rev-parse HEAD)"
   680	          candidate_toolchain="$(awk -F'"' '/channel/ {print $2; exit}' rust-toolchain.toml)"
   681	          decision_log="${RUNNER_TEMP}/affected-set-derive.log"
   682	          full_required_marker="${RUNNER_TEMP}/affected-set-full-required"
   683	          echo "false" > "${full_required_marker}"
   684	          gate_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-affected-set-bin --show-output | awk '{print $2}')"
   685	          telemetry_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-step-telemetry-bin --show-output | awk '{print $2}')"
   686	          ci_step_telemetry="${RUNNER_TEMP}/oya-cloud-ci-step-telemetry"
   687	          cp "${telemetry_bin}" "${ci_step_telemetry}"
   688	          chmod +x "${ci_step_telemetry}"
   689	          echo "affected-set preflight: derive plan before merge-base baseline"
   690	          "${ci_step_telemetry}" --phase derive-affected-set-tier -- "${gate_bin}" \
   691	            --policy ci/facade/affected-target-set/affected-set-policy.json \
   692	            --base "origin/${BASE_REF}" --mode auto --derive-only \
   693	            --decision-artifact-out "${RUNNER_TEMP}/affected-set-derive-decision.json" \
   694	            | tee "${decision_log}"
   695	          if grep -Eq '^affected-set: (decision=FULL|ESCALATE to FULL)' "${decision_log}"; then
   696	            echo "true" > "${full_required_marker}"
   697	            echo "affected-set preflight: FULL decision requires merge-base build-health baseline"
   698	          else
   699	            echo "affected-set preflight: derived non-FULL decision; skipping merge-base baseline"
   700	            exit 0
   701	          fi
   702	          echo "build-health baseline: merge-base=${merge_base} candidate=${orig_ref}"
   703	          artifact_name="build-health-baseline-${merge_base}"
   704	          echo "build-health baseline: attempting trusted dev-push artifact ${artifact_name}"
   705	          try_trusted_baseline_artifact() {
   706	            if ! command -v gh >/dev/null 2>&1; then
   707	              echo "build-health baseline: gh unavailable; falling back to in-job merge-base rebuild"
   708	              return 1
   709	            fi
   710	            local runs_json="${RUNNER_TEMP}/build-health-trusted-runs.json"
   711	            local artifacts_json="${RUNNER_TEMP}/build-health-trusted-artifacts.json"
   712	            local trusted_zip="${RUNNER_TEMP}/build-health-trusted.zip"
   713	            local trusted_dir="${RUNNER_TEMP}/build-health-trusted"
   714	            local trusted_report="${trusted_dir}/build-health-admission-report.json"
   715	
   716	            gh api --method GET -H "Accept: application/vnd.github+json" \
   717	              "repos/${GITHUB_REPOSITORY}/actions/workflows/oya-ci-required.yml/runs" \
   718	              -f branch=dev -f event=push -f status=success -F per_page=50 > "${runs_json}" \
   719	              || { echo "build-health baseline: trusted run lookup failed; falling back to in-job rebuild"; return 1; }
   720	
   721	            local run_id
   722	            run_id="$(python3 - "${runs_json}" "${merge_base}" <<'PY'
   723	          import json
   724	          import sys
   725	
   726	          path, merge_base = sys.argv[1], sys.argv[2]
   727	          with open(path, encoding="utf-8") as fh:
   728	              payload = json.load(fh)
   729	          for run in payload.get("workflow_runs", []):
   730	              if (
   731	                  run.get("head_sha") == merge_base
   732	                  and run.get("event") == "push"
   733	                  and run.get("head_branch") == "dev"
   734	                  and run.get("conclusion") == "success"
   735	              ):
   736	                  print(run["id"])
   737	                  break
   738	          PY
   739	            )"
   740	            if [ -z "${run_id}" ]; then
   741	              echo "build-health baseline: no successful trusted push-to-dev run for ${merge_base}; falling back to in-job rebuild"
   742	              return 1
   743	            fi
   744	
   745	            gh api --method GET -H "Accept: application/vnd.github+json" \
   746	              "repos/${GITHUB_REPOSITORY}/actions/runs/${run_id}/artifacts" \
   747	              -F per_page=100 > "${artifacts_json}" \
   748	              || { echo "build-health baseline: trusted artifact lookup failed; falling back to in-job rebuild"; return 1; }
   749	
   750	            local artifact_id
   751	            artifact_id="$(python3 - "${artifacts_json}" "${artifact_name}" <<'PY'
   752	          import json
   753	          import sys
   754	
   755	          path, expected_name = sys.argv[1], sys.argv[2]
   756	          with open(path, encoding="utf-8") as fh:
   757	              payload = json.load(fh)
   758	          for artifact in payload.get("artifacts", []):
   759	              if artifact.get("name") == expected_name and not artifact.get("expired", True):
   760	                  print(artifact["id"])
   761	                  break
   762	          PY
   763	            )"
   764	            if [ -z "${artifact_id}" ]; then
   765	              echo "build-health baseline: no unexpired exact artifact ${artifact_name} on trusted run ${run_id}; falling back to in-job rebuild"
   766	              return 1
   767	            fi
   768	
   769	            gh api "repos/${GITHUB_REPOSITORY}/actions/artifacts/${artifact_id}/zip" > "${trusted_zip}" \
   770	              || { echo "build-health baseline: artifact download failed; falling back to in-job rebuild"; return 1; }
   771	            rm -rf "${trusted_dir}"
   772	            mkdir -p "${trusted_dir}"
   773	            python3 -m zipfile -e "${trusted_zip}" "${trusted_dir}" \
   774	              || { echo "build-health baseline: artifact unzip failed; falling back to in-job rebuild"; return 1; }
   775	
   776	            if ! python3 - "${trusted_report}" <<'PY'
   777	          import json
   778	          import os
   779	          import sys
   780	
   781	          path = sys.argv[1]
   782	          if not os.path.getsize(path):
   783	              raise SystemExit("empty report file")
   784	          with open(path, encoding="utf-8") as fh:
   785	              payload = json.load(fh)
   786	          results = payload.get("results")
   787	          if not isinstance(results, dict) or not results:
   788	              raise SystemExit("missing or empty results object")
   789	          PY
   790	            then
   791	              echo "build-health baseline: trusted artifact schema/emptiness invalid; falling back to in-job rebuild"
   792	              return 1
   793	            fi
   794	            cp "${trusted_report}" "${RUNNER_TEMP}/build-health-baseline.json"
   795	            echo "build-health baseline: trusted artifact hit run_id=${run_id} artifact_id=${artifact_id}"
   796	            return 0
   797	          }
   798	          if try_trusted_baseline_artifact; then
   799	            exit 0
   800	          fi
   801	          # ALWAYS restore the candidate tree on EXIT — a failed baseline build can never strand CI
   802	          # on the merge-base tree (the subsequent Binding affected-set step runs on the candidate).
   803	          # NOTE: if the timeout-minutes:45 rail SIGKILLs this build, the bash EXIT trap does NOT
   804	          # fire (tree left detached at merge-base) — but a timeout fails the whole job RED → fan-in
   805	          # RED, so it is fail-closed and never produces a wrong-baseline verdict.
   806	          restore_candidate_tree() {
   807	            local exit_status="$?"
   808	            git checkout --quiet --detach "${orig_ref}" 2>/dev/null || git checkout --quiet "${orig_ref}"
   809	            if [ "${candidate_toolchain}" != "${baseline_toolchain:-${candidate_toolchain}}" ]; then
   810	              echo "build-health baseline: cleaning buck-out after restoring candidate toolchain ${candidate_toolchain}"
   811	              buck2 clean
   812	            fi
   813	            exit "${exit_status}"
   814	          }
   815	          trap restore_candidate_tree EXIT
   816	          # Detach the MAIN working tree to the merge-base COMMITTED tree-ish: the baseline is
   817	          # computed from git object history (candidate-uncontrollable), and the candidate working
   818	          # tree is removed from disk for the build, so a PR cannot grow its own baseline to
   819	          # launder a regression.
   820	          git checkout --quiet --detach "${merge_base}"
   821	          baseline_toolchain="$(awk -F'"' '/channel/ {print $2; exit}' rust-toolchain.toml)"
   822	          rustup toolchain install
   823	          if [ "${candidate_toolchain}" != "${baseline_toolchain}" ]; then
   824	            echo "build-health baseline: Rust toolchain changed ${baseline_toolchain} -> ${candidate_toolchain}; isolating buck-out"
   825	            buck2 clean
   826	          fi
   827	          # Build the whole merge-base workspace keep-going. Same-channel PRs share warm ./buck-out;
   828	          # Rust-channel bump PRs intentionally go cold on both sides to avoid mixed-rustc rlibs.
   829	          # The build is EXPECTED to be non-zero (dev carries pre-existing breakage) — that is the
   830	          # baseline, not a failure, so we never propagate its exit code.
   831	          "${ci_step_telemetry}" --phase materialize-merge-base-build-health-baseline -- \
   832	            buck2 build //... --keep-going \
   833	              --build-report "${RUNNER_TEMP}/build-health-baseline.json" || true
   834	          test -s "${RUNNER_TEMP}/build-health-baseline.json" \
   835	            || { echo "build-health: FATAL empty merge-base baseline report"; exit 1; }
   836	      - name: Binding affected-set build + test (cone-binding; FULL tier = build-health ratchet)
   837	        env:
   838	          EVENT_NAME: ${{ github.event_name }}
   839	          BASE_REF: ${{ github.base_ref || 'dev' }}
   840	        run: |
   841	          set -euo pipefail
   842	          gate_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-affected-set-bin --show-output | awk '{print $2}')"
   843	          telemetry_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-step-telemetry-bin --show-output | awk '{print $2}')"
   844	          if [ "${EVENT_NAME}" = "pull_request" ]; then
   845	            # PR tier: auto (cone-binding, hard-fail on a NEW break in the changed cone); a FULL
   846	            # escalation runs the build-health ratchet against the merge-base baseline. The
   847	            # baseline is only materialized when the derive preflight proves FULL is needed.
   848	            baseline_args=()
   849	            if [ -f "${RUNNER_TEMP}/affected-set-full-required" ] \
   850	                && grep -qx 'true' "${RUNNER_TEMP}/affected-set-full-required"; then
   851	              test -s "${RUNNER_TEMP}/build-health-baseline.json" \
   852	                || { echo "affected-set: FATAL missing baseline after FULL derive preflight"; exit 1; }
   853	              baseline_args=(--baseline-report "${RUNNER_TEMP}/build-health-baseline.json")
   854	            fi
   855	            "${telemetry_bin}" --phase binding-affected-set-build-test -- "${gate_bin}" \
   856	              --policy ci/facade/affected-target-set/affected-set-policy.json \
   857	              --base "origin/${BASE_REF}" --mode auto \
   858	              --decision-artifact-out "${RUNNER_TEMP}/affected-set-binding-decision.json" \
   859	              "${baseline_args[@]}"
   860	          else
   861	            # Admission/integration tier (merge_group/push/dispatch): hard full build+test — the
   862	            # integration tip MUST be green, no grandfathering. ADR-0554 D7: this run ALSO captures
   863	            # a build-report at ${RUNNER_TEMP}/build-health-admission-report.json (the binary's
   864	            # stable RUNNER_TEMP-anchored path) as a pure byproduct; the verdict is unchanged
   865	            # (non-empty failure set = hard fail) and the report is uploaded below only on
   866	            # trusted push-to-dev.
   867	            "${telemetry_bin}" --phase binding-affected-set-build-test -- "${gate_bin}" \
   868	              --policy ci/facade/affected-target-set/affected-set-policy.json \
   869	              --base "origin/${BASE_REF}" --mode full \
   870	              --decision-artifact-out "${RUNNER_TEMP}/affected-set-binding-decision.json"
   871	          fi
   872	      # ── FULL-GRAPH CACHE SAVE (ADR-0554 D9; sole canonical writer). On dev-push this job runs
   873	      #    --mode full (buck2 build + test //...), so buck-out is populated with the FULL workspace
   874	      #    graph — not just //ci/... as in the buck2 lane. Saving here means PR
   875	      #    gate-affected-target-set restores a full-graph buck-out, so the same-root merge-base baseline
   876	      #    build is near-fully-warm (the merge-base IS a recent dev commit whose full-graph buck-out
   877	      #    was just saved). The buck2 lane restores the same key and is a subset hit. One save step,
   878	      #    one job, one key — no two-writer race. Runs AFTER the Binding step (buck-out fully
   879	      #    populated). Guarded push-to-dev so PRs remain read-only (restore-only via the step above).
   880	      #    Size note: the full-graph buck-out is one blob per stable key (overwrites, non-accumulating
   881	      #    due to dev-push-sole-writer), bounded by the dependency-set change cadence — worth watching
   882	      #    against the GitHub 10GB cache limit; NativeLink CAS (ADR-0560) removes this at cutover.
   883	      - name: Save buck-out (dev-push only; sole canonical full-graph writer)
   884	        if: ${{ github.event_name == 'push' && github.ref == 'refs/heads/dev' }}
   885	        # actions/cache/save@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   886	        uses: actions/cache/save@27d5ce7f107fe9357f9df03efb73ab90386fccae
   887	        with:
   888	          path: buck-out
   889	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   890	      # ── BUILD-HEALTH BASELINE PRODUCER (ADR-0554 D7; ADR-0556 D5 QW-1 trusted-producer +
   891	      #    postmerge-dev-trunk warmth class). ONLY a trusted push-to-dev publishes the admission
   892	      #    build-report as the merge-base-to-be baseline artifact — NOT merge_group, NOT
   893	      #    pull_request — so the artifact namespace stays clean of attacker-controllable producers
   894	      #    (part of the DEFERRED D8 consumer's anti-laundering defense; D8 trusts workflow_run
   895	      #    PROVENANCE, never the artifact name). Producer-only: nothing consumes this yet, so it is
   896	      #    sound + harmless (no merge-authority change, no new permissions). The artifact is on the
   897	      #    critical path of BOTH the deferred D8 cross-run consumer AND the ADR-0560 warm-CAS.
   898	      - name: Upload build-health baseline artifact (trusted push-to-dev producer)
   899	        if: ${{ github.event_name == 'push' && github.ref == 'refs/heads/dev' }}
   900	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   901	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   902	        with:
   903	          name: build-health-baseline-${{ github.sha }}
   904	          path: ${{ runner.temp }}/build-health-admission-report.json
   905	          retention-days: 90
   906	          if-no-files-found: error
   907	      - name: Upload affected-set operator artifacts
   908	        if: always()
   909	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   910	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   911	        with:
   912	          name: affected-set-operator-artifacts
   913	          path: |
   914	            ${{ runner.temp }}/runner-disk-reclaim-affected-set.json
   915	            ${{ runner.temp }}/affected-set-derive-decision.json
   916	            ${{ runner.temp }}/affected-set-binding-decision.json
   917	          retention-days: 30
   918	          if-no-files-found: warn
   919	
   920	  # ── LIVE-POSTGRES DURABLE-SUBSTRATE LANES (#101/#901). Runs the env-gated
   921	  #    cross-tenant-deny / RLS / CDC / SCIM durability integration tests against
   922	  #    CONTAINERIZED Postgres and GATES merge. Both adapter and facade groups block
   923	  #    the single required `oya-ci-required` context.
   924	  #
   925	  #    SPLIT SAFETY (#901): adapter and facade groups run in parallel only because
   926	  #    each job owns an independent Postgres service container and repeats the
   927	  #    deterministic bootstrap. Inside each group, `--num-threads 1` and sequential
   928	  #    target invocations remain because the harnesses in that group still share a
   929	  #    local database.
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '920,1236p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   920	  # ── LIVE-POSTGRES DURABLE-SUBSTRATE LANES (#101/#901). Runs the env-gated
   921	  #    cross-tenant-deny / RLS / CDC / SCIM durability integration tests against
   922	  #    CONTAINERIZED Postgres and GATES merge. Both adapter and facade groups block
   923	  #    the single required `oya-ci-required` context.
   924	  #
   925	  #    SPLIT SAFETY (#901): adapter and facade groups run in parallel only because
   926	  #    each job owns an independent Postgres service container and repeats the
   927	  #    deterministic bootstrap. Inside each group, `--num-threads 1` and sequential
   928	  #    target invocations remain because the harnesses in that group still share a
   929	  #    local database.
   930	  gate-live-postgres-adapters:
   931	    name: "gate-live-postgres-adapters (durable adapters: RLS / CDC / SCIM, #901)"
   932	    runs-on: ubuntu-latest
   933	    timeout-minutes: 25
   934	    services:
   935	      postgres:
   936	        image: postgres:16
   937	        env:
   938	          POSTGRES_USER: postgres
   939	          POSTGRES_PASSWORD: postgres
   940	          POSTGRES_DB: oyatie
   941	        ports:
   942	          - 5432:5432
   943	        options: >-
   944	          --health-cmd "pg_isready -U postgres -d oyatie"
   945	          --health-interval 5s
   946	          --health-timeout 5s
   947	          --health-retries 20
   948	    env:
   949	      OYA_PG_ADMIN_URL: postgres://postgres:postgres@127.0.0.1:5432/oyatie
   950	      OYA_PG_APP_URL: postgres://oya_app:app@127.0.0.1:5432/oyatie
   951	    steps:
   952	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   953	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   954	        with:
   955	          persist-credentials: false
   956	      - name: Install buck2 (digest-pinned prebuilt release)
   957	        run: infra/ci/install-buck2.sh
   958	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   959	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   960	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   961	        with:
   962	          path: |
   963	            ~/.rustup/toolchains
   964	            ~/.rustup/update-hashes
   965	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   966	      - name: Pre-provision pinned Rust toolchain for Buck2 live tests
   967	        run: |
   968	          set -euo pipefail
   969	          rustup toolchain install
   970	          rustc --version
   971	      - name: Install postgresql-client for the bootstrap
   972	        run: |
   973	          set -euo pipefail
   974	          sudo apt-get update
   975	          sudo apt-get install -y --no-install-recommends postgresql-client
   976	          psql --version
   977	      - name: Bootstrap app role + durable schemas/roles (admin, adapters)
   978	        env:
   979	          PGPASSWORD: postgres
   980	        run: |
   981	          set -euo pipefail
   982	          ADMIN_PSQL=(psql -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U postgres -d oyatie)
   983	          "${ADMIN_PSQL[@]}" -c "DO \$\$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='oya_app') THEN CREATE ROLE oya_app LOGIN PASSWORD 'app' NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE; END IF; END \$\$;"
   984	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql
   985	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql
   986	          "${ADMIN_PSQL[@]}" -c "GRANT tenancy_lifecycle_runtime TO oya_app;"
   987	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql
   988	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql
   989	          "${ADMIN_PSQL[@]}" -c "GRANT identity_scim_runtime TO oya_app;"
   990	          "${ADMIN_PSQL[@]}" -c "SELECT rolname, rolsuper, rolbypassrls FROM pg_roles WHERE rolname IN ('postgres','oya_app','tenancy_lifecycle_runtime','identity_scim_runtime') ORDER BY rolname;"
   991	          server_version="$("${ADMIN_PSQL[@]}" -Atqc "SHOW server_version;")"
   992	          cat > "${RUNNER_TEMP}/live-postgres-adapters-bootstrap-provenance.json" <<JSON
   993	          {
   994	            "schema_version": 2,
   995	            "artifact_type": "cloud_ci_operator_artifact",
   996	            "artifact_id": "live-postgres-bootstrap-provenance",
   997	            "gate_id": "gate-live-postgres-adapters",
   998	            "lane": "adapters",
   999	            "postgres": {
  1000	              "image": "postgres:16",
  1001	              "server_version": "${server_version}",
  1002	              "database": "oyatie",
  1003	              "host": "127.0.0.1",
  1004	              "port": 5432
  1005	            },
  1006	            "roles": [
  1007	              {"name": "postgres", "purpose": "admin bootstrap superuser; DSN/password redacted"},
  1008	              {"name": "oya_app", "purpose": "non-superuser NOBYPASSRLS app login; DSN/password redacted"},
  1009	              {"name": "tenancy_lifecycle_runtime", "purpose": "tenancy runtime role granted to oya_app"},
  1010	              {"name": "identity_scim_runtime", "purpose": "SCIM runtime role granted to oya_app"}
  1011	            ],
  1012	            "migrations": [
  1013	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql",
  1014	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql",
  1015	              "iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql",
  1016	              "iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql"
  1017	            ],
  1018	            "source_revision": "${GITHUB_SHA}",
  1019	            "retention_and_pii": {
  1020	              "retention_days": 30,
  1021	              "pii": "none; local CI database metadata and repo paths only",
  1022	              "secret_redaction": "admin/app DSNs, passwords, tenant secrets, idempotency keys, and tokens are not emitted"
  1023	            }
  1024	          }
  1025	          JSON
  1026	      - name: buck2 test — durable adapters (admin=superuser, app=app-role)
  1027	        env:
  1028	          OYA_DATA_LIVE_POSTGRES: "1"
  1029	          OYA_DATA_POSTGRES_ADMIN_URL: ${{ env.OYA_PG_ADMIN_URL }}
  1030	          OYA_DATA_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1031	          OYA_OUTBOX_LIVE_POSTGRES: "1"
  1032	          OYA_OUTBOX_POSTGRES_ADMIN_URL: ${{ env.OYA_PG_ADMIN_URL }}
  1033	          OYA_OUTBOX_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1034	          OYA_BACKBONE_LIVE_POSTGRES: "1"
  1035	          OYA_BACKBONE_POSTGRES_URL: ${{ env.OYA_PG_ADMIN_URL }}
  1036	          OYA_BACKBONE_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1037	        run: |
  1038	          set -euo pipefail
  1039	          LIVE_ENV=(
  1040	            --env RUST_TEST_THREADS=1
  1041	            --env OYA_DATA_LIVE_POSTGRES="${OYA_DATA_LIVE_POSTGRES}"
  1042	            --env OYA_DATA_POSTGRES_ADMIN_URL="${OYA_DATA_POSTGRES_ADMIN_URL}"
  1043	            --env OYA_DATA_POSTGRES_APP_URL="${OYA_DATA_POSTGRES_APP_URL}"
  1044	            --env OYA_OUTBOX_LIVE_POSTGRES="${OYA_OUTBOX_LIVE_POSTGRES}"
  1045	            --env OYA_OUTBOX_POSTGRES_ADMIN_URL="${OYA_OUTBOX_POSTGRES_ADMIN_URL}"
  1046	            --env OYA_OUTBOX_POSTGRES_APP_URL="${OYA_OUTBOX_POSTGRES_APP_URL}"
  1047	            --env OYA_BACKBONE_LIVE_POSTGRES="${OYA_BACKBONE_LIVE_POSTGRES}"
  1048	            --env OYA_BACKBONE_POSTGRES_URL="${OYA_BACKBONE_POSTGRES_URL}"
  1049	            --env OYA_BACKBONE_POSTGRES_APP_URL="${OYA_BACKBONE_POSTGRES_APP_URL}"
  1050	          )
  1051	          buck2 test --local-only --num-threads 1 //libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest -- "${LIVE_ENV[@]}"
  1052	          buck2 test --local-only --num-threads 1 //libs/oya-data-outbox-adapter-postgres:oya-data-outbox-adapter-postgres-unittest -- "${LIVE_ENV[@]}"
  1053	          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-unittest -- "${LIVE_ENV[@]}"
  1054	          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-live -- "${LIVE_ENV[@]}"
  1055	          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-unittest -- "${LIVE_ENV[@]}"
  1056	          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-live -- "${LIVE_ENV[@]}"
  1057	      - name: Upload live-postgres adapter bootstrap provenance
  1058	        if: always()
  1059	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1060	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  1061	        with:
  1062	          name: live-postgres-adapters-bootstrap-provenance
  1063	          path: ${{ runner.temp }}/live-postgres-adapters-bootstrap-provenance.json
  1064	          retention-days: 30
  1065	          if-no-files-found: warn
  1066	
  1067	  gate-live-postgres-facades:
  1068	    name: "gate-live-postgres-facades (durable facades: tenant lifecycle / SCIM, #901)"
  1069	    runs-on: ubuntu-latest
  1070	    timeout-minutes: 25
  1071	    services:
  1072	      postgres:
  1073	        image: postgres:16
  1074	        env:
  1075	          POSTGRES_USER: postgres
  1076	          POSTGRES_PASSWORD: postgres
  1077	          POSTGRES_DB: oyatie
  1078	        ports:
  1079	          - 5432:5432
  1080	        options: >-
  1081	          --health-cmd "pg_isready -U postgres -d oyatie"
  1082	          --health-interval 5s
  1083	          --health-timeout 5s
  1084	          --health-retries 20
  1085	    env:
  1086	      OYA_PG_ADMIN_URL: postgres://postgres:postgres@127.0.0.1:5432/oyatie
  1087	      OYA_PG_APP_URL: postgres://oya_app:app@127.0.0.1:5432/oyatie
  1088	    steps:
  1089	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1090	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
  1091	        with:
  1092	          persist-credentials: false
  1093	      - name: Install buck2 (digest-pinned prebuilt release)
  1094	        run: infra/ci/install-buck2.sh
  1095	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
  1096	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1097	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
  1098	        with:
  1099	          path: |
  1100	            ~/.rustup/toolchains
  1101	            ~/.rustup/update-hashes
  1102	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
  1103	      - name: Pre-provision pinned Rust toolchain for Buck2 live tests
  1104	        run: |
  1105	          set -euo pipefail
  1106	          rustup toolchain install
  1107	          rustc --version
  1108	      - name: Install postgresql-client for the bootstrap
  1109	        run: |
  1110	          set -euo pipefail
  1111	          sudo apt-get update
  1112	          sudo apt-get install -y --no-install-recommends postgresql-client
  1113	          psql --version
  1114	      - name: Bootstrap app role + durable schemas/roles (admin, facades)
  1115	        env:
  1116	          PGPASSWORD: postgres
  1117	        run: |
  1118	          set -euo pipefail
  1119	          ADMIN_PSQL=(psql -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U postgres -d oyatie)
  1120	          "${ADMIN_PSQL[@]}" -c "DO \$\$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='oya_app') THEN CREATE ROLE oya_app LOGIN PASSWORD 'app' NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE; END IF; END \$\$;"
  1121	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql
  1122	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql
  1123	          "${ADMIN_PSQL[@]}" -c "GRANT tenancy_lifecycle_runtime TO oya_app;"
  1124	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql
  1125	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql
  1126	          "${ADMIN_PSQL[@]}" -c "GRANT identity_scim_runtime TO oya_app;"
  1127	          "${ADMIN_PSQL[@]}" -c "SELECT rolname, rolsuper, rolbypassrls FROM pg_roles WHERE rolname IN ('postgres','oya_app','tenancy_lifecycle_runtime','identity_scim_runtime') ORDER BY rolname;"
  1128	          server_version="$("${ADMIN_PSQL[@]}" -Atqc "SHOW server_version;")"
  1129	          cat > "${RUNNER_TEMP}/live-postgres-facades-bootstrap-provenance.json" <<JSON
  1130	          {
  1131	            "schema_version": 2,
  1132	            "artifact_type": "cloud_ci_operator_artifact",
  1133	            "artifact_id": "live-postgres-bootstrap-provenance",
  1134	            "gate_id": "gate-live-postgres-facades",
  1135	            "lane": "facades",
  1136	            "postgres": {
  1137	              "image": "postgres:16",
  1138	              "server_version": "${server_version}",
  1139	              "database": "oyatie",
  1140	              "host": "127.0.0.1",
  1141	              "port": 5432
  1142	            },
  1143	            "roles": [
  1144	              {"name": "postgres", "purpose": "admin bootstrap superuser; DSN/password redacted"},
  1145	              {"name": "oya_app", "purpose": "non-superuser NOBYPASSRLS app login; DSN/password redacted"},
  1146	              {"name": "tenancy_lifecycle_runtime", "purpose": "tenancy runtime role granted to oya_app"},
  1147	              {"name": "identity_scim_runtime", "purpose": "SCIM runtime role granted to oya_app"}
  1148	            ],
  1149	            "migrations": [
  1150	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql",
  1151	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql",
  1152	              "iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql",
  1153	              "iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql"
  1154	            ],
  1155	            "source_revision": "${GITHUB_SHA}",
  1156	            "retention_and_pii": {
  1157	              "retention_days": 30,
  1158	              "pii": "none; local CI database metadata and repo paths only",
  1159	              "secret_redaction": "admin/app DSNs, passwords, tenant secrets, idempotency keys, and tokens are not emitted"
  1160	            }
  1161	          }
  1162	          JSON
  1163	      - name: buck2 test — durable facades (live test = app-role, non-live = in-memory)
  1164	        env:
  1165	          OYA_BACKBONE_LIVE_POSTGRES: "1"
  1166	          OYA_BACKBONE_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1167	        run: |
  1168	          set -euo pipefail
  1169	          FACADE_ENV=(
  1170	            --env RUST_TEST_THREADS=1
  1171	            --env OYA_BACKBONE_LIVE_POSTGRES="${OYA_BACKBONE_LIVE_POSTGRES}"
  1172	            --env OYA_BACKBONE_POSTGRES_APP_URL="${OYA_BACKBONE_POSTGRES_APP_URL}"
  1173	          )
  1174	          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-tests -- "${FACADE_ENV[@]}"
  1175	          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-e2e -- "${FACADE_ENV[@]}"
  1176	          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-unittest -- "${FACADE_ENV[@]}"
  1177	          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-acceptance -- "${FACADE_ENV[@]}"
  1178	      - name: Upload live-postgres facade bootstrap provenance
  1179	        if: always()
  1180	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1181	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  1182	        with:
  1183	          name: live-postgres-facades-bootstrap-provenance
  1184	          path: ${{ runner.temp }}/live-postgres-facades-bootstrap-provenance.json
  1185	          retention-days: 30
  1186	          if-no-files-found: warn
  1187	
  1188	
  1189	
  1190	  # ── THE FAN-IN. This is the single required context branch protection keys on. It has NO
  1191	  #    command of its own (Principle 1 — it NEVER re-runs a narrower subset): it is green IFF
  1192	  #    every gate lane above is green. `needs:` lists EVERY gate job; the gate-registration
  1193	  #    meta-test (in the firewall lane) asserts every in-tree gate crate is represented here.
  1194	  oya-ci-required:
  1195	    name: oya-ci-required
  1196	    runs-on: ubuntu-latest
  1197	    if: ${{ always() }}
  1198	    needs:
  1199	      - gate                    # the matrix of homogeneous gate lanes (success IFF every leg passed)
  1200	      - gate-generated-artifact-freshness          # bespoke: stale Cargo.lock + generated faces first diagnosis
  1201	      - gate-inventory-registry-drift     # bespoke: materialized == regenerated byte-parity
  1202	      - gate-baseline-ratchet  # bespoke: baseline ratchet + gate_registration meta-test
  1203	      - generated-output-diff-policy # generated outputs are never PR merge surfaces
  1204	      - buck2                   # hermetic buck2 build + affected gate tests
  1205	      - gate-affected-target-set       # binding workspace coverage: affected cone or fail-closed full run (ADR-0554)
  1206	      - gate-live-postgres-adapters # #901: durable adapter RLS / CDC / SCIM tests against isolated live Postgres
  1207	      - gate-live-postgres-facades  # #901: durable facade tenant lifecycle / SCIM tests against isolated live Postgres
  1208	    steps:
  1209	      - name: Fan-in verdict (green IFF every gate lane is green)
  1210	        run: |
  1211	          echo "Gate results:"
  1212	          echo "  gate (matrix)     = ${{ needs.gate.result }}"
  1213	          echo "  freshness         = ${{ needs.gate-generated-artifact-freshness.result }}"
  1214	          echo "  registry-drift    = ${{ needs.gate-inventory-registry-drift.result }}"
  1215	          echo "  cloud-ci-firewall = ${{ needs.gate-baseline-ratchet.result }}"
  1216	          echo "  generated-diff    = ${{ needs.generated-output-diff-policy.result }}"
  1217	          echo "  buck2             = ${{ needs.buck2.result }}"
  1218	          echo "  affected-set      = ${{ needs.gate-affected-target-set.result }}"
  1219	          echo "  live-postgres/adapters = ${{ needs.gate-live-postgres-adapters.result }}"
  1220	          echo "  live-postgres/facades  = ${{ needs.gate-live-postgres-facades.result }}"
  1221	          # A matrix job's aggregate `.result` is success IFF ALL legs succeeded, failure if any
  1222	          # leg failed, skipped if it never ran — so one check covers every gate in the matrix.
  1223	          if [ "${{ needs.gate.result }}" = "success" ] \
  1224	            && [ "${{ needs.gate-generated-artifact-freshness.result }}" = "success" ] \
  1225	            && [ "${{ needs.gate-inventory-registry-drift.result }}" = "success" ] \
  1226	            && [ "${{ needs.gate-baseline-ratchet.result }}" = "success" ] \
  1227	            && [ "${{ needs.generated-output-diff-policy.result }}" = "success" ] \
  1228	            && [ "${{ needs.buck2.result }}" = "success" ] \
  1229	            && [ "${{ needs.gate-affected-target-set.result }}" = "success" ] \
  1230	            && [ "${{ needs.gate-live-postgres-adapters.result }}" = "success" ] \
  1231	            && [ "${{ needs.gate-live-postgres-facades.result }}" = "success" ]; then
  1232	            echo "oya-ci-required: GREEN — every constituent gate lane passed."
  1233	            exit 0
  1234	          fi
  1235	          echo "oya-ci-required: RED — at least one constituent gate lane failed or was skipped."
  1236	          exit 1
exec
/bin/zsh -lc 'git ls-tree -r --name-only origin/dev ci/facade/affected-target-set' in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc 'git show origin/dev:ci/facade/affected-target-set/affected-set-policy.json | nl -ba' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
ci/facade/affected-target-set/BUCK
ci/facade/affected-target-set/Cargo.toml
ci/facade/affected-target-set/affected-set-policy.json
ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs
ci/facade/affected-target-set/src/bin/oya-cloud-ci-step-telemetry.rs
ci/facade/affected-target-set/src/lib.rs
ci/facade/affected-target-set/src/main.rs
ci/facade/affected-target-set/tests/affected_set.rs
 succeeded in 0ms:
     1	{
     2	  "_comment": "cloud-ci-affected-set binding workspace-coverage policy (DATA, not code). ADR-0554, converts FRIC-1781310000: the only binding buck2 lane was scoped to //cloud/cloud-ci/..., so code anywhere else could merge broken (PR #651 head cf16525 did not compile yet its buck2 lane was green). ALL repo-specifics live here; the Rust kernel hardcodes no oyatie path and runs on any buck2 repo by editing this pack. full_trigger_patterns are the rdeps-cone ESCAPE classes — graph-semantic files whose blast radius the owner()/rdeps() derivation cannot model (build config, toolchains, vendored third-party, Starlark macros, the lockfile): any touch escalates to the FULL workspace, mechanically, with no skip and no human judgment. require_owner_patterns are the classes that MUST map to an owning target; an existing file in these classes with no owner FAILS the lane (graph-invisible code is not made safe by running more targets).",
     3	  "gate_id": "cloud-ci-affected-set",
     4	  "schema_version": "1.0.0",
     5	  "universe": "//...",
     6	  "full_run_targets": [
     7	    "//..."
     8	  ],
     9	  "_full_trigger_note": "Two seam classes here. (1) Build config/macros whose blast radius the per-package rdeps cone cannot bound — .buckconfig + .buckconfig.local + .buckconfig.d/** (all read by buck2, all committable), toolchains/**, third-party/** (reindeer vendor + fixups), Starlark **/*.bzl + **/*.bxl, rust-toolchain.toml. (2) Buildfiles and PACKAGE files are handled by package_definition_basenames (escalate to FULL on any change) AND mirrored here as **/PACKAGE so a NEW PACKAGE file (which evaluates to [] and would otherwise look like a plain no-owner file) is never a silent no-op. Cargo.lock is deliberately NOT a trigger: buck2 never reads it — a dependency change that affects buck2 semantics MUST touch third-party/**; the cargo lanes + ADR-0539 freshness gate own lock hygiene.",
    10	  "full_trigger_patterns": [
    11	    ".buckconfig",
    12	    ".buckconfig.local",
    13	    ".buckconfig.d/**",
    14	    "toolchains/**",
    15	    "third-party/**",
    16	    "**/*.bzl",
    17	    "**/*.bxl",
    18	    "**/PACKAGE",
    19	    "rust-toolchain.toml"
    20	  ],
    21	  "require_owner_patterns": [
    22	    "**/*.rs",
    23	    "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-aarch64-adapter/linker.ld",
    24	    "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/linker.ld"
    25	  ],
    26	  "_package_sibling_note": "Cargo.toml + build.rs are NOT buck2 graph inputs (buck2 never reads them; reindeer/BUCK mirror them) and have no owner() BY DESIGN, so requiring an owner would refuse every manifest edit (proven by this lane's own first dogfood run). They are semantically bound to their crate, so they seed the ENCLOSING package target pattern; if that package does not exist the seed query fails and the lane escalates to FULL.",
    27	  "package_sibling_basenames": [
    28	    "Cargo.toml",
    29	    "build.rs"
    30	  ],
    31	  "_package_definition_note": "Ground-truth buck2 buildfile-name set in PRECEDENCE order: BUCK.v2 SHADOWS BUCK when both exist (empirically verified against this buck2 binary — adding an empty BUCK.v2 next to a real BUCK drops the BUCK targets). A change to ANY buildfile escalates to FULL (these basenames are also escape-class): a buildfile edit can add/remove targets or shadow the file dependents load, so its blast radius is NOT bounded by its own package's rdeps. owner() is empty for a buildfile by design, so seeding 'its package' alone would silently miss every dependent (F2). If buck2's [buildfile] name/extra_for_test config ever adds names, mirror them here — this list IS the repo's buildfile-name ground truth, not a single hand-set name.",
    32	  "package_definition_basenames": [
    33	    "BUCK.v2",
    34	    "BUCK"
    35	  ],
    36	  "cell_roots": {
    37	    "": "//"
    38	  },
    39	  "default_base_ref": "origin/dev",
    40	  "product_contract": {
    41	    "born_pack_shaped": "The escape-trigger classes, owner-required classes, universe, full-run patterns, cell roots, and base ref are DATA here; the Rust kernel hardcodes no repo path nor any oyatie string (R0, ADR-0548). The kernel DOES fix the decision SEMANTICS — RefuseUnowned > Full > Affected > NoGraphTargets dominance, derivation-uncertainty-escalates-to-full, owner() on every existing changed file regardless of extension — that contract is the engine, not a per-repo pack value.",
    42	    "fail_closed": "Derivation NEVER skips: git/uquery/rdeps errors, unmappable package files, deleted graph files, and empty rdeps closures all escalate to the full workspace run. The only hard failures are an unreadable/invalid pack, owner-required files with no owning target, and the build/test verdict itself.",
    43	    "tiers": "pull_request -> auto (affected cone, escalation binding); merge_group + push + workflow_dispatch -> full (admission/integration tier per ADR-0515 Tide direction).",
    44	    "precedent": "Bazel target determination / bazel-diff (Tinder), Meta/Google affected-target CI; reimplemented Rust-native on buck2 uquery owner()/rdeps() per the proven-patterns doctrine.",
    45	    "execution": "Buck2-native Rust lane; no shell logic beyond the workflow YAML step (G011 Rust-successor of the transitional infra/ci/buck2-affected-gate.sh)."
    46	  },
    47	  "purpose": "Binding workspace coverage: every PR builds+tests the buck2 reverse-dependency cone of its merge-base diff as a REQUIRED context, with mechanical fail-closed escalation to the full workspace whenever the cone cannot be trusted. Any owner-required source change with a buck2 target builds+tests its cone; any buildfile/config/macro change escalates to FULL; a genuinely-unowned owner-required file (e.g. a source in a sub-workspace buck2 does not model) REFUSES the merge until it is wired — never silently passes."
    48	}
exec
/bin/zsh -lc 'git show origin/dev:.buckconfig | nl -ba; git show origin/dev:.buckroot 2>/dev/null | nl -ba' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	[cells]
     2	  root = .
     3	  prelude = prelude
     4	  toolchains = toolchains
     5	  none = none
     6	  third-party = third-party
     7	
     8	[cell_aliases]
     9	  config = prelude
    10	  ovr_config = prelude
    11	  fbcode = none
    12	  fbsource = none
    13	  fbcode_macros = none
    14	  buck = none
    15	
    16	# Uses a copy of the prelude bundled with the buck2 binary. You can alternatively delete this
    17	# section and vendor a copy of the prelude to the `prelude` directory of your project.
    18	[external_cells]
    19	  prelude = bundled
    20	
    21	[parser]
    22	  target_platform_detector_spec = target:root//...->prelude//platforms:default \
    23	    target:prelude//...->prelude//platforms:default \
    24	    target:toolchains//...->prelude//platforms:default \
    25	    target:third-party//...->prelude//platforms:default
    26	
    27	[build]
    28	  execution_platforms = prelude//platforms:default
    29	
    30	# Bare-stock buck2 daemons default to a 4-day idle timeout, which lets stale
    31	# daemons (one per worktree/branch checkout) accumulate and sprawl host RAM.
    32	# These three keys are confirmed-recognized buck2 [buck2] section properties
    33	# (verified against facebook/buck2 upstream source: app/buck2_common/src/init.rs
    34	# for daemon_idle_timeout_s, app/buck2_execute_impl/.../clean_stale.rs for
    35	# clean_stale_enabled, docs/users/advanced/restarter.md for restarter).
    36	[buck2]
    37	  daemon_idle_timeout_s = 14400
    38	  clean_stale_enabled = true
    39	  restarter = true
exec
/bin/zsh -lc "git grep -n -E \"decision=FULL|ESCALATE|full_build|full_test|global|escape|owner\\(|rdeps|merge-base|deleted|renamed|uncertain|mode\" origin/dev -- ci/facade/affected-target-set | sed -n '1,260p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
origin/dev:ci/facade/affected-target-set/BUCK:24:# head against the merge-base, blocking only REGRESSIONS (grandfathering pre-existing build debt)
origin/dev:ci/facade/affected-target-set/BUCK:25:# so the FULL tier is a ratchet, not a flag-day requirement. Reuses the ADR-0551 merge-base
origin/dev:ci/facade/affected-target-set/BUCK:26:# frozen-baseline pattern; the merge-base report is materialized out-of-band (never candidate-controlled).
origin/dev:ci/facade/affected-target-set/BUCK:73:# in the decided set) + every fail-closed seam (escape->FULL, deletion->FULL, unowned->REFUSE).
origin/dev:ci/facade/affected-target-set/Cargo.toml:13:purpose = "Binding workspace-coverage lane (ADR-0554): derives the buck2 affected-target set from the merge-base diff and builds+tests it, with fail-closed escalation to full-workspace on any derivation uncertainty."
origin/dev:ci/facade/affected-target-set/Cargo.toml:18:# all repo facts (escape-trigger path classes, universe, cell roots) live in
origin/dev:ci/facade/affected-target-set/Cargo.toml:30:# ADR-0554 round-3: build-health ratchet — blocks FULL-tier build REGRESSIONS vs the merge-base
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:2:  "_comment": "cloud-ci-affected-set binding workspace-coverage policy (DATA, not code). ADR-0554, converts FRIC-1781310000: the only binding buck2 lane was scoped to //cloud/cloud-ci/..., so code anywhere else could merge broken (PR #651 head cf16525 did not compile yet its buck2 lane was green). ALL repo-specifics live here; the Rust kernel hardcodes no oyatie path and runs on any buck2 repo by editing this pack. full_trigger_patterns are the rdeps-cone ESCAPE classes — graph-semantic files whose blast radius the owner()/rdeps() derivation cannot model (build config, toolchains, vendored third-party, Starlark macros, the lockfile): any touch escalates to the FULL workspace, mechanically, with no skip and no human judgment. require_owner_patterns are the classes that MUST map to an owning target; an existing file in these classes with no owner FAILS the lane (graph-invisible code is not made safe by running more targets).",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:9:  "_full_trigger_note": "Two seam classes here. (1) Build config/macros whose blast radius the per-package rdeps cone cannot bound — .buckconfig + .buckconfig.local + .buckconfig.d/** (all read by buck2, all committable), toolchains/**, third-party/** (reindeer vendor + fixups), Starlark **/*.bzl + **/*.bxl, rust-toolchain.toml. (2) Buildfiles and PACKAGE files are handled by package_definition_basenames (escalate to FULL on any change) AND mirrored here as **/PACKAGE so a NEW PACKAGE file (which evaluates to [] and would otherwise look like a plain no-owner file) is never a silent no-op. Cargo.lock is deliberately NOT a trigger: buck2 never reads it — a dependency change that affects buck2 semantics MUST touch third-party/**; the cargo lanes + ADR-0539 freshness gate own lock hygiene.",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:26:  "_package_sibling_note": "Cargo.toml + build.rs are NOT buck2 graph inputs (buck2 never reads them; reindeer/BUCK mirror them) and have no owner() BY DESIGN, so requiring an owner would refuse every manifest edit (proven by this lane's own first dogfood run). They are semantically bound to their crate, so they seed the ENCLOSING package target pattern; if that package does not exist the seed query fails and the lane escalates to FULL.",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:31:  "_package_definition_note": "Ground-truth buck2 buildfile-name set in PRECEDENCE order: BUCK.v2 SHADOWS BUCK when both exist (empirically verified against this buck2 binary — adding an empty BUCK.v2 next to a real BUCK drops the BUCK targets). A change to ANY buildfile escalates to FULL (these basenames are also escape-class): a buildfile edit can add/remove targets or shadow the file dependents load, so its blast radius is NOT bounded by its own package's rdeps. owner() is empty for a buildfile by design, so seeding 'its package' alone would silently miss every dependent (F2). If buck2's [buildfile] name/extra_for_test config ever adds names, mirror them here — this list IS the repo's buildfile-name ground truth, not a single hand-set name.",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:41:    "born_pack_shaped": "The escape-trigger classes, owner-required classes, universe, full-run patterns, cell roots, and base ref are DATA here; the Rust kernel hardcodes no repo path nor any oyatie string (R0, ADR-0548). The kernel DOES fix the decision SEMANTICS — RefuseUnowned > Full > Affected > NoGraphTargets dominance, derivation-uncertainty-escalates-to-full, owner() on every existing changed file regardless of extension — that contract is the engine, not a per-repo pack value.",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:42:    "fail_closed": "Derivation NEVER skips: git/uquery/rdeps errors, unmappable package files, deleted graph files, and empty rdeps closures all escalate to the full workspace run. The only hard failures are an unreadable/invalid pack, owner-required files with no owning target, and the build/test verdict itself.",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:44:    "precedent": "Bazel target determination / bazel-diff (Tinder), Meta/Google affected-target CI; reimplemented Rust-native on buck2 uquery owner()/rdeps() per the proven-patterns doctrine.",
origin/dev:ci/facade/affected-target-set/affected-set-policy.json:47:  "purpose": "Binding workspace coverage: every PR builds+tests the buck2 reverse-dependency cone of its merge-base diff as a REQUIRED context, with mechanical fail-closed escalation to the full workspace whenever the cone cannot be trusted. Any owner-required source change with a buck2 target builds+tests its cone; any buildfile/config/macro change escalates to FULL; a genuinely-unowned owner-required file (e.g. a source in a sub-workspace buck2 does not model) REFUSES the merge until it is wired — never silently passes."
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:1://! cloud-ci build-health ratchet (ADR-0554 round-3; reuses the ADR-0551/#698 merge-base
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:10://!     merge-base, or the target is brand-new) -> BLOCK (exit 1);
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:40:    /// (a real build happened at the merge-base). Without it, a truncated/empty baseline would
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:75:                "{LOG}: usage: oya-cloud-ci-build-health --baseline-report <merge-base.json> \
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:116:    // look pre-existing. CI builds the whole merge-base workspace, so the baseline is never
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:120:            "{LOG}: BASELINE EMPTY — the merge-base build-report has no `results`. Refusing to \
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:122:             Re-run the merge-base `buck2 build //... --keep-going --build-report`."
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:135:            "{LOG}: GREEN — no build regressions vs the merge-base ({} pre-existing failure(s) \
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:142:            "{LOG}: RED — {} build REGRESSION(S) vs the merge-base (target(s) that build at \
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:150:            "{LOG}: REMEDIATION: fix these targets (they compiled at the merge-base), or revert \
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:160:        "{LOG}: build-health ratchet vs merge-base — baseline targets={baseline_total}, head \
origin/dev:ci/facade/affected-target-set/src/bin/oya-cloud-ci-build-health.rs:176:            "{LOG}:   fixed (burned down vs merge-base) = {}",
origin/dev:ci/facade/affected-target-set/src/lib.rs:3://! Binding workspace-coverage derivation: the pure decision kernel that turns a merge-base
origin/dev:ci/facade/affected-target-set/src/lib.rs:14://! the cone cannot model (macros, toolchains, vendored third-party).
origin/dev:ci/facade/affected-target-set/src/lib.rs:17://! The kernel hardcodes NO repo facts: escape-trigger path classes, owner-required path
origin/dev:ci/facade/affected-target-set/src/lib.rs:21://! ## Decision contract (fully mechanical — zero manual escape hatches)
origin/dev:ci/facade/affected-target-set/src/lib.rs:24://!   `owner()` results into the final verdict.
origin/dev:ci/facade/affected-target-set/src/lib.rs:29://!   - `Full`: an escape-trigger path class matched, a graph-relevant file was deleted, or the
origin/dev:ci/facade/affected-target-set/src/lib.rs:30://!     adapter reported derivation uncertainty — the rdeps cone cannot be trusted, so the
origin/dev:ci/facade/affected-target-set/src/lib.rs:34://!     (docs/config-text outside the buildfile + escape-trigger classes).
origin/dev:ci/facade/affected-target-set/src/lib.rs:56:    /// Path classes that escape the rdeps cone -> FULL run (micro-glob patterns).
origin/dev:ci/facade/affected-target-set/src/lib.rs:62:    /// "BUCK"]` — `BUCK.v2` SHADOWS `BUCK` when both exist). `owner()` is empty for these BY
origin/dev:ci/facade/affected-target-set/src/lib.rs:65:    /// package definition can break arbitrary dependents the rdeps cone of the package alone
origin/dev:ci/facade/affected-target-set/src/lib.rs:75:    /// (e.g. `{"": "//"}`). A package file under no mapped cell is derivation uncertainty.
origin/dev:ci/facade/affected-target-set/src/lib.rs:77:    /// Default base ref for the merge-base anchor (e.g. `origin/dev`); CLI `--base` overrides.
origin/dev:ci/facade/affected-target-set/src/lib.rs:248:    /// File no longer exists at HEAD (deleted, rename source).
origin/dev:ci/facade/affected-target-set/src/lib.rs:264:    /// Matched an escape-trigger pattern -> FULL.
origin/dev:ci/facade/affected-target-set/src/lib.rs:268:    /// Buildfile (BUCK/BUCK.v2/PACKAGE) changed or deleted -> FULL (blast radius exceeds its
origin/dev:ci/facade/affected-target-set/src/lib.rs:273:    /// Sent to `owner()` resolution.
origin/dev:ci/facade/affected-target-set/src/lib.rs:280:/// `owner()` results, then [`resolve`] (phase B) folds them into the verdict.
origin/dev:ci/facade/affected-target-set/src/lib.rs:283:    /// Reasons forcing a FULL run (escape triggers, graph deletions, uncertainty).
origin/dev:ci/facade/affected-target-set/src/lib.rs:293:/// Classify every change (PURE). Order per path: escape-trigger -> package-definition ->
origin/dev:ci/facade/affected-target-set/src/lib.rs:294:/// deletion handling -> owner query. EVERY existing file goes to `owner()` regardless of
origin/dev:ci/facade/affected-target-set/src/lib.rs:307:                .push(format!("`{path}` matches escape-trigger `{pat}`"));
origin/dev:ci/facade/affected-target-set/src/lib.rs:314:        // bounded by its own package's rdeps: a new BUCK.v2 SHADOWS the BUCK that dependents
origin/dev:ci/facade/affected-target-set/src/lib.rs:316:        // PACKAGE file mutates parse-time values for the whole subtree. owner() is empty for a
origin/dev:ci/facade/affected-target-set/src/lib.rs:325:                Change::Deleted(_) => "deleted",
origin/dev:ci/facade/affected-target-set/src/lib.rs:343:                        .push(format!("package sibling `{path}` was deleted"));
origin/dev:ci/facade/affected-target-set/src/lib.rs:355:                            "package sibling `{path}` maps to no configured cell root (derivation uncertainty)"
origin/dev:ci/facade/affected-target-set/src/lib.rs:372:                        .push(format!("graph-relevant file `{path}` was deleted"));
origin/dev:ci/facade/affected-target-set/src/lib.rs:427:/// Fold per-file `owner()` results into the verdict (PURE).
origin/dev:ci/facade/affected-target-set/src/lib.rs:511:    mode: &str,
origin/dev:ci/facade/affected-target-set/src/lib.rs:547:        "mode": mode,
origin/dev:ci/facade/affected-target-set/src/lib.rs:554:            "required": matches!(decision, Decision::Full { .. }) && mode == "auto",
origin/dev:ci/facade/affected-target-set/src/lib.rs:556:            "anti_laundering": "baseline report must be produced from the merge-base committed tree, never the candidate tree"
origin/dev:ci/facade/affected-target-set/src/lib.rs:574:// ── BUILD-HEALTH RATCHET (ADR-0554 round-3; reuses the ADR-0551/#698 merge-base frozen-baseline
origin/dev:ci/facade/affected-target-set/src/lib.rs:577://    founder merge-base-ratchet doctrine (block NEW debt, grandfather pre-existing; FRIC-1781112000
origin/dev:ci/facade/affected-target-set/src/lib.rs:579://    merge-base: a head failure that was ALSO failing at the merge-base is GRANDFATHERED (shrink-only
origin/dev:ci/facade/affected-target-set/src/lib.rs:580://    burn-down); a head failure that built at the merge-base — or a brand-new target that fails — is a
origin/dev:ci/facade/affected-target-set/src/lib.rs:581://    REGRESSION and BLOCKS. Soundness (the #698 F1 lesson): the baseline is the merge-base build
origin/dev:ci/facade/affected-target-set/src/lib.rs:582://    result, materialized out-of-band from the merge-base checkout, NEVER the candidate tree, so a PR
origin/dev:ci/facade/affected-target-set/src/lib.rs:631:/// and SHA shape so stale/wrong artifacts cannot be confused with an exact merge-base baseline.
origin/dev:ci/facade/affected-target-set/src/lib.rs:636:            "merge-base SHA must be a 40-character hex object id, got `{merge_base_sha}`"
origin/dev:ci/facade/affected-target-set/src/lib.rs:642:/// Select the trusted push-to-dev workflow run whose head SHA is the exact merge-base.
origin/dev:ci/facade/affected-target-set/src/lib.rs:729:    /// Targets that FAIL at head but did NOT fail at the merge-base (built there, or are brand
origin/dev:ci/facade/affected-target-set/src/lib.rs:732:    /// Targets that fail at head AND failed at the merge-base: GRANDFATHERED (shrink-only).
origin/dev:ci/facade/affected-target-set/src/lib.rs:734:    /// Targets that failed at the merge-base but now BUILD at head: burned-down (informational).
origin/dev:ci/facade/affected-target-set/src/lib.rs:751:/// merge-base, or the target is brand-new). A head failure that IS in the baseline is
origin/dev:ci/facade/affected-target-set/src/lib.rs:754:/// baseline is supplied from the merge-base build, not from any candidate-controlled input.
origin/dev:ci/facade/affected-target-set/src/lib.rs:1001:        // baseline (merge-base) red: {blake3, sqlx}. head red: {blake3, sqlx, NEW}.
origin/dev:ci/facade/affected-target-set/src/lib.rs:1018:        // present at the merge-base -> all grandfathered -> GREEN (no flag-day requirement).
origin/dev:ci/facade/affected-target-set/src/lib.rs:1039:        // that can add it to `baseline` (the baseline comes from the merge-base build), so a PR
origin/dev:ci/facade/affected-target-set/src/lib.rs:1041:        let baseline = set(&["root//tp:blake3"]); // only blake3 was red at merge-base
origin/dev:ci/facade/affected-target-set/src/lib.rs:1122:                "rdeps-closure",
origin/dev:ci/facade/affected-target-set/src/lib.rs:1124:                "rdeps returned an empty closure for non-empty seeds",
origin/dev:ci/facade/affected-target-set/src/lib.rs:1129:                "FULL escalation executed after rdeps failure",
origin/dev:ci/facade/affected-target-set/src/lib.rs:1159:            "rdeps-closure"
origin/dev:ci/facade/affected-target-set/src/lib.rs:1183:            GatePhaseOutcome::new("rdeps-closure", "completed", "2 affected targets"),
origin/dev:ci/facade/affected-target-set/src/main.rs:3://! Orchestrates: merge-base diff (git) -> pure kernel classification -> per-file `owner()`
origin/dev:ci/facade/affected-target-set/src/main.rs:4://! + `rdeps()` closure (buck2 uquery) -> `buck2 build` + `buck2 test` of the decided set.
origin/dev:ci/facade/affected-target-set/src/main.rs:6://! FAIL-CLOSED SEAMS (the escalation IS the automation — zero manual escape hatches):
origin/dev:ci/facade/affected-target-set/src/main.rs:7://! - any git/uquery/rdeps derivation failure escalates to the FULL workspace run, never skips;
origin/dev:ci/facade/affected-target-set/src/main.rs:10://! - `--mode full` (merge-queue admission / post-merge on the integration branch) bypasses
origin/dev:ci/facade/affected-target-set/src/main.rs:38:    mode: Mode,
origin/dev:ci/facade/affected-target-set/src/main.rs:40:    /// Optional path to the merge-base build-health baseline report (ADR-0554 round-3). When set,
origin/dev:ci/facade/affected-target-set/src/main.rs:44:    /// The baseline MUST be produced from the merge-base checkout out-of-band (never the
origin/dev:ci/facade/affected-target-set/src/main.rs:54:    mode: &'static str,
origin/dev:ci/facade/affected-target-set/src/main.rs:71:    let mut mode = Mode::Auto;
origin/dev:ci/facade/affected-target-set/src/main.rs:80:            "--mode" => {
origin/dev:ci/facade/affected-target-set/src/main.rs:81:                mode = match argv.next().as_deref() {
origin/dev:ci/facade/affected-target-set/src/main.rs:84:                    other => return Err(format!("--mode must be auto|full, got {other:?}")),
origin/dev:ci/facade/affected-target-set/src/main.rs:102:        mode,
origin/dev:ci/facade/affected-target-set/src/main.rs:115:                "{LOG}: usage: oya-cloud-ci-affected-set --policy <pack.json> [--base <ref>] [--head <ref>] [--mode auto|full] [--derive-only] [--baseline-report <merge-base-build-report.json>] [--decision-artifact-out <path>]"
origin/dev:ci/facade/affected-target-set/src/main.rs:152:    let decision = match args.mode {
origin/dev:ci/facade/affected-target-set/src/main.rs:154:            reasons: vec!["--mode full (admission/integration tier)".to_owned()],
origin/dev:ci/facade/affected-target-set/src/main.rs:201:                 owner-required class (docs/config-text outside the buildfile/escape classes) -> PASS"
origin/dev:ci/facade/affected-target-set/src/main.rs:209:            println!("{LOG}: decision=FULL — running the complete workspace, because:");
origin/dev:ci/facade/affected-target-set/src/main.rs:217:                        "materialize-merge-base-build-health-baseline",
origin/dev:ci/facade/affected-target-set/src/main.rs:220:                        } else if args.mode == Mode::Auto {
origin/dev:ci/facade/affected-target-set/src/main.rs:244:                    if args.mode == Mode::Full {
origin/dev:ci/facade/affected-target-set/src/main.rs:245:                        "bypassed-mode-full"
origin/dev:ci/facade/affected-target-set/src/main.rs:252:                    "materialize-merge-base-build-health-baseline",
origin/dev:ci/facade/affected-target-set/src/main.rs:255:                    } else if args.mode == Mode::Auto {
origin/dev:ci/facade/affected-target-set/src/main.rs:280:            match rdeps_closure(&buck2, &policy, &seeds) {
origin/dev:ci/facade/affected-target-set/src/main.rs:296:                                "rdeps-closure",
origin/dev:ci/facade/affected-target-set/src/main.rs:318:                                    "rdeps-closure",
origin/dev:ci/facade/affected-target-set/src/main.rs:344:                            println!("{LOG}: ESCALATE to FULL — argfile write failed: {e}");
origin/dev:ci/facade/affected-target-set/src/main.rs:354:                                    "rdeps-closure",
origin/dev:ci/facade/affected-target-set/src/main.rs:377:                    println!("{LOG}: ESCALATE to FULL — {reason}");
origin/dev:ci/facade/affected-target-set/src/main.rs:380:                            "rdeps closure failed after AFFECTED decision: {reason}"
origin/dev:ci/facade/affected-target-set/src/main.rs:386:                            phase("rdeps-closure", "failed-escalated", reason.clone()),
origin/dev:ci/facade/affected-target-set/src/main.rs:405:                        phase("rdeps-closure", "failed-escalated", reason),
origin/dev:ci/facade/affected-target-set/src/main.rs:409:                            "FULL escalation executed after rdeps failure",
origin/dev:ci/facade/affected-target-set/src/main.rs:424:fn mode_name(mode: Mode) -> &'static str {
origin/dev:ci/facade/affected-target-set/src/main.rs:425:    match mode {
origin/dev:ci/facade/affected-target-set/src/main.rs:437:        mode: mode_name(args.mode),
origin/dev:ci/facade/affected-target-set/src/main.rs:464:        context.mode,
origin/dev:ci/facade/affected-target-set/src/main.rs:491:/// Auto-mode derivation. Any uncertainty returns `Decision::Full` with the reason (fail-closed
origin/dev:ci/facade/affected-target-set/src/main.rs:494:    let merge_base = match capture("git", &["merge-base", &args.head, base]) {
origin/dev:ci/facade/affected-target-set/src/main.rs:499:                    "derivation uncertainty: git merge-base {} {base} failed: {e}",
origin/dev:ci/facade/affected-target-set/src/main.rs:506:        "{LOG}: base={base} head={} merge-base={merge_base}",
origin/dev:ci/facade/affected-target-set/src/main.rs:516:                reasons: vec![format!("derivation uncertainty: git diff failed: {e}")],
origin/dev:ci/facade/affected-target-set/src/main.rs:525:                    "derivation uncertainty: unparseable git diff entry: {e}"
origin/dev:ci/facade/affected-target-set/src/main.rs:531:        println!("{LOG}: no changed files vs merge-base — nothing to derive");
origin/dev:ci/facade/affected-target-set/src/main.rs:541:                    "derivation uncertainty: buck2 owner() query failed: {e}"
origin/dev:ci/facade/affected-target-set/src/main.rs:589:            // produce — surface as uncertainty rather than guessing.
origin/dev:ci/facade/affected-target-set/src/main.rs:596:/// Batched per-file owner resolution: `buck2 uquery --json "owner(%s)" @argfile` returns a
origin/dev:ci/facade/affected-target-set/src/main.rs:597:/// JSON object keyed by each path. A query ERROR is uncertainty (caller escalates) — it is
origin/dev:ci/facade/affected-target-set/src/main.rs:609:            "owner(%s)",
origin/dev:ci/facade/affected-target-set/src/main.rs:614:        serde_json::from_str(&out).map_err(|e| format!("owner() output is not JSON: {e}"))?;
origin/dev:ci/facade/affected-target-set/src/main.rs:615:    let obj = v.as_object().ok_or("owner() JSON is not an object")?;
origin/dev:ci/facade/affected-target-set/src/main.rs:618:        let list = owners.as_array().ok_or("owner() entry is not an array")?;
origin/dev:ci/facade/affected-target-set/src/main.rs:623:                    .ok_or("owner() target is not a string")?
origin/dev:ci/facade/affected-target-set/src/main.rs:634:fn rdeps_closure(buck2: &str, policy: &Policy, seeds: &[String]) -> Result<Vec<String>, String> {
origin/dev:ci/facade/affected-target-set/src/main.rs:636:    let query = format!("rdeps({}, %Ss)", policy.universe);
origin/dev:ci/facade/affected-target-set/src/main.rs:649:            "rdeps returned an empty closure for non-empty seeds (query problem)".to_owned(),
origin/dev:ci/facade/affected-target-set/src/main.rs:663:                println!("{LOG}:   FULL-TRIGGER {path} (graph file deleted/unmappable)")
origin/dev:ci/facade/affected-target-set/src/main.rs:676:                println!("{LOG}:   NO-GRAPH     {path} (deleted, outside graph classes)")
origin/dev:ci/facade/affected-target-set/src/main.rs:682:/// The FULL-tier runner (ADR-0554 round-3; D7 round-4 producer). Two modes:
origin/dev:ci/facade/affected-target-set/src/main.rs:684:/// - WITHOUT a baseline report (`--mode full` at admission, or any caller that does not pass
origin/dev:ci/facade/affected-target-set/src/main.rs:691:///   artifact (the merge-base-to-be baseline for the DEFERRED D8 cross-run consumer + ADR-0560
origin/dev:ci/facade/affected-target-set/src/main.rs:696:///   FAILURE set against the merge-base baseline failure set: only REGRESSIONS (targets that build
origin/dev:ci/facade/affected-target-set/src/main.rs:697:///   at the merge-base but fail at head, or brand-new failing targets) block; pre-existing build
origin/dev:ci/facade/affected-target-set/src/main.rs:726:        "{LOG}: FULL tier (build-health ratchet vs merge-base baseline {baseline_path}): \
origin/dev:ci/facade/affected-target-set/src/main.rs:753:                "{LOG}: FAIL — cannot read merge-base baseline report `{baseline_path}`: {e}"
origin/dev:ci/facade/affected-target-set/src/main.rs:768:            eprintln!("{LOG}: FAIL — merge-base baseline report parse error: {e}");
origin/dev:ci/facade/affected-target-set/src/main.rs:779:    // Fail-closed laundering guard: an empty merge-base baseline would grandfather every head
origin/dev:ci/facade/affected-target-set/src/main.rs:780:    // failure. CI builds the whole merge-base workspace, so the baseline is never legitimately
origin/dev:ci/facade/affected-target-set/src/main.rs:784:            "{LOG}: FAIL — merge-base baseline build-report has no `results`. Refusing to \
origin/dev:ci/facade/affected-target-set/src/main.rs:807:            "{LOG}: RED — {} build REGRESSION(S) vs the merge-base (built at origin/dev, FAIL at \
origin/dev:ci/facade/affected-target-set/src/main.rs:827:    // ACTUAL changed code is the cone path's job (auto mode, hard-fail, unchanged — the cf16525
origin/dev:ci/facade/affected-target-set/src/main.rs:832:        "{LOG}: PASS — no build regressions vs the merge-base ({} pre-existing build failure(s) \
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:6:// the old advisory lane could silently under-test must either ESCALATE TO FULL or REFUSE.
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:95:    // owner() ran and found nothing — and .md is not owner-required (F3: the claim is
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:105:    // existing file to owner(), so an owned asset becomes a seed.
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:122:// ── Escape classes: the rdeps cone cannot model these -> FULL, mechanically ──────────────
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:158:    // buck2 and committable. Each is added by an empty/valid file -> owner() empty -> would be
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:170:        // owner() is empty for buildfile/config files (BY DESIGN) — pass an empty owner map to
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:200:fn deleted_source_file_escalates_to_full() {
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:201:    // owner() cannot resolve a path that no longer exists at HEAD, but deleting a source can
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:213:fn deleted_package_definition_escalates_to_full() {
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:224:fn deleted_doc_is_not_an_escalation() {
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:234:// ── Buildfile changes escalate to FULL (blast radius exceeds the package's own rdeps) ────
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:277:    // Manifests are not buck2 graph inputs (no owner() BY DESIGN — proven by this lane's own
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:297:fn deleted_cargo_manifest_escalates_to_full() {
origin/dev:ci/facade/affected-target-set/tests/affected_set.rs:516:    // F2: PACKAGE + .buckconfig.local are escape triggers in the shipped pack.
exec
/bin/zsh -lc "git grep -n -F '.claire/worktrees' origin/dev -- .buckconfig .buckroot BUCK '**/BUCK' ':"'!vendor'"' 2>/dev/null | sed -n '1,160p'; git ls-tree -r --name-only origin/dev .claire | sed -n '1,120p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/tests/affected_set.rs | nl -ba | sed -n '180,330p'" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/lib.rs | nl -ba | sed -n '240,470p'" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/tests/affected_set.rs | nl -ba | sed -n '1,180p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   180	            }
   181	        }
   182	    }
   183	}
   184	
   185	#[test]
   186	fn red_f2_buck_v2_precedence_first_in_basenames() {
   187	    // The basename list is PRECEDENCE-ordered: BUCK.v2 before BUCK (BUCK.v2 shadows BUCK in
   188	    // buck2). The engine treats either as a buildfile -> FULL; this pins the ground-truth order
   189	    // in the shipped pack so a future editor cannot silently drop BUCK.v2.
   190	    let p = policy();
   191	    assert_eq!(
   192	        p.package_definition_basenames.first().map(String::as_str),
   193	        Some("BUCK.v2"),
   194	        "BUCK.v2 must be first (it shadows BUCK)"
   195	    );
   196	    assert!(p.package_definition_basenames.iter().any(|b| b == "BUCK"));
   197	}
   198	
   199	#[test]
   200	fn deleted_source_file_escalates_to_full() {
   201	    // owner() cannot resolve a path that no longer exists at HEAD, but deleting a source can
   202	    // break every dependent of its former target — mechanical escalation, never skip.
   203	    let p = policy();
   204	    let changes = [Change::Deleted("libs/oya-thing/src/gone.rs".into())];
   205	    let plan = plan_changes(&changes, &p);
   206	    assert!(matches!(
   207	        resolve(&plan, &BTreeMap::new(), &p),
   208	        Decision::Full { .. }
   209	    ));
   210	}
   211	
   212	#[test]
   213	fn deleted_package_definition_escalates_to_full() {
   214	    let p = policy();
   215	    let changes = [Change::Deleted("libs/oya-thing/BUCK".into())];
   216	    let plan = plan_changes(&changes, &p);
   217	    assert!(matches!(
   218	        resolve(&plan, &BTreeMap::new(), &p),
   219	        Decision::Full { .. }
   220	    ));
   221	}
   222	
   223	#[test]
   224	fn deleted_doc_is_not_an_escalation() {
   225	    let p = policy();
   226	    let changes = [Change::Deleted("docs/old-note.md".into())];
   227	    let plan = plan_changes(&changes, &p);
   228	    assert_eq!(
   229	        resolve(&plan, &BTreeMap::new(), &p),
   230	        Decision::NoGraphTargets
   231	    );
   232	}
   233	
   234	// ── Buildfile changes escalate to FULL (blast radius exceeds the package's own rdeps) ────
   235	
   236	#[test]
   237	fn modified_buck_file_escalates_to_full() {
   238	    // A BUCK edit can add/remove targets or rewire deps that arbitrary OTHER packages resolve;
   239	    // seeding only "its own package" (the previous, wrong behavior) missed those dependents.
   240	    let p = policy();
   241	    let changes = [Change::Present(
   242	        "cloud/cloud-iam/crates/oya-iam/BUCK".into(),
   243	    )];
   244	    let plan = plan_changes(&changes, &p);
   245	    match resolve(&plan, &BTreeMap::new(), &p) {
   246	        Decision::Full { reasons } => assert!(
   247	            reasons.iter().any(|r| r.contains("oya-iam/BUCK")),
   248	            "FULL reason must name the buildfile; got {reasons:?}"
   249	        ),
   250	        other => panic!("a BUCK change must escalate to FULL, got {other:?}"),
   251	    }
   252	}
   253	
   254	// ── Graph-invisible code REFUSES (running more targets cannot make it safe) ──────────────
   255	
   256	#[test]
   257	fn unowned_source_file_refuses_instead_of_passing_or_fulling() {
   258	    let p = policy();
   259	    let changes = [
   260	        Change::Present("oya/new-svc/src/lib.rs".into()),
   261	        // Even with a co-present full trigger, refusal must dominate: a full run would not
   262	        // compile the unowned file, so FULL would still be a false-green for it.
   263	        Change::Present("third-party/reindeer.toml".into()),
   264	    ];
   265	    let plan = plan_changes(&changes, &p);
   266	    let decision = resolve(&plan, &owners(&[("oya/new-svc/src/lib.rs", &[])]), &p);
   267	    assert_eq!(
   268	        decision,
   269	        Decision::RefuseUnowned {
   270	            paths: vec!["oya/new-svc/src/lib.rs".into()]
   271	        }
   272	    );
   273	}
   274	
   275	#[test]
   276	fn cargo_manifest_seeds_its_enclosing_package() {
   277	    // Manifests are not buck2 graph inputs (no owner() BY DESIGN — proven by this lane's own
   278	    // first dogfood run, which refused on its own crate's Cargo.toml under an owner-required
   279	    // pack). They are semantically bound to their crate: seed the enclosing package pattern,
   280	    // exactly like a BUCK edit. A manifest in a package-less dir makes the seed query fail
   281	    // downstream -> the adapter escalates to FULL.
   282	    let p = policy();
   283	    let changes = [Change::Present(
   284	        "oya/svc/crates/oya-svc-app/Cargo.toml".into(),
   285	    )];
   286	    let plan = plan_changes(&changes, &p);
   287	    let decision = resolve(&plan, &BTreeMap::new(), &p);
   288	    assert_eq!(
   289	        decision,
   290	        Decision::Affected {
   291	            seeds: vec!["//oya/svc/crates/oya-svc-app:".into()]
   292	        }
   293	    );
   294	}
   295	
   296	#[test]
   297	fn deleted_cargo_manifest_escalates_to_full() {
   298	    let p = policy();
   299	    let changes = [Change::Deleted(
   300	        "oya/svc/crates/oya-svc-app/Cargo.toml".into(),
   301	    )];
   302	    let plan = plan_changes(&changes, &p);
   303	    assert!(matches!(
   304	        resolve(&plan, &BTreeMap::new(), &p),
   305	        Decision::Full { .. }
   306	    ));
   307	}
   308	
   309	#[test]
   310	fn build_script_is_a_package_sibling_not_a_refusal() {
   311	    // build.rs is cargo-side crate metadata (live-repo audit 2026-06-12: the only first-party
   312	    // build.rs has no buck2 owner) — owner-requiring it would be a refusal landmine; it seeds
   313	    // the enclosing package like Cargo.toml.
   314	    let p = policy();
   315	    let changes = [Change::Present(
   316	        "oya/svc/crates/oya-svc-app/build.rs".into(),
   317	    )];
   318	    let plan = plan_changes(&changes, &p);
   319	    assert_eq!(
   320	        resolve(&plan, &BTreeMap::new(), &p),
   321	        Decision::Affected {
   322	            seeds: vec!["//oya/svc/crates/oya-svc-app:".into()]
   323	        }
   324	    );
   325	}
   326	
   327	#[test]
   328	fn red_f1_owned_kernel_source_lands_in_its_cone_no_exemption() {
   329	    // F1 (reviewer-reproduced, the bad one): the prior pack out-of-graph-exempted
   330	    // cloud/cloud-kernel/** — FACTUALLY FALSE (the cited
 succeeded in 0ms:
   240	    pi == p.len()
   241	}
   242	
   243	/// One diff entry, as parsed by the adapter from `git diff --name-status`.
   244	#[derive(Debug, Clone, PartialEq, Eq)]
   245	pub enum Change {
   246	    /// File exists at HEAD (added, modified, type-changed, copy/rename destination).
   247	    Present(String),
   248	    /// File no longer exists at HEAD (deleted, rename source).
   249	    Deleted(String),
   250	}
   251	
   252	impl Change {
   253	    pub fn path(&self) -> &str {
   254	        match self {
   255	            Change::Present(p) | Change::Deleted(p) => p,
   256	        }
   257	    }
   258	}
   259	
   260	/// Why a path was classified the way it was — carried verbatim into the transparency output
   261	/// (the founder automation directive: FAIL output must say exactly what ran and why).
   262	#[derive(Debug, Clone, PartialEq, Eq)]
   263	pub enum PathClass {
   264	    /// Matched an escape-trigger pattern -> FULL.
   265	    FullTrigger(String),
   266	    /// Deleted file in a graph-relevant class -> FULL (its cone is uncomputable at HEAD).
   267	    DeletedGraphFile,
   268	    /// Buildfile (BUCK/BUCK.v2/PACKAGE) changed or deleted -> FULL (blast radius exceeds its
   269	    /// own package: it can add/remove targets or shadow the file dependents load).
   270	    Buildfile,
   271	    /// Package-definition file -> expands to this package target pattern.
   272	    PackagePattern(String),
   273	    /// Sent to `owner()` resolution.
   274	    OwnerQuery,
   275	    /// Deleted file outside every graph-relevant class -> no targets.
   276	    DeletedIrrelevant,
   277	}
   278	
   279	/// The pure classification of a diff (phase A). The adapter answers `owner_paths` with buck2
   280	/// `owner()` results, then [`resolve`] (phase B) folds them into the verdict.
   281	#[derive(Debug, Clone, PartialEq, Eq, Default)]
   282	pub struct Plan {
   283	    /// Reasons forcing a FULL run (escape triggers, graph deletions, uncertainty).
   284	    pub full_reasons: Vec<String>,
   285	    /// Package target patterns from package-definition files (seeds).
   286	    pub package_patterns: Vec<String>,
   287	    /// Existing files whose owning targets must be queried.
   288	    pub owner_paths: Vec<String>,
   289	    /// Per-path classification, for the transparency block.
   290	    pub classified: Vec<(String, PathClass)>,
   291	}
   292	
   293	/// Classify every change (PURE). Order per path: escape-trigger -> package-definition ->
   294	/// deletion handling -> owner query. EVERY existing file goes to `owner()` regardless of
   295	/// extension: a non-source file can be a declared src of a target (`include_str!` assets),
   296	/// so extension pre-filtering would be a false-negative hole.
   297	pub fn plan_changes(changes: &[Change], policy: &Policy) -> Plan {
   298	    let mut plan = Plan::default();
   299	    for change in changes {
   300	        let path = change.path();
   301	        if let Some(pat) = policy
   302	            .full_trigger_patterns
   303	            .iter()
   304	            .find(|pat| glob_match(pat, path))
   305	        {
   306	            plan.full_reasons
   307	                .push(format!("`{path}` matches escape-trigger `{pat}`"));
   308	            plan.classified
   309	                .push((path.to_owned(), PathClass::FullTrigger(pat.clone())));
   310	            continue;
   311	        }
   312	        let basename = path.rsplit('/').next().unwrap_or(path);
   313	        // Buildfile change (BUCK.v2/BUCK/PACKAGE) -> FULL, ALWAYS. Its blast radius is NOT
   314	        // bounded by its own package's rdeps: a new BUCK.v2 SHADOWS the BUCK that dependents
   315	        // load (F2), a new/edited buildfile can add/remove targets dependents resolve, and a
   316	        // PACKAGE file mutates parse-time values for the whole subtree. owner() is empty for a
   317	        // buildfile by design, so seeding "its package" would silently miss every dependent —
   318	        // the exact F2 false-negative. Escalate to the full workspace.
   319	        if policy
   320	            .package_definition_basenames
   321	            .iter()
   322	            .any(|b| b == basename)
   323	        {
   324	            let verb = match change {
   325	                Change::Deleted(_) => "deleted",
   326	                Change::Present(_) => "changed",
   327	            };
   328	            plan.full_reasons.push(format!(
   329	                "buildfile `{path}` {verb} (blast radius exceeds its own package)"
   330	            ));
   331	            plan.classified
   332	                .push((path.to_owned(), PathClass::Buildfile));
   333	            continue;
   334	        }
   335	        if policy
   336	            .package_sibling_basenames
   337	            .iter()
   338	            .any(|b| b == basename)
   339	        {
   340	            match change {
   341	                Change::Deleted(_) => {
   342	                    plan.full_reasons
   343	                        .push(format!("package sibling `{path}` was deleted"));
   344	                    plan.classified
   345	                        .push((path.to_owned(), PathClass::DeletedGraphFile));
   346	                }
   347	                Change::Present(_) => match package_pattern(path, policy) {
   348	                    Some(pat) => {
   349	                        plan.package_patterns.push(pat.clone());
   350	                        plan.classified
   351	                            .push((path.to_owned(), PathClass::PackagePattern(pat)));
   352	                    }
   353	                    None => {
   354	                        plan.full_reasons.push(format!(
   355	                            "package sibling `{path}` maps to no configured cell root (derivation uncertainty)"
   356	                        ));
   357	                        plan.classified
   358	                            .push((path.to_owned(), PathClass::DeletedGraphFile));
   359	                    }
   360	                },
   361	            }
   362	            continue;
   363	        }
   364	        match change {
   365	            Change::Deleted(_) => {
   366	                if policy
   367	                    .require_owner_patterns
   368	                    .iter()
   369	                    .any(|pat| glob_match(pat, path))
   370	                {
   371	                    plan.full_reasons
   372	                        .push(format!("graph-relevant file `{path}` was deleted"));
   373	                    plan.classified
   374	                        .push((path.to_owned(), PathClass::DeletedGraphFile));
   375	                } else {
   376	                    plan.classified
   377	                        .push((path.to_owned(), PathClass::DeletedIrrelevant));
   378	                }
   379	            }
   380	            Change::Present(_) => {
   381	                plan.owner_paths.push(path.to_owned());
   382	                plan.classified
   383	                    .push((path.to_owned(), PathClass::OwnerQuery));
   384	            }
   385	        }
   386	    }
   387	    plan.package_patterns.sort();
   388	    plan.package_patterns.dedup();
   389	    plan
   390	}
   391	
   392	/// Map a package-sibling manifest to its enclosing package target pattern via the cell roots
   393	/// (longest prefix wins). `cloud/x/Cargo.toml` + `{"": "//"}` -> `//cloud/x:`.
   394	fn package_pattern(path: &str, policy: &Policy) -> Option<String> {
   395	    let dir = match path.rsplit_once('/') {
   396	        Some((d, _)) => d,
   397	        None => "",
   398	    };
   399	    let mut best: Option<(&str, &str)> = None;
   400	    for (prefix, cell) in &policy.cell_roots {
   401	        let applies = prefix.is_empty() || dir == prefix || dir.starts_with(&format!("{prefix}/"));
   402	        if applies && best.is_none_or(|(bp, _)| prefix.len() > bp.len()) {
   403	            best = Some((prefix, cell));
   404	        }
   405	    }
   406	    best.map(|(prefix, cell)| {
   407	        let rel = dir.strip_prefix(prefix).unwrap_or(dir);
   408	        let rel = rel.strip_prefix('/').unwrap_or(rel);
   409	        format!("{cell}{rel}:")
   410	    })
   411	}
   412	
   413	/// The final verdict. Dominance: `RefuseUnowned` > `Full` > `Affected` > `NoGraphTargets`.
   414	#[derive(Debug, Clone, PartialEq, Eq)]
   415	pub enum Decision {
   416	    /// Owner-required files with NO owning target: graph-invisible code. Even a full run
   417	    /// would not compile these — running anything would be false-green, so the lane fails.
   418	    RefuseUnowned { paths: Vec<String> },
   419	    /// Run the policy's full-run target patterns.
   420	    Full { reasons: Vec<String> },
   421	    /// Run these seed targets + their reverse-dependency closure.
   422	    Affected { seeds: Vec<String> },
   423	    /// Every change is unowned AND not in any owner-required class.
   424	    NoGraphTargets,
   425	}
   426	
   427	/// Fold per-file `owner()` results into the verdict (PURE).
   428	pub fn resolve(
   429	    plan: &Plan,
   430	    owner_results: &BTreeMap<String, Vec<String>>,
   431	    policy: &Policy,
   432	) -> Decision {
   433	    let mut refusals: Vec<String> = Vec::new();
   434	    let mut seeds: BTreeSet<String> = plan.package_patterns.iter().cloned().collect();
   435	    for path in &plan.owner_paths {
   436	        let owners = owner_results.get(path).map(Vec::as_slice).unwrap_or(&[]);
   437	        if owners.is_empty() {
   438	            if policy
   439	                .require_owner_patterns
   440	                .iter()
   441	                .any(|pat| glob_match(pat, path))
   442	            {
   443	                refusals.push(path.clone());
   444	            }
   445	            // else: provably outside the graph (docs class) — fine.
   446	        } else {
   447	            seeds.extend(owners.iter().cloned());
   448	        }
   449	    }
   450	    if !refusals.is_empty() {
   451	        refusals.sort();
   452	        return Decision::RefuseUnowned { paths: refusals };
   453	    }
   454	    if !plan.full_reasons.is_empty() {
   455	        return Decision::Full {
   456	            reasons: plan.full_reasons.clone(),
   457	        };
   458	    }
   459	    if !seeds.is_empty() {
   460	        return Decision::Affected {
   461	            seeds: seeds.into_iter().collect(),
   462	        };
   463	    }
   464	    Decision::NoGraphTargets
   465	}
   466	
   467	/// One actual phase outcome recorded by the affected-set composition root.
   468	#[derive(Debug, Clone, PartialEq, Eq)]
   469	pub struct GatePhaseOutcome {
   470	    /// Stable phase id.
 succeeded in 0ms:
     1	// cloud-ci-affected-set decision fixtures (ADR-0554, FRIC-1781310000).
     2	//
     3	// Pins the pure kernel contract that closes the cf16525 false-green class: a change to code
     4	// OUTSIDE any fixed CI scope (the PR #651 shape — oya/identity not compiling while the binding
     5	// lane only ran //cloud/cloud-ci/...) must land in the decided target set, and every seam where
     6	// the old advisory lane could silently under-test must either ESCALATE TO FULL or REFUSE.
     7	//
     8	// The live integration proof is the lane itself: it runs as a REQUIRED context on the PR that
     9	// ships it, with this crate (and the in-PR oya/ci-webhook-gateway fix) inside its own cone.
    10	//
    11	// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
    12	#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    13	
    14	use std::collections::BTreeMap;
    15	use std::fs;
    16	use std::path::PathBuf;
    17	
    18	use ci_affected_target_set::{Change, Decision, GATE_ID, Policy, plan_changes, resolve};
    19	
    20	/// A pack mirroring the shipped oyatie policy shape (the tests stay engine-side: the kernel
    21	/// must work against ANY pack, so fixtures carry their own).
    22	fn policy() -> Policy {
    23	    Policy::from_json(
    24	        r#"{
    25	            "gate_id": "cloud-ci-affected-set",
    26	            "universe": "//...",
    27	            "full_run_targets": ["//..."],
    28	            "full_trigger_patterns": [
    29	                ".buckconfig",
    30	                ".buckconfig.local",
    31	                ".buckconfig.d/**",
    32	                "toolchains/**",
    33	                "third-party/**",
    34	                "**/*.bzl",
    35	                "**/*.bxl",
    36	                "**/PACKAGE",
    37	                "rust-toolchain.toml"
    38	            ],
    39	            "require_owner_patterns": [
    40	                "**/*.rs",
    41	                "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-aarch64-adapter/linker.ld",
    42	                "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/linker.ld"
    43	            ],
    44	            "package_definition_basenames": ["BUCK.v2", "BUCK"],
    45	            "package_sibling_basenames": ["Cargo.toml", "build.rs"],
    46	            "cell_roots": {"": "//"},
    47	            "default_base_ref": "origin/dev"
    48	        }"#,
    49	    )
    50	    .expect("fixture pack parses")
    51	}
    52	
    53	fn owners(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    54	    entries
    55	        .iter()
    56	        .map(|(p, ts)| (p.to_string(), ts.iter().map(|t| t.to_string()).collect()))
    57	        .collect()
    58	}
    59	
    60	// ── The cf16525 class: out-of-scope source change MUST be in the decided set ─────────────
    61	
    62	#[test]
    63	fn red_class_cf16525_out_of_scope_source_lands_in_the_affected_seeds() {
    64	    // PR #651 head cf16525: oya/identity server code did not compile (E0433 x3) yet the
    65	    // binding lane was green because it only ran //cloud/cloud-ci/... . The kernel decision
    66	    // for that diff shape MUST include the owning identity targets as seeds.
    67	    let p = policy();
    68	    let changes = [Change::Present(
    69	        "oya/identity/crates/oya-identity-workload-app/src/server.rs".into(),
    70	    )];
    71	    let plan = plan_changes(&changes, &p);
    72	    let owner_map = owners(&[(
    73	        "oya/identity/crates/oya-identity-workload-app/src/server.rs",
    74	        &["root//oya/identity/crates/oya-identity-workload-app:oya-identity-workload-app"],
    75	    )]);
    76	    let decision = resolve(&plan, &owner_map, &p);
    77	    match decision {
    78	        Decision::Affected { seeds } => {
    79	            assert!(
    80	                seeds
    81	                    .iter()
    82	                    .any(|s| s.contains("oya-identity-workload-app")),
    83	                "the out-of-scope target must be a seed; got {seeds:?}"
    84	            );
    85	        }
    86	        other => panic!("expected Affected, got {other:?}"),
    87	    }
    88	}
    89	
    90	#[test]
    91	fn docs_only_diff_is_unowned_and_not_owner_required() {
    92	    let p = policy();
    93	    let changes = [Change::Present("docs/decisions/ADR-0001-x.md".into())];
    94	    let plan = plan_changes(&changes, &p);
    95	    // owner() ran and found nothing — and .md is not owner-required (F3: the claim is
    96	    // "unowned and not owner-required", NOT "provably outside the build graph").
    97	    let decision = resolve(&plan, &owners(&[("docs/decisions/ADR-0001-x.md", &[])]), &p);
    98	    assert_eq!(decision, Decision::NoGraphTargets);
    99	}
   100	
   101	#[test]
   102	fn owned_non_source_asset_closes_the_include_str_seam() {
   103	    // A .md/.json file CAN be a declared src of a target (include_str! assets). Extension
   104	    // pre-filtering was the old shell driver's false-negative hole; the kernel sends EVERY
   105	    // existing file to owner(), so an owned asset becomes a seed.
   106	    let p = policy();
   107	    let changes = [Change::Present("oya/svc/asset/template.md".into())];
   108	    let plan = plan_changes(&changes, &p);
   109	    let decision = resolve(
   110	        &plan,
   111	        &owners(&[("oya/svc/asset/template.md", &["root//oya/svc:oya-svc"])]),
   112	        &p,
   113	    );
   114	    assert_eq!(
   115	        decision,
   116	        Decision::Affected {
   117	            seeds: vec!["root//oya/svc:oya-svc".into()]
   118	        }
   119	    );
   120	}
   121	
   122	// ── Escape classes: the rdeps cone cannot model these -> FULL, mechanically ──────────────
   123	
   124	#[test]
   125	fn buckconfig_toolchains_third_party_bzl_and_toolchain_pin_escalate_to_full() {
   126	    let p = policy();
   127	    for path in [
   128	        ".buckconfig",
   129	        ".buckconfig.d/extra.bcfg",
   130	        "toolchains/BUCK",
   131	        "toolchains/rust.bzl",
   132	        "third-party/BUCK",
   133	        "third-party/reindeer.toml",
   134	        "third-party/fixups/ring/fixups.toml",
   135	        "infra/macros/defs.bzl",
   136	        "rust-toolchain.toml",
   137	    ] {
   138	        let changes = [Change::Present(path.into())];
   139	        let plan = plan_changes(&changes, &p);
   140	        let decision = resolve(&plan, &BTreeMap::new(), &p);
   141	        match decision {
   142	            Decision::Full { ref reasons } => {
   143	                assert!(
   144	                    reasons.iter().any(|r| r.contains(path)),
   145	                    "FULL reason must name the trigger `{path}`; got {reasons:?}"
   146	                );
   147	            }
   148	            ref other => panic!("`{path}` must escalate to FULL, got {other:?}"),
   149	        }
   150	    }
   151	}
   152	
   153	#[test]
   154	fn red_f2_buildfile_and_config_classes_escalate_to_full() {
   155	    // F2 (reviewer-reproduced silent PASS): buck2 honors more buildfile/config names than a
   156	    // single hand-set "BUCK". A NEW BUCK.v2 SHADOWS the BUCK dependents load; a NEW PACKAGE
   157	    // file evaluates to [] (looks like a plain no-owner file); .buckconfig.local is read by
   158	    // buck2 and committable. Each is added by an empty/valid file -> owner() empty -> would be
   159	    // a silent no-op without these classes. ALL must escalate to FULL.
   160	    let p = policy();
   161	    for path in [
   162	        "libs/oya-buck-syntax-kernel/BUCK.v2",
   163	        "ci/facade/affected-target-set/BUCK.v2",
   164	        "libs/oya-thing/PACKAGE",
   165	        "PACKAGE",
   166	        ".buckconfig.local",
   167	    ] {
   168	        let changes = [Change::Present(path.into())];
   169	        let plan = plan_changes(&changes, &p);
   170	        // owner() is empty for buildfile/config files (BY DESIGN) — pass an empty owner map to
   171	        // prove the FULL escalation comes from classification, not from any owner result.
   172	        let decision = resolve(&plan, &owners(&[(path, &[])]), &p);
   173	        match decision {
   174	            Decision::Full { ref reasons } => assert!(
   175	                reasons.iter().any(|r| r.contains(path)),
   176	                "FULL reason must name `{path}`; got {reasons:?}"
   177	            ),
   178	            ref other => {
   179	                panic!("`{path}` (buildfile/config class) must escalate to FULL, got {other:?}")
   180	            }
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/main.rs | nl -ba | sed -n '480,675p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   480	    ExitCode::from(2)
   481	}
   482	
   483	fn phase(
   484	    phase: impl Into<String>,
   485	    status: impl Into<String>,
   486	    operator_signal: impl Into<String>,
   487	) -> GatePhaseOutcome {
   488	    GatePhaseOutcome::new(phase, status, operator_signal)
   489	}
   490	
   491	/// Auto-mode derivation. Any uncertainty returns `Decision::Full` with the reason (fail-closed
   492	/// escalation); only determined graph-invisibility returns `RefuseUnowned`.
   493	fn derive(args: &Args, base: &str, policy: &Policy, buck2: &str) -> Decision {
   494	    let merge_base = match capture("git", &["merge-base", &args.head, base]) {
   495	        Ok(out) => out.trim().to_owned(),
   496	        Err(e) => {
   497	            return Decision::Full {
   498	                reasons: vec![format!(
   499	                    "derivation uncertainty: git merge-base {} {base} failed: {e}",
   500	                    args.head
   501	                )],
   502	            };
   503	        }
   504	    };
   505	    println!(
   506	        "{LOG}: base={base} head={} merge-base={merge_base}",
   507	        args.head
   508	    );
   509	    let diff = match capture(
   510	        "git",
   511	        &["diff", "--name-status", "-z", &merge_base, &args.head],
   512	    ) {
   513	        Ok(out) => out,
   514	        Err(e) => {
   515	            return Decision::Full {
   516	                reasons: vec![format!("derivation uncertainty: git diff failed: {e}")],
   517	            };
   518	        }
   519	    };
   520	    let changes = match parse_name_status_z(&diff) {
   521	        Ok(c) => c,
   522	        Err(e) => {
   523	            return Decision::Full {
   524	                reasons: vec![format!(
   525	                    "derivation uncertainty: unparseable git diff entry: {e}"
   526	                )],
   527	            };
   528	        }
   529	    };
   530	    if changes.is_empty() {
   531	        println!("{LOG}: no changed files vs merge-base — nothing to derive");
   532	        return Decision::NoGraphTargets;
   533	    }
   534	    println!("{LOG}: {} changed file(s) vs {merge_base}", changes.len());
   535	    let plan = plan_changes(&changes, policy);
   536	    let owner_results = match query_owners(buck2, &plan) {
   537	        Ok(map) => map,
   538	        Err(e) => {
   539	            return Decision::Full {
   540	                reasons: vec![format!(
   541	                    "derivation uncertainty: buck2 owner() query failed: {e}"
   542	                )],
   543	            };
   544	        }
   545	    };
   546	    print_classification(&plan, &owner_results);
   547	    resolve(&plan, &owner_results, policy)
   548	}
   549	
   550	/// Parse `git diff --name-status -z` output: NUL-separated records, `R`/`C` carry two paths.
   551	fn parse_name_status_z(raw: &str) -> Result<Vec<Change>, String> {
   552	    let mut fields = raw.split('\0').filter(|s| !s.is_empty());
   553	    let mut changes = Vec::new();
   554	    while let Some(status) = fields.next() {
   555	        let kind = status.chars().next().ok_or("empty status field")?;
   556	        match kind {
   557	            'A' | 'M' | 'T' => {
   558	                let p = fields
   559	                    .next()
   560	                    .ok_or_else(|| format!("status `{status}` without a path"))?;
   561	                changes.push(Change::Present(p.to_owned()));
   562	            }
   563	            'D' => {
   564	                let p = fields
   565	                    .next()
   566	                    .ok_or_else(|| format!("status `{status}` without a path"))?;
   567	                changes.push(Change::Deleted(p.to_owned()));
   568	            }
   569	            'R' => {
   570	                let old = fields
   571	                    .next()
   572	                    .ok_or_else(|| format!("status `{status}` without source path"))?;
   573	                let new = fields
   574	                    .next()
   575	                    .ok_or_else(|| format!("status `{status}` without destination path"))?;
   576	                changes.push(Change::Deleted(old.to_owned()));
   577	                changes.push(Change::Present(new.to_owned()));
   578	            }
   579	            'C' => {
   580	                let _src = fields
   581	                    .next()
   582	                    .ok_or_else(|| format!("status `{status}` without source path"))?;
   583	                let dst = fields
   584	                    .next()
   585	                    .ok_or_else(|| format!("status `{status}` without destination path"))?;
   586	                changes.push(Change::Present(dst.to_owned()));
   587	            }
   588	            // U (unmerged), X (unknown), B (broken pair): states a clean CI checkout cannot
   589	            // produce — surface as uncertainty rather than guessing.
   590	            other => return Err(format!("unsupported diff status `{other}`")),
   591	        }
   592	    }
   593	    Ok(changes)
   594	}
   595	
   596	/// Batched per-file owner resolution: `buck2 uquery --json "owner(%s)" @argfile` returns a
   597	/// JSON object keyed by each path. A query ERROR is uncertainty (caller escalates) — it is
   598	/// NEVER treated as "no owner" (the historic false-pass bug class).
   599	fn query_owners(buck2: &str, plan: &Plan) -> Result<BTreeMap<String, Vec<String>>, String> {
   600	    if plan.owner_paths.is_empty() {
   601	        return Ok(BTreeMap::new());
   602	    }
   603	    let argfile = write_argfile("owner-paths", &plan.owner_paths).map_err(|e| e.to_string())?;
   604	    let out = capture(
   605	        buck2,
   606	        &[
   607	            "uquery",
   608	            "--json",
   609	            "owner(%s)",
   610	            &format!("@{}", argfile.display()),
   611	        ],
   612	    )?;
   613	    let v: serde_json::Value =
   614	        serde_json::from_str(&out).map_err(|e| format!("owner() output is not JSON: {e}"))?;
   615	    let obj = v.as_object().ok_or("owner() JSON is not an object")?;
   616	    let mut map = BTreeMap::new();
   617	    for (path, owners) in obj {
   618	        let list = owners.as_array().ok_or("owner() entry is not an array")?;
   619	        let mut targets = Vec::with_capacity(list.len());
   620	        for t in list {
   621	            targets.push(
   622	                t.as_str()
   623	                    .ok_or("owner() target is not a string")?
   624	                    .to_owned(),
   625	            );
   626	        }
   627	        map.insert(path.clone(), targets);
   628	    }
   629	    Ok(map)
   630	}
   631	
   632	/// Seeds -> reverse-dependency closure within the policy universe, via @argfile + `%Ss`
   633	/// (arbitrary set size; an inline set overflows on large packages).
   634	fn rdeps_closure(buck2: &str, policy: &Policy, seeds: &[String]) -> Result<Vec<String>, String> {
   635	    let argfile = write_argfile("seeds", seeds).map_err(|e| e.to_string())?;
   636	    let query = format!("rdeps({}, %Ss)", policy.universe);
   637	    let out = capture(
   638	        buck2,
   639	        &["uquery", &query, &format!("@{}", argfile.display())],
   640	    )?;
   641	    let targets: Vec<String> = out
   642	        .lines()
   643	        .map(str::trim)
   644	        .filter(|l| !l.is_empty())
   645	        .map(str::to_owned)
   646	        .collect();
   647	    if targets.is_empty() {
   648	        return Err(
   649	            "rdeps returned an empty closure for non-empty seeds (query problem)".to_owned(),
   650	        );
   651	    }
   652	    Ok(targets)
   653	}
   654	
   655	fn print_classification(plan: &Plan, owners: &BTreeMap<String, Vec<String>>) {
   656	    println!("{LOG}: classification (every changed file, mechanically derived):");
   657	    for (path, class) in &plan.classified {
   658	        match class {
   659	            PathClass::FullTrigger(pat) => {
   660	                println!("{LOG}:   FULL-TRIGGER {path} (matches `{pat}`)")
   661	            }
   662	            PathClass::DeletedGraphFile => {
   663	                println!("{LOG}:   FULL-TRIGGER {path} (graph file deleted/unmappable)")
   664	            }
   665	            PathClass::Buildfile => {
   666	                println!(
   667	                    "{LOG}:   FULL-TRIGGER {path} (buildfile — blast radius exceeds its package)"
   668	                )
   669	            }
   670	            PathClass::PackagePattern(pat) => println!("{LOG}:   PACKAGE      {path} -> {pat}"),
   671	            PathClass::OwnerQuery => {
   672	                let n = owners.get(path).map(Vec::len).unwrap_or(0);
   673	                println!("{LOG}:   OWNER        {path} -> {n} target(s)");
   674	            }
   675	            PathClass::DeletedIrrelevant => {
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/main.rs | nl -ba | sed -n '120,430p'; git show origin/dev:ci/facade/affected-target-set/src/main.rs | nl -ba | sed -n '680,855p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   120	    // The policy pack is the ONLY hard input: without it not even the full-run target set is
   121	    // known, so a missing/invalid pack cannot escalate — it fails.
   122	    let policy_bytes = match fs::read_to_string(&args.policy_path) {
   123	        Ok(b) => b,
   124	        Err(e) => {
   125	            eprintln!(
   126	                "{LOG}: POLICY ERROR: cannot read `{}`: {e}",
   127	                args.policy_path
   128	            );
   129	            return ExitCode::from(2);
   130	        }
   131	    };
   132	    let policy = match Policy::from_json(&policy_bytes) {
   133	        Ok(p) => p,
   134	        Err(e) => {
   135	            eprintln!("{LOG}: POLICY ERROR ({GATE_ID}): {e}");
   136	            return ExitCode::from(2);
   137	        }
   138	    };
   139	    let buck2 = std::env::var("BUCK2").unwrap_or_else(|_| "buck2".to_owned());
   140	    let base = args
   141	        .base
   142	        .clone()
   143	        .unwrap_or_else(|| policy.default_base_ref.clone());
   144	    let artifact_context = match build_artifact_context(&args, &base) {
   145	        Ok(context) => context,
   146	        Err(e) => {
   147	            eprintln!("{LOG}: FAIL — cannot resolve refs for affected-set operator artifact: {e}");
   148	            return ExitCode::from(2);
   149	        }
   150	    };
   151	
   152	    let decision = match args.mode {
   153	        Mode::Full => Decision::Full {
   154	            reasons: vec!["--mode full (admission/integration tier)".to_owned()],
   155	        },
   156	        Mode::Auto => derive(&args, &base, &policy, &buck2),
   157	    };
   158	
   159	    match decision {
   160	        Decision::RefuseUnowned { paths } => {
   161	            let final_decision = Decision::RefuseUnowned {
   162	                paths: paths.clone(),
   163	            };
   164	            let phases = vec![
   165	                phase("derive-affected-set-tier", "completed", "decision.tier"),
   166	                phase(
   167	                    "binding-build-test",
   168	                    "not-run",
   169	                    "owner-required file refused before build",
   170	                ),
   171	            ];
   172	            if let Err(e) =
   173	                maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
   174	            {
   175	                return artifact_failure(e);
   176	            }
   177	            eprintln!("{LOG}: FAIL — owner-required file(s) with NO owning buck2 target:");
   178	            for p in &paths {
   179	                eprintln!("{LOG}:   {p}");
   180	            }
   181	            eprintln!(
   182	                "{LOG}: graph-invisible code cannot be made safe by running more targets — even a \
   183	                 full-workspace run would not compile these files. Wire them into a BUCK target \
   184	                 (or delete them); refusing to false-green."
   185	            );
   186	            ExitCode::from(2)
   187	        }
   188	        Decision::NoGraphTargets => {
   189	            let final_decision = Decision::NoGraphTargets;
   190	            let phases = vec![
   191	                phase("derive-affected-set-tier", "completed", "decision.tier"),
   192	                phase("binding-build-test", "not-run", "no graph targets"),
   193	            ];
   194	            if let Err(e) =
   195	                maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
   196	            {
   197	                return artifact_failure(e);
   198	            }
   199	            println!(
   200	                "{LOG}: decision=NO-GRAPH-TARGETS — every changed file is unowned and not in any \
   201	                 owner-required class (docs/config-text outside the buildfile/escape classes) -> PASS"
   202	            );
   203	            ExitCode::SUCCESS
   204	        }
   205	        Decision::Full { reasons } => {
   206	            let final_decision = Decision::Full {
   207	                reasons: reasons.clone(),
   208	            };
   209	            println!("{LOG}: decision=FULL — running the complete workspace, because:");
   210	            for r in &reasons {
   211	                println!("{LOG}:   - {r}");
   212	            }
   213	            if args.derive_only {
   214	                let phases = vec![
   215	                    phase("derive-affected-set-tier", "completed", "decision.tier"),
   216	                    phase(
   217	                        "materialize-merge-base-build-health-baseline",
   218	                        if args.baseline_report.is_some() {
   219	                            "present"
   220	                        } else if args.mode == Mode::Auto {
   221	                            "absent"
   222	                        } else {
   223	                            "not-required"
   224	                        },
   225	                        "merge_base_build_health_baseline.report_present",
   226	                    ),
   227	                    phase("binding-build-test", "not-run", "--derive-only"),
   228	                ];
   229	                if let Err(e) =
   230	                    maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
   231	                {
   232	                    return artifact_failure(e);
   233	                }
   234	                println!(
   235	                    "{LOG}: --derive-only: would run `{buck2} build` + `{buck2} test` on: {}",
   236	                    policy.full_run_targets.join(" ")
   237	                );
   238	                return ExitCode::SUCCESS;
   239	            }
   240	            let code = run_full(&buck2, &policy, args.baseline_report.as_deref());
   241	            let phases = vec![
   242	                phase(
   243	                    "derive-affected-set-tier",
   244	                    if args.mode == Mode::Full {
   245	                        "bypassed-mode-full"
   246	                    } else {
   247	                        "completed"
   248	                    },
   249	                    "decision.tier",
   250	                ),
   251	                phase(
   252	                    "materialize-merge-base-build-health-baseline",
   253	                    if args.baseline_report.is_some() {
   254	                        "present"
   255	                    } else if args.mode == Mode::Auto {
   256	                        "absent"
   257	                    } else {
   258	                        "not-required"
   259	                    },
   260	                    "merge_base_build_health_baseline.report_present",
   261	                ),
   262	                phase(
   263	                    "binding-build-test",
   264	                    "completed-check-exit-code",
   265	                    "FULL workspace run completed; verdict is process exit code",
   266	                ),
   267	            ];
   268	            if let Err(e) =
   269	                maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
   270	            {
   271	                return artifact_failure(e);
   272	            }
   273	            code
   274	        }
   275	        Decision::Affected { seeds } => {
   276	            println!(
   277	                "{LOG}: decision=AFFECTED — {} seed target(s) from owners + package patterns",
   278	                seeds.len()
   279	            );
   280	            match rdeps_closure(&buck2, &policy, &seeds) {
   281	                Ok(targets) => {
   282	                    println!(
   283	                        "{LOG}: {} affected target(s) (seeds + reverse-dependency closure):",
   284	                        targets.len()
   285	                    );
   286	                    for t in &targets {
   287	                        println!("{LOG}:   {t}");
   288	                    }
   289	                    let final_decision = Decision::Affected {
   290	                        seeds: seeds.clone(),
   291	                    };
   292	                    if args.derive_only {
   293	                        let phases = vec![
   294	                            phase("derive-affected-set-tier", "completed", "decision.tier"),
   295	                            phase(
   296	                                "rdeps-closure",
   297	                                "completed",
   298	                                format!("{} affected target(s)", targets.len()),
   299	                            ),
   300	                            phase("binding-build-test", "not-run", "--derive-only"),
   301	                        ];
   302	                        if let Err(e) = maybe_write_decision_artifact(
   303	                            &artifact_context,
   304	                            &final_decision,
   305	                            &phases,
   306	                        ) {
   307	                            return artifact_failure(e);
   308	                        }
   309	                        println!("{LOG}: --derive-only: stopping before build/test.");
   310	                        return ExitCode::SUCCESS;
   311	                    }
   312	                    match write_argfile("targets", &targets) {
   313	                        Ok(path) => {
   314	                            let code = run_buck(&buck2, &[], Some(&path));
   315	                            let phases = vec![
   316	                                phase("derive-affected-set-tier", "completed", "decision.tier"),
   317	                                phase(
   318	                                    "rdeps-closure",
   319	                                    "completed",
   320	                                    format!("{} affected target(s)", targets.len()),
   321	                                ),
   322	                                phase(
   323	                                    "target-argfile",
   324	                                    "completed",
   325	                                    format!("target list preserved at {}", path.display()),
   326	                                ),
   327	                                phase(
   328	                                    "binding-build-test",
   329	                                    "completed-check-exit-code",
   330	                                    "affected target build/test completed; verdict is process exit code",
   331	                                ),
   332	                            ];
   333	                            if let Err(e) = maybe_write_decision_artifact(
   334	                                &artifact_context,
   335	                                &final_decision,
   336	                                &phases,
   337	                            ) {
   338	                                return artifact_failure(e);
   339	                            }
   340	                            code
   341	                        }
   342	                        Err(e) => {
   343	                            // Cannot even materialize the argfile: escalate, never skip.
   344	                            println!("{LOG}: ESCALATE to FULL — argfile write failed: {e}");
   345	                            let final_decision = Decision::Full {
   346	                                reasons: vec![format!(
   347	                                    "argfile write failed after AFFECTED decision: {e}"
   348	                                )],
   349	                            };
   350	                            let code = run_full(&buck2, &policy, args.baseline_report.as_deref());
   351	                            let phases = vec![
   352	                                phase("derive-affected-set-tier", "completed", "decision.tier"),
   353	                                phase(
   354	                                    "rdeps-closure",
   355	                                    "completed",
   356	                                    format!("{} affected target(s)", targets.len()),
   357	                                ),
   358	                                phase("target-argfile", "failed-escalated", e.to_string()),
   359	                                phase(
   360	                                    "binding-build-test",
   361	                                    "completed-check-exit-code",
   362	                                    "FULL escalation executed after argfile failure",
   363	                                ),
   364	                            ];
   365	                            if let Err(e) = maybe_write_decision_artifact(
   366	                                &artifact_context,
   367	                                &final_decision,
   368	                                &phases,
   369	                            ) {
   370	                                return artifact_failure(e);
   371	                            }
   372	                            code
   373	                        }
   374	                    }
   375	                }
   376	                Err(reason) => {
   377	                    println!("{LOG}: ESCALATE to FULL — {reason}");
   378	                    let final_decision = Decision::Full {
   379	                        reasons: vec![format!(
   380	                            "rdeps closure failed after AFFECTED decision: {reason}"
   381	                        )],
   382	                    };
   383	                    if args.derive_only {
   384	                        let phases = vec![
   385	                            phase("derive-affected-set-tier", "completed", "decision.tier"),
   386	                            phase("rdeps-closure", "failed-escalated", reason.clone()),
   387	                            phase("binding-build-test", "not-run", "--derive-only"),
   388	                        ];
   389	                        if let Err(e) = maybe_write_decision_artifact(
   390	                            &artifact_context,
   391	                            &final_decision,
   392	                            &phases,
   393	                        ) {
   394	                            return artifact_failure(e);
   395	                        }
   396	                        println!(
   397	                            "{LOG}: --derive-only: would run the full workspace: {}",
   398	                            policy.full_run_targets.join(" ")
   399	                        );
   400	                        return ExitCode::SUCCESS;
   401	                    }
   402	                    let code = run_full(&buck2, &policy, args.baseline_report.as_deref());
   403	                    let phases = vec![
   404	                        phase("derive-affected-set-tier", "completed", "decision.tier"),
   405	                        phase("rdeps-closure", "failed-escalated", reason),
   406	                        phase(
   407	                            "binding-build-test",
   408	                            "completed-check-exit-code",
   409	                            "FULL escalation executed after rdeps failure",
   410	                        ),
   411	                    ];
   412	                    if let Err(e) =
   413	                        maybe_write_decision_artifact(&artifact_context, &final_decision, &phases)
   414	                    {
   415	                        return artifact_failure(e);
   416	                    }
   417	                    code
   418	                }
   419	            }
   420	        }
   421	    }
   422	}
   423	
   424	fn mode_name(mode: Mode) -> &'static str {
   425	    match mode {
   426	        Mode::Auto => "auto",
   427	        Mode::Full => "full",
   428	    }
   429	}
   430	
   680	}
   681	
   682	/// The FULL-tier runner (ADR-0554 round-3; D7 round-4 producer). Two modes:
   683	///
   684	/// - WITHOUT a baseline report (`--mode full` at admission, or any caller that does not pass
   685	///   `--baseline-report`): a hard `buck2 build //... --keep-going --build-report` + `buck2 test
   686	///   //...` — EVERY build failure blocks (non-empty failure set = hard fail; no grandfathering:
   687	///   the integration tip MUST be green). D7 (round-4): the admission build now captures a
   688	///   `--build-report` as a PURE BYPRODUCT and derives the same hard verdict from the report's
   689	///   failure set being EMPTY. The report is written to a stable path (`admission_report_path`)
   690	///   so the trusted push-to-dev workflow can publish it as the `build-health-baseline-<sha>`
   691	///   artifact (the merge-base-to-be baseline for the DEFERRED D8 cross-run consumer + ADR-0560
   692	///   warm-CAS). Merge authority is UNCHANGED — the verdict is identical to the prior hard build,
   693	///   nothing consumes the artifact yet, so there is zero laundering surface.
   694	/// - WITH a baseline report (the PR `pull_request` FULL tier): the BUILD-HEALTH RATCHET. It builds
   695	///   `//... --keep-going --build-report` at HEAD and tests them, then compares the HEAD build
   696	///   FAILURE set against the merge-base baseline failure set: only REGRESSIONS (targets that build
   697	///   at the merge-base but fail at head, or brand-new failing targets) block; pre-existing build
   698	///   debt is grandfathered. This turns the FULL tier from a flag-day requirement into a true
   699	///   ratchet (block new debt, grandfather pre-existing — FRIC-1781112000 / #698). Tests are still
   700	///   run and a TEST regression in a buildable target blocks via the test exit (the ratchet governs
   701	///   BUILD failures; a build that succeeds then test-fails is a normal hard failure).
   702	fn run_full(buck2: &str, policy: &Policy, baseline_report: Option<&str>) -> ExitCode {
   703	    let Some(baseline_path) = baseline_report else {
   704	        // Admission/integration tier: hard full build+test, every failure blocks. D7: emit the
   705	        // build-report as a byproduct and derive the hard verdict from an EMPTY failure set.
   706	        return run_full_admission_producer(buck2, policy);
   707	    };
   708	
   709	    // PR FULL tier: build-health ratchet. Build the whole workspace with --keep-going so every
   710	    // target's status is recorded even past the first failure, into a build-report.
   711	    let head_report = match std::env::temp_dir()
   712	        .join(format!(
   713	            "{GATE_ID}-head-build-report-{}.json",
   714	            std::process::id()
   715	        ))
   716	        .into_os_string()
   717	        .into_string()
   718	    {
   719	        Ok(p) => p,
   720	        Err(_) => {
   721	            eprintln!("{LOG}: FAIL — could not form a temp path for the head build-report");
   722	            return ExitCode::from(2);
   723	        }
   724	    };
   725	    println!(
   726	        "{LOG}: FULL tier (build-health ratchet vs merge-base baseline {baseline_path}): \
   727	         {buck2} build //... --keep-going --build-report {head_report}"
   728	    );
   729	    // We intentionally do NOT propagate this build's exit code: --keep-going still exits non-zero
   730	    // if ANY target failed, but pre-existing failures must NOT block. The verdict comes from the
   731	    // build-report diff below. (A genuine infra failure surfaces as an unparseable/empty report,
   732	    // which the ratchet then refuses on — fail-closed.)
   733	    let mut command = Command::new(buck2);
   734	    command.args([
   735	        "build",
   736	        "//...",
   737	        "--keep-going",
   738	        "--build-report",
   739	        &head_report,
   740	    ]);
   741	    if let Err(e) = run_command_with_progress(
   742	        "build-health-ratchet-head-build",
   743	        &mut command,
   744	        &format!("{buck2} build //... --keep-going --build-report {head_report}"),
   745	    ) {
   746	        eprintln!("{LOG}: WARN — could not execute head build-health command: {e}");
   747	    }
   748	
   749	    let baseline_json = match fs::read_to_string(baseline_path) {
   750	        Ok(s) => s,
   751	        Err(e) => {
   752	            eprintln!(
   753	                "{LOG}: FAIL — cannot read merge-base baseline report `{baseline_path}`: {e}"
   754	            );
   755	            return ExitCode::from(2);
   756	        }
   757	    };
   758	    let head_json = match fs::read_to_string(&head_report) {
   759	        Ok(s) => s,
   760	        Err(e) => {
   761	            eprintln!("{LOG}: FAIL — cannot read head build-report `{head_report}`: {e}");
   762	            return ExitCode::from(2);
   763	        }
   764	    };
   765	    let baseline = match parse_build_report(&baseline_json) {
   766	        Ok(r) => r,
   767	        Err(e) => {
   768	            eprintln!("{LOG}: FAIL — merge-base baseline report parse error: {e}");
   769	            return ExitCode::from(2);
   770	        }
   771	    };
   772	    let head = match parse_build_report(&head_json) {
   773	        Ok(r) => r,
   774	        Err(e) => {
   775	            eprintln!("{LOG}: FAIL — head build-report parse error: {e}");
   776	            return ExitCode::from(2);
   777	        }
   778	    };
   779	    // Fail-closed laundering guard: an empty merge-base baseline would grandfather every head
   780	    // failure. CI builds the whole merge-base workspace, so the baseline is never legitimately
   781	    // empty — refuse rather than silently pass.
   782	    if baseline.is_empty() {
   783	        eprintln!(
   784	            "{LOG}: FAIL — merge-base baseline build-report has no `results`. Refusing to \
   785	             grandfather every head failure against an empty baseline (the laundering hole)."
   786	        );
   787	        return ExitCode::from(2);
   788	    }
   789	
   790	    let baseline_failures = failing_targets(&baseline);
   791	    let head_failures = failing_targets(&head);
   792	    let verdict = build_health_verdict(&baseline_failures, &head_failures);
   793	    println!(
   794	        "{LOG}: build-health — head build failures={}, baseline failures={}, regressions={}, \
   795	         grandfathered={}, fixed={}",
   796	        head_failures.len(),
   797	        baseline_failures.len(),
   798	        verdict.regressions.len(),
   799	        verdict.grandfathered.len(),
   800	        verdict.fixed.len()
   801	    );
   802	    for t in &verdict.grandfathered {
   803	        println!("{LOG}:   pre-existing-red (grandfathered) {t}");
   804	    }
   805	    if !verdict.is_green() {
   806	        eprintln!(
   807	            "{LOG}: RED — {} build REGRESSION(S) vs the merge-base (built at origin/dev, FAIL at \
   808	             head — or brand-new failing target):",
   809	            verdict.regressions.len()
   810	        );
   811	        for t in &verdict.regressions {
   812	            eprintln!("{LOG}:   REGRESSION {t}");
   813	        }
   814	        eprintln!(
   815	            "{LOG}: REMEDIATION: fix these targets or revert the change that broke them; \
   816	             pre-existing failures are grandfathered, only NEW build debt blocks. REPRODUCE: \
   817	             {buck2} build {} --keep-going",
   818	            verdict.regressions.join(" ")
   819	        );
   820	        return ExitCode::from(1);
   821	    }
   822	
   823	    // No build regressions -> GREEN. SCOPE (ADR-0554 round-3): the FULL tier governs BUILD health
   824	    // (the cf16525 class is a COMPILE break). It deliberately does NOT run a workspace-wide
   825	    // `buck2 test //...`: that would reintroduce a flag-day on PRE-EXISTING test failures (the
   826	    // exact debt-grandfathering problem this round fixes, one layer up). Test coverage of the
   827	    // ACTUAL changed code is the cone path's job (auto mode, hard-fail, unchanged — the cf16525
   828	    // fixture); a FULL-tier TEST-health ratchet (same baseline-diff over a test report) is the
   829	    // declared next IP. Conservative and sound: never false-green on a build regression, never
   830	    // flag-day on pre-existing debt.
   831	    println!(
   832	        "{LOG}: PASS — no build regressions vs the merge-base ({} pre-existing build failure(s) \
   833	         grandfathered).",
   834	        verdict.grandfathered.len()
   835	    );
   836	    ExitCode::SUCCESS
   837	}
   838	
   839	/// The stable path the admission build-report is written to (D7). GitHub Actions sets
   840	/// `RUNNER_TEMP`; we anchor the report there so the workflow's upload step references the SAME
   841	/// path without guessing a PID. Off-CI (or if `RUNNER_TEMP` is unset) it falls back to the OS
   842	/// temp dir with the identical basename — deterministic either way.
   843	fn admission_report_path() -> PathBuf {
   844	    let dir = std::env::var_os("RUNNER_TEMP")
   845	        .map(PathBuf::from)
   846	        .unwrap_or_else(std::env::temp_dir);
   847	    dir.join("build-health-admission-report.json")
   848	}
   849	
   850	fn long_step_telemetry_interval() -> Duration {
   851	    std::env::var("OYA_CI_LONG_STEP_TELEMETRY_INTERVAL_SECS")
   852	        .ok()
   853	        .and_then(|raw| raw.parse::<u64>().ok())
   854	        .filter(|seconds| *seconds > 0)
   855	        .map(Duration::from_secs)
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/main.rs | nl -ba | sed -n '855,980p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   855	        .map(Duration::from_secs)
   856	        .unwrap_or_else(|| Duration::from_secs(30))
   857	}
   858	
   859	fn run_command_with_progress(
   860	    phase: &str,
   861	    command: &mut Command,
   862	    pretty: &str,
   863	) -> std::io::Result<ExitStatus> {
   864	    let started = Instant::now();
   865	    println!(
   866	        "{}",
   867	        long_step_telemetry_line(LOG, phase, "started", 0, &format!("command={pretty}"))
   868	    );
   869	
   870	    let mut child = command.stdin(Stdio::null()).spawn()?;
   871	    let interval = long_step_telemetry_interval();
   872	    let poll_interval = if interval < Duration::from_millis(250) {
   873	        interval
   874	    } else {
   875	        Duration::from_millis(250)
   876	    };
   877	    let mut last_running_emit = started;
   878	
   879	    loop {
   880	        if let Some(status) = child.try_wait()? {
   881	            println!(
   882	                "{}",
   883	                long_step_telemetry_line(
   884	                    LOG,
   885	                    phase,
   886	                    "completed",
   887	                    started.elapsed().as_secs(),
   888	                    &format!("exit_status={status}"),
   889	                )
   890	            );
   891	            return Ok(status);
   892	        }
   893	
   894	        if last_running_emit.elapsed() >= interval {
   895	            println!(
   896	                "{}",
   897	                long_step_telemetry_line(
   898	                    LOG,
   899	                    phase,
   900	                    "running",
   901	                    started.elapsed().as_secs(),
   902	                    &format!("command={pretty}"),
   903	                )
   904	            );
   905	            last_running_emit = Instant::now();
   906	        }
   907	        thread::sleep(poll_interval);
   908	    }
   909	}
   910	
   911	/// The admission/integration FULL tier (D7 producer). Runs `buck2 build //... --keep-going
   912	/// --build-report <stable path>` so the WHOLE workspace builds and every target's status is
   913	/// captured into a report (a pure byproduct the trusted push-to-dev workflow publishes), then
   914	/// derives the HARD verdict from the report's FAILURE SET being EMPTY — non-empty = hard fail,
   915	/// NO grandfathering (the integration tip MUST be green, preserving `run_buck`'s admission
   916	/// semantics). Finally runs `buck2 test //...` exactly as before. The verdict is identical to the
   917	/// prior hard `buck2 build //...`; emitting the report does not change merge authority.
   918	fn run_full_admission_producer(buck2: &str, policy: &Policy) -> ExitCode {
   919	    let report_path = admission_report_path();
   920	    let report_str = report_path.display().to_string();
   921	    println!(
   922	        "{LOG}: admission FULL tier (D7 producer): {buck2} build //... --keep-going \
   923	         --build-report {report_str}"
   924	    );
   925	    // --keep-going still exits non-zero if ANY target failed; we do NOT read that exit code as the
   926	    // verdict — the verdict comes from the build-report failure set below, so the report (the
   927	    // published byproduct) and the pass/fail decision are derived from the SAME source of truth. A
   928	    // genuine infra failure (buck2 could not run, no report) surfaces as an unparseable/empty
   929	    // report, which is refused fail-closed.
   930	    let mut command = Command::new(buck2);
   931	    command.args([
   932	        "build",
   933	        "//...",
   934	        "--keep-going",
   935	        "--build-report",
   936	        &report_str,
   937	    ]);
   938	    if let Err(e) = run_command_with_progress(
   939	        "admission-full-build-health-baseline",
   940	        &mut command,
   941	        &format!("{buck2} build //... --keep-going --build-report {report_str}"),
   942	    ) {
   943	        eprintln!("{LOG}: WARN — could not execute admission build-health command: {e}");
   944	    }
   945	
   946	    let report_json = match fs::read_to_string(&report_path) {
   947	        Ok(s) => s,
   948	        Err(e) => {
   949	            eprintln!("{LOG}: FAIL — cannot read admission build-report `{report_str}`: {e}");
   950	            return ExitCode::from(2);
   951	        }
   952	    };
   953	    let report = match parse_build_report(&report_json) {
   954	        Ok(r) => r,
   955	        Err(e) => {
   956	            eprintln!("{LOG}: FAIL — admission build-report parse error: {e}");
   957	            return ExitCode::from(2);
   958	        }
   959	    };
   960	    // Fail-closed: an admission build with no `results` is an infra failure, not a clean
   961	    // workspace — refuse rather than false-green on an empty report.
   962	    if report.is_empty() {
   963	        eprintln!(
   964	            "{LOG}: FAIL — admission build-report has no `results` (buck2 produced no targets). \
   965	             Refusing to PASS the integration tip on an empty report."
   966	        );
   967	        return ExitCode::from(2);
   968	    }
   969	
   970	    let failures = failing_targets(&report);
   971	    if !failures.is_empty() {
   972	        // No grandfathering at admission: the integration tip MUST be green (the `run_buck`
   973	        // hard-build semantics, now derived from the report's failure set).
   974	        eprintln!(
   975	            "{LOG}: RED — admission FULL build failed on {} target(s) (integration tip must be \
   976	             green, no grandfathering):",
   977	            failures.len()
   978	        );
   979	        for t in &failures {
   980	            eprintln!("{LOG}:   BUILD-FAIL {t}");
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/main.rs | nl -ba | sed -n '970,1065p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   970	    let failures = failing_targets(&report);
   971	    if !failures.is_empty() {
   972	        // No grandfathering at admission: the integration tip MUST be green (the `run_buck`
   973	        // hard-build semantics, now derived from the report's failure set).
   974	        eprintln!(
   975	            "{LOG}: RED — admission FULL build failed on {} target(s) (integration tip must be \
   976	             green, no grandfathering):",
   977	            failures.len()
   978	        );
   979	        for t in &failures {
   980	            eprintln!("{LOG}:   BUILD-FAIL {t}");
   981	        }
   982	        eprintln!(
   983	            "{LOG}: REPRODUCE: {buck2} build {} --keep-going",
   984	            failures.iter().cloned().collect::<Vec<_>>().join(" ")
   985	        );
   986	        return ExitCode::from(1);
   987	    }
   988	    println!(
   989	        "{LOG}: admission build GREEN — all {} workspace target(s) built; running {buck2} test \
   990	         //... (report byproduct at {report_str})",
   991	        report.len()
   992	    );
   993	    // Build is green -> run the full test suite exactly as the prior admission path did.
   994	    run_buck(buck2, &policy.full_run_targets, None)
   995	}
   996	
   997	/// Run `buck2 build` then `buck2 test` on either explicit patterns or an @argfile, streaming
   998	/// output. On failure prints the exact reproduction command and propagates the exit code.
   999	fn run_buck(buck2: &str, patterns: &[String], argfile: Option<&PathBuf>) -> ExitCode {
  1000	    let spec: Vec<String> = match argfile {
  1001	        Some(path) => vec![format!("@{}", path.display())],
  1002	        None => patterns.to_vec(),
  1003	    };
  1004	    for verb in ["build", "test"] {
  1005	        let mut pretty = format!("{buck2} {verb}");
  1006	        for s in &spec {
  1007	            pretty.push(' ');
  1008	            pretty.push_str(s);
  1009	        }
  1010	        println!("{LOG}: === {pretty} ===");
  1011	        let phase = match verb {
  1012	            "build" => "binding-build",
  1013	            "test" => "binding-test",
  1014	            _ => "binding-build-test",
  1015	        };
  1016	        let mut command = Command::new(buck2);
  1017	        command.arg(verb).args(&spec);
  1018	        let status = run_command_with_progress(phase, &mut command, &pretty);
  1019	        match status {
  1020	            Ok(st) if st.success() => {}
  1021	            Ok(st) => {
  1022	                eprintln!("{LOG}: FAIL — `{pretty}` exited with {st}");
  1023	                eprintln!("{LOG}: ran on: {}", spec.join(" "));
  1024	                if let Some(path) = argfile {
  1025	                    eprintln!("{LOG}: target list preserved at {}", path.display());
  1026	                }
  1027	                eprintln!("{LOG}: REPRODUCE: {pretty}");
  1028	                return ExitCode::from(u8::try_from(st.code().unwrap_or(1)).unwrap_or(1));
  1029	            }
  1030	            Err(e) => {
  1031	                eprintln!("{LOG}: FAIL — could not execute `{pretty}`: {e}");
  1032	                return ExitCode::from(1);
  1033	            }
  1034	        }
  1035	    }
  1036	    println!("{LOG}: PASS");
  1037	    ExitCode::SUCCESS
  1038	}
  1039	
  1040	fn capture(bin: &str, args: &[&str]) -> Result<String, String> {
  1041	    let out = Command::new(bin)
  1042	        .args(args)
  1043	        .stdin(Stdio::null())
  1044	        .output()
  1045	        .map_err(|e| format!("could not execute `{bin} {}`: {e}", args.join(" ")))?;
  1046	    if !out.status.success() {
  1047	        return Err(format!(
  1048	            "`{bin} {}` exited with {}: {}",
  1049	            args.join(" "),
  1050	            out.status,
  1051	            String::from_utf8_lossy(&out.stderr).trim()
  1052	        ));
  1053	    }
  1054	    String::from_utf8(out.stdout).map_err(|e| format!("`{bin}` output is not UTF-8: {e}"))
  1055	}
  1056	
  1057	fn write_argfile(stem: &str, lines: &[String]) -> std::io::Result<PathBuf> {
  1058	    let path = std::env::temp_dir().join(format!("{GATE_ID}-{stem}-{}.txt", std::process::id()));
  1059	    let mut f = fs::File::create(&path)?;
  1060	    for line in lines {
  1061	        writeln!(f, "{line}")?;
  1062	    }
  1063	    Ok(path)
  1064	}
exec
/bin/zsh -lc "git ls-tree -r --name-only HEAD .claire | sed -n '1,80p'; git ls-tree -r --name-only origin/dev .claire | sed -n '1,80p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
exec
/bin/zsh -lc "git log --all --diff-filter=D --format='%H %ad %s' --date=short -- .claire/worktrees | sed -n '1,40p'" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git grep -n -F 'materialize-cloud-ci-generated-faces.sh' origin/dev -- . || true" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git grep -n -F 'buck2-affected-gate.sh' origin/dev -- ':"'!infra/ci/buck2-affected-gate.sh'"'" in /Users/jasonlee/Developer/oyatie
 succeeded in 298ms:
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:63:        "infra/ci/materialize-cloud-ci-generated-faces.sh",
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:151:      "path": "infra/ci/materialize-cloud-ci-generated-faces.sh",
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:553:      "snippet": "run: infra/ci/materialize-cloud-ci-generated-faces.sh . | - name: Upload regenerated faces | # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI."
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:567:      "snippet": "run: infra/ci/materialize-cloud-ci-generated-faces.sh . | - name: cargo test ${{ matrix.crate }}"
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:609:      "snippet": "infra/ci/materialize-cloud-ci-generated-faces.sh . | cargo test --locked --no-fail-fast -p registry-drift -- --test-threads=1 | # ── cloud-ci-firewall: the baseline ratchet (blocks only NEW debt) + the gate-registration | #    meta-test (no in-tree gate may go"
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:623:      "snippet": "run: infra/ci/materialize-cloud-ci-generated-faces.sh . | - name: cargo test cloud-ci-firewall"
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:672:      "snippet": "run: infra/ci/materialize-cloud-ci-generated-faces.sh . | # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication — | # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests | # run green, ful"
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:27:| `scm_facts_emitter_edge` | 2 | `infra/ci/materialize-cloud-ci-generated-faces.sh`<br>`.github/workflows/oya-ci-required.yml materialization run blocks` | Keep edge singular; shell body can become Rust/controller, but ambient SCM read remains graph-edge ledger item. |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:42:| `infra/ci/materialize-cloud-ci-generated-faces.sh` | 28 | `irreducible_glue_ledger_entry` | scm_facts_emitter_edge |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:97:| `.github/workflows/oya-ci-required.yml:60` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| - name: Upload regenerated faces \| # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.` |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:99:| `.github/workflows/oya-ci-required.yml:115` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| - name: cargo test ${{ matrix.crate }}` |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:105:| `.github/workflows/oya-ci-required.yml:167` | `irreducible_glue_ledger_entry` | `infra/ci/materialize-cloud-ci-generated-faces.sh . \| cargo test --locked --no-fail-fast -p registry-drift -- --test-threads=1 \| # ── cloud-ci-firewall: the baseline ratchet (blocks only NEW debt) + the gate-registration \| #    meta-test (no in-tree gate may go` |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:107:| `.github/workflows/oya-ci-required.yml:192` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| - name: cargo test cloud-ci-firewall` |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:114:| `.github/workflows/oya-ci-required.yml:306` | `irreducible_glue_ledger_entry` | `run: infra/ci/materialize-cloud-ci-generated-faces.sh . \| # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication — \| # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests \| # run green, ful` |
origin/dev:evidence/multispectrum/cloud-intelligence-canary-status-salvage-20260612-1781239694.json:86:      "expected": "PASS with 0 failures after infra/ci/materialize-cloud-ci-generated-faces.sh"
origin/dev:evidence/multispectrum/g011-main-checkout-guard-20260610-1781108359.json:32:      "Generated JSON faces are materialized by infra/ci/materialize-cloud-ci-generated-faces.sh, not hand edited."
origin/dev:evidence/multispectrum/g011-main-checkout-guard-20260610-1781108359.json:82:    "infra/ci/materialize-cloud-ci-generated-faces.sh . -- final recovery PASS, 18225 tracked paths and 18209 accounting rows.",
origin/dev:evidence/multispectrum/g011-rust-test-wiring-generator-20260610-1781107105.json:65:      "No generated JSON face is hand-edited; generated faces are materialized only by infra/ci/materialize-cloud-ci-generated-faces.sh in the separate settle step.",
origin/dev:evidence/multispectrum/g011-target-parity-20260610-1781096256.json:33:      "No generated JSON face is hand-edited; generated faces are materialized only by infra/ci/materialize-cloud-ci-generated-faces.sh.",
origin/dev:evidence/multispectrum/g011-target-parity-20260610-1781096256.json:58:    "infra/ci/materialize-cloud-ci-generated-faces.sh . -- PASS: scm-facts 18197 tracked paths, accounting-registry 18181 rows",
origin/dev:evidence/multispectrum/g013-friction-accounting-20260610-1781147578.json:22:            "No generated JSON face is hand-edited; generated faces are materialized only by infra/ci/materialize-cloud-ci-generated-faces.sh.",
origin/dev:evidence/multispectrum/g013-friction-accounting-20260610-1781147578.json:47:        "infra/ci/materialize-cloud-ci-generated-faces.sh . -- settle generated faces (decision-crosswalk picks up ADR-0544)"
origin/dev:evidence/multispectrum/pr-633-generated-output-hygiene-20260609-1781033990.json:46:      "command": "bash -n infra/ci/install-buck2.sh infra/ci/materialize-cloud-ci-generated-faces.sh",
origin/dev:evidence/multispectrum/pr-633-generated-output-hygiene-20260609-1781033990.json:62:      "command": "infra/ci/materialize-cloud-ci-generated-faces.sh . && buck2 test generated-artifact-control-plane unit+gate",
 succeeded in 295ms:
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:135:      "path": "infra/ci/buck2-affected-gate.sh",
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json:686:      "snippet": "set -euo pipefail | chmod +x infra/ci/buck2-affected-gate.sh | BUCK2=buck2 infra/ci/buck2-affected-gate.sh \"origin/${{ github.base_ref || 'dev' }}\" | # ── APP-SHELL GENERATED CLIENT LANE. Generated TypeScript clients are intentionally ignored | #    in git, so"
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:40:| `infra/ci/buck2-affected-gate.sh` | 137 | `migrate_to_console_api` | Rust cloud-ci affected-set app / generated adapter; no bash logic in CI. |
origin/dev:evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md:116:| `.github/workflows/oya-ci-required.yml:326` | `migrate_to_console_api` | `set -euo pipefail \| chmod +x infra/ci/buck2-affected-gate.sh \| BUCK2=buck2 infra/ci/buck2-affected-gate.sh "origin/${{ github.base_ref \|\| 'dev' }}" \| # ── APP-SHELL GENERATED CLIENT LANE. Generated TypeScript clients are intentionally ignored \| #    in git, so` |
origin/dev:oya/ci-controller/crates/oya-ci-controller-k8s-adapter/src/lib.rs:505:            !command.contains("buck2-affected-gate.sh") && !command.contains("oya "),
origin/dev:specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json:39:      "infra/ci/buck2-affected-gate.sh only"
origin/dev:specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1-bad-buck2-affected-only-producer.json:27:    "command": "sh infra/ci/buck2-affected-gate.sh origin/dev origin/pr-42",
origin/dev:specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-mutable-producer.json:21:    "infra/ci/buck2-affected-gate.sh exits 0 before testing",
origin/dev:specs/phase0-ci-enforcement-baseline.json:57:        "gate_command": "sh infra/ci/buck2-affected-gate.sh origin/dev",
origin/dev:specs/repo-hygiene-automation.json:245:      "infra/ci/buck2-affected-gate.sh github-mirror/dev HEAD"
origin/dev:specs/repo-hygiene-automation.json:254:      "infra/ci/buck2-affected-gate.sh origin/dev HEAD"
 succeeded in 301ms:
f93a44f9bcdc236c95c4ef9c13aa8aefc7b9095a 2026-06-09 ci: productize generated output hygiene
exec
/bin/zsh -lc "git log origin/dev -n 12 --format='%H %ad %s' --date=short -- ci/facade/generated-artifact-freshness .github/workflows/oya-ci-required.yml | sed -n '1,80p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
11ce32e7c361f6241b3b27ed1b1564542a004dd1 2026-07-10 De-commit the firewall frozen-reference baseline (ADR-0616) — Bazel recompute-from-merge-base-source (#1251)
c78c50d6347e50e6e41db27eb3e8c419a700261a 2026-07-09 feat(ci): paved-road contract-slice-conformance gate (retire scripts/tests/*_check.py) (#1309)
eef42cac51044c4336a229923d52e32bf49892bb 2026-07-09 gate(control-plane): declare hand-curated ratchet baselines in the SSOT (close governance gap) (#1250)
339f8f63dec8a7feb2a995e6cc15a16028b2f728 2026-07-09 chore(ci): de-commit the reorg move-manifest bijection (ADR-0614, amends ADR-0563) (#1227)
1801128b16a0fe6318e84c1939d6e0937a677708 2026-07-09 feat(ci): capability-first keystone — cloud/cloud-ci → ci/facade + PathResolver + owned Cargo.lock lifecycle (#1216)
2929693daff4e93bc9db1bc0b62ef3b1a73cf6a4 2026-07-02 fix(ci): fetch live PR title/body in metadata preflight instead of stale event payload (#1196)
d197094a69ddc47bff120ffedfcd4cb417a85121 2026-07-02 plan(masterplan-v2): SSOT consolidation + fabric drive loop; four plan gates wired blocking into oya-ci-required (#1182)
4d97bd033144838e886039740618ea9f07941428 2026-07-02 fix(ci): expose affected-set long-step telemetry
18c781a9fcb2acb519ede9e92c1db21fd43afab9 2026-07-02 ci: add license-policy cloud-ci gate
cdc309531f2f761364fe2c544f86179d4c077363 2026-07-01 feat(cloud-ci): migrate cloud resource validators to Rust gate
6aaeb6878774c23b7aa72966f48532b822a5fd54 2026-07-01 feat(cell): add topology manifest contract gate (#1130)
d39c5c42f9e793b00a89449a872f73f788124486 2026-07-01 fix(ci): require rust-first infra automation hygiene
exec
/bin/zsh -lc 'git show origin/dev:registry/generated-artifact-control-plane.json | nl -ba | rg -n "artifact-inventory|scm-volatile|gate-baseline.merge-base|materialization_mode|not-tracked" -C 3' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
8-     8	  "artifacts": [
9-     9	    {
10-    10	      "artifact_id": "cloud-ci-accounting-registry-face",
11:    11	      "path": "ci/facade/artifact-inventory-registry/accounting-registry.generated.json",
12-    12	      "artifact_class": "main-materialized-aggregate",
13:    13	      "materialization_mode": "not-tracked-in-git",
14-    14	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
15-    15	      "owner_team": "cloud-ci-platform",
16-    16	      "source_inputs": [
--
24-    24	      "public_product_contract": "A repo-wide accounting projection must be controller-regenerated; humans and agents must not resolve conflicts by editing the JSON.",
25-    25	      "generator": {
26-    26	        "runner": "buck2",
27:    27	        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
28-    28	        "operation_id": "emit-accounting-face",
29-    29	        "parameters": {
30-    30	          "face": "registry"
--
39-    39	    },
40-    40	    {
41-    41	      "artifact_id": "cloud-ci-decision-crosswalk-face",
42:    42	      "path": "ci/facade/artifact-inventory-registry/decision-crosswalk.generated.json",
43-    43	      "artifact_class": "main-materialized-aggregate",
44:    44	      "materialization_mode": "not-tracked-in-git",
45-    45	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
46-    46	      "owner_team": "cloud-ci-platform",
47-    47	      "source_inputs": [
--
53-    53	      "public_product_contract": "Cross-artifact agreement data is a generated projection and must be regenerated rather than hand-merged.",
54-    54	      "generator": {
55-    55	        "runner": "buck2",
56:    56	        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
57-    57	        "operation_id": "emit-accounting-face",
58-    58	        "parameters": {
59-    59	          "face": "decision-crosswalk"
--
68-    68	    },
69-    69	    {
70-    70	      "artifact_id": "cloud-ci-enforcement-inventory-face",
71:    71	      "path": "ci/facade/artifact-inventory-registry/enforcement-inventory.generated.json",
72-    72	      "artifact_class": "main-materialized-aggregate",
73:    73	      "materialization_mode": "not-tracked-in-git",
74-    74	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
75-    75	      "owner_team": "cloud-ci-platform",
76-    76	      "source_inputs": [
--
82-    82	      "public_product_contract": "Enforcement inventory is generated CI product metadata and must have deterministic final-tree parity.",
83-    83	      "generator": {
84-    84	        "runner": "buck2",
85:    85	        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
86-    86	        "operation_id": "emit-accounting-face",
87-    87	        "parameters": {
88-    88	          "face": "enforcement-inventory"
--
97-    97	    },
98-    98	    {
99-    99	      "artifact_id": "cloud-ci-enforcement-liveness-face",
100:   100	      "path": "ci/facade/artifact-inventory-registry/enforcement-liveness.generated.json",
101-   101	      "artifact_class": "main-materialized-aggregate",
102:   102	      "materialization_mode": "not-tracked-in-git",
103-   103	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
104-   104	      "owner_team": "cloud-ci-platform",
105-   105	      "source_inputs": [
--
112-   112	      "public_product_contract": "Enforcement liveness is generated hook wiring metadata and must be regenerated rather than hand-merged.",
113-   113	      "generator": {
114-   114	        "runner": "buck2",
115:   115	        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
116-   116	        "operation_id": "emit-accounting-face",
117-   117	        "parameters": {
118-   118	          "face": "enforcement-liveness"
--
127-   127	    },
128-   128	    {
129-   129	      "artifact_id": "cloud-ci-gate-baseline-ratchet-face",
130:   130	      "path": "ci/facade/artifact-inventory-registry/gate-baseline.generated.json",
131-   131	      "artifact_class": "main-materialized-aggregate",
132:   132	      "materialization_mode": "not-tracked-in-git",
133-   133	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
134-   134	      "owner_team": "cloud-ci-platform",
135-   135	      "source_inputs": [
--
141-   141	      "public_product_contract": "The baseline ratchet is shrink-only generated policy; the frozen reference is regenerated from the merge-base source and validated by determinism + provenance, never a committed merge surface that could launder new debt.",
142-   142	      "generator": {
143-   143	        "runner": "buck2",
144:   144	        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
145-   145	        "operation_id": "emit-accounting-face",
146-   146	        "parameters": {
147-   147	          "face": "baseline"
--
156-   156	    },
157-   157	    {
158-   158	      "artifact_id": "cloud-ci-scm-facts-boundary-snapshot",
159:   159	      "path": "ci/facade/artifact-inventory-registry/scm-facts.generated.json",
160-   160	      "artifact_class": "scm-facts-boundary-snapshot",
161:   161	      "materialization_mode": "not-tracked-in-git",
162-   162	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
163-   163	      "owner_team": "cloud-ci-platform",
164-   164	      "source_inputs": [
--
183-   183	    },
184-   184	    {
185-   185	      "artifact_id": "cloud-ci-ttl-policy-face",
186:   186	      "path": "ci/facade/artifact-inventory-registry/ttl-policy.generated.json",
187-   187	      "artifact_class": "main-materialized-aggregate",
188:   188	      "materialization_mode": "not-tracked-in-git",
189-   189	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
190-   190	      "owner_team": "cloud-ci-platform",
191-   191	      "source_inputs": [
--
196-   196	      "public_product_contract": "TTL policy is data-derived generated CI policy and must be deterministic across repositories.",
197-   197	      "generator": {
198-   198	        "runner": "buck2",
199:   199	        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
200-   200	        "operation_id": "emit-accounting-face",
201-   201	        "parameters": {
202-   202	          "face": "ttl-policy"
--
213-   213	      "artifact_id": "machine-readable-board-sync-projection",
214-   214	      "path": "docs/machine-readable/board-sync.generated.json",
215-   215	      "artifact_class": "main-materialized-aggregate",
216:   216	      "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
217-   217	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
218-   218	      "owner_team": "planning-platform",
219-   219	      "source_inputs": [
--
241-   241	      "artifact_id": "machine-readable-masterplan-projection",
242-   242	      "path": "docs/machine-readable/masterplan.generated.json",
243-   243	      "artifact_class": "main-materialized-aggregate",
244:   244	      "materialization_mode": "not-tracked-in-git",
245-   245	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
246-   246	      "owner_team": "planning-platform",
247-   247	      "source_inputs": [
--
269-   269	      "artifact_id": "architecture-product-graph-dashboard",
270-   270	      "path": "docs/architecture/product-graph.html",
271-   271	      "artifact_class": "main-materialized-aggregate",
272:   272	      "materialization_mode": "not-tracked-in-git",
273-   273	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
274-   274	      "owner_team": "architecture-platform",
275-   275	      "source_inputs": [
--
297-   297	      "artifact_id": "cloud-ci-reorg-move-manifest",
298-   298	      "path": "specs/reorg/move-manifest.generated.json",
299-   299	      "artifact_class": "review-artifact",
300:   300	      "materialization_mode": "not-tracked-in-git",
301-   301	      "merge_policy": "never-manual-merge-regenerate-from-source-tree",
302-   302	      "owner_team": "cloud-ci-platform",
303-   303	      "source_inputs": [
--
324-   324	      "artifact_id": "ci-friction-accounting-hand-shrunk-baseline",
325-   325	      "path": "ci/facade/action-item-accounting/friction-accounting-baseline.json",
326-   326	      "artifact_class": "hand-curated-ratchet",
327:   327	      "materialization_mode": "hand-curated-committed",
328-   328	      "merge_policy": "normal-source-merge",
329-   329	      "owner_team": "cloud-ci-platform",
330-   330	      "source_inputs": [
331-   331	        "human review of the friction ledger legacy debt at ADR-0544 gate go-live",
332-   332	        "ci/facade/action-item-accounting/friction-accounting-policy.json"
333-   333	      ],
334:   334	      "final_tree_validation": "HAND-CURATED-COMMITTED (not producer-regenerated): the reviewed, hand-shrunk shrink-only ratchet reference for the friction-ledger legacy debt set frozen at ADR-0544 gate go-live. The live-repo gate test asserts the MEASURED legacy set equals these keys EXACTLY (set equality). It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree — a candidate recompute would erase the hand-shrunk burn-down and re-launder aged-out debt. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
335-   335	      "public_product_contract": "A human-authored shrink-only debt baseline is governed by the control plane: it cannot be silently de-committed or recomputed, only reviewed-shrunk."
336-   336	    },
337-   337	    {
338-   338	      "artifact_id": "ci-embedded-asset-hermeticity-hand-shrunk-baseline",
339-   339	      "path": "ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json",
340-   340	      "artifact_class": "hand-curated-ratchet",
341:   341	      "materialization_mode": "hand-curated-committed",
342-   342	      "merge_policy": "normal-source-merge",
343-   343	      "owner_team": "cloud-ci-platform",
344-   344	      "source_inputs": [
345-   345	        "human review of embedded-asset skip sites at ADR-0545 gate go-live",
346-   346	        "ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json"
347-   347	      ],
348:   348	      "final_tree_validation": "HAND-CURATED-COMMITTED (not producer-regenerated): the reviewed, hand-shrunk shrink-only ratchet reference for the embedded-asset skip_* debt set frozen at ADR-0545 gate go-live. The live-repo gate test asserts the MEASURED skip set equals these keys EXACTLY (set equality). It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree — a candidate recompute would erase the hand-shrunk burn-down. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
349-   349	      "public_product_contract": "A human-authored shrink-only skip-debt baseline is governed by the control plane: it cannot be silently de-committed or recomputed, only reviewed-shrunk."
350-   350	    },
351-   351	    {
352-   352	      "artifact_id": "ci-tier-dependency-acyclicity-frozen-baseline",
353-   353	      "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-baseline.json",
354-   354	      "artifact_class": "hand-curated-ratchet",
355:   355	      "materialization_mode": "hand-curated-committed",
356-   356	      "merge_policy": "normal-source-merge",
357-   357	      "owner_team": "cloud-ci-platform",
358-   358	      "source_inputs": [
359-   359	        "human-frozen tier-dependency violation set captured on the live tree at gate birth (ADR-0245/0280/0562)",
360-   360	        "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json"
361-   361	      ],
362:   362	      "final_tree_validation": "HAND-CURATED-COMMITTED, FROZEN-INTENT: the frozen known-debt baseline of pre-existing tier-dependency violations; the gate reports these advisory and blocks only on a NEW code|subject not in the set (subset/no-regression semantics). It has a local `--emit-baseline` producer, but that producer MUST NOT own the committed face: a candidate recompute would absorb (LAUNDER) new regressions into the baseline. It MUST stay a committed git blob, burn down only by reviewed removal, and MUST NOT be de-committed to a candidate recompute. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
363-   363	      "public_product_contract": "A frozen subset/no-regression debt baseline is governed by the control plane: its own emit-baseline producer is advisory only and can never recompute the committed reference over the candidate tree."
364-   364	    },
365-   365	    {
366-   366	      "artifact_id": "ci-port-placement-hand-frozen-baseline",
367-   367	      "path": "ci/facade/port-placement/port-placement-baseline.json",
368-   368	      "artifact_class": "hand-curated-ratchet",
369:   369	      "materialization_mode": "hand-curated-committed",
370-   370	      "merge_policy": "normal-source-merge",
371-   371	      "owner_team": "cloud-ci-platform",
372-   372	      "source_inputs": [
373-   373	        "human-frozen storage-port trait set captured on the dev tip at ADR-0570 gate birth",
374-   374	        "ci/facade/port-placement/port-placement-policy.json"
375-   375	      ],
376:   376	      "final_tree_validation": "HAND-CURATED-COMMITTED, hand-frozen (NOT a producer-emitted face): the frozen baseline of pre-existing storage-port traits defined in adapter crates; born-advisory + enforce-no-regression (subset, ratchet-down only), self-cleaning as ports relocate to their capability's core/ports crate. It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
377-   377	      "public_product_contract": "A hand-frozen no-regression port-placement baseline is governed by the control plane: it cannot be silently de-committed or recomputed, only reviewed-shrunk."
378-   378	    },
379-   379	    {
380-   380	      "artifact_id": "glossary-vocabulary-warning-allowlist-baseline",
381-   381	      "path": "registry/glossary-vocabulary/warning-baseline.tsv",
382-   382	      "artifact_class": "hand-curated-ratchet",
383:   383	      "materialization_mode": "hand-curated-committed",
384-   384	      "merge_policy": "normal-source-merge",
385-   385	      "owner_team": "cloud-ci-platform",
386-   386	      "source_inputs": [
387-   387	        "human/tool-curated known-warning allowlist for the glossary-vocabulary gate",
388-   388	        "registry/glossary-vocabulary/warning-sources.tsv"
389-   389	      ],
390:   390	      "final_tree_validation": "HAND-CURATED-COMMITTED: the hand/tool-curated known-warnings allowlist (<kind>\\t<token>) the glossary-vocabulary gate treats as accepted legacy warnings. It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree — a candidate recompute would launder new warnings into the allowlist. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
391-   391	      "public_product_contract": "A human/tool-curated known-warning allowlist is governed by the control plane: it cannot be silently de-committed or recomputed, only reviewed-shrunk."
392-   392	    }
393-   393	  ],
exec
/bin/zsh -lc 'for p in ci/facade/artifact-inventory-registry/accounting-registry.generated.json ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json ci/facade/baseline-ratchet/gate-baseline.merge-base.generated.json; do git cat-file -e origin/dev:$p 2>/dev/null && echo "tracked $p" || echo "untracked-at-origin/dev $p"; done' in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git grep -n -E \"gate-baseline\\.merge-base\\.generated|scm-volatile-facts\\.generated|accounting-registry\\.generated\" origin/dev -- 'ci/**/BUCK' 'ci/**/*.rs' | sed -n '1,260p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
untracked-at-origin/dev ci/facade/artifact-inventory-registry/accounting-registry.generated.json
untracked-at-origin/dev ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json
untracked-at-origin/dev ci/facade/baseline-ratchet/gate-baseline.merge-base.generated.json
 succeeded in 0ms:
origin/dev:ci/facade/artifact-accountability/src/lib.rs:3://! The total-accounting gate that owns + produces `accounting-registry.generated.json`
origin/dev:ci/facade/artifact-inventory-registry/src/lib.rs:3://! Generates `accounting-registry.generated.json` — one record per `git ls-files`
origin/dev:ci/facade/artifact-inventory-registry/src/main.rs:432:        &out_dir.join("accounting-registry.generated.json"),
origin/dev:ci/facade/baseline-ratchet/src/lib.rs:20://! git boundary) into `gate-baseline.merge-base.generated.json`; this crate only parses it
origin/dev:ci/facade/baseline-ratchet/src/lib.rs:83:/// materializes (`gate-baseline.merge-base.generated.json`). Bumped only on a breaking
origin/dev:ci/facade/baseline-ratchet/src/lib.rs:118:    "ci/facade/baseline-ratchet/gate-baseline.merge-base.generated.json";
origin/dev:ci/facade/crate-registration/src/lib.rs:116:    ("accounting-registry.generated.json", "registry"),
origin/dev:ci/facade/crate-registration/src/lib.rs:663:    /// (`accounting-registry.generated.json`) and the live-tree membership scan are read directly by
origin/dev:ci/facade/crate-registration/src/tests.rs:698:            .contains(&"accounting-registry.generated.json".to_owned()),
origin/dev:ci/facade/crate-registration/src/tests.rs:1012:        "ci/facade/artifact-inventory-registry/accounting-registry.generated.json",
origin/dev:ci/facade/generated-artifact-freshness/src/lib.rs:43:    "ci/facade/artifact-inventory-registry/accounting-registry.generated.json",
origin/dev:ci/facade/generated-artifact-freshness/src/lib.rs:65:    ("accounting-registry.generated.json", "registry"),
origin/dev:ci/facade/generated-artifact-policy/src/lib.rs:2669:        let mut pure_view = artifact("pure-view-face", "out/accounting-registry.generated.json");
origin/dev:ci/facade/inventory-registry-drift/src/lib.rs:5://! byte-diff against the committed `accounting-registry.generated.json`. A hand-edit to
origin/dev:ci/facade/inventory-registry-drift/tests/registry_drift.rs:48:    ("accounting-registry.generated.json", "registry"),
origin/dev:ci/facade/scm-facts-snapshot/src/main.rs:20://! `scm-volatile-facts.generated.json` beside this crate — the same materialized-snapshot
origin/dev:ci/facade/scm-facts-snapshot/src/main.rs:1747:            "out_path": "ci/facade/baseline-ratchet/gate-baseline.merge-base.generated.json"
origin/dev:ci/facade/scm-facts-snapshot/src/main.rs:1759:                .ends_with("gate-baseline.merge-base.generated.json")
origin/dev:ci/facade/stale-artifact-detection/tests/staleness_reaper.rs:262:        "ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json",
origin/dev:ci/ports/path-resolver/src/lib.rs:41:    /// The untracked, gitignored `scm-volatile-facts.generated.json` snapshot (candidate WRITE
origin/dev:ci/ports/path-resolver/src/lib.rs:70:            "ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json"
exec
/bin/zsh -lc "git show origin/dev:specs/cache-warmth-policy.json | nl -ba | sed -n '1,260p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	{
     2	  "_comment": "Build cache-warmth classification (DATA, not code) — ADR-0556, founder directive 2026-06-12: 'some things should be cold. some things can be warm. make that distinction well.' Every build class maps to {warmth, cache_read, cache_write, reason}. Consumers (interim CI quick-wins, the W3 NativeLink CAS vertical, the future cache-policy-conformance gate) read this policy rather than re-deciding warmth per change. R0 pack-shape: ALL repo-specifics live here; an adopting repo edits the classes, never an engine. Surface model (ADR-0556 D4): this file is declarative policy consumed by services/controllers — never a CLI an operator runs; cache-write authorization is enforced at the CAS service boundary (keyed authn), never by client discipline.",
     3	  "policy_id": "cache-warmth-policy",
     4	  "schema_version": "1.0.0",
     5	  "adr": "ADR-0556",
     6	  "schema": {
     7	    "build_class": {
     8	      "warmth": "cold | warm",
     9	      "cache_read": "bool — may this class read the shared cache",
    10	      "cache_write": "bool — may this class write to the shared cache",
    11	      "reason": "string — WHY, citing the governing invariant"
    12	    }
    13	  },
    14	  "trust_invariant": {
    15	    "statement": "A build class MAY run warm IFF (a) it is warm-eligible in this policy AND (b) the most recent scheduled cold integrity-canary run is GREEN (cold output digests byte-identical to the warm CAS digests for the same action keys). Warm-by-default is sound ONLY BECAUSE the cold canary continuously proves warm == cold; the canary is the trust anchor that licenses warmth.",
    16	    "canary_build_class": "integrity-canary",
    17	    "canary_red_response": [
    18	      "per the IFF: a RED canary SUSPENDS ALL WARM READS FLEET-WIDE pending the next GREEN run — the items below are the durable remediation, never a license to keep serving warm on non-divergent keys while RED stands",
    19	      "RED is a blocking hermeticity/non-determinism defect (ADR-0525 D4 violation), never tolerable noise; the warm cache is structurally suspect from that moment",
    20	      "divergent action keys are evicted/quarantined from the CAS immediately (reconciler/API action by the canary controller, not a hand operation)",
    21	      "a friction-ledger row opens mechanically (ADR-0544 closed loop); if root cause is not established within one canary period, the warm-eligible classes covering the divergent cone degrade to cold via this DATA (shrink-of-warmth needs no door)",
    22	      "NOT permitted: serving ANY warm hit while RED stands (the IFF is unsatisfied), resuming on divergent keys after GREEN returns without the eviction above, or widening the comparison tolerance — cold == warm is byte-equality, the registry-drift bar"
    23	    ],
    24	    "deployment_precondition": "the CAS vertical MUST ship the integrity-canary in the same change that enables fleet-wide warm reads — no canary, no warm"
    25	  },
    26	  "trust_boundary": {
    27	    "trusted_author": "a same-repo branch pushed by an authorized writer and admitted into the governance pipeline (required_workflow lanes); holds the CAS write key",
    28	    "untrusted_author": "fork PRs and any context without the CAS write key (GitHub fork PRs receive read-only tokens and no secrets — a natural seam, but the binding enforcement is the CAS service boundary authn/authz, never runner configuration)"
    29	  },
    30	  "default_for_unlisted_classes": {
    31	    "warmth": "cold",
    32	    "cache_read": false,
    33	    "cache_write": false,
    34	    "reason": "fail-closed default the trust invariant already implies, made mechanical for the conformance gate: a build class not listed in build_classes has no warm license — warmth is granted by reviewed classification, never by omission"
    35	  },
    36	  "build_classes": {
    37	    "release-production-image": {
    38	      "warmth": "cold",
    39	      "cache_read": false,
    40	      "cache_write": false,
    41	      "reason": "reproducibility + SBOM/provenance integrity (ADR-0039, ADR-0181): the shipped artifact must derive from exactly its sources via a from-source build; a cache hit substitutes bytes whose derivation was attested elsewhere or nowhere. No write-back: release builds run with the most-privileged signing identity — writing from that context maximizes blast radius. The rust-purity sole cargo exception (cargo --release + lto fat + locked) lives on this path, outside the buck2 graph."
    42	    },
    43	    "integrity-canary": {
    44	      "warmth": "cold",
    45	      "cache_read": false,
    46	      "cache_write": false,
    47	      "reason": "the trust anchor that licenses warm-by-default (ADR-0556 D2): a scheduled from-empty build of the pinned graph whose output digests are byte-compared against the warm CAS digests for the same action keys; any cache participation makes the proof circular. cold != warm = hermeticity/non-determinism bug, fail-closed."
    48	    },
    49	    "untrusted-author-presubmit": {
    50	      "warmth": "cold",
    51	      "cache_read": false,
    52	      "cache_write": false,
    53	      "reason": "anti-poisoning (Bazel/Google RBE security model): an untrusted PR controls action inputs; with write access it seeds poisoned outputs under action keys trusted builds will later hit. Write prohibition is one-way; default is full isolation (defense in depth, no cache-probing side channel). A read-only relaxation (cache_read true) is a reviewed two-way policy edit — reads cannot inject. Enforced at the CAS service boundary: untrusted contexts hold no key."
    54	    },
    55	    "provenance-attestation": {
    56	      "warmth": "cold",
    57	      "cache_read": false,
    58	      "cache_write": false,
    59	      "reason": "SLSA: provenance must describe the build that actually happened; serving cached outputs while attesting build steps fabricates provenance, and reproducible-build verification requires re-derivation."
    60	    },
    61	    "presubmit-trusted-dep-closure": {
    62	      "warmth": "warm",
    63	      "cache_read": true,
    64	      "cache_write": true,
    65	      "reason": "the third-party crate closure (reindeer-vendored, lockfile-pinned) is identical across every PR sharing a lockfile; rebuilding it per leg and per run is pure waste. Content-addressed: a hit is bit-identical to cold (licensed by the trust invariant)."
    66	    },
    67	    "presubmit-trusted-affected-cone": {
    68	      "warmth": "warm",
    69	      "cache_read": true,
    70	      "cache_write": true,
    71	      "reason": "the affected-target cone (ADR-0525 D3 uquery owner->rdeps, binding via the ADR-0554 affected-set lane): only genuinely changed actions miss; the unchanged cone is a hit — ADR-0515 D4 (wall-clock tracks the change, not the repo) made real."
    72	    },
    73	    "dev-agentic-iteration": {
    74	      "warmth": "warm",
    75	      "cache_read": true,
    76	      "cache_write": true,
    77	      "reason": "agent-lane and dev-loop builds in throwaway worktrees see 0% hits today (FRIC-1781070457-buck2-no-shared-cache); a warm shared cache makes the agent fleet's wall-clock track the size of each change."
    78	    },
    79	    "gate-fleet-shared-graph": {
    80	      "warmth": "warm",
    81	      "cache_read": true,
    82	      "cache_write": true,
    83	      "reason": "the gate fleet's shared dependency hub — the accounting-registry producer and the common workspace graph rebuilt ~13x per oya-ci-required run across legs (faces re-materialized in every matrix leg). One build, many consumers. Same-run artifact reuse (QW-1) is this class without a CAS; deliberate exception: registry-drift keeps its own in-job rematerialization — detectors never consume the thing they attest."
    84	    },
    85	    "postmerge-dev-trunk": {
    86	      "warmth": "warm",
    87	      "cache_read": true,
    88	      "cache_write": true,
    89	      "reason": "the canonical trusted populator (Bazel/Google pattern: post-merge CI fills the cache, presubmits hit it); trunk content passed oya-ci-required and is by definition admitted content."
    90	    }
    91	  },
    92	  "consumers": [
    93	    "interim CI quick-wins (ADR-0556 D5: QW-1 artifact reuse, QW-2 cargo-matrix collapse, QW-3 stable-key buck-out restore, QW-4 toolchain caching) — warm-safe list only; no cache step may be added to a cold-required path",
    94	    "NativeLink CAS W3 vertical (ADR-0556 D3: buck2_re_client cache-only; allow_cache_uploads per-class from this policy; MUST ship the integrity-canary in the same change)",
    95	    "future cache-policy-conformance gate (the enforcement successor, ADR-0555 D4 gate-test-plus-policy-DATA: asserts the live CI cache configuration matches this policy)"
    96	  ],
    97	  "product_contract": {
    98	    "born_pack_shaped": "ADR-0548 paved-road rule: the classes, the trust boundary, and the invariant prose are DATA here; consumers hardcode no class list. An adopting repo edits build_classes for its own pipeline shapes.",
    99	    "door_asymmetry": "the cold-required floor is one-way (moving a cold-required class to warm requires superseding ADR-0556); warm-eligible membership is two-way DATA (degrading warm to cold is always allowed; growing warmth is a reviewed change)."
   100	  }
   101	}
exec
/bin/zsh -lc "git ls-tree -r --name-only origin/dev | rg 'ADR-0560|cache-warmth-policy|build-cache|nativelink|NativeLink' | sed -n '1,160p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
ci/facade/build-cache-policy/BUCK
ci/facade/build-cache-policy/Cargo.toml
ci/facade/build-cache-policy/src/canary-policy.json
ci/facade/build-cache-policy/src/lib.rs
ci/facade/build-cache-policy/src/main.rs
ci/facade/build-cache-policy/tests/cache_conformance.rs
docs/decisions/ADR-0556-build-cache-warmth-classification.md
docs/decisions/ADR-0560-nativelink-cas-slice1-deployable-cache-cold-canary.md
docs/decisions/ADR-0612-buck2-remote-execution-phase-nativelink-scheduler-worker.md
docs/ideas/nativelink-remote-cache-first.md
infra/nativelink/OWNERS
infra/nativelink/nativelink-cas.k8s.yaml
specs/cache-warmth-policy.json
exec
/bin/zsh -lc "git grep -n -i -E 'NativeLink|cold.*warm|warm.*cold|content.address|remote cache|cache poisoning|read.only' origin/dev -- specs docs/decisions ci/facade/build-cache-policy | sed -n '1,300p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 287ms:
roles, and custom data classes. Phase 3: JIT access, approval workflows, and API-key tiering. Phase 4: advanced policy simulation, dry-run impact analysis, and per-tenant evidence export.
origin/dev:docs/decisions/ADR-0222-saga-compensation-portfolio-policy.md:99:action is provably side-effect-free (read-only, query-only). Any step
origin/dev:docs/decisions/ADR-0222-saga-compensation-portfolio-policy.md:108:`oya-saga-coordinator-token` issued by the engine. Read-only GETs are
origin/dev:docs/decisions/ADR-0241-dr-business-continuity-portfolio-policy.md:218:- Cold standby (T4) consumes ~5% of primary substrate. Warm standby
origin/dev:docs/decisions/ADR-0243-cedar-as-universal-gate.md:329:   archived (read-only, regulator-retrieval-only).
origin/dev:docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md:1211:- The shadow is read-only by default; write capability requires an
origin/dev:docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md:1496:3. **Read-only flip.** Source cell tenant traffic is paused (Cedar
origin/dev:docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md:1516:| `enterprise` | < 60 seconds (read-only flip → cutover) |
origin/dev:docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md:2280:- The shadow remains projected (read-only, no longer executable);
origin/dev:docs/decisions/ADR-0245-substrate-vs-product-layering.md:950:- `analytics`: 99.5% availability acceptable (read-only product;
origin/dev:docs/decisions/ADR-0246-policy-engine-substrate-promotion.md:711:- Image: `distroless-rust` per ADR-0146; non-root; read-only root FS.
origin/dev:docs/decisions/ADR-0246-policy-engine-substrate-promotion.md:2062:v1 fragment moves to read-only archive in SeaweedFS cold tier. The
origin/dev:docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md:842:    read-only traffic for cross-region reads + DR pre-warming).
origin/dev:docs/decisions/ADR-0252-time-coordination-distributed-consistency.md:661:| Read-only operations | Not required | N/A |
origin/dev:docs/decisions/ADR-0253-network-topology-edge-service-mesh.md:1120:   (warm V8 isolate, cached bundle) to ~8ms p99 (cold isolate, first
origin/dev:docs/decisions/ADR-0254-deployment-model-spectrum.md:629:  compliant container images, content-addressed by digest, cosign-
origin/dev:docs/decisions/ADR-0254-deployment-model-spectrum.md:699:   §D-5) from upstream release tags. Bundles are content-addressed
origin/dev:docs/decisions/ADR-0263-observability-emission-contract.md:620:  retention rules: hot 7 days, warm 30 days, cold 1 year unless
origin/dev:docs/decisions/ADR-0263-observability-emission-contract.md:822:- Read-only queries.
origin/dev:docs/decisions/ADR-0263-observability-emission-contract.md:1578:| D-10 (cross-cell aggregation) | "Cell-Replicated Read-Only Aggregator" | Amazon shape cellular architecture (per ADR-0248); Google Borg cross-cell aggregation | "Single global cluster" — blast radius cross-cell |
origin/dev:docs/decisions/ADR-0272-cookie-consent-per-purpose-analytics-opt-in.md:1371:- Git-style content-addressed append-only chains.
origin/dev:docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md:602:The dashboard is read-only for tenant admins. Apply /
origin/dev:docs/decisions/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md:875:warm-up history. Cold domains that send at full volume
origin/dev:docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md:664:content-addressed chain. The export includes:
origin/dev:docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md:825:  read-only historical artifacts attached to the imported
origin/dev:docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md:1719:audit chain is attached to the peer system as a read-only
origin/dev:docs/decisions/ADR-0276-backup-portability-format-gdpr-article-20.md:1789:7. Audit chain is imported as a read-only historical artifact.
origin/dev:docs/decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md:138:`object-store-kernel`, the DB trait, the gate contract, the content-address). The acyclicity invariant
origin/dev:docs/decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md:1166:1. **Local fallback caches** — read-only fallback that survives
origin/dev:docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md:476:  with read-only access to a "parent-pending" stub UI until the
origin/dev:docs/decisions/ADR-0294-cedar-fragment-soak-anomaly-rollback.md:697:is too short to warm the baseline, the cold-baseline carve-out per
origin/dev:docs/decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md:967:- All 11 ceremony participants (with read-only access).
origin/dev:docs/decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md:968:- All council members (with read-only access).
origin/dev:docs/decisions/ADR-0295-bootstrap-ci-spiffe-kill-switch.md:969:- ops-sre-reliability on-call (with read-only access).
origin/dev:docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md:681:- **Cold-vs-warm path latency separation.** First request from new
origin/dev:docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md:1039:  Cedar forbids high-risk operations; allows read-only legitimate
origin/dev:docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md:2193:// read-only routes; subject to their tier's rate cap.
origin/dev:docs/decisions/ADR-0298-emergency-services-bypass-life-safety.md:803:  Warm-cache hit rate ≥99.9% under normal operation; cold-cache
origin/dev:docs/decisions/ADR-0298-emergency-services-bypass-life-safety.md:816:  verification (≤30ms p99 warm + ≤120ms p99 cold) + Cedar evaluation
origin/dev:docs/decisions/ADR-0298-emergency-services-bypass-life-safety.md:835:- **Cold-vs-warm path latency separation.** Warm: ≤30ms p99. Cold:
origin/dev:docs/decisions/ADR-0299-account-recovery-resilience.md:703:- **Cold-vs-warm.** Warm: cached recovery state + sidecar key
origin/dev:docs/decisions/ADR-0299-account-recovery-resilience.md:809:warm; ≤2s p99 cold (first-time challenge issuance).
origin/dev:docs/decisions/ADR-0300-whistleblower-press-freedom-anonymity.md:847:- **Cold-vs-warm.** Warm: per-session sealed-sender key cached.
origin/dev:docs/decisions/ADR-0301-survivor-safety-domestic-abuse-mode.md:771:- **Cold-vs-warm.** Warm: per-account DEK + active-session
origin/dev:docs/decisions/ADR-0305-delegated-agent-authority-chain.md:374:  a read-only delegation, the delegate cannot write.
origin/dev:docs/decisions/ADR-0307-detection-substrate-streaming-batch.md:819:+ Feast online tier warm). Cold-start is rare (planned restarts
origin/dev:docs/decisions/ADR-0307-detection-substrate-streaming-batch.md:858:Cold-vs-warm path latency: cold (first scoring per user) ≈ 50ms
origin/dev:docs/decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md:1045:Cold-vs-warm path latency: cold (first inference on new model
origin/dev:docs/decisions/ADR-0309-detection-fairness-audit-civil-rights.md:1045:Cold-vs-warm path latency: cold (first variant lookup after deploy)
origin/dev:docs/decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md:783:- Cold-vs-warm latency split: warm path 50-200 µs; cold path (cache
origin/dev:docs/decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md:1305:   to read-only (per the work-tenant's retention rules).
origin/dev:docs/decisions/ADR-0312-court-warrant-scoped-piercing.md:440:   - `actions_permitted`: `{ "ReadInScope" }` (read only).
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:419:(read-only-financial / read-only-operational / read-write-board-
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:420:decisions / audit-only / cross-jurisdiction-read-only / joint-venture-
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:756:cache warm). Acceptable per ADR-0044 cold-start SLO tier.
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:777:- **Cold-vs-warm path latency separation.** Warm path: P99 ≤ 1 ms
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1026:        --   "tiers": ["read-only-financial", "read-only-operational"],
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1277:              "read-only-financial",
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1278:              "read-only-operational",
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1281:              "cross-jurisdiction-read-only",
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1376:#### §D-3.1 `read-only-financial`
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1382:statements). Read-only; cannot modify child financial records. Cross-
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1385:#### §D-3.2 `read-only-operational`
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1390:`ReadWorkflowEngineMetadata`. Read-only; aggregated metrics;
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1421:#### §D-3.5 `cross-jurisdiction-read-only`
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1424:parent with EU-incorporated subsidiaries). The scope is read-only
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1426:cell via a read-only proxy but is NEVER exfiltrated across the
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1441:- Parent A: `read-only-financial` only.
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1442:- Parent B: `read-only-operational` only.
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1719:     20-50%); the grant is amended to `read-only-financial` +
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1748:   - (A, JV) grant with scope `{"tiers": ["read-only-financial",
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1751:   - (B, JV) grant with scope `{"tiers": ["read-only-operational",
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1820:   `read-only-financial` + `audit-only` + `read-write-board-
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1824:   division) — scope: `read-only-operational` +
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1927:- (bigbank-holdings, bigbank-retail-bank): scope `read-only-financial`
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:1929:- (bigbank-holdings, bigbank-investment-bank): scope `read-only-
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:2896:- **`read-only-financial`** — `<access-class>-<domain>` kebab-case
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:2897:  per the per-tier scope-naming convention; `read-only` is the
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:2899:- **`read-only-operational`** — same shape; domain is `operational`.
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:2905:- **`cross-jurisdiction-read-only`** — `<modifier>-<access-class>`
origin/dev:docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md:2907:  `read-only`.
origin/dev:docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md:471:Migration starts with connect adapter registration, source-system inventory, SAP table/export classification, data-class mapping, tenant authority confirmation, and read-only dry run. Bulk import produces per-table migration plans with row counts, rejected rows, transform digests, and rollback envelopes.
origin/dev:docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md:251:- A capability tier MUST include at least one workflow template or explicitly declare why it is read-only.
origin/dev:docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md:608:D8.state.sunset. Tier cannot execute user actions; only read-only export and audit queries remain.
origin/dev:docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md:2022:R.027. Lifecycle stage read-only-period: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1066:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1076:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1086:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1096:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1106:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1116:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0318-collar-color-workspace-universality.md:1126:D-9 anti-pattern: accessible view that only supports read-only work.
origin/dev:docs/decisions/ADR-0320-apprentice-intern-resident-fellow-transient-identity.md:565:F.10 Roll out in read-only mirror mode, then policy-shadow mode, then enforcement mode per service.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:178:  7. **Sunset**: Salesforce org placed in read-only (User Profile → Read Only); after 90-day delta-capture window, org closure with retained-org-id ledger in `oya-governance-evidence-store`.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:197:  - **Verification phase**: Parallel run — 14-day shadow window with dual-write to Salesforce + Oyatie; delta detection — daily reconciliation report per object (count, sum-of-Amount, opp-by-stage); cut-over gate — |delta| ≤ 0.5% on Amount and ≤ 0% on count; post-cutover — 90-day read-only Salesforce retention for delta-capture; sunset evidence — retained-org-id ledger + final Bulk API export sealed in `oya-governance-evidence-store`.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:308:  9. **Sunset**: 90-day read-only org window for delta-capture; archived to `oya-governance-evidence-store`.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:326:  - **Verification phase**: Parallel run — 14-day shadow window with dual-write; delta detection — daily case-count + SLA-violation-count reconciliation; cut-over gate — |delta| ≤ 0.3% on counts; sunset — 90-day read-only retention with delta-capture.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:435:  - **Send-IP reputation cliff**: warm IPs at SFMC, cold IPs at Oyatie; mitigation — parallel-warmup window + reputation-state import + per-domain DKIM dual-signing.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:704:- **Capability-tier mapping per ADR-0316**: Viewer → **Bronze** (read-only dashboards); Explorer → **Silver** (interactive filter + web-edit); Creator → **Gold** (full authoring + Prep); Tableau+ + Tableau AI + Pulse → **Platinum** (AI-driven natural-language analytics + mobile metrics + Einstein-class).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:929:  10. **Cutover**: per-channel cut-over with member-notification; Slack channel set to read-only; new posts route to Oyatie.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:934:  - **Cross-tenant federation trust-break** (sovereign child leaves federation): per ADR-0313 grant revocation; cross-tenant channel becomes read-only then archived.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:943:  - **Bot token revocation race**: bot uninstall mid-cutover; mitigation — dual-token window with read-only fallback.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1743:  - **Federation trust-break**: per-federation link revocation; cross-tenant content becomes read-only then archived per ADR-0313.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:1789:  - Trailhead Profile API (read-only): `GET https://trailblazer.me/id/<username>` (HTML scrape + structured JSON-LD); unofficial GraphQL endpoint `POST https://profile.api.trailhead.com/graphql` (Query: GetTrailheadProfileData + GetTrailheadProfileBadges + GetTrailheadProfileCertifications + GetTrailheadProfileSkills + GetTrailheadProfileSuperbadges).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3192:  - "Executive Dashboard" — read-only mobile-friendly KPI rollup.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:3487:  - "Auditor Workspace" — read-only auditor view with journal-line drill + control-evidence + change-history.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:4979:  - "Smart-Mirroring" (Data Center) — read-only mirrors in remote regions with sync-status.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5039:  2. **Land**: per-tenant + per-workspace partitioned ingest; per-repo Git mirror created on Oyatie Git substrate with `oya git` per `feedback_oya_git_canonical_2026_05_18`; per-LFS-object content-addressable storage.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5061:  - **LFS-pointer migration** (Git-LFS attachment): per-object content-addressable migration + verifier + dedupe across repos.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5264:  - Atlassian Analytics: per-data-lake SQL-query endpoint (read-only, cross-product).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:5992:  - `Entity_Form` (Form_ID, Site_Reference, Bound_Dataverse_Table, Bound_Form_Reference (Dataverse main/quick-create form), Mode {Insert / Edit / Read-Only}, Pre_Populate_Fields[], Auto_Save: bool, Captcha_Required: bool, Attach_File_Storage_Location, Confirmation_Page).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7051:- UX shell adaptation: white-collar back-office (data-governor + privacy-officer + compliance-officer + eDiscovery-attorney + Insider-Risk-investigator + Customer-Lockbox-approver); desktop-primary with asset-graph + lineage-trace + policy-canvas + case-evidence-binder; per-classification banner; auditor read-only with audit-trail; per-tenant + per-region accessibility.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:7398:- UX shell adaptation: IT-admin desktop-primary (admin-console + risky-users dashboard); end-user mobile-primary "My Apps" + Passkey-mgmt + Access-Package self-service; HR-admin lifecycle-workflow trigger; manager mobile for access-review approvals; auditor read-only with sign-in-logs + audit-logs export.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8065:  - **IP-Pool reputation cliff on cutover** (new infrastructure cold IPs throttled by ISPs): per-IP-Pool reputation-warmup gradual + per-ISP handshake + dual-pool window.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8252:- UX shell adaptation: white-collar back-office (digital-analyst + product-analyst + marketer + analytics-engineer + tag-engineer); desktop-primary Analysis Workspace with drag-drop panel + project-share + segment-builder + calculated-metric-builder; mobile read-only dashboard; per-project share-with-role with audit; per-RSID admin console.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8425:- UX shell adaptation: white-collar back-office (data-architect + marketer + privacy-officer + CDP-engineer + activation-mgr); desktop-primary segment-builder (PQL) + profile-viewer + identity-graph visualizer + per-purpose consent-canvas + governance-label dashboard + destination-mapping editor + Privacy-Service queue; mobile read-only profile + segment + audience views.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8572:  - **Channel-Config IP-Pool reputation cliff on cutover** (cold IPs throttled): per-pool reputation-warmup gradual + per-ISP handshake.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8589:- UX shell adaptation: white-collar back-office (journey-strategist + decision-strategist + marketer + content-designer + B2B-strategist); desktop-primary canvas with event + condition + action nodes + per-offer eligibility + ranking visualization + per-decisioning explainability + per-EU-AI-Act evidence-bundle viewer + Action-Center queue + B2B Account-Journey + Engagement-Score dashboard; mobile read-only journey-monitor.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:8607:  - `Form_Field` (Field_ID, Document-Reference, Page, X-Y-Width-Height, Type {Signature / Initial / Date / Title / Company / Email / Text-Field / Drop-Down / Radio-Button / Checkbox / Image-Stamp / Hyperlink / File-Attachment / Calculated-Field / Read-Only-Form-Field / Notification-Field}, Assigned-Signer-Order, Required: bool, Default-Value, Validation-Pattern, Conditional-Visibility, Tooltip).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9111:- UX shell adaptation: white-collar back-office (B2B marketer + content-creator + marketing-ops + SEO-strategist + campaign-mgr); desktop-primary workflow + content-editor + smart-content + AB-test + smart-list + lead-scoring + subscription-preferences; per-asset preview + mobile-preview; mobile read-only campaign-monitor; per-team partition view.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9649:- UX shell adaptation: white-collar back-office (web-team + content-creator + theme-designer + developer + UX-designer); desktop-primary page-editor with drag-drop module + Smart-Content + AB-Test + Content-Staging + Multi-Language + Membership-Config + HubDB-Editor + Serverless-Function editor; mobile-preview + read-only review; per-team partition view.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9686:**UX shell adaptation.** White-collar (RevOps + data-engineer + analytics-engineer); desktop-primary sync-config builder + field-map UI + sync-history grid; per-sync error-detail drawer; per-deduplication review-queue with side-by-side comparison; embedded Monaco IDE for custom-code (with HubSpot autocomplete replaced by oyatie SDK autocomplete). Mobile read-only sync-status overview for on-call RevOps.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9842:**Vendor data model — 18 top objects.** `call` (id, direction, status ∈ {created, ringing, in_progress, completed, missed, voicemail}, duration, recording-url, from, to, agent, group, transferred_from); `queue` (per-skills-route container with overflow + max-wait); `ivr-tree` (root-node + branch-nodes; each node has prompt + key-press-action + speech-recognition-action); `ivr-node` (greeting / menu / capture-digits / route-to-queue / route-to-agent / route-to-voicemail / business-hours-switch / time-of-day-switch); `agent-session` (per-shift, with status ∈ {available, away, transfers_only, offline}, skills-bound); `recording` (audio + url + duration + consent-state); `transcript` (text + word-level-timing + speaker-diarization + confidence); `DID` (E.164 phone number with country + region + capability-set {voice, sms, mms}); `SIP-trunk` (BYO carrier with TLS + SRTP); `voicemail` (audio + transcript + email-notification + ticket-link); `callback-request` (queued callback with position-preservation + retry-policy); `business-hours-config` (per-line, per-tz, with holiday-overrides); `skill` (per-agent skill tag for routing); `transfer-target` (warm vs cold transfer destination); `whisper-message` (supervisor-to-agent private side-channel); `barge-event` (supervisor joining call); `outbound-campaign` (preview + progressive + predictive dialer) — Advanced edition only; `wallboard` (real-time supervisor display).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9857:// Verbs: accept-call, initiate-outbound-call, transfer-call-warm, transfer-call-cold, hold-call, resume-call,
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:9995:**UX shell adaptation.** White-collar (data-engineer + analyst + analytics-engineer + ML-engineer); desktop-primary Snowsight Worksheet equivalent with multi-tab + query-editor + result-grid + plan-profile-viewer; admin desktop-primary warehouse-monitor + cost-dashboard + role-grant-explorer; per-Marketplace listing browser white-collar + per-Native-App installer; executive read-only data-app via Streamlit-equivalent.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10058:**UX shell adaptation.** White-collar (data-engineer + ML-engineer + data-scientist + analytics-engineer); desktop-primary workspace with notebook IDE + SQL editor + ML-experiment-tracker; per-Unity-Catalog hierarchical explorer with lineage-tab; admin desktop-primary cluster-manager + cost-dashboard + audit-log-viewer; executive read-only AI/BI Dashboard; per-Mosaic-AI Agent Studio for ML-engineer; per-Genie text-to-SQL workspace for analyst.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10122:**UX shell adaptation.** White-collar (data-engineer + analyst + ML-engineer); desktop-primary BigQuery Studio with query-editor + Dataform + notebook + ML in unified workspace; admin desktop-primary slot-reservation manager + BI Engine reservation + Policy Tag taxonomy editor; executive read-only data-canvas via Gemini; per-Analytics-Hub listing browser white-collar; mobile read-only query-result snapshot for on-call.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10144:- remote-function Cloud-Run cold-start timeout: per-function min-instance + warm-pool.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10186:**UX shell adaptation.** White-collar (data-engineer + analyst); desktop-primary Query Editor v2 with notebook + visualizer + saved-queries; admin desktop-primary cluster/workgroup manager + WLM queue editor + snapshot manager + datashare manager; per-Spectrum external-schema config; per-RLS-policy editor; per-Redshift-ML-model browser; mobile read-only query-result snapshot.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10247:**UX shell adaptation.** White-collar (data-engineer + analytics-engineer + analyst); desktop-primary dbt-Cloud IDE equivalent (browser-based SQL+Jinja editor with Git + lineage graph + docs); per-environment manager + per-job scheduler; per-Semantic-Layer metric browser; per-Explorer with column-level lineage + impact-analysis; mobile read-only run-status; admin desktop-primary repository config + credential vault.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10310:**UX shell adaptation.** White-collar (data-engineer); desktop-primary connector-list + per-connector-sync-history + schema-preview + logs-viewer; admin desktop-primary destination-manager + private-link-config + agent-install + MAR-usage-dashboard; per-Quickstart-Data-Model browser; per-Webhook + per-Notification config; mobile read-only sync-status overview for on-call data-engineer.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10328:- Function-Connector cold-start timeout > sync-window: per-function min-instance + warm-pool.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10372:**UX shell adaptation.** White-collar (data-engineer + citizen-data-engineer for Connector Builder); desktop-primary workspace-home + source/destination catalog + connection-builder + sync-history; admin desktop-primary multi-workspace + permissions + notifications + geography config; per-Connector-Builder browser-IDE for no-code source-creation; per-CDK code-editor for code-based connector; mobile read-only sync-status; per-Marketplace connector-browser with cert-tier filtering.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10435:**UX shell adaptation.** White-collar (analytics-engineer + marketer + data-engineer + privacy-officer); desktop-primary debugger + workspace-explorer + audience-builder + tracking-plan editor; per-Function editor with Monaco IDE; per-Engage-Journey designer; per-Profile-Explorer with event-timeline; per-Privacy-Portal for DSAR queue + consent-mapping; mobile read-only debugger; embedded analytics.js / mobile-SDK in customer apps + sites (no oyatie UX on customer-property surface).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10499:**UX shell adaptation.** White-collar (analytics-engineer + data-engineer + marketer + privacy-officer); desktop-primary workspace + connection-builder + transformation editor (Monaco IDE with git + Node/Python runtime); per-Profiles workspace with dbt-style SQL models + lineage; per-Live-Event-Debugger; per-Reverse-ETL config; per-Cloud-Extract source config; per-Tracking-Plan Data Catalog; mobile read-only debugger; embedded rudder-sdk-js / mobile-SDK in customer apps + sites.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10655:  14. Cut over: per-folder + per-user content + per-embed-host iframe-URL redirected; Looker instance kept read-only for 90 days for audit + appeal.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10684:  9. Cutover wave-by-wave per folder + per-embed-host; Looker read-only for 90-day audit grace.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10807:  13. Cutover wave-by-wave per project + per-embed-host; Hex workspace kept read-only for 60 days.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10813:  - Collaborative-edit conflict on concurrent cell-edit during migration cutover: per-cell operational-transform or vector-clock + read-only-window during cutover.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10835:  9. Wave-by-wave cutover per project + per-embed-host; Hex read-only 60-day grace.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:10988:  8. Wave-by-wave cutover per Workspace + per-embed-host; Mode read-only 60-day grace.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11122:  16. Cutover wave-by-wave per Folder + per-Embed-Host; Sigma kept read-only for 60 days; tenant ledger evidence per ADR-0247.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11151:  10. Wave-by-wave cutover per Folder + per-Embed-Host; Sigma read-only 60-day grace; ledger entry per ADR-0247.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11290:  15. Cutover wave-by-wave per Folder + per-Embed-Host; ThoughtSpot kept read-only for 90 days; tenant ledger evidence per ADR-0247.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11320:  10. Wave-by-wave cutover per Folder + per-Embed-Host; ThoughtSpot read-only 90-day grace; ledger entry per ADR-0247.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11333:  - `User` (User_ID, Email, Name, Role {limited_user / observer / responder / user / restricted_access / manager / admin / read_only_user / global_admin / owner}, Teams[], Contact_Methods[Contact_ID → Type {email / sms / voice / push} + Address + Push-Device], Notification_Rules[Rule_ID → Urgency-Filter + Contact_Reference + Delay-Minutes], Time_Zone, License-Type).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11685:  - `Index` (Index_Name, Index_Type {events / metrics / federated / summary}, Bucket-Policy {hot / warm / cold / frozen}, Retention-Days, Max-Size, Storage_Tier, Replication_Factor, Search_Factor, App_Owner, Tenant_Reference).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:11811:  3. Per-Index: re-create in Oyatie `siem` with bucket-policy mapped to Oyatie tiered-storage (hot=SSD, warm=SSD, cold=HDD/S3, frozen=tenant-cold-archive); preserve retention + CMEK.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:12448:- Oyatie destination: `observability` (logs + metrics + traces + APM + RUM + synthetic + profiler + uptime anchor with Elasticsearch storage tier compatibility {hot / warm / cold / frozen / searchable-snapshot}); `siem` (SIEM rules + detection-engine + Endpoint EDR + Cases); `intelligence` (Anomaly-Detection + Data-Frame-Analytics + ESRE + ELSER/E5 + AI-Assistant — BYOK per ADR-0255 §D-4); `secops` (CSPM + CWP + Endpoint); `workflow-engine` (Alerting v2 + Connectors); `developer-sdk` (ES|QL + KQL + ES-SQL + REST API superset); `data-pipeline` (Elastic-Agent + Fleet + Logstash + Beats + Ingest-Pipeline-compat); `apigw` (API-keys + Fleet enrollment-token); `compliance` (audit + EU/US/Govt residency); `community` (Workplace Search content collaboration adjacency).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:12451:  - `Cluster` (Cluster_Name, Cluster_UUID, Nodes[Node_ID → {Role {master / data / data_hot / data_warm / data_cold / data_frozen / ingest / ml / transform / remote_cluster_client / coordinating_only}, OS, JVM, Allocation}], License, Region, Tier).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:12455:  - `ILM_Policy` (Policy_Name, Phases {hot / warm / cold / frozen / delete → {actions: rollover / shrink / forcemerge / freeze / migrate / searchable_snapshot / delete}}).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:16333:  - "Mirror Columns" — read-only cross-board reference.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:16444:  - Mirror-column read-only enforcement (mirror-columns reflect source but cannot be written): per-mirror direction preserve.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:17189:  - `Shared_Folder` (Folder_ID, Account-Ref, Name, Members[User-Ref + Permission{Allow-Editing / Read-Only / Allow-Admin / Hide-Passwords}], Items-Count, Created-At).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:17302:  13. Migrate Shared-Folders → Oyatie shared-vault with permission-mapping (Allow-Editing / Read-Only / Allow-Admin / Hide-Passwords).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:17533:  - `Member` (Member_ID, Tenant-Ref, User-Ref, Role {Admin / Group-Admin / Org-Admin / Collaborator / Read-Only / Custom}, SAML-SSO-Bound, SCIM-Provisioned).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:17874:  - Nexus REST API v1: `https://<nexus>/service/rest/v1/` — `repositories`, `repositories/{format}/{type}/{name}`, `components`, `assets`, `blobstores`, `routing-rules`, `cleanup-policies`, `read-only`, `script`, `security/users`, `security/roles`, `security/privileges`, `security/realms`, `security/anonymous`, `security/ldap`, `security/saml`, `tasks`, `tasks/{id}/run`, `lifecycle/bounce`, `audit-events`, `search`, `search/assets`, `status/check`.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:18581:  - `Space` (formerly Room) (Space_ID, Org-Ref, Title, Type {direct / group}, Is-Locked, Last-Activity, Creator-Ref, Members[Member-ID + Person-Ref + IsModerator + IsMonitor], Team-Ref, Classification-ID, Is-Public, Is-Read-Only, Is-Announcement-Only).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:19339:  - JSON API: `?format=json` query for any page returns JSON representation (read-only).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:20216:  - Content API: `https://<site>/ghost/api/content/` read-only for `posts` + `pages` + `tags` + `authors` + `tiers` + `settings`.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:20522:  - `Cluster` (Cluster_ID, Project-Ref, Name, Provider {AWS / GCP / Azure}, Region, Instance-Size {M0..M700 / Serverless / Flex}, MongoDB-Version, Disk-Size-GB, Replication-Spec{Num-Shards + Replication-Factor + Read-Only-Specs + Analytics-Specs}, Backup-Enabled, Encryption-At-Rest-Provider, Auto-Scaling, Paused).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21413:- Vendor name and category: PlanetScale (PlanetScale, Inc.) — Managed MySQL-compatible serverless database built on Vitess: Databases + Branches (Git-style schema branching) + Deploy Requests (schema deployment PRs) + Schema Snapshots + Connection Strings (per-branch + per-password) + Insights (query performance + index suggestions) + Boost (managed Vitess query caching) + Safe Migrations (online DDL via gh-ost) + Foreign Key Constraints (limited) + Vector indexing (preview) + PlanetScale CLI (`pscale`) + REST API + Terraform provider + AWS / GCP regions + PlanetScale (logical replication CDC to Snowflake/BigQuery via Singer + ETL partners) + Workflow rollback + Audit Log + SSO (SAML for Scaler Pro+) + IP Allowlist + AWS PrivateLink (Enterprise) + Customer-Managed-Keys (Enterprise) + Multi-region + Backup (daily snapshots + PITR) + Read-only regions (replicas) + Scaler / Scaler Pro / Enterprise tiers.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21419:  - `Database` (DB_ID, Org-Ref, Name, Region, Cluster-Size {PS-10 / PS-20 / PS-40 / PS-80 / PS-160 / PS-320 / PS-400}, Allow-Data-Branching, Default-Branch-Name, Production-Branches-Web-Console-Read-Only).
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21450:  - "Console (in-browser SQL)" — read-only on production, full on dev branches.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21455:  - "Multi-region Replicas" — read-only replicas in additional regions.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21540:  12. Per-Read-Only-Region + PrivateLink + IP-Allowlist + CMK: re-establish.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21557:  - Production-branch read-only web-console invariant: per-branch console-policy preserve.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21568:  5. Migrate audit-log + SSO + IP-allowlist + PrivateLink + CMK + read-only regions.
origin/dev:docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:21588:  - `Endpoint` (Endpoint_ID, Project-Ref, Branch-Ref, Type {read_write / read_only}, Compute-Provisioner {k8s-pod / k8s-neonvm}, Suspend-Timeout-Seconds, Autoscaling-Limit-Min-CU, Autoscaling-Limit-Max-CU, Host, Region, State {init / active / idle / inactive}).
origin/dev:docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:445:states that the work is read-only or findings-only and cannot promote ahead of
origin/dev:docs/decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md:436:defined in ADR-0316 §D-7 is retained read-only as historical evidence
origin/dev:docs/decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md:440:ADR-0316 §D-7 is retired. The table is retained read-only as historical
origin/dev:docs/decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md:622:retained read-only as historical evidence in the migration ledger. No
origin/dev:docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md:952:- Surface tenant_class in the user profile API (read-only for end
origin/dev:docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md:1706:  former tenant with retained read-only access for the contractual
origin/dev:docs/decisions/ADR-0337-iceberg-canonical-olap-write-path.md:256:This ADR binds Oyatie's canonical OLAP catalog as Iceberg REST Catalog (Polaris reference implementation when self-managed; hyperscaler-managed Polaris / Glue / BigLake / Unity Catalog when on AWS / GCP / Databricks / Azure). Delta UniForm catalogs are read-accepted (because UniForm emits Iceberg metadata pointing at Delta data); native Delta Lake catalogs (not UniForm) are read-only adapter-side. Hudi catalogs are read-only adapter-side.
origin/dev:docs/decisions/ADR-0338-pod-runtime-tier-0-to-3.md:553:D-2.8. A µservice MAY use auxiliary mechanisms to harden Tier 2 placement beyond baseline (e.g., AppArmor profiles, SecComp filters, read-only root filesystem, no-new-privileges). These are independent of the tier classification and applied uniformly.
origin/dev:docs/decisions/ADR-0343-dr-rto-rpo-matrix-per-microservice-per-compliance-pack.md:168:ADR-0241 established the canonical four-tier DR portfolio: T1 (< 5 min RTO, 0 RPO, active-active multi-AZ cross-region warm), T2 (60 min RTO, 60 s RPO, active-passive cross-region continuous), T3 (240 min RTO, 900 s RPO, backup-restore cross-region warm), T4 (1440 min RTO, 3600 s RPO, backup-restore cold). The portfolio is declared per µservice via `manifest.json#dr_tier`. It is sufficient when a µservice has a single DR contract that holds across every tenant context the µservice serves.
origin/dev:docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md:448:(9) Cold-merge merges shards while a tenant is in the middle of a write → atomicity invariant: the merge atomically redirects writes to the merged shard after a brief read-only window.
origin/dev:docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md:606:6. Brief read-only window: writes pause for the atomicity boundary; reads continue.
origin/dev:docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md:281:B2.003. Jenkins configuration is config-as-code via JCasC (Jenkins Configuration as Code) plugin. Controller state is authored declaratively under `microservices/cloud-iac/modules/<context>/jenkins/jcasc/` and applied by the OpenTofu module at controller boot. The UI is read-only for state changes per the `oya-governance-jenkins-jcasc-only` lane per E.4.
origin/dev:docs/decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md:516:| Coverage-audit scanning | Determining whether every µservice's declared actions have a permit fragment + default-deny requires fleet-wide manifest enumeration + cross-µservice intersection. A caller's process has no visibility into other callers' manifests. | `coverage-audit` BC: scheduled scan of every µservice's `capabilities/*.yaml` + OpenAPI + AsyncAPI + Cedar fragment store; emits `CoverageReport` rows. CI lane + nightly drift detection. Read-only consumer of µservice manifests + fragment store; not a per-call participant. |
origin/dev:docs/decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md:518:| Cross-cell coverage rollup | Per-call coverage telemetry emits at the caller's library; cross-cell aggregate views (fleet-wide fragment-hit rate by tenant by action) require a rollup process. | Subscribe to the coverage telemetry stream; emit aggregate rows + dashboards. Read-only consumer of coverage telemetry; not a per-call participant. |
origin/dev:docs/decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md:521:| Evaluation-audit rollup | Per-call audit rows emit at the caller; cross-cell aggregate views (fleet-wide permit/forbid ratio by fragment by tenant by audience) require a rollup process. | Subscribe to the audit-chain stream's `PolicyEvaluated` rows; aggregate to compliance dashboards. Read-only consumer of audit-chain; not a per-call participant. |
origin/dev:docs/decisions/ADR-0353-amendment-library-first-network-opt-in-clarification.md:1253:| `GET /v1/admin/observability/evaluation-rollup` | Cross-cell evaluation rollup (read-only consumer of audit-chain stream). | Compliance dashboards + tenant admin. |
origin/dev:docs/decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md:348:| Tool-registry lookup | In-process tool-registry lookup via library-bundled registry snapshot. The registry snapshot is refreshed periodically from the Intelligence µservice's read-only registry endpoint (sub-second freshness is not required; the registry is a slow-moving object). |
origin/dev:docs/decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md:405:| Cross-cell observability rollup | Per-call audit rows emit at the caller; cross-cell aggregate views (fleet-wide LLM spend by provider by tenant by audience) require a rollup process. | Subscribe to the audit-chain stream; emit aggregate rows + dashboards. Read-only consumer of audit-chain; not a per-call participant. |
origin/dev:docs/decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md:406:| Cost-attribution aggregator | Per-call cost rows emit at the caller; tenant-scoped FinOps rollups (per ADR-0242 §D-7 deepest-declared-sub-scope) require aggregation. | Subscribe to the audit-chain stream's `IntelligenceCostAttributed` rows; aggregate to FinOps portal. Read-only consumer. |
origin/dev:docs/decisions/ADR-0355-amendment-library-first-network-opt-in-clarification.md:844:| `GET /v1/admin/observability/cost-rollup` | Cross-cell cost rollup (read-only consumer of audit-chain stream). | FinOps portal + tenant admin. |
origin/dev:docs/decisions/ADR-0356-amendment-library-first-ontology-read-path.md:567:| Audit-stream consumer | Per-call read-sample audit rows emit at the caller's library; cross-cell aggregate read views require a rollup. | Subscribe to the audit-chain stream's `OntologyRead` rows; aggregate. Read-only consumer; not a per-call participant. |
origin/dev:docs/decisions/ADR-0358-ideal-production-roadmap-strangler-bazel-oya-overlay.md:31:Research grounding: Google TAP (affected-target presubmit) + Bazel remote build execution; Amazon Builders' Library continuous delivery (one-box → cell → region, bake time, automatic metric-gated rollback); Microsoft deployment rings; Oracle OCI DevOps blue-green/canary; Nygard/AWS/Azure ADR immutability+supersession; SSOT single-master + the "constitution-with-amendments" anti-pattern; Rust CI tooling (cargo-nextest partition sharding, sccache remote cache, cargo-deny, cargo-machete, bacon for local). Bazel `rules_rust` is chosen over Buck2 because Buck2 requires Reindeer to vendor all Cargo deps (hostile to Git/code-review) and is less battle-tested in OSS, while `rules_rust` supports Cargo.toml-as-SSOT (`crate_universe`) with mature RBE.
origin/dev:docs/decisions/ADR-0358-ideal-production-roadmap-strangler-bazel-oya-overlay.md:36:2. **Toolchain overhaul = Bazel `rules_rust` build graph + `oya` governance overlay.** Bazel provides the build/test DAG, hermetic remote cache, remote build execution, and affected-target selection (Cargo.toml stays the dependency SSOT via `crate_universe`). `oya` is rebuilt as a thin governance/verify orchestrator that delegates build/test to `bazel query`/`bazel test` and runs only the bespoke governance gates — retiring the duplicated cargo-mirror `verify`/`run-all` engine.
origin/dev:docs/decisions/ADR-0359-jenkins-completely-replaces-github-actions.md:38:4. **Migration is sequenced and reversible** (see masterplan `ideal_production_roadmap.P-TOOLCHAIN`): stand up cloud-ci controller + config-as-code + ephemeral K8s agents; port the lanes; wire the GitHub App + switch required checks; retire `.github/workflows`; then layer the remote cache (sccache->SeaweedFS) and Bazel affected-targets (ADR-0358). The local `oya verify` mirror is the gate during the transition.
origin/dev:docs/decisions/ADR-0360-ci-pipeline-optimization-program.md:3:title: CI/CD pipeline optimization program — affected-target precision, gate-only overlay, warm shared cache, test sharding, pinned+signed agent image, speculative merge queue, content-addressed gate caching
origin/dev:docs/decisions/ADR-0360-ci-pipeline-optimization-program.md:20:# ADR-0360: CI/CD pipeline optimization program — affected-target precision, gate-only overlay, warm shared cache, test sharding, pinned+signed agent image, speculative merge queue, content-addressed gate caching
origin/dev:docs/decisions/ADR-0360-ci-pipeline-optimization-program.md:28:Direct observation (2026-05-25): `oya verify --ci-required` runs `cargo {check,clippy,nextest} --workspace --all-targets` with **no affected-target selection**, so a change touching only docs/specs/evidence YAML still triggers a whole-workspace cargo + test mirror (observed: a 1342-file diff that was ~99% non-Rust ran the full mirror for 10+ minutes). Hyperscaler CI (Google TAP, Bazel) instead runs the **reverse-dependency closure** of the change, shards tests, reuses a warm remote cache populated by trunk, and gates merges with a speculative always-green queue. `specs/cloud-toolchain-target.json` already names these as targets; this ADR commits the program and its correctness rules, grounded in `docs/ideas/pipeline-optimization.md` and best-practice research (Google SWE book Ch.23, Bazel remote-cache/query, cargo-nextest, sccache, Zuul/GitHub merge queue, cosign/Kyverno).
origin/dev:docs/decisions/ADR-0360-ci-pipeline-optimization-program.md:36:- **O3 — Warm shared cache + cached downloads.** Trunk/postsubmit builds get read-write to the blessed sccache prefix; PR builds are read-through (read blessed, write a PR-scoped prefix, promote on merge) — the write principal must equal the trust boundary. `SCCACHE_S3_KEY_PREFIX` encodes the toolchain identity; `CARGO_INCREMENTAL=0`; basedir normalization for path-independent keys. Crate downloads are served by a sparse-registry mirror (Panamax) + a warm read-only `CARGO_HOME` (sccache caches compilation, not downloads).
origin/dev:docs/decisions/ADR-0360-ci-pipeline-optimization-program.md:40:- **O7 — Content-addressed gate caching.** `verdict_key = H(merkle(declared_inputs) ‖ gate_version ‖ config_digest ‖ env_subset)`; cache hit ⇒ reuse verdict, skip the gate. Correctness rule (load-bearing): a gate is cacheable ONLY if it declares all its inputs and is deterministic; per-file gates key on their file set; cross-file/global gates key on the whole corpus digest; **a gate that cannot enumerate its inputs is un-cacheable and always runs** — never risk a false PASS.
origin/dev:docs/decisions/ADR-0374-ci-webhook-gateway-github-actions.md:25:    description: "HMAC-SHA256 webhook-signature verification that fails closed on the RAW body BEFORE any parse/route, constant-time, with the secret redacted in Debug and read only from sref://openbao/oya/ci/github-webhook-secret."
origin/dev:docs/decisions/ADR-0380-ci-loop-closure-on-talos-jenkins-farm-re-establishment.md:132:  maintenance/archive in 2024 — the GitHub repo is read-only. Fails lens (a).
origin/dev:docs/decisions/ADR-0380-ci-loop-closure-on-talos-jenkins-farm-re-establishment.md:134:  uses; daemonless `buildctl`, OCI output, content-addressed cache with
origin/dev:docs/decisions/ADR-0381-kaniko-to-buildkit-and-multinode-talos-cell-topology.md:21:    description: "Replace Kaniko (Google Container Tools — placed into maintenance/archive in 2024; GitHub repo is read-only) with BuildKit (Moby, Apache 2 — what Docker itself uses) as the in-cluster image-build substrate. Rewrite infra/ci-webhook-gateway/kaniko-build.yaml as buildkit-build.yaml (a buildkitd Deployment on the CI specialty pool from D2 + a buildctl client invoked from the Jenkins agent pod template); update infra/registry/registry.k8s.yaml and microservices/ci-webhook-gateway/Dockerfile so the build path is buildctl-driven. Wire BuildKit's `s3` cache backend to SeaweedFS-on-Talos (per ADR-0349) once D4 (storage pool + SeaweedFS) lands. Hyperscaler-lens: BuildKit is Apache 2, actively maintained, used by Docker / GitHub Actions / Cloud Build / Earthly — passes (a)-(d)."
origin/dev:docs/decisions/ADR-0381-kaniko-to-buildkit-and-multinode-talos-cell-topology.md:62:   maintenance/archive in 2024; the GitHub repo is read-only. Current Oyatie
origin/dev:docs/decisions/ADR-0391-n-lane-parallel-safety-proof-and-devops-console.md:40:- Write operations from console v0 (read-only; writes in v1).
origin/dev:docs/decisions/ADR-0391-n-lane-parallel-safety-proof-and-devops-console.md:119:| Subscription admin panel (read-only) | Calls cloud-intelligence admin API; renders seat pool state + token windows. |
origin/dev:docs/decisions/ADR-0391-n-lane-parallel-safety-proof-and-devops-console.md:130:| Write operations in console v0 | Read-only is sufficient for visibility; writes (seat provisioning, policy reload) require additional Cedar policy + audit trail work that is out of v0 scope. |
origin/dev:docs/decisions/ADR-0392-buck2-canonical-build-graph.md:45:Reindeer + NativeLink decision below is unchanged.
origin/dev:docs/decisions/ADR-0392-buck2-canonical-build-graph.md:79:Research grounding (2026-05): Buck2 is Meta's open-source successor to Buck, the build system Meta runs across its monorepo at a scale far beyond most OSS Bazel deployments; Buck2's core is a Rust binary with a Starlark-configured, fully-hermetic, content-addressed action graph and constraint-based incrementality (it recomputes the exact minimal set of affected actions from a precise dependency graph). `buck2-prelude` ships first-party Rust rules. Reindeer (also Meta, the tool Meta itself uses to vendor third-party Rust into its Buck monorepo) reads `Cargo.toml` + `Cargo.lock` and GENERATES a checked-in `third-party/rust/BUCK` plus a vendored/fixups layout — i.e. the buckified third-party graph is a generated, pinned, code-reviewable artifact, not opaque vendoring. NativeLink is an open-source, self-hostable Remote Build Execution + CAS backend (Apache-2, Rust) that speaks the Bazel Remote Execution v2 API and is used in production with Buck2. Self-hostable NativeLink passes the hyperscaler-lens filter (active upstream + clean license + fully self-hostable + a hyperscaler-internal equivalent, with no managed-service dependency); a managed RBE SaaS would NOT pass.
origin/dev:docs/decisions/ADR-0392-buck2-canonical-build-graph.md:83:1. **Buck2 + `buck2-prelude` + Reindeer-buckified third-party is the canonical build graph.** Buck2 (the Rust binary) drives the build/test action DAG with content-addressed, graph-exact incrementality. `buck2-prelude` supplies the first-party Rust toolchain rules. This reverses ADR-0358 §2's "Bazel `rules_rust` build graph"; everything else in ADR-0358 stands.
origin/dev:docs/decisions/ADR-0392-buck2-canonical-build-graph.md:89:4. **Self-hostable NativeLink RBE is the remote backend.** Buck2 targets a self-hosted NativeLink RBE + content-addressed cache (replacing ADR-0358's "bazel RBE" target and the interim sccache→SeaweedFS cache). NativeLink passes the hyperscaler-lens (self-hostable, Apache-2, active upstream, hyperscaler-internal equivalent); a managed RBE service is rejected for the same lens reason.
origin/dev:docs/decisions/ADR-0392-buck2-canonical-build-graph.md:91:5. **Honesty / non-claims.** Buck2 is 0% adopted — there is no `BUCK` file, no `buck2-prelude` vendor, no Reindeer-generated `third-party/rust/BUCK`, no NativeLink deployment, and no migration executed by this ADR. This is doctrine + target, not implementation. NO numeric build-speedup, cache-hit-rate, or incrementality figure is asserted; any such claim is `blocked_until_required_evidence_is_green` per `hyperscaler-gates.json` and may only be made once the migration lands and the evidence is green.
origin/dev:docs/decisions/ADR-0392-buck2-canonical-build-graph.md:95:Positive: the canonical build graph moves to a Rust-native engine (Buck2) consistent with the kernel+OS bespoke-Rust ambition and the Meta-monorepo production pedigree; graph-exact incrementality + content-addressed correctness; a self-hostable NativeLink RBE that passes the hyperscaler-lens with no managed-service dependency; `Cargo.toml`/`Cargo.lock` remain the human dependency SSOT (Reindeer is a generator off it). Negative/cost: we accept the Reindeer buckification step (an extra generated `third-party/rust/BUCK` that must be regenerated and reviewed when deps change) — the exact objection ADR-0358 raised; the migration across the first-party crates is a substantial program; Buck2 first-party Rust + NativeLink operational maturity must be proven before any parity claim. Neutral: ADR-0358's strangler-fig posture, define-production-100-first phase, and masterplan planning authority are unchanged; ADR-0359 (Jenkins-sole-CI) is complementary (Jenkins drives Buck2 — see ADR-0408); the machine-readable specs that encode Bazel are superseded inputs awaiting a separate generated-artifact update; this ADR is doctrine, not the migration execution.
origin/dev:docs/decisions/ADR-0394-bespoke-rust-idp-central-hub.md:60:- **ADR-0209** (compliance evidence automation) — lists "Backstage developer portal (ADR-0170) — read-only auditor view" as one of the existing primitives.
origin/dev:docs/decisions/ADR-0394-bespoke-rust-idp-central-hub.md:114:- **ADR-0209** (compliance evidence automation) — "Backstage developer portal (ADR-0170) — read-only auditor view" → "the ADR-0394 IDP audit/rbac module — read-only auditor view." The auditor-view *primitive* is unchanged; it is now served by the bespoke audit/rbac surface over `audit-chain` + `oya-policy-cedar-api`.
origin/dev:docs/decisions/ADR-0394-bespoke-rust-idp-central-hub.md:141:- The ops-BFF holds all upstream credentials; the WASM shell holds none. Secrets surface is read-only metadata/rotation/lease-TTL only (Cedar + step-up gated), NEVER values.
origin/dev:docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md:41:- **Pipeline-optimized.** The destination pipeline is Buck2 RBE + remote cache + speculative merge-train (ADR-0111/0369). Argo Workflows composes cleanly with Buck2 container steps, Argo Events (cloud-scm webhook → workflow trigger), and ArgoCD/Argo Rollouts CD — one CNCF-aligned, self-hostable family.
origin/dev:docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md:73:- **Pipeline-optimized:** Buck2 RBE + remote cache + the speculative merge-train (ADR-0111/0369) compose as workflow steps; affected-target selection drives the DAG.
origin/dev:docs/decisions/ADR-0512-canonical-monorepo-pattern.md:85:5. **Build / parallelism.** Buck2 fine-grained per-crate targets + remote execution + hermetic content-addressed caching (ADR-0392/0408) is the destination; sccache is the interim shared cache. Because Buck2 is rustc-direct, per-target rustc tuning (codegen-units/LTO/opt/target-cpu), affected-targets test selection via graph query, one polyglot graph (Rust + proto/OpenAPI codegen + Leptos/WASM IDP + container images + IaC), dependency-rule-as-`visibility`, and the `no_std`/custom-sysroot path for the bespoke-kernel ambition all become available. The canonical crate=bounded-context boundary is exactly what Buck2 targets mirror.
origin/dev:docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md:54:    description: "Affected-gate rdeps depth-cap + presubmit/postsubmit two-tier + NativeLink CAS MVP."
origin/dev:docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md:107:                  → buck2 build/test with NativeLink CAS]
origin/dev:docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md:173:| bespoke NativeLink RE | Scheduler + workers (measurement-gated) | open (deferred) | parallelism (after CAS hit-rate validated) |
origin/dev:docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md:281:Keep caches **warm** and ensure **only the part that CHANGED is cold**: content-addressed actions + a
origin/dev:docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md:282:remote/shared read-through cache (Buck2 RE-API → BuildBuddy/EngFlow/BuildBarn/NativeLink) so an
origin/dev:docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md:284:runs only the changed targets + reverse-deps); content-addressed producer artifact (0 recompiles when
origin/dev:docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md:318:stay consistent — executed under a read-only kill-list manifest with per-batch door:one-way founder
origin/dev:docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md:363:throughput that tracks change-size not repo-size (warm-cache + delta-only-cold), every cache layer still
origin/dev:docs/decisions/ADR-0517-one-owned-ast-substrate-content-addressed.md:3:title: "One owned AST substrate (tree-sitter-our-way, rowan-style, content-addressed) read by every consumer; one work-area hash = SCM id + buck2/RBE cache key + CD artifact hash"
origin/dev:docs/decisions/ADR-0517-one-owned-ast-substrate-content-addressed.md:27:Decomposes ADR-0516 Component-wide. The content-address contract is one of the W1 interfaces
origin/dev:docs/decisions/ADR-0517-one-owned-ast-substrate-content-addressed.md:35:violates the minimal-deps doctrine, and it cannot natively give content-addressed node identity (the
context compacted
codex
I’ve confirmed the supplied measurements mix two CI generations. I’m grounding the plan in the local `origin/dev` productized workflow, then using the dirty working-tree copy only to explain the mismatch; the critical finding is that current affected-set scoping is not yet sound enough to qualify as a speedup.
exec
/bin/zsh -lc 'git show origin/dev:.github/workflows/oya-ci-required.yml | rg -n "''^(  [a-zA-Z0-9_-]+:|      - name: (Install Buck2|Materialize|Restore buck-out|Run binding|Bootstrap|Install PostgreSQL)|    name:|    needs:|        matrix:|          gate:)|continue-on-error|CACHE_MODE=|buck2 test //ci/|postgres:16|apt-get|num-threads 1|actions/cache/(restore|save)|producer-regen|gate-live-postgres"' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
23:  workflow_dispatch:
24:  push:
26:  pull_request:
29:  merge_group:
32:  contents: read
33:  actions: read
42:  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
43:  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
50:  producer-regen:
51:    name: producer-regen (accounting-registry)
66:      - name: Materialize cloud-ci generated faces
87:  #    `buck2 test //ci/facade/<crate>:{ci-<crate>-unittest,ci-<crate>-gate}` — so instead of copy-pasting a job per gate, a single
102:  #    itself, so each leg downloads the producer-regen artifact instead of paying its own
106:  #    byte-parity detector — detectors never consume the thing they attest), producer-regen
108:  #    ADR-0551). `needs: producer-regen` serializes these legs behind a ~75s producer job;
109:  #    the workflow critical path (affected-set/buck2 lanes) is unaffected. If producer-regen
114:  gate:
115:    needs: producer-regen
119:    name: ${{ matrix.label }}
172:      # Consume the producer-regen artifact (faces + volatile scm snapshot) instead of
176:      - name: Download regenerated faces (producer-regen artifact, ADR-0556 D5 QW-1)
201:  gate-generated-artifact-freshness:
202:    name: freshness (lock + generated faces, ADR-0539)
235:  #    producer-regen; it rematerializes in-job so it is hermetic and self-contained. The
236:  #    producer-regen needs-edge was cosmetic (evidence only, nothing consumed) and serialized
238:  gate-inventory-registry-drift:
239:    name: registry-drift (materialized == regenerated)
256:      # feeding it the producer-regen artifact it is supposed to verify would make the
258:      - name: Materialize faces then assert byte-parity
263:          buck2 test //ci/facade/inventory-registry-drift:ci-inventory-registry-drift-gate
269:  gate-baseline-ratchet:
270:    name: cloud-ci-firewall (baseline ratchet + gate-registration meta-test)
287:      # out-of-band bootstrap ref, and is deliberately absent from the producer-regen artifact.
290:      - name: Materialize cloud-ci generated faces
307:  generated-output-diff-policy:
308:    name: generated-output-diff-policy (no generated merge surfaces)
363:  buck2:
364:    name: buck2 (hermetic build + affected gate tests)
463:      - name: Restore buck-out (read-only; dev-push is the sole writer)
464:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
465:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
477:      # by design (ADR-0551) and deliberately absent from the producer-regen artifact; this
479:      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
497:          buck2 test //ci/... --unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json
508:          CACHE_MODE=bypass
543:  gate-affected-target-set:
544:    name: "gate · affected-set (ADR-0554, binding workspace coverage)"
629:      - name: Restore buck-out (read-only; dev-push is the sole writer)
630:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
631:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
641:      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
671:      - name: Materialize merge-base build-health baseline when affected-set needs FULL
885:        # actions/cache/save@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
886:        uses: actions/cache/save@27d5ce7f107fe9357f9df03efb73ab90386fccae
927:  #    deterministic bootstrap. Inside each group, `--num-threads 1` and sequential
930:  gate-live-postgres-adapters:
931:    name: "gate-live-postgres-adapters (durable adapters: RLS / CDC / SCIM, #901)"
936:        image: postgres:16
974:          sudo apt-get update
975:          sudo apt-get install -y --no-install-recommends postgresql-client
977:      - name: Bootstrap app role + durable schemas/roles (admin, adapters)
997:            "gate_id": "gate-live-postgres-adapters",
1000:              "image": "postgres:16",
1051:          buck2 test --local-only --num-threads 1 //libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest -- "${LIVE_ENV[@]}"
1052:          buck2 test --local-only --num-threads 1 //libs/oya-data-outbox-adapter-postgres:oya-data-outbox-adapter-postgres-unittest -- "${LIVE_ENV[@]}"
1053:          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-unittest -- "${LIVE_ENV[@]}"
1054:          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-live -- "${LIVE_ENV[@]}"
1055:          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-unittest -- "${LIVE_ENV[@]}"
1056:          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-live -- "${LIVE_ENV[@]}"
1067:  gate-live-postgres-facades:
1068:    name: "gate-live-postgres-facades (durable facades: tenant lifecycle / SCIM, #901)"
1073:        image: postgres:16
1111:          sudo apt-get update
1112:          sudo apt-get install -y --no-install-recommends postgresql-client
1114:      - name: Bootstrap app role + durable schemas/roles (admin, facades)
1134:            "gate_id": "gate-live-postgres-facades",
1137:              "image": "postgres:16",
1174:          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-tests -- "${FACADE_ENV[@]}"
1175:          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-e2e -- "${FACADE_ENV[@]}"
1176:          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-unittest -- "${FACADE_ENV[@]}"
1177:          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-acceptance -- "${FACADE_ENV[@]}"
1194:  oya-ci-required:
1195:    name: oya-ci-required
1198:    needs:
1206:      - gate-live-postgres-adapters # #901: durable adapter RLS / CDC / SCIM tests against isolated live Postgres
1207:      - gate-live-postgres-facades  # #901: durable facade tenant lifecycle / SCIM tests against isolated live Postgres
1219:          echo "  live-postgres/adapters = ${{ needs.gate-live-postgres-adapters.result }}"
1220:          echo "  live-postgres/facades  = ${{ needs.gate-live-postgres-facades.result }}"
1230:            && [ "${{ needs.gate-live-postgres-adapters.result }}" = "success" ] \
1231:            && [ "${{ needs.gate-live-postgres-facades.result }}" = "success" ]; then
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/lib.rs | nl -ba | sed -n '350,390p;680,840p;900,1045p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   350	                        plan.classified
   351	                            .push((path.to_owned(), PathClass::PackagePattern(pat)));
   352	                    }
   353	                    None => {
   354	                        plan.full_reasons.push(format!(
   355	                            "package sibling `{path}` maps to no configured cell root (derivation uncertainty)"
   356	                        ));
   357	                        plan.classified
   358	                            .push((path.to_owned(), PathClass::DeletedGraphFile));
   359	                    }
   360	                },
   361	            }
   362	            continue;
   363	        }
   364	        match change {
   365	            Change::Deleted(_) => {
   366	                if policy
   367	                    .require_owner_patterns
   368	                    .iter()
   369	                    .any(|pat| glob_match(pat, path))
   370	                {
   371	                    plan.full_reasons
   372	                        .push(format!("graph-relevant file `{path}` was deleted"));
   373	                    plan.classified
   374	                        .push((path.to_owned(), PathClass::DeletedGraphFile));
   375	                } else {
   376	                    plan.classified
   377	                        .push((path.to_owned(), PathClass::DeletedIrrelevant));
   378	                }
   379	            }
   380	            Change::Present(_) => {
   381	                plan.owner_paths.push(path.to_owned());
   382	                plan.classified
   383	                    .push((path.to_owned(), PathClass::OwnerQuery));
   384	            }
   385	        }
   386	    }
   387	    plan.package_patterns.sort();
   388	    plan.package_patterns.dedup();
   389	    plan
   390	}
   680	    let artifacts = payload
   681	        .get("artifacts")
   682	        .and_then(Value::as_array)
   683	        .ok_or("workflow-artifacts payload has no `artifacts` array")?;
   684	
   685	    for artifact in artifacts {
   686	        if artifact.get("name").and_then(Value::as_str) == Some(artifact_name) {
   687	            if artifact
   688	                .get("expired")
   689	                .and_then(Value::as_bool)
   690	                .unwrap_or(true)
   691	            {
   692	                return Ok(None);
   693	            }
   694	            return artifact
   695	                .get("id")
   696	                .and_then(Value::as_u64)
   697	                .map(Some)
   698	                .ok_or("matching trusted artifact has no numeric `id`".to_owned());
   699	        }
   700	    }
   701	
   702	    Ok(None)
   703	}
   704	
   705	/// Validate a trusted build-health baseline artifact payload after provenance selection.
   706	///
   707	/// Returns the number of build-report results. Empty/invalid reports are refused because an empty
   708	/// baseline would launder every head failure into "brand-new but unproven" ambiguity.
   709	pub fn validate_trusted_build_health_baseline_artifact(
   710	    artifact_name: &str,
   711	    merge_base_sha: &str,
   712	    report_json: &str,
   713	) -> Result<usize, String> {
   714	    let expected = build_health_baseline_artifact_name(merge_base_sha)?;
   715	    if artifact_name != expected {
   716	        return Err(format!(
   717	            "build-health baseline artifact name `{artifact_name}` does not match expected `{expected}`"
   718	        ));
   719	    }
   720	    let report = parse_build_report(report_json)?;
   721	    if report.is_empty() {
   722	        return Err("build-health baseline artifact has an empty `results` object".to_owned());
   723	    }
   724	    Ok(report.len())
   725	}
   726	/// The build-health verdict: regressions BLOCK, pre-existing failures are GRANDFATHERED.
   727	#[derive(Debug, Clone, PartialEq, Eq)]
   728	pub struct BuildHealthVerdict {
   729	    /// Targets that FAIL at head but did NOT fail at the merge-base (built there, or are brand
   730	    /// new): REGRESSIONS. Non-empty => BLOCK.
   731	    pub regressions: Vec<String>,
   732	    /// Targets that fail at head AND failed at the merge-base: GRANDFATHERED (shrink-only).
   733	    pub grandfathered: Vec<String>,
   734	    /// Targets that failed at the merge-base but now BUILD at head: burned-down (informational).
   735	    pub fixed: Vec<String>,
   736	}
   737	
   738	impl BuildHealthVerdict {
   739	    /// The required-context verdict: green IFF there are no regressions.
   740	    pub fn is_green(&self) -> bool {
   741	        self.regressions.is_empty()
   742	    }
   743	}
   744	
   745	/// Compute the build-health verdict (PURE).
   746	///
   747	/// `baseline_failures` = failing-target set at the MERGE-BASE (frozen, out-of-band).
   748	/// `head_failures`     = failing-target set at the PR HEAD.
   749	///
   750	/// A head failure is a REGRESSION iff it is not in the baseline failure set (it built at the
   751	/// merge-base, or the target is brand-new). A head failure that IS in the baseline is
   752	/// grandfathered. This is exactly set-difference, so a PR can only ever SHRINK the grandfathered
   753	/// set or ADD a regression — it can never launder a regression into the baseline, because the
   754	/// baseline is supplied from the merge-base build, not from any candidate-controlled input.
   755	pub fn build_health_verdict(
   756	    baseline_failures: &BTreeSet<String>,
   757	    head_failures: &BTreeSet<String>,
   758	) -> BuildHealthVerdict {
   759	    let regressions: Vec<String> = head_failures
   760	        .difference(baseline_failures)
   761	        .cloned()
   762	        .collect();
   763	    let grandfathered: Vec<String> = head_failures
   764	        .intersection(baseline_failures)
   765	        .cloned()
   766	        .collect();
   767	    let fixed: Vec<String> = baseline_failures
   768	        .difference(head_failures)
   769	        .cloned()
   770	        .collect();
   771	    BuildHealthVerdict {
   772	        regressions,
   773	        grandfathered,
   774	        fixed,
   775	    }
   776	}
   777	
   778	#[cfg(test)]
   779	mod tests {
   780	    use super::*;
   781	
   782	    #[test]
   783	    fn glob_star_within_segment() {
   784	        assert!(glob_match("*.bzl", "macros.bzl"));
   785	        assert!(!glob_match("*.bzl", "dir/macros.bzl"));
   786	        assert!(glob_match("**/*.bzl", "a/b/macros.bzl"));
   787	        assert!(glob_match("**/*.bzl", "macros.bzl"));
   788	    }
   789	
   790	    #[test]
   791	    fn glob_double_star_prefix_and_exact() {
   792	        assert!(glob_match("toolchains/**", "toolchains/BUCK"));
   793	        assert!(glob_match("toolchains/**", "toolchains/a/b.bzl"));
   794	        assert!(!glob_match("toolchains/**", "toolchainsx/a"));
   795	        assert!(glob_match(".buckconfig", ".buckconfig"));
   796	        assert!(!glob_match(".buckconfig", "x/.buckconfig"));
   797	    }
   798	
   799	    #[test]
   800	    fn glob_double_star_matches_zero_segments() {
   801	        assert!(glob_match("third-party/**", "third-party/BUCK"));
   802	        assert!(glob_match("a/**/b", "a/b"));
   803	        assert!(glob_match("a/**/b", "a/x/y/b"));
   804	    }
   805	
   806	    #[test]
   807	    fn package_pattern_root_cell() {
   808	        let policy = test_policy();
   809	        assert_eq!(
   810	            package_pattern("cloud/x/Cargo.toml", &policy),
   811	            Some("//cloud/x:".to_owned())
   812	        );
   813	        assert_eq!(
   814	            package_pattern("Cargo.toml", &policy),
   815	            Some("//:".to_owned())
   816	        );
   817	    }
   818	
   819	    fn test_policy() -> Policy {
   820	        Policy {
   821	            gate_id: GATE_ID.to_owned(),
   822	            universe: "//...".to_owned(),
   823	            full_run_targets: vec!["//...".to_owned()],
   824	            full_trigger_patterns: vec![
   825	                ".buckconfig".to_owned(),
   826	                "toolchains/**".to_owned(),
   827	                "third-party/**".to_owned(),
   828	                "**/*.bzl".to_owned(),
   829	                "Cargo.lock".to_owned(),
   830	            ],
   831	            require_owner_patterns: vec!["**/*.rs".to_owned()],
   832	            package_definition_basenames: vec!["BUCK.v2".to_owned(), "BUCK".to_owned()],
   833	            package_sibling_basenames: vec!["Cargo.toml".to_owned(), "build.rs".to_owned()],
   834	            cell_roots: BTreeMap::from([(String::new(), "//".to_owned())]),
   835	            default_base_ref: "origin/main".to_owned(),
   836	        }
   837	    }
   838	
   839	    #[test]
   840	    fn policy_rejects_wrong_gate_id() {
   900	    }
   901	    #[test]
   902	    fn trusted_build_health_artifact_accepts_exact_non_empty_baseline() {
   903	        let sha = "0123456789abcdef0123456789abcdef01234567";
   904	        let name = build_health_baseline_artifact_name(sha).unwrap();
   905	        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
   906	        assert_eq!(
   907	            validate_trusted_build_health_baseline_artifact(&name, sha, json),
   908	            Ok(1)
   909	        );
   910	    }
   911	
   912	    #[test]
   913	    fn trusted_build_health_artifact_rejects_stale_name() {
   914	        let sha = "0123456789abcdef0123456789abcdef01234567";
   915	        let stale = "build-health-baseline-89abcdef0123456789abcdef0123456789abcdef";
   916	        let json = r#"{"results":{"root//a:a":{"success":"SUCCESS"}}}"#;
   917	        let err = validate_trusted_build_health_baseline_artifact(stale, sha, json).unwrap_err();
   918	        assert!(err.contains("does not match expected"), "{err}");
   919	    }
   920	
   921	    #[test]
   922	    fn trusted_build_health_artifact_rejects_invalid_or_empty_report() {
   923	        let sha = "0123456789abcdef0123456789abcdef01234567";
   924	        let name = build_health_baseline_artifact_name(sha).unwrap();
   925	
   926	        let invalid =
   927	            validate_trusted_build_health_baseline_artifact(&name, sha, "not json").unwrap_err();
   928	        assert!(invalid.contains("not valid JSON"), "{invalid}");
   929	
   930	        let empty =
   931	            validate_trusted_build_health_baseline_artifact(&name, sha, r#"{"results":{}}"#)
   932	                .unwrap_err();
   933	        assert!(empty.contains("empty `results`"), "{empty}");
   934	    }
   935	
   936	    #[test]
   937	    fn trusted_build_health_artifact_rejects_bad_sha_shape() {
   938	        let err = build_health_baseline_artifact_name("dev").unwrap_err();
   939	        assert!(err.contains("40-character hex"), "{err}");
   940	    }
   941	
   942	    #[test]
   943	    fn trusted_push_run_selection_accepts_exact_successful_dev_push() {
   944	        let sha = "0123456789abcdef0123456789abcdef01234567";
   945	        let runs = r#"{
   946	            "workflow_runs": [
   947	                {"id": 11, "head_sha": "fedcba9876543210fedcba9876543210fedcba98", "event": "push", "head_branch": "dev", "conclusion": "success"},
   948	                {"id": 12, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "pull_request", "head_branch": "dev", "conclusion": "success"},
   949	                {"id": 13, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "success"}
   950	            ]
   951	        }"#;
   952	        assert_eq!(trusted_dev_push_run_id(runs, sha), Ok(Some(13)));
   953	    }
   954	
   955	    #[test]
   956	    fn trusted_push_run_selection_falls_back_on_missing_or_untrusted() {
   957	        let sha = "0123456789abcdef0123456789abcdef01234567";
   958	        let runs = r#"{
   959	            "workflow_runs": [
   960	                {"id": 12, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "feature", "conclusion": "success"},
   961	                {"id": 13, "head_sha": "0123456789abcdef0123456789abcdef01234567", "event": "push", "head_branch": "dev", "conclusion": "failure"}
   962	            ]
   963	        }"#;
   964	        assert_eq!(trusted_dev_push_run_id(runs, sha), Ok(None));
   965	    }
   966	
   967	    #[test]
   968	    fn trusted_baseline_artifact_selection_accepts_unexpired_exact_match() {
   969	        let artifacts = r#"{
   970	            "artifacts": [
   971	                {"id": 21, "name": "build-health-baseline-fedcba9876543210fedcba9876543210fedcba98", "expired": false},
   972	                {"id": 22, "name": "build-health-baseline-0123456789abcdef0123456789abcdef01234567", "expired": false}
   973	            ]
   974	        }"#;
   975	        assert_eq!(
   976	            trusted_build_health_baseline_artifact_id(
   977	                artifacts,
   978	                "build-health-baseline-0123456789abcdef0123456789abcdef01234567",
   979	            ),
   980	            Ok(Some(22))
   981	        );
   982	    }
   983	
   984	    #[test]
   985	    fn trusted_baseline_artifact_selection_falls_back_on_missing_or_stale() {
   986	        let artifact_name = "build-health-baseline-0123456789abcdef0123456789abcdef01234567";
   987	        assert_eq!(
   988	            trusted_build_health_baseline_artifact_id(r#"{"artifacts":[]}"#, artifact_name),
   989	            Ok(None)
   990	        );
   991	        assert_eq!(
   992	            trusted_build_health_baseline_artifact_id(
   993	                r#"{"artifacts":[{"id":22,"name":"build-health-baseline-0123456789abcdef0123456789abcdef01234567","expired":true}]}"#,
   994	                artifact_name,
   995	            ),
   996	            Ok(None)
   997	        );
   998	    }
   999	    #[test]
  1000	    fn build_health_regression_blocks_grandfathered_does_not() {
  1001	        // baseline (merge-base) red: {blake3, sqlx}. head red: {blake3, sqlx, NEW}.
  1002	        // blake3+sqlx grandfathered; NEW is a regression -> BLOCK.
  1003	        let baseline = set(&["root//tp:blake3", "root//libs:sqlx"]);
  1004	        let head = set(&["root//tp:blake3", "root//libs:sqlx", "root//oya:new-break"]);
  1005	        let v = build_health_verdict(&baseline, &head);
  1006	        assert_eq!(v.regressions, vec!["root//oya:new-break".to_string()]);
  1007	        // BTreeSet intersection yields sorted order.
  1008	        assert_eq!(
  1009	            v.grandfathered,
  1010	            vec!["root//libs:sqlx".to_string(), "root//tp:blake3".to_string()]
  1011	        );
  1012	        assert!(!v.is_green(), "a regression must block");
  1013	    }
  1014	
  1015	    #[test]
  1016	    fn build_health_only_pre_existing_red_is_green_via_grandfather() {
  1017	        // This is the #702 shape: the FULL run is red ONLY on the 4 pre-existing breakages, all
  1018	        // present at the merge-base -> all grandfathered -> GREEN (no flag-day requirement).
  1019	        let baseline = set(&[
  1020	            "root//third-party:blake3",
  1021	            "root//libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest",
  1022	            "root//oya/ci-controller/crates/oya-ci-controller-app:oya-ci-controller",
  1023	            "root//libs/oya-shared-backbone-grpc-generated-adapter:oya-shared-backbone-grpc-generated-adapter-build-script-run",
  1024	        ]);
  1025	        let head = baseline.clone();
  1026	        let v = build_health_verdict(&baseline, &head);
  1027	        assert!(v.regressions.is_empty());
  1028	        assert_eq!(v.grandfathered.len(), 4);
  1029	        assert!(
  1030	            v.is_green(),
  1031	            "pre-existing-only red must be green via grandfather"
  1032	        );
  1033	    }
  1034	
  1035	    #[test]
  1036	    fn build_health_a_target_that_built_at_merge_base_then_fails_is_a_regression() {
  1037	        // The core ratchet semantics + the laundering guard: a target NOT in the baseline
  1038	        // failure set that fails at head is a regression, FULL STOP. There is no candidate input
  1039	        // that can add it to `baseline` (the baseline comes from the merge-base build), so a PR
  1040	        // cannot reclassify its own regression as pre-existing (#698 F1 lesson).
  1041	        let baseline = set(&["root//tp:blake3"]); // only blake3 was red at merge-base
  1042	        let head = set(&["root//tp:blake3", "root//libs:was-green-now-red"]);
  1043	        let v = build_health_verdict(&baseline, &head);
  1044	        assert_eq!(
  1045	            v.regressions,
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/affected-set-policy.json | nl -ba && git show origin/dev:specs/cache-warm-license.json | nl -ba && git show origin/dev:specs/cache-warmth-policy.json | nl -ba | sed -n '1,110p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	{
     2	  "_comment": "cloud-ci-affected-set binding workspace-coverage policy (DATA, not code). ADR-0554, converts FRIC-1781310000: the only binding buck2 lane was scoped to //cloud/cloud-ci/..., so code anywhere else could merge broken (PR #651 head cf16525 did not compile yet its buck2 lane was green). ALL repo-specifics live here; the Rust kernel hardcodes no oyatie path and runs on any buck2 repo by editing this pack. full_trigger_patterns are the rdeps-cone ESCAPE classes — graph-semantic files whose blast radius the owner()/rdeps() derivation cannot model (build config, toolchains, vendored third-party, Starlark macros, the lockfile): any touch escalates to the FULL workspace, mechanically, with no skip and no human judgment. require_owner_patterns are the classes that MUST map to an owning target; an existing file in these classes with no owner FAILS the lane (graph-invisible code is not made safe by running more targets).",
     3	  "gate_id": "cloud-ci-affected-set",
     4	  "schema_version": "1.0.0",
     5	  "universe": "//...",
     6	  "full_run_targets": [
     7	    "//..."
     8	  ],
     9	  "_full_trigger_note": "Two seam classes here. (1) Build config/macros whose blast radius the per-package rdeps cone cannot bound — .buckconfig + .buckconfig.local + .buckconfig.d/** (all read by buck2, all committable), toolchains/**, third-party/** (reindeer vendor + fixups), Starlark **/*.bzl + **/*.bxl, rust-toolchain.toml. (2) Buildfiles and PACKAGE files are handled by package_definition_basenames (escalate to FULL on any change) AND mirrored here as **/PACKAGE so a NEW PACKAGE file (which evaluates to [] and would otherwise look like a plain no-owner file) is never a silent no-op. Cargo.lock is deliberately NOT a trigger: buck2 never reads it — a dependency change that affects buck2 semantics MUST touch third-party/**; the cargo lanes + ADR-0539 freshness gate own lock hygiene.",
    10	  "full_trigger_patterns": [
    11	    ".buckconfig",
    12	    ".buckconfig.local",
    13	    ".buckconfig.d/**",
    14	    "toolchains/**",
    15	    "third-party/**",
    16	    "**/*.bzl",
    17	    "**/*.bxl",
    18	    "**/PACKAGE",
    19	    "rust-toolchain.toml"
    20	  ],
    21	  "require_owner_patterns": [
    22	    "**/*.rs",
    23	    "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-aarch64-adapter/linker.ld",
    24	    "cloud/cloud-kernel/crates/oya-cloud-kernel-arch-x86-64-adapter/linker.ld"
    25	  ],
    26	  "_package_sibling_note": "Cargo.toml + build.rs are NOT buck2 graph inputs (buck2 never reads them; reindeer/BUCK mirror them) and have no owner() BY DESIGN, so requiring an owner would refuse every manifest edit (proven by this lane's own first dogfood run). They are semantically bound to their crate, so they seed the ENCLOSING package target pattern; if that package does not exist the seed query fails and the lane escalates to FULL.",
    27	  "package_sibling_basenames": [
    28	    "Cargo.toml",
    29	    "build.rs"
    30	  ],
    31	  "_package_definition_note": "Ground-truth buck2 buildfile-name set in PRECEDENCE order: BUCK.v2 SHADOWS BUCK when both exist (empirically verified against this buck2 binary — adding an empty BUCK.v2 next to a real BUCK drops the BUCK targets). A change to ANY buildfile escalates to FULL (these basenames are also escape-class): a buildfile edit can add/remove targets or shadow the file dependents load, so its blast radius is NOT bounded by its own package's rdeps. owner() is empty for a buildfile by design, so seeding 'its package' alone would silently miss every dependent (F2). If buck2's [buildfile] name/extra_for_test config ever adds names, mirror them here — this list IS the repo's buildfile-name ground truth, not a single hand-set name.",
    32	  "package_definition_basenames": [
    33	    "BUCK.v2",
    34	    "BUCK"
    35	  ],
    36	  "cell_roots": {
    37	    "": "//"
    38	  },
    39	  "default_base_ref": "origin/dev",
    40	  "product_contract": {
    41	    "born_pack_shaped": "The escape-trigger classes, owner-required classes, universe, full-run patterns, cell roots, and base ref are DATA here; the Rust kernel hardcodes no repo path nor any oyatie string (R0, ADR-0548). The kernel DOES fix the decision SEMANTICS — RefuseUnowned > Full > Affected > NoGraphTargets dominance, derivation-uncertainty-escalates-to-full, owner() on every existing changed file regardless of extension — that contract is the engine, not a per-repo pack value.",
    42	    "fail_closed": "Derivation NEVER skips: git/uquery/rdeps errors, unmappable package files, deleted graph files, and empty rdeps closures all escalate to the full workspace run. The only hard failures are an unreadable/invalid pack, owner-required files with no owning target, and the build/test verdict itself.",
    43	    "tiers": "pull_request -> auto (affected cone, escalation binding); merge_group + push + workflow_dispatch -> full (admission/integration tier per ADR-0515 Tide direction).",
    44	    "precedent": "Bazel target determination / bazel-diff (Tinder), Meta/Google affected-target CI; reimplemented Rust-native on buck2 uquery owner()/rdeps() per the proven-patterns doctrine.",
    45	    "execution": "Buck2-native Rust lane; no shell logic beyond the workflow YAML step (G011 Rust-successor of the transitional infra/ci/buck2-affected-gate.sh)."
    46	  },
    47	  "purpose": "Binding workspace coverage: every PR builds+tests the buck2 reverse-dependency cone of its merge-base diff as a REQUIRED context, with mechanical fail-closed escalation to the full workspace whenever the cone cannot be trusted. Any owner-required source change with a buck2 target builds+tests its cone; any buildfile/config/macro change escalates to FULL; a genuinely-unowned owner-required file (e.g. a source in a sub-workspace buck2 does not model) REFUSES the merge until it is wired — never silently passes."
    48	}
     1	{
     2	  "_comment": "Declarative warm-license kill-switch (DATA, not code) — ADR-0560, enforcing the ADR-0556 D2 trust invariant: a build class MAY run warm IFF it is warm-eligible in /specs/cache-warmth-policy.json AND the most recent scheduled cold integrity-canary run is GREEN. This file is the mechanical carrier of clause (b): the cache-wiring resolver refuses every warm mode while warm_reads_licensed is false, regardless of class. It ships FALSE and stays false until (1) the NativeLink CAS endpoints are live and reachable from the executing lanes and (2) the integrity-canary has produced a GREEN verdict against the warm substrate. Flipping to true is a reviewed change citing that first GREEN run; flipping to false is the ADR-0556 canary-RED response (suspend-all-warm, shrink needs no door) — performed today by PR (transitional local bridge), by the canary reconciler when it lands (ADR-0556 D4.3 successor). Surface model per ADR-0556 D4: consumed by the resolver and the conformance gate, never an operator CLI.",
     3	  "schema_version": "1.0.0",
     4	  "adr": "ADR-0560",
     5	  "policy": "specs/cache-warmth-policy.json",
     6	  "warm_reads_licensed": false,
     7	  "reason": "no live CAS endpoint is reachable from any executing lane and the cold integrity-canary has never run GREEN against a warm substrate — the ADR-0556 D2 IFF is unsatisfied on clause (b); slice 1 ships the wiring dark",
     8	  "licensed_by_canary_run": null,
     9	  "red_response": "on canary RED: this flag flips to false fleet-wide before any divergence remediation begins (ADR-0556 D2: a RED canary suspends ALL warm reads pending the next GREEN run); divergent-key eviction and the friction-ledger row open per the canary_red_response steps in the policy"
    10	}
     1	{
     2	  "_comment": "Build cache-warmth classification (DATA, not code) — ADR-0556, founder directive 2026-06-12: 'some things should be cold. some things can be warm. make that distinction well.' Every build class maps to {warmth, cache_read, cache_write, reason}. Consumers (interim CI quick-wins, the W3 NativeLink CAS vertical, the future cache-policy-conformance gate) read this policy rather than re-deciding warmth per change. R0 pack-shape: ALL repo-specifics live here; an adopting repo edits the classes, never an engine. Surface model (ADR-0556 D4): this file is declarative policy consumed by services/controllers — never a CLI an operator runs; cache-write authorization is enforced at the CAS service boundary (keyed authn), never by client discipline.",
     3	  "policy_id": "cache-warmth-policy",
     4	  "schema_version": "1.0.0",
     5	  "adr": "ADR-0556",
     6	  "schema": {
     7	    "build_class": {
     8	      "warmth": "cold | warm",
     9	      "cache_read": "bool — may this class read the shared cache",
    10	      "cache_write": "bool — may this class write to the shared cache",
    11	      "reason": "string — WHY, citing the governing invariant"
    12	    }
    13	  },
    14	  "trust_invariant": {
    15	    "statement": "A build class MAY run warm IFF (a) it is warm-eligible in this policy AND (b) the most recent scheduled cold integrity-canary run is GREEN (cold output digests byte-identical to the warm CAS digests for the same action keys). Warm-by-default is sound ONLY BECAUSE the cold canary continuously proves warm == cold; the canary is the trust anchor that licenses warmth.",
    16	    "canary_build_class": "integrity-canary",
    17	    "canary_red_response": [
    18	      "per the IFF: a RED canary SUSPENDS ALL WARM READS FLEET-WIDE pending the next GREEN run — the items below are the durable remediation, never a license to keep serving warm on non-divergent keys while RED stands",
    19	      "RED is a blocking hermeticity/non-determinism defect (ADR-0525 D4 violation), never tolerable noise; the warm cache is structurally suspect from that moment",
    20	      "divergent action keys are evicted/quarantined from the CAS immediately (reconciler/API action by the canary controller, not a hand operation)",
    21	      "a friction-ledger row opens mechanically (ADR-0544 closed loop); if root cause is not established within one canary period, the warm-eligible classes covering the divergent cone degrade to cold via this DATA (shrink-of-warmth needs no door)",
    22	      "NOT permitted: serving ANY warm hit while RED stands (the IFF is unsatisfied), resuming on divergent keys after GREEN returns without the eviction above, or widening the comparison tolerance — cold == warm is byte-equality, the registry-drift bar"
    23	    ],
    24	    "deployment_precondition": "the CAS vertical MUST ship the integrity-canary in the same change that enables fleet-wide warm reads — no canary, no warm"
    25	  },
    26	  "trust_boundary": {
    27	    "trusted_author": "a same-repo branch pushed by an authorized writer and admitted into the governance pipeline (required_workflow lanes); holds the CAS write key",
    28	    "untrusted_author": "fork PRs and any context without the CAS write key (GitHub fork PRs receive read-only tokens and no secrets — a natural seam, but the binding enforcement is the CAS service boundary authn/authz, never runner configuration)"
    29	  },
    30	  "default_for_unlisted_classes": {
    31	    "warmth": "cold",
    32	    "cache_read": false,
    33	    "cache_write": false,
    34	    "reason": "fail-closed default the trust invariant already implies, made mechanical for the conformance gate: a build class not listed in build_classes has no warm license — warmth is granted by reviewed classification, never by omission"
    35	  },
    36	  "build_classes": {
    37	    "release-production-image": {
    38	      "warmth": "cold",
    39	      "cache_read": false,
    40	      "cache_write": false,
    41	      "reason": "reproducibility + SBOM/provenance integrity (ADR-0039, ADR-0181): the shipped artifact must derive from exactly its sources via a from-source build; a cache hit substitutes bytes whose derivation was attested elsewhere or nowhere. No write-back: release builds run with the most-privileged signing identity — writing from that context maximizes blast radius. The rust-purity sole cargo exception (cargo --release + lto fat + locked) lives on this path, outside the buck2 graph."
    42	    },
    43	    "integrity-canary": {
    44	      "warmth": "cold",
    45	      "cache_read": false,
    46	      "cache_write": false,
    47	      "reason": "the trust anchor that licenses warm-by-default (ADR-0556 D2): a scheduled from-empty build of the pinned graph whose output digests are byte-compared against the warm CAS digests for the same action keys; any cache participation makes the proof circular. cold != warm = hermeticity/non-determinism bug, fail-closed."
    48	    },
    49	    "untrusted-author-presubmit": {
    50	      "warmth": "cold",
    51	      "cache_read": false,
    52	      "cache_write": false,
    53	      "reason": "anti-poisoning (Bazel/Google RBE security model): an untrusted PR controls action inputs; with write access it seeds poisoned outputs under action keys trusted builds will later hit. Write prohibition is one-way; default is full isolation (defense in depth, no cache-probing side channel). A read-only relaxation (cache_read true) is a reviewed two-way policy edit — reads cannot inject. Enforced at the CAS service boundary: untrusted contexts hold no key."
    54	    },
    55	    "provenance-attestation": {
    56	      "warmth": "cold",
    57	      "cache_read": false,
    58	      "cache_write": false,
    59	      "reason": "SLSA: provenance must describe the build that actually happened; serving cached outputs while attesting build steps fabricates provenance, and reproducible-build verification requires re-derivation."
    60	    },
    61	    "presubmit-trusted-dep-closure": {
    62	      "warmth": "warm",
    63	      "cache_read": true,
    64	      "cache_write": true,
    65	      "reason": "the third-party crate closure (reindeer-vendored, lockfile-pinned) is identical across every PR sharing a lockfile; rebuilding it per leg and per run is pure waste. Content-addressed: a hit is bit-identical to cold (licensed by the trust invariant)."
    66	    },
    67	    "presubmit-trusted-affected-cone": {
    68	      "warmth": "warm",
    69	      "cache_read": true,
    70	      "cache_write": true,
    71	      "reason": "the affected-target cone (ADR-0525 D3 uquery owner->rdeps, binding via the ADR-0554 affected-set lane): only genuinely changed actions miss; the unchanged cone is a hit — ADR-0515 D4 (wall-clock tracks the change, not the repo) made real."
    72	    },
    73	    "dev-agentic-iteration": {
    74	      "warmth": "warm",
    75	      "cache_read": true,
    76	      "cache_write": true,
    77	      "reason": "agent-lane and dev-loop builds in throwaway worktrees see 0% hits today (FRIC-1781070457-buck2-no-shared-cache); a warm shared cache makes the agent fleet's wall-clock track the size of each change."
    78	    },
    79	    "gate-fleet-shared-graph": {
    80	      "warmth": "warm",
    81	      "cache_read": true,
    82	      "cache_write": true,
    83	      "reason": "the gate fleet's shared dependency hub — the accounting-registry producer and the common workspace graph rebuilt ~13x per oya-ci-required run across legs (faces re-materialized in every matrix leg). One build, many consumers. Same-run artifact reuse (QW-1) is this class without a CAS; deliberate exception: registry-drift keeps its own in-job rematerialization — detectors never consume the thing they attest."
    84	    },
    85	    "postmerge-dev-trunk": {
    86	      "warmth": "warm",
    87	      "cache_read": true,
    88	      "cache_write": true,
    89	      "reason": "the canonical trusted populator (Bazel/Google pattern: post-merge CI fills the cache, presubmits hit it); trunk content passed oya-ci-required and is by definition admitted content."
    90	    }
    91	  },
    92	  "consumers": [
    93	    "interim CI quick-wins (ADR-0556 D5: QW-1 artifact reuse, QW-2 cargo-matrix collapse, QW-3 stable-key buck-out restore, QW-4 toolchain caching) — warm-safe list only; no cache step may be added to a cold-required path",
    94	    "NativeLink CAS W3 vertical (ADR-0556 D3: buck2_re_client cache-only; allow_cache_uploads per-class from this policy; MUST ship the integrity-canary in the same change)",
    95	    "future cache-policy-conformance gate (the enforcement successor, ADR-0555 D4 gate-test-plus-policy-DATA: asserts the live CI cache configuration matches this policy)"
    96	  ],
    97	  "product_contract": {
    98	    "born_pack_shaped": "ADR-0548 paved-road rule: the classes, the trust boundary, and the invariant prose are DATA here; consumers hardcode no class list. An adopting repo edits build_classes for its own pipeline shapes.",
    99	    "door_asymmetry": "the cold-required floor is one-way (moving a cold-required class to warm requires superseding ADR-0556); warm-eligible membership is two-way DATA (degrading warm to cold is always allowed; growing warmth is a reviewed change)."
   100	  }
   101	}
exec
/bin/zsh -lc 'git status --short --branch && git rev-parse HEAD && git rev-parse origin/dev && git rev-list --count HEAD..origin/dev' in /Users/jasonlee/Developer/oyatie
 succeeded in 163ms:
## preserve/hermes-w1-dirty-20260630...origin/preserve/hermes-w1-dirty-20260630
 M .github/branch-protection.yaml
 M .github/workflows/oya-ci-required.yml
 M AGENTS.md
 M CLAUDE.md
 M Cargo.lock
 M Cargo.toml
 M Makefile
 M cloud/cell-lifecycle/ARCH.md
 M cloud/cell-lifecycle/PRD.md
 M cloud/cell-lifecycle/README.md
 M cloud/cell-lifecycle/capabilities/capability-catalog.md
 M cloud/cell-lifecycle/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cell-lifecycle/runbooks/auto-rebalance.md
 M cloud/cell-lifecycle/runbooks/cold-merge.md
 M cloud/cell-lifecycle/runbooks/hot-split.md
 M cloud/cell-lifecycle/threat-models/threat-model.md
 M cloud/cell-rebalancer/ARCH.md
 M cloud/cell-rebalancer/PRD.md
 M cloud/cell-rebalancer/README.md
 M cloud/cell-rebalancer/capabilities/capability-catalog.md
 M cloud/cell-rebalancer/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cell-rebalancer/runbooks/auto-rebalance.md
 M cloud/cell-rebalancer/runbooks/cold-merge.md
 M cloud/cell-rebalancer/runbooks/hot-split.md
 M cloud/cell-rebalancer/threat-models/threat-model.md
 M cloud/cloud-billing-tax/ARCH.md
 M cloud/cloud-billing-tax/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-billing-tax/PRD.md
 M cloud/cloud-billing-tax/README.md
 M cloud/cloud-billing-tax/capabilities/capability-catalog.md
 M cloud/cloud-billing-tax/crates/oya-cloud-billing-tax-app/src/lib.rs
 M cloud/cloud-billing-tax/crates/oya-cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs
 M cloud/cloud-billing-tax/dpia/dpia.md
 M cloud/cloud-billing-tax/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-billing-tax/runbooks/auto-rebalance.md
 M cloud/cloud-billing-tax/runbooks/cold-merge.md
 M cloud/cloud-billing-tax/runbooks/hot-split.md
 M cloud/cloud-billing-tax/threat-models/threat-model.md
 M cloud/cloud-billing/ARCH.md
 M cloud/cloud-billing/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-billing/PRD.md
 M cloud/cloud-billing/README.md
 M cloud/cloud-billing/capabilities/capability-catalog.md
 M cloud/cloud-billing/crates/oya-cloud-billing-domain/src/lib.rs
 M cloud/cloud-billing/dpia/dpia.md
 M cloud/cloud-billing/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-billing/runbooks/auto-rebalance.md
 M cloud/cloud-billing/runbooks/cold-merge.md
 M cloud/cloud-billing/runbooks/hot-split.md
 M cloud/cloud-billing/runbooks/invoice-generation-timeout.md
 M cloud/cloud-billing/runbooks/per-tenant-cost-attribution-mismatch.md
 M cloud/cloud-billing/runbooks/reservation-recommendation-engine-stall.md
 M cloud/cloud-billing/threat-models/threat-model.md
 M cloud/cloud-capacity/crates/oya-cloud-capacity-domain/src/lib.rs
 M cloud/cloud-capacity/crates/oya-cloud-capacity-kernel/src/committed_use.rs
 M cloud/cloud-capacity/crates/oya-cloud-capacity-kernel/src/lib.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/BUCK
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/Cargo.toml
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/accounting-registry.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/decision-crosswalk.generated.json
D  cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-liveness.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/gate-disposition.json
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app/src/lib.rs
 M cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/BUCK
 M cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/Cargo.toml
 M cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app/src/lib.rs
 M cloud/cloud-ci/gates/registry-drift/BUCK
 M cloud/cloud-ci/gates/registry-drift/Cargo.toml
 M cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs
 M cloud/cloud-compute/crates/oya-cloud-compute-domain/src/lib.rs
 M cloud/cloud-compute/crates/oya-cloud-resource-domain/src/lib.rs
 M cloud/cloud-data/ARCH.md
 M cloud/cloud-data/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-data/PRD.md
 M cloud/cloud-data/README.md
 M cloud/cloud-data/capabilities/capability-catalog.md
 M cloud/cloud-data/dpia/dpia.md
 M cloud/cloud-data/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-data/runbooks/auto-rebalance.md
 M cloud/cloud-data/runbooks/cold-merge.md
 M cloud/cloud-data/runbooks/hot-split.md
 M cloud/cloud-data/threat-models/threat-model.md
 M cloud/cloud-iac/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-iac/cell-topology/foundation.json
 M cloud/cloud-iac/manifest.json
 M cloud/cloud-iac/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-iam/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-iam/capabilities/capability-catalog.md
 M cloud/cloud-iam/dpia/dpia.md
 M cloud/cloud-iam/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-intelligence/crates/oya-cloud-intelligence-kernel/src/lib.rs
 M cloud/cloud-intelligence/manifest.json
 M cloud/cloud-k8s/ARCH.md
 M cloud/cloud-k8s/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-k8s/PRD.md
 M cloud/cloud-k8s/README.md
 M cloud/cloud-k8s/capabilities/capability-catalog.md
 M cloud/cloud-k8s/dpia/dpia.md
 M cloud/cloud-k8s/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-k8s/runbooks/auto-rebalance.md
 M cloud/cloud-k8s/runbooks/cold-merge.md
 M cloud/cloud-k8s/runbooks/hot-split.md
 M cloud/cloud-k8s/threat-models/threat-model.md
 M cloud/cloud-kms/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-kms/capabilities/capability-catalog.md
 M cloud/cloud-kms/crates/oya-cloud-kms-api/src/lib.rs
 M cloud/cloud-kms/crates/oya-cloud-kms-api/tests/cloud_kms_api.rs
 M cloud/cloud-kms/crates/oya-cloud-kms-domain/src/lib.rs
 M cloud/cloud-kms/dpia/dpia.md
 M cloud/cloud-kms/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-network-dns/ARCH.md
 M cloud/cloud-network-dns/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-network-dns/PRD.md
 M cloud/cloud-network-dns/README.md
 M cloud/cloud-network-dns/capabilities/capability-catalog.md
 M cloud/cloud-network-dns/dpia/dpia.md
 M cloud/cloud-network-dns/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-network-dns/runbooks/auto-rebalance.md
 M cloud/cloud-network-dns/runbooks/cold-merge.md
 M cloud/cloud-network-dns/runbooks/hot-split.md
 M cloud/cloud-network-dns/threat-models/threat-model.md
 M cloud/cloud-network/ARCH.md
 M cloud/cloud-network/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-network/PRD.md
 M cloud/cloud-network/README.md
 M cloud/cloud-network/capabilities/capability-catalog.md
 M cloud/cloud-network/crates/oya-cloud-network-adapter-selfhosted/src/lib.rs
 M cloud/cloud-network/crates/oya-cloud-network-domain/src/lib.rs
 M cloud/cloud-network/crates/oya-cloud-network-lb-api/src/lib.rs
 M cloud/cloud-network/crates/oya-cloud-network-vpc-api/src/lib.rs
 M cloud/cloud-network/dpia/dpia.md
 M cloud/cloud-network/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-network/runbooks/auto-rebalance.md
 M cloud/cloud-network/runbooks/cold-merge.md
 M cloud/cloud-network/runbooks/cross-cell-routing-stall.md
 M cloud/cloud-network/runbooks/ddos-mitigation-engagement.md
 M cloud/cloud-network/runbooks/hot-split.md
 M cloud/cloud-network/runbooks/mtls-handshake-failure-cascade.md
 M cloud/cloud-network/threat-models/threat-model.md
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/ca.rs
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/error.rs
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/lib.rs
 M cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/signer.rs
 M cloud/cloud-secrets/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-secrets/README.md
 M cloud/cloud-secrets/capabilities/capability-catalog.md
 M cloud/cloud-secrets/dpia/dpia.md
 M cloud/cloud-secrets/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-storage/ARCH.md
 M cloud/cloud-storage/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/cloud-storage/PRD.md
 M cloud/cloud-storage/README.md
 M cloud/cloud-storage/capabilities/capability-catalog.md
 M cloud/cloud-storage/dpia/dpia.md
 M cloud/cloud-storage/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/cloud-storage/runbooks/auto-rebalance.md
 M cloud/cloud-storage/runbooks/cold-merge.md
 M cloud/cloud-storage/runbooks/hot-split.md
 M cloud/cloud-storage/threat-models/threat-model.md
 M cloud/managed-k8s-cluster-lifecycle/PRD.md
 M cloud/managed-k8s-cluster-lifecycle/audit-evidence-emission.md
 M cloud/managed-k8s-cluster-lifecycle/cost-budget.md
 M cloud/managed-k8s-cluster-lifecycle/crates/oya-managed-k8s-cluster-lifecycle-api/src/lib.rs
 M cloud/managed-k8s-cluster-lifecycle/crates/oya-managed-k8s-cluster-lifecycle-app/src/lib.rs
 M cloud/managed-k8s-cluster-lifecycle/crates/oya-managed-k8s-cluster-lifecycle-kernel/src/lib.rs
 M cloud/managed-k8s-cluster-lifecycle/failure-modes.md
 M cloud/managed-k8s-cluster-lifecycle/implementation_ready_acceptance_criteria.md
 M cloud/managed-k8s-cluster-lifecycle/manifest.json
 M cloud/managed-k8s-cluster-lifecycle/operational-boundaries.md
 M cloud/managed-k8s-cluster-lifecycle/runbooks/cluster-create-fail-closed.md
 M cloud/managed-k8s-cluster-lifecycle/runbooks/runbooks/quota-store-unavailable.md
 M cloud/managed-k8s-cluster-lifecycle/tenant-isolation.md
 M cloud/managed-k8s-cluster-lifecycle/threat-model.md
 M cloud/managed-k8s-control-plane-host/IPs/IP-001-control-plane-host-foundation.md
 M cloud/managed-k8s-control-plane-host/PRD.md
 M cloud/managed-k8s-control-plane-host/adr-links.md
 M cloud/managed-k8s-control-plane-host/capabilities/control-plane-provision.yaml
 M cloud/managed-k8s-control-plane-host/capabilities/control-plane-status.yaml
 M cloud/managed-k8s-control-plane-host/capabilities/control-plane-teardown.yaml
 M cloud/managed-k8s-control-plane-host/contracts/openapi.yaml
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-adapter-capi/Cargo.toml
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-adapter-capi/src/lib.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-api/src/lib.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-app/src/lib.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-app/src/main.rs
 M cloud/managed-k8s-control-plane-host/crates/oya-managed-k8s-control-plane-host-kernel/src/lib.rs
 M cloud/managed-k8s-control-plane-host/implementation_ready_acceptance_criteria.md
 M cloud/managed-k8s-control-plane-host/manifest.json
 M cloud/managed-k8s-control-plane-host/tenant-isolation.md
 M cloud/managed-k8s-control-plane-host/threat-model.md
 M cloud/managed-k8s-sla-observability/PRD.md
 M cloud/managed-k8s-sla-observability/audit-evidence-emission.md
 M cloud/managed-k8s-sla-observability/cedar/quota-rbac.cedar
 M cloud/managed-k8s-sla-observability/contracts/asyncapi-v1.yaml
 M cloud/managed-k8s-sla-observability/contracts/managed-k8s-sla-observability.proto
 M cloud/managed-k8s-sla-observability/contracts/openapi-v1.yaml
 M cloud/managed-k8s-sla-observability/cost-budget.md
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-api/src/lib.rs
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-app/src/lib.rs
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-kernel/src/lib.rs
 M cloud/managed-k8s-sla-observability/crates/oya-managed-k8s-sla-observability-kernel/tests/mwmb_acceptance.rs
 M cloud/managed-k8s-sla-observability/failure-modes.md
 M cloud/managed-k8s-sla-observability/manifest.json
 M cloud/managed-k8s-sla-observability/operational-boundaries.md
 D cloud/managed-k8s-sla-observability/runbooks/runbooks/quota-store-unavailable.md
 M cloud/managed-k8s-sla-observability/slos/managed-cluster-availability.openslo.yaml
 M cloud/managed-k8s-sla-observability/slos/provisioning-latency.openslo.yaml
 M cloud/managed-k8s-sla-observability/tenant-isolation.md
 M cloud/managed-k8s-sla-observability/threat-model.md
 M cloud/managed-k8s-tenant-quota/crates/oya-managed-k8s-tenant-quota-app/src/lib.rs
 M cloud/managed-k8s-tenant-quota/crates/oya-managed-k8s-tenant-quota-app/src/main.rs
 M cloud/managed-k8s-tenant-quota/manifest.json
 M cloud/tenancy/ARCH.md
 M cloud/tenancy/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M cloud/tenancy/PRD.md
 M cloud/tenancy/README.md
 M cloud/tenancy/capabilities/capability-catalog.md
 M cloud/tenancy/contracts/openapi/tenancy.yaml
 M cloud/tenancy/contracts/proto/tenancy.proto
 M cloud/tenancy/crates/oya-tenancy-api/BUCK
 M cloud/tenancy/crates/oya-tenancy-api/Cargo.toml
 M cloud/tenancy/crates/oya-tenancy-api/src/lib.rs
 M cloud/tenancy/crates/oya-tenancy-api/tests/tenant_create_api.rs
 M cloud/tenancy/dpia/dpia.md
 M cloud/tenancy/manifest.json
 M cloud/tenancy/migration-playbooks/wave-15-zd-adr-0346-0349-migration-playbook.md
 M cloud/tenancy/multi-region.md
 M cloud/tenancy/policy/tenant-scope.cedar
 M cloud/tenancy/runbooks/auto-rebalance.md
 M cloud/tenancy/runbooks/citus-rebalance.md
 M cloud/tenancy/runbooks/cold-merge.md
 M cloud/tenancy/runbooks/cross-tenant-data-leak-containment.md
 M cloud/tenancy/runbooks/dr-pair-promotion-drill.md
 M cloud/tenancy/runbooks/hot-split.md
 M cloud/tenancy/runbooks/jwt-key-rotation.md
 M cloud/tenancy/runbooks/kyb-kyc-pipeline-stalled.md
 M cloud/tenancy/runbooks/parent-child-permit-revocation.md
 M cloud/tenancy/runbooks/rls-drift-recovery.md
 M cloud/tenancy/runbooks/tenant-deletion-dsr-cascade.md
 M cloud/tenancy/runbooks/tenant-isolation-breach-response.md
 M cloud/tenancy/runbooks/tenant-onboarding.md
 M cloud/tenancy/runbooks/tenant-suspension.md
 M cloud/tenancy/threat-models/threat-model.md
 M docs/ADR-INDEX.md
 M docs/AGENTS-OPERATING-CONTRACT.md
 M docs/AGENTS.md
 M docs/CHANGELOG.md
 M docs/MASTERPLAN.md
 M docs/PRIVACY-PROGRAM.md
 M docs/README.md
 M docs/RELEASE-MANAGEMENT.md
 M docs/RUNBOOKS-INDEX.md
 M docs/STANDARDS-AND-TEMPLATES.md
 M docs/checklists/done-definition-checklist.md
 M docs/checklists/per-implementation-plan-checklist.md
 M docs/checklists/pr-review-checklist.md
 M docs/checklists/pre-merge.md
 M docs/checklists/pre-push.md
 M docs/checklists/release-readiness-checklist.md
 M docs/ci/forge-of-record.md
 M docs/decisions/ADR-0032-dcim-software-for-own-dc-ops.md
 M docs/decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md
 M docs/decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md
 M docs/decisions/ADR-0157-api-gateway-tier.md
 M docs/decisions/ADR-0158-multi-region-active-active.md
 M docs/decisions/ADR-0163-tenant-environment-tiers.md
 M docs/decisions/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md
 M docs/decisions/ADR-0187-canonical-oidc-idp-zitadel-primary.md
 M docs/decisions/ADR-0211-in-house-tech-stack-policy.md
 M docs/decisions/ADR-0334-shorts-microservice-merged-into-social.md
 M docs/decisions/ADR-0394-bespoke-rust-idp-central-hub.md
 M docs/oya-ci/config-reference.md
 M docs/oya-ci/gate-catalog.md
 M docs/products/cloud/PRD.md
 M docs/products/foundry/PRD.md
 M docs/standards/INDEX.md
 M docs/standards/agent-instructions-discipline.md
 M docs/standards/claude-code-harness.md
 M docs/standards/clean-architecture.md
 M docs/standards/code-style-rust.md
 M docs/standards/crate-naming-convention.md
 M docs/standards/data-class.md
 M docs/standards/dependency-policy.md
 M docs/standards/doc-style.md
 M docs/standards/error-handling.md
 M docs/standards/git-workflow.md
 M docs/standards/image-discipline.md
 M docs/standards/observability.md
 M docs/standards/on-call.md
 M docs/standards/release-management.md
 M docs/standards/release.md
 M docs/standards/security-review.md
 M docs/standards/testing.md
 M docs/templates/INDEX.md
 M docs/templates/adr-template.md
 M docs/templates/implementation-plan-template.md
 M docs/templates/pull-request-template-v2.md
 M docs/templates/pull-request-template.md
 M docs/templates/team-charter-template.md
 M infra/branch-protection/dev.json
 M infra/capi/clusters/README.md
 M infra/capi/clusters/templates/clusters.yaml
 M infra/capi/clusters/values-example.yaml
 M infra/capi/crs/clusterresourceset.yaml
 M infra/capi/crs/render.sh
 M infra/cloudflare/main.tf
 M infra/talos/installation-media/README.md
 M infra/talos/installation-media/gen-media.sh
 M libs/oya-bus-boundary-kernel/src/lib.rs
 M libs/oya-check-high-risk-auto-decision-refusal/src/lib.rs
 M libs/oya-check-honest-claims/BUCK
 M libs/oya-check-honest-claims/Cargo.toml
 M libs/oya-check-honest-claims/src/lib.rs
 M libs/oya-check-layered-architecture-discipline/src/lib.rs
 M libs/oya-check-otel-trace-propagation/src/lib.rs
 M libs/oya-check-pr-traceability/src/lib.rs
 M libs/oya-check-pre-push/src/lib.rs
 M libs/oya-check-supply-chain/src/lib.rs
 M libs/oya-ci-config/src/bundled/gate-disposition.json
 M libs/oya-ci-config/src/lib.rs
 M libs/oya-ci-gate-contract/src/lib.rs
 M libs/oya-data-boundary-kernel/src/retention_policy.rs
 M libs/oya-data-sql-adapter-sqlx/src/lib.rs
 M libs/oya-gen-microservice-manifests-app/src/lib.rs
 M libs/oya-gen-microservice-manifests-app/src/main.rs
 M libs/oya-gen-microservice-manifests-app/tests/check_mode.rs
 M libs/oya-governance-adapter-with-no-importer-kernel/src/lib.rs
 M libs/oya-governance-gate-catalog-domain/src/lib.rs
 M libs/oya-governance-mistakes-ledger-kernel/src/lib.rs
 M libs/oya-http-latency-budget-middleware-infrastructure/BUCK
 M libs/oya-http-latency-budget-middleware-infrastructure/Cargo.toml
 M libs/oya-http-latency-budget-middleware-infrastructure/src/lib.rs
 M libs/oya-http-router-kernel/src/lib.rs
 M libs/oya-http-telemetry-middleware-infrastructure/src/lib.rs
 M libs/oya-http-wide-event-middleware-infrastructure/BUCK
 M libs/oya-http-wide-event-middleware-infrastructure/Cargo.toml
 M libs/oya-http-wide-event-middleware-infrastructure/src/lib.rs
 M libs/oya-messaging-substrate-kernel/src/conformance.rs
 M libs/oya-messaging-substrate-kernel/src/lib.rs
 M libs/oya-messaging-substrate-kernel/src/reference.rs
 M libs/oya-queue-boundary-kernel/src/lib.rs
 M libs/oya-shared-pdp-adapter-cedar/BUCK
 M libs/oya-shared-pdp-adapter-cedar/Cargo.toml
 M libs/oya-shared-pdp-adapter-cedar/src/lib.rs
 M libs/oya-shared-pdp-adapter-cedar/tests/cedar_pdp_conformance.rs
 M libs/oya-shared-pdp-kernel/src/lib.rs
 M libs/oya-stream-boundary-kernel/src/lib.rs
 M oya-ci.toml
 M oya/accounting/contracts/openapi-v1.meta.yaml
 M oya/accounting/crates/oya-accounting-journal-app/src/lib.rs
 M oya/accounting/crates/oya-accounting-journal-domain/src/lib.rs
 M oya/accounting/crates/oya-accounting-journal-storage-adapter-inmemory/tests/storage.rs
 M oya/analytics/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/analytics/README.md
 M oya/analytics/dpia/dpia.md
 M oya/analytics/runbooks/auto-rebalance.md
 M oya/analytics/runbooks/cold-merge.md
 M oya/analytics/runbooks/hot-split.md
 M oya/api-gateway/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/api-gateway/README.md
 M oya/api-gateway/contracts/api-gateway.openapi.yaml
 M oya/api-gateway/dpia/dpia.md
 M oya/api-gateway/iac/k8s/helm/values.yaml
 M oya/api-gateway/runbooks/auto-rebalance.md
 M oya/api-gateway/runbooks/cold-merge.md
 M oya/api-gateway/runbooks/hot-split.md
 M oya/application/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/application/README.md
 M oya/application/crates/oya-application-shell-frontend/src/app.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/audit_evidence_timeline.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/ops_deployment_status_panel.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/policy_disclosure_banner.rs
 M oya/application/crates/oya-application-shell-frontend/src/design_system/tenant_context_switcher.rs
 M oya/application/crates/oya-application-shell-frontend/src/lib.rs
 M oya/application/crates/oya-application-shell-frontend/src/render_envelope.rs
 M oya/application/crates/oya-application-shell-frontend/src/shell_capability_registry.rs
 M oya/application/crates/oya-application-shell-frontend/src/token_broker.rs
 M oya/application/crates/oya-application-shell-frontend/style/app.css
 M oya/application/dpia/dpia.md
 M oya/application/manifest.json
 M oya/application/runbooks/auto-rebalance.md
 M oya/application/runbooks/cold-merge.md
 M oya/application/runbooks/hot-split.md
 M oya/audit-chain/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/audit-chain/README.md
 M oya/audit-chain/dpia/dpia.md
 M oya/audit-chain/manifest.json
 M oya/audit-chain/runbooks/audit-chain-restart.md
 M oya/audit-chain/runbooks/audit-export.md
 M oya/audit-chain/runbooks/auto-rebalance.md
 M oya/audit-chain/runbooks/chain-replay-from-snapshot-protocol.md
 M oya/audit-chain/runbooks/cold-merge.md
 M oya/audit-chain/runbooks/hot-split.md
 M oya/audit-chain/runbooks/hsm-key-rotation.md
 M oya/audit-chain/runbooks/merkle-root-discrepancy-investigation.md
 M oya/audit-chain/runbooks/merkle-seal-recovery.md
 M oya/audit-chain/runbooks/regulator-evidence-export-failure.md
 M oya/audit-chain/runbooks/retention-cascade.md
 M oya/audit-chain/runbooks/signature-verification-failure.md
 M oya/calendar/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/calendar/README.md
 M oya/calendar/dpia/dpia.md
 M oya/calendar/manifest.json
 M oya/calendar/runbooks/auto-rebalance.md
 M oya/calendar/runbooks/cold-merge.md
 M oya/calendar/runbooks/hot-split.md
 M oya/ci-webhook-gateway/src/dispatch.rs
 M oya/ci-webhook-gateway/src/error.rs
 M oya/ci-webhook-gateway/src/lib.rs
 M oya/comms-email/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/comms-email/README.md
 M oya/comms-email/dpia/dpia.md
 M oya/comms-email/runbooks/auto-rebalance.md
 M oya/comms-email/runbooks/cold-merge.md
 M oya/comms-email/runbooks/hot-split.md
 M oya/community/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/community/README.md
 M oya/community/dpia/dpia.md
 M oya/community/manifest.json
 M oya/community/runbooks/auto-rebalance.md
 M oya/community/runbooks/cold-merge.md
 M oya/community/runbooks/coordinated-spam-attack-response.md
 M oya/community/runbooks/hot-split.md
 M oya/community/runbooks/kb-attachment-restore.md
 M oya/community/runbooks/moderation-queue-clear.md
 M oya/community/runbooks/moderator-decision-appeal-protocol.md
 M oya/community/runbooks/post-mass-deletion.md
 M oya/community/runbooks/search-rebuild.md
 M oya/community/runbooks/spam-flood-throttle.md
 M oya/community/runbooks/verified-anonymous-deanonymization-incident.md
 M oya/community/runbooks/vote-anomaly.md
 M oya/compliance/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/compliance/README.md
 M oya/compliance/dpia/dpia.md
 M oya/compliance/runbooks/audit-seal-verify-failure.md
 M oya/compliance/runbooks/auto-rebalance.md
 M oya/compliance/runbooks/breach-notification-72h-clock-at-risk.md
 M oya/compliance/runbooks/certification-evidence-pipeline-stall.md
 M oya/compliance/runbooks/cold-merge.md
 M oya/compliance/runbooks/cross-tenant-dsar-leak-suspected.md
 M oya/compliance/runbooks/dsar-backlog-overflow.md
 M oya/compliance/runbooks/engagement-cedar-revoke-failed.md
 M oya/compliance/runbooks/evidence-collector-degraded.md
 M oya/compliance/runbooks/hot-split.md
 M oya/compliance/runbooks/manual-evidence-upload-rejected.md
 M oya/compliance/runbooks/pack-overlay-conflict-resolution.md
 M oya/compliance/runbooks/phi-access-anomaly.md
 M oya/compliance/runbooks/regulator-engagement-grant-revoke.md
 M oya/compliance/runbooks/regulator-evidence-export-failure.md
 M oya/compliance/runbooks/seaweedfs-evidence-bucket-loss.md
 M oya/connector/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/connector/README.md
 M oya/connector/crates/oya-connector-slack-adapter/src/lib.rs
 M oya/connector/dpia/dpia.md
 M oya/connector/runbooks/auto-rebalance.md
 M oya/connector/runbooks/cold-merge.md
 M oya/connector/runbooks/hot-split.md
 M oya/consent-graph/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/consent-graph/README.md
 M oya/consent-graph/dpia/dpia.md
 M oya/consent-graph/runbooks/auto-rebalance.md
 M oya/consent-graph/runbooks/cold-merge.md
 M oya/consent-graph/runbooks/hot-split.md
 M oya/contact-center/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/contact-center/README.md
 M oya/contact-center/dpia/dpia.md
 M oya/contact-center/runbooks/auto-rebalance.md
 M oya/contact-center/runbooks/cold-merge.md
 M oya/contact-center/runbooks/hot-split.md
 M oya/contract-lifecycle-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/contract-lifecycle-management/README.md
 M oya/contract-lifecycle-management/dpia/dpia.md
 M oya/contract-lifecycle-management/runbooks/auto-rebalance.md
 M oya/contract-lifecycle-management/runbooks/cold-merge.md
 M oya/contract-lifecycle-management/runbooks/hot-split.md
 M oya/crm/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/crm/README.md
 M oya/crm/crates/oya-crm-customer-engagement-domain/tests/customer_engagement.rs
 M oya/crm/dpia/dpia.md
 M oya/crm/manifest.json
 M oya/crm/runbooks/auto-rebalance.md
 M oya/crm/runbooks/cold-merge.md
 M oya/crm/runbooks/hot-split.md
 M oya/data-pipeline/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/data-pipeline/README.md
 M oya/data-pipeline/dpia/dpia.md
 M oya/data-pipeline/runbooks/auto-rebalance.md
 M oya/data-pipeline/runbooks/cold-merge.md
 M oya/data-pipeline/runbooks/hot-split.md
 M oya/data-warehouse/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/data-warehouse/README.md
 M oya/data-warehouse/dpia/dpia.md
 M oya/data-warehouse/runbooks/auto-rebalance.md
 M oya/data-warehouse/runbooks/cold-merge.md
 M oya/data-warehouse/runbooks/hot-split.md
 M oya/design-collaboration/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/design-collaboration/README.md
 M oya/design-collaboration/dpia/dpia.md
 M oya/design-collaboration/runbooks/auto-rebalance.md
 M oya/design-collaboration/runbooks/cold-merge.md
 M oya/design-collaboration/runbooks/hot-split.md
 M oya/detection/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/detection/PRD.md
 M oya/detection/README.md
 M oya/detection/dpia/dpia.md
 M oya/detection/runbooks/auto-rebalance.md
 M oya/detection/runbooks/cold-merge.md
 M oya/detection/runbooks/hot-split.md
 M oya/developer-sdk/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/developer-sdk/README.md
 M oya/developer-sdk/crates/oya-dev-cli/src/bin/fake-verify-command.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/cloud_iac_cell_topology_gate.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/commands/gate/mod.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/commands/verify.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/hyperscaler_maturity_claims_gate.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/lib.rs
 M oya/developer-sdk/crates/oya-dev-cli/src/supply_chain_gates.rs
 M oya/developer-sdk/crates/oya-dev-cli/tests/gate_cli.rs
 M oya/developer-sdk/crates/oya-dev-cli/tests/oya_verify_ci_mirror.rs
 M oya/developer-sdk/crates/oya-dev-cli/tests/pr_traceability_cli.rs
 M oya/developer-sdk/dpia/dpia.md
 M oya/developer-sdk/runbooks/auto-rebalance.md
 M oya/developer-sdk/runbooks/cold-merge.md
 M oya/developer-sdk/runbooks/hot-split.md
 M oya/diagnostics/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/diagnostics/README.md
 M oya/diagnostics/dpia/dpia.md
 M oya/diagnostics/runbooks/auto-rebalance.md
 M oya/diagnostics/runbooks/cold-merge.md
 M oya/diagnostics/runbooks/hot-split.md
 M oya/docs/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/docs/README.md
 M oya/docs/dpia/dpia.md
 M oya/docs/manifest.json
 M oya/docs/runbooks/auto-rebalance.md
 M oya/docs/runbooks/cold-merge.md
 M oya/docs/runbooks/hot-split.md
 M oya/drive/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/drive/README.md
 M oya/drive/crates/oya-drive-domain/src/lib.rs
 M oya/drive/dpia/dpia.md
 M oya/drive/manifest.json
 M oya/drive/runbooks/auto-rebalance.md
 M oya/drive/runbooks/cold-merge.md
 M oya/drive/runbooks/hot-split.md
 M oya/emergency/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/emergency/README.md
 M oya/emergency/dpia/dpia.md
 M oya/emergency/runbooks/auto-rebalance.md
 M oya/emergency/runbooks/cold-merge.md
 M oya/emergency/runbooks/hot-split.md
 M oya/emr/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/emr/README.md
 M oya/emr/dpia/dpia.md
 M oya/emr/runbooks/auto-rebalance.md
 M oya/emr/runbooks/cold-merge.md
 M oya/emr/runbooks/hot-split.md
 M oya/feature-flags/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/feature-flags/README.md
 M oya/feature-flags/dpia/dpia.md
 M oya/feature-flags/runbooks/auto-rebalance.md
 M oya/feature-flags/runbooks/cold-merge.md
 M oya/feature-flags/runbooks/hot-split.md
 M oya/financial-planning/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/financial-planning/README.md
 M oya/financial-planning/dpia/dpia.md
 M oya/financial-planning/runbooks/auto-rebalance.md
 M oya/financial-planning/runbooks/cold-merge.md
 M oya/financial-planning/runbooks/hot-split.md
 M oya/finops-portal/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/finops-portal/README.md
 M oya/finops-portal/dpia/dpia.md
 M oya/finops-portal/runbooks/auto-rebalance.md
 M oya/finops-portal/runbooks/budget-alert-runaway-firings.md
 M oya/finops-portal/runbooks/cold-merge.md
 M oya/finops-portal/runbooks/cost-allocation-policy-rollback.md
 M oya/finops-portal/runbooks/cost-attribution-mismatch-investigation.md
 M oya/finops-portal/runbooks/credit-application-reconciliation.md
 M oya/finops-portal/runbooks/focus-export-failure.md
 M oya/finops-portal/runbooks/hot-split.md
 M oya/finops-portal/runbooks/quarterly-regulator-emit-miss.md
 M oya/finops-portal/runbooks/reservation-recommendation-engine-stall.md
 M oya/finops-portal/runbooks/tenant-bill-mismatch-resolution.md
 M oya/finops-portal/runbooks/tenant-budget-exhausted.md
 M oya/finops-portal/runbooks/tenant-budget-headroom-low.md
 M oya/finops-portal/runbooks/tenant-cost-anomaly-spike.md
 M oya/forms/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/forms/README.md
 M oya/forms/dpia/dpia.md
 M oya/forms/manifest.json
 M oya/forms/runbooks/auto-rebalance.md
 M oya/forms/runbooks/cold-merge.md
 M oya/forms/runbooks/hot-split.md
 M oya/global-trade/AUDIT-FINDINGS-2026-05-21.json
 M oya/global-trade/IPs/IP-ADR-0339-Shared-IaC-Modules.md
 M oya/global-trade/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/global-trade/README.md
 M oya/global-trade/capabilities/customs-declaration-command.yaml
 M oya/global-trade/capabilities/export-control-classification-export.yaml
 M oya/global-trade/capabilities/sanctions-screening-reconcile.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-api.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-application.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-broker-filing-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-api.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-application.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-customs-declaration-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-api.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-application.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-denied-party-hit-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-api.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-application.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-export-control-classification-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-api.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-application.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-sanctions-screening-worker.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-adapter.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-api.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-application.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-domain.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-governance.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-kernel.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-rest.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-usecase.yaml
 M oya/global-trade/catalog/oya-global-trade-trade-document-worker.yaml
 M oya/global-trade/cedar/policies.cedar
 M oya/global-trade/contracts/asyncapi-v1.yaml
 M oya/global-trade/contracts/global-trade-v1.proto
 M oya/global-trade/contracts/openapi-v1.yaml
 M oya/global-trade/dashboards/customs-declaration-health.json
 M oya/global-trade/dashboards/global-trade-overview.json
 M oya/global-trade/decisions/ADR-GT-001-sanctions-export-control-and-broker-filing-hold-state-machine.md
 M oya/global-trade/dpia/dpia.md
 M oya/global-trade/iac/ech-config.yaml
 M oya/global-trade/iac/edge-waf.yaml
 M oya/global-trade/iac/helm-values.yaml
 M oya/global-trade/iac/k8s-deployment.yaml
 M oya/global-trade/iac/k8s/helm/Chart.yaml
 M oya/global-trade/iac/k8s/helm/templates/cedar.yaml
 M oya/global-trade/iac/k8s/helm/templates/configmap.yaml
 M oya/global-trade/iac/k8s/helm/templates/deployment.yaml
 M oya/global-trade/iac/k8s/helm/templates/service.yaml
 M oya/global-trade/iac/k8s/helm/values.yaml
 M oya/global-trade/iac/network-policy.yaml
 M oya/global-trade/iac/openbao-policy.hcl
 M oya/global-trade/iac/pqc-cert.yaml
 M oya/global-trade/iac/secret-bindings.yaml
 M oya/global-trade/iac/terraform-module/main.tf
 M oya/global-trade/manifest.json
 M oya/global-trade/policy/abuse-defence.cedar
 M oya/global-trade/policy/auditor-scope.cedar
 M oya/global-trade/policy/broker-filing-authorization.cedar
 M oya/global-trade/policy/ci-scope.cedar
 M oya/global-trade/policy/customs-declaration-authorization.cedar
 M oya/global-trade/policy/denied-party-hit-authorization.cedar
 M oya/global-trade/policy/emergency-services-bypass.cedar
 M oya/global-trade/policy/export-control-classification-authorization.cedar
 M oya/global-trade/policy/pack-overlay-authorization.cedar
 M oya/global-trade/policy/sanctions-screening-authorization.cedar
 M oya/global-trade/policy/trade-document-authorization.cedar
 M oya/global-trade/runbooks/approval-deadletter.md
 M oya/global-trade/runbooks/auto-rebalance.md
 M oya/global-trade/runbooks/capacity-saturation.md
 M oya/global-trade/runbooks/cold-merge.md
 M oya/global-trade/runbooks/hot-split.md
 M oya/global-trade/runbooks/marketplace-settlement-blocked.md
 M oya/global-trade/runbooks/policy-deny-spike.md
 M oya/global-trade/runbooks/regional-failover.md
 M oya/global-trade/runbooks/source-import-stalled.md
 M oya/global-trade/scorecards/overrides.json
 M oya/global-trade/slos/autosharding-events.openslo.yaml
 M oya/global-trade/slos/customs-declaration-success-rate.openslo.yaml
 M oya/global-trade/slos/global-trade-availability.openslo.yaml
 M oya/global-trade/slos/global-trade-latency-p99.openslo.yaml
 M oya/global-trade/slos/global-trade-throughput.openslo.yaml
 M oya/governance/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/governance/README.md
 M oya/governance/dpia/dpia.md
 M oya/governance/manifest.json
 M oya/governance/runbooks/aggregation-rebuild.md
 M oya/governance/runbooks/audit-event-emission-stall.md
 M oya/governance/runbooks/auto-rebalance.md
 M oya/governance/runbooks/cedar-policy-rollback-protocol.md
 M oya/governance/runbooks/cold-merge.md
 M oya/governance/runbooks/consent-collection-pipeline-failure.md
 M oya/governance/runbooks/envoy-wasm-filter-rollback.md
 M oya/governance/runbooks/evidence-replay.md
 M oya/governance/runbooks/hot-split.md
 M oya/governance/runbooks/industry-baseline-refresh.md
 M oya/governance/runbooks/lane-bypass-emergency.md
 M oya/governance/runbooks/lane-failure-triage.md
 M oya/governance/runbooks/migration-execution.md
 M oya/governance/runbooks/wasm-filter-bytecode-quarantine.md
 M oya/healthcare-integration/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/healthcare-integration/README.md
 M oya/healthcare-integration/dpia/dpia.md
 M oya/healthcare-integration/runbooks/auto-rebalance.md
 M oya/healthcare-integration/runbooks/cold-merge.md
 M oya/healthcare-integration/runbooks/hot-split.md
 M oya/hr/contracts/openapi-v1.meta.yaml
 M oya/hr/contracts/openapi-v1.yaml
 M oya/hr/crates/oya-hr-employment-api/src/lib.rs
 M oya/hr/crates/oya-hr-employment-api/tests/contracts.rs
 M oya/hr/crates/oya-hr-employment-app/BUCK
 M oya/hr/crates/oya-hr-employment-app/Cargo.toml
 M oya/hr/crates/oya-hr-employment-app/src/lib.rs
 M oya/hr/crates/oya-hr-employment-app/tests/app_envelopes.rs
 M oya/hr/crates/oya-hr-employment-app/tests/leave.rs
 M oya/hr/crates/oya-hr-employment-app/tests/privacy.rs
 M oya/hr/crates/oya-hr-employment-domain/BUCK
 M oya/hr/crates/oya-hr-employment-domain/src/lib.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/leave_balance.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/leave_carryover_forfeiture.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/onboarding.rs
 M oya/hr/crates/oya-hr-employment-domain/tests/rulepack_manifest.rs
 M oya/hr/crates/oya-hr-employment-infrastructure/src/lib.rs
 M oya/hr/crates/oya-hr-employment-infrastructure/tests/runtime.rs
 M oya/identity/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/identity/PRD.md
 M oya/identity/README.md
 M oya/identity/crates/oya-identity-workload-oidc-adapter/src/eddsa.rs
 M oya/identity/crates/oya-identity-workload-oidc-adapter/src/lib.rs
 M oya/identity/crates/oya-identity-workload-rest/Cargo.toml
 M oya/identity/crates/oya-identity-workload-rest/tests/common/mod.rs
 M oya/identity/crates/oya-identity-workload-rest/tests/rest_endpoints.rs
 M oya/identity/dpia/dpia.md
 M oya/identity/manifest.json
 M oya/identity/runbooks/auto-rebalance.md
 M oya/identity/runbooks/brute-force-mitigation.md
 M oya/identity/runbooks/cold-merge.md
 M oya/identity/runbooks/hot-split.md
 M oya/identity/runbooks/idp-failover-drill.md
 M oya/identity/runbooks/ip-block-incident.md
 M oya/identity/runbooks/jwks-rotation.md
 M oya/identity/runbooks/passkey-cross-device-debug.md
 M oya/identity/runbooks/passkey-replay-attack-response.md
 M oya/identity/runbooks/passkey-reset.md
 M oya/identity/runbooks/recovery-key-mass-issue-investigation.md
 M oya/identity/runbooks/scim-provisioning-debug.md
 M oya/identity/runbooks/tenant-admin-onboard.md
 M oya/identity/runbooks/webauthn-rp-id-rotation.md
 M oya/imaging/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/imaging/README.md
 M oya/imaging/dpia/dpia.md
 M oya/imaging/runbooks/auto-rebalance.md
 M oya/imaging/runbooks/cold-merge.md
 M oya/imaging/runbooks/hot-split.md
 M oya/incident-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/incident-management/README.md
 M oya/incident-management/dpia/dpia.md
 M oya/incident-management/runbooks/auto-rebalance.md
 M oya/incident-management/runbooks/cold-merge.md
 M oya/incident-management/runbooks/hot-split.md
 M oya/intelligence/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/intelligence/README.md
 M oya/intelligence/_legacy-foundry/README.md
 M oya/intelligence/crates/oya-intelligence-dispatch-usecase/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-domain/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-kernel/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-usecase/src/lib.rs
 M oya/intelligence/crates/oya-intelligence-model-routing-usecase/tests/acceptance.rs
 M oya/intelligence/crates/oya-intelligence-subagent-runtime-app/src/main.rs
 M oya/intelligence/dpia/dpia.md
 M oya/intelligence/manifest.json
 M oya/intelligence/runbooks/assist-draft-policy-refusal.md
 M oya/intelligence/runbooks/audit-row-forgery-detected.md
 M oya/intelligence/runbooks/auto-rebalance.md
 M oya/intelligence/runbooks/byok-rotation-tenant-cascade.md
 M oya/intelligence/runbooks/cold-merge.md
 M oya/intelligence/runbooks/eu-ai-act-incident-notification.md
 M oya/intelligence/runbooks/hot-split.md
 M oya/intelligence/runbooks/model-inference-timeout-investigation.md
 M oya/intelligence/runbooks/model-router-stall-investigation.md
 M oya/intelligence/runbooks/prompt-fence-bypass-attempt-response.md
 M oya/intelligence/runbooks/prompt-fence-bypass-detection.md
 M oya/intelligence/runbooks/prompt-injection-detected.md
 M oya/intelligence/runbooks/provider-outage-anthropic.md
 M oya/intelligence/runbooks/provider-outage-google.md
 M oya/intelligence/runbooks/provider-outage-openai.md
 M oya/intelligence/runbooks/provider-rate-limit-saturation.md
 M oya/intelligence/runbooks/rag-corpus-drift-detection.md
 M oya/intelligence/runbooks/rag-retrieval-quality-regression.md
 M oya/intelligence/runbooks/refusal-false-positive-cascade.md
 M oya/intelligence/runbooks/sidecar-credential-handle-expired.md
 M oya/itsm/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/itsm/README.md
 M oya/itsm/dpia/dpia.md
 M oya/itsm/runbooks/auto-rebalance.md
 M oya/itsm/runbooks/cold-merge.md
 M oya/itsm/runbooks/hot-split.md
 M oya/learning-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/learning-management/README.md
 M oya/learning-management/dpia/dpia.md
 M oya/learning-management/runbooks/auto-rebalance.md
 M oya/learning-management/runbooks/cold-merge.md
 M oya/learning-management/runbooks/hot-split.md
 M oya/mail/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/mail/README.md
 M oya/mail/dpia/dpia.md
 M oya/mail/iac/helm/templates/networkpolicy.yaml
 M oya/mail/iac/helm/templates/service.yaml
 M oya/mail/iac/helm/values.yaml
 M oya/mail/manifest.json
 M oya/mail/runbooks/auto-rebalance.md
 M oya/mail/runbooks/cold-merge.md
 M oya/mail/runbooks/hot-split.md
 M oya/marketing-automation/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/marketing-automation/README.md
 M oya/marketing-automation/dpia/dpia.md
 M oya/marketing-automation/runbooks/auto-rebalance.md
 M oya/marketing-automation/runbooks/cold-merge.md
 M oya/marketing-automation/runbooks/hot-split.md
 M oya/marketplace/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/marketplace/README.md
 M oya/marketplace/dpia/dpia.md
 M oya/marketplace/runbooks/auto-rebalance.md
 M oya/marketplace/runbooks/buyer-order-double-submit.md
 M oya/marketplace/runbooks/cold-merge.md
 M oya/marketplace/runbooks/cross-border-tax-hold.md
 M oya/marketplace/runbooks/cross-tenant-buyer-seller-mediation-stall.md
 M oya/marketplace/runbooks/deal-acceptance-stalled.md
 M oya/marketplace/runbooks/deal-settlement-discrepancy-resolution.md
 M oya/marketplace/runbooks/dispute-escalation-protocol.md
 M oya/marketplace/runbooks/escrow-reservation-mismatch.md
 M oya/marketplace/runbooks/hot-split.md
 M oya/marketplace/runbooks/mediation-queue-saturation.md
 M oya/marketplace/runbooks/order-export-deadletter.md
 M oya/marketplace/runbooks/revenue-share-drift.md
 M oya/marketplace/runbooks/sanctions-screen-latency.md
 M oya/marketplace/runbooks/seller-onboarding-deny-spike.md
 M oya/marketplace/runbooks/settlement-ledger-replay.md
 M oya/meet/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/meet/README.md
 M oya/meet/dpia/dpia.md
 M oya/meet/manifest.json
 M oya/meet/runbooks/auto-rebalance.md
 M oya/meet/runbooks/cold-merge.md
 M oya/meet/runbooks/hot-split.md
 M oya/messenger/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/messenger/README.md
 M oya/messenger/crates/oya-messenger-message-stream-rest/src/lib.rs
 M oya/messenger/dpia/dpia.md
 M oya/messenger/manifest.json
 M oya/messenger/runbooks/auto-rebalance.md
 M oya/messenger/runbooks/cold-merge.md
 M oya/messenger/runbooks/hot-split.md
 M oya/notes/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/notes/README.md
 M oya/notes/dpia/dpia.md
 M oya/notes/manifest.json
 M oya/notes/runbooks/auto-rebalance.md
 M oya/notes/runbooks/cold-merge.md
 M oya/notes/runbooks/hot-split.md
 M oya/observability/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/observability/README.md
 M oya/observability/dpia/dpia.md
 M oya/observability/manifest.json
 M oya/observability/runbooks/auto-rebalance.md
 M oya/observability/runbooks/cold-merge.md
 M oya/observability/runbooks/hot-split.md
 M oya/ontology/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/ontology/PRD.md
 M oya/ontology/README.md
 M oya/ontology/dpia/dpia.md
 M oya/ontology/manifest.json
 M oya/ontology/runbooks/auto-rebalance.md
 M oya/ontology/runbooks/cedar-fragment-rollback.md
 M oya/ontology/runbooks/clickhouse-rebalance.md
 M oya/ontology/runbooks/cold-merge.md
 M oya/ontology/runbooks/cross-tenant-entity-collision-resolution.md
 M oya/ontology/runbooks/cross-tenant-leak-recovery.md
 M oya/ontology/runbooks/entity-projection-mismatch-recovery.md
 M oya/ontology/runbooks/graph-query-performance-regression.md
 M oya/ontology/runbooks/hot-split.md
 M oya/ontology/runbooks/object-type-deprecation.md
 M oya/ontology/runbooks/ontology-bot-score-recalibration.md
 M oya/ontology/runbooks/ontology-read-library-fallback.md
 M oya/ontology/runbooks/postgres-citus-rebalance.md
 M oya/ontology/runbooks/query-engine-restart.md
 M oya/ontology/runbooks/share-token-revocation.md
 M oya/ontology/runbooks/type-registry-migration.md
 M oya/ops-dashboard-control-center/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/ops-dashboard-control-center/README.md
 M oya/ops-dashboard-control-center/dpia/dpia.md
 M oya/ops-dashboard-control-center/iac/prod-spiffe-kill-switch.yaml
 M oya/ops-dashboard-control-center/runbooks/auto-rebalance.md
 M oya/ops-dashboard-control-center/runbooks/cold-merge.md
 M oya/ops-dashboard-control-center/runbooks/hot-split.md
 M oya/patient-monitoring/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/patient-monitoring/README.md
 M oya/patient-monitoring/dpia/dpia.md
 M oya/patient-monitoring/runbooks/auto-rebalance.md
 M oya/patient-monitoring/runbooks/cold-merge.md
 M oya/patient-monitoring/runbooks/hot-split.md
 M oya/payments/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/payments/PRD.md
 M oya/payments/README.md
 M oya/payments/crates/oya-payments-charge-domain/BUCK
 M oya/payments/crates/oya-payments-charge-domain/src/lib.rs
 M oya/payments/dpia/dpia.md
 M oya/payments/runbooks/aml-suspicious-activity-detected.md
 M oya/payments/runbooks/auto-rebalance.md
 M oya/payments/runbooks/chargeback-cascade-investigation.md
 M oya/payments/runbooks/cold-merge.md
 M oya/payments/runbooks/dispute-escalation.md
 M oya/payments/runbooks/double-charge-detected.md
 M oya/payments/runbooks/elder-financial-abuse.md
 M oya/payments/runbooks/fraud-spike-detected.md
 M oya/payments/runbooks/hot-split.md
 M oya/payments/runbooks/kr-fss-audit-pull.md
 M oya/payments/runbooks/kyc-aml-screening-pipeline-stall.md
 M oya/payments/runbooks/payout-failed.md
 M oya/payments/runbooks/pci-incident-response.md
 M oya/payments/runbooks/psp-failover-cascade-execution.md
 M oya/payments/runbooks/psp-outage.md
 M oya/payments/runbooks/refund-mismatch.md
 M oya/payroll/README.md
 M oya/payroll/catalog/oya-payroll-run-api.yaml
 M oya/payroll/catalog/oya-payroll-run-app.yaml
 M oya/payroll/catalog/oya-payroll-run-domain.yaml
 M oya/payroll/contracts/openapi-v1.meta.yaml
 M oya/payroll/contracts/openapi-v1.yaml
 M oya/payroll/crates/oya-payroll-run-api/BUCK
 M oya/payroll/crates/oya-payroll-run-api/Cargo.toml
 M oya/payroll/crates/oya-payroll-run-api/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-app/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-domain/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-domain/tests/rollback.rs
 M oya/payroll/crates/oya-payroll-run-infrastructure/BUCK
 M oya/payroll/crates/oya-payroll-run-infrastructure/src/lib.rs
 M oya/payroll/crates/oya-payroll-run-infrastructure/tests/runtime.rs
 M oya/performance-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/performance-management/README.md
 M oya/performance-management/dpia/dpia.md
 M oya/performance-management/runbooks/auto-rebalance.md
 M oya/performance-management/runbooks/cold-merge.md
 M oya/performance-management/runbooks/hot-split.md
 M oya/pharmacy/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/pharmacy/README.md
 M oya/pharmacy/dpia/dpia.md
 M oya/pharmacy/runbooks/auto-rebalance.md
 M oya/pharmacy/runbooks/cold-merge.md
 M oya/pharmacy/runbooks/hot-split.md
 M oya/plant-maintenance/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/plant-maintenance/README.md
 M oya/plant-maintenance/contracts/asyncapi-v1.yaml
 M oya/plant-maintenance/contracts/openapi-v1.yaml
 M oya/plant-maintenance/contracts/plant-maintenance-v1.proto
 M oya/plant-maintenance/crates/oya-plant-maintenance-domain/tests/plant_maintenance.rs
 M oya/plant-maintenance/crates/oya-plant-maintenance-work-order-app/tests/integration.rs
 M oya/plant-maintenance/dpia/dpia.md
 M oya/plant-maintenance/manifest.json
 M oya/plant-maintenance/runbooks/auto-rebalance.md
 M oya/plant-maintenance/runbooks/cold-merge.md
 M oya/plant-maintenance/runbooks/hot-split.md
 M oya/plugin-app-store/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/plugin-app-store/README.md
 M oya/plugin-app-store/dpia/dpia.md
 M oya/plugin-app-store/runbooks/auto-rebalance.md
 M oya/plugin-app-store/runbooks/cold-merge.md
 M oya/plugin-app-store/runbooks/hot-split.md
 M oya/policy/crates/oya-policy-cedar-domain/BUCK
 M oya/policy/crates/oya-policy-cedar-domain/src/lib.rs
 M oya/policy/crates/oya-policy-cedar-domain/src/policy_diff.rs
 M oya/production-planning/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/production-planning/README.md
 M oya/production-planning/crates/oya-production-planning-domain/tests/production_planning.rs
 M oya/production-planning/dpia/dpia.md
 M oya/production-planning/manifest.json
 M oya/production-planning/runbooks/auto-rebalance.md
 M oya/production-planning/runbooks/cold-merge.md
 M oya/production-planning/runbooks/hot-split.md
 M oya/quality-management/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/quality-management/README.md
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/asyncapi.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/grpc.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/http.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/mod.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/src/domain/mod.rs
 M oya/quality-management/crates/oya-quality-management-inspection-app/tests/integration.rs
 M oya/quality-management/dpia/dpia.md
 M oya/quality-management/manifest.json
 M oya/quality-management/runbooks/auto-rebalance.md
 M oya/quality-management/runbooks/cold-merge.md
 M oya/quality-management/runbooks/hot-split.md
 M oya/real-estate/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/real-estate/README.md
 M oya/real-estate/crates/oya-real-estate-portfolio-domain/src/lib.rs
 M oya/real-estate/crates/oya-real-estate-portfolio-domain/tests/real_estate_portfolio.rs
 M oya/real-estate/dpia/dpia.md
 M oya/real-estate/manifest.json
 M oya/real-estate/runbooks/auto-rebalance.md
 M oya/real-estate/runbooks/cold-merge.md
 M oya/real-estate/runbooks/hot-split.md
 M oya/recordings/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/recordings/README.md
 M oya/recordings/crates/oya-recordings-domain/src/lib.rs
 M oya/recordings/dpia/dpia.md
 M oya/recordings/manifest.json
 M oya/recordings/runbooks/auto-rebalance.md
 M oya/recordings/runbooks/cold-merge.md
 M oya/recordings/runbooks/hot-split.md
 M oya/sheets/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/sheets/README.md
 M oya/sheets/dpia/dpia.md
 M oya/sheets/manifest.json
 M oya/sheets/runbooks/auto-rebalance.md
 M oya/sheets/runbooks/cold-merge.md
 M oya/sheets/runbooks/hot-split.md
 M oya/sites/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/sites/README.md
 M oya/sites/dpia/dpia.md
 M oya/sites/manifest.json
 M oya/sites/runbooks/auto-rebalance.md
 M oya/sites/runbooks/cold-merge.md
 M oya/sites/runbooks/hot-split.md
 M oya/slides/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/slides/README.md
 M oya/slides/dpia/dpia.md
 M oya/slides/manifest.json
 M oya/slides/runbooks/auto-rebalance.md
 M oya/slides/runbooks/cold-merge.md
 M oya/slides/runbooks/hot-split.md
 M oya/social/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/social/README.md
 M oya/social/dpia/dpia.md
 M oya/social/manifest.json
 M oya/social/runbooks/auto-rebalance.md
 M oya/social/runbooks/cold-merge.md
 M oya/social/runbooks/dr-failover.md
 M oya/social/runbooks/hot-split.md
 M oya/supply-chain-planning/IPs/IP-ADR-0339-Shared-IaC-Modules.md
 M oya/supply-chain-planning/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/supply-chain-planning/README.md
 M oya/supply-chain-planning/capabilities/available-to-promise-export.yaml
 M oya/supply-chain-planning/capabilities/demand-plan-command.yaml
 M oya/supply-chain-planning/capabilities/supply-network-plan-reconcile.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-available-to-promise-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-demand-plan-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-planning-scenario-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-replenishment-plan-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-supply-network-plan-worker.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-adapter.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-api.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-application.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-domain.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-governance.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-kernel.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-rest.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-usecase.yaml
 M oya/supply-chain-planning/catalog/oya-supply-chain-planning-transportation-plan-worker.yaml
 M oya/supply-chain-planning/cedar/policies.cedar
 M oya/supply-chain-planning/contracts/asyncapi-v1.yaml
 M oya/supply-chain-planning/contracts/openapi-v1.yaml
 M oya/supply-chain-planning/contracts/supply-chain-planning-v1.proto
 M oya/supply-chain-planning/crates/oya-supply-chain-planning-domain/tests/supply_chain_planning.rs
 M oya/supply-chain-planning/crates/oya-supply-chain-planning-network-app/tests/integration.rs
 M oya/supply-chain-planning/dashboards/demand-plan-health.json
 M oya/supply-chain-planning/dashboards/supply-chain-planning-overview.json
 M oya/supply-chain-planning/dpia/dpia.md
 M oya/supply-chain-planning/iac/ech-config.yaml
 M oya/supply-chain-planning/iac/edge-waf.yaml
 M oya/supply-chain-planning/iac/helm-values.yaml
 M oya/supply-chain-planning/iac/k8s-deployment.yaml
 M oya/supply-chain-planning/iac/k8s/helm/Chart.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/cedar.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/configmap.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/deployment.yaml
 M oya/supply-chain-planning/iac/k8s/helm/templates/service.yaml
 M oya/supply-chain-planning/iac/k8s/helm/values.yaml
 M oya/supply-chain-planning/iac/network-policy.yaml
 M oya/supply-chain-planning/iac/openbao-policy.hcl
 M oya/supply-chain-planning/iac/pqc-cert.yaml
 M oya/supply-chain-planning/iac/secret-bindings.yaml
 M oya/supply-chain-planning/iac/terraform-module/main.tf
 M oya/supply-chain-planning/manifest.json
 M oya/supply-chain-planning/runbooks/approval-deadletter.md
 M oya/supply-chain-planning/runbooks/auto-rebalance.md
 M oya/supply-chain-planning/runbooks/capacity-saturation.md
 M oya/supply-chain-planning/runbooks/cold-merge.md
 M oya/supply-chain-planning/runbooks/hot-split.md
 M oya/supply-chain-planning/runbooks/marketplace-settlement-blocked.md
 M oya/supply-chain-planning/runbooks/policy-deny-spike.md
 M oya/supply-chain-planning/runbooks/regional-failover.md
 M oya/supply-chain-planning/runbooks/source-import-stalled.md
 M oya/supply-chain-planning/slos/autosharding-events.openslo.yaml
 M oya/supply-chain-planning/slos/demand-plan-success-rate.openslo.yaml
 M oya/supply-chain-planning/slos/supply-chain-planning-availability.openslo.yaml
 M oya/supply-chain-planning/slos/supply-chain-planning-latency-p99.openslo.yaml
 M oya/supply-chain-planning/slos/supply-chain-planning-throughput.openslo.yaml
 M oya/tasks/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/tasks/README.md
 M oya/tasks/dpia/dpia.md
 M oya/tasks/manifest.json
 M oya/tasks/runbooks/auto-rebalance.md
 M oya/tasks/runbooks/cold-merge.md
 M oya/tasks/runbooks/hot-split.md
 M oya/tenant-rbac/contracts/openapi-v1.meta.yaml
 M oya/tenant-rbac/crates/oya-tenant-rbac-api/src/lib.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-api/tests/contracts.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/BUCK
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/Cargo.toml
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/src/lib.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-app/tests/runtime.rs
 M oya/tenant-rbac/crates/oya-tenant-rbac-domain/src/lib.rs
 M oya/translate/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/translate/README.md
 M oya/translate/dpia/dpia.md
 M oya/translate/manifest.json
 M oya/translate/runbooks/auto-rebalance.md
 M oya/translate/runbooks/cold-merge.md
 M oya/translate/runbooks/hot-split.md
 M oya/treasury/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/treasury/README.md
 M oya/treasury/cedar/policies.cedar
 M oya/treasury/contracts/asyncapi-v1.yaml
 M oya/treasury/contracts/openapi-v1.yaml
 M oya/treasury/contracts/treasury-v1.proto
 M oya/treasury/crates/oya-treasury-cash-domain/BUCK
 M oya/treasury/crates/oya-treasury-cash-domain/src/lib.rs
 M oya/treasury/crates/oya-treasury-cash-domain/tests/cash_position.rs
 M oya/treasury/dashboards/cash-position-health.json
 M oya/treasury/dashboards/treasury-overview.json
 M oya/treasury/dpia/dpia.md
 M oya/treasury/iac/ech-config.yaml
 M oya/treasury/iac/edge-waf.yaml
 M oya/treasury/iac/helm-values.yaml
 M oya/treasury/iac/k8s-deployment.yaml
 M oya/treasury/iac/k8s/helm/Chart.yaml
 M oya/treasury/iac/k8s/helm/templates/cedar.yaml
 M oya/treasury/iac/k8s/helm/templates/configmap.yaml
 M oya/treasury/iac/k8s/helm/templates/deployment.yaml
 M oya/treasury/iac/k8s/helm/templates/service.yaml
 M oya/treasury/iac/k8s/helm/values.yaml
 M oya/treasury/iac/network-policy.yaml
 M oya/treasury/iac/openbao-policy.hcl
 M oya/treasury/iac/pqc-cert.yaml
 M oya/treasury/iac/secret-bindings.yaml
 M oya/treasury/iac/terraform-module/main.tf
 M oya/treasury/manifest.json
 M oya/treasury/policy/auditor-scope.cedar
 M oya/treasury/runbooks/approval-deadletter.md
 M oya/treasury/runbooks/auto-rebalance.md
 M oya/treasury/runbooks/capacity-saturation.md
 M oya/treasury/runbooks/cold-merge.md
 M oya/treasury/runbooks/hot-split.md
 M oya/treasury/runbooks/marketplace-settlement-blocked.md
 M oya/treasury/runbooks/policy-deny-spike.md
 M oya/treasury/runbooks/regional-failover.md
 M oya/treasury/runbooks/source-import-stalled.md
 M oya/treasury/slos/autosharding-events.openslo.yaml
 M oya/treasury/slos/cash-position-success-rate.openslo.yaml
 M oya/treasury/slos/treasury-availability.openslo.yaml
 M oya/treasury/slos/treasury-latency-p99.openslo.yaml
 M oya/treasury/slos/treasury-throughput.openslo.yaml
 M oya/warehouse/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/warehouse/README.md
 M oya/warehouse/crates/oya-warehouse-inventory-domain/src/lib.rs
 M oya/warehouse/crates/oya-warehouse-inventory-domain/tests/inventory.rs
 M oya/warehouse/dpia/dpia.md
 M oya/warehouse/manifest.json
 M oya/warehouse/runbooks/auto-rebalance.md
 M oya/warehouse/runbooks/cold-merge.md
 M oya/warehouse/runbooks/hot-split.md
 M oya/whiteboard/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/whiteboard/README.md
 M oya/whiteboard/dpia/dpia.md
 M oya/whiteboard/runbooks/auto-rebalance.md
 M oya/whiteboard/runbooks/cold-merge.md
 M oya/whiteboard/runbooks/hot-split.md
 M oya/workflow-engine/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/workflow-engine/PRD.md
 M oya/workflow-engine/README.md
 M oya/workflow-engine/crates/oya-workflow-engine-execution-engine-sdk/src/lib.rs
 M oya/workflow-engine/crates/oya-workflow-engine-execution-engine-usecase/src/lib.rs
 M oya/workflow-engine/dpia/dpia.md
 M oya/workflow-engine/manifest.json
 M oya/workflow-engine/runbooks/auto-rebalance.md
 M oya/workflow-engine/runbooks/cold-merge.md
 M oya/workflow-engine/runbooks/deadlock-resolution.md
 M oya/workflow-engine/runbooks/durable-execution-history-replay.md
 M oya/workflow-engine/runbooks/durable-execution-restart.md
 M oya/workflow-engine/runbooks/event-bus-replay.md
 M oya/workflow-engine/runbooks/hot-split.md
 M oya/workflow-engine/runbooks/saga-compensation-failure-investigation.md
 M oya/workflow-engine/runbooks/spec-rollback.md
 M oya/workflow-engine/runbooks/stuck-workflow-recovery.md
 M oya/workflow-engine/runbooks/valkey-failover.md
 M oya/workflow-engine/runbooks/workflow-state-corruption-recovery.md
 M oya/workflow-studio/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M oya/workflow-studio/README.md
 M oya/workflow-studio/dpia/dpia.md
 M oya/workflow-studio/manifest.json
 M oya/workflow-studio/runbooks/ai-assisted-generation-quality-regression.md
 M oya/workflow-studio/runbooks/auto-rebalance.md
 M oya/workflow-studio/runbooks/canvas-perf-regression-debug.md
 M oya/workflow-studio/runbooks/canvas-perf-regression.md
 M oya/workflow-studio/runbooks/cold-merge.md
 M oya/workflow-studio/runbooks/collab-conflict-resolution.md
 M oya/workflow-studio/runbooks/collaborative-canvas-merge-conflict.md
 M oya/workflow-studio/runbooks/copilot-degraded-fallback.md
 M oya/workflow-studio/runbooks/crdt-merge-conflict.md
 M oya/workflow-studio/runbooks/hot-split.md
 M oya/workflow-studio/runbooks/node-graph-validation-failure.md
 M oya/workflow-studio/runbooks/presence-disconnect.md
 M oya/workflow-studio/runbooks/run-history-replay-corruption.md
 M oya/workflow-studio/runbooks/session-storm-throttle.md
 M oya/workflow-studio/runbooks/template-marketplace-quarantine.md
 M oya/workplace-integration/IPs/IP-WAVE-15-ZD-sharding-automation.md
 M registry/artifact-capabilities-registry.json
 M registry/catalog/oya-payroll-run-api.yaml
 M registry/catalog/oya-payroll-run-app.yaml
 M registry/catalog/oya-payroll-run-domain.yaml
 M registry/catalog/oya-payroll-run-infrastructure.yaml
 M registry/catalog/oya-payroll-run-storage-adapter-inmemory.yaml
 M registry/dependency-rationales.json
 M registry/generated-artifact-control-plane.json
 M registry/placeholder-debt/adr-follow-ups.yaml
 M scripts/hooks/pre-push.sh
 M scripts/tests/cloud_observability_slo_evidence_check.py
 M specs/agent-durable-goal.json
 M specs/agent-operating-contract.json
 M specs/agentic-slo-gated-promotion.json
 M specs/audit-event-class-registry.json
 M specs/audit-event-schema.json
 M specs/bespoke-cloud-toolchain-services.json
 M specs/cedar-policy-schema.json
 M specs/chaos-engineering-substrate-canonical.json
 M specs/ci-farm-substrate-canonical.json
 M specs/ci-fix-loop-context-bundle.json
 M specs/cloud-hyperscaler-parity-taxonomy.json
 M specs/cloud-observability-slo-evidence-contract.json
 M specs/cloud-strangler-migration-target.json
 M specs/cloud-toolchain-target.json
 M specs/compliance-pack-floors.json
 M specs/compliance-pack-schema.json
 M specs/csi-storage-class-canonical.json
 M specs/deployment-ops-contract.json
 M specs/design-spec-maturity-claims.json
 M specs/design-system/audit-evidence-timeline.json
 M specs/design-system/catalog.json
 M specs/design-system/cloud-cell-topology-map.json
 M specs/design-system/communication-thread-list.json
 M specs/design-system/entity-action-policy-preview.json
 M specs/design-system/foundry-agent-run-timeline.json
 M specs/design-system/ontology-graph-explorer.json
 M specs/design-system/ops-deployment-status-panel.json
 M specs/design-system/policy-disclosure-banner.json
 M specs/design-system/score-card-result-table.json
 M specs/design-system/spec-diff-viewer.json
 M specs/design-system/tenant-context-switcher.json
 M specs/design-system/workflow-canvas.json
 M specs/design-system/workflow-node-config-panel.json
 M specs/design-system/workflow-replay-timeline.json
 M specs/feature-flag-substrate-canonical.json
 M specs/finops-dimensional-model.json
 M specs/gitops-vcs-replacement.json
 M specs/hyperscaler-architecture-invariants.json
 M specs/hyperscaler-gates.json
 M specs/markdown-retirement-policy.json
 M specs/master-plan-sequencing.json
 M specs/masterplan.json
 M specs/merge-queue-parked-pr.json
 M specs/microservice-migration-tooling.json
 M specs/microservices/accounting.json
 M specs/microservices/anonymous.json
 M specs/microservices/calendar.json
 M specs/microservices/crm.json
 M specs/microservices/global-trade.json
 M specs/microservices/hr.json
 M specs/microservices/intelligence.json
 M specs/microservices/mail.json
 M specs/microservices/manifest-schema.json
 M specs/microservices/manifests-index.json
 M specs/microservices/messenger.json
 M specs/microservices/ontology.json
 M specs/microservices/payroll.json
 M specs/microservices/plant-maintenance.json
 M specs/microservices/procurement.json
 M specs/microservices/production-planning.json
 M specs/microservices/quality-management.json
 M specs/microservices/real-estate.json
 M specs/microservices/social.json
 M specs/microservices/supply-chain-planning.json
 M specs/microservices/tenant-rbac.json
 M specs/microservices/treasury.json
 M specs/microservices/warehouse.json
 M specs/microservices/workflow-studio.json
 M specs/microservices/workflow.json
 M specs/multi-region-disposition-canonical.json
 M specs/ontology-projection-schema.json
 M specs/oyatie-doctrine.json
 M specs/pack-overlay-schema.json
 M specs/per-tenant-audit-log-slicing-canonical.json
 M specs/plan-schema.json
 M specs/planning-closure-contract.json
 M specs/planning-closure-status-closure-ledger.json
 M specs/platform-architecture.json
 M specs/repo-hygiene-automation.json
 M specs/root-hub-pointers.json
 M specs/schema-registry-canonical.json
 M specs/score-cards.json
 M specs/sovereign-cloud-air-gapped-canonical.json
 M specs/tenant-environment-tiers-canonical.json
 M specs/toolchain-tenant-isolation-fixtures.json
 M specs/workspace-hygiene.json
 M templates/INDEX.md
 M templates/checklists/done-definition-checklist.md
 M templates/checklists/per-implementation-plan-checklist.md
 M templates/checklists/pr-review-checklist.md
 M templates/checklists/release-readiness-checklist.md
 M templates/implementation-plan-template.md
 M templates/pull-request-template.md
?? .agents/
?? .claude/skills/
?? .github/CODE_OF_CONDUCT.md
?? .github/CONTRIBUTING.md
?? .github/ISSUE_TEMPLATE/
?? .github/OWNERS
?? .github/PULL_REQUEST_TEMPLATE.md
?? .hermes/
?? .ouroboros/
?? .ouroboros_eval_artifact.md
?? .worktrees/
?? cloud/cloud-billing/crates/oya-cloud-billing-domain/tests/env_tier_outbound_metadata.rs
?? cloud/cloud-capacity/manifest.json
?? cloud/cloud-ci/gates/oya-cloud-ci-license-policy-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-load-balancer-inventory-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-multi-region-disposition-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-sovereign-tenant-pin-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-tenant-environment-tier-app/
?? cloud/cloud-ci/gates/oya-cloud-ci-zero-static-secrets-app/
?? cloud/cloud-iac/IPs/IP-sustainability-emission-model.md
?? cloud/cloud-iac/cell-topology/cell-001-contract-snapshot.json
?? cloud/cloud-intelligence/IPs/
?? cloud/cloud-intelligence/contracts/env-tier-gateway-budget-contract.json
?? cloud/cloud-network/crates/oya-cloud-network-domain/tests/cloud_network_resource_contract.rs
?? cloud/cloud-os/crates/oya-cloud-os-trustd-domain/src/persistence.rs
?? cloud/cloud-secrets/contracts/cloud-secrets-resource-contract.json
?? cloud/cloud-secrets/contracts/cloud-secrets-resource-contract.md
?? cloud/cloud-secrets/contracts/secretprovider-rotation-contract.md
?? cloud/cloud-secrets/crates/oya-secrets-domain/tests/cloud_secrets_resource_contract.rs
?? cloud/cloud-secrets/runbooks/non-prod-secretprovider-rotation-drill.md
?? cloud/managed-k8s-cluster-lifecycle/IPs/IP-sustainability-emission-model.md
?? cloud/managed-k8s-control-plane-host/IPs/IP-sustainability-emission-model.md
?? cloud/managed-k8s-sla-observability/IPs/IP-sustainability-emission-model.md
?? cloud/managed-k8s-sla-observability/runbooks/runbooks/sla-observation-store-unavailable.md
?? cloud/managed-k8s-tenant-quota/IPs/
?? cloud/managed-k8s-tenant-quota/crates/oya-managed-k8s-tenant-quota-adapter-postgres/
?? cloud/tenancy/IPs/IP-sustainability-emission-model.fixture.json
?? docs/audits/trust-center-security-privacy-docs-review-packet-2026-07-01.md
?? docs/ideas/ecosystem-as-code.md
?? docs/ideas/policy-pack-substrate.md
?? docs/runbooks/cloud/root-of-trust-ceremony.md
?? docs/standards/autonomous-kanban-lifecycle.md
?? evidence/audits/audit-002-retrieval-assumptions-contract-snapshot-2026-07-01.md
?? evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json
?? evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.md
?? evidence/cell-topology/
?? evidence/cloud/
?? evidence/conformance/
?? evidence/contract-snapshots/
?? evidence/devportal/
?? evidence/multispectrum/arch-own-ratchet-001-20260701-1782886584.json
?? evidence/multispectrum/founder-call-own-ops-001-20260702-1782954777.json
?? evidence/multispectrum/regsec-001-vulnerability-intelligence-sbom-vex-20260701.json
?? evidence/multispectrum/regvuln-002-vulnerability-contract-integration-decision-20260701.json
?? evidence/multispectrum/t_3acb7585-qk05-review-fix-1782943388.json
?? evidence/multispectrum/t_5f5d9a01-shell-004-audit-evidence-timeline-1783612741.json
?? evidence/multispectrum/t_f60cb75d-finops-high-risk-emission-models-1782944098.json
?? evidence/multispectrum/t_ff6ecba7-manifest-index-inventory-recon-1782912252.json
?? evidence/multispectrum/w4-003-virtual-materialization-20260701.json
?? evidence/observability/
?? evidence/regulatory/
?? evidence/toolchain-isolation/
?? infra/capi/.gitignore
?? infra/capi/fleet-preflight.sh
?? libs/oya-ci-config/fixtures/
?? libs/oya-ci-gate-contract/BUCK
?? libs/oya-data-boundary-kernel/fixtures/
?? libs/oya-data-sql-adapter-sqlx/src/envelope.rs
?? oya/api-gateway/iac/k8s/helm/templates/mail-protocol-routes.yaml
?? oya/application/IPs/IP-sustainability-emission-model.md
?? oya/audit-chain/IPs/IP-sustainability-emission-model.fixture.json
?? oya/calendar/IPs/IP-sustainability-emission-model.md
?? oya/community/IPs/IP-sustainability-emission-model.md
?? oya/community/crates/oya-community-anonymous/
?? oya/connector/crates/oya-connector-slack-adapter/tests/
?? oya/developer-sdk/crates/oya-dev-cli/src/terminal_verifier_harness.rs
?? oya/docs/IPs/IP-sustainability-emission-model.md
?? oya/drive/IPs/IP-sustainability-emission-model.md
?? oya/finops-portal/contracts/env-tier-outbound-emission-plan.contract.json
?? oya/finops-portal/contracts/fixtures/
?? oya/forms/IPs/IP-sustainability-emission-model.md
?? oya/governance/IPs/IP-sustainability-emission-model.fixture.json
?? oya/hr/crates/oya-hr-employment-domain/tests/statutory_filing_manifest.rs
?? oya/hr/crates/oya-hr-employment-storage-adapter-postgres/
?? oya/identity/IPs/IP-sustainability-emission-model.fixture.json
?? oya/intelligence/IPs/IP-sustainability-emission-model.fixture.json
?? oya/intelligence/contracts/env-tier-model-budget-contract.json
?? oya/intelligence/contracts/fixtures/
?? oya/mail/IPs/IP-sustainability-emission-model.md
?? oya/mail/iac/helm/templates/_helpers.tpl
?? oya/mail/iac/helm/templates/ciliumnetworkpolicy-mail-edge.yaml
?? oya/meet/IPs/IP-sustainability-emission-model.md
?? oya/messenger/IPs/IP-sustainability-emission-model.fixture.json
?? oya/notes/IPs/IP-sustainability-emission-model.md
?? oya/observability/IPs/IP-sustainability-emission-model.md
?? oya/ontology/IPs/IP-sustainability-emission-model.fixture.json
?? oya/ops-dashboard-control-center/IPs/IP-sustainability-emission-model.md
?? oya/payments/crates/oya-payments-charge-domain/tests/
?? oya/payroll/crates/oya-payroll-run-api/tests/hr_payroll_boundary.rs
?? oya/payroll/crates/oya-payroll-run-api/tests/statutory_preview.rs
?? oya/payroll/crates/oya-payroll-run-domain/tests/statutory_calculation.rs
?? oya/payroll/crates/oya-payroll-run-domain/tests/statutory_source_pack.rs
?? oya/payroll/crates/oya-payroll-run-domain/tests/year_end_settlement.rs
?? oya/payroll/crates/oya-payroll-run-infrastructure/tests/local_close_replay.rs
?? oya/payroll/crates/oya-payroll-run-infrastructure/tests/statutory_replay.rs
?? oya/payroll/crates/oya-payroll-run-storage-adapter-postgres/
?? oya/policy/crates/oya-policy-cedar-domain/src/rebac.rs
?? oya/policy/crates/oya-policy-cedar-domain/tests/
?? oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/cedar.rs
?? oya/quality-management/crates/oya-quality-management-inspection-app/src/adapter/repository.rs
?? oya/recordings/IPs/IP-sustainability-emission-model.md
?? oya/sheets/IPs/IP-sustainability-emission-model.md
?? oya/sites/IPs/IP-sustainability-emission-model.md
?? oya/slides/IPs/IP-sustainability-emission-model.md
?? oya/social/IPs/IP-sustainability-emission-model.md
?? oya/social/crates/
?? oya/supply-chain-planning/iac/PROVENANCE-INVENTORY.md
?? oya/tasks/IPs/IP-sustainability-emission-model.md
?? oya/translate/IPs/IP-sustainability-emission-model.md
?? oya/treasury/crates/oya-treasury-cash-domain/tests/env_tier_outbound_metadata.rs
?? oya/trust/
?? oya/workflow-engine/IPs/IP-sustainability-emission-model.fixture.json
?? oya/workflow-engine/contracts/env-tier-run-handoff-contract.yaml
?? oya/workflow-engine/crates/oya-workflow-engine-execution-engine-usecase/tests/
?? oya/workflow-engine/policy/env-tier-run-start.cedar
?? oya/workflow-studio/IPs/IP-sustainability-emission-model.md
?? oya/workflow-studio/contracts/env-tier-promotion-contract.yaml
?? oya/workflow-studio/policy/env-tier-promotion.cedar
?? oya/workplace-integration/contracts/env-tier-outbound-emission-plan.contract.json
?? oya/workplace-integration/contracts/fixtures/
?? oya/workplace-integration/crates/oya-workplace-integration-outbound-metadata-domain/
?? plan/cloud-quality-kits/
?? plan/community/
?? plan/compliance-selective-cell-placement-architecture.md
?? plan/tasks/
?? registry/lts-pins.yaml
?? scripts/tests/anonymous_prd_red_fixture_contract_check.py
?? scripts/tests/calendar_prd_red_fixture_contract_check.py
?? scripts/tests/calendar_user_story_red_fixture_check.py
?? scripts/tests/community_fd001_red_fixture_contract_check.py
?? scripts/tests/conf_001_hyperscaler_conformance_check.py
?? scripts/tests/finops_collab_emission_models_check.py
?? scripts/tests/global_trade_inventory_authority_check.py
?? scripts/tests/hr_cloud_deployment_evidence_plan_check.py
?? scripts/tests/hr_group_ops_scale_plan_check.py
?? scripts/tests/hr_runtime_audit_event_registry_contract_check.py
?? scripts/tests/meet_source_map_contract_replay_check.py
?? scripts/tests/payroll_audit_event_registry_contract_check.py
?? scripts/tests/qk_01_overload_fairness_future_harness_check.py
?? scripts/tests/qk_02_shuffle_shard_isolation_future_harness_check.py
?? scripts/tests/qk_03_privacy_data_governance_future_harness_check.py
?? scripts/tests/qk_04_canary_prr_future_harness_check.py
?? scripts/tests/qk_05_focus_cost_export_future_harness_check.py
?? scripts/tests/qk_06_k8s_pod_security_future_harness_check.py
?? scripts/tests/qk_07_abuse_fraud_ddos_future_harness_check.py
?? scripts/tests/recordings_prd_red_fixture_contract_check.py
?? scripts/tests/sheets_source_map_authority_check.py
?? scripts/tests/slides_prd_red_fixture_contract_check.py
?? scripts/tests/social_prd_red_fixture_contract_check.py
?? scripts/tests/tasks_red_fixture_contract_check.py
?? scripts/tests/tls_001_ech_runtime_reconciliation_check.py
?? scripts/tests/translate_source_map_authority_check.py
?? skills-lock.json
?? specs/bespoke-scm-virtual-materialization-plan.json
?? specs/compliance-security-radar-cadence-contract.json
?? specs/dogfood/
?? specs/fixtures/anonymous-prd/
?? specs/fixtures/calendar-prd/
?? specs/fixtures/community-fd001/
?? specs/fixtures/crate-adr-design-doc-coverage/
?? specs/fixtures/drive/
?? specs/fixtures/hr-cloud-deployment/
?? specs/fixtures/hr-group-ops-scale/
?? specs/fixtures/meet/
?? specs/fixtures/passkey-kyc-policy-context/
?? specs/fixtures/recordings-prd/
?? specs/fixtures/sheets/
?? specs/fixtures/slides/
?? specs/fixtures/social/
?? specs/fixtures/tasks/
?? specs/fixtures/translate/
?? specs/microservices/community.json
?? specs/microservices/drive.json
?? specs/microservices/forms.json
?? specs/microservices/meet.json
?? specs/microservices/recordings.json
?? specs/microservices/sheets.json
?? specs/microservices/sites.json
?? specs/microservices/slides.json
?? specs/microservices/tasks.json
?? specs/microservices/translate.json
?? specs/passkey-kyc-policy-context-contract.json
?? specs/pci-dss-level-1-readiness-plan.json
?? specs/platform-vertical-status.json
?? specs/portfolio-ownership-ratchet.json
?? specs/regulatory-identity-kyc-policy-evidence-architecture.json
?? specs/regulatory-identity-source-of-truth.json
?? specs/regulatory-legacy-gate-scanner-normalization-table.json
?? specs/reorg/
?? specs/root-of-trust-ceremony-contract.json
?? specs/security-validation-pipeline-matrix.json
?? specs/sre-operations-center-contract.json
?? specs/structural-blockers.json
?? specs/trust-center-compliance-evidence-portal.json
?? specs/vulnerability-intelligence-sbom-vex-pipeline.json
c52bdb09ea337de103b05317de0c120f2b7a3e45
d7568c731b5ef5ec01974da87f186d804748b44b
424
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '395,515p;620,915p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   395	      # `rustup::utils::rename ... No such file or directory (os error 2)` (a different component
   396	      # each run: clippy, then a second toolchain component — proving a concurrency race, not a config defect). rustup
   397	      # is not concurrency-safe. Installing the toolchain once here makes it ambient so the
   398	      # parallel actions find it already present (no download).
   399	      - name: Pre-provision pinned rust toolchain (serialize rustup before parallel buck2)
   400	        run: |
   401	          set -euo pipefail
   402	          rustup toolchain install
   403	          rustc --version
   404	      # Restore buck-out across runs so ephemeral runners start warm (design §3.1 / ADR-0515 D4).
   405	      #
   406	      # WHAT IS WARMED, PRECISELY (ADR-0554 D9, round-5): 100% of buck2 cross-run warmth lives in
   407	      # ./buck-out — buck-out/v2/cache/{materializer_state,incremental_state}/db.sqlite plus
   408	      # buck-out/v2/art (the materialized action outputs). buck-out is PATH-RELOCATABLE (relative
   409	      # paths + --remap-cwd-prefix=., no absolute project root baked in; a restored hit is keyed
   410	      # only on buck2_revision + os + arch), so restoring it into any runner checkout is sound.
   411	      # ~/.buck2 and ~/.buck hold ONLY daemon pid/endpoint/log scratch — ZERO action results — so
   412	      # caching `path: buck-out` warms everything cacheable and ~/.buck2/~/.buck is DELIBERATELY
   413	      # NOT cached (caching daemon scratch warms nothing; do not copy-paste global-state caching).
   414	      #
   415	      # SAVE/RESTORE SPLIT (ADR-0554 D9; specs/cache-warmth-policy.json postmerge-dev-trunk = sole
   416	      # writer, presubmit-trusted-affected-cone = reader). dev-push is the SOLE canonical writer via
   417	      # the gate-affected-target-set job's SAVE step (see below): that job builds the FULL //... graph on
   418	      # push, so the saved buck-out is the full-graph superset. This buck2 job is READ-ONLY on every
   419	      # trigger — it restores the full-graph buck-out and its //ci/... build is a subset
   420	      # hit. PRs never write, eliminating the per-commit multi-GB write-churn/eviction (the
   421	      # Bazel/Google post-merge-fills-the-cache pattern).
   422	      #
   423	      # Cache key is STABLE per dependency/toolchain-set (.buckconfig + toolchains/BUCK +
   424	      # Cargo.lock + rust-toolchain.toml),
   425	      # NOT per-commit. The previous `-${{ github.sha }}` suffix made the primary key unique
   426	      # every commit, so actions/cache SAVED a fresh full buck-out (multi-GB) on EVERY run and
   427	      # never hit the primary key — bloating the 10GB repo cache into constant LRU eviction and
   428	      # exhausting ephemeral-runner disk at the save step (the "No space left on device" failure,
   429	      # FRIC-017). A stable key saves once per dependency-set and restores it exactly: deterministic
   430	      # warm start, no per-commit bloat. Changed crates still rebuild (buck2 is content-addressed,
   431	      # so a restored hit is bit-identical to a cold build); only a Cargo.lock/toolchain/.buckconfig
   432	      # or Rust channel change mints a new entry. The restore prefix is scoped by
   433	      # rust-toolchain.toml so a Rust-version bump never reuses old rlibs into the new compiler.
   434	      # Interim warm-by-default until the shared content-addressed
   435	      # remote cache (NativeLink/CAS, ADR-0560, HANDOFF W3) lands with a cold-canary integrity job
   436	      # proving cold==warm. See friction-ledger buck2-no-shared-cache.
   437	      # Reclaim preinstalled ubuntu-latest bloat (.NET/Android/GHC/CodeQL/preloaded Docker images:
   438	      # ~25-30 GiB) BEFORE the multi-GB buck-out restore. This lane decompresses a ~5.78 GiB buck-out
   439	      # blob (~12-15 GiB on disk) on top of a fetch-depth:0 monorepo checkout, exhausting the ~14 GiB
   440	      # free on / on GitHub-hosted ubuntu-latest; FRIC-017 recurred on PR #741 (No space left on device
   441	      # at this restore, before any build ran). Hermetic: removes only vendor preinstall dirs that no
   442	      # oya/buck2 action consumes; touches NO repo content and NO cache (buck-out / ~/.rustup / the
   443	      # restored blob untouched), so the cold==warm integrity canary (ADR-0556/0560) is unaffected. df
   444	      # is emitted so a genuine disk-NEED growth surfaces as a true RED instead of being masked.
   445	      - name: Reclaim runner disk before warm restore (FRIC-017 preflight)
   446	        # Rust-first, data-driven preflight (ADR-0548 pipeline-as-product): retires the two
   447	        # duplicated inline `sudo rm -rf` blocks. The policy (reclaim_dirs + min_free_gib_after)
   448	        # lives in runner-disk-reclaim-policy.json; the bin best-effort removes the profile's
   449	        # vendor preinstall dirs and logs structured disk-before/after plus a JSON operator
   450	        # artifact. Policy is explicit fail-closed: threshold-miss exits INFRA-RED unless a future
   451	        # caller supplies a typed fail-open waiver, so the required context cannot silently green on
   452	        # insufficient runner capacity. Built as the runner user (buck2 on user PATH; daemon must
   453	        # not run as root); only the prebuilt binary is sudo'd (needs root for root-owned dirs).
   454	        run: |
   455	          # Build as the runner user (buck2 on user PATH; never run buck2 daemon as root —
   456	          # that corrupts cache/daemon ownership). Then sudo ONLY the prebuilt binary (needs
   457	          # root solely to remove the root-owned vendor preinstall dirs).
   458	          BIN="$(buck2 build //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin --show-output 2>/dev/null | awk '{print $2}')"
   459	          sudo -E "$BIN" \
   460	            --profile github-hosted-ubuntu-latest \
   461	            --infra-red-policy fail-closed \
   462	            --artifact-out "${RUNNER_TEMP}/runner-disk-reclaim-buck2.json"
   463	      - name: Restore buck-out (read-only; dev-push is the sole writer)
   464	        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   465	        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
   466	        with:
   467	          path: buck-out
   468	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   469	          restore-keys: |
   470	            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
   471	      # Generated-face materialization — the SINGLE out-of-graph git boundary. Re-run the emitter
   472	      # and producer against the checked-out candidate tree, then let buck2 consume those files as
   473	      # declared inputs. We deliberately do NOT byte-compare against committed JSON here: that was
   474	      # the self-referential merge-conflict surface. Byte-parity is checked after materialization.
   475	      # KEEPS materializing (not converted to ADR-0556 D5 QW-1 artifact reuse): the hermetic
   476	      # graph's gate tests consume the firewall's merge-base frozen baseline, which is per-job
   477	      # by design (ADR-0551) and deliberately absent from the producer-regen artifact; this
   478	      # step is the sanctioned boundary that feeds ALL declared generated inputs to the graph.
   479	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   480	        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   481	      # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication —
   482	      # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests
   483	      # run green, fully hermetic, with verdicts identical to the targeted gate matrix). This is the
   484	      # refactor's scope and is the binding hermetic check for this stage.
   485	      #
   486	      # The repo-wide affected-set verdict is owned by the binding gate-affected-target-set job below.
   487	      # Do not run a duplicate best-effort affected-set probe here: a non-blocking BUILD FAILED
   488	      # line inside a green job is indistinguishable from a false-green to humans and agents.
   489	      - name: buck2 build + test (//ci/..., hermetic — binding)
   490	        run: |
   491	          set -euo pipefail
   492	          # buck2 test builds its targets before running them, so a standalone
   493	          # `buck2 build` immediately before is redundant — removed (item 4 quick win).
   494	          # --unstable-write-invocation-record is additive observability only: it
   495	          # writes buck2's structured run record (cache_hit_rate, run_* counters)
   496	          # for the telemetry step below and changes nothing about the build.
   497	          buck2 test //ci/... --unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json
   498	      # Per-lane cache-hit telemetry + warm-mode guard (ADR-0560; the audit's missing-SLO item):
   499	      # structured counters from buck2's invocation record — never log-grep — labeled with this
   500	      # lane's ADR-0556 build class. The report is now binding for record-shape / warm-mode
   501	      # sanity: once owned cloud-ci flips this lane from `bypass` to warm-ro/rw, a 0%-hit run or
   502	      # missing cache counters is an INFRA-RED misconfiguration, not advisory noise. Today GitHub
   503	      # Actions remains the transitional adapter and this lane stays bypass while NativeLink is dark.
   504	      - name: Cache-hit telemetry + warm-mode guard (ADR-0560)
   505	        if: always()
   506	        run: |
   507	          set -euo pipefail
   508	          CACHE_MODE=bypass
   509	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- report --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}" --out /tmp/cache-hit-report.json
   510	          cat /tmp/cache-hit-report.json
   511	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- assert-warm --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}"
   512	      - name: Upload cache-hit telemetry artifact
   513	        if: always()
   514	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   515	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   620	        run: |
   621	          # Build as the runner user (buck2 on user PATH; never run buck2 daemon as root —
   622	          # that corrupts cache/daemon ownership). Then sudo ONLY the prebuilt binary (needs
   623	          # root solely to remove the root-owned vendor preinstall dirs).
   624	          BIN="$(buck2 build //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin --show-output 2>/dev/null | awk '{print $2}')"
   625	          sudo -E "$BIN" \
   626	            --profile github-hosted-ubuntu-latest \
   627	            --infra-red-policy fail-closed \
   628	            --artifact-out "${RUNNER_TEMP}/runner-disk-reclaim-affected-set.json"
   629	      - name: Restore buck-out (read-only; dev-push is the sole writer)
   630	        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   631	        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
   632	        with:
   633	          path: buck-out
   634	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   635	          restore-keys: |
   636	            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
   637	      # KEEPS materializing (not converted to ADR-0556 D5 QW-1 artifact reuse): same rationale
   638	      # as the buck2 lane — the cone's gate tests consume the per-job merge-base frozen
   639	      # baseline (ADR-0551), and this lane's own build-health baseline below is per-job by
   640	      # design (ADR-0554 round-3).
   641	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   642	        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   643	      - name: Fetch base ref for the merge-base anchor
   644	        if: ${{ github.event_name == 'pull_request' }}
   645	        env:
   646	          BASE_REF: ${{ github.base_ref || 'dev' }}
   647	        run: git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}"
   648	      # ── BUILD-HEALTH BASELINE (ADR-0554 D9 same-root build, round-5; ADR-0551 merge-base frozen
   649	      #    pattern). On a pull_request, derive the affected-set plan FIRST. Most PRs stay in the
   650	      #    affected cone and do not need a merge-base full-workspace build at all; only a derived
   651	      #    FULL decision needs the MERGE-BASE build-health baseline used by the ratchet. When FULL
   652	      #    is required, materialize that baseline IN THE MAIN ROOT so it shares the warm ./buck-out
   653	      #    restored above (the merge-base IS a dev commit, so the dev-keyed buck-out is near-fully
   654	      #    warm for it). We detach the SAME working tree to the merge-base COMMITTED tree-ish (the
   655	      #    candidate working tree is removed from disk for the build), run the full keep-going
   656	      #    build, capture per-target pass/fail, then a TRAP restores the candidate on EXIT. The
   657	      #    affected-set FULL tier grandfathers targets already failing at the merge-base and blocks
   658	      #    only REGRESSIONS. Skipped for push/merge_group/dispatch (the admission tier is a hard
   659	      #    full build — no grandfathering).
   660	      #
   661	      #    ANTI-LAUNDERING (ADR-0554 D6, preserved): the baseline failure-set comes ENTIRELY from
   662	      #    the merge-base COMMITTED tree (git object history — candidate-uncontrollable); during the
   663	      #    baseline build the candidate working tree is GONE from disk, so it cannot feed the
   664	      #    baseline; the report reaches the verdict ONLY via --baseline-report. The warm ./buck-out
   665	      #    is a content-addressed substrate — a buck2 hit is bit-identical to a cold build (ADR-0556
   666	      #    D1/D2) — so warmth changes only wall-clock, never the baseline SOURCE. Warm-eligible
   667	      #    under ADR-0556 with no policy change (trusted-author, content-addressed; not the
   668	      #    integrity-canary/release cold floor). GH #899 activates the trusted D8 consumer first:
   669	      #    use an exact push-to-dev baseline artifact when provenance and schema validate, else
   670	      #    fail closed to the same in-job merge-base rebuild below.
   671	      - name: Materialize merge-base build-health baseline when affected-set needs FULL
   672	        if: ${{ github.event_name == 'pull_request' }}
   673	        env:
   674	          BASE_REF: ${{ github.base_ref || 'dev' }}
   675	          GH_TOKEN: ${{ github.token }}
   676	        run: |
   677	          set -euo pipefail
   678	          merge_base="$(git merge-base "origin/${BASE_REF}" HEAD)"
   679	          orig_ref="$(git rev-parse HEAD)"
   680	          candidate_toolchain="$(awk -F'"' '/channel/ {print $2; exit}' rust-toolchain.toml)"
   681	          decision_log="${RUNNER_TEMP}/affected-set-derive.log"
   682	          full_required_marker="${RUNNER_TEMP}/affected-set-full-required"
   683	          echo "false" > "${full_required_marker}"
   684	          gate_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-affected-set-bin --show-output | awk '{print $2}')"
   685	          telemetry_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-step-telemetry-bin --show-output | awk '{print $2}')"
   686	          ci_step_telemetry="${RUNNER_TEMP}/oya-cloud-ci-step-telemetry"
   687	          cp "${telemetry_bin}" "${ci_step_telemetry}"
   688	          chmod +x "${ci_step_telemetry}"
   689	          echo "affected-set preflight: derive plan before merge-base baseline"
   690	          "${ci_step_telemetry}" --phase derive-affected-set-tier -- "${gate_bin}" \
   691	            --policy ci/facade/affected-target-set/affected-set-policy.json \
   692	            --base "origin/${BASE_REF}" --mode auto --derive-only \
   693	            --decision-artifact-out "${RUNNER_TEMP}/affected-set-derive-decision.json" \
   694	            | tee "${decision_log}"
   695	          if grep -Eq '^affected-set: (decision=FULL|ESCALATE to FULL)' "${decision_log}"; then
   696	            echo "true" > "${full_required_marker}"
   697	            echo "affected-set preflight: FULL decision requires merge-base build-health baseline"
   698	          else
   699	            echo "affected-set preflight: derived non-FULL decision; skipping merge-base baseline"
   700	            exit 0
   701	          fi
   702	          echo "build-health baseline: merge-base=${merge_base} candidate=${orig_ref}"
   703	          artifact_name="build-health-baseline-${merge_base}"
   704	          echo "build-health baseline: attempting trusted dev-push artifact ${artifact_name}"
   705	          try_trusted_baseline_artifact() {
   706	            if ! command -v gh >/dev/null 2>&1; then
   707	              echo "build-health baseline: gh unavailable; falling back to in-job merge-base rebuild"
   708	              return 1
   709	            fi
   710	            local runs_json="${RUNNER_TEMP}/build-health-trusted-runs.json"
   711	            local artifacts_json="${RUNNER_TEMP}/build-health-trusted-artifacts.json"
   712	            local trusted_zip="${RUNNER_TEMP}/build-health-trusted.zip"
   713	            local trusted_dir="${RUNNER_TEMP}/build-health-trusted"
   714	            local trusted_report="${trusted_dir}/build-health-admission-report.json"
   715	
   716	            gh api --method GET -H "Accept: application/vnd.github+json" \
   717	              "repos/${GITHUB_REPOSITORY}/actions/workflows/oya-ci-required.yml/runs" \
   718	              -f branch=dev -f event=push -f status=success -F per_page=50 > "${runs_json}" \
   719	              || { echo "build-health baseline: trusted run lookup failed; falling back to in-job rebuild"; return 1; }
   720	
   721	            local run_id
   722	            run_id="$(python3 - "${runs_json}" "${merge_base}" <<'PY'
   723	          import json
   724	          import sys
   725	
   726	          path, merge_base = sys.argv[1], sys.argv[2]
   727	          with open(path, encoding="utf-8") as fh:
   728	              payload = json.load(fh)
   729	          for run in payload.get("workflow_runs", []):
   730	              if (
   731	                  run.get("head_sha") == merge_base
   732	                  and run.get("event") == "push"
   733	                  and run.get("head_branch") == "dev"
   734	                  and run.get("conclusion") == "success"
   735	              ):
   736	                  print(run["id"])
   737	                  break
   738	          PY
   739	            )"
   740	            if [ -z "${run_id}" ]; then
   741	              echo "build-health baseline: no successful trusted push-to-dev run for ${merge_base}; falling back to in-job rebuild"
   742	              return 1
   743	            fi
   744	
   745	            gh api --method GET -H "Accept: application/vnd.github+json" \
   746	              "repos/${GITHUB_REPOSITORY}/actions/runs/${run_id}/artifacts" \
   747	              -F per_page=100 > "${artifacts_json}" \
   748	              || { echo "build-health baseline: trusted artifact lookup failed; falling back to in-job rebuild"; return 1; }
   749	
   750	            local artifact_id
   751	            artifact_id="$(python3 - "${artifacts_json}" "${artifact_name}" <<'PY'
   752	          import json
   753	          import sys
   754	
   755	          path, expected_name = sys.argv[1], sys.argv[2]
   756	          with open(path, encoding="utf-8") as fh:
   757	              payload = json.load(fh)
   758	          for artifact in payload.get("artifacts", []):
   759	              if artifact.get("name") == expected_name and not artifact.get("expired", True):
   760	                  print(artifact["id"])
   761	                  break
   762	          PY
   763	            )"
   764	            if [ -z "${artifact_id}" ]; then
   765	              echo "build-health baseline: no unexpired exact artifact ${artifact_name} on trusted run ${run_id}; falling back to in-job rebuild"
   766	              return 1
   767	            fi
   768	
   769	            gh api "repos/${GITHUB_REPOSITORY}/actions/artifacts/${artifact_id}/zip" > "${trusted_zip}" \
   770	              || { echo "build-health baseline: artifact download failed; falling back to in-job rebuild"; return 1; }
   771	            rm -rf "${trusted_dir}"
   772	            mkdir -p "${trusted_dir}"
   773	            python3 -m zipfile -e "${trusted_zip}" "${trusted_dir}" \
   774	              || { echo "build-health baseline: artifact unzip failed; falling back to in-job rebuild"; return 1; }
   775	
   776	            if ! python3 - "${trusted_report}" <<'PY'
   777	          import json
   778	          import os
   779	          import sys
   780	
   781	          path = sys.argv[1]
   782	          if not os.path.getsize(path):
   783	              raise SystemExit("empty report file")
   784	          with open(path, encoding="utf-8") as fh:
   785	              payload = json.load(fh)
   786	          results = payload.get("results")
   787	          if not isinstance(results, dict) or not results:
   788	              raise SystemExit("missing or empty results object")
   789	          PY
   790	            then
   791	              echo "build-health baseline: trusted artifact schema/emptiness invalid; falling back to in-job rebuild"
   792	              return 1
   793	            fi
   794	            cp "${trusted_report}" "${RUNNER_TEMP}/build-health-baseline.json"
   795	            echo "build-health baseline: trusted artifact hit run_id=${run_id} artifact_id=${artifact_id}"
   796	            return 0
   797	          }
   798	          if try_trusted_baseline_artifact; then
   799	            exit 0
   800	          fi
   801	          # ALWAYS restore the candidate tree on EXIT — a failed baseline build can never strand CI
   802	          # on the merge-base tree (the subsequent Binding affected-set step runs on the candidate).
   803	          # NOTE: if the timeout-minutes:45 rail SIGKILLs this build, the bash EXIT trap does NOT
   804	          # fire (tree left detached at merge-base) — but a timeout fails the whole job RED → fan-in
   805	          # RED, so it is fail-closed and never produces a wrong-baseline verdict.
   806	          restore_candidate_tree() {
   807	            local exit_status="$?"
   808	            git checkout --quiet --detach "${orig_ref}" 2>/dev/null || git checkout --quiet "${orig_ref}"
   809	            if [ "${candidate_toolchain}" != "${baseline_toolchain:-${candidate_toolchain}}" ]; then
   810	              echo "build-health baseline: cleaning buck-out after restoring candidate toolchain ${candidate_toolchain}"
   811	              buck2 clean
   812	            fi
   813	            exit "${exit_status}"
   814	          }
   815	          trap restore_candidate_tree EXIT
   816	          # Detach the MAIN working tree to the merge-base COMMITTED tree-ish: the baseline is
   817	          # computed from git object history (candidate-uncontrollable), and the candidate working
   818	          # tree is removed from disk for the build, so a PR cannot grow its own baseline to
   819	          # launder a regression.
   820	          git checkout --quiet --detach "${merge_base}"
   821	          baseline_toolchain="$(awk -F'"' '/channel/ {print $2; exit}' rust-toolchain.toml)"
   822	          rustup toolchain install
   823	          if [ "${candidate_toolchain}" != "${baseline_toolchain}" ]; then
   824	            echo "build-health baseline: Rust toolchain changed ${baseline_toolchain} -> ${candidate_toolchain}; isolating buck-out"
   825	            buck2 clean
   826	          fi
   827	          # Build the whole merge-base workspace keep-going. Same-channel PRs share warm ./buck-out;
   828	          # Rust-channel bump PRs intentionally go cold on both sides to avoid mixed-rustc rlibs.
   829	          # The build is EXPECTED to be non-zero (dev carries pre-existing breakage) — that is the
   830	          # baseline, not a failure, so we never propagate its exit code.
   831	          "${ci_step_telemetry}" --phase materialize-merge-base-build-health-baseline -- \
   832	            buck2 build //... --keep-going \
   833	              --build-report "${RUNNER_TEMP}/build-health-baseline.json" || true
   834	          test -s "${RUNNER_TEMP}/build-health-baseline.json" \
   835	            || { echo "build-health: FATAL empty merge-base baseline report"; exit 1; }
   836	      - name: Binding affected-set build + test (cone-binding; FULL tier = build-health ratchet)
   837	        env:
   838	          EVENT_NAME: ${{ github.event_name }}
   839	          BASE_REF: ${{ github.base_ref || 'dev' }}
   840	        run: |
   841	          set -euo pipefail
   842	          gate_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-affected-set-bin --show-output | awk '{print $2}')"
   843	          telemetry_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-step-telemetry-bin --show-output | awk '{print $2}')"
   844	          if [ "${EVENT_NAME}" = "pull_request" ]; then
   845	            # PR tier: auto (cone-binding, hard-fail on a NEW break in the changed cone); a FULL
   846	            # escalation runs the build-health ratchet against the merge-base baseline. The
   847	            # baseline is only materialized when the derive preflight proves FULL is needed.
   848	            baseline_args=()
   849	            if [ -f "${RUNNER_TEMP}/affected-set-full-required" ] \
   850	                && grep -qx 'true' "${RUNNER_TEMP}/affected-set-full-required"; then
   851	              test -s "${RUNNER_TEMP}/build-health-baseline.json" \
   852	                || { echo "affected-set: FATAL missing baseline after FULL derive preflight"; exit 1; }
   853	              baseline_args=(--baseline-report "${RUNNER_TEMP}/build-health-baseline.json")
   854	            fi
   855	            "${telemetry_bin}" --phase binding-affected-set-build-test -- "${gate_bin}" \
   856	              --policy ci/facade/affected-target-set/affected-set-policy.json \
   857	              --base "origin/${BASE_REF}" --mode auto \
   858	              --decision-artifact-out "${RUNNER_TEMP}/affected-set-binding-decision.json" \
   859	              "${baseline_args[@]}"
   860	          else
   861	            # Admission/integration tier (merge_group/push/dispatch): hard full build+test — the
   862	            # integration tip MUST be green, no grandfathering. ADR-0554 D7: this run ALSO captures
   863	            # a build-report at ${RUNNER_TEMP}/build-health-admission-report.json (the binary's
   864	            # stable RUNNER_TEMP-anchored path) as a pure byproduct; the verdict is unchanged
   865	            # (non-empty failure set = hard fail) and the report is uploaded below only on
   866	            # trusted push-to-dev.
   867	            "${telemetry_bin}" --phase binding-affected-set-build-test -- "${gate_bin}" \
   868	              --policy ci/facade/affected-target-set/affected-set-policy.json \
   869	              --base "origin/${BASE_REF}" --mode full \
   870	              --decision-artifact-out "${RUNNER_TEMP}/affected-set-binding-decision.json"
   871	          fi
   872	      # ── FULL-GRAPH CACHE SAVE (ADR-0554 D9; sole canonical writer). On dev-push this job runs
   873	      #    --mode full (buck2 build + test //...), so buck-out is populated with the FULL workspace
   874	      #    graph — not just //ci/... as in the buck2 lane. Saving here means PR
   875	      #    gate-affected-target-set restores a full-graph buck-out, so the same-root merge-base baseline
   876	      #    build is near-fully-warm (the merge-base IS a recent dev commit whose full-graph buck-out
   877	      #    was just saved). The buck2 lane restores the same key and is a subset hit. One save step,
   878	      #    one job, one key — no two-writer race. Runs AFTER the Binding step (buck-out fully
   879	      #    populated). Guarded push-to-dev so PRs remain read-only (restore-only via the step above).
   880	      #    Size note: the full-graph buck-out is one blob per stable key (overwrites, non-accumulating
   881	      #    due to dev-push-sole-writer), bounded by the dependency-set change cadence — worth watching
   882	      #    against the GitHub 10GB cache limit; NativeLink CAS (ADR-0560) removes this at cutover.
   883	      - name: Save buck-out (dev-push only; sole canonical full-graph writer)
   884	        if: ${{ github.event_name == 'push' && github.ref == 'refs/heads/dev' }}
   885	        # actions/cache/save@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   886	        uses: actions/cache/save@27d5ce7f107fe9357f9df03efb73ab90386fccae
   887	        with:
   888	          path: buck-out
   889	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   890	      # ── BUILD-HEALTH BASELINE PRODUCER (ADR-0554 D7; ADR-0556 D5 QW-1 trusted-producer +
   891	      #    postmerge-dev-trunk warmth class). ONLY a trusted push-to-dev publishes the admission
   892	      #    build-report as the merge-base-to-be baseline artifact — NOT merge_group, NOT
   893	      #    pull_request — so the artifact namespace stays clean of attacker-controllable producers
   894	      #    (part of the DEFERRED D8 consumer's anti-laundering defense; D8 trusts workflow_run
   895	      #    PROVENANCE, never the artifact name). Producer-only: nothing consumes this yet, so it is
   896	      #    sound + harmless (no merge-authority change, no new permissions). The artifact is on the
   897	      #    critical path of BOTH the deferred D8 cross-run consumer AND the ADR-0560 warm-CAS.
   898	      - name: Upload build-health baseline artifact (trusted push-to-dev producer)
   899	        if: ${{ github.event_name == 'push' && github.ref == 'refs/heads/dev' }}
   900	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   901	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   902	        with:
   903	          name: build-health-baseline-${{ github.sha }}
   904	          path: ${{ runner.temp }}/build-health-admission-report.json
   905	          retention-days: 90
   906	          if-no-files-found: error
   907	      - name: Upload affected-set operator artifacts
   908	        if: always()
   909	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   910	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   911	        with:
   912	          name: affected-set-operator-artifacts
   913	          path: |
   914	            ${{ runner.temp }}/runner-disk-reclaim-affected-set.json
   915	            ${{ runner.temp }}/affected-set-derive-decision.json
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/main.rs | nl -ba | sed -n '650,850p'" in /Users/jasonlee/Developer/oyatie
exec
/bin/zsh -lc "git show origin/dev:ci/facade/affected-target-set/src/lib.rs | nl -ba | sed -n '360,385p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   360	                },
   361	            }
   362	            continue;
   363	        }
   364	        match change {
   365	            Change::Deleted(_) => {
   366	                if policy
   367	                    .require_owner_patterns
   368	                    .iter()
   369	                    .any(|pat| glob_match(pat, path))
   370	                {
   371	                    plan.full_reasons
   372	                        .push(format!("graph-relevant file `{path}` was deleted"));
   373	                    plan.classified
   374	                        .push((path.to_owned(), PathClass::DeletedGraphFile));
   375	                } else {
   376	                    plan.classified
   377	                        .push((path.to_owned(), PathClass::DeletedIrrelevant));
   378	                }
   379	            }
   380	            Change::Present(_) => {
   381	                plan.owner_paths.push(path.to_owned());
   382	                plan.classified
   383	                    .push((path.to_owned(), PathClass::OwnerQuery));
   384	            }
   385	        }
 succeeded in 0ms:
   650	        );
   651	    }
   652	    Ok(targets)
   653	}
   654	
   655	fn print_classification(plan: &Plan, owners: &BTreeMap<String, Vec<String>>) {
   656	    println!("{LOG}: classification (every changed file, mechanically derived):");
   657	    for (path, class) in &plan.classified {
   658	        match class {
   659	            PathClass::FullTrigger(pat) => {
   660	                println!("{LOG}:   FULL-TRIGGER {path} (matches `{pat}`)")
   661	            }
   662	            PathClass::DeletedGraphFile => {
   663	                println!("{LOG}:   FULL-TRIGGER {path} (graph file deleted/unmappable)")
   664	            }
   665	            PathClass::Buildfile => {
   666	                println!(
   667	                    "{LOG}:   FULL-TRIGGER {path} (buildfile — blast radius exceeds its package)"
   668	                )
   669	            }
   670	            PathClass::PackagePattern(pat) => println!("{LOG}:   PACKAGE      {path} -> {pat}"),
   671	            PathClass::OwnerQuery => {
   672	                let n = owners.get(path).map(Vec::len).unwrap_or(0);
   673	                println!("{LOG}:   OWNER        {path} -> {n} target(s)");
   674	            }
   675	            PathClass::DeletedIrrelevant => {
   676	                println!("{LOG}:   NO-GRAPH     {path} (deleted, outside graph classes)")
   677	            }
   678	        }
   679	    }
   680	}
   681	
   682	/// The FULL-tier runner (ADR-0554 round-3; D7 round-4 producer). Two modes:
   683	///
   684	/// - WITHOUT a baseline report (`--mode full` at admission, or any caller that does not pass
   685	///   `--baseline-report`): a hard `buck2 build //... --keep-going --build-report` + `buck2 test
   686	///   //...` — EVERY build failure blocks (non-empty failure set = hard fail; no grandfathering:
   687	///   the integration tip MUST be green). D7 (round-4): the admission build now captures a
   688	///   `--build-report` as a PURE BYPRODUCT and derives the same hard verdict from the report's
   689	///   failure set being EMPTY. The report is written to a stable path (`admission_report_path`)
   690	///   so the trusted push-to-dev workflow can publish it as the `build-health-baseline-<sha>`
   691	///   artifact (the merge-base-to-be baseline for the DEFERRED D8 cross-run consumer + ADR-0560
   692	///   warm-CAS). Merge authority is UNCHANGED — the verdict is identical to the prior hard build,
   693	///   nothing consumes the artifact yet, so there is zero laundering surface.
   694	/// - WITH a baseline report (the PR `pull_request` FULL tier): the BUILD-HEALTH RATCHET. It builds
   695	///   `//... --keep-going --build-report` at HEAD and tests them, then compares the HEAD build
   696	///   FAILURE set against the merge-base baseline failure set: only REGRESSIONS (targets that build
   697	///   at the merge-base but fail at head, or brand-new failing targets) block; pre-existing build
   698	///   debt is grandfathered. This turns the FULL tier from a flag-day requirement into a true
   699	///   ratchet (block new debt, grandfather pre-existing — FRIC-1781112000 / #698). Tests are still
   700	///   run and a TEST regression in a buildable target blocks via the test exit (the ratchet governs
   701	///   BUILD failures; a build that succeeds then test-fails is a normal hard failure).
   702	fn run_full(buck2: &str, policy: &Policy, baseline_report: Option<&str>) -> ExitCode {
   703	    let Some(baseline_path) = baseline_report else {
   704	        // Admission/integration tier: hard full build+test, every failure blocks. D7: emit the
   705	        // build-report as a byproduct and derive the hard verdict from an EMPTY failure set.
   706	        return run_full_admission_producer(buck2, policy);
   707	    };
   708	
   709	    // PR FULL tier: build-health ratchet. Build the whole workspace with --keep-going so every
   710	    // target's status is recorded even past the first failure, into a build-report.
   711	    let head_report = match std::env::temp_dir()
   712	        .join(format!(
   713	            "{GATE_ID}-head-build-report-{}.json",
   714	            std::process::id()
   715	        ))
   716	        .into_os_string()
   717	        .into_string()
   718	    {
   719	        Ok(p) => p,
   720	        Err(_) => {
   721	            eprintln!("{LOG}: FAIL — could not form a temp path for the head build-report");
   722	            return ExitCode::from(2);
   723	        }
   724	    };
   725	    println!(
   726	        "{LOG}: FULL tier (build-health ratchet vs merge-base baseline {baseline_path}): \
   727	         {buck2} build //... --keep-going --build-report {head_report}"
   728	    );
   729	    // We intentionally do NOT propagate this build's exit code: --keep-going still exits non-zero
   730	    // if ANY target failed, but pre-existing failures must NOT block. The verdict comes from the
   731	    // build-report diff below. (A genuine infra failure surfaces as an unparseable/empty report,
   732	    // which the ratchet then refuses on — fail-closed.)
   733	    let mut command = Command::new(buck2);
   734	    command.args([
   735	        "build",
   736	        "//...",
   737	        "--keep-going",
   738	        "--build-report",
   739	        &head_report,
   740	    ]);
   741	    if let Err(e) = run_command_with_progress(
   742	        "build-health-ratchet-head-build",
   743	        &mut command,
   744	        &format!("{buck2} build //... --keep-going --build-report {head_report}"),
   745	    ) {
   746	        eprintln!("{LOG}: WARN — could not execute head build-health command: {e}");
   747	    }
   748	
   749	    let baseline_json = match fs::read_to_string(baseline_path) {
   750	        Ok(s) => s,
   751	        Err(e) => {
   752	            eprintln!(
   753	                "{LOG}: FAIL — cannot read merge-base baseline report `{baseline_path}`: {e}"
   754	            );
   755	            return ExitCode::from(2);
   756	        }
   757	    };
   758	    let head_json = match fs::read_to_string(&head_report) {
   759	        Ok(s) => s,
   760	        Err(e) => {
   761	            eprintln!("{LOG}: FAIL — cannot read head build-report `{head_report}`: {e}");
   762	            return ExitCode::from(2);
   763	        }
   764	    };
   765	    let baseline = match parse_build_report(&baseline_json) {
   766	        Ok(r) => r,
   767	        Err(e) => {
   768	            eprintln!("{LOG}: FAIL — merge-base baseline report parse error: {e}");
   769	            return ExitCode::from(2);
   770	        }
   771	    };
   772	    let head = match parse_build_report(&head_json) {
   773	        Ok(r) => r,
   774	        Err(e) => {
   775	            eprintln!("{LOG}: FAIL — head build-report parse error: {e}");
   776	            return ExitCode::from(2);
   777	        }
   778	    };
   779	    // Fail-closed laundering guard: an empty merge-base baseline would grandfather every head
   780	    // failure. CI builds the whole merge-base workspace, so the baseline is never legitimately
   781	    // empty — refuse rather than silently pass.
   782	    if baseline.is_empty() {
   783	        eprintln!(
   784	            "{LOG}: FAIL — merge-base baseline build-report has no `results`. Refusing to \
   785	             grandfather every head failure against an empty baseline (the laundering hole)."
   786	        );
   787	        return ExitCode::from(2);
   788	    }
   789	
   790	    let baseline_failures = failing_targets(&baseline);
   791	    let head_failures = failing_targets(&head);
   792	    let verdict = build_health_verdict(&baseline_failures, &head_failures);
   793	    println!(
   794	        "{LOG}: build-health — head build failures={}, baseline failures={}, regressions={}, \
   795	         grandfathered={}, fixed={}",
   796	        head_failures.len(),
   797	        baseline_failures.len(),
   798	        verdict.regressions.len(),
   799	        verdict.grandfathered.len(),
   800	        verdict.fixed.len()
   801	    );
   802	    for t in &verdict.grandfathered {
   803	        println!("{LOG}:   pre-existing-red (grandfathered) {t}");
   804	    }
   805	    if !verdict.is_green() {
   806	        eprintln!(
   807	            "{LOG}: RED — {} build REGRESSION(S) vs the merge-base (built at origin/dev, FAIL at \
   808	             head — or brand-new failing target):",
   809	            verdict.regressions.len()
   810	        );
   811	        for t in &verdict.regressions {
   812	            eprintln!("{LOG}:   REGRESSION {t}");
   813	        }
   814	        eprintln!(
   815	            "{LOG}: REMEDIATION: fix these targets or revert the change that broke them; \
   816	             pre-existing failures are grandfathered, only NEW build debt blocks. REPRODUCE: \
   817	             {buck2} build {} --keep-going",
   818	            verdict.regressions.join(" ")
   819	        );
   820	        return ExitCode::from(1);
   821	    }
   822	
   823	    // No build regressions -> GREEN. SCOPE (ADR-0554 round-3): the FULL tier governs BUILD health
   824	    // (the cf16525 class is a COMPILE break). It deliberately does NOT run a workspace-wide
   825	    // `buck2 test //...`: that would reintroduce a flag-day on PRE-EXISTING test failures (the
   826	    // exact debt-grandfathering problem this round fixes, one layer up). Test coverage of the
   827	    // ACTUAL changed code is the cone path's job (auto mode, hard-fail, unchanged — the cf16525
   828	    // fixture); a FULL-tier TEST-health ratchet (same baseline-diff over a test report) is the
   829	    // declared next IP. Conservative and sound: never false-green on a build regression, never
   830	    // flag-day on pre-existing debt.
   831	    println!(
   832	        "{LOG}: PASS — no build regressions vs the merge-base ({} pre-existing build failure(s) \
   833	         grandfathered).",
   834	        verdict.grandfathered.len()
   835	    );
   836	    ExitCode::SUCCESS
   837	}
   838	
   839	/// The stable path the admission build-report is written to (D7). GitHub Actions sets
   840	/// `RUNNER_TEMP`; we anchor the report there so the workflow's upload step references the SAME
   841	/// path without guessing a PID. Off-CI (or if `RUNNER_TEMP` is unset) it falls back to the OS
   842	/// temp dir with the identical basename — deterministic either way.
   843	fn admission_report_path() -> PathBuf {
   844	    let dir = std::env::var_os("RUNNER_TEMP")
   845	        .map(PathBuf::from)
   846	        .unwrap_or_else(std::env::temp_dir);
   847	    dir.join("build-health-admission-report.json")
   848	}
   849	
   850	fn long_step_telemetry_interval() -> Duration {
exec
/bin/zsh -lc 'git show origin/dev:.github/workflows/docs-graph-drift.yml | nl -ba' in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	name: docs-graph-drift
     2	
     3	# Generator health check for the architecture dashboard (docs/architecture/product-graph.html).
     4	#
     5	# The dashboard is GENERATED from SSOT by an owned-Rust CI pipeline tool
     6	# (tools/oya-architecture-graph-generator-app) — NOT a developer-facing `oya`
     7	# subcommand. As of ADR-0613 the dashboard AND its masterplan input are DE-COMMITTED
     8	# (materialization_mode: not-tracked-in-git); byte-parity against a committed HTML copy
     9	# is intentionally RETIRED. Dashboard freshness/determinism is now enforced by the
    10	# REQUIRED generated-artifact-freshness gate (regenerate-twice determinism canary over the
    11	# controller-materialized path). This job's remaining role is to build the generator and run
    12	# its owned golden/regeneration tests — materializing the de-committed masterplan input first,
    13	# since it is absent from a fresh checkout.
    14	#
    15	# Transitional runner model: GitHub Actions executes this feedback adapter today,
    16	# but the durable policy source is the owned Rust generator/gate and future cloud-ci
    17	# runner. Intentionally ABSENT from the branch-protection required set
    18	# (.github/branch-protection.yaml) — feedback only; branch protection is NOT
    19	# changed here.
    20	
    21	on:
    22	  pull_request:
    23	    paths:
    24	      # product-graph.html + masterplan.generated.json are de-committed (ADR-0613, untracked) and
    25	      # can no longer appear in a PR file list; trigger only on the generator + its tracked inputs.
    26	      - ".github/workflows/docs-graph-drift.yml"
    27	      - "tools/oya-architecture-graph-generator-app/**"
    28	      - "docs/architecture/product-graph.template.html"
    29	      - "docs/machine-readable/architecture-graph.json"
    30	  push:
    31	    branches: [dev]
    32	    paths:
    33	      - ".github/workflows/docs-graph-drift.yml"
    34	      - "tools/oya-architecture-graph-generator-app/**"
    35	      - "docs/architecture/product-graph.template.html"
    36	      - "docs/machine-readable/architecture-graph.json"
    37	
    38	permissions:
    39	  contents: read
    40	
    41	concurrency:
    42	  group: docs-graph-drift-${{ github.workflow }}-${{ github.head_ref || github.run_id }}
    43	  cancel-in-progress: true
    44	
    45	jobs:
    46	  docs-graph-drift:
    47	    name: docs-graph-drift
    48	    runs-on: ubuntu-latest
    49	    timeout-minutes: 15
    50	    steps:
    51	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    52	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
    53	        with:
    54	          persist-credentials: false
    55	          # fetch-depth: 0 so merge-base(HEAD, origin/dev) resolves — the materializer's
    56	          # landed-plan carve-out needs it to exclude the committed move-plans (matches the
    57	          # required legs; a shallow checkout would fail-closed on the >1-plan guard).
    58	          fetch-depth: 0
    59	      - name: Install pinned Rust toolchain
    60	        uses: dtolnay/rust-toolchain@21dc36fb71dd22e3317045c0c31a3f4249868b17
    61	        with:
    62	          toolchain: 1.96.0
    63	      - name: Cache Buck2 official prebuilt
    64	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    65	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
    66	        with:
    67	          path: /tmp/oya-ci-buck2-2026-06-01
    68	          # The cached asset is always SHA-256 verified by infra/ci/install-buck2.sh before execution.
    69	          key: docs-graph-drift-${{ runner.os }}-buck2-2026-06-01-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555
    70	          restore-keys: |
    71	            docs-graph-drift-${{ runner.os }}-buck2-2026-06-01-
    72	      - name: Materialize de-committed inputs, build + test the generator
    73	        run: |
    74	          set -euo pipefail
    75	          infra/ci/install-buck2.sh
    76	          export PATH="/tmp/oya-ci-buck2-2026-06-01:${PATH}"
    77	          rustc --version
    78	          # masterplan.generated.json (ADR-0613 de-commit) is absent from a fresh checkout;
    79	          # materialize it (and the other on-demand faces) from SSOT before the generator and its
    80	          # golden/regeneration tests consume it. Dashboard freshness/determinism is enforced by the
    81	          # required generated-artifact-freshness gate (regenerate-twice determinism canary), not by a
    82	          # byte-parity-against-committed check here (retired with the de-commit).
    83	          buck2 run root//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
    84	          buck2 build root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator
    85	          buck2 test \
    86	            root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator-regeneration-test \
    87	            root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator-golden-test
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | rg -n \"install-buck2\\.sh|rustup toolchain install|dtolnay/rust-toolchain|Materialize cloud-ci generated faces|materialize-generated-faces-bin|actions/cache/(restore|save)@|uses: actions/cache/(restore|save)\"" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
65:        run: infra/ci/install-buck2.sh
66:      - name: Materialize cloud-ci generated faces
67:        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
183:        run: infra/ci/install-buck2.sh
189:          rustup toolchain install
211:        run: infra/ci/install-buck2.sh
226:          rustup toolchain install
253:        run: infra/ci/install-buck2.sh
260:          rustup toolchain install
262:          buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
284:        run: infra/ci/install-buck2.sh
290:      - name: Materialize cloud-ci generated faces
291:        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
317:        run: infra/ci/install-buck2.sh
331:          rustup toolchain install
378:        run: infra/ci/install-buck2.sh
402:          rustup toolchain install
464:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
465:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
479:      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
480:        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
560:        run: infra/ci/install-buck2.sh
587:          rustup toolchain install
630:        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
631:        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
641:      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
642:        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
822:          rustup toolchain install
885:        # actions/cache/save@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
886:        uses: actions/cache/save@27d5ce7f107fe9357f9df03efb73ab90386fccae
957:        run: infra/ci/install-buck2.sh
969:          rustup toolchain install
1094:        run: infra/ci/install-buck2.sh
1106:          rustup toolchain install
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '45,205p;225,365p;920,1190p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
    45	jobs:
    46	  # ── Producer regen: materialize the cloud-ci generated faces from the checked-out candidate
    47	  #    tree. Generated JSON is not a contributor merge surface; the CI/controller workspace
    48	  #    regenerates it before gates consume it, then uploads it both as evidence AND as the
    49	  #    faces source the mere-reader gate matrix legs download (ADR-0556 D5 QW-1).
    50	  producer-regen:
    51	    name: producer-regen (accounting-registry)
    52	    runs-on: ubuntu-latest
    53	    steps:
    54	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    55	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
    56	        with:
    57	          persist-credentials: false
    58	          # Full history: the accounting producer derives last_touch_commit via
    59	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
    60	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
    61	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
    62	          # and identical to a full local clone.
    63	          fetch-depth: 0
    64	      - name: Install buck2 (digest-pinned prebuilt release)
    65	        run: infra/ci/install-buck2.sh
    66	      - name: Materialize cloud-ci generated faces
    67	        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
    68	      - name: Upload regenerated faces
    69	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
    70	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
    71	        with:
    72	          name: accounting-faces
    73	          # Two upload roots -> the artifact is rooted at their least common ancestor
    74	          # (ci/facade/): the regenerated accounting faces PLUS the untracked
    75	          # volatile scm snapshot (ADR-0552) the staleness-reaper leg ages rows from. The
    76	          # mere-reader gate matrix legs download this artifact instead of re-materializing
    77	          # per leg (ADR-0556 D5 QW-1, gate-fleet-shared-graph same-run trusted reuse).
    78	          # Deliberately NOT uploaded: the firewall's merge-base frozen baseline — its
    79	          # materialization is per-job by design (ADR-0551 frozen-policy-wins) and must
    80	          # never become a shareable artifact.
    81	          path: |
    82	            ci/facade/artifact-inventory-registry/*.generated.json
    83	            ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json
    84	          if-no-files-found: error
    85	
    86	  # ── GATE LANES (reusable matrix). Every homogeneous gate is the SAME step — one
    87	  #    `buck2 test //ci/facade/<crate>:{ci-<crate>-unittest,ci-<crate>-gate}` — so instead of copy-pasting a job per gate, a single
    88	  #    matrixed `gate` job fans out over the crate list. Adding a homogeneous gate is ONE line in the matrix
    89	  #    below; the `gate_registration` meta-test (in the cloud-ci-firewall lane) ENFORCES that
    90	  #    every in-tree gate crate is listed here, every oya-ci.toml accounting/firewall gate has
    91	  #    bundled disposition DATA, and every gate lane is a fan-in dependency. Each matrix leg is its own check-run
    92	  #    `gate (oya-cloud-ci-<x>-app)`, preserving per-gate attribution; legs with a live-corpus
    93	  #    self-test are born-blocking. `fail-fast: false` = surface-all (every leg runs to
    94	  #    completion even if a sibling fails).
    95	  #    (Deliberately a matrix, NOT a `workflow_call` reusable workflow: a called workflow would
    96	  #    rename the published check-runs [`<caller> / <job>`], breaking the `oya-ci-required`
    97	  #    branch-protection context. A future owned oya-ci runner can reuse this matrix verbatim —
    98	  #    "one logic, two runners", D-CICD-AUTHORITY.)
    99	  #
   100	  #    FACES VIA ARTIFACT (ADR-0556 D5 QW-1, gate-fleet-shared-graph warm-safe same-run reuse):
   101	  #    every leg here is a MERE READER of the generated faces — none re-verifies regeneration
   102	  #    itself, so each leg downloads the producer-regen artifact instead of paying its own
   103	  #    materialize step (the audit-measured ~45-55s/leg hub rebuild, 16x per run).
   104	  #    Same-run, same candidate tree, same trusted writer — no cross-run cache participation.
   105	  #    The regeneration-checking gates KEEP materializing independently: registry-drift (the
   106	  #    byte-parity detector — detectors never consume the thing they attest), producer-regen
   107	  #    itself, and cloud-ci-firewall (its merge-base frozen baseline is per-job by design,
   108	  #    ADR-0551). `needs: producer-regen` serializes these legs behind a ~75s producer job;
   109	  #    the workflow critical path (affected-set/buck2 lanes) is unaffected. If producer-regen
   110	  #    fails, legs are skipped and the fan-in goes RED (fail-closed, same join as today).
   111	  #    `fetch-depth: 0` retained conservatively: no matrix gate calls git (verified — faces
   112	  #    carry all history-derived data per ADR-0552), but shallowing the checkout is a separate
   113	  #    reviewed change, not a side effect of artifact reuse.
   114	  gate:
   115	    needs: producer-regen
   116	    # Descriptive per-leg check-run name (matrix.label) — each leg publishes as
   117	    # "gate · <discipline>", not a bare "gate (crate)". Adding a gate = one `include` line
   118	    # (crate + label); the gate_registration meta-test enforces every gate crate is listed.
   119	    name: ${{ matrix.label }}
   120	    runs-on: ubuntu-latest
   121	    strategy:
   122	      fail-fast: false
   123	      matrix:
   124	        include:
   125	          - { crate: cross-artifact-agreement, label: "gate · cross-artifact-agreement (GATE-1, incl. the 4 masterplan-v2 plan gates: structural ID/DAG/orphans · projection-freshness · plan-vs-evidence · read-contract/entry-surface)" }
   126	          - { crate: artifact-accountability,         label: "gate · total-accounting (GATE-2)" }
   127	          - { crate: stale-artifact-detection,         label: "gate · staleness-reaper (GATE-3, born-blocking)" }
   128	          - { crate: automation-coverage,       label: "gate · automation-ratchet (GATE-4, polices gates)" }
   129	          - { crate: crate-layer-suffix,         label: "gate · bnf-layer-suffix (BNF §2.5#4, born-blocking)" }
   130	          - { crate: package-manifest-hygiene,         label: "gate · manifest-hygiene (§2.5#7, born-blocking)" }
   131	          - { crate: crate-name-prefix,            label: "gate · cargo-prefix (ADR-0017)" }
   132	          - { crate: slo-coverage,            label: "gate · slo-coverage (catalog SLO input contract, born-blocking)" }
   133	          - { crate: license-policy,          label: "gate · license-policy (workspace package license policy, shrink-only)" }
   134	          - { crate: service-catalog-parity,        label: "gate · catalog-liveness (PR-C3 founder live-OR-explicitly-marked policy, born-blocking, EMPTY frozen baseline)" }
   135	          - { crate: workspace-member-coverage, label: "gate: workspace-glob-coverage (ADR-0538)" }
   136	          - { crate: build-target-parity,           label: "gate · target-parity (ADR-0540, test-wiring false-green)" }
   137	          - { crate: hook-wiring,    label: "gate · enforcement-liveness (FRIC-012, hook mirror liveness)" }
   138	          - { crate: action-item-accounting,     label: "gate · friction-accounting (ADR-0544, closed-loop friction-ledger accounting)" }
   139	          - { crate: canonical-json,          label: "gate · canonical-json (ADR-0546, deterministic JSON serialization)" }
   140	          - { crate: parity-claim-evidence, label: "gate · hyperscaler-parity-taxonomy (cloud hyperscaler parity taxonomy, born-blocking)" }
   141	          - { crate: resource-contract-conformance, label: "gate · cloud-resource-contracts (Rust/API replacement for P0 cloud-resource Python validators)" }
   142	          - { crate: contract-slice-conformance, label: "gate · contract-slice-conformance (paved-road Rust/Buck2 replacement for scripts/tests/*_check.py contract-slice validators; ADR-0515/0523/0528)" }
   143	          - { crate: embedded-asset-hermeticity, label: "gate · embedded-asset-hermeticity (ADR-0545, include_str!/include_bytes! __srcs-tree mapping)" }
   144	          - { crate: core-dependency-isolation,           label: "gate · kernel-purity (ADR-0547, *-kernel/*-core zero transient-tech deps)" }
   145	          - { crate: crypto-backend-policy,   label: "gate · crypto-backend-purity (ADR-0506, ring forbidden / aws-lc-rs mandated — zero ring activation)" }
   146	          - { crate: graphql-usage-policy,  label: "gate · no-graphql-without-adr (ADR-0565, zero-GraphQL: no graphql lib / .graphql/.gql/.sdl reintroduction without a reversing ADR — candidate-tree evaluated, EMPTY frozen baseline)" }
   147	          - { crate: endpoint-authorization-coverage,          label: "gate · authz-coverage (issue #770 / AUTH-005, NEW unauthenticated HTTP control planes blocked vs frozen baseline)" }
   148	          - { crate: caller-supplied-authorization,         label: "gate · dto-authz-trust (ADR-0582, the CLASS-FIX for caller-supplied-authz-trust: a NEW fn that trusts a forged *Authorization DTO / x-authorization-* header in place of a server-side PDP decide() is blocked vs the frozen baseline of ~92 known instances; v2: FN-01/02/03/04/05/06 hardened)" }
   149	          - { crate: generated-artifact-policy, label: "gate · generated-artifact-control-plane (public hermetic CI artifact policy)" }
   150	          - { crate: build-cache-policy,            label: "gate · cache-wiring conformance (ADR-0560/ADR-0556, dark-wiring + cold floor + kill-switch)" }
   151	          - { crate: dependency-graph-acyclicity, label: "gate · substrate-dependency-dag-acyclicity (ADR-0280 §D-3, Tarjan SCC + Kahn topo-order + forbidden-edge honouring)" }
   152	          - { crate: service-tier-metadata,     label: "gate · tier-field-coverage (Phase-0 reorg ADR-0562/0536/0245, per-service tier/tier_subtype/dr_tier coverage + enum validity + no type-overload, born-blocking)" }
   153	          - { crate: layer-dependency-acyclicity, label: "gate · tier-dependency-acyclicity (Phase-0 reorg ADR-0245/0280/0562, cargo+buck crate-graph tier rules + S-rank + Tarjan cycle backstop, born-ADVISORY vs frozen baseline → enforce-no-regression)" }
   154	          - { crate: module-membership,   label: "gate · capability-membership (Phase-0 reorg ADR-0562 §6, the anti-junk-drawer MEMBERSHIP lint: every crate → exactly one registered capability/meta home, no NEW unmapped crate, no NEW top-level dir, base/-admission; born-advisory + enforce-no-regression vs the frozen unmapped baseline)" }
   155	          - { crate: runner-disk-reclaim,     label: "gate · runner-disk-reclaim conformance (FRIC-017 productization, ADR-0548 pipeline-as-product: policy parse + threshold/INFRA-RED discrimination + reclaim plan)" }
   156	          - { crate: port-placement,          label: "gate · port-placement (ADR-0570, clean-arch ports-in-core: no storage-port trait DEFINED in an */adapters/* crate — productizes the #116 defect class; born-advisory + enforce-no-regression vs frozen baseline)" }
   157	          - { crate: repo-root-hygiene,  label: "gate · root-workspace-hygiene (ADR-0600, allowlist-as-DATA default-DENY: every TRACKED repo-root file must match the allowlist + every top-level dir must be a permitted capability/meta home — makes committed root scratch structurally impossible; complements the scratch DENYLIST)" }
   158	          - { crate: dependency-automation, label: "gate · dependency-automation (ADR-0535, owned oya-deps.toml Rust bump-bot contract; external bot configs remain absent)" }
   159	          - { crate: supply-chain-audit,    label: "gate · supply-chain-audit (owned RustSec advisory scan over vendored mirror, born-blocking)" }
   160	          - { crate: feature-maturity-policy, label: "gate · planned-maturity (GH #992, product PRD acceptance/verification contracts + rich capability records + retired-plan provenance boundary)" }
   161	          - { crate: operator-secret-rbac, label: "gate · operator-secret-bootstrap (GH #980 + GH #988 / ADR-0606, least-privilege secret RBAC, declarative join-token bootstrap, ESO/OpenBao role+namespace+prefix scope, and plaintext OpenBao NetworkPolicy isolation)" }
   162	          - { crate: policy-deploy-parity, label: "gate · cedar-deploy-parity (GH #16 / ADR-0608, deployed-vs-authored Cedar parity: no deployed ConfigMap permit may leave the action unconstrained (action-agnostic blanket grant) and every deployed permit MUST be ⊆ the capability's authored <cap>/{policy,cedar}/*.cedar set; fail-closed on missing-authored/un-extractable/empty-scan; GH #16 byte-identical blanket ConfigMaps grandfathered in a documented shrink-only baseline pending the blanket-disarm follow-up)" }
   163	          - { crate: topology-manifest-contract, label: "gate · cell-topology-manifest-contract (CELL-001R manifest contract)" }
   164	          - { crate: automation-language-policy, label: "gate · rust-first automation hygiene (cloud-native infra anti-patterns: scripts, workflow shell, retired interpreters, and new CLI packages)" }
   165	          - { crate: gate-self-conformance, label: "gate · gate-self-conformance (GH #777, pipeline-as-product 7-property bar over every gate)" }
   166	    steps:
   167	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   168	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   169	        with:
   170	          persist-credentials: false
   171	          fetch-depth: 0
   172	      # Consume the producer-regen artifact (faces + volatile scm snapshot) instead of
   173	      # re-materializing per leg — see the FACES VIA ARTIFACT note on this job. The download
   174	      # restores the same regenerated bytes the producer derived from this run's candidate
   175	      # tree; registry-drift separately proves that derivation is byte-deterministic.
   176	      - name: Download regenerated faces (producer-regen artifact, ADR-0556 D5 QW-1)
   177	        # actions/download-artifact@v8.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   178	        uses: actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c
   179	        with:
   180	          name: accounting-faces
   181	          path: ci/facade
   182	      - name: Install buck2 (digest-pinned prebuilt release)
   183	        run: infra/ci/install-buck2.sh
   184	      # Buck2/rustc still use the pinned workspace toolchain + components; provision it before
   185	      # the test action starts to avoid concurrent rustup writes inside parallel Buck2 actions.
   186	      - name: Pre-provision pinned Rust toolchain for Buck2 gate tests
   187	        run: |
   188	          set -euo pipefail
   189	          rustup toolchain install
   190	          rustc --version
   191	      - name: buck2 test ${{ matrix.crate }}
   192	        run: |
   193	          set -euo pipefail
   194	          buck2 test \
   195	            //ci/facade/${{ matrix.crate }}:ci-${{ matrix.crate }}-unittest \
   196	            //ci/facade/${{ matrix.crate }}:ci-${{ matrix.crate }}-gate
   197	
   198	  # ── freshness: first-diagnosis gate for the two stale-output failures from PR #662.
   199	  #    Runs as its own fast job with no needs edge so stale Cargo.lock and stale generated faces
   200	  #    surface together before the broader Buck2 lanes spend a full matrix round-trip.
   201	  gate-generated-artifact-freshness:
   202	    name: freshness (lock + generated faces, ADR-0539)
   203	    runs-on: ubuntu-latest
   204	    steps:
   205	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   225	          set -euo pipefail
   226	          rustup toolchain install
   227	          rustc --version
   228	      - name: Run freshness gate
   229	        run: |
   230	          set -euo pipefail
   231	          freshness_bin="$(buck2 build //ci/facade/generated-artifact-freshness:oya-cloud-ci-freshness-app-bin --show-output | awk '{print $2}')"
   232	          "${freshness_bin}" --repo-root .
   233	
   234	  # ── registry-drift: materialized workspace == regenerated byte-equal. Starts at t=0 alongside
   235	  #    producer-regen; it rematerializes in-job so it is hermetic and self-contained. The
   236	  #    producer-regen needs-edge was cosmetic (evidence only, nothing consumed) and serialized
   237	  #    this job unnecessarily — removed so it starts at t=0.
   238	  gate-inventory-registry-drift:
   239	    name: registry-drift (materialized == regenerated)
   240	    runs-on: ubuntu-latest
   241	    steps:
   242	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   243	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   244	        with:
   245	          persist-credentials: false
   246	          # Full history: the accounting producer derives last_touch_commit via
   247	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
   248	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
   249	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
   250	          # and identical to a full local clone.
   251	          fetch-depth: 0
   252	      - name: Install buck2 (digest-pinned prebuilt release)
   253	        run: infra/ci/install-buck2.sh
   254	      # HERMETICITY CONTRACT (ADR-0556 D5 QW-1 deliberate exception): this gate IS the
   255	      # byte-parity detector (committed == regenerated), so it MUST regenerate in-job —
   256	      # feeding it the producer-regen artifact it is supposed to verify would make the
   257	      # check self-referential. Detectors never consume the thing they attest.
   258	      - name: Materialize faces then assert byte-parity
   259	        run: |
   260	          rustup toolchain install
   261	          rustc --version
   262	          buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   263	          buck2 test //ci/facade/inventory-registry-drift:ci-inventory-registry-drift-gate
   264	
   265	  # ── cloud-ci-firewall: the baseline ratchet (blocks only NEW debt) + the gate-registration
   266	  #    meta-test (no in-tree gate may go unregistered in this workflow). This is the surface-all
   267	  #    runner; per the runbook the existing firewall runner suffices — no separate aggregator bin
   268	  #    is required for PRE-work.
   269	  gate-baseline-ratchet:
   270	    name: cloud-ci-firewall (baseline ratchet + gate-registration meta-test)
   271	    runs-on: ubuntu-latest
   272	    steps:
   273	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   274	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   275	        with:
   276	          persist-credentials: false
   277	          # Full history: the accounting producer derives last_touch_commit via
   278	          # `git log --name-only`, and the staleness gate ages rows via `git log %H %ct`.
   279	          # A shallow (depth-1) checkout truncates history to HEAD -> ages collapse to 0
   280	          # and git-derived faces degrade (false-green). fetch-depth:0 keeps CI reproducible
   281	          # and identical to a full local clone.
   282	          fetch-depth: 0
   283	      - name: Install buck2 (digest-pinned prebuilt release)
   284	        run: infra/ci/install-buck2.sh
   285	      # HERMETICITY CONTRACT (ADR-0551 frozen-policy-wins): the firewall's frozen reference —
   286	      # the merge-base baseline snapshot — is materialized per-job BY DESIGN via the emitter's
   287	      # out-of-band bootstrap ref, and is deliberately absent from the producer-regen artifact.
   288	      # This lane therefore KEEPS its own materialization (ADR-0556 D5 cold-must-stay list);
   289	      # it is never converted to artifact reuse.
   290	      - name: Materialize cloud-ci generated faces
   291	        run: rustup toolchain install && rustc --version && buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   292	      - name: buck2 test cloud-ci-firewall
   293	        run: |
   294	          set -euo pipefail
   295	          buck2 test \
   296	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-unittest \
   297	            //ci/facade/baseline-ratchet:oya-cloud-ci-firewall-signoff-fixer-unittest \
   298	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-gate \
   299	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-gate-registration \
   300	            //ci/facade/baseline-ratchet:ci-baseline-ratchet-run-observability-packet
   301	
   302	  # ── GENERATED OUTPUT DIFF POLICY. Generated files may be deleted to retire a tracked output,
   303	  #    but PRs must not add/modify generated outputs as merge surfaces. Classification comes from
   304	  #    registry/generated-artifact-control-plane.json `generated_path_rules` so adopters can encode
   305	  #    their generated-output conventions once; .gitignore is preventive hygiene, not policy
   306	  #    authority. The candidate workspace is regenerated by cloud-ci before gates consume it.
   307	  generated-output-diff-policy:
   308	    name: generated-output-diff-policy (no generated merge surfaces)
   309	    runs-on: ubuntu-latest
   310	    steps:
   311	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   312	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   313	        with:
   314	          persist-credentials: false
   315	          fetch-depth: 0
   316	      - name: Install buck2 (digest-pinned prebuilt release)
   317	        run: infra/ci/install-buck2.sh
   318	      # Warm the pinned toolchain across runs (ADR-0556 D5 QW-4); rustup still resolves and
   319	      # validates the toolchain on every run.
   320	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   321	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   322	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   323	        with:
   324	          path: |
   325	            ~/.rustup/toolchains
   326	            ~/.rustup/update-hashes
   327	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   328	      - name: Pre-provision pinned Rust toolchain for Buck2 policy binary
   329	        run: |
   330	          set -euo pipefail
   331	          rustup toolchain install
   332	          rustc --version
   333	      - name: Reject non-deletion generated output edits
   334	        env:
   335	          EVENT_NAME: ${{ github.event_name }}
   336	          BASE_REF: ${{ github.base_ref || 'dev' }}
   337	        run: |
   338	          set -euo pipefail
   339	          if [ "${EVENT_NAME}" = "push" ]; then
   340	            echo "generated-output-diff-policy: push event; presubmit diff policy not applicable."
   341	            exit 0
   342	          fi
   343	          git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}"
   344	          policy_bin="$(buck2 build //ci/facade/generated-artifact-policy:oya-cloud-ci-generated-output-diff-policy --show-output | awk '{print $2}')"
   345	          # --find-renames is REQUIRED, not cosmetic: the policy's sanctioned-relocation exemption
   346	          # only accepts byte-identical (R100) renames of declared generated artifacts (a capability
   347	          # move relocating the firewall's frozen gate-baseline). Without explicit rename detection a
   348	          # runner with diff.renames=off would surface the move as A+D and RED the legit move (fails
   349	          # safe, but false-blocks). Detection ON + the R100-only exemption is the correct behavior.
   350	          git diff --find-renames --name-status "origin/${BASE_REF}"...HEAD \
   351	            | "${policy_bin}" --manifest registry/generated-artifact-control-plane.json
   352	
   353	  # ── HERMETIC BUCK2 LANE (OYA-CI-HERMETIC-EXECUTION-DESIGN §3 + Stage P1/P2). Runs the SAME
   354	  #    gate logic through buck2: `buck2 build` compiles every
   355	  #    target (the env!CARGO eradication) and `buck2 test` runs the gate rust_tests fully
   356	  #    hermetically (no ambient git in any action — the producer reads the materialized scm-facts
   357	  #    face; the scm-facts emitter is the single out-of-graph boundary, run in the
   358	  #    materialization step BELOW, never inside a cacheable action). Scoped by the
   359	  #    affected-set driver (`infra/ci/buck2-affected-gate.sh`: uquery owner -> rdeps closure,
   360	  #    FAILS CLOSED) for speed. RBE/NativeLink is staged LAST (D4) and NOT required for
   361	  #    hermeticity — local-on-runner execution via the wired `noop_test_toolchain` is sufficient
   362	  #    here. This lane feeds the same fan-in as the targeted Buck2 gate matrix above.
   363	  buck2:
   364	    name: buck2 (hermetic build + affected gate tests)
   365	    runs-on: ubuntu-latest
   920	  # ── LIVE-POSTGRES DURABLE-SUBSTRATE LANES (#101/#901). Runs the env-gated
   921	  #    cross-tenant-deny / RLS / CDC / SCIM durability integration tests against
   922	  #    CONTAINERIZED Postgres and GATES merge. Both adapter and facade groups block
   923	  #    the single required `oya-ci-required` context.
   924	  #
   925	  #    SPLIT SAFETY (#901): adapter and facade groups run in parallel only because
   926	  #    each job owns an independent Postgres service container and repeats the
   927	  #    deterministic bootstrap. Inside each group, `--num-threads 1` and sequential
   928	  #    target invocations remain because the harnesses in that group still share a
   929	  #    local database.
   930	  gate-live-postgres-adapters:
   931	    name: "gate-live-postgres-adapters (durable adapters: RLS / CDC / SCIM, #901)"
   932	    runs-on: ubuntu-latest
   933	    timeout-minutes: 25
   934	    services:
   935	      postgres:
   936	        image: postgres:16
   937	        env:
   938	          POSTGRES_USER: postgres
   939	          POSTGRES_PASSWORD: postgres
   940	          POSTGRES_DB: oyatie
   941	        ports:
   942	          - 5432:5432
   943	        options: >-
   944	          --health-cmd "pg_isready -U postgres -d oyatie"
   945	          --health-interval 5s
   946	          --health-timeout 5s
   947	          --health-retries 20
   948	    env:
   949	      OYA_PG_ADMIN_URL: postgres://postgres:postgres@127.0.0.1:5432/oyatie
   950	      OYA_PG_APP_URL: postgres://oya_app:app@127.0.0.1:5432/oyatie
   951	    steps:
   952	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   953	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
   954	        with:
   955	          persist-credentials: false
   956	      - name: Install buck2 (digest-pinned prebuilt release)
   957	        run: infra/ci/install-buck2.sh
   958	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
   959	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   960	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
   961	        with:
   962	          path: |
   963	            ~/.rustup/toolchains
   964	            ~/.rustup/update-hashes
   965	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
   966	      - name: Pre-provision pinned Rust toolchain for Buck2 live tests
   967	        run: |
   968	          set -euo pipefail
   969	          rustup toolchain install
   970	          rustc --version
   971	      - name: Install postgresql-client for the bootstrap
   972	        run: |
   973	          set -euo pipefail
   974	          sudo apt-get update
   975	          sudo apt-get install -y --no-install-recommends postgresql-client
   976	          psql --version
   977	      - name: Bootstrap app role + durable schemas/roles (admin, adapters)
   978	        env:
   979	          PGPASSWORD: postgres
   980	        run: |
   981	          set -euo pipefail
   982	          ADMIN_PSQL=(psql -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U postgres -d oyatie)
   983	          "${ADMIN_PSQL[@]}" -c "DO \$\$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='oya_app') THEN CREATE ROLE oya_app LOGIN PASSWORD 'app' NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE; END IF; END \$\$;"
   984	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql
   985	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql
   986	          "${ADMIN_PSQL[@]}" -c "GRANT tenancy_lifecycle_runtime TO oya_app;"
   987	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql
   988	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql
   989	          "${ADMIN_PSQL[@]}" -c "GRANT identity_scim_runtime TO oya_app;"
   990	          "${ADMIN_PSQL[@]}" -c "SELECT rolname, rolsuper, rolbypassrls FROM pg_roles WHERE rolname IN ('postgres','oya_app','tenancy_lifecycle_runtime','identity_scim_runtime') ORDER BY rolname;"
   991	          server_version="$("${ADMIN_PSQL[@]}" -Atqc "SHOW server_version;")"
   992	          cat > "${RUNNER_TEMP}/live-postgres-adapters-bootstrap-provenance.json" <<JSON
   993	          {
   994	            "schema_version": 2,
   995	            "artifact_type": "cloud_ci_operator_artifact",
   996	            "artifact_id": "live-postgres-bootstrap-provenance",
   997	            "gate_id": "gate-live-postgres-adapters",
   998	            "lane": "adapters",
   999	            "postgres": {
  1000	              "image": "postgres:16",
  1001	              "server_version": "${server_version}",
  1002	              "database": "oyatie",
  1003	              "host": "127.0.0.1",
  1004	              "port": 5432
  1005	            },
  1006	            "roles": [
  1007	              {"name": "postgres", "purpose": "admin bootstrap superuser; DSN/password redacted"},
  1008	              {"name": "oya_app", "purpose": "non-superuser NOBYPASSRLS app login; DSN/password redacted"},
  1009	              {"name": "tenancy_lifecycle_runtime", "purpose": "tenancy runtime role granted to oya_app"},
  1010	              {"name": "identity_scim_runtime", "purpose": "SCIM runtime role granted to oya_app"}
  1011	            ],
  1012	            "migrations": [
  1013	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql",
  1014	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql",
  1015	              "iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql",
  1016	              "iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql"
  1017	            ],
  1018	            "source_revision": "${GITHUB_SHA}",
  1019	            "retention_and_pii": {
  1020	              "retention_days": 30,
  1021	              "pii": "none; local CI database metadata and repo paths only",
  1022	              "secret_redaction": "admin/app DSNs, passwords, tenant secrets, idempotency keys, and tokens are not emitted"
  1023	            }
  1024	          }
  1025	          JSON
  1026	      - name: buck2 test — durable adapters (admin=superuser, app=app-role)
  1027	        env:
  1028	          OYA_DATA_LIVE_POSTGRES: "1"
  1029	          OYA_DATA_POSTGRES_ADMIN_URL: ${{ env.OYA_PG_ADMIN_URL }}
  1030	          OYA_DATA_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1031	          OYA_OUTBOX_LIVE_POSTGRES: "1"
  1032	          OYA_OUTBOX_POSTGRES_ADMIN_URL: ${{ env.OYA_PG_ADMIN_URL }}
  1033	          OYA_OUTBOX_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1034	          OYA_BACKBONE_LIVE_POSTGRES: "1"
  1035	          OYA_BACKBONE_POSTGRES_URL: ${{ env.OYA_PG_ADMIN_URL }}
  1036	          OYA_BACKBONE_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1037	        run: |
  1038	          set -euo pipefail
  1039	          LIVE_ENV=(
  1040	            --env RUST_TEST_THREADS=1
  1041	            --env OYA_DATA_LIVE_POSTGRES="${OYA_DATA_LIVE_POSTGRES}"
  1042	            --env OYA_DATA_POSTGRES_ADMIN_URL="${OYA_DATA_POSTGRES_ADMIN_URL}"
  1043	            --env OYA_DATA_POSTGRES_APP_URL="${OYA_DATA_POSTGRES_APP_URL}"
  1044	            --env OYA_OUTBOX_LIVE_POSTGRES="${OYA_OUTBOX_LIVE_POSTGRES}"
  1045	            --env OYA_OUTBOX_POSTGRES_ADMIN_URL="${OYA_OUTBOX_POSTGRES_ADMIN_URL}"
  1046	            --env OYA_OUTBOX_POSTGRES_APP_URL="${OYA_OUTBOX_POSTGRES_APP_URL}"
  1047	            --env OYA_BACKBONE_LIVE_POSTGRES="${OYA_BACKBONE_LIVE_POSTGRES}"
  1048	            --env OYA_BACKBONE_POSTGRES_URL="${OYA_BACKBONE_POSTGRES_URL}"
  1049	            --env OYA_BACKBONE_POSTGRES_APP_URL="${OYA_BACKBONE_POSTGRES_APP_URL}"
  1050	          )
  1051	          buck2 test --local-only --num-threads 1 //libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest -- "${LIVE_ENV[@]}"
  1052	          buck2 test --local-only --num-threads 1 //libs/oya-data-outbox-adapter-postgres:oya-data-outbox-adapter-postgres-unittest -- "${LIVE_ENV[@]}"
  1053	          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-unittest -- "${LIVE_ENV[@]}"
  1054	          buck2 test --local-only --num-threads 1 //tenancy/adapters/tenant-lifecycle-store-postgres:tenancy-tenant-lifecycle-store-postgres-live -- "${LIVE_ENV[@]}"
  1055	          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-unittest -- "${LIVE_ENV[@]}"
  1056	          buck2 test --local-only --num-threads 1 //iam/adapters/identity-scim-store-postgres:identity-scim-store-postgres-live -- "${LIVE_ENV[@]}"
  1057	      - name: Upload live-postgres adapter bootstrap provenance
  1058	        if: always()
  1059	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1060	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  1061	        with:
  1062	          name: live-postgres-adapters-bootstrap-provenance
  1063	          path: ${{ runner.temp }}/live-postgres-adapters-bootstrap-provenance.json
  1064	          retention-days: 30
  1065	          if-no-files-found: warn
  1066	
  1067	  gate-live-postgres-facades:
  1068	    name: "gate-live-postgres-facades (durable facades: tenant lifecycle / SCIM, #901)"
  1069	    runs-on: ubuntu-latest
  1070	    timeout-minutes: 25
  1071	    services:
  1072	      postgres:
  1073	        image: postgres:16
  1074	        env:
  1075	          POSTGRES_USER: postgres
  1076	          POSTGRES_PASSWORD: postgres
  1077	          POSTGRES_DB: oyatie
  1078	        ports:
  1079	          - 5432:5432
  1080	        options: >-
  1081	          --health-cmd "pg_isready -U postgres -d oyatie"
  1082	          --health-interval 5s
  1083	          --health-timeout 5s
  1084	          --health-retries 20
  1085	    env:
  1086	      OYA_PG_ADMIN_URL: postgres://postgres:postgres@127.0.0.1:5432/oyatie
  1087	      OYA_PG_APP_URL: postgres://oya_app:app@127.0.0.1:5432/oyatie
  1088	    steps:
  1089	      # actions/checkout@v6.0.3 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1090	      - uses: actions/checkout@df4cb1c069e1874edd31b4311f1884172cec0e10
  1091	        with:
  1092	          persist-credentials: false
  1093	      - name: Install buck2 (digest-pinned prebuilt release)
  1094	        run: infra/ci/install-buck2.sh
  1095	      - name: Cache pinned Rust toolchain (ADR-0556 D5 QW-4)
  1096	        # actions/cache@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1097	        uses: actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae
  1098	        with:
  1099	          path: |
  1100	            ~/.rustup/toolchains
  1101	            ~/.rustup/update-hashes
  1102	          key: rustup-toolchain-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}
  1103	      - name: Pre-provision pinned Rust toolchain for Buck2 live tests
  1104	        run: |
  1105	          set -euo pipefail
  1106	          rustup toolchain install
  1107	          rustc --version
  1108	      - name: Install postgresql-client for the bootstrap
  1109	        run: |
  1110	          set -euo pipefail
  1111	          sudo apt-get update
  1112	          sudo apt-get install -y --no-install-recommends postgresql-client
  1113	          psql --version
  1114	      - name: Bootstrap app role + durable schemas/roles (admin, facades)
  1115	        env:
  1116	          PGPASSWORD: postgres
  1117	        run: |
  1118	          set -euo pipefail
  1119	          ADMIN_PSQL=(psql -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U postgres -d oyatie)
  1120	          "${ADMIN_PSQL[@]}" -c "DO \$\$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='oya_app') THEN CREATE ROLE oya_app LOGIN PASSWORD 'app' NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE; END IF; END \$\$;"
  1121	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql
  1122	          "${ADMIN_PSQL[@]}" -f tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql
  1123	          "${ADMIN_PSQL[@]}" -c "GRANT tenancy_lifecycle_runtime TO oya_app;"
  1124	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql
  1125	          "${ADMIN_PSQL[@]}" -f iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql
  1126	          "${ADMIN_PSQL[@]}" -c "GRANT identity_scim_runtime TO oya_app;"
  1127	          "${ADMIN_PSQL[@]}" -c "SELECT rolname, rolsuper, rolbypassrls FROM pg_roles WHERE rolname IN ('postgres','oya_app','tenancy_lifecycle_runtime','identity_scim_runtime') ORDER BY rolname;"
  1128	          server_version="$("${ADMIN_PSQL[@]}" -Atqc "SHOW server_version;")"
  1129	          cat > "${RUNNER_TEMP}/live-postgres-facades-bootstrap-provenance.json" <<JSON
  1130	          {
  1131	            "schema_version": 2,
  1132	            "artifact_type": "cloud_ci_operator_artifact",
  1133	            "artifact_id": "live-postgres-bootstrap-provenance",
  1134	            "gate_id": "gate-live-postgres-facades",
  1135	            "lane": "facades",
  1136	            "postgres": {
  1137	              "image": "postgres:16",
  1138	              "server_version": "${server_version}",
  1139	              "database": "oyatie",
  1140	              "host": "127.0.0.1",
  1141	              "port": 5432
  1142	            },
  1143	            "roles": [
  1144	              {"name": "postgres", "purpose": "admin bootstrap superuser; DSN/password redacted"},
  1145	              {"name": "oya_app", "purpose": "non-superuser NOBYPASSRLS app login; DSN/password redacted"},
  1146	              {"name": "tenancy_lifecycle_runtime", "purpose": "tenancy runtime role granted to oya_app"},
  1147	              {"name": "identity_scim_runtime", "purpose": "SCIM runtime role granted to oya_app"}
  1148	            ],
  1149	            "migrations": [
  1150	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sql",
  1151	              "tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0001_tenant_lifecycle_store.sql",
  1152	              "iam/adapters/identity-scim-store-postgres/migrations/0000_runtime_role.sql",
  1153	              "iam/adapters/identity-scim-store-postgres/migrations/0001_identity_scim_store.sql"
  1154	            ],
  1155	            "source_revision": "${GITHUB_SHA}",
  1156	            "retention_and_pii": {
  1157	              "retention_days": 30,
  1158	              "pii": "none; local CI database metadata and repo paths only",
  1159	              "secret_redaction": "admin/app DSNs, passwords, tenant secrets, idempotency keys, and tokens are not emitted"
  1160	            }
  1161	          }
  1162	          JSON
  1163	      - name: buck2 test — durable facades (live test = app-role, non-live = in-memory)
  1164	        env:
  1165	          OYA_BACKBONE_LIVE_POSTGRES: "1"
  1166	          OYA_BACKBONE_POSTGRES_APP_URL: ${{ env.OYA_PG_APP_URL }}
  1167	        run: |
  1168	          set -euo pipefail
  1169	          FACADE_ENV=(
  1170	            --env RUST_TEST_THREADS=1
  1171	            --env OYA_BACKBONE_LIVE_POSTGRES="${OYA_BACKBONE_LIVE_POSTGRES}"
  1172	            --env OYA_BACKBONE_POSTGRES_APP_URL="${OYA_BACKBONE_POSTGRES_APP_URL}"
  1173	          )
  1174	          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-tests -- "${FACADE_ENV[@]}"
  1175	          buck2 test --local-only --num-threads 1 //iam/facade/identity-service:iam-identity-service-e2e -- "${FACADE_ENV[@]}"
  1176	          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-unittest -- "${FACADE_ENV[@]}"
  1177	          buck2 test --local-only --num-threads 1 //tenancy/facade/tenant-lifecycle-app:tenancy-tenant-lifecycle-app-acceptance -- "${FACADE_ENV[@]}"
  1178	      - name: Upload live-postgres facade bootstrap provenance
  1179	        if: always()
  1180	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
  1181	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  1182	        with:
  1183	          name: live-postgres-facades-bootstrap-provenance
  1184	          path: ${{ runner.temp }}/live-postgres-facades-bootstrap-provenance.json
  1185	          retention-days: 30
  1186	          if-no-files-found: warn
  1187	
  1188	
  1189	
  1190	  # ── THE FAN-IN. This is the single required context branch protection keys on. It has NO
exec
/bin/zsh -lc "git show origin/dev:infra/ci/install-buck2.sh | nl -ba && git ls-tree -r --name-only origin/dev -- infra/ci | rg 'buck2-affected-gate|materialize-cloud-ci-generated-faces|install-buck2' && git show origin/dev:infra/ci/buck2-affected-gate.sh | nl -ba | sed -n '1,180p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	#!/usr/bin/env bash
     2	# Install the digest-pinned Buck2 release used by the canonical cloud-ci bridge.
     3	set -euo pipefail
     4	
     5	BUCK2_RELEASE="${BUCK2_RELEASE:-2026-06-01}"
     6	BUCK2_INSTALL_DIR="${BUCK2_INSTALL_DIR:-/tmp/oya-ci-buck2-${BUCK2_RELEASE}}"
     7	
     8	case "$(uname -s)-$(uname -m)" in
     9	  Linux-x86_64)
    10	    BUCK2_ASSET="${BUCK2_ASSET:-buck2-x86_64-unknown-linux-gnu.zst}"
    11	    BUCK2_SHA256="${BUCK2_SHA256:-4dd9ae54c87fdcf795101074f8788232af55523885135d5e3358c77365993555}"
    12	    ;;
    13	  *)
    14	    if [ "${OYA_CI_ALLOW_AMBIENT_BUCK2:-}" = "1" ] && command -v buck2 >/dev/null 2>&1; then
    15	      echo "Using ambient buck2 only because OYA_CI_ALLOW_AMBIENT_BUCK2=1 was set." >&2
    16	      buck2 --version
    17	      exit 0
    18	    fi
    19	    echo "Unsupported host for default pinned Buck2 install; set OYA_CI_ALLOW_AMBIENT_BUCK2=1 for local advisory use." >&2
    20	    exit 1
    21	    ;;
    22	esac
    23	
    24	mkdir -p "${BUCK2_INSTALL_DIR}"
    25	
    26	# Cache-hit fast path (ADR-0556 D5 QW-4: the tool binary is a digest-pinned INPUT, not a build
    27	# output — warm-eligible velocity). If the compressed release asset is already present (e.g.
    28	# restored by actions/cache) and its bytes match the pinned SHA-256, skip the network download.
    29	# A present-but-mismatching asset is discarded and re-downloaded.
    30	asset_path="${BUCK2_INSTALL_DIR}/${BUCK2_ASSET}"
    31	if [ -f "${asset_path}" ] \
    32	  && echo "${BUCK2_SHA256}  ${asset_path}" | sha256sum -c - >/dev/null 2>&1; then
    33	  echo "buck2 release asset cache hit (SHA-256 verified): ${asset_path} — skipping download." >&2
    34	else
    35	  rm -f "${asset_path}"
    36	  curl --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 20 -fsSL "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/${BUCK2_ASSET}" -o "${asset_path}"
    37	fi
    38	
    39	# Integrity is non-negotiable (ADR-0556: the SHA check is the integrity anchor that makes the
    40	# warm path admissible). The pinned-digest verification ALWAYS runs on the exact bytes about to
    41	# be decompressed and executed — cached and fresh paths alike — and the executable is ALWAYS
    42	# re-derived from those verified bytes (never trusted as a loose cached binary).
    43	echo "${BUCK2_SHA256}  ${asset_path}" | sha256sum -c -
    44	zstd -d -f "${asset_path}" -o "${BUCK2_INSTALL_DIR}/buck2"
    45	chmod +x "${BUCK2_INSTALL_DIR}/buck2"
    46	
    47	if [ -n "${GITHUB_PATH:-}" ]; then
    48	  echo "${BUCK2_INSTALL_DIR}" >> "${GITHUB_PATH}"
    49	fi
    50	
    51	"${BUCK2_INSTALL_DIR}/buck2" --version
infra/ci/buck2-affected-gate.sh
infra/ci/install-buck2.sh
     1	#!/bin/sh
     2	# buck2-native affected-only CI gate.
     3	#
     4	# Builds + tests the reverse-dependency closure of the PR's changed files —
     5	# the hyperscaler "affected targets" pattern (Google/Meta), buck2-native via
     6	# `uquery owner()` + `rdeps()`. Replaces the cargo-era `oya verify --affected`.
     7	# No oya-dev-cli dependency.
     8	#
     9	# Usage:  buck2-affected-gate.sh <base-ref> [head-ref]
    10	#         base-ref  — the merge-base anchor (e.g. origin/dev)
    11	#         head-ref  — the tip to diff (default: HEAD)
    12	#
    13	# The 1-arg form (buck2-affected-gate.sh origin/dev) diffs the current
    14	# checkout: HEAD is the PR checkout in the GitHub Actions runner, so omitting
    15	# head-ref is the default invocation.
    16	#
    17	# The 2-arg form (buck2-affected-gate.sh origin/dev origin/pr-N) is used by
    18	# the controller Job, where the working tree is trunk (dev) and the PR ref
    19	# is fetched as data via `git fetch origin refs/pull/N/head:refs/remotes/origin/pr-N`.
    20	#
    21	# Exit 0 = pass (incl. non-Rust / no-affected PRs); non-zero = build/test failure.
    22	set -eu
    23	
    24	BASE="${1:-origin/dev}"
    25	HEAD_REF="${2:-HEAD}"
    26	BUCK2="${BUCK2:-buck2}"
    27	
    28	echo "buck2-affected-gate: start (pwd=$(pwd) base=$BASE head-ref=$HEAD_REF resolved=$(git rev-parse --short "$HEAD_REF" 2>&1))"
    29	echo "buck2-affected-gate: .buckconfig=$(test -f .buckconfig && echo present || echo MISSING) HOME=${HOME:-unset} buck2=$($BUCK2 --version 2>&1 | head -1)"
    30	if ! git rev-parse --verify --quiet "$BASE" >/dev/null 2>&1; then
    31	  echo "buck2-affected-gate: FATAL base ref '$BASE' does not resolve in this checkout"
    32	  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
    33	  exit 1
    34	fi
    35	if ! git rev-parse --verify --quiet "$HEAD_REF" >/dev/null 2>&1; then
    36	  echo "buck2-affected-gate: FATAL head ref '$HEAD_REF' does not resolve in this checkout"
    37	  echo "  remotes: $(git remote 2>&1)  | refs: $(git for-each-ref --format='%(refname)' refs/remotes 2>&1 | paste -sd' ' -)"
    38	  exit 1
    39	fi
    40	if ! MERGE_BASE=$(git merge-base "$HEAD_REF" "$BASE" 2>&1); then
    41	  echo "buck2-affected-gate: FATAL merge-base $HEAD_REF $BASE failed (need full history): $MERGE_BASE"
    42	  exit 1
    43	fi
    44	CHANGED=$(git diff --name-only "$MERGE_BASE" "$HEAD_REF")
    45	if [ -z "$CHANGED" ]; then
    46	  echo "buck2-affected-gate: no changed files vs $BASE ($HEAD_REF) -> PASS"
    47	  exit 0
    48	fi
    49	echo "buck2-affected-gate: $(printf '%s\n' "$CHANGED" | wc -l | tr -d ' ') changed file(s) vs $BASE..${HEAD_REF} (merge-base $MERGE_BASE)"
    50	
    51	# Classify. Only docs/non-graph files (e.g. .md/.yaml/.json outside crates) may
    52	# legitimately map to no target. A *.rs / Cargo.toml / buck-graph file MUST map to
    53	# a target — FAIL CLOSED if it doesn't (never silently pass a Rust change unbuilt).
    54	RUST_REL=$(printf '%s\n' "$CHANGED" | grep -E '\.rs$|/Cargo\.toml$|^Cargo\.(toml|lock)$|^\.buckconfig$|(^|/)BUCK$|^toolchains/|^third-party/' || true)
    55	if [ -z "$RUST_REL" ]; then
    56	  echo "buck2-affected-gate: no Rust/buck-graph files changed -> NoRust PASS"
    57	  exit 0
    58	fi
    59	
    60	# owner() resolution — batched to minimise buck2 daemon round-trips.
    61	#
    62	# Strategy:
    63	#   1. BUCK files: no owner() result by design (they ARE the package definition).
    64	#      Run a small per-file pass to expand each to its package target pattern.
    65	#      (One buck2 uquery per BUCK file — these are typically 0-1 files per PR.)
    66	#   2. Non-BUCK Rust/graph files: build ONE "owner('f1') union owner('f2') union ..."
    67	#      expression and run a single buck2 uquery call for all files at once.
    68	#      owner() takes file-path strings, not target-set placeholders, so %Ss/@argfile
    69	#      cannot be used here — the union expression is the correct single-call form.
    70	#      A uquery ERROR (non-zero exit) FAILS the gate — it is NOT 'no owner'.
    71	#      (The false-pass bug was: 2>/dev/null||true swallowed buck2 errors.)
    72	
    73	OWNERS=""
    74	
    75	# ── Pass 1: BUCK files → package target pattern (unchanged semantics, separate pass) ──
    76	BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -E '(^|/)BUCK$' || true)
    77	for f in $BUCK_FILES; do
    78	  [ -e "$f" ] || continue
    79	  d=$(dirname "$f")
    80	  case "$d" in
    81	    third-party)   pat="third-party//:" ;;
    82	    third-party/*) pat="third-party//${d#third-party/}:" ;;
    83	    toolchains)    pat="toolchains//:" ;;
    84	    toolchains/*)  pat="toolchains//${d#toolchains/}:" ;;
    85	    .)             pat="//:" ;;
    86	    *)             pat="//$d:" ;;
    87	  esac
    88	  if ! o=$("$BUCK2" uquery "$pat" 2>/tmp/uqerr); then
    89	    echo "buck2-affected-gate: FATAL buck2 uquery '$pat' (BUCK pkg for $f) errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
    90	  fi
    91	  [ -n "$o" ] && OWNERS="$OWNERS $o"
    92	done
    93	
    94	# ── Pass 2: non-BUCK files → ONE batched uquery call via union-of-owner() expression ──
    95	# Build: owner('f1') union owner('f2') union ... and run as a single buck2 uquery invocation.
    96	# This replaces N serial daemon round-trips (one per file) with a single round-trip.
    97	NON_BUCK_FILES=$(printf '%s\n' "$RUST_REL" | grep -vE '(^|/)BUCK$' || true)
    98	NON_BUCK_EXISTING=$(printf '%s\n' "$NON_BUCK_FILES" | while read -r f; do [ -e "$f" ] && printf '%s\n' "$f"; done)
    99	if [ -n "$NON_BUCK_EXISTING" ]; then
   100	  OWNER_EXPR=$(printf '%s\n' "$NON_BUCK_EXISTING" | \
   101	    awk 'NR==1{printf "owner('"'"'%s'"'"')", $0; next} {printf " union owner('"'"'%s'"'"')", $0}')
   102	  if ! o=$("$BUCK2" uquery "$OWNER_EXPR" 2>/tmp/uqerr); then
   103	    echo "buck2-affected-gate: FATAL buck2 uquery owner() errored:"; sed 's/^/    /' /tmp/uqerr; exit 1
   104	  fi
   105	  [ -n "$o" ] && OWNERS="$OWNERS $o"
   106	fi
   107	
   108	OWNERS=$(printf '%s\n' $OWNERS | sed '/^$/d' | sort -u)
   109	if [ -z "$OWNERS" ]; then
   110	  echo "buck2-affected-gate: FATAL Rust/buck files changed but NO owning target found (refusing to false-pass):"
   111	  printf '    %s\n' $RUST_REL
   112	  exit 1
   113	fi
   114	echo "buck2-affected-gate: $(printf '%s\n' "$OWNERS" | wc -l | tr -d ' ') owning target(s)"
   115	
   116	# Affected = changed targets + reverse-dep closure. rdeps error also FAILS closed.
   117	# Pass owners via @argfile + the %Ss set placeholder, NOT an inline set(...): a change
   118	# to a large BUCK package (e.g. third-party/BUCK owns 1689 targets) overflows the inline
   119	# query string and buck2 errors out (uquery RC=3, no build attempted) — which silently
   120	# blocked landing ANY third-party change. @argfile + %Ss handles an arbitrary set size
   121	# (verified: 1689 owners -> 1919 affected). One owner per line.
   122	printf '%s\n' $OWNERS | sed '/^$/d' > /tmp/gate-owners.txt
   123	if ! AFFECTED=$("$BUCK2" uquery 'rdeps(//..., %Ss)' @/tmp/gate-owners.txt 2>/tmp/rqerr); then
   124	  echo "buck2-affected-gate: FATAL rdeps query errored:"; sed 's/^/    /' /tmp/rqerr; exit 1
   125	fi
   126	N=$(printf '%s\n' "$AFFECTED" | sed '/^$/d' | wc -l | tr -d ' ')
   127	echo "buck2-affected-gate: $N affected target(s) (owners + reverse-dep closure)"
   128	if [ "$N" = "0" ]; then echo "buck2-affected-gate: FATAL owners found but rdeps empty (query problem)"; exit 1; fi
   129	
   130	# Build then test the affected set. @- reads the newline-delimited target list
   131	# from stdin, avoiding ARG_MAX limits on large closures.
   132	printf '%s\n' "$AFFECTED" | sed '/^$/d' > /tmp/affected-targets.txt
   133	echo "=== buck2 build (affected) ==="
   134	"$BUCK2" build @/tmp/affected-targets.txt
   135	echo "=== buck2 test (affected) ==="
   136	"$BUCK2" test @/tmp/affected-targets.txt
   137	echo "buck2-affected-gate: PASS"
exec
/bin/zsh -lc "git ls-tree -r --name-only origin/dev -- .claire | head -20 && git log --all --format='%h %cs %s' -- .claire/worktrees | head -10" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
f93a44f9b 2026-06-09 ci: productize generated output hygiene
e38624dc4 2026-06-07 checkpoint: full source tree pre-aggressive-cleanup — recovery anchor for the D-SSOT-CURRENT-TRUTH whole-tree cleanup (founder-authorized aggressive removal; over-delete recoverable from this commit)
exec
/bin/zsh -lc "git ls-tree -r --name-only origin/dev -- ci/facade | rg '\\.generated\\.json"'$'"' | head -80" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
exec
/bin/zsh -lc "git show origin/dev:registry/generated-artifact-control-plane.json | rg -n 'materialization_mode|generated_faces|ci/facade' | head -80" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
11:      "path": "ci/facade/artifact-inventory-registry/accounting-registry.generated.json",
13:      "materialization_mode": "not-tracked-in-git",
27:        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
42:      "path": "ci/facade/artifact-inventory-registry/decision-crosswalk.generated.json",
44:      "materialization_mode": "not-tracked-in-git",
56:        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
71:      "path": "ci/facade/artifact-inventory-registry/enforcement-inventory.generated.json",
73:      "materialization_mode": "not-tracked-in-git",
78:        "ci/facade/**",
85:        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
100:      "path": "ci/facade/artifact-inventory-registry/enforcement-liveness.generated.json",
102:      "materialization_mode": "not-tracked-in-git",
115:        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
130:      "path": "ci/facade/artifact-inventory-registry/gate-baseline.generated.json",
132:      "materialization_mode": "not-tracked-in-git",
137:        "ci/facade/baseline-ratchet/gate-baseline.signoff.json",
144:        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
159:      "path": "ci/facade/artifact-inventory-registry/scm-facts.generated.json",
161:      "materialization_mode": "not-tracked-in-git",
174:        "generator_target": "//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot",
186:      "path": "ci/facade/artifact-inventory-registry/ttl-policy.generated.json",
188:      "materialization_mode": "not-tracked-in-git",
199:        "generator_target": "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin",
216:      "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
244:      "materialization_mode": "not-tracked-in-git",
252:      "final_tree_validation": "ADR-0613 de-commit class: this face is NOT tracked in git; cloud-ci materializes it on demand from the checked-out candidate tree (//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin) before gates consume it. Required-CI freshness is enforced TRANSITIVELY by the generated-artifact-freshness gate, whose product-graph determinism canary regenerates masterplan then the dashboard twice per run (a masterplan regeneration failure is RED; masterplan nondeterminism surfaces as dashboard byte instability). The masterplan-drift lane (dev-cli local bridge) is feedback, not merge authority. Byte parity against a committed copy is intentionally retired for this face.",
272:      "materialization_mode": "not-tracked-in-git",
280:      "final_tree_validation": "ADR-0613 de-commit class: this face is NOT tracked in git; cloud-ci materializes it on demand from the checked-out candidate tree (//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin runs the arch-graph generator --write after masterplan) before gates consume it. The generated-artifact-freshness gate requires it to regenerate successfully and be byte-stable across two regenerations (determinism canary). Byte parity against a committed copy is intentionally retired for this face.",
300:      "materialization_mode": "not-tracked-in-git",
307:      "final_tree_validation": "ADR-0563-amended de-commit class (ADR-0614): this face is NOT tracked in git; cloud-ci materializes it on demand from the checked-out candidate tree. //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin runs the codemod `manifest` subcommand as step 1 (materialize_move_manifest) before the scm-facts emitter's rename-aware relabel consumes it. The registry-drift gate validates regenerate-twice determinism (two fresh codemod emissions of the manifest must be byte-identical), not committed-byte parity. Byte parity against a committed copy is intentionally retired for this face; the reviewed move surface is the committed move-plan (specs/reorg/<capability>-move-plan.json) plus git rename detection, not a committed bijection.",
325:      "path": "ci/facade/action-item-accounting/friction-accounting-baseline.json",
327:      "materialization_mode": "hand-curated-committed",
332:        "ci/facade/action-item-accounting/friction-accounting-policy.json"
334:      "final_tree_validation": "HAND-CURATED-COMMITTED (not producer-regenerated): the reviewed, hand-shrunk shrink-only ratchet reference for the friction-ledger legacy debt set frozen at ADR-0544 gate go-live. The live-repo gate test asserts the MEASURED legacy set equals these keys EXACTLY (set equality). It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree — a candidate recompute would erase the hand-shrunk burn-down and re-launder aged-out debt. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
339:      "path": "ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json",
341:      "materialization_mode": "hand-curated-committed",
346:        "ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json"
348:      "final_tree_validation": "HAND-CURATED-COMMITTED (not producer-regenerated): the reviewed, hand-shrunk shrink-only ratchet reference for the embedded-asset skip_* debt set frozen at ADR-0545 gate go-live. The live-repo gate test asserts the MEASURED skip set equals these keys EXACTLY (set equality). It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree — a candidate recompute would erase the hand-shrunk burn-down. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
353:      "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-baseline.json",
355:      "materialization_mode": "hand-curated-committed",
360:        "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json"
362:      "final_tree_validation": "HAND-CURATED-COMMITTED, FROZEN-INTENT: the frozen known-debt baseline of pre-existing tier-dependency violations; the gate reports these advisory and blocks only on a NEW code|subject not in the set (subset/no-regression semantics). It has a local `--emit-baseline` producer, but that producer MUST NOT own the committed face: a candidate recompute would absorb (LAUNDER) new regressions into the baseline. It MUST stay a committed git blob, burn down only by reviewed removal, and MUST NOT be de-committed to a candidate recompute. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
367:      "path": "ci/facade/port-placement/port-placement-baseline.json",
369:      "materialization_mode": "hand-curated-committed",
374:        "ci/facade/port-placement/port-placement-policy.json"
376:      "final_tree_validation": "HAND-CURATED-COMMITTED, hand-frozen (NOT a producer-emitted face): the frozen baseline of pre-existing storage-port traits defined in adapter crates; born-advisory + enforce-no-regression (subset, ratchet-down only), self-cleaning as ports relocate to their capability's core/ports crate. It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
383:      "materialization_mode": "hand-curated-committed",
390:      "final_tree_validation": "HAND-CURATED-COMMITTED: the hand/tool-curated known-warnings allowlist (<kind>\\t<token>) the glossary-vocabulary gate treats as accepted legacy warnings. It MUST stay a committed git blob and MUST NOT be recomputed over the candidate tree — a candidate recompute would launder new warnings into the allowlist. Declaring it with any de-commit materialization_mode is RED (hand_curated_ratchet_artifact_must_stay_committed).",
exec
/bin/zsh -lc "git show origin/dev:toolchains/BUCK | nl -ba | sed -n '1,80p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	load("@prelude//toolchains:rust.bzl", "system_rust_toolchain")
     2	load("@prelude//toolchains:cxx.bzl", "system_cxx_toolchain")
     3	load("@prelude//toolchains:genrule.bzl", "system_genrule_toolchain")
     4	load("@prelude//toolchains:python.bzl", "system_python_bootstrap_toolchain")
     5	load("@prelude//toolchains:remote_test_execution.bzl", "remote_test_execution_toolchain")
     6	load("@prelude//tests:test_toolchain.bzl", "noop_test_toolchain")
     7	
     8	# Rust 1.96.0 toolchain — pinned to match rust-toolchain.toml
     9	# rustc is resolved via rustup shim; PATH must include ~/.rustup/toolchains/1.96.0-aarch64-apple-darwin/bin
    10	system_rust_toolchain(
    11	    name = "rust",
    12	    default_edition = "2024",
    13	    visibility = ["PUBLIC"],
    14	)
    15	
    16	# Absolute compiler paths (not bare "clang") so the prelude's build-script cc-shim
    17	# resolves the compiler regardless of the action PATH — root cause of the Linux
    18	# `exec failed ['clang' ... FileNotFoundError`. /usr/bin/clang + /usr/bin/ar exist on
    19	# BOTH darwin-arm64 and linux-arm64 → cross-platform-correct. With this, the prelude
    20	# injects CC/CXX/AR into build scripts (ring/aws-lc-sys/openssl-sys), so their fixups
    21	# must NOT hardcode per-OS CC. (Hermetic step later: absolute host paths -> cell artifacts.)
    22	system_cxx_toolchain(
    23	    name = "cxx",
    24	    compiler = "/usr/bin/clang",
    25	    compiler_type = "clang",
    26	    linker = "/usr/bin/clang",
    27	    archiver = "/usr/bin/ar",
    28	    visibility = ["PUBLIC"],
    29	)
    30	
    31	# REQUIRED by the prelude: prelude//decls/toolchains_common.bzl `_cxx_toolchain()`
    32	# defaults to select({DEFAULT: toolchains//:cxx, config//:none: toolchains//:cxx_no_default_deps}).
    33	# The config//:none branch is NEVER selected, but buck2 within-view/query validation
    34	# requires the target to EXIST — without it, `//...` graph traversal (deps/rdeps, hence
    35	# the affected-gate query) fails with "Unknown target cxx_no_default_deps". Mirror :cxx.
    36	system_cxx_toolchain(
    37	    name = "cxx_no_default_deps",
    38	    compiler = "/usr/bin/clang",
    39	    compiler_type = "clang",
    40	    linker = "/usr/bin/clang",
    41	    archiver = "/usr/bin/ar",
    42	    visibility = ["PUBLIC"],
    43	)
    44	
    45	system_genrule_toolchain(
    46	    name = "genrule",
    47	    visibility = ["PUBLIC"],
    48	)
    49	
    50	# Required by prelude//rust/tools:internal_tools_toolchain
    51	system_python_bootstrap_toolchain(
    52	    name = "python_bootstrap",
    53	    visibility = ["PUBLIC"],
    54	)
    55	
    56	# Test runner toolchains — REQUIRED by `buck2 test`. The prelude resolves
    57	# toolchains//:test (ExternalRunnerTestToolchainInfo, via noop_test_toolchain — the
    58	# standard local runner the prelude demo wires) AND toolchains//:remote_test_execution
    59	# (RemoteTestExecutionToolchainInfo). Without :test, any rust_test target fails to
    60	# CONFIGURE ("Unknown target `test`"), so the gate's test phase can never run. Local
    61	# execution (no RE configured) runs tests on the gate pod. Prerequisite for #84 (emit
    62	# rust_test → gate builds AND TESTS). The RE target is wired now for when NativeLink RE lands.
    63	noop_test_toolchain(
    64	    name = "test",
    65	    visibility = ["PUBLIC"],
    66	)
    67	
    68	remote_test_execution_toolchain(
    69	    name = "remote_test_execution",
    70	    visibility = ["PUBLIC"],
    71	)
exec
/bin/zsh -lc "git show origin/dev:.buckconfig | nl -ba | sed -n '1,180p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
     1	[cells]
     2	  root = .
     3	  prelude = prelude
     4	  toolchains = toolchains
     5	  none = none
     6	  third-party = third-party
     7	
     8	[cell_aliases]
     9	  config = prelude
    10	  ovr_config = prelude
    11	  fbcode = none
    12	  fbsource = none
    13	  fbcode_macros = none
    14	  buck = none
    15	
    16	# Uses a copy of the prelude bundled with the buck2 binary. You can alternatively delete this
    17	# section and vendor a copy of the prelude to the `prelude` directory of your project.
    18	[external_cells]
    19	  prelude = bundled
    20	
    21	[parser]
    22	  target_platform_detector_spec = target:root//...->prelude//platforms:default \
    23	    target:prelude//...->prelude//platforms:default \
    24	    target:toolchains//...->prelude//platforms:default \
    25	    target:third-party//...->prelude//platforms:default
    26	
    27	[build]
    28	  execution_platforms = prelude//platforms:default
    29	
    30	# Bare-stock buck2 daemons default to a 4-day idle timeout, which lets stale
    31	# daemons (one per worktree/branch checkout) accumulate and sprawl host RAM.
    32	# These three keys are confirmed-recognized buck2 [buck2] section properties
    33	# (verified against facebook/buck2 upstream source: app/buck2_common/src/init.rs
    34	# for daemon_idle_timeout_s, app/buck2_execute_impl/.../clean_stale.rs for
    35	# clean_stale_enabled, docs/users/advanced/restarter.md for restarter).
    36	[buck2]
    37	  daemon_idle_timeout_s = 14400
    38	  clean_stale_enabled = true
    39	  restarter = true
exec
/bin/zsh -lc "git grep -n -E 'glob\\(["'^'"\\n]*(\\*\\*/\\*|\\*\\*/\\*\\.json)|glob\\(' origin/dev -- '*BUCK*' | head -120" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
origin/dev:audit/adapters/file/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/chain-domain/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/chain-domain/BUCK:16:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/emission-domain/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/query-domain/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/retention-cascade-domain/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/sealing-domain/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/usecase/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/core/verification-domain/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/emission-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/emission-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/query-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/query-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/retention-cascade-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/retention-cascade-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/sealing-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/sealing-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/verification-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:audit/ports/verification-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/adapters/accounting-http/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/adapters/accounting-http/BUCK:20:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/adapters/accounting-storage-inmemory/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/accounting-app/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/accounting-journal/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/billing-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/billing-kernel/BUCK:11:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/billing/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/billing/BUCK:17:    srcs = glob(["tests/cloud_billing_foundation.rs"]),
origin/dev:billing/core/billing/BUCK:28:    srcs = glob(["tests/invoice_lifecycle_transitions.rs"]),
origin/dev:billing/core/billing/BUCK:41:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/finops-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/finops-kernel/BUCK:11:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/finops/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/finops/BUCK:18:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/metering-pipeline-kernel/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/metering-pipeline-kernel/BUCK:12:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/metering-pipeline-kernel/BUCK:21:    srcs = glob(["tests/**/*.rs"]),
origin/dev:billing/core/metering/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/core/metering/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/billing-service/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/billing-service/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/billing-service/BUCK:25:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/cost-service/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/cost-service/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/meter-service/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/meter-service/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/meter-service/BUCK:25:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/saas-bench/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/facade/saas-bench/BUCK:18:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/ports/accounting-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/ports/finops-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/ports/finops-api/BUCK:17:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:billing/ports/tax-api/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/capacity-commercial/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/capacity-commercial/BUCK:18:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/capacity/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/capacity/BUCK:11:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/region/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/region/BUCK:16:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/regional-pack/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/regional-pack/BUCK:17:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/routing/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/core/routing/BUCK:15:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/ports/cell-bind/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/ports/cell-bind/BUCK:11:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/ports/region/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:cell/ports/regional-pack/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/adapters/path-resolver/BUCK:4:    srcs = glob(["src/**/*.rs"]),
origin/dev:ci/adapters/path-resolver/BUCK:16:    srcs = glob(["src/**/*.rs"]),
origin/dev:ci/facade/action-item-accounting/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/action-item-accounting/BUCK:14:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/action-item-accounting/BUCK:29:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/affected-target-set/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/affected-target-set/BUCK:14:    srcs = glob(["src/**/*.rs"]),
origin/dev:ci/facade/affected-target-set/BUCK:29:    srcs = glob(["src/**/*.rs"]),
origin/dev:ci/facade/affected-target-set/BUCK:42:    srcs = glob(["src/**/*.rs"]),
origin/dev:ci/facade/affected-target-set/BUCK:52:    srcs = glob(["src/**/*.rs"]),
origin/dev:ci/facade/affected-target-set/BUCK:63:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/affected-target-set/BUCK:76:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/artifact-accountability/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/artifact-accountability/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/artifact-accountability/BUCK:30:    srcs = glob(["tests/**/*.rs"]),
origin/dev:ci/facade/artifact-inventory-registry/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/artifact-inventory-registry/BUCK:31:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/artifact-inventory-registry/BUCK:57:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/artifact-inventory-registry/BUCK:87:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/automation-coverage/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/automation-coverage/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/automation-coverage/BUCK:31:    srcs = glob(["tests/**/*.rs"]),
origin/dev:ci/facade/automation-language-policy/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/automation-language-policy/BUCK:16:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/automation-language-policy/BUCK:36:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/baseline-ratchet/BUCK:11:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/baseline-ratchet/BUCK:22:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/baseline-ratchet/BUCK:68:    srcs = glob(["tests/**/*.rs"]),
origin/dev:ci/facade/baseline-ratchet/BUCK:97:    srcs = glob(["tests/**/*.rs"]),
origin/dev:ci/facade/baseline-ratchet/BUCK:113:    srcs = glob(["tests/**/*.rs"]),
origin/dev:ci/facade/build-cache-policy/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/build-cache-policy/BUCK:20:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/build-cache-policy/BUCK:31:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/build-cache-policy/BUCK:49:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/build-target-parity/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/build-target-parity/BUCK:14:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
origin/dev:ci/facade/build-target-parity/BUCK:28:    srcs = glob(["tests/**/*.rs"]),
origin/dev:ci/facade/caller-supplied-authorization/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/caller-supplied-authorization/BUCK:14:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/caller-supplied-authorization/BUCK:25:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/caller-supplied-authorization/BUCK:43:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/canonical-json/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/canonical-json/BUCK:17:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/canonical-json/BUCK:28:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/canonical-json/BUCK:43:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/contract-slice-conformance/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/contract-slice-conformance/BUCK:14:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/contract-slice-conformance/BUCK:26:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/core-dependency-isolation/BUCK:3:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/core-dependency-isolation/BUCK:17:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/core-dependency-isolation/BUCK:31:    srcs = glob(["src/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/core-dependency-isolation/BUCK:49:    srcs = glob(["tests/**/*.rs", "**/*.json"]),
origin/dev:ci/facade/crate-layer-suffix/BUCK:3:    srcs = glob(["src/**/*.rs", "migrations/**/*.sql", "**/*.cedar", "**/*.sql", "**/*.json", "**/*.toml", "**/*.yaml", "**/*.yml", "**/*.proto", "**/*.html", "**/*.css", "**/*.txt"]),
exec
/bin/zsh -lc "git show origin/dev:.github/workflows/oya-ci-required.yml | nl -ba | sed -n '480,520p;500,515p;620,655p;660,710p;800,840p'" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
   480	        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   481	      # The hermetic gate: buck2 BUILDS every cloud-ci target (proves the env!CARGO eradication —
   482	      # these targets could not compile under buck2 before) and TESTS them (the gate rust_tests
   483	      # run green, fully hermetic, with verdicts identical to the targeted gate matrix). This is the
   484	      # refactor's scope and is the binding hermetic check for this stage.
   485	      #
   486	      # The repo-wide affected-set verdict is owned by the binding gate-affected-target-set job below.
   487	      # Do not run a duplicate best-effort affected-set probe here: a non-blocking BUILD FAILED
   488	      # line inside a green job is indistinguishable from a false-green to humans and agents.
   489	      - name: buck2 build + test (//ci/..., hermetic — binding)
   490	        run: |
   491	          set -euo pipefail
   492	          # buck2 test builds its targets before running them, so a standalone
   493	          # `buck2 build` immediately before is redundant — removed (item 4 quick win).
   494	          # --unstable-write-invocation-record is additive observability only: it
   495	          # writes buck2's structured run record (cache_hit_rate, run_* counters)
   496	          # for the telemetry step below and changes nothing about the build.
   497	          buck2 test //ci/... --unstable-write-invocation-record /tmp/buck2-lane-invocation-record.json
   498	      # Per-lane cache-hit telemetry + warm-mode guard (ADR-0560; the audit's missing-SLO item):
   499	      # structured counters from buck2's invocation record — never log-grep — labeled with this
   500	      # lane's ADR-0556 build class. The report is now binding for record-shape / warm-mode
   500	      # lane's ADR-0556 build class. The report is now binding for record-shape / warm-mode
   501	      # sanity: once owned cloud-ci flips this lane from `bypass` to warm-ro/rw, a 0%-hit run or
   501	      # sanity: once owned cloud-ci flips this lane from `bypass` to warm-ro/rw, a 0%-hit run or
   502	      # missing cache counters is an INFRA-RED misconfiguration, not advisory noise. Today GitHub
   502	      # missing cache counters is an INFRA-RED misconfiguration, not advisory noise. Today GitHub
   503	      # Actions remains the transitional adapter and this lane stays bypass while NativeLink is dark.
   503	      # Actions remains the transitional adapter and this lane stays bypass while NativeLink is dark.
   504	      - name: Cache-hit telemetry + warm-mode guard (ADR-0560)
   504	      - name: Cache-hit telemetry + warm-mode guard (ADR-0560)
   505	        if: always()
   505	        if: always()
   506	        run: |
   506	        run: |
   507	          set -euo pipefail
   507	          set -euo pipefail
   508	          CACHE_MODE=bypass
   508	          CACHE_MODE=bypass
   509	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- report --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}" --out /tmp/cache-hit-report.json
   509	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- report --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}" --out /tmp/cache-hit-report.json
   510	          cat /tmp/cache-hit-report.json
   510	          cat /tmp/cache-hit-report.json
   511	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- assert-warm --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}"
   511	          buck2 run //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin -- assert-warm --record /tmp/buck2-lane-invocation-record.json --build-class gate-fleet-shared-graph --mode "${CACHE_MODE}"
   512	      - name: Upload cache-hit telemetry artifact
   512	      - name: Upload cache-hit telemetry artifact
   513	        if: always()
   513	        if: always()
   514	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   514	        # actions/upload-artifact@v7.0.1 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   515	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   515	        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
   516	        with:
   517	          name: cache-hit-report-buck2-lane
   518	          path: /tmp/cache-hit-report.json
   519	          if-no-files-found: error
   520	      - name: Upload runner disk reclaim operator artifact (buck2 lane)
   620	        run: |
   621	          # Build as the runner user (buck2 on user PATH; never run buck2 daemon as root —
   622	          # that corrupts cache/daemon ownership). Then sudo ONLY the prebuilt binary (needs
   623	          # root solely to remove the root-owned vendor preinstall dirs).
   624	          BIN="$(buck2 build //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin --show-output 2>/dev/null | awk '{print $2}')"
   625	          sudo -E "$BIN" \
   626	            --profile github-hosted-ubuntu-latest \
   627	            --infra-red-policy fail-closed \
   628	            --artifact-out "${RUNNER_TEMP}/runner-disk-reclaim-affected-set.json"
   629	      - name: Restore buck-out (read-only; dev-push is the sole writer)
   630	        # actions/cache/restore@v5.0.5 — Node 24 runtime; pinned to immutable release commit for hermetic CI.
   631	        uses: actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae
   632	        with:
   633	          path: buck-out
   634	          key: buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}
   635	          restore-keys: |
   636	            buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-
   637	      # KEEPS materializing (not converted to ADR-0556 D5 QW-1 artifact reuse): same rationale
   638	      # as the buck2 lane — the cone's gate tests consume the per-job merge-base frozen
   639	      # baseline (ADR-0551), and this lane's own build-health baseline below is per-job by
   640	      # design (ADR-0554 round-3).
   641	      - name: Materialize cloud-ci generated faces (out-of-graph boundary)
   642	        run: buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
   643	      - name: Fetch base ref for the merge-base anchor
   644	        if: ${{ github.event_name == 'pull_request' }}
   645	        env:
   646	          BASE_REF: ${{ github.base_ref || 'dev' }}
   647	        run: git fetch --no-tags --prune origin "+refs/heads/${BASE_REF}:refs/remotes/origin/${BASE_REF}"
   648	      # ── BUILD-HEALTH BASELINE (ADR-0554 D9 same-root build, round-5; ADR-0551 merge-base frozen
   649	      #    pattern). On a pull_request, derive the affected-set plan FIRST. Most PRs stay in the
   650	      #    affected cone and do not need a merge-base full-workspace build at all; only a derived
   651	      #    FULL decision needs the MERGE-BASE build-health baseline used by the ratchet. When FULL
   652	      #    is required, materialize that baseline IN THE MAIN ROOT so it shares the warm ./buck-out
   653	      #    restored above (the merge-base IS a dev commit, so the dev-keyed buck-out is near-fully
   654	      #    warm for it). We detach the SAME working tree to the merge-base COMMITTED tree-ish (the
   655	      #    candidate working tree is removed from disk for the build), run the full keep-going
   660	      #
   661	      #    ANTI-LAUNDERING (ADR-0554 D6, preserved): the baseline failure-set comes ENTIRELY from
   662	      #    the merge-base COMMITTED tree (git object history — candidate-uncontrollable); during the
   663	      #    baseline build the candidate working tree is GONE from disk, so it cannot feed the
   664	      #    baseline; the report reaches the verdict ONLY via --baseline-report. The warm ./buck-out
   665	      #    is a content-addressed substrate — a buck2 hit is bit-identical to a cold build (ADR-0556
   666	      #    D1/D2) — so warmth changes only wall-clock, never the baseline SOURCE. Warm-eligible
   667	      #    under ADR-0556 with no policy change (trusted-author, content-addressed; not the
   668	      #    integrity-canary/release cold floor). GH #899 activates the trusted D8 consumer first:
   669	      #    use an exact push-to-dev baseline artifact when provenance and schema validate, else
   670	      #    fail closed to the same in-job merge-base rebuild below.
   671	      - name: Materialize merge-base build-health baseline when affected-set needs FULL
   672	        if: ${{ github.event_name == 'pull_request' }}
   673	        env:
   674	          BASE_REF: ${{ github.base_ref || 'dev' }}
   675	          GH_TOKEN: ${{ github.token }}
   676	        run: |
   677	          set -euo pipefail
   678	          merge_base="$(git merge-base "origin/${BASE_REF}" HEAD)"
   679	          orig_ref="$(git rev-parse HEAD)"
   680	          candidate_toolchain="$(awk -F'"' '/channel/ {print $2; exit}' rust-toolchain.toml)"
   681	          decision_log="${RUNNER_TEMP}/affected-set-derive.log"
   682	          full_required_marker="${RUNNER_TEMP}/affected-set-full-required"
   683	          echo "false" > "${full_required_marker}"
   684	          gate_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-affected-set-bin --show-output | awk '{print $2}')"
   685	          telemetry_bin="$(buck2 build //ci/facade/affected-target-set:oya-cloud-ci-step-telemetry-bin --show-output | awk '{print $2}')"
   686	          ci_step_telemetry="${RUNNER_TEMP}/oya-cloud-ci-step-telemetry"
   687	          cp "${telemetry_bin}" "${ci_step_telemetry}"
   688	          chmod +x "${ci_step_telemetry}"
   689	          echo "affected-set preflight: derive plan before merge-base baseline"
   690	          "${ci_step_telemetry}" --phase derive-affected-set-tier -- "${gate_bin}" \
   691	            --policy ci/facade/affected-target-set/affected-set-policy.json \
   692	            --base "origin/${BASE_REF}" --mode auto --derive-only \
   693	            --decision-artifact-out "${RUNNER_TEMP}/affected-set-derive-decision.json" \
   694	            | tee "${decision_log}"
   695	          if grep -Eq '^affected-set: (decision=FULL|ESCALATE to FULL)' "${decision_log}"; then
   696	            echo "true" > "${full_required_marker}"
   697	            echo "affected-set preflight: FULL decision requires merge-base build-health baseline"
   698	          else
   699	            echo "affected-set preflight: derived non-FULL decision; skipping merge-base baseline"
   700	            exit 0
   701	          fi
   702	          echo "build-health baseline: merge-base=${merge_base} candidate=${orig_ref}"
   703	          artifact_name="build-health-baseline-${merge_base}"
   704	          echo "build-health baseline: attempting trusted dev-push artifact ${artifact_name}"
   705	          try_trusted_baseline_artifact() {
   706	            if ! command -v gh >/dev/null 2>&1; then
   707	              echo "build-health baseline: gh unavailable; falling back to in-job merge-base rebuild"
   708	              return 1
   709	            fi
   710	            local runs_json="${RUNNER_TEMP}/build-health-trusted-runs.json"
   800	          fi
   801	          # ALWAYS restore the candidate tree on EXIT — a failed baseline build can never strand CI
   802	          # on the merge-base tree (the subsequent Binding affected-set step runs on the candidate).
   803	          # NOTE: if the timeout-minutes:45 rail SIGKILLs this build, the bash EXIT trap does NOT
   804	          # fire (tree left detached at merge-base) — but a timeout fails the whole job RED → fan-in
   805	          # RED, so it is fail-closed and never produces a wrong-baseline verdict.
   806	          restore_candidate_tree() {
   807	            local exit_status="$?"
   808	            git checkout --quiet --detach "${orig_ref}" 2>/dev/null || git checkout --quiet "${orig_ref}"
   809	            if [ "${candidate_toolchain}" != "${baseline_toolchain:-${candidate_toolchain}}" ]; then
   810	              echo "build-health baseline: cleaning buck-out after restoring candidate toolchain ${candidate_toolchain}"
   811	              buck2 clean
   812	            fi
   813	            exit "${exit_status}"
   814	          }
   815	          trap restore_candidate_tree EXIT
   816	          # Detach the MAIN working tree to the merge-base COMMITTED tree-ish: the baseline is
   817	          # computed from git object history (candidate-uncontrollable), and the candidate working
   818	          # tree is removed from disk for the build, so a PR cannot grow its own baseline to
   819	          # launder a regression.
   820	          git checkout --quiet --detach "${merge_base}"
   821	          baseline_toolchain="$(awk -F'"' '/channel/ {print $2; exit}' rust-toolchain.toml)"
   822	          rustup toolchain install
   823	          if [ "${candidate_toolchain}" != "${baseline_toolchain}" ]; then
   824	            echo "build-health baseline: Rust toolchain changed ${baseline_toolchain} -> ${candidate_toolchain}; isolating buck-out"
   825	            buck2 clean
   826	          fi
   827	          # Build the whole merge-base workspace keep-going. Same-channel PRs share warm ./buck-out;
   828	          # Rust-channel bump PRs intentionally go cold on both sides to avoid mixed-rustc rlibs.
   829	          # The build is EXPECTED to be non-zero (dev carries pre-existing breakage) — that is the
   830	          # baseline, not a failure, so we never propagate its exit code.
   831	          "${ci_step_telemetry}" --phase materialize-merge-base-build-health-baseline -- \
   832	            buck2 build //... --keep-going \
   833	              --build-report "${RUNNER_TEMP}/build-health-baseline.json" || true
   834	          test -s "${RUNNER_TEMP}/build-health-baseline.json" \
   835	            || { echo "build-health: FATAL empty merge-base baseline report"; exit 1; }
   836	      - name: Binding affected-set build + test (cone-binding; FULL tier = build-health ratchet)
   837	        env:
   838	          EVENT_NAME: ${{ github.event_name }}
   839	          BASE_REF: ${{ github.base_ref || 'dev' }}
   840	        run: |
exec
/bin/zsh -lc "git grep -n 'artifact-inventory-registry/.*generated.json\\|scm-volatile-facts.generated.json\\|masterplan.generated.json' origin/dev -- '*BUCK*' | head -160" in /Users/jasonlee/Developer/oyatie
 succeeded in 0ms:
codex
The evidence now supports a hard split: three current rigor defects must be repaired before any scoping cutover, while the largest safe savings come from hermetic shared CAS, removing duplicate gate executions, and optimizing—not bypassing—the live/database and materialization work. I’m closing with conservative savings based on the supplied timings; local git contains no run-duration logs.
Read-only UltraQA audit complete against local productized origin/dev; findings reconciled across workflow topology, affected-set soundness, cache/setup rigor, materialization, Postgres lanes, and docs drift.
  ✓ Resolve governing repo authority and establish the correct local CI revision
  ✓ Audit workflow topology, duplicated setup, materialization, and job dependencies
  ✓ Hostile-review affected-set fail-closed correctness and baseline isolation
  ✓ Verify cache-key/CAS rigor, setup hermeticity, Postgres, and runner-sizing options
  ✓ Produce ranked exact-verdict optimization plan with file-line evidence and rejected shortcuts
exec
/bin/zsh -lc "git grep -n -E 'postgres(:16)?@sha256|postgres@sha256|image_digest.*postgres|postgres.*sha256' origin/dev -- ':"'!Cargo.lock'"' ':"'!third-party/**'"' | head -80" in /Users/jasonlee/Developer/oyatie
 succeeded in 756ms:
codex
Verdict: the productized CI is under-optimized—51 runner executions duplicate substantial work, while three rigor defects make affected scoping and warm caching unsafe to credit today.
### Audit-base correction
The dirty cwd is at `c52bdb09e`; local `origin/dev@d7568c731` is 424 commits ahead and is the only local revision matching the supplied Postgres jobs/timings. References below therefore use the local `origin/dev:` blob.
| Fact | Verified productized state |
|---|---|
| Jobs | 11 YAML jobs, expanding to **51 runner executions** because the gate matrix has 41 entries (`oya-ci-required.yml:125-165`) |
| Buck2 setup | 10 call sites, expanding to **50 installs** |
| Rust setup | Roughly **50 normal `rustup toolchain install` executions**, plus a conditional merge-base install |
| Materialization | **5 owned-Rust executions**; matrix reuse is already implemented |
| `buck-out` | 2 restores, 1 trusted-dev save |
| Binding scope | `buck2 test //ci/...` (`:489-497`) plus a binding workspace affected-set job (`:543-870`) |
| Stale `.claire` graph | Historical; removed on 2026-06-09 and absent from both HEAD and `origin/dev` |
| Old scripts | `buck2-affected-gate.sh` survives but is unused; the shell materializer is already gone; `install-buck2.sh` remains heavily used |
The supplied timings are treated as measured inputs; local git has no workflow-duration logs.
## Ranked optimization plan
### 1. Repair affected-set soundness before using it to remove full work
**Change**
The current Rust successor is already binding; do not “flip” the historical shell driver. Fix these defects first:
- `ci/facade/affected-target-set/src/lib.rs:365-378` treats deleted non-Rust files as irrelevant. First safe version: every deletion, rename, copy, type change, and submodule change selects FULL. Later, query both isolated base and head graphs and union their owners.
- Replace the narrow owner-required set at `affected-set-policy.json:21-25`. Every changed path must map to a Buck owner or an explicit synthetic dependency; otherwise FULL.
- `src/main.rs:823-836` returns green after a PR FULL build without running full tests. Add a full test-health ratchet: run base/head full test reports and block new failures. A full fallback that only builds is “checking less,” not an optimization.
- Move merge-base evaluation out of the candidate filesystem. Workflow `:641-642` creates ignored candidate JSON, then `git checkout` at `:816-820` leaves it in place; broad `**/*.json` BUCK globs can consume it. Use a clean detached merge-base worktree and merge-base-owned Rust materialization.
- Replace workflow shell/Python baseline handling at `:671-835` with the owned Rust selector/baseline component. Validate commit, toolchain, universe digest, report completeness, and provenance—not merely a non-empty `results` object.
- Retire unused `infra/ci/buck2-affected-gate.sh`; do not extend it.
The safe selector is:
`selected = always_run ∪ owners(base,head,diff) ∪ rdeps(universe, owners ∪ synthetic_dependencies)`
Any merge-base, diff, owner, graph, policy, or query uncertainty sets `selected = universe`. Buildfiles, Buck config, prelude, toolchains, dependencies, workflow/setup, selector, materializer, generator, and global-corpus changes also select FULL. Push, merge-group, and dispatch remain full.
Only after this is proven should the selector replace the unconditional `buck2 test //ci/...`. Whole-tree scanners remain `always_run` until their implicit inputs become declared graph or synthetic edges.
**Saving**
No saving may be credited until shadow scoped/full runs show zero target or verdict mismatches. Thereafter, approximately **50–80 runner-seconds** on eligible small PRs; FULL-trigger changes save zero. Current end-to-end wall impact is near zero while the 501/497-second jobs dominate.
**Rigor preserved because**
Every changed path is accounted for; the scoped set is mechanically a superset; every uncertainty executes the exact full build-and-test universe. Shadow comparison validates the proof but does not replace fail-closed derivation.
---
### 2. Replace archive warmth and repeated setup with a hermetic image plus Buck2-native CAS
**Change**
- First make toolchains actual immutable inputs. `toolchains/BUCK:8-28` currently resolves ambient rustup and `/usr/bin/clang`; those compiler bytes are outside the declared graph.
- Introduce a digest-pinned, attested declarative CI image containing the exact Buck2 binary, Rust toolchain/components, C/C++ toolchain, and PostgreSQL client. Its successor belongs under the debranded `ci/` capability.
- Retire `infra/ci/install-buck2.sh` and the repeated installer/rustup steps. A composite action alone merely deduplicates YAML and saves essentially no runtime.
- After hermetic closure, replace the multi-gigabyte archive blocks at workflow `:463-470`, `:629-636`, and `:883-889` with authenticated NativeLink/CAS compile-action reuse across warm-eligible jobs.
- Do **not** add `buck-out` archive restores to the other jobs. The current archive is documented as approximately 5.78 GiB compressed and 12–15 GiB restored (`:437-440`).
- Never cache live-test verdicts. Tests still execute; CAS supplies only content-addressed build artifacts.
**Saving**
- At 20–40 seconds per 41 matrix legs: **13.7–27.3 runner-minutes**.
- Every 10 seconds removed from 50 repeated setups saves another **8.3 runner-minutes**.
- Likely **1–3 minutes critical-path improvement** if compilation/setup materially contributes to the 497/501-second jobs; step telemetry must confirm.
**Rigor preserved because**
Action identity must include image/platform digest, Buck2 revision, prelude, Rust/C++ toolchains, graph configuration, and declared sources. Fork/untrusted execution remains cold; trusted post-merge is the writer. Misses recompute locally, digest mismatches quarantine and fail, and a binding cold-versus-warm byte-digest canary disables all warm reads on any mismatch.
The repository already encodes that contract, but `specs/cache-warm-license.json:6-7` is still false. Warm CAS must not activate before the first green canary.
**Current slow-and-weak configuration:** the pipeline pays for multi-gigabyte warmth while telemetry declares `CACHE_MODE=bypass` (`oya-ci-required.yml:504-511`), and ambient compiler identity is not rigorously closed. This is a false-green risk, not a proven observed false green.
---
### 3. Eliminate duplicate gate execution with an exact, surface-all gate fleet
**Change**
The 41 matrix legs (`:114-196`) run named CI test pairs, while `buck2 test //ci/...` (`:489-497`) runs the CI test universe again.
Replace both with an owned Rust gate-fleet scheduler under `ci/facade/gate-fleet`:
- Query and hash the exact existing `//ci/...` test-target universe.
- Compare it against the registered matrix target union; refuse cutover if they differ.
- Partition the exact union into approximately 8 weighted shards.
- Execute every target despite sibling failures and emit per-gate result packets/annotations.
- Make the fan-in red if any target is missing, duplicated, unexecuted, or failed.
- Dual-run old and new topology until target lists and verdicts are identical.
**Saving**
The supplied ten 95–99-second legs plus the 148-second catalog leg already represent a lower bound of roughly **18.5 runner-minutes** of duplicated cost. Going from 41 to eight shards also removes 33 setup/startup sequences.
Current wall saving may be nearly zero because `215s producer + 148s catalog = 363s`, below the 501-second Postgres critical path. It becomes important after Postgres/freshness optimization.
**Rigor preserved because**
The exact same test target union executes, with surface-all failure reporting and mechanically enforced coverage. No matrix leg is deleted merely because it appears redundant.
---
### 4. Optimize materialization internally; preserve independent attestations
**Change**
Producer reuse already exists: `producer-regen` creates the artifact at `:50-84`, and matrix legs download it at `:176-181`.
Further optimize the owned-Rust materializer by:
- Scanning git/history once and sharing the immutable scan across emitters.
- Running byte-independent emitters concurrently.
- Separating candidate-face generation from merge-base frozen-baseline generation.
- Letting mere-reader jobs consume a candidate-SHA-bound artifact while independently producing private frozen baselines.
- Retaining independent regeneration in registry-drift (`:238-263`) and baseline-ratchet (`:269-300`).
Reject “materialize once and feed every job.” A detector consuming the artifact it attests is self-referential and weaker.
**Saving**
Freshness + registry-drift + producer total 1,102 supplied seconds. A profiling-confirmed 25–40% materializer reduction would save approximately **275–441 runner-seconds**, potentially **124–199 seconds** from the freshness critical path.
**Rigor preserved because**
Every emitted byte is compared against the existing implementation during qualification; artifacts bind candidate SHA, generator/policy digests, path list, sizes, and content hashes. Missing or mismatched metadata is red. Regeneration/determinism detectors remain independent.
---
### 5. Optimize live Postgres execution without skipping or sharing coverage
**Change**
For both lanes (`:930-1065`, `:1067-1186`):
- Replace mutable `postgres:16` (`:936`, `:1073`) with a reviewed immutable digest. No digest is present locally, so this audit will not invent one.
- Remove repeated mutable `apt-get` client installation (`:971-976`, `:1108-1113`) via the pinned CI image.
- Replace inline bootstrap shell with owned Rust while preserving every role, grant, migration, schema, and provenance field.
- Submit the existing six and four targets in one Buck2 invocation per lane while retaining `--local-only`, `--num-threads 1`, and `RUST_TEST_THREADS=1`.
- A later concurrency experiment may allocate a fresh isolated database per target and run targets concurrently, but only after serial-versus-parallel verdict parity is proven.
Reject:
- Cross-job service-container reuse: it sacrifices isolation and turns two parallel lanes into a shared-state failure domain.
- Smaller fixtures: reduced coverage.
- Skipping the jobs for “unrelated” diffs: under the literal no-skipped-check constraint, live environment behavior can change independently of the source cone. Give this proposal zero credit.
**Saving**
CAS, prewarmed client/toolchain, and fewer Buck2/bootstrap round trips should target **120–360 combined runner-seconds**, with perhaps **60–200 seconds** off the 501-second critical lane if profiling confirms setup/compilation dominance.
**Rigor preserved because**
All ten test targets, fixtures, roles, migrations, environment variables, and fresh live-database executions remain. No live verdict is cached.
---
### 6. Benchmark larger runners only after CAS and topology fixes
**Change**
Benchmark identical digest-pinned x86_64 images on larger runners for compile-bound full Buck2, affected FULL, freshness, and materializer shards. Do not blanket-upsize the matrix or Postgres lanes.
**Saving**
Plausible **20–45% cold compile wall reduction**. Cost is favorable only when:
`new_duration / old_duration < old_price / new_price`
**Rigor preserved because**
Image, architecture, toolchain, target set, test concurrency, and output digests remain identical. Reject any runner change whose cold differential produces a different artifact or verdict manifest.
---
### 7. Apply setup/CAS improvements to docs drift, but preserve all checks
**Change**
For `docs-graph-drift.yml`:
- Replace the executable cache and shell installer at `:63-76` with the same digest-pinned image/CAS setup.
- Keep materialization, generator build, regeneration test, and golden test at `:72-87`.
- Move the generator to its debranded capability home when reorganization authority provides that target; do not introduce a compatibility alias with deprecated naming.
**Saving**
Approximately **20–60 seconds per triggered run**; zero required-context wall impact because this workflow is feedback-only and path-filtered.
**Rigor preserved because**
The exact materialization, build, regeneration, and golden-test sequence remains. Do not remove the explicit build until graph-level target equivalence proves it redundant.
## Hostile QA result
| Scenario | Verdict |
|---|---|
| Revision/timing consistency | PASS after selecting local `origin/dev` |
| Deleted/non-Rust affected inputs | **FAIL — selector hole** |
| Uncertain FULL fallback | **FAIL — full tests omitted** |
| Merge-base isolation | **FAIL — candidate ignored files can survive checkout** |
| Cold/warm bit identity | BLOCKED — warm license false and ambient toolchain incomplete |
| Matrix artifact reuse | PASS; already optimized |
| Blanket materialization reuse | REJECTED; weakens independent detection |
| Postgres skipping/fixture reduction | REJECTED; violates hard constraint |
**ULTRAQA COMPLETE, read-only:** no files or runtime state were changed. Optimization cutover should stop at the three P0 affected-set/baseline defects; the safe high-value implementation order is hermetic image/CAS → exact gate-fleet collapse → owned-Rust materialization/Postgres optimization → runner sizing.
258,352
Verdict: the productized CI is under-optimized—51 runner executions duplicate substantial work, while three rigor defects make affected scoping and warm caching unsafe to credit today.
### Audit-base correction
The dirty cwd is at `c52bdb09e`; local `origin/dev@d7568c731` is 424 commits ahead and is the only local revision matching the supplied Postgres jobs/timings. References below therefore use the local `origin/dev:` blob.
| Fact | Verified productized state |
|---|---|
| Jobs | 11 YAML jobs, expanding to **51 runner executions** because the gate matrix has 41 entries (`oya-ci-required.yml:125-165`) |
| Buck2 setup | 10 call sites, expanding to **50 installs** |
| Rust setup | Roughly **50 normal `rustup toolchain install` executions**, plus a conditional merge-base install |
| Materialization | **5 owned-Rust executions**; matrix reuse is already implemented |
| `buck-out` | 2 restores, 1 trusted-dev save |
| Binding scope | `buck2 test //ci/...` (`:489-497`) plus a binding workspace affected-set job (`:543-870`) |
| Stale `.claire` graph | Historical; removed on 2026-06-09 and absent from both HEAD and `origin/dev` |
| Old scripts | `buck2-affected-gate.sh` survives but is unused; the shell materializer is already gone; `install-buck2.sh` remains heavily used |
The supplied timings are treated as measured inputs; local git has no workflow-duration logs.
## Ranked optimization plan
### 1. Repair affected-set soundness before using it to remove full work
**Change**
The current Rust successor is already binding; do not “flip” the historical shell driver. Fix these defects first:
- `ci/facade/affected-target-set/src/lib.rs:365-378` treats deleted non-Rust files as irrelevant. First safe version: every deletion, rename, copy, type change, and submodule change selects FULL. Later, query both isolated base and head graphs and union their owners.
- Replace the narrow owner-required set at `affected-set-policy.json:21-25`. Every changed path must map to a Buck owner or an explicit synthetic dependency; otherwise FULL.
- `src/main.rs:823-836` returns green after a PR FULL build without running full tests. Add a full test-health ratchet: run base/head full test reports and block new failures. A full fallback that only builds is “checking less,” not an optimization.
- Move merge-base evaluation out of the candidate filesystem. Workflow `:641-642` creates ignored candidate JSON, then `git checkout` at `:816-820` leaves it in place; broad `**/*.json` BUCK globs can consume it. Use a clean detached merge-base worktree and merge-base-owned Rust materialization.
- Replace workflow shell/Python baseline handling at `:671-835` with the owned Rust selector/baseline component. Validate commit, toolchain, universe digest, report completeness, and provenance—not merely a non-empty `results` object.
- Retire unused `infra/ci/buck2-affected-gate.sh`; do not extend it.
The safe selector is:
`selected = always_run ∪ owners(base,head,diff) ∪ rdeps(universe, owners ∪ synthetic_dependencies)`
Any merge-base, diff, owner, graph, policy, or query uncertainty sets `selected = universe`. Buildfiles, Buck config, prelude, toolchains, dependencies, workflow/setup, selector, materializer, generator, and global-corpus changes also select FULL. Push, merge-group, and dispatch remain full.
Only after this is proven should the selector replace the unconditional `buck2 test //ci/...`. Whole-tree scanners remain `always_run` until their implicit inputs become declared graph or synthetic edges.
**Saving**
No saving may be credited until shadow scoped/full runs show zero target or verdict mismatches. Thereafter, approximately **50–80 runner-seconds** on eligible small PRs; FULL-trigger changes save zero. Current end-to-end wall impact is near zero while the 501/497-second jobs dominate.
**Rigor preserved because**
Every changed path is accounted for; the scoped set is mechanically a superset; every uncertainty executes the exact full build-and-test universe. Shadow comparison validates the proof but does not replace fail-closed derivation.
---
### 2. Replace archive warmth and repeated setup with a hermetic image plus Buck2-native CAS
**Change**
- First make toolchains actual immutable inputs. `toolchains/BUCK:8-28` currently resolves ambient rustup and `/usr/bin/clang`; those compiler bytes are outside the declared graph.
- Introduce a digest-pinned, attested declarative CI image containing the exact Buck2 binary, Rust toolchain/components, C/C++ toolchain, and PostgreSQL client. Its successor belongs under the debranded `ci/` capability.
- Retire `infra/ci/install-buck2.sh` and the repeated installer/rustup steps. A composite action alone merely deduplicates YAML and saves essentially no runtime.
- After hermetic closure, replace the multi-gigabyte archive blocks at workflow `:463-470`, `:629-636`, and `:883-889` with authenticated NativeLink/CAS compile-action reuse across warm-eligible jobs.
- Do **not** add `buck-out` archive restores to the other jobs. The current archive is documented as approximately 5.78 GiB compressed and 12–15 GiB restored (`:437-440`).
- Never cache live-test verdicts. Tests still execute; CAS supplies only content-addressed build artifacts.
**Saving**
- At 20–40 seconds per 41 matrix legs: **13.7–27.3 runner-minutes**.
- Every 10 seconds removed from 50 repeated setups saves another **8.3 runner-minutes**.
- Likely **1–3 minutes critical-path improvement** if compilation/setup materially contributes to the 497/501-second jobs; step telemetry must confirm.
**Rigor preserved because**
Action identity must include image/platform digest, Buck2 revision, prelude, Rust/C++ toolchains, graph configuration, and declared sources. Fork/untrusted execution remains cold; trusted post-merge is the writer. Misses recompute locally, digest mismatches quarantine and fail, and a binding cold-versus-warm byte-digest canary disables all warm reads on any mismatch.
The repository already encodes that contract, but `specs/cache-warm-license.json:6-7` is still false. Warm CAS must not activate before the first green canary.
**Current slow-and-weak configuration:** the pipeline pays for multi-gigabyte warmth while telemetry declares `CACHE_MODE=bypass` (`oya-ci-required.yml:504-511`), and ambient compiler identity is not rigorously closed. This is a false-green risk, not a proven observed false green.
---
### 3. Eliminate duplicate gate execution with an exact, surface-all gate fleet
**Change**
The 41 matrix legs (`:114-196`) run named CI test pairs, while `buck2 test //ci/...` (`:489-497`) runs the CI test universe again.
Replace both with an owned Rust gate-fleet scheduler under `ci/facade/gate-fleet`:
- Query and hash the exact existing `//ci/...` test-target universe.
- Compare it against the registered matrix target union; refuse cutover if they differ.
- Partition the exact union into approximately 8 weighted shards.
- Execute every target despite sibling failures and emit per-gate result packets/annotations.
- Make the fan-in red if any target is missing, duplicated, unexecuted, or failed.
- Dual-run old and new topology until target lists and verdicts are identical.
**Saving**
The supplied ten 95–99-second legs plus the 148-second catalog leg already represent a lower bound of roughly **18.5 runner-minutes** of duplicated cost. Going from 41 to eight shards also removes 33 setup/startup sequences.
Current wall saving may be nearly zero because `215s producer + 148s catalog = 363s`, below the 501-second Postgres critical path. It becomes important after Postgres/freshness optimization.
**Rigor preserved because**
The exact same test target union executes, with surface-all failure reporting and mechanically enforced coverage. No matrix leg is deleted merely because it appears redundant.
---
### 4. Optimize materialization internally; preserve independent attestations
**Change**
Producer reuse already exists: `producer-regen` creates the artifact at `:50-84`, and matrix legs download it at `:176-181`.
Further optimize the owned-Rust materializer by:
- Scanning git/history once and sharing the immutable scan across emitters.
- Running byte-independent emitters concurrently.
- Separating candidate-face generation from merge-base frozen-baseline generation.
- Letting mere-reader jobs consume a candidate-SHA-bound artifact while independently producing private frozen baselines.
- Retaining independent regeneration in registry-drift (`:238-263`) and baseline-ratchet (`:269-300`).
Reject “materialize once and feed every job.” A detector consuming the artifact it attests is self-referential and weaker.
**Saving**
Freshness + registry-drift + producer total 1,102 supplied seconds. A profiling-confirmed 25–40% materializer reduction would save approximately **275–441 runner-seconds**, potentially **124–199 seconds** from the freshness critical path.
**Rigor preserved because**
Every emitted byte is compared against the existing implementation during qualification; artifacts bind candidate SHA, generator/policy digests, path list, sizes, and content hashes. Missing or mismatched metadata is red. Regeneration/determinism detectors remain independent.
---
### 5. Optimize live Postgres execution without skipping or sharing coverage
**Change**
For both lanes (`:930-1065`, `:1067-1186`):
- Replace mutable `postgres:16` (`:936`, `:1073`) with a reviewed immutable digest. No digest is present locally, so this audit will not invent one.
- Remove repeated mutable `apt-get` client installation (`:971-976`, `:1108-1113`) via the pinned CI image.
- Replace inline bootstrap shell with owned Rust while preserving every role, grant, migration, schema, and provenance field.
- Submit the existing six and four targets in one Buck2 invocation per lane while retaining `--local-only`, `--num-threads 1`, and `RUST_TEST_THREADS=1`.
- A later concurrency experiment may allocate a fresh isolated database per target and run targets concurrently, but only after serial-versus-parallel verdict parity is proven.
Reject:
- Cross-job service-container reuse: it sacrifices isolation and turns two parallel lanes into a shared-state failure domain.
- Smaller fixtures: reduced coverage.
- Skipping the jobs for “unrelated” diffs: under the literal no-skipped-check constraint, live environment behavior can change independently of the source cone. Give this proposal zero credit.
**Saving**
CAS, prewarmed client/toolchain, and fewer Buck2/bootstrap round trips should target **120–360 combined runner-seconds**, with perhaps **60–200 seconds** off the 501-second critical lane if profiling confirms setup/compilation dominance.
**Rigor preserved because**
All ten test targets, fixtures, roles, migrations, environment variables, and fresh live-database executions remain. No live verdict is cached.
---
### 6. Benchmark larger runners only after CAS and topology fixes
**Change**
Benchmark identical digest-pinned x86_64 images on larger runners for compile-bound full Buck2, affected FULL, freshness, and materializer shards. Do not blanket-upsize the matrix or Postgres lanes.
**Saving**
Plausible **20–45% cold compile wall reduction**. Cost is favorable only when:
`new_duration / old_duration < old_price / new_price`
**Rigor preserved because**
Image, architecture, toolchain, target set, test concurrency, and output digests remain identical. Reject any runner change whose cold differential produces a different artifact or verdict manifest.
---
### 7. Apply setup/CAS improvements to docs drift, but preserve all checks
**Change**
For `docs-graph-drift.yml`:
- Replace the executable cache and shell installer at `:63-76` with the same digest-pinned image/CAS setup.
- Keep materialization, generator build, regeneration test, and golden test at `:72-87`.
- Move the generator to its debranded capability home when reorganization authority provides that target; do not introduce a compatibility alias with deprecated naming.
**Saving**
Approximately **20–60 seconds per triggered run**; zero required-context wall impact because this workflow is feedback-only and path-filtered.
**Rigor preserved because**
The exact materialization, build, regeneration, and golden-test sequence remains. Do not remove the explicit build until graph-level target equivalence proves it redundant.
## Hostile QA result
| Scenario | Verdict |
|---|---|
| Revision/timing consistency | PASS after selecting local `origin/dev` |
| Deleted/non-Rust affected inputs | **FAIL — selector hole** |
| Uncertain FULL fallback | **FAIL — full tests omitted** |
| Merge-base isolation | **FAIL — candidate ignored files can survive checkout** |
| Cold/warm bit identity | BLOCKED — warm license false and ambient toolchain incomplete |
| Matrix artifact reuse | PASS; already optimized |
| Blanket materialization reuse | REJECTED; weakens independent detection |
| Postgres skipping/fixture reduction | REJECTED; violates hard constraint |
**ULTRAQA COMPLETE, read-only:** no files or runtime state were changed. Optimization cutover should stop at the three P0 affected-set/baseline defects; the safe high-value implementation order is hermetic image/CAS → exact gate-fleet collapse → owned-Rust materialization/Postgres optimization → runner sizing.
