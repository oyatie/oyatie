# IP-028 Design Collaboration design-token promotion gate

Service: design-collaboration
ChangeSet scope: microservices/design-collaboration/IP-028-design-token-promotion-gate.md
Benchmarks: Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Status: Batch C B2B-leader IP substance deepening pass

## IP-Specific Intent
design-token promotion gate owns promotion decisions cover color, typography, spacing, motion, accessibility, semantic slots, and consumer compatibility.
The primary planning object is `DesignTokenPromotion`, and every example in this IP is written around that object rather than a generic artifact row.
This file stays inside the design-collaboration slice; it does not move authority to ERP, journeys, manifests, ADR-0321, or another microservice.
Figma Enterprise variables, Penpot tokens, and Framer styles are displaced by staged token promotion.
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
`DesignTokenPromotion` stores the durable identity for design-token promotion gate.
The object begins with `tokenFamily` because downstream evidence, policy checks, and replay need a stable lookup key.
`semanticSlot` is not treated as display metadata; it is part of the command identity used by idempotency and rollback checks.
`oldValue` carries the human or service actor dimension that Cedar evaluates before the usecase layer creates side effects.
`newValue` records the current lifecycle posture so replay can distinguish a rejected attempt from an accepted but not-yet-promoted change.
`contrastEvidence` is captured before workflow dispatch because pack overlays and public-link behavior can change the permitted path.
`motionPolicy` ties the object to reviewer, owner, or source-system evidence that cannot be reconstructed from a vendor export later.
`consumerAckSet` is the field auditors should use to connect the record to a policy decision, SLO measurement, or rollback bundle.
`consumerAckSet` is deliberately part of the shape so external handoff evidence does not depend on UI state.
The table projection keeps tenant id, cell, pack overlay, policy decision id, workflow run id, audit event id, object version, and source vendor beside the IP-specific fields.
Large binary design payloads, font files, image bytes, rendered previews, and comment bodies are not control-plane fields; they stay behind storage and evidence references.

## Lifecycle Semantics
`token_proposed` follows new command intake and can move toward contrast_checked only when the policy decision, workflow step, and audit target agree.
`contrast_checked` follows token_proposed and can move toward consumers_acknowledged only when the policy decision, workflow step, and audit target agree.
`consumers_acknowledged` follows contrast_checked and can move toward token_promoted only when the policy decision, workflow step, and audit target agree.
`token_promoted` follows consumers_acknowledged and can move toward token_reverted only when the policy decision, workflow step, and audit target agree.
`token_reverted` follows token_promoted and can move toward closeout or rollback only when the policy decision, workflow step, and audit target agree.
A transition into `contrast_checked` is meaningful only when the object version matches the expected version carried by the command.
`consumers_acknowledged` is not a soft warning; it emits refusal evidence with actor, action, policy fragment hash, and tenant scope.
`token_reverted` exists so replay can rebuild the decision from captured source rows and hashes rather than from a current vendor snapshot.
Lifecycle state is visible to operators through dashboards and runbooks; no state is allowed to live only in a user-interface badge.

## API and Event Semantics
The REST command family for design-token promotion gate uses create-or-evaluate, commit, reject, replay, rollback, and export-evidence operations rather than a single overloaded update call.
Requests carry `Idempotency-Key`, `X-Oya-Tenant`, `X-Oya-Policy-Decision`, expected object version, pack overlay, trace id, and audit target.
The OpenAPI 3.2.0 schema names `tokenFamily` and `semanticSlot` as first-class properties so generated clients cannot hide the object identity.
The proto3 internal message uses typed references for tenant, actor, workflow, policy, audit, rollback, and evidence handles; it does not tunnel vendor JSON.
The AsyncAPI 3.1.0 event for `contrast_checked` includes object kind, object id, version, source vendor, workflow run id, policy decision id, audit event id, and replay checkpoint.
The refusal event for `consumers_acknowledged` includes denial class, denied field, policy fragment hash, current lifecycle state, and the operator-facing remediation hint.
The replay event for `token_reverted` points to source row, transform hash, checkpoint hash, and rollback anchor.
Event metadata excludes raw design payload, preview bytes, font bytes, and full comment bodies; consumers fetch those through authorized storage paths.

## Cedar and Workflow Decisions
Cedar facts for design-token promotion gate include actor role, tenant membership, object state, data class, pack overlay, source vendor trust, workflow step, DealSet entitlement, cell route, and abuse score.
If `oldValue` represents a guest or external reviewer, the policy also evaluates invitation expiry, watermark mode, object scope, and export prohibition.
If `contrastEvidence` changes during workflow execution, the usecase pauses and asks workflow-engine to rerun the affected approval step.
Workflow decisions name the owner role, approver role, expiry, delegated actor, evidence checklist, compensation command, and reopen trigger.
A policy allow is not enough to publish; DealSet and pack checks still run when design-token promotion gate touches licensed assets, templates, plugins, public links, or customer handoff.

## Evidence Packet Contents
The evidence packet for design-token promotion gate contains `DesignTokenPromotion` snapshot, command id, source vendor, source object id, policy decision id, workflow run id, audit event id, and rollback anchor.
It also carries object version before and after the transition, selected pack overlay, SLO window, runbook reference, dashboard reference, and reviewer closeout note.
For benchmark displacement, the packet records which of Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel created the product-depth pressure and why the Oyatie control model differs.
The packet is exportable for audit without revealing raw design payload; payload inspection remains a separate authorized storage action.

## Risk Cases and Tests
- Risk case: contrast regression. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Risk case: motion reduction ignored. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Risk case: semantic slot reused incorrectly. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Risk case: consumer ack missing. Test with a tenant fixture, a denied actor or stale version, and an assertion that the refusal event links to the audit packet.
- Positive test: `contrast_checked` can be reached from a clean tenant fixture with current policy, matching version, valid pack overlay, and complete workflow evidence.
- Replay test: rebuilding `DesignTokenPromotion` from source rows produces the same object version, event names, policy hash, and rollback anchor.
- Contract test: REST, AsyncAPI, and proto3 examples agree on `tokenFamily`, `semanticSlot`, policy decision id, workflow run id, audit event id, and replay checkpoint.
- SLO test: dashboards expose the latency or freshness metric that matters to design-token promotion gate, and the runbook names the rollback threshold.
- Security test: a tenant-crossing attempt against `tokenFamily` fails before adapter access and produces no storage mutation.
- Pack test: the stricter overlay wins when residency, retention, export, or reviewer geography conflicts with the default path.
- DealSet test: export or publication pauses when a licensed asset, plugin, template, or generated asset lacks settlement evidence.

## Rollback Mechanics
Rollback starts by freezing new side effects for `DesignTokenPromotion` and reading the rollback anchor from the evidence packet.
The compensation command restores the prior lifecycle state and writes a rollback audit event with the original policy decision and workflow run.
Async consumers receive a replay-safe correction event instead of a destructive delete.
Operators use the cited runbook to notify the tenant when user-visible collaboration state changed.
Closeout compares the pre-rollback and post-rollback snapshots and records any residual risk in the audit packet.

## Benchmark Displacement
Figma Enterprise is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Sketch Cloud is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Adobe XD Enterprise is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
InVision Enterprise is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Framer is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Penpot is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.
Marvel is a benchmark for feature depth in design-token promotion gate; it is not accepted as an authority for tenant isolation, Cedar authorization, pack overlays, audit evidence, SLO gates, or rollback semantics.

## Acceptance Notes
- Token evidence gate: `IP-028-design-token-promotion-gate.md` must bind token family, semantic slot, old value, new value, contrast evidence, motion policy, and consumer acknowledgement set.
- Repetition gate: no generic numbered matrix, no repeated field sentence frame, no repeated command result frame, and no repeated SLO frame are used as filler.
- Scope gate: this correction touches only `microservices/design-collaboration/IP-*.md`.
- Lifecycle gate: `oya vcs verify`, `oya vcs done`, and `oya vcs promote` remain for the leader.
- Citation gate: each IP keeps local docs, contracts, policy, DPIA, threat model, SLOs, runbooks, ADRs, and allowed benchmark names in the file.
The benchmark list stays fixed to Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel.
A future implementation slice can turn this IP into tests without inventing new lifecycle states.

## Additional Design Notes
Implementation note: design-token promotion gate should start at the usecase layer with a preflight that reads the current `DesignTokenPromotion` snapshot before calling adapters.
Implementation note: design-token promotion gate should persist refusal evidence even when no domain mutation is committed.
Implementation note: design-token promotion gate should name the source vendor in evidence, but source vendor names must not become aggregate types.
Implementation note: design-token promotion gate should keep tenant-facing messages separate from operator remediation hints.
Implementation note: design-token promotion gate should treat `tokenFamily` as immutable after the first accepted command.
Data note: design-token promotion gate should record the source object hash separately from rendered preview hashes.
Data note: design-token promotion gate should version the evidence packet when the local contract family changes.
Data note: design-token promotion gate should keep reviewer display names out of broker metadata and use actor references instead.
Data note: design-token promotion gate should preserve deleted-source markers because imported design artifacts often disappear upstream.
Data note: design-token promotion gate should attach pack overlay resolution to the object version that saw it, not only to the tenant.
Policy note: design-token promotion gate should evaluate guest access with the same seriousness as employee access because agency workflows cross client boundaries.
Policy note: design-token promotion gate should include source-vendor trust level when imported content changes publication or export behavior.
Policy note: design-token promotion gate should deny when policy facts were collected against a different object version.
Policy note: design-token promotion gate should produce a stable denial code suitable for contract tests and runbook branching.
Policy note: design-token promotion gate should not let workflow approval override Cedar denial.
Workflow note: design-token promotion gate should pause instead of auto-retrying when approver membership changes during review.
Workflow note: design-token promotion gate should record the human-readable decision reason beside machine ids for audit readability.
Workflow note: design-token promotion gate should include a reopen path because design collaboration decisions commonly reverse after stakeholder review.
Workflow note: design-token promotion gate should expose timeout behavior before external reviewers receive a link.
Workflow note: design-token promotion gate should bind compensation to the same command family as the original change.
Event note: design-token promotion gate should emit refusal, replay, rollback, and evidence-export events as separate semantic events.
Event note: design-token promotion gate should give workers enough cursor information to resume without scanning tenant-wide history.
Event note: design-token promotion gate should never publish raw comments, file names, or asset bytes as broker routing keys.
Event note: design-token promotion gate should include source object references only after they have passed tenant-scope checks.
Event note: design-token promotion gate should identify replayed events so dashboards do not count them as fresh user work.
SLO note: design-token promotion gate should connect the relevant dashboard to an operator action, not just to a graph.
SLO note: design-token promotion gate should fail promotion when measurement exists but lacks the affected artifact kind.
SLO note: design-token promotion gate should distinguish interactive user latency from worker replay freshness.
SLO note: design-token promotion gate should make audit emission lag visible when the user-facing command succeeds.
SLO note: design-token promotion gate should define whether degraded mode permits reads, comments, exports, or only evidence inspection.
Test note: design-token promotion gate should include a stale-version test with a realistic imported artifact fixture.
Test note: design-token promotion gate should include a cross-tenant agency user scenario rather than only single-tenant employees.
Test note: design-token promotion gate should include a replay fixture that proves source hash and transform hash stability.
Test note: design-token promotion gate should include an evidence-export assertion that checks ADR and local doc references are present.
Test note: design-token promotion gate should include a benchmark-displacement assertion so older vendor names cannot re-enter the file.
Rollback note: design-token promotion gate should restore user-visible state and also correct downstream read models.
Rollback note: design-token promotion gate should leave a durable explanation for why the rollback happened.
Rollback note: design-token promotion gate should avoid destructive delete semantics; consumers need a correction event.
Rollback note: design-token promotion gate should keep the pre-rollback packet inspectable for audit and dispute review.
Rollback note: design-token promotion gate should name the operator runbook that owns tenant communication after visible state changes.
Closeout note: design-token promotion gate should be considered ready only when contract examples, Cedar examples, workflow examples, SLO evidence, and rollback evidence all point to the same object shape.
Closeout note: design-token promotion gate should preserve this exact benchmark list: Figma Enterprise, Sketch Cloud, Adobe XD Enterprise, InVision Enterprise, Framer, Penpot, Marvel.
The implementer should start by modelling `DesignTokenPromotion` as a value-owned design-collaboration record rather than a vendor import wrapper.
A fixture for design-token promotion gate needs at least one internal employee, one external reviewer, one service worker, and one auditor persona.
The first negative fixture should mutate `tokenFamily` across tenant boundaries and prove the adapter is never called.
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
Design-token implementation evidence should include a contrast regression that blocks promotion while preserving the previous semantic slot and rollback bundle.
