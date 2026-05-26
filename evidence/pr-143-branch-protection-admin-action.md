---
doc_class: AdminActionRequired
pr: 143
target_branch: dev
generated_at: 2026-05-18
status: admin_action_required
---

# PR #143 — Branch protection admin action required

## Summary

Branch protection on `dev` requires status checks named `oya-governance-*`. Per handoff Q2 + ADR-0128 + ADR-0133, `oya-governance-*` naming was retired in favor of the `governance` µservice. PR #143's commits no longer emit checks under the old names — therefore PR #143 cannot merge under current branch-protection rules without admin action.

This document is `documented` (not `green`) on the merge-admissibility report because it requires repo admin scope, not self-fixable in the PR.

## Affected check names (likely still required on dev)

Inferred from `.github/workflows/oya-governance-*.yml` files on origin/dev:

| Old check name (retired) | New equivalent |
|---|---|
| `oya-governance-supply-chain` | `oya-governance-supply-chain` (via `microservices/governance/` IP-NNN — handoff #13/#18) |
| `oya-governance-cohesion` | `oya-governance-authority-cohesion` |
| `oya-governance-api-semver` | `oya-governance-api-semver` |
| `oya-governance-protection-context-match` | `oya-governance-protection-context-match` |
| `oya-governance-aspirational-enforcement` | `oya-governance-aspirational-enforcement` (handoff #13) |
| `oya-governance-evidence-secret-scan` | `oya-governance-evidence-secret-scan` |
| `oya-governance-honest-claims` | `oya-governance-honest-claims` (handoff #13) |
| `oya-governance-master-plan-completion` | `oya-governance-master-plan-completion` |
| `oya-governance-sequential-pr-merge-conflicts` | `oya-governance-sequential-pr-merge-conflicts` |

## Admin command (run as repo admin)

```bash
gh api -X PATCH repos/{OWNER}/{REPO}/branches/dev/protection \
  --input <(gh api repos/{OWNER}/{REPO}/branches/dev/protection \
    | jq '.required_status_checks.contexts |= map(
        gsub("oya-governance-"; "oya-governance-")
      )')
```

## Verification after admin runs the command

```bash
gh api repos/{OWNER}/{REPO}/branches/dev/protection \
  | jq -r '.required_status_checks.contexts[]' \
  | grep -E '^oya-(foundry-fitness|governance)-'
```

Expected: zero `oya-governance-*` rows.

## Rollback if PR #143 must merge before admin action

Option A (recommended): admin updates branch protection FIRST, then PR #143 rebases + merges with new check names.

Option B (emergency): admin bypasses branch protection for PR #143 single-merge via `merge with admin override`. PR #143's commit body must then carry the rationale + the rotation IP scheduled within the next 24h.

## Cross-references

- HANDOFF-2026-05-17-claude-to-pr143-agent.md §"Open questions" Q2
- ADR-0128 hyperscaler invariants binding
- ADR-0133 industry-best-practice conformance program
- `microservices/governance/PRD.md` — single source of truth for the governance µservice
- handoff #13 — Fitness-is-Governance migration (governance is new home for all check crates)
