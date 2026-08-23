---
doc_class: Spec
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Concrete branch-protection rules (the SPEC, not the hooks). Mutator allowlists,
  force-push policy, merge-method, Cosign-signed-commit requirement, SLSA L2+
  provenance requirement. Git-server-agnostic; same shape on GitHub / GitLab /
  Gitea / Bitbucket. Encoded in branch-protection.yaml schema.
planned_enforcement_ref:
  - governance-branch-protection-drift
  - governance-no-direct-origin-dev-commit
  - governance-no-direct-staging-commit
  - governance-no-direct-prod-commit
related_adrs: [ADR-0039, ADR-0041]
doc_status: published
---

# Branch Protection Rules

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Provider-agnostic protection schema

Branch protection is encoded in `.github/branch-protection.yaml` (the GitHub-native filename retained for backward compatibility; the schema is the source of truth). The schema is applied through the GitHub branch-protection API.

A nightly drift-check (`governance-branch-protection-drift`, BLOCKER) compares the live branch-protection state to the schema and refuses divergence.

## 2. `dev` branch (origin/dev) protection

```yaml
branches:
  dev:
    protection:
      required_signatures: true             # Cosign-keyless commits per ADR-0039
      enforce_admins: true                  # no admin bypass
      required_linear_history: true         # squash-merge only via dev-promoter
      allow_force_pushes: false
      allow_deletions: false
      restrictions:
        users: []                           # no human users
        teams: []                           # no human teams
        apps:
          - intelligence-dev-promoter        # the ONLY mutator
      required_pull_request_reviews:
        required_approving_review_count: 0  # human-button-free
        require_code_owner_reviews: false   # CODEOWNERS not honoured here
        bypass_pull_request_allowances:
          users: []
          teams: []
          apps:
            - intelligence-dev-promoter
      required_status_checks:
        contexts:
        - governance-cohesion
        - governance-supply-chain
        - governance-api-semver
        - governance-pr-shape
        - governance-pr-review-verdict-present
        - governance-promotion-gate-local-dev-to-origin-dev
        - governance-image-discipline
      merge_methods:
        squash: true
        rebase: false
        merge: false
```

**Constraints encoded.** No force-push (ever); no direct human commits (no human in the `restrictions.users` allowlist); only the `dev-promoter` app may merge; squash-merge only; Cosign-signed commits mandatory; the 3-gate lanes are required status checks.

## 3. `staging` branch protection

```yaml
branches:
  staging:
    protection:
      required_signatures: true             # Cosign-keyless mandatory
      enforce_admins: true
      required_linear_history: true         # fast-forward only
      allow_force_pushes: false
      allow_deletions: false
      restrictions:
        users: []
        teams: []
        apps:
          - intelligence-staging-promoter    # the ONLY mutator
      required_pull_request_reviews:
        required_approving_review_count: 0  # no PR mechanism here; ff-only
      required_status_checks:
        contexts:
        # observational lanes (re-run on push); not strictly gated
        - governance-cohesion
        - governance-supply-chain
      merge_methods:
        squash: false
        rebase: false
        merge: false
        fast_forward: true                  # extension; provider-specific
```

**Constraints encoded.** No force-push; commits come only from `staging-promoter`; fast-forward only (no squash, no merge commits — the staging history mirrors origin/dev's squash-merge sequence); Cosign-signed commits mandatory.

## 4. `prod` branch protection

```yaml
branches:
  prod:
    protection:
      required_signatures: true             # Cosign-keyless mandatory
      required_attestations:
        - slsa-provenance-l2-plus           # NEW; provider-specific via adapter
        - sbom-spdx-2.3
      enforce_admins: true                  # NEVER bypass
      required_linear_history: true         # ff-only
      allow_force_pushes: false             # NEVER (even admins)
      allow_deletions: false                # NEVER
      restrictions:
        users: []
        teams: []
        apps:
          - intelligence-prod-promoter       # the ONLY mutator
      required_pull_request_reviews:
        required_approving_review_count: 0
      required_status_checks:
        contexts:
        - governance-cohesion
        - governance-supply-chain
        - governance-api-semver
        - governance-promotion-gate-staging-to-prod
        - governance-canary-required
        - governance-rollback-evidence
        - governance-cohort-honor
        - governance-slo-burn-rate-fast
      merge_methods:
        squash: false
        rebase: false
        merge: false
        fast_forward: true
```

**Constraints encoded.** No force-push (NEVER); commits come only from `prod-promoter`; fast-forward only; Cosign-signed commits + SLSA L2+ provenance + SBOM 2.3 attestations all mandatory per [ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md); the 5-gate verification lanes are required status checks.

## 5. Per-PR branch protection (the local-dev → origin/dev PR)

PR branches (the agent's local-dev clone tip) have no protection — they are private workspaces. The protection sits at the **target** branch (`origin/dev`). The 3-gate verification fires via the `required_status_checks` on `dev` plus the `dev-promoter`'s orchestration of reviewer agents.

## 6. Mutator allowlist provenance

Each promoter agent ships with a Cosign-keyless identity (per [ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md)). The git-server's `restrictions.apps` allowlist accepts only commits signed by those identities. The identity binding is itself audit-chain-emitted at agent boot; rotation requires a Cosign-signed rotation record + a council-architecture approval.

## 7. CODEOWNERS — minimized

Because reviewer-agent dispatch is per-change-class via `docs/AGENTS.md`, CODEOWNERS is reduced to:

```
# .github/CODEOWNERS — minimal; the heavy lifting is reviewer-agent dispatch
docs/decisions/                          @council-architecture
contracts/                               @council-architecture
.github/branch-protection.yaml           @council-architecture
.github/CODEOWNERS                       @council-architecture
infra/argo-rollouts/templates/**         @axis-foundry
infra/kyverno/policies/**                @axis-foundry
```

Human reviewers are involved only for **CODEOWNERS-pathed** changes (ADR text, contract text, branch-protection itself). All other paths use reviewer-agent dispatch.

## 8. Drift detection

`governance-branch-protection-drift` (BLOCKER) runs nightly. Reconciles live state to schema. Drift → PR auto-opened by `dev-promoter` to restore the schema (with a synthetic reviewer-agent verdict from `branch-protection-reviewer` agent; an exception to the change-class table for this specific file).

## 9. Hot-fix path (carve-out)

Per [`rollback-mechanics-per-stage.md`](rollback-mechanics-per-stage.md), the hot-fix path uses the same protection — no carve-out for "emergency direct push." Critical fixes land on local-dev → origin/dev with reviewer-agent verdict + CI green; `staging-promoter` accelerates the batch (emergency cadence ≤ 60 sec); `prod-promoter` uses a reduced gate set documented in the rollback doc + Directive 12 human-orchestrator signature.

## 10. Anti-scope

This file does not own:

- Cosign identity rotation — owned by [ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md).
- Per-axis fitness-lane logic — owned per ADR.
- Git-server provider choice — agnostic via adapter pattern.

## 11. Lift target

`oyatie/docs/release/branch-pipeline/branch-protection-rules.md` on approval.
