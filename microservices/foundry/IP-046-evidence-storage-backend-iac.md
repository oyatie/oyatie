---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-001-storage-backend-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: cloud-secrets + axis-foundry-evidence
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance, oya-governance-cross-pack-replication-forbidden, oya-governance-evidence-index-append-only]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Storage backend IaC (Postgres + evidence-blob-store proxy to audit-chain WORM)

## Intent

Helm + Kustomize for Postgres HA evidence index + `evidence-blob-store` chart (which configures the audit-chain WORM bucket consumption — foundry-evidence does NOT own its own WORM bucket per ADR-0131 substrate split). Per-pack overlay. LTS pins per `docs/standards/foundry-evidence.md` (Slice D extension).

## ChangeSet boundary

Pure IaC. 3 Helm chart bundles (evidence-builder + postgres + evidence-blob-store) + shared Kustomize base + pack-kr overlay (M01 launch). Cross-cutting Terraform-managed subscription to audit-chain substrate's per-pack S3 export.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/iac/helm/postgres/Chart.yaml` | create | upstream dep on bitnami/postgresql-ha at pinned LTS |
| `microservices/foundry/iac/helm/postgres/values.yaml` | create | HA primary + replica; INSERT-only role for `foundry_evidence_writer`; SELECT for `foundry_evidence_reader`; SELECT+UPDATE+DELETE for `foundry_evidence_retention_cascader` (Cedar-gated RPC only) |
| `microservices/foundry/iac/helm/evidence-builder/Chart.yaml` | create | deploys pack-builder + recorder REST + bridge worker + regulator-export worker + archive-cascade worker |
| `microservices/foundry/iac/helm/evidence-builder/values.yaml` | create | per-component replicas + resources; SPIFFE binding; PDB; HPA |
| `microservices/foundry/iac/helm/evidence-blob-store/Chart.yaml` | create | wraps S3 bucket-policy chart for the audit-chain WORM bucket SUBSCRIPTION (read-side; foundry-evidence reads from substrate's bucket via cross-µservice IAM) |
| `microservices/foundry/iac/helm/evidence-blob-store/values.yaml` | create | per-pack substrate-bucket reference; cross-µservice IAM principal |
| `microservices/foundry/iac/kustomize/base/kustomization.yaml` | create | shared base |
| `microservices/foundry/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (3y retention reference; KR-pinned) |
| `microservices/foundry/iac/terraform/oci-evidence-blob-store-iam.tf` | create | cross-µservice IAM grant to read substrate-owned WORM bucket |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/postgres
helm lint microservices/foundry/iac/helm/evidence-builder
helm lint microservices/foundry/iac/helm/evidence-blob-store
kubectl --dry-run=client apply -k microservices/foundry/iac/kustomize/overlays/pack-kr
tofu plan microservices/foundry/iac/terraform/
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
cargo run -p oya-dev-cli -- gate validate cross-pack-replication-forbidden --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate evidence-index-append-only --microservice foundry-evidence
```

## Halt Conditions

- Postgres role grant LEAN check fails — block; writer must be INSERT-only.
- Cross-µservice IAM grant attempts to write to substrate-owned WORM — block; foundry-evidence is a READ-side consumer of WORM, not writer.
- Helm version pin drift — block; LTS only.

## Next IP

[`IP-002-self-slo-manifest.md`](IP-002-self-slo-manifest.md)

## References

- ADR-0117 §"Cloud-native infra"; ADR-0131 §"Substrate split".
- `microservices/foundry/policy/evidence-pack-integrity.md` §"EPI-04".
- `microservices/foundry/policy/data-residency.md`.
- `microservices/audit-chain/iac/helm/audit-storage/values.yaml` (substrate-side WORM config).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `evidence`-bounded-context slice for `IP-001: Storage backend IaC (Postgres + evidence-blob-store proxy to audit-chain WORM)`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: cryptographically sealed evidence packs for invocations, evals, guardrails, and regulator exports. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`, `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml`, and the policy set `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `evidence` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-evidence-domain/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

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
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-evidence-domain/src/lib.rs`.
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
