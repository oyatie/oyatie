---
doc_status: published
---

# Checklist: Pre-merge

> **When:** Before `gh pr merge`. After all CI lanes green + reviewer-agent verdict.
> **Owner:** PR author + reviewer.
> **Validator:** protected PR status `oya-ci-required` + reviewer-agent verdict. Local hook output is advisory evidence only.

---

1. ☐ All CI lanes green per [RELEASE-MANAGEMENT.md §2](../RELEASE-MANAGEMENT.md):
   - Buck2/cloud-ci format, lint, test, and type lanes for affected targets
   - license, architecture-boundary, claim-ceiling, foundation-bypass, and plane-class gates through cloud-ci/oya-ci
   - Trivy / Cosign / SBOM
   - oya-governance-{license, data-class, cohesion, doc-catalog, slo-coverage, blast-radius}
2. ☐ PR has 5 mandatory H2s: `## Issue / Summary / Verification / Traceability / Evidence`
3. ☐ `## Verification` lists every check from CI + outcome (no hand-wave), including the `oya-ci-required` head SHA and check/status URL
4. ☐ `## Traceability` lists flat-crates targets touched + cross-axis contract impact
5. ☐ `## Evidence` links to CI runs + eval-set output (if governed capability) + audit-chain emission record, and includes the review/fix evidence packet: exact failing/fixed checks, review-thread resolution, reviewer approval state, local-CLI non-authority, and generated-face no-hand-edit status
6. ☐ Dogfood tenant invariant evidence is present when a cloud/product boundary changes: tenant identity, tenancy boundary, policy/RBAC, residency/isolation, audit/evidence, and lifecycle remain first-class contract surfaces consumed through APIs/controllers/GitOps/admission/policy/frontends, with no privileged product shortcut into cloud internals.
7. ☐ Reviewer-agent verdict added as `## Code Review` H2 (lead-only — never as worker), with APPROVE bound to the current PR head and all review threads resolved
8. ☐ Per-blast-radius reviewers approved per [DESIGN §3.0.5.3](../DESIGN.md):
   - cross-axis-contract → all affected axis teams
   - flat-crates-move → merge-queue serialization on root Cargo.toml
   - data-class-impact → privacy-governance
   - regulatory-impact → ops-compliance
   - security-class → ops-security
9. ☐ Glossary alignment per `oya-governance-glossary` — no new domain term without GLOSSARY entry
10. ☐ ADR cited where applicable (only new pack ADR-0001..0051, plus future pack ADRs after their files exist; legacy ADR-#### forbidden in active text)
11. ☐ Brand check — canonical `Oyatie` usage preserved; deprecated aliases and tautological rebrand statements rejected (per ADR-0017 / MFL-0011)
12. ☐ Capability eval-set passes (if governed capability change)
13. ☐ Migration ledger entry (if ADR-0015 flat-crates phase PR)
14. ☐ `Co-Authored-By:` footer present if agent-paired
15. ☐ `Signed-off-by:` per signed-commits posture
16. ☐ Bypass-reason logged (if `# review-bypass:` used; never for cross-axis / privacy / security / ADR / release-tag)

## After merge

17. ☐ Promoted commit SHA recorded; post-merge `oya-ci-required` remains green on
    the promoted commit within 5 min, with status URL
18. ☐ Rollout verification recorded: deployment/canary/flag state, tenant/customer surface, and operator
19. ☐ Rollback note recorded: exact rollback command/runbook/digest, or `no deployable artifact` with rationale
20. ☐ Observability check recorded: golden-signal/SLO dashboard, time window, and no active burn-rate block
21. ☐ Browser UX/user-story evidence recorded for user-visible surfaces:
    browser/session, story path, and screenshot/video/artifact; if not user-visible,
    record `not user-visible` with rationale
22. ☐ Release-governance/release-note impact recorded: release PR/link or
    generated notes from the configured release system, or `no user-facing
    release-note impact`; Release Please is only required when a live repo
    config/workflow exists
23. ☐ Audit-chain emits `EVT-PR-MERGED` with PR # + commit SHA
24. ☐ Per-affected-team Slack ping (auto)
25. ☐ Per-changelog auto-emit (governed capability `pr.changelog.row`)

## Anti-patterns

- Bypassing review for cross-axis contract change — never
- Bundling unrelated changes — never (one PR per concern)
- Skipping `## Verification` — fails traceability
- Citing legacy ADR-#### — replace with new pack ADR-0001..0051 per [ADR-CONSOLIDATION-PLAN.md](../ADR-CONSOLIDATION-PLAN.md)

## Sources
CLAUDE.md PR rules; `scripts/hooks/guard-pr-merge-review.mjs`; [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md); [standards/code-review.md](../standards/code-review.md).
