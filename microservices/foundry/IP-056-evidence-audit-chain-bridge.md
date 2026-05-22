---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-011-audit-chain-bridge
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + axis-audit-chain
acceptance_lanes: [cargo-clippy, lean-cross-microservice-import-forbidden, audit-chain-bridge-availability-drill]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: audit-chain bridge (the only cross-µservice consumer)

## Intent

Implement `oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge` as the SOLE adapter that consumes `oya-audit-chain-emission-sdk`. Encapsulates retry policy, back-off, dead-letter integration, SPIFFE identity binding. Mirror exists for regulator-export bundle-seal path.

## ChangeSet boundary

1 adapter crate (logically; physically already created in IP-006 but this IP delivers the substantive bridge implementation including retry + back-off + dead-letter contract). Plus the regulator-export bundle-seal bridge subset.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/lib.rs` | edit | implement `AuditChainBridgePort` |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/retry.rs` | create | exponential back-off with jitter; bounded retry budget |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/dead_letter.rs` | create | enqueue to dead-letter on persistent failure; emit `foundry.evidence.pack.assembly_failed.v1` on retry exhaustion |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/src/spiffe.rs` | create | SPIFFE workload-identity binding |
| `crates/oya-foundry-evidence-regulator-export-adapter/src/audit_chain_bundle_seal_bridge.rs` | create | bundle-seal path that asks substrate to mint a bundle root from the per-pack tree |
| `crates/oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge/tests/availability_drill.rs` | create | drill: substrate down → bridge holds; substrate recovers → bridge drains |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge --test availability_drill
oya gate validate cross-microservice-import-forbidden --microservice foundry-evidence
oya gate validate audit-chain-bridge-availability-drill --microservice foundry-evidence
```

## Halt Conditions

- Any crate other than this bridge or the bundle-seal bridge subset imports `oya-audit-chain-*` — block.
- Retry budget unbounded — block (DOS risk on the substrate).
- Dead-letter contract missing emit on permanent failure — block (silent loss is forbidden).

## Next IP

[`IP-012-sdk-cross-microservice.md`](IP-012-sdk-cross-microservice.md)

## References

- ADR-0131 §"Substrate split".
- `microservices/audit-chain/contracts/openapi/audit-emit.yaml`.
- `microservices/audit-chain/IP-014-cross-microservice-emission-adapter.md` (substrate-side pattern).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `evidence`-bounded-context slice for `IP-011: audit-chain bridge (the only cross-µservice consumer)`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: cryptographically sealed evidence packs for invocations, evals, guardrails, and regulator exports. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`, `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml`, and the policy set `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `evidence` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-foundry-evidence-domain/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, and `microservices/foundry/contracts/proto/evidence-foundry-evidence.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `evidence` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-foundry-evidence-domain/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml` and `microservices/foundry/contracts/proto/evidence-foundry-evidence.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Palantir AIP audit evidence and ServiceNow GRC evidence export | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
