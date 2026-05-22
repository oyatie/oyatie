---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-003-rule-store-postgres-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, rule-store-migrations-up-to-date, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: Postgres rule-store IaC + migrations

## Intent

Helm chart for per-pack HA Postgres (rule store + Cedar fragment registry + audit-mutation log). Postgres 16 LTS; HA primary + 2 read replicas; pgaudit + Postgres RLS enabled. Migration framework + initial schema migrations.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry/iac/helm/postgres/Chart.yaml` | create | bitnami/postgresql-ha pinned LTS |
| `microservices/foundry/iac/helm/postgres/values.yaml` | create | HA + RLS + pgaudit + TDE |
| `microservices/foundry/iac/helm/postgres/values-pack-kr.yaml` | create | pack-kr |
| `microservices/foundry/iac/postgres/migrations/001-init-schema.sql` | create | tables: rule_definitions, cedar_fragments, audit_mutation_log, classifier_model_versions |
| `microservices/foundry/iac/postgres/migrations/002-rls-policies.sql` | create | Row-level security per tenant + pack |
| `microservices/foundry/iac/postgres/migrations/003-indexes.sql` | create | per `capacity-model.md` access patterns |
| `microservices/foundry/iac/postgres/migrations/manifest.yaml` | create | ordered migration list + checksums |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/postgres
kubectl --dry-run=client apply -k microservices/foundry/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate rule-store-migrations-up-to-date
```

## Test Plan

- helm-install smoke; verify primary + 2 RR reach Ready.
- Migration test: apply 001 → 002 → 003 against ephemeral Postgres; verify schema matches manifest checksums.
- RLS test: insert rows as tenant-A; query as tenant-B; verify zero rows returned.
- Backup-restore test: pg_dump + pg_restore round-trip via `runbooks/rule-store-restore.md`.

## Halt Conditions

- Postgres version drift from LTS — escalate.
- pgaudit not enabled in values — refuse merge.
- Any migration without explicit rollback step — refuse merge.

## Next IP

[`IP-004-prompt-classifier-kernel.md`](IP-004-prompt-classifier-kernel.md)

## References

- ADR-0131; `policy/tenant-isolation.md`; `capacity-model.md`.
- Postgres 16 LTS — `postgresql.org/docs/16/`.
- pgaudit — `github.com/pgaudit/pgaudit`.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `guardrails`-bounded-context slice for `IP-003: Postgres rule-store IaC + migrations`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: inline safety and autonomy enforcement before provider invocation and before output release. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/guardrails-classify-prompt.yaml`, `microservices/foundry/capabilities/guardrails-enforce-autonomy.yaml`, `microservices/foundry/capabilities/guardrails-validate-output.yaml`, `microservices/foundry/contracts/openapi/guardrails-guardrails.yaml`, and the policy set `microservices/foundry/policy/guardrails-tenant-scope.cedar`, `microservices/foundry/policy/guardrails-guardrail-enforcement.md`, `microservices/foundry/policy/guardrails-schema.cedarschema`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `guardrails` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-foundry-autonomy-ceiling-kernel/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

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
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-foundry-autonomy-ceiling-kernel/src/lib.rs`.
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
