---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-001-runtime-cluster-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + axis-foundry-runtime
acceptance_lanes: [helm-install-smoke, kustomize-build, foundry-runtime-iac-smoke, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Runtime cluster IaC (Kubernetes + Istio + SPIRE + OpenBao bindings)

## Intent

Ship Helm charts + Kustomize overlays for the foundry-runtime dedicated cluster: pod baseline (seccomp + AppArmor + non-root + RO FS), Istio mesh integration (mTLS + SPIFFE), SPIRE wiring, OpenBao SecretReference materialisation, NetworkPolicy default-deny + sibling-allowlist.

## ChangeSet boundary

All paths under `microservices/intelligence/iac/`. No Rust crate changes in this IP.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/runtime-pool/Chart.yaml` | create (LTS-pinned Kubernetes 1.31 baseline) |
| `iac/helm/runtime-pool/values.yaml` | create (per-pack capacity per `capacity-model.md` XS tier) |
| `iac/kustomize/base/kustomization.yaml` | create (shared base) |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | create (pack-kr overlay) |
| `iac/kustomize/base/namespace.yaml` | create (Pod Security Standards `restricted`) |
| `iac/kustomize/base/networkpolicy-default-deny.yaml` | create |
| `iac/kustomize/base/networkpolicy-sibling-allowlist.yaml` | create (mTLS to providers/guardrails/evidence/supervisor) |
| `iac/kustomize/base/openbao-secret-references.yaml` | create |
| `iac/terraform/spire-server.tf` | create (SPIRE server config-as-code) |

## Acceptance Gates

```bash
helm lint microservices/intelligence/iac/helm/runtime-pool/
helm template microservices/intelligence/iac/helm/runtime-pool/ --values microservices/intelligence/iac/helm/runtime-pool/values.yaml > /tmp/runtime-pool-rendered.yaml
kubectl apply --dry-run=server -f /tmp/runtime-pool-rendered.yaml
kustomize build microservices/intelligence/iac/kustomize/overlays/pack-kr/
buck2 build //:quality-lane-registry-authority-check # lane=foundry-runtime-iac-smoke
```

End-to-end kind smoke: deploy charts to ephemeral kind cluster; verify pods Ready ≤2min; verify mTLS handshake via Istio; verify NetworkPolicy refuses non-sibling egress.

## Test Plan

| Test | Verifies |
|---|---|
| `helm-install-smoke.sh` | Charts deploy clean against kind |
| `pod-security-baseline.sh` | Every runtime pod runs non-root + RO FS + seccomp + AppArmor |
| `networkpolicy-default-deny.sh` | Non-sibling egress refused |
| `spire-svid-issued.sh` | Each runtime pod receives SPIFFE SVID within 30s of start |

## Halt Conditions

- Chart deploys but pod hardening missing (seccomp / AppArmor / non-root) — refactor.
- mTLS not enforced on inter-pod traffic.

## Next IP

[`IP-002-redis-and-postgres-baseline.md`](IP-002-redis-and-postgres-baseline.md)

## References

- ADR-0117 (cloud-native infra); ADR-0131; `policy/runtime-isolation.md` TI-10, TI-11.
- Kubernetes Pod Security Standards — `kubernetes.io/docs/concepts/security/pod-security-standards/`.
- SPIRE — `spiffe.io`.
- Istio mTLS — `istio.io/latest/docs/concepts/security/`.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `runtime`-bounded-context slice for `IP-001: Runtime cluster IaC (Kubernetes + Istio + SPIRE + OpenBao bindings)`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: session-coherent hosted agent invocation without tenant-owned runtime infrastructure. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/runtime-capability-execute.yaml`, `microservices/intelligence/capabilities/runtime-session-create.yaml`, `microservices/intelligence/capabilities/runtime-session-resume.yaml`, `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml`, and the policy set `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, `microservices/intelligence/policy/runtime-ci-scope.cedar`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `runtime` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/intelligence/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-api/src/lib.rs` plus the matching catalog records under `microservices/intelligence/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml`, `microservices/intelligence/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, and `microservices/intelligence/contracts/proto/runtime-foundry-runtime.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, `microservices/intelligence/policy/runtime-ci-scope.cedar`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/intelligence/slos/runtime-latency.openslo.yaml`, `microservices/intelligence/slos/runtime-availability.openslo.yaml`, `microservices/intelligence/slos/runtime-freshness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/intelligence/capabilities/runtime-capability-execute.yaml`, `microservices/intelligence/capabilities/runtime-session-create.yaml`, `microservices/intelligence/capabilities/runtime-session-resume.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/intelligence/PRD.md` and the `runtime` row in `microservices/intelligence/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/intelligence/catalog/`, `microservices/intelligence/contracts/`, `microservices/intelligence/policy/`, or `microservices/intelligence/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-api/src/lib.rs`.
- Contract parity for `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml` and `microservices/intelligence/contracts/proto/runtime-foundry-runtime.proto` when DTOs or handlers change.
- Policy resolution against `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, `microservices/intelligence/policy/runtime-ci-scope.cedar`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/intelligence/slos/runtime-latency.openslo.yaml`, `microservices/intelligence/slos/runtime-availability.openslo.yaml`, `microservices/intelligence/slos/runtime-freshness.openslo.yaml`; no acceptance by line count alone.
- `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/intelligence/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/intelligence/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| OpenAI Assistants threads/runs and AWS Bedrock Agents runtime | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
