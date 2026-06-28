---
purpose: Oyatie — Code Review Standard
doc_status: published
---

# Oyatie — Code Review Standard

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `council-architecture`.
> **Companion:** CLAUDE.md / docs/AGENTS.md Code Review rules. The historical
> `scripts/hooks/guard-pr-merge-review.mjs` reference is advisory only; it is not
> live cloud admission authority. `F-PR5-06` tracks the trusted server-side/cloud-ci review producer gap for `oya-pr-review`.

## 1. Per-change-class reviewer agent

Every PR auto-classifies into change classes (per [DESIGN §3.0.5.3](../DESIGN.md) blast-radius):

| Change class | Reviewer agent | Review aim |
|---|---|---|
| Rust code | `rust-reviewer` | clippy + idioms + perf + safety |
| TypeScript / JS | `typescript-reviewer` | tsc + lint + idioms |
| Database / SQL / migration | `database-reviewer` | migration safety + rollback + schema versioning |
| Security / secrets / auth | `security-reviewer` | threat model + OWASP + secrets handling |
| Privacy / data-class | `council-privacy` | Data Use Boundary check + class annotations |
| Cross-axis contract | per-axis owners + `council-architecture` | contract semver + consumer impact |
| Foundry capability | `axis-foundry` reviewer | eval-set pass + autonomy-tier check |
| Regional pack | per-pack maintainer | seam-impl coverage + regulatory-bind |
| ADR | `crew-adr-promotion` | ADR template adherence + supersession graph |
| Docs (consolidated) | per-doc owner per [DOC-CATALOG.md](../DOC-CATALOG.md) | dep-doc cascade + glossary alignment |

## 2. Mandatory PR sections

Per CLAUDE.md (5 H2s):
- `## Issue` — what + Refs/Closes/Blocks
- `## Summary` — what changed (1-3 sentences)
- `## Verification` — what was tested + outcome
- `## Traceability` — flat-crates targets touched + cross-axis contract impact
- `## Evidence` — links to CI runs, eval-set output, audit-chain emission

Target requirement for merge-ready PRs: `## Code Review` — supplied by the
automated reviewer-agent verdict. Author-only drafts may omit it only when
running the traceability gate with the explicit author policy. Today this is
target/advisory evidence; it becomes merge authority only when a trusted
server-side/cloud-ci review producer is live and required (`F-PR5-06`).

## 3. Per-class review requirements

| Change class | Required reviewers | Mandatory checks |
|---|---|---|
| Rust code | rust-reviewer + 1 human peer | clippy clean; nextest pass; bench gate if perf-tagged |
| Cross-axis contract | each affected axis team + council co-sign | semver-diff + cohesion-fitness |
| Privacy / data-class | council-privacy | data-class annotation + DSR-cascade test |
| Security | ops-security | threat-model update if needed |
| Migration | database-reviewer | rollback path + dry-run + ≥ 2 prior version backward-read |
| ADR | crew-adr-promotion | template adherence + supersession back-link |
| Brand-rename batch | per-batch owner | per-batch checklist completion |

## 4. Verdict format

`## Code Review` H2 (target: added by reviewer agent at merge time):

```
## Code Review

**Reviewer agent:** rust-reviewer
**Verdict:** APPROVE | REQUEST CHANGES | BLOCKED
**Resolved items:** ...
**Deferred items:** ...
**Outstanding concerns:** ...
```

Multiple reviewers: one block per reviewer agent.

## 5. Bypass

When the server-side reviewer gate is live, any intentional skip must be a
cloud-recorded bypass packet, not a shell-only or local-hook convention.
Historical examples such as `# review-bypass: <reason>` are local/advisory
prose until `F-PR5-06` closes.

Every bypass:
- Recorded by the trusted review-admission producer once live
- Emitted as `EVT-CODE-REVIEW-BYPASS` audit event
- Quarterly review by council
- Excessive bypass per-team triggers escalation

Never bypass for: cross-axis contract changes; privacy / data-class changes; security changes; ADR changes; release tags.

## 6. Async review SLA

- Sev-impacting (post-incident hotfix): same-day
- Cross-axis contract: 5 business days
- Per-axis routine: 2 business days
- Doc-only: 5 business days

## 7. Anti-patterns

- Self-approving in same active context — never (per project memory)
- Reviewer-shopping for laxer reviewer — never
- Bundling unrelated changes — never (one PR per concern)
- Skipping `## Verification` section — fails traceability CI

## 8. Sources
CLAUDE.md "Code Review", `scripts/hooks/guard-pr-merge-review.mjs`, [DESIGN.md §3.0.5](../DESIGN.md), ADR-0007, ADR-0019, ADR-0050.
