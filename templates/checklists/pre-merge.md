---
doc_status: published
---

# Checklist: Pre-merge

> **When:** Before `gh pr merge`. After all CI lanes green + reviewer-agent verdict.
> **Owner:** PR author + reviewer.
> **Validator:** protected PR status `oya-ci-required` + reviewer-agent verdict. Local hook output is advisory evidence only.

---

1. ☐ All CI lanes green per [RELEASE-MANAGEMENT.md §2](../RELEASE-MANAGEMENT.md):
   - Cargo/cloud-ci format, lint, test, and type lanes for affected targets
   - license, architecture-boundary, claim-ceiling, foundation-bypass, and plane-class gates through cloud-ci/oya-ci
   - Trivy / Cosign / SBOM
   - oya-governance-{license, data-class, cohesion, doc-catalog, slo-coverage, blast-radius}
2. ☐ PR has the four canonical H2s: `## Issue / Summary / Verification / Code Review`
3. ☐ `## Verification` lists every check from CI + outcome (no hand-wave), including the `oya-ci-required` head SHA and check/status URL
4. ☐ `## Summary` cites the canonical authority and names flat-crate or cross-axis impact when applicable
5. ☐ Scope-specific evidence (for example eval-set or security evidence) is in `## Verification` or the review thread; no generic packet is required
6. ☐ Dogfood tenant invariant evidence is present when a cloud/product boundary changes: tenant identity, tenancy boundary, policy/RBAC, residency/isolation, audit/evidence, and lifecycle remain first-class contract surfaces consumed through APIs/controllers/GitOps/admission/policy/frontends, with no privileged product shortcut into cloud internals.
7. ☐ An author-distinct reviewer APPROVE is bound to the current PR head and all review threads are resolved; one reviewer is sufficient and CI green alone is not approval
8. ☐ The single author-distinct reviewer applied the relevant blast-radius
   lenses per [DESIGN §3.0.5.3](../DESIGN.md); affected axis, privacy,
   compliance, and security owners were notified for non-binding input
9. ☐ Glossary alignment per `oya-governance-glossary` — no new domain term without GLOSSARY entry
10. ☐ ADR cited where applicable (only new pack ADR-0001..0051, plus future pack ADRs after their files exist; legacy ADR-#### forbidden in active text)
11. ☐ Brand check — canonical `Oyatie` usage preserved; deprecated aliases and tautological rebrand statements rejected (per ADR-0017 / MFL-0011)
12. ☐ Capability eval-set passes (if governed capability change)
13. ☐ Migration ledger entry (if ADR-0015 flat-crates phase PR)
14. ☐ `Co-Authored-By:` footer present if agent-paired
15. ☐ `Signed-off-by:` per signed-commits posture
16. ☐ No review bypass or self-approval is present

## Anti-patterns

- Bypassing review for cross-axis contract change — never
- Bundling unrelated changes — never (one PR per concern)
- Skipping `## Verification` — leaves the reviewer without execution evidence
- Citing legacy ADR-#### — replace with new pack ADR-0001..0051 per [ADR-CONSOLIDATION-PLAN.md](../ADR-CONSOLIDATION-PLAN.md)

## Sources
CLAUDE.md PR rules; [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md); [standards/code-review.md](../standards/code-review.md).
