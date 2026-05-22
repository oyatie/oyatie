---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0294]
acceptance_status: draft
companion_docs: [microservices/social/catalog/oya-social-dsa-transparency-worker.yaml]
inbound_citations: [microservices/social/manifest.json]
---

# IP-018: DSA compliance overlay

## A. Problem
EU DSA obligations require transparency, notice, appeal, statement-of-reasons, and reporting evidence for moderation decisions; this cannot be a later prose-only compliance promise.

## B. Approach
Implement the cataloged DSA transparency worker and bind moderation verdicts, appeals, notices, reports, and retained evidence to contracts and dashboards. The overlay is EU/pack-aware but should not weaken non-EU audit trails.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-dsa-transparency-worker.yaml` | Existing worker catalog anchor. |
| `src/crates/oya-social-dsa-transparency-worker/` | Planned worker path named by catalog/IP. |
| `runbooks/dsa-transparency-report-generation.md` | Report generation runbook. |
| `dashboards/moderation-and-safety.json` | Moderation evidence dashboard. |
| `policy/content-policy.cedar` | Moderation basis source. |

## D. Ordered implementation steps
1. Define statement-of-reasons, notice, appeal, report, and transparency-export records.
2. Consume moderation verdict and appeal events.
3. Generate per-tenant DSA transparency report artifacts.
4. Preserve audit correlation, classifier version, policy basis, and appeal status.
5. Test deleted content, reversed decisions, appealed decisions, and regulator export.
6. Wire dashboard and report runbook evidence.
7. Add retention and data-residency checks for EU packs.

## E. Acceptance
- `cargo nextest run -p oya-social-dsa-transparency-worker` passes.
- `runbooks/dsa-transparency-report-generation.md` includes generation, validation, and rollback steps.
- `dashboards/moderation-and-safety.json` validates as JSON.
- DSA report fixtures include statement-of-reasons and appeal status.
- Policy basis resolves to `policy/content-policy.cedar`.

## F. Evidence
- Catalog: `catalog/oya-social-dsa-transparency-worker.yaml`.
- Policy: `policy/content-policy.cedar`, `policy/data-residency.md`.
- Runbook/dashboard: `runbooks/dsa-transparency-report-generation.md`, `dashboards/moderation-and-safety.json`.
- Compliance: `compliance.md`, `dpia.md`.

## G. Counterpart comparison
X, Threads/Instagram, TikTok, Snapchat, Reddit, Mastodon, and Bluesky all face DSA transparency expectations in Europe. Oyatie's target is report generation and appeal evidence as repo-governed service behavior, not after-the-fact manual compliance.

## H. Foundation delivery expansion
- Deliverable detail: DSA records include statement of reasons, notice id, user action, policy basis, classifier version, appeal state, and export id.
- Deliverable detail: worker consumes moderation verdict, appeal, deletion, restoration, and regulator-export events.
- Deliverable detail: per-tenant reports include counts by action, policy basis, appeal outcome, and region.
- Deliverable detail: retention rules keep regulator evidence without over-retaining removed content.
- Deliverable detail: EU pack behavior binds to data residency and disclosure controls.
- Deliverable detail: dashboard metrics include report generation, failed exports, appeal reversals, and backlog.
- Deliverable detail: runbook covers generation, validation, export, rollback, and correction.
- Deliverable detail: Slack community moderation transparency is counterpart pressure for explainable notices.

## I. Acceptance expansion
- Acceptance detail: report fixtures must include statement-of-reasons and appeal status.
- Acceptance detail: reversed-decision tests must preserve original and corrected transparency records.
- Acceptance detail: deleted-content tests must retain evidence without exposing restricted content.
- Acceptance detail: EU residency tests must bind DSA artifacts to allowed storage locations.
- Acceptance detail: dashboard JSON must validate for moderation and safety metrics.
- Acceptance detail: runbook must include validation and rollback steps.
- Acceptance detail: policy basis must resolve to `policy/content-policy.cedar`.
- Acceptance detail: Slack, Reddit, X, TikTok, and Mastodon comparisons must map to moderation transparency evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for `oya-social-dsa-transparency-worker`.
- Evidence detail: capture generated DSA report fixture validation.
- Evidence detail: capture dashboard JSON validation for `moderation-and-safety.json`.
- Evidence detail: cite `catalog/oya-social-dsa-transparency-worker.yaml`.
- Evidence detail: cite `policy/content-policy.cedar` and `policy/data-residency.md`.
- Evidence detail: cite `compliance.md` and `dpia.md`.
- Evidence detail: cite Slack as community moderation transparency pressure alongside European social-platform counterparts.
