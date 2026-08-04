# G033 CODEOWNERS retirement surface — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Goal: `G033-resolve-broken-codeowners-lane`
Authority: live GitHub codeowners/errors API + origin/dev tree + ADR-0634 D4 recommendation to delete.

## Live failure evidence
- API: `GET /repos/jason931225/oyatie/codeowners/errors` → **111 errors**.
- Sample class: every `@teams/*` owner is Unknown owner (team does not exist / not public / no write access).
- Distinct logical team owners in `.github/CODEOWNERS`: **39**.
- File header states handles are "logical owner IDs until the GitHub org/team namespace is provisioned" — i.e. the lane is already declared non-provisioned, yet GitHub still surfaces unknown-owner errors.

## Owners enumerated
- `@teams/axis-ads-analytics`
- `@teams/axis-cloud`
- `@teams/axis-foundry`
- `@teams/axis-saas`
- `@teams/axis-search`
- `@teams/axis-workspace`
- `@teams/council-architecture`
- `@teams/council-privacy`
- `@teams/crew-adr-promotion`
- `@teams/gtm-customer-success`
- `@teams/gtm-marketing`
- `@teams/gtm-partnerships`
- `@teams/gtm-sales-se`
- `@teams/ops-compliance`
- `@teams/ops-dr-capacity`
- `@teams/ops-finops`
- `@teams/ops-security`
- `@teams/ops-sre-reliability`
- `@teams/platform-api-sdk`
- `@teams/platform-audit-evidence`
- `@teams/platform-eventing-og`
- `@teams/platform-privacy-dub`
- `@teams/platform-tenancy-identity`
- `@teams/regional-packs`
- `@teams/tactical-first-vertical-pilot`
- `@teams/vertical-agriculture`
- `@teams/vertical-construction`
- `@teams/vertical-corporate`
- `@teams/vertical-education`
- `@teams/vertical-fintech`
- `@teams/vertical-food`
- `@teams/vertical-healthcare`
- `@teams/vertical-hospitality`
- `@teams/vertical-industrial`
- `@teams/vertical-legal`
- `@teams/vertical-logistics`
- `@teams/vertical-public-sector`
- `@teams/vertical-real-estate`
- `@teams/vertical-retail`

## Atomic retirement surface (candidate)
If founder chooses deletion (ADR-0634 D4), the atomic PR must not replace unknown-owner noise with silent zero-routing. Required surfaces:

### Delete / retire
- `.github/CODEOWNERS` (primary broken surface)
- any root/docs CODEOWNERS mirrors if present (currently absent on origin/dev)
- registered gate/check that asserts CODEOWNERS mirror/routing if it only certifies the broken file

### Executable consumers to reconcile
- `.github/ISSUE_TEMPLATE/blocker-resolution-card.yml` (1 hits) e.g. `description: Team handle from .github/CODEOWNERS or worker profile.`
- `ci/facade/affected-target-set/affected-set-policy.json` (1 hits) e.g. `"root//governance/check/codeowners-mirror:check-codeowners-mirror-unittest",`
- `ci/facade/affected-target-set/tests/affected_set.rs` (1 hits) e.g. `"root//governance/check/codeowners-mirror:check-codeowners-mirror-unittest",`
- `docs/architecture/transition-classification-2026-05-21.json` (1 hits) e.g. `"file": "/Users/jasonlee/oyatie/docs/governance-lanes/codeowners-mirror.md",`
- `docs/machine-readable/catalog.json` (3 hits) e.g. `".github/CODEOWNERS"`
- `docs/security-program/security-program.json` (1 hits) e.g. `"mitigations": "cloud-iam PDP policy (Cedar policy engine; fail-closed, server-side decide()) + break-glass audit + foundation-bypass ledger`
- `evidence/audits/ci-011-shell-cli-retirement-inventory-2026-06-30.json` (1 hits) e.g. `"subcommand": "codeowners-mirror",`
- `evidence/multispectrum/g011-rust-test-wiring-generator-20260610-1781107105.json` (2 hits) e.g. `"libs/oya-check-codeowners-mirror",`
- `governance/check/codeowners-mirror/BUCK` (4 hits) e.g. `name = "check-codeowners-mirror",`
- `governance/check/codeowners-mirror/Cargo.toml` (1 hits) e.g. `name = "check-codeowners-mirror"`
- `governance/check/codeowners-mirror/src/lib.rs` (27 hits) e.g. `//! Foundry CODEOWNERS mirror fitness kernel.`
- `governance/check/doc-catalog/src/lib.rs` (2 hits) e.g. `".github/CODEOWNERS",`
- `governance/check/raci-coverage/src/lib.rs` (8 hits) e.g. `MissingCodeownersCoverage { team_id: String },`
- `libs/oya-governance-gate-catalog-domain/src/lib.rs` (2 hits) e.g. `"codeowners-mirror",`
- `marketplace/facade/dev-cli/BUCK` (5 hits) e.g. `"//governance/check/codeowners-mirror:check-codeowners-mirror",`
- `marketplace/facade/dev-cli/src/commands/gate/mod.rs` (5 hits) e.g. `(Some("validate"), Some("codeowners-mirror")) => {`
- `marketplace/facade/dev-cli/src/documentation_gates.rs` (2 hits) e.g. `if workspace_root.join(".github/CODEOWNERS").is_file() {`
- `marketplace/facade/dev-cli/src/lib.rs` (4 hits) e.g. `list_team_ids, parse_codeowners_mirror_validate_args, parse_raci_team_coverage_validate_args,`
- `marketplace/facade/dev-cli/src/team_ownership_gates.rs` (23 hits) e.g. `use check_codeowners_mirror::{CodeownersEntry, validate_codeowners_mirror};`
- `marketplace/facade/dev-cli/tests/gate_cli.rs` (36 hits) e.g. `fn codeowners_mirror_gate_rejects_unknown_team_owner() {`
- `oya/governance/lane-prefix-migration-task-37.json` (1 hits) e.g. `{"ip_id": "GOV-PREFIX-003", "old_lane_id": "oya-governance-codeowners-mirror", "new_lane_id": "oya-governance-codeowners-mirror", "status": `
- `oya/identity/scorecards/slsa-l3.json` (1 hits) e.g. `"evidence": "CODEOWNERS + branch protection"`
- `oya/intelligence/catalog/oya-intelligence-providers-adapter-openbao.yaml` (1 hits) e.g. `codeowners_review_required: [axis-foundry, ops-security]`
- `registry/catalog/check-codeowners-mirror.yaml` (1 hits) e.g. `capability: check-codeowners-mirror`
- `registry/fixuptasks.jsonl` (2 hits) e.g. `{"id": "F-DRI-CODEOWNERS", "title": "Generate CODEOWNERS from [package.metadata.oya.owner_team] (rejected dri.json + role-roster.json duplic`
- `registry/glossary-vocabulary/ignored-uppercase-words.tsv` (1 hits) e.g. `CODEOWNERS	repository artifact basename, not an acronym`
- `registry/glossary-vocabulary/warning-sources.tsv` (3 hits) e.g. `uncited-acronym	EXISTING	docs/governance-lanes/codeowners-mirror.md`
- `registry/graph/architecture-map.json` (2 hits) e.g. `"id": "crates/oya-check-codeowners-mirror",`
- `registry/quality/lanes.yaml` (4 hits) e.g. `- id: oya-governance-codeowners-mirror`
- `specs/decision-rights.json` (2 hits) e.g. `"lane": "check-codeowners-mirror",`
- `specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002A-good-governance-check-gates-owner-batch.json` (2 hits) e.g. `"crate_name": "oya-check-codeowners-mirror",`
- `specs/markdown-retirement-policy.json` (1 hits) e.g. `"scope": "Rewrite ~11 check crates that grep markdown to consume JSON: check-adr-citation, oya-check-adr-index, check-authority-cohesion, ch`
- `specs/masterplan.json` (1 hits) e.g. `"Treat CI-only gh api/gh pr view carve-out as ADR-0093 Proposed in Step 4 and Accepted only in Step 6 with CODEOWNERS plus same-PR guard."`
- `specs/microservices/scorecards/canonical/slsa-l3.json` (1 hits) e.g. `"evidence_pattern": ".github/branch-protection.yaml (signed commits + required reviewers); CODEOWNERS enforced"`
- `specs/per-microservice-flat-layout.json` (1 hits) e.g. `"path": "CODEOWNERS",`
- `specs/reorg/governance-check-move-plan.json` (6 hits) e.g. `"old_path": "libs/oya-check-codeowners-mirror",`
- `specs/repo-hygiene-automation.json` (3 hits) e.g. `"automation": "GitHub adapter contract tracks visibility, CODEOWNERS/rulesets, status publishing, PR mirror, and public/private split metada`
- `tasks/adr-0357-crate-classification.json` (3 hits) e.g. `"oya-check-codeowners-mirror": {`
- total executable-ish paths with references: 38

### Docs / compliance prose that cite CODEOWNERS
- doc/md paths with references: 53
- `.github/CODE_OF_CONDUCT.md`
- `cloud/cloud-iac/compliance.md`
- `cloud/cloud-k8s/coherence-audit-2026-05-20.md`
- `cloud/cloud-k8s/compliance.md`
- `cloud/cloud-k8s/threat-model.md`
- `cloud/tenancy/compliance.md`
- `cloud/tenancy/threat-model.md`
- `docs/CHANGELOG.md`
- `docs/DESIGN.md`
- `docs/DOC-CATALOG.md`
- `docs/RACI-OWNERSHIP.md`
- `docs/advanced-cicd/branch-pipeline/ADR-0055-branch-pipeline.md`
- `docs/advanced-cicd/branch-pipeline/branch-protection-rules.md`
- `docs/architecture/diagrams/inter-microservice-call-graph.md`
- `docs/architecture/foundry-fitness-to-governance-transition-2026-05-21.md`
- `docs/architecture/six-hops-reachability-audit-2026-05-20.md`
- `docs/audit/initial-sweep-2026-06-06/FOUNDRY-ADJUDICATION-TABLE.md`
- `docs/audit/initial-sweep-2026-06-06/OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md`
- `docs/audit/initial-sweep-2026-06-06/STEP1-TRIAGE.md`
- `docs/audit/initial-sweep-2026-06-06/adr/source-44.md`
- `docs/audit/initial-sweep-2026-06-06/adr/source-6.md`
- `docs/audit/initial-sweep-2026-06-06/backlog-reconciliation/10-extract-backlog.md`
- `docs/audit/initial-sweep-2026-06-06/bominal-reconciliation/12-bominal-strategy.md`
- `docs/audit/initial-sweep-2026-06-06/doc-org/_census-all-docs.txt`
- `docs/audit/initial-sweep-2026-06-06/hyperscaler/specific-tech-choices.md`
- `docs/audit/initial-sweep-2026-06-06/justify-account-robustness/00-JUSTIFY-ACCOUNT-ROBUSTNESS.md`
- `docs/audit/initial-sweep-2026-06-06/justify-account-robustness/10-total-accounting.md`
- `docs/automation/changelog-pipeline.md`
- `docs/checklists/doc-freshness-checklist.md`
- `docs/decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md`
- `docs/decisions/ADR-0092-workspace-dependency-seam-policy.md`
- `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md`
- `docs/decisions/ADR-0131-per-microservice-flat-layout.md`
- `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md`
- `docs/decisions/ADR-0357-vertical-slice-monorepo-nesting.md`
- `docs/decisions/ADR-0364-generative-adr-template-and-masterplan-generation.md`
- `docs/decisions/ADR-0369-gated-stacked-trunk-change-flow.md`
- `docs/decisions/ADR-0555-unaccounted-artifacts-unmergeable-structural-accounting.md`
- `docs/decisions/ADR-0634-approval-attaches-to-the-producer-not-the-reader.md`
- `docs/governance-lanes/INDEX.md`
- `docs/governance-lanes/codeowners-mirror.md`
- `docs/governance/risk-register-2026-05-20.md`
- `docs/ideas/hyperscaler-practices-to-adopt.md`
- `docs/ideas/planning-ssot-consolidation.md`
- `docs/plans/rename-plan-2026-05-12.md`
- `docs/raw/big-tech-dev-cycle-agentic-optimization.md`
- `docs/release/branch-pipeline/branch-protection-rules.md`
- `docs/standards/ci-lanes.md`
- `docs/standards/observability-slo.md`
- `docs/templates/team-charter-template.md`
- `oya/intelligence/runbooks/guardrails-policy-rule-rollback.md`
- `tasks/adr-0357-vertical-slice-nesting-plan.md`
- `templates/checklists/doc-freshness-checklist.md`

### Branch-protection / required-review interaction
- `.github/branch-protection.yaml:20` `#                adr-follow-ups.yaml#adr-0361-promote-pipelines). No PR review;`
- `.github/branch-protection.yaml:26` `#                entry). No PR review; signed-commits + linear-history; FULL`
- `.github/branch-protection.yaml:41` `# Target reviews are agent-run and fully automated. The repo now carries a`
- `.github/branch-protection.yaml:42` `# fail-closed `oya-pr-review` GitHub API adapter contract that binds PR,`
- `.github/branch-protection.yaml:43` `# head SHA, author, designated eligible reviewer, APPROVED verdict, and`
- `.github/branch-protection.yaml:44` `# durable review URL without trusting PR-body text. It is not deployed or wired into the`
- `.github/branch-protection.yaml:46` `# REVIEW-ADMISSION-GAP-LIVE-BOUNDARY: F-PR5-06 remains open. PR #964`
- `.github/branch-protection.yaml:47` `# merged with green `oya-ci-required`, empty reviewDecision, and only an`
- `.github/branch-protection.yaml:48` `# owner COMMENTED review, proving review admission is target-only here.`
- `.github/branch-protection.yaml:49` `required_approving_reviews: 0`
- `.github/branch-protection.yaml:55` `# NOTE: oya-pr-review is intentionally ABSENT from required checks. Once`
- `.github/branch-protection.yaml:66` `# No PR required, no review, no CI gate at THIS layer — those gates`

## Non-negotiable acceptance for either disposition
1. Live `codeowners/errors` after change is either empty because file is gone, or empty because owners resolve for real teams.
2. Required PR review admission still has an executable path (reviewDecision / oya-ci-required / reviewer-agent), not silent zero-routing.
3. RED fixture: unknown-owner config is rejected OR file absence is the explicit retired state with a gate asserting absence.
4. GREEN fixture: chosen end-state admitted under protected CI.
5. Specs/masterplan/security-program/acceptance tests updated atomically with the same PR.
6. No second manual owner registry invented if source-derived authority can be extended.

## Recommended default (not activated)
ADR-0634 D4 recommends **deletion**. Given 111 live unknown-owner errors and explicit "logical until provisioned" header, deletion + explicit no-CODEOWNERS gate is the honest hyperscaler posture until real org teams exist.

## Explicit non-claims
- No file deleted in this planning package.
- No branch-protection mutation.
- No founder decision recorded as accepted beyond the pre-existing ADR-0634 recommendation.

## Parallel dark-lane census notes (G036/G037/G032/G034)
- governance/check Cargo.toml count on origin/dev: **56**
- quality lanes id rows: **96**; status counter: **{'active': 91, 'planned': 5}**; buck2 token count: **80**
- fixuptasks.jsonl has live consumers in baseline-ratchet tests and docs; ADR-0622 names friction-ledger successor foundation.
- friction ledger is live-gated via `ci/facade/action-item-accounting` with hand-curated shrink-only baseline; G032 is founder lifecycle decision, not silent deletion.

## Next executable move (blocked on founder choice for G033)
- If delete: isolated writer PR that removes `.github/CODEOWNERS`, retires/rewires executable consumers, adds absence assertion gate, updates specs atomically, independent review, protected CI, promoted observation.
- If replace: provision real GitHub teams with write access first; unknown-owner must go to zero before claiming routing works.
