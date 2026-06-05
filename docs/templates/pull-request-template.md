---
doc_status: published
doc_class: Template
template_id: TPL-PR
status: Accepted
owner_team: axis-foundry + council-architecture
---

# PR template

> Canonical PR body for every Oyatie change. Keep this file current and avoid
> parallel PR-template variants; legacy aliases should point here.

<!-- agent-instructions:start -->
canonical_template: docs/templates/pull-request-template.md
merge_authority: Prow/Kubernetes-native oya-ci-required context + Buck2 evidence
compatibility_adapter: GitHub pull request + GitHub Actions shadow checks while native SCM/CI matures
required_local_evidence:
  - targeted Buck2 build/test/check commands for touched surfaces
  - multispectrum evidence file under evidence/multispectrum/
  - stale-reference scan for retired surface names touched by the change
forbidden_pr_evidence:
  - non-Buck2 local output as sole merge authority
  - retired local governance CLI authority
  - retired external SCM/CI/CD substrates as interim authority
<!-- agent-instructions:end -->

## Issue
Refs #<n> or Closes #<n>. Name the change class on this line: `feature | bugfix | refactor | migration | docs | chore | capability | plugin | runbook | ADR | pack-update`.

## Summary
- 1-3 bullets on what changed and why.
- Cite the canonical authority read first when the change is policy, template, runtime, CI, or architecture related.

## Verification
Paste fresh command output or concise PASS/FAIL evidence. Prefer Buck2 targets as build/test/check authority.

- [ ] Targeted Buck2 evidence: `<command>` — `<PASS|FAIL>` — `<output excerpt>`
- [ ] Repo/policy evidence where applicable: `buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check` — `<PASS|FAIL>`
- [ ] Coverage evidence where applicable: `buck2 build //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check` — `<PASS|FAIL>`
- [ ] Formatting/static checks for touched Rust: `rustfmt --edition 2024 --check <files>` and/or Buck2 lint target — `<PASS|FAIL>`
- [ ] Retired-surface/stale-reference scan for touched domains — `<PASS|FAIL>`
- [ ] GitHub adapter checks before merge: governance, buck2-authority, rust-llvm-coverage, affected-build, `github-lane-unlocker-required`, and cd-dry-run — `<PASS|FAIL>`

## Traceability
- Catalog/spec/registry records touched: `<list>`
- Cross-axis contracts touched: `<list or none>`
- Decisions/specs cited: `<list>`
- Shared-surface conflict risk: `<none|low|medium|high>` and mitigation.

## Evidence
- Multispectrum evidence: `evidence/multispectrum/<change-id>-<timestamp>.json`
- Audit-chain emission: `<event-id or none>`
- Foundation-bypass referenced: `<bypass-id or none>`
- Per-pack regulator-watch impact: `<list or none>`

## Code Review
- Required merge context: `oya-ci-required` target; GitHub Actions checks are compatibility/shadow evidence until native cutover.
- Reviewer evidence: `<agent/human reviewer, or self-review with explicit local+remote evidence when no reviewer service is live>`
- Verdict: `<APPROVE|REQUEST CHANGES>`
- Resolved items: `<list or none>`
- Deferred items: `<list with owners, or none>`
