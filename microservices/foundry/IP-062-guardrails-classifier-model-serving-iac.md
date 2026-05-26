---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-002-classifier-model-serving-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, classifier-model-cosign-signed, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: ONNX-runtime classifier-model-serving IaC

## Intent

Helm chart for in-cluster ONNX-runtime classifier-serving (4 models per pack: PII/PHI BERT-class, jailbreak classifier, content-safety Llama-Guard-class, AI-slop BERT-small). Cosign-signed model artifacts; per-pack S3 model registry; pod-start integrity verification.

## ChangeSet boundary

One ChangeSet: Helm chart + per-model values + Cosign key references via OpenBao + model registry S3 layout + pod-start integrity-check init container. M01 ships placeholder artifacts (small distilled BERT) per pack; final production models per ADR successor-IP.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/iac/helm/classifier-model-serving/Chart.yaml` | create | upstream onnxruntime-server chart pinned |
| `microservices/foundry/iac/helm/classifier-model-serving/values.yaml` | create | 4-model spec; replica counts per `capacity-model.md` |
| `microservices/foundry/iac/helm/classifier-model-serving/values-pack-kr.yaml` | create | pack-kr overlay |
| `microservices/foundry/iac/cosign/keys.yaml` | create | Cosign public-key references (OpenBao-bound) |
| `microservices/foundry/iac/cosign/init-verify.sh` | create | pod-start verification script |
| `microservices/foundry/iac/model-registry/layout.md` | create | per-pack S3 bucket layout + signed-manifest schema |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/classifier-model-serving
kubectl --dry-run=client apply -k microservices/foundry/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate classifier-model-cosign-signed
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm-lint + helm-install smoke per chart.
- E2E: kind cluster; deploy 4 placeholder models; verify integrity-check passes; verify pods reach Ready; verify classifier-serving exposes `/v1/classify` endpoint.
- Negative: deploy tampered model; verify integrity-check refuses pod-start + emits `classifier_model_integrity_violation` metric.

## Halt Conditions

- Cosign key not present in OpenBao — block.
- Model artifact size > 1GB — escalate (compute envelope review).
- ONNX runtime version drift from LTS — escalate.

## Next IP

[`IP-003-rule-store-postgres-iac.md`](IP-003-rule-store-postgres-iac.md)

## References

- ADR-0131; `policy/guardrail-enforcement.md`; `capacity-model.md`.
- ONNX Runtime — `onnxruntime.ai`.
- Cosign — `docs.sigstore.dev/cosign/`.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `guardrails`-bounded-context slice for `IP-002: ONNX-runtime classifier-model-serving IaC`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: inline safety and autonomy enforcement before provider invocation and before output release. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/guardrails-classify-prompt.yaml`, `microservices/foundry/capabilities/guardrails-enforce-autonomy.yaml`, `microservices/foundry/capabilities/guardrails-validate-output.yaml`, `microservices/foundry/contracts/openapi/guardrails-guardrails.yaml`, and the policy set `microservices/foundry/policy/guardrails-tenant-scope.cedar`, `microservices/foundry/policy/guardrails-guardrail-enforcement.md`, `microservices/foundry/policy/guardrails-schema.cedarschema`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `guardrails` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-autonomy-ceiling-kernel/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/guardrails-guardrails.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, and `microservices/foundry/contracts/proto/guardrails-guardrails.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/guardrails-tenant-scope.cedar`, `microservices/foundry/policy/guardrails-guardrail-enforcement.md`, `microservices/foundry/policy/guardrails-schema.cedarschema`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/guardrails-classify-prompt.yaml`, `microservices/foundry/capabilities/guardrails-enforce-autonomy.yaml`, `microservices/foundry/capabilities/guardrails-validate-output.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `guardrails` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-autonomy-ceiling-kernel/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/guardrails-guardrails.yaml` and `microservices/foundry/contracts/proto/guardrails-guardrails.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/guardrails-tenant-scope.cedar`, `microservices/foundry/policy/guardrails-guardrail-enforcement.md`, `microservices/foundry/policy/guardrails-schema.cedarschema`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| AWS Bedrock Guardrails, OpenAI Moderation, and NVIDIA NeMo Guardrails | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
