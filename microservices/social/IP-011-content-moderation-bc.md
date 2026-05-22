---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-011-content-moderation-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + trust-safety
acceptance_lanes: [cargo-nextest, moderation-policy-test, appeal-evidence-test]
---

# IP-011: Content-moderation bounded context

## A. Problem
Social must moderate abuse, malware, CSAM-adjacent routing, appeals, and DSA/AI Act evidence without turning classifiers into unreviewable platform magic.

## B. Approach
Implement the cataloged content-moderation kernel and ClamAV/OPSWAT adapters plus planned domain/usecase/worker/sdk layers. Verdicts carry classifier version, policy basis, appeal state, audit correlation, and transparency-log fields.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-social-content-moderation-{kernel,adapter-clamav,adapter-opswat}.yaml` | Existing anchors. |
| `src/crates/oya-social-content-moderation-{kernel,domain,usecase,api,adapter-clamav,adapter-opswat,worker,sdk}/` | Planned family named by PRD/IP/catalog. |
| `policy/content-policy.cedar` and `policy/abuse-defence.cedar` | Moderation and abuse controls. |
| `slos/moderation-classifier-latency.openslo.yaml` and `slos/content-policy-enforcement-correctness.openslo.yaml` | Promotion SLOs. |

## D. Ordered implementation steps
1. Define moderation verdict, abuse report, appeal, classifier version, and transparency-event types.
2. Add malware scanner adapter ports for ClamAV and OPSWAT.
3. Implement synchronous pre-publish and async escalation paths.
4. Add appeal workflow state transitions and audit evidence.
5. Test policy denials, quarantine, classifier timeout, rollback, and appeal reversal.
6. Wire DSA transparency worker inputs.
7. Connect dashboards and runbooks for rollback and CSAM/report queues.

## E. Acceptance
- `cargo nextest run -p oya-social-content-moderation-kernel` passes.
- ClamAV and OPSWAT adapter tests pass.
- `slos/moderation-classifier-latency.openslo.yaml` and `slos/content-policy-enforcement-correctness.openslo.yaml` resolve.
- `cargo run -p oya-dev-cli -- gate validate content-policy --microservice social` passes.
- `runbooks/content-moderation-rollback.md` remains current.

## F. Evidence
- PRD FR-14, FR-15, audit/compliance sections: `PRD.md`.
- Policies: `policy/content-policy.cedar`, `policy/abuse-defence.cedar`.
- Decisions: `decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md`.
- Runbooks: `runbooks/content-moderation-rollback.md`, `runbooks/csam-detect-and-ncmec-report.md`.

## G. Counterpart comparison
X, Meta/Instagram/Threads, TikTok, Snapchat, Reddit, Mastodon, and Bluesky all face moderation pressure. Oyatie's differentiator is not looser moderation; it is policy-bound moderation with appeals, audit-chain evidence, DSA transparency, and explicit classifier rollback.

## H. Foundation delivery expansion
- Deliverable detail: moderation verdicts include policy basis, classifier version, confidence, action, appeal state, and audit correlation.
- Deliverable detail: malware adapters expose ClamAV and OPSWAT results through normalized scan records.
- Deliverable detail: pre-publish path supports synchronous refusal and asynchronous escalation.
- Deliverable detail: appeal workflow records submitter, reviewer class, previous verdict, new verdict, and reason.
- Deliverable detail: DSA transparency worker consumes moderation and appeal events.
- Deliverable detail: rollback path can freeze or revert a classifier version.
- Deliverable detail: dashboards report classifier latency, policy enforcement correctness, appeals, CSAM, and backlog counts.
- Deliverable detail: Slack workspace/community moderation is counterpart pressure for channel-scale policy enforcement.

## I. Acceptance expansion
- Acceptance detail: policy denial tests must cover text, media, link, spam, abuse, and CSAM categories.
- Acceptance detail: scanner timeout tests must fail closed where policy requires quarantine.
- Acceptance detail: appeal tests must preserve original and reversed decisions for transparency.
- Acceptance detail: classifier rollback tests must restore previous behavior and emit audit evidence.
- Acceptance detail: DSA event validation must include statement-of-reasons fields.
- Acceptance detail: SLO resolution must include moderation latency and correctness.
- Acceptance detail: runbooks must include rollback, backlog drain, and report export.
- Acceptance detail: Slack, Reddit, X, and TikTok comparisons must map to moderation controls and appeal evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for moderation kernel and scanner adapters.
- Evidence detail: capture content-policy gate output.
- Evidence detail: capture SLO resolution for moderation classifier and enforcement correctness.
- Evidence detail: cite `ADR-SOC-0003-content-moderation-classifier-bounds.md`.
- Evidence detail: cite `runbooks/content-moderation-rollback.md`.
- Evidence detail: cite `runbooks/csam-detect-and-ncmec-report.md` if present.
- Evidence detail: cite Slack as the approved counterpart for community/channel moderation comparison.
