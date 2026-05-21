---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-001
title: Branch-protection deploy + live drift enforcement
status: in-progress
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_blocker_ref: "Top blocker #1: branch protection declared but not deployed"
purpose: Deploy the existing `.github/branch-protection.yaml` ruleset to live GitHub and keep live branch protection mechanically aligned with repo policy so Layer-2's CI gate is server-enforced rather than declarative-only.
---

# M01-P17-IP-001 — Branch-protection deploy + live drift enforcement

## Scope

`.github/branch-protection.yaml` now declares 15 required checks for `dev`: the cargo quartet, Oya VCS admission/provider execution, supply-chain, cohesion, api-semver, honest-claims, aspirational-enforcement, banned-primitives, protection-context-match, governance dependency-seam, and `oya-pr-review`. The 2026-05-19 live audit found GitHub `dev` protection requiring only 10 contexts, leaving five canonical checks unenforced even while PR #148 reported `CLEAN/MERGEABLE`.

This IP deploys the `dev` ruleset, keeps `infra/branch-protection/dev.json` aligned with `.github/branch-protection.yaml`, and extends the existing `oya-foundry-fitness-protection-context-match` lane so it re-verifies live GitHub required contexts against the on-disk policy on every PR. The live-check step fails closed unless a read token with GitHub Administration read permission is available.

## Dependencies

None. Wave-1 IP; can ship before all other M01-P17 IPs.

## Acceptance

- `gh api repos/<owner>/oyatie/branches/dev/protection/required_status_checks` returns all 15 `.github/branch-protection.yaml` contexts with no missing or extra contexts.
- `infra/branch-protection/dev.json` mirrors `.github/branch-protection.yaml` for the `dev.required_status_checks.contexts` set.
- `oya-foundry-fitness-protection-context-match` blocks if either YAML→workflow names drift or live GitHub required contexts drift from YAML.
- The workflow has a configured `OYA_BRANCH_PROTECTION_READ_TOKEN` secret backed by a token with GitHub Administration read permission; missing token is a hard failure, not a skipped advisory signal.
- Evidence at `/evidence/pipeline-maturity-glue/ip-001-branch-protection-live-drift-2026-05-19.json` includes the live API response, repo-policy comparison, and local/live drift gate output.
- A test PR with a deliberately failing required context is BLOCKED from merge by the live ruleset (not just by the workflow rollup).

## Symbols to grit-claim

- `.github/branch-protection.yaml::*` (file-level claim — content + drift baseline)
- `infra/branch-protection/dev.json::*` (GitOps deploy source)
- `crates/oya-dev-cli/src/protection_context_match_gate.rs::*` (existing gate runner extended with live drift input)
- `crates/oya-dev-cli/tests/gate_cli.rs::*` (synthetic live-drift violation)
- `.github/workflows/oya-foundry-fitness-protection-context-match.yml::*` (required existing workflow fails closed on live drift)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-001-branch-protection-live-drift-2026-05-19.json`
- `/evidence/pipeline-maturity-glue/ip-001-deploy-runbook-completion.json` (admin attests one-time deploy executed)


## Closeout evidence (2026-05-20)

- Live dev branch-protection required contexts match `.github/branch-protection.yaml` and `infra/branch-protection/dev.json` with no missing or extra contexts.
- `scripts/github-actions-required-secrets-check.sh --repo jason931225/oyatie --branch dev --config infra/branch-protection/dev.json` passed, proving the hosted `OYA_BRANCH_PROTECTION_READ_TOKEN` secret is visible enough for the governance check.
- `oya gate validate protection-context-match` passed with 15 required contexts and 45 workflow jobs indexed.
- Hosted PR evidence: `https://github.com/jason931225/oyatie/pull/151` head `7a923b90701b9fef281821d23129f5b15bff0cfe` has successful `oya-governance-protection-context-match` and required governance/fitness workflow runs.
- Evidence: `/evidence/fd001/cs-fd001-branch-protection-closeout-2026-05-20.json` and `/evidence/multispectrum/cs-fd001-branch-protection-closeout-2026-05-20.json`.
