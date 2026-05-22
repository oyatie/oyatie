---
doc_status: published
---

# Checklist: Pre-merge

> **When:** Before `gh pr merge`. After all CI lanes green + reviewer-agent verdict.
> **Owner:** PR author + reviewer.
> **Validator:** `oya gate validate` + `guard-pr-merge-review.mjs`.

---

1. ☐ All CI lanes green per [RELEASE-MANAGEMENT.md §2](../RELEASE-MANAGEMENT.md):
   - cargo-fmt / cargo-clippy / cargo-nextest --all-features / cargo check --all-features
   - cargo-deny licenses / `oya gate validate architecture-boundaries`
   - oya catalog validate / oya gate validate (claim-ceiling, foundation-bypass, plane-class)
   - Trivy / Cosign / SBOM
   - oya-governance-{license, data-class, cohesion, doc-catalog, slo-coverage, blast-radius}
2. ☐ PR has 5 mandatory H2s: `## Issue / Summary / Verification / Traceability / Evidence`
3. ☐ `## Verification` lists every check from CI + outcome (no hand-wave)
4. ☐ `## Traceability` lists flat-crates targets touched + cross-axis contract impact
5. ☐ `## Evidence` links to CI runs + eval-set output (if Foundry capability) + audit-chain emission record
6. ☐ Reviewer-agent verdict added as `## Code Review` H2 (lead-only — never as worker)
7. ☐ Per-blast-radius reviewers approved per [DESIGN §3.0.5.3](../DESIGN.md):
   - cross-axis-contract → all affected axis teams
   - flat-crates-move → merge-queue serialization on root Cargo.toml
   - data-class-impact → council-privacy
   - regulatory-impact → ops-compliance
   - security-class → ops-security
8. ☐ Glossary alignment per `oya-governance-glossary` — no new domain term without GLOSSARY entry
9. ☐ ADR cited where applicable (only new pack ADR-0001..0051, plus future pack ADRs after their files exist; legacy ADR-#### forbidden in active text)
10. ☐ Brand check — canonical `Oyatie` usage preserved; deprecated aliases and tautological rebrand statements rejected (per ADR-0017 / MFL-0011)
11. ☐ Capability eval-set passes (if Foundry capability change)
12. ☐ Migration ledger entry (if ADR-0015 flat-crates phase PR)
13. ☐ `Co-Authored-By:` footer present if agent-paired
14. ☐ `Signed-off-by:` per signed-commits posture
15. ☐ Bypass-reason logged (if `# review-bypass:` used; never for cross-axis / privacy / security / ADR / release-tag)

## After merge

16. ☐ Post-merge `cargo check --workspace --all-features` on `main` green within 5 min
17. ☐ Audit-chain emits `EVT-PR-MERGED` with PR # + commit SHA
18. ☐ Per-affected-team Slack ping (auto)
19. ☐ Per-changelog auto-emit (Foundry capability `pr.changelog.row`)

## Anti-patterns

- Bypassing review for cross-axis contract change — never
- Bundling unrelated changes — never (one PR per concern)
- Skipping `## Verification` — fails traceability
- Citing legacy ADR-#### — replace with new pack ADR-0001..0051 per [ADR-CONSOLIDATION-PLAN.md](../ADR-CONSOLIDATION-PLAN.md)

## Sources
CLAUDE.md PR rules; `scripts/hooks/guard-pr-merge-review.mjs`; [RELEASE-MANAGEMENT.md](../RELEASE-MANAGEMENT.md); [standards/code-review.md](../standards/code-review.md).
