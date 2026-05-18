---
doc_class: SDKPlan
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: axis-foundry-evidence + axis-developer-experience
related_artifacts:
  - microservices/foundry-evidence/PRD.md
  - microservices/foundry-evidence/contracts/openapi/foundry-evidence.yaml
  - microservices/foundry-evidence/contracts/proto/foundry-evidence.proto
  - microservices/foundry-evidence/contracts/asyncapi/foundry-evidence-events.yaml
doc_status: published
---

# foundry-evidence — SDK plan

## Scope

The `oya-foundry-evidence-sdk` crate is the canonical client for every Foundry µservice that records evidence (foundry-runtime, foundry-guardrails, foundry-supervisor, foundry-eval) and for internal forensic users that query packs or request regulator-exports.

## Languages

| Phase | Language | Status |
|---|---|---|
| M01 (launch) | Rust (idiomatic; preferred for in-cluster Foundry µservices) | required-at-exit-gate |
| M01+30d | TypeScript (for `application` µservice portal frontends; tenant-side SDKs) | required-at-M02 |
| M01+60d | Python (for foundry-eval data-science workflows + tenant-side notebooks) | required-at-M02 |
| Future | Go (selected on demand) | scheduled for M03 |

## Surface (Rust crate `oya-foundry-evidence-sdk`)

```rust
// Layer: sdk; per ADR-0105 13-layer enum.
// Imports: own-BC api crates' read-only types ONLY.

pub mod recorder {
    pub struct CapabilityInvocationRecorderClient { /* … */ }

    impl CapabilityInvocationRecorderClient {
        /// Synchronous record_invocation; returns receipt within p99 ≤ 500 ms.
        /// SDK handles SPIFFE-bound mTLS automatically via Workload Identity.
        pub async fn record(
            &self,
            envelope: InvocationEnvelope,
        ) -> Result<RecordInvocationReceipt, RecorderError>;

        /// Status of a previously-recorded invocation.
        pub async fn status(
            &self,
            pack_id: &PackId,
        ) -> Result<RecordStatusResponse, RecorderError>;
    }
}

pub mod query {
    pub struct EvidenceQueryClient { /* … */ }

    impl EvidenceQueryClient {
        /// Streaming pack query; first-page p99 ≤ 100 ms.
        pub fn query(
            &self,
            request: EvidencePackQuery,
        ) -> impl Stream<Item = Result<EvidencePack, QueryError>>;

        /// Single pack get; plaintext gated on principal entitlements.
        pub async fn get(
            &self,
            pack_id: &PackId,
            include_plaintext: bool,
        ) -> Result<EvidencePack, QueryError>;
    }
}

pub mod regulator_export {
    pub struct RegulatorExportClient { /* … */ }

    impl RegulatorExportClient {
        /// Issue a regulator-export bundle. 2-person rule enforced server-side.
        pub async fn request(
            &self,
            request: RegulatorExportRequest,
        ) -> Result<RegulatorExportReceipt, ExportError>;

        pub async fn status(
            &self,
            bundle_id: &BundleId,
        ) -> Result<RegulatorExportBundleStatus, ExportError>;
    }
}
```

## Cross-language types

Types are generated from `contracts/proto/foundry-evidence.proto` (gRPC) + `contracts/openapi/foundry-evidence.yaml` (REST). The generation pipeline:

- Rust: `prost` + `tonic` for gRPC; `oapi-codegen` for REST types.
- TypeScript: `@bufbuild/protoc-gen-ts` for proto; `openapi-typescript` for REST types.
- Python: `betterproto` for proto; `openapi-python-client` for REST types.
- Go: `protoc-gen-go` for proto; `oapi-codegen` for REST types.

Type-generation is hermetic + reproducible; LEAN lane `sdk-codegen-reproducible` blocks commits that produce different output.

## SPIFFE integration

SDK auto-loads SPIFFE Workload API via SPIRE agent socket (default `/run/spire/sockets/agent.sock`). No static credentials. Re-issuance handled transparently; rotated certs propagated without app restart.

## Idempotency

SDK auto-generates `idempotency_key=ulid()` per record call; caller may override for explicit idempotency control. Server-side dedup window is 24 h per `(tenant_id, idempotency_key)`.

## Retry semantics

- record_invocation: SDK retries on 5xx + 429 with exponential back-off (50ms → 5s; max 5 retries within 30s). 429 with `Retry-After` honoured.
- evidence-query: SDK retries idempotently on 5xx; never retries 4xx.
- regulator-export: NO automatic retry on 5xx after server-acknowledged request (request_id is durable; caller polls status).

## Observability

SDK exposes OpenTelemetry trace + metric instrumentation:
- Spans: `foundry_evidence.record_invocation`, `foundry_evidence.query`, `foundry_evidence.regulator_export`.
- Metrics: `oya_foundry_evidence_sdk_call_duration_seconds_bucket`, `oya_foundry_evidence_sdk_retry_total`, `oya_foundry_evidence_sdk_idempotency_collision_total`.
- Trace context propagated via W3C trace-context headers.

## Versioning + compatibility

- SDK MAJOR matches server MAJOR.
- SDK MINOR bumps on additive server changes.
- SDK PATCH for bug fixes.
- LEAN lane `sdk-no-silent-regression` blocks any SDK change that removes a public symbol without ADR + sunset.

## Distribution

| Language | Channel |
|---|---|
| Rust | crates.io: `oya-foundry-evidence-sdk` (private mirror at OCI Artifact Registry until M02 public release) |
| TypeScript | npm: `@oyatie/foundry-evidence-sdk` |
| Python | PyPI: `oyatie-foundry-evidence-sdk` |
| Go | Go modules: `github.com/oyatie/foundry-evidence-sdk-go` |

## Verification

- `cargo doc --workspace --no-deps` clean.
- Integration drill against test-tenant fixtures.
- SDK calls measured in load-drill lane for SLO conformance.
- `hyperscaler-maturity-claims` lane refuses SDK release if drill numbers exceed declared per-call SLO.

## Examples (Rust)

```rust
use oya_foundry_evidence_sdk::recorder::CapabilityInvocationRecorderClient;
use oya_foundry_evidence_sdk::types::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CapabilityInvocationRecorderClient::from_env()?;

    let envelope = InvocationEnvelope {
        schema_version: "1.0".into(),
        invocation_id: Ulid::new().to_string(),
        attempt_no: 1,
        tenant_id: "tenant:0123456789abcdef".into(),
        source_microservice: SourceMicroservice::FoundryRuntime,
        agent_id: "agent:claim-validator".into(),
        capability_id: "capability:check-claim".into(),
        autonomy_tier_decision: AutonomyTier::T2,
        invocation_ts: Utc::now(),
        pack: "pack-eu".into(),
        idempotency_key: Ulid::new().to_string(),
        payload_data_class: PayloadDataClass::InternalOnly,
        prompt_payload_sha: prompt_sha_hex,
        output_payload_sha: output_sha_hex,
        model_version: Some("claude-opus-4-7-2026-01".into()),
        provider: Some("anthropic".into()),
        request_token_count: Some(1234),
        response_token_count: Some(567),
        subject_hash: None,
        framework_tags: vec![Framework::EuAiAct, Framework::Gdpr],
    };

    let receipt = client.record(envelope).await?;
    println!("pack_id = {}, sealed = {}", receipt.pack_id, receipt.sealed);
    Ok(())
}
```

## ADR-0133 honesty annotation

The SDK does not silently degrade behaviour; if substrate is unavailable and bridge backlog grows, the SDK does NOT pretend the pack is sealed. `sealed=false` in the receipt is a contractual signal the caller can act on.

## References

- `contracts/openapi/foundry-evidence.yaml`.
- `contracts/proto/foundry-evidence.proto`.
- `contracts/asyncapi/foundry-evidence-events.yaml`.
- ADR-0105 (13-layer enum; sdk layer).
- ADR-0131 (per-microservice layout).
- ADR-0133 (claim honesty).
