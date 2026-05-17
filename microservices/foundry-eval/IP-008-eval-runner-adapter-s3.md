---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-008-eval-runner-adapter-s3
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

# IP-008: oya-foundry-eval-eval-runner-adapter-s3

## Intent

S3-backend-qualified adapter (per ADR-0105 Amendment 3): golden-output GET (Cosign-verified) + eval-run PUT + replay-trace GET (per-subject DEK unwrap via KMS).

## ChangeSet boundary

`microservices/foundry-eval/src/crates/oya-foundry-eval-eval-runner-adapter-s3/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-adapter-s3/Cargo.toml` | create — `aws-sdk-s3`, `aws-sdk-kms`, `sigstore` |
| `src/crates/oya-foundry-eval-eval-runner-adapter-s3/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-adapter-s3/src/golden_reader.rs` | create — Cosign-verified GET |
| `src/crates/oya-foundry-eval-eval-runner-adapter-s3/src/eval_run_store.rs` | create — PUT |
| `src/crates/oya-foundry-eval-eval-runner-adapter-s3/src/replay_trace_fetcher.rs` | create — DEK-unwrap GET |
| `src/crates/oya-foundry-eval-eval-runner-adapter-s3/src/dek_envelope.rs` | create — KMS unwrap helper |
| `catalog/oya-foundry-eval-eval-runner-adapter-s3.yaml` | create |

## Test Plan

85% line: minio-backed integration tests; LocalStack KMS for DEK; fixture-signed Cosign artifacts.
