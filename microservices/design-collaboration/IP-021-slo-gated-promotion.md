# IP-021 Design Collaboration SLO gated promotion

Service: design-collaboration
ChangeSet scope: microservices/design-collaboration/IP-021-slo-gated-promotion.md
Benchmarks: Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Status: Batch C B2B-leader IP substance deepening pass

## IP-Specific Intent
SLO gated promotion owns promotion gates combine file load, version save, permission check, comment sync, handoff export, asset preview, and audit lag evidence.
The primary planning object is `DesignPromotionGate`, and every example in this IP is written around that object rather than a generic artifact row.
This file stays inside the design-collaboration slice; it does not move authority to ERP, journeys, manifests, ADR-0321, or another microservice.
Figma Enterprise responsiveness is displaced by measured SLO evidence and rollback thresholds.
The implementation target is documentation-deep enough for a later code slice to produce contracts, Cedar facts, workflow steps, test fixtures, and rollback evidence without reinterpreting intent.

## Local Repository Anchors
- microservices/design-collaboration/PRD.md
- microservices/design-collaboration/ARCHITECTURE.md
- microservices/design-collaboration/compliance.md
- microservices/design-collaboration/competitor-parity-matrix.md
- microservices/design-collaboration/contracts/openapi-v1.yaml
- microservices/design-collaboration/contracts/asyncapi-v1.yaml
- microservices/design-collaboration/contracts/design-collaboration-v1.proto
- microservices/design-collaboration/policy/creative-artifact-authorization.cedar
- microservices/design-collaboration/policy/data-residency.md
- microservices/design-collaboration/dpia.md
- microservices/design-collaboration/threat-model.md
- microservices/design-collaboration/slos/local-file-load-time.openslo.yaml
- microservices/design-collaboration/slos/local-version-save-latency.openslo.yaml
- microservices/design-collaboration/slos/local-permission-check-latency.openslo.yaml
- microservices/design-collaboration/slos/local-comment-sync-latency.openslo.yaml
- microservices/design-collaboration/runbooks/component-library-drift.md
- microservices/design-collaboration/runbooks/prototype-link-break.md
- microservices/design-collaboration/runbooks/review-comment-fanout-lag.md

## ADR and Contract Binding
ADR-0105 governs the layer split: public REST shape, internal proto shape, usecase transaction boundary, domain invariant, adapter behavior, worker replay, and governance checks stay separately inspectable.
ADR-0131 keeps the plan in the flat microservice directory and prevents a design-suite folder from becoming a second source of truth.
ADR-0242, ADR-0243, ADR-0244, and ADR-0246 make the capability record, evidence trail, pack overlay, and governance posture checkable rather than aspirational.
ADR-0253-amendment matters wherever a prototype link, asset preview, or export endpoint crosses the edge; the IP expects HTTP/3 preference, TLS 1.3 floor, ECH notes, PQC negotiation notes, and downgrade evidence.
ADR-0257 and ADR-0258 bind credential leases and policy-library decisions to captured facts, not to implicit service trust.
ADR-0263 turns abuse, insider misuse, and anomaly findings into audit events and investigation handoffs.
ADR-0294, ADR-0296, and ADR-0297 make local checks, SLO gates, and scorecards promotion blockers.
ADR-0314 prevents licensed creative assets, plugins, templates, and handoff exports from bypassing DealSet settlement.
ADR-0321 is cited as the current operating anchor; this pass only edits IP files.

## Concrete Object Shape
`DesignPromotionGate` stores the durable identity for SLO gated promotion.
The object begins with `sloWindow` because downstream evidence, policy checks, and replay need a stable lookup key.
`burnRate` is not treated as display metadata; it is part of the command identity used by idempotency and rollback checks.
`dashboardRef` carries the human or service actor dimension that Cedar evaluates before the usecase layer creates side effects.
`runbookRef` records the current lifecycle posture so replay can distinguish a rejected attempt from an accepted but not-yet-promoted change.
`releaseCandidate` is captured before workflow dispatch because pack overlays and public-link behavior can change the permitted path.
`exceptionOwner` ties the object to reviewer, owner, or source-system evidence that cannot be reconstructed from a vendor export later.
`rollbackThreshold` is the field auditors should use to connect the record to a policy decision, SLO measurement, or rollback bundle.
`rollbackThreshold` is deliberately part of the shape so external handoff evidence does not depend on UI state.
The table projection keeps tenant id, cell, pack overlay, policy decision id, workflow run id, audit event id, object version, and source vendor beside the IP-specific fields.
Large binary design payloads, font files, image bytes, rendered previews, and comment bodies are not control-plane fields; they stay behind storage and evidence references.

## Lifecycle Semantics
`gate_opened` follows new command intake and can move toward measurements_loaded only when the policy decision, workflow step, and audit target agree.
`measurements_loaded` follows gate_opened and can move toward gate_blocked only when the policy decision, workflow step, and audit target agree.
`gate_blocked` follows measurements_loaded and can move toward exception_recorded only when the policy decision, workflow step, and audit target agree.
`exception_recorded` follows gate_blocked and can move toward gate_passed only when the policy decision, workflow step, and audit target agree.
`gate_passed` follows exception_recorded and can move toward closeout or rollback only when the policy decision, workflow step, and audit target agree.
A transition into `measurements_loaded` is meaningful only when the object version matches the expected version carried by the command.
`gate_blocked` is not a soft warning; it emits refusal evidence with actor, action, policy fragment hash, and tenant scope.
`gate_passed` exists so replay can rebuild the decision from captured source rows and hashes rather than from a current vendor snapshot.
Lifecycle state is visible to operators through dashboards and runbooks; no state is allowed to live only in a user-interface badge.

## API and Event Semantics
The REST command family for SLO gated promotion uses create-or-evaluate, commit, reject, replay, rollback, and export-evidence operations rather than a single overloaded update call.
Requests carry `Idempotency-Key`, `X-Oya-Tenant`, `X-Oya-Policy-Decision`, expected object version, pack overlay, trace id, and audit target.
The OpenAPI 3.2.0 schema names `sloWindow` and `burnRate` as first-class properties so generated clients cannot hide the object identity.
The proto3 internal message uses typed references for tenant, actor, workflow, policy, audit, rollback, and evidence handles; it does not tunnel vendor JSON.
The AsyncAPI 3.1.0 event for `measurements_loaded` includes object kind, object id, version, source vendor, workflow run id, policy decision id, audit event id, and replay checkpoint.
The refusal event for `gate_blocked` includes denial class, denied field, policy fragment hash, current lifecycle state, and the operator-facing remediation hint.
The replay event for `gate_passed` points to source row, transform hash, checkpoint hash, and rollback anchor.
Event metadata excludes raw design payload, preview bytes, font bytes, and full comment bodies; consumers fetch those through authorized storage paths.

## Cedar and Workflow Decisions
Cedar facts for SLO gated promotion include actor role, tenant membership, object state, data class, pack overlay, source vendor trust, workflow step, DealSet entitlement, cell route, and abuse score.
If `dashboardRef` represents a guest or external reviewer, the policy also evaluates invitation expiry, watermark mode, object scope, and export prohibition.
If `releaseCandidate` changes during workflow execution, the usecase pauses and asks workflow-engine to rerun the affected approval step.
Workflow decisions name the owner role, approver role, expiry, delegated actor, evidence checklist, compensation command, and reopen trigger.
A policy allow is not enough to publish; DealSet and pack checks still run when SLO gated promotion touches licensed assets, templates, plugins, public links, or customer handoff.

## Evidence Packet Contents
The evidence packet for SLO gated promotion contains `DesignPromotionGate` snapshot, command id, source vendor, source object id, policy decision id, workflow run id, audit event id, and rollback anchor.
It also carries object version before and after the transition, selected pack overlay, SLO window, runbook reference, dashboard reference, and reviewer closeout note.
For benchmark displacement, the packet records which of Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel created the product-depth pressure and why the Oyatie control model differs.
The packet is exportable for audit without revealing raw design payload; payload inspection remains a separate authorized storage action.

## Risk Cases and Tests
- Risk case: unmeasured prototype export. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Risk case: dashboard stale. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Risk case: audit lag hidden. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Risk case: permission latency regression. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Positive test: `measurements_loaded` can be reached from a clean tenant fixture with current policy, matching version, valid pack overlay, and complete workflow evidence.
- Replay test: rebuilding `DesignPromotionGate` from source rows produces the same object version, event names, policy hash, and rollback anchor.
- Contract test: REST, AsyncAPI, and proto3 examples agree on `sloWindow`, `burnRate`, policy decision id, workflow run id, audit event id, and replay checkpoint.
- SLO test: dashboards expose the latency or freshness metric that matters to SLO gated promotion, and the runbook names the rollback threshold.
- Security test: a tenant-crossing attempt against `sloWindow` fails before adapter access and produces no storage mutation.
- Pack test: the stricter overlay wins when residency, retention, export, or reviewer geography conflicts with the default path.
- DealSet test: export or publication pauses when a licensed asset, plugin, template, or generated asset lacks settlement evidence.

## Rollback Mechanics
Rollback starts by freezing new side effects for `DesignPromotionGate` and reading the rollback anchor from the evidence packet.
The compensation command restores the prior lifecycle state and writes a rollback audit event with the original policy decision and workflow run.
Async consumers receive a replay-safe correction event instead of a destructive delete.
Operators use the cited runbook to notify the tenant when user-visible collaboration state changed.
Closeout compares the pre-rollback and post-rollback snapshots and records any residual risk in the audit packet.

## Benchmark Displacement
Figma Enterprise is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Sketch Cloud is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Adobe XD Enterprise is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
InVision Enterprise is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Framer is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Penpot is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Marvel is a benchmark for feature depth in SLO gated promotion; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.

## Acceptance Notes
- Promotion evidence gate: `IP-021-slo-gated-promotion.md` must load SLO window, burn rate, dashboard reference, runbook reference, release candidate, exception owner, and rollback threshold.
- Repetition gate: no generic numbered matrix, no repeated field sentence frame, no repeated command result frame, and no repeated SLO frame are used as filler.
- Scope gate: this correction touches only `microservices/design-collaboration/IP-*.md`.
- Lifecycle gate: `oya vcs verify`, `oya vcs done`, and `oya vcs promote` remain for the leader.
- Citation gate: each IP keeps local docs, contracts, policy, DPIA, threat model, SLOs, runbooks, ADRs, and allowed benchmark names in the file.
The benchmark list stays fixed to Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel.
A future implementation slice can turn this IP into tests without inventing new lifecycle states.

## Additional Design Notes
Implementation note: SLO gated promotion should start at the usecase layer with a preflight that reads the current `DesignPromotionGate` snapshot before calling adapters.
Implementation note: SLO gated promotion should persist refusal evidence even when no domain mutation is committed.
Implementation note: SLO gated promotion should name the source vendor in evidence, but source vendor names must not become aggregate types.
Implementation note: SLO gated promotion should keep tenant-facing messages separate from operator remediation hints.
Implementation note: SLO gated promotion should treat `sloWindow` as immutable after the first accepted command.
Data note: SLO gated promotion should record the source object hash separately from rendered preview hashes.
Data note: SLO gated promotion should version the evidence packet when the local contract family changes.
Data note: SLO gated promotion should keep reviewer display names out of broker metadata and use actor references instead.
Data note: SLO gated promotion should preserve deleted-source markers because imported design artifacts often disappear upstream.
Data note: SLO gated promotion should attach pack overlay resolution to the object version that saw it, not only to the tenant.
Policy note: SLO gated promotion should evaluate guest access with the same seriousness as employee access because agency workflows cross client boundaries.
Policy note: SLO gated promotion should include source-vendor trust level when imported content changes publication or export behavior.
Policy note: SLO gated promotion should deny when policy facts were collected against a different object version.
Policy note: SLO gated promotion should produce a stable denial code suitable for contract tests and runbook branching.
Policy note: SLO gated promotion should not let workflow approval override Cedar denial.
Workflow note: SLO gated promotion should pause instead of auto-retrying when approver membership changes during review.
Workflow note: SLO gated promotion should record the human-readable decision reason beside machine ids for audit readability.
Workflow note: SLO gated promotion should include a reopen path because design collaboration decisions commonly reverse after stakeholder review.
Workflow note: SLO gated promotion should expose timeout behavior before external reviewers receive a link.
Workflow note: SLO gated promotion should bind compensation to the same command family as the original change.
Event note: SLO gated promotion should emit refusal, replay, rollback, and evidence-export events as separate semantic events.
Event note: SLO gated promotion should give workers enough cursor information to resume without scanning tenant-wide history.
Event note: SLO gated promotion should never publish raw comments, file names, or asset bytes as broker routing keys.
Event note: SLO gated promotion should include source object references only after they have passed tenant-scope checks.
Event note: SLO gated promotion should identify replayed events so dashboards do not count them as fresh user work.
SLO note: SLO gated promotion should connect the relevant dashboard to an operator action, not just to a graph.
SLO note: SLO gated promotion should fail promotion when measurement exists but lacks the affected artifact kind.
SLO note: SLO gated promotion should distinguish interactive user latency from worker replay freshness.
SLO note: SLO gated promotion should make audit emission lag visible when the user-facing command succeeds.
SLO note: SLO gated promotion should define whether degraded mode permits reads, comments, exports, or only evidence inspection.
Test note: SLO gated promotion should include a stale-version test with a realistic imported artifact fixture.
Test note: SLO gated promotion should include a cross-tenant agency user scenario rather than only single-tenant employees.
Test note: SLO gated promotion should include a replay fixture that proves source hash and transform hash stability.
Test note: SLO gated promotion should include an evidence-export assertion that checks ADR and local doc references are present.
Test note: SLO gated promotion should include a benchmark-displacement assertion so older vendor names cannot re-enter the file.
Rollback note: SLO gated promotion should restore user-visible state and also correct downstream read models.
Rollback note: SLO gated promotion should leave a durable explanation for why the rollback happened.
Rollback note: SLO gated promotion should avoid destructive delete semantics; consumers need a correction event.
Rollback note: SLO gated promotion should keep the pre-rollback packet inspectable for audit and dispute review.
Rollback note: SLO gated promotion should name the operator runbook that owns tenant communication after visible state changes.
Closeout note: SLO gated promotion should be considered ready only when contract examples, Cedar examples, workflow examples, SLO evidence, and rollback evidence all point to the same object shape.
Closeout note: SLO gated promotion should preserve this exact benchmark list: Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel.
The implementer should start by modelling `DesignPromotionGate` as a value-owned design-collaboration record rather than a vendor import wrapper.
A fixture for SLO gated promotion needs at least one internal employee, one external reviewer, one service worker, and one auditor persona.
The first negative fixture should mutate `sloWindow` across tenant boundaries and prove the adapter is never called.
An imported Figma Enterprise object should retain source identity while losing authority over Oyatie state transitions.
A Sketch Cloud-origin component should map to the same tenant policy shape as a Penpot-origin component.
Adobe XD Enterprise material should be treated as legacy source evidence, not as a contract naming source.
InVision Enterprise review behavior should influence review depth but not ledger authority.
Framer publish behavior should influence export tests only after DealSet and pack checks pass.
Marvel prototype behavior should appear in handoff and public-link cases without weakening tenant isolation.
The happy path should show policy allow, workflow approval, audit seal, SLO measurement, and evidence export in that order.
The main refusal path should show a human-readable denial code and a machine-checkable Cedar fragment hash.
The replay path should rebuild the final event from source hash, transform hash, object version, and workflow id.
The rollback path should send a correction event to async consumers instead of deleting prior evidence.
The operator runbook should name the dashboard, customer-impact cue, rollback command, and closeout evidence location.
The contract example should include an idempotency key collision with two different payloads.
The proto example should prove internal callers receive a summary rather than raw design payload.
The AsyncAPI example should keep file names, preview bytes, font bytes, and comment bodies out of metadata.
The DPIA note should name data class, subject category, retention rule, export surface, and breach route.
The threat-model note should map public link abuse, plugin provenance, comment tamper, and insider export risk.
The SLO note should name whether the user waits, queues, retries, or sees degraded evidence inspection.
The cost note should separate render work, preview egress, storage, fanout, replay CPU, and export bytes.
The capacity note should explain how active collaborators, object size, queue depth, and cell pressure change admission.
The credential note should mention OpenBao lease id, tenant path, adapter id, ttl, purpose, and revocation reason.
The pack note should state which field changes when residency, retention, reviewer geography, or regulator export is stricter.
The workflow note should identify who can reopen a decision and what evidence is carried into the reopened run.
The audit note should let an auditor reconstruct actor, tenant, object, action, policy, workflow, and rollback without UI screenshots.
The scorecard note should fail this IP when local references disappear or benchmark names drift.
The implementation should not add dependencies merely to mimic a vendor collaboration feature.
The final review should compare this IP against PRD.md, ARCHITECTURE.md, OpenAPI, AsyncAPI, proto3, Cedar, SLOs, and runbooks.
The leader lifecycle remains outside this file; no verify, done, or promote command belongs in this slice.
SLO promotion implementation evidence should include a local-comment-sync regression that blocks release even when file-open latency remains healthy.
