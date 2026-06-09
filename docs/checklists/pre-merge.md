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
3. ☐ `## Verification` lists every check from CI + outcome (no hand-wave)
4. ☐ `## Traceability` lists flat-crates targets touched + cross-axis contract impact
5. ☐ `## Evidence` links to CI runs + eval-set output (if governed capability) + audit-chain emission record
6. ☐ Dogfood tenant invariant evidence is present when a cloud/product boundary changes: tenant identity, tenancy boundary, policy/RBAC, residency/isolation, audit/evidence, and lifecycle remain first-class contract surfaces consumed through APIs/controllers/GitOps/admission/policy/frontends, with no privileged product shortcut into cloud internals.
8. ☐ Reviewer-agent verdict added as `## Code Review` H2 (lead-only — never as worker)
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

16. ☐ Post-merge cloud-ci/oya-ci required status remains green on the promoted commit within 5 min
18. ☐ Audit-chain emits `EVT-PR-MERGED` with PR # + commit SHA
19. ☐ Per-affected-team Slack ping (auto)
110. ☐ Per-changelog auto-emit (governed capability `pr.changelog.row`)

## Anti-patterns

- Bypassing review for cross-axis contract change — never
- Bundling unrelated changes — never (one PR per concern)
- Skipping `## Verification` — fails traceability
- Citing legacy ADR-#### — replace with new pack ADR-0001..0051 per [ADR-CONSOLIDATION-PLAN.md](../ADR-CONSOLIDATION-PLAN.md)

## Sources
CLAUDE.md PR rules; `scripts/hooks/guard-pr-merge-review.mjs`; [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md); [standards/code-review.md](../standards/code-review.md).
