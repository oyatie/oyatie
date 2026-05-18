---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-011-jobs-handoff-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + axis-ats
acceptance_lanes: [cargo-check, cargo-nextest, oya-gate-jobs-handoff-contract]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: jobs-handoff BC end-to-end (clean boundary to Tier-G ATS µservice; ADR-NET-0004)

## Intent

Author the full `jobs-handoff` BC per ADR-NET-0004:

- **Workflow-mediated handoff to ATS µservice.** Per the
  workflow-vs-direct-gRPC rubric (`docs/standards/workflow-vs-direct-grpc-rubric.md`)
  this handoff matches the canonical Workflow case: multi-µservice handoff
  with ack semantics, ATS may be unavailable for hours, audit-chain causal
  ordering required. The handoff routes via the workflow-engine event-bus;
  `network` does NOT open a direct gRPC channel to ATS at runtime.
- ATS µservice activation is gated by ADR-0132 forward policy. The ATS
  µservice is `tracked-for-tier-g-onboarding`: it does NOT exist as a
  shipping µservice at the time this IP is authored. ATS µservice
  activates when the first ATS-tier tenant signals onboarding, per
  ADR-0132's forward-policy substrate. Until then, the workflow-engine
  egress carries the `oya.network.jobposting.v1.published` events into a
  bounded queue that ATS will consume once the µservice is deployed.
- The prior carrier-exemption framing (feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)
  + ADR-0140 (retired per ADR-0145)) is retired per ADR-0145. Under ADR-0145 direct sibling-µservice
  gRPC is permitted in general; this BC continues to route through
  workflow-engine for the rubric-driven reasons above (durable execution +
  saga semantics), NOT because of a universal-mediator rule.
- Event-based handoff via the workflow-engine event bus:
  `oya.network.jobposting.v1.published` + `oya.network.jobapplication.v1.filed`.
  workflow-engine routes these to ATS; ATS subscribes via the same bus.
- The `-adapter-ats-bridge` adapter is renamed conceptually to `-adapter-workflow-engine-jobs-bridge`
  in any future relayer change; for now the adapter MUST publish to
  workflow-engine and MUST NOT open a direct gRPC channel to ATS at runtime.
- ATS µservice owns pipeline state; `network` owns posting + referral metadata only.
- Idempotent + replay-safe via ULID event_id; ATS ack returns through workflow-engine
  via `oya.ats.v1.application-accepted` (the bridge consumes the ack from
  workflow-engine, not from ATS directly).
- Backfill-replay per `backfill-replay.md` §"Jobs-Handoff Replay" — replays
  flow through workflow-engine; the resume cursor is a workflow-engine offset.

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait JobPostingRepository: Send + Sync {
    async fn publish(&self, posting: JobPostingNew) -> Result<JobPosting, JobError>;
    async fn ack_ats_handoff(&self, posting_id: &JobPostingId, status: AtsHandoffStatus) -> Result<(), JobError>;
}

#[async_trait]
pub trait ATSHandoffBridge: Send + Sync {
    async fn emit_published(&self, posting: &JobPosting, contract_version: &str) -> Result<(), BridgeError>;
    async fn emit_application_filed(&self, application: &JobApplication, contract_version: &str) -> Result<(), BridgeError>;
    async fn handle_resume_ready(&self, from_event_id: &EventId, contract_version: &str) -> Result<(), BridgeError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-jobs-handoff-kernel
cargo nextest run -p oya-network-jobs-handoff-adapter-ats-bridge
cargo run -p oya-dev-cli -- gate validate jobs-handoff-contract --microservice network
```

## Test Plan

- Job-posting publish: emits `oya.network.jobposting.v1.published` with `contract_version: v1`; ATS µservice acks via `oya.ats.v1.application-accepted`; `network_job_postings.ats_handoff_status` updates.
- Application filed: emits `oya.network.jobapplication.v1.filed`; ATS µservice receives + acks.
- Backfill-replay: ATS sends `ATSResumeReady{from_event_id, contract_version}` after outage; worker re-emits + ATS dedupes on event_id.
- Dual-version-window: deploy v2 contract while v1 still active; both versions emit successfully for 6mo.
- ATS-bridge degraded (FM-19): queue holds in Valkey Streams (Redis wire-compat); per `runbooks/jobs-handoff-ats-failure.md` recovers on ATS restore.

## Halt Conditions

- ATS µservice not yet deployed in target pack — stub the ack with mock; gate the production rollout on ATS µservice availability.
- Contract-version drift detected in CI — fix before merging.

## Next IP

[`IP-012-mentions-hashtags-trending-notifications-bcs.md`](IP-012-mentions-hashtags-trending-notifications-bcs.md)

## References

- ADR-NET-0004 (jobs-handoff to ATS).
- ADR-0145 (inter-microservice communication reform — supersedes the
  prior carrier-exemption framing; this handoff routes through workflow-engine
  per the workflow-vs-direct-gRPC rubric, not because of a universal-mediator
  rule).
- ADR-0140 (retired per ADR-0145; kept for back-reference only).
- `docs/standards/workflow-vs-direct-grpc-rubric.md` — why this handoff
  matches the canonical Workflow case.
- `microservices/network/backfill-replay.md` §"Jobs-Handoff Replay".
- `microservices/network/runbooks/jobs-handoff-ats-failure.md`.
- `iac/helm/network/templates/networkpolicy.yaml` (egress to workflow-engine
  namespace; ATS µservice activates per ADR-0132 forward policy when first
  ATS-tier tenant signals onboarding).
