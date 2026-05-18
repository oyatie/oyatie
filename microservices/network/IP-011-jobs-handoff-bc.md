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

- gRPC contract to ATS µservice with `contract_version: v1`; dual-version-window discipline.
- Event-based handoff via Workflow event bus: `oya.network.jobposting.v1.published` + `oya.network.jobapplication.v1.filed`.
- ATS µservice owns pipeline state; `network` owns posting + referral metadata only.
- Idempotent + replay-safe via ULID event_id; ATS ack via `oya.ats.v1.application-accepted`.
- Backfill-replay per `backfill-replay.md` §"Jobs-Handoff Replay".

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
- ATS-bridge degraded (FM-19): queue holds in Redis Streams; per `runbooks/jobs-handoff-ats-failure.md` recovers on ATS restore.

## Halt Conditions

- ATS µservice not yet deployed in target pack — stub the ack with mock; gate the production rollout on ATS µservice availability.
- Contract-version drift detected in CI — fix before merging.

## Next IP

[`IP-012-mentions-hashtags-trending-notifications-bcs.md`](IP-012-mentions-hashtags-trending-notifications-bcs.md)

## References

- ADR-NET-0004 (jobs-handoff to ATS).
- `microservices/network/backfill-replay.md` §"Jobs-Handoff Replay".
- `microservices/network/runbooks/jobs-handoff-ats-failure.md`.
