---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P17-IP-001
title: Branch-protection deploy + auto-merge enablement
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_audit: ../../../../../../evidence/audits/pipeline-maturity-audit-2026-05-15.md
audit_blocker_ref: "Top blocker #1: branch protection declared but not deployed"
purpose: Deploy the existing `.github/branch-protection.yaml` ruleset to live GitHub and enable auto-merge so Layer-2's CI gate becomes server-enforced rather than declarative-only.
---

# M01-P17-IP-001 — Branch-protection deploy + auto-merge enablement

## Scope

`.github/branch-protection.yaml` already declares 9 required checks (`cargo-fmt`, `cargo-check`, `cargo-clippy`, `cargo-nextest`, `oya-vcs-admission`, `oya-vcs-provider-execution`, `oya-foundry-fitness-supply-chain`, `-cohesion`, `-api-semver`). The live GitHub API returns `Branch not protected` for `main` — the ruleset is unenforced server-side. This IP deploys the ruleset, flips `allow_auto_merge: true`, sets `delete_branch_on_merge: true`, and adds a fitness lane that re-verifies the live ruleset matches the on-disk YAML on every PR. **URGENT — highest-leverage single fix in the audit.**

## Dependencies

None. Wave-1 IP; can ship before all other M01-P17 IPs.

## Acceptance

- `gh api repos/<owner>/oyatie/branches/main/protection` returns 200 with all 9 required checks listed (not 404).
- Repo metadata `allow_auto_merge: true`, `delete_branch_on_merge: true`.
- A new fitness lane `oya-foundry-fitness-branch-protection-drift-kernel` diffs live GitHub ruleset against on-disk `.github/branch-protection.yaml` and BLOCKS on drift; baseline-zero on day 1.
- Evidence at `/evidence/pipeline-maturity-glue/ip-001-branch-protection.json` includes the GitHub API response, repo-metadata snapshot, and drift-lane first green run.
- A test PR with a deliberately failing `cargo-fmt` step is BLOCKED from merge by the live ruleset (not just by the workflow).

## Symbols to grit-claim

- `.github/branch-protection.yaml::*` (file-level claim — content + drift baseline)
- `crates/oya-foundry-fitness-branch-protection-drift-kernel/src/lib.rs::*` (new crate)
- `tools/oya-foundry-fitness-branch-protection-drift-app/src/main.rs::main` (new binary)
- `.github/workflows/branch-protection-drift.yml::*` (new workflow)
- `docs/runbooks/branch-protection-deploy.md::*` (admin runbook for the one-time GitHub API deploy)

## Exit evidence

- `/evidence/pipeline-maturity-glue/ip-001-branch-protection.json`
- `/evidence/pipeline-maturity-glue/ip-001-deploy-runbook-completion.json` (admin attests one-time deploy executed)
