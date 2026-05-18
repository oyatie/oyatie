---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-015-replay-engine-bootstrap
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: replay-engine bootstrap (kernel + domain + adapter-s3 + usecase + worker)

## Intent

Bootstrap the replay-engine BC: kernel + domain (deterministic-seed validation + divergence math) + adapter-s3 (replay-trace fetch + per-subject DEK shred) + usecase (sampling + replay orchestrator) + worker (continuous replay-sample worker).

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-replay-engine-{kernel,domain,adapter-s3,usecase,worker}/`.

## Concrete File Targets

For each crate: `Cargo.toml + src/lib.rs + src/<modules>.rs + catalog/<crate>.yaml`.

### replay-engine-kernel

- `entities.rs`: `ReplaySample`, `DivergenceReport`, `SubjectDek`.
- `ports.rs`: `ReplaySampler`, `DivergenceDetector`, `SubjectDekStore`.
- `errors.rs`.

### replay-engine-domain

- `divergence.rs`: ms-tolerance arithmetic; ≤ 100ms per ADR-0024.
- `deterministic_seed.rs`: seed-presence validation; replay-equivalence assertion.

### replay-engine-adapter-s3

- `trace_fetcher.rs`: S3 GET + chain verify.
- `dek_envelope.rs`: KMS unwrap; shred via KMS delete.
- `shred_audit.rs`: emit `EvalSubjectShred` event.

### replay-engine-usecase

- `sampler.rs`: cohort sampling per `capabilities/replay-execute.yaml`.
- `replay_orchestrator.rs`: dispatch + collect + emit divergence.
- `dsr_cascade.rs`: consume `EraseSubjectRequested`, dispatch shred.

### replay-engine-worker

- `continuous_sampler.rs`: continuous sampling loop.
- `leader_election.rs`: HA.

## Test Plan

Kernel + domain: per-class minima; adapter: integration tests with minio + LocalStack KMS; usecase: scenario tests; worker: continuous-loop behavior + HA.
