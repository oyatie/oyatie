---
doc_class: Implementation-Plan
ip_id: IP-002-migrate-tier-a-check-crates-batch-1
status: planned
owner: axis-governance
wave_scrub: Wave 15-IP-substance 2026-05-21
microservice: governance
---

# Migrate Tier-A check crates batch 1

## A. Problem

The previous slice for `IP-002-migrate-tier-a-check-crates-batch-1` was too close to a design-anchor shell: it named the intended control but did not bind the work to governance's actual contracts, policy files, SLOs, and runbooks. This IP closes the `clean-architecture lane relocation` gap for the governance µservice, not for a generic operations or governance product. The implementation must be reviewable as a single Oya VCS changeset and must not claim runtime maturity until the named artifacts exist and validate.

The service-local grounding is `microservices/governance/manifest.json`, `PRD.md`, `ARCHITECTURE.md`, `contracts/openapi/governance.yaml`, `contracts/asyncapi/governance-events.yaml`, and `contracts/proto/governance.proto`. The authorization grounding is `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`, `policy/tenant-scope.cedar`, and `policy/cedar-canonical-imports.cedar`. The work must preserve ADR-0243 default-deny Cedar semantics, ADR-0244 tenant-scoped evidence, ADR-0263 audit event emission, and ADR-0131 flat µservice ownership.

## B. Approach

Implement the slice as a bounded, contract-first change. Start from the existing capability and catalog surfaces, add or amend only the smallest kernel/usecase/adapter/rest/worker pieces needed for this IP, then wire the dashboard, SLO, and runbook evidence named below. Every mutating path must require an idempotency key, authenticated principal, Cedar decision id, audit event id, and rollback/evidence reference. Read paths must distinguish observed state from operator decisions.

Technical target set: catalog/oya-check-lean-a1.yaml, catalog/oya-check-lean-a2.yaml, catalog/oya-check-port-location.yaml, catalog/oya-check-layer-correctness.yaml, catalog/oya-check-naming-bnf-v41.yaml. If one of these paths is absent when implementation starts, create that exact missing artifact or record an explicit IaC/catalog gap in this IP's evidence; do not cite fake Terraform, fake Cedar entity types, or unavailable endpoints.

## C. Deliverables

- Contract updates in the relevant OpenAPI, AsyncAPI, or Proto file named by the service manifest.
- Domain/kernel value object for `clean-architecture lane relocation` with tenant, principal, cell, HLC timestamp, Cedar decision, audit event, and idempotency fields.
- Usecase orchestration that fails closed when Cedar, OpenBao, audit-chain, or required source projections are unavailable.
- Adapter/rest/worker wiring only where this IP needs runtime I/O; no unrelated refactor across sibling bounded contexts.
- Dashboard, SLO, and runbook linkage using the concrete artifact set: catalog/oya-check-lean-a1.yaml, catalog/oya-check-lean-a2.yaml, catalog/oya-check-port-location.yaml, catalog/oya-check-layer-correctness.yaml, catalog/oya-check-naming-bnf-v41.yaml.
- Catalog/capability row update when the IP exposes or changes an operator/governance capability.

## D. Implementation Steps

1. Read `microservices/governance/manifest.json`, `PRD.md`, `ARCHITECTURE.md`, `contracts/openapi/governance.yaml`, `contracts/asyncapi/governance-events.yaml`, and `contracts/proto/governance.proto` and confirm the bounded context that owns `clean-architecture lane relocation`; update the manifest or catalog only if that owner is missing.
2. Add the kernel/domain type with explicit tenant scope, principal scope, HLC time, decision ids, and audit seal refs; keep provider credentials as OpenBao references rather than raw secrets.
3. Add usecase logic that evaluates Cedar before storage/provider access and returns structured refusal evidence on deny, stale pack, missing tenant, or audit-chain backpressure.
4. Update the selected REST/gRPC/event contract so external callers and workers share the same envelope and error shape.
5. Wire dashboard/SLO/runbook evidence from catalog/oya-check-lean-a1.yaml, catalog/oya-check-lean-a2.yaml, catalog/oya-check-port-location.yaml, catalog/oya-check-layer-correctness.yaml, catalog/oya-check-naming-bnf-v41.yaml; dashboard panels must point to real metric/event names and runbook links must resolve.
6. Add tests for allow, deny, stale policy/pack, duplicate idempotency key, audit emission failure, and rollback/evidence replay.
7. Run the service-local validation commands named in acceptance, then attach the command output and changed-file list to the changeset evidence.

## E. Acceptance

- The IP cites real service artifacts and no placeholder paths.
- Contract validation parses the touched OpenAPI/AsyncAPI/Proto surface.
- Cedar tests prove at least one permit and one forbid path for the concrete action in this IP.
- Audit evidence includes an ADR-0263 event class, Ed25519/Merkle seal reference where applicable, and a replay or rollback reference.
- SLO/dashboard/runbook references resolve from the repo tree.
- `oya vcs verify --agent <id> --changeset <id>` passes before done/promote.

## F. Evidence

- Service docs: `microservices/governance/manifest.json`, `PRD.md`, `ARCHITECTURE.md`, `contracts/openapi/governance.yaml`, `contracts/asyncapi/governance-events.yaml`, and `contracts/proto/governance.proto`.
- Policy docs: `policy/auditor-scope.cedar`, `policy/ci-scope.cedar`, `policy/tenant-scope.cedar`, and `policy/cedar-canonical-imports.cedar`.
- Operational evidence: catalog/oya-check-lean-a1.yaml, catalog/oya-check-lean-a2.yaml, catalog/oya-check-port-location.yaml, catalog/oya-check-layer-correctness.yaml, catalog/oya-check-naming-bnf-v41.yaml.
- Doctrine: ADR-0324 anti-template-stamping, ADR-0328 D-20 Big-8 elevation, ADR-0131 flat µservice layout, ADR-0263 audit events, ADR-0243 Cedar deny-wins.

## G. Counterparts

| Counterpart | Relevant pressure | Oyatie closure in this IP |
|---|---|---|
| SonarQube and GitHub Advanced Security | Mature external control surface to compare against. | SonarQube and GitHub Advanced Security are counterpart PR gates; Oyatie preserves crate history while moving enforcement under the governance µservice. |
| GitHub | Required verification regex and PR/evidence control-plane precedent. | This IP remains changeset-driven, reviewable, and tied to branch/admission evidence rather than prose-only approval. |

## H. Service-Specific Drilldown
1. Move lean-a1, lean-a2, port-location, layer-correctness, and naming-bnf-v41 as history-preserving `git mv` operations.
2. Update workspace members and catalog rows atomically so no lane points to both old and new homes.
3. Run self-application against governance after migration because these lanes define the architecture boundary they now live under.
4. Use SonarQube/GitHub Advanced Security only as PR-gate counterparts; Oyatie-specific clean-architecture lanes remain bespoke.
5. Halt if path rewrites require raw blanket sed; ADR-0131 requires structured migration.
6. Evidence includes old path, new path, cargo metadata member, and per-lane validation output.

## I. Review Notes

This section is intentionally specific to this IP; do not copy it to sibling IPs. Reviewers should reject the changeset if the implementation evidence cannot trace each drilldown row to a real file, test, command, dashboard, SLO, runbook, or policy decision.

## J. Verification Hooks

- Hook 5.1: changed-file evidence must include this IP path and the concrete service artifacts named above.
- Hook 5.2: contract parsing must run after any OpenAPI, AsyncAPI, or Proto edit for this slice.
- Hook 5.3: Cedar permit and forbid cases must cite the real policy file and action name used by this slice.
- Hook 5.4: audit evidence must include event class, seal reference, actor, tenant/cell scope, and idempotency key.
- Hook 5.5: rollback evidence must name the runbook or explain why the slice is read-only.
- Hook 5.6: counterpart closure must be reviewed against the named GitHub/Stripe/Snowflake/etc. row, not inferred from line count.
- Hook 5.7: promotion is blocked if any cited dashboard, SLO, catalog, capability, or runbook path is absent.
