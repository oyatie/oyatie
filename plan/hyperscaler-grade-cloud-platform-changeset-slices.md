---
plan_id: PLAN-HYPERSCALER-CLOUD-CHANGESET-SLICES
title: Hyperscaler Cloud Platform Changeset Slices
status: Draft
date: 2026-05-23
source_plan: plan/hyperscaler-grade-cloud-platform-greenfield-plan.md
audit_plan: plan/hyperscaler-grade-cloud-platform-main-plan-audit.md
scope: greenfield-implementation-slices
---

# Hyperscaler Cloud Platform Changeset Slices

## 1. Purpose

This document decomposes the greenfield hyperscaler cloud platform plan into
actionable, disjoint, parallelizable implementation changesets. Each changeset
belongs to a larger phase, has a narrow path envelope, has explicit acceptance
criteria, and can be assigned to an isolated agent worktree.

The goal is to prevent "giant platform build" failure by forcing:

- independent ownership;
- small reviewable changes;
- explicit dependency order;
- phase checkpoints;
- no hidden file/path contention;
- no undocumented cross-service coupling;
- verification before promotion.

## 2. Specification Anchor

### Objective

Build a hyperscaler-grade cloud platform from scratch with a systematic
platform-engineering, project-management, and development-lifecycle operating
model. The first implementation target is a functional internal cloud preview
that proves account/IAM, audit/metering, region/cell substrate, compute,
networking, storage, managed Kubernetes, observability, security, and launch
governance.

### Commands

Greenfield implementation should standardize on these commands:

```bash
make fmt
make lint
make test
make test-contract
make test-integration
make test-load
make test-security
make build
make sbom
make provenance
make sign
make verify
make deploy ENV=dev REGION=dev-1 CELL=cell-a
make rollback ENV=dev REGION=dev-1 CELL=cell-a RELEASE=<release-id>
make launch-check SERVICE=<service-name> REGION=<region> CELL=<cell>
```

### Project Structure

```text
cloud/
  services/<service-name>/
  control-plane/<component-name>/
  data-plane/<component-name>/
  platform/<platform-capability>/
  contracts/
  infra/
  tests/
  runbooks/
  dashboards/
  plans/
```

### Code Style Baseline

Mutations are typed, scoped, idempotent, and auditable:

```rust
pub struct CreateResourceRequest {
    pub account_id: AccountId,
    pub region: RegionId,
    pub idempotency_key: IdempotencyKey,
    pub tags: Tags,
}

pub trait ControlPlaneCommand {
    type Output;

    fn execute(
        &self,
        request: CreateResourceRequest,
        actor: AuthenticatedActor,
    ) -> Result<Self::Output, ControlPlaneError>;
}
```

### Testing Strategy

Every changeset declares the narrowest verification command, but promotion to a
phase checkpoint requires:

- contract tests for every API or event;
- unit tests for domain/application behavior;
- integration tests for cross-service workflows;
- security tests for authz, secrets, and isolation;
- load or chaos tests for service families that claim performance,
  reliability, scalability, or availability.

### Boundaries

Always:

- one owner per changeset;
- one path envelope per changeset;
- one acceptance checklist per changeset;
- one verification story per changeset;
- contract-first when another changeset depends on the API.

Ask first:

- changing shared resource identity;
- changing IAM policy semantics;
- changing common error model;
- adding foundational dependency;
- crossing another changeset's path envelope.

Never:

- write another service's database;
- add a public mutation without idempotency;
- add a list endpoint without pagination;
- add a customer mutation without audit and metering;
- create global dependency in a regional data-plane hot path.

## 3. Changeset Rules

### 3.1 Disjointness Rule

Every changeset owns one **path envelope**. No other changeset may edit that
path envelope until the changeset is promoted. If two agents need a shared
contract, create or promote the contract changeset first, then implement
consumers.

### 3.2 Sizing Rule

Target size:

- **S:** one service/component skeleton, one contract, or one test harness.
- **M:** one vertical slice with contract + implementation + tests within one
  service boundary.
- Avoid **L** except for generated skeletons or explicit integration checkpoints.

### 3.3 Parallelization Rule

Changesets in the same **parallel batch** may run concurrently if they do not
share path envelopes and all listed dependencies are complete.

### 3.4 Promotion Rule

A phase checkpoint can promote only when every changeset in the phase is either:

- complete and verified;
- explicitly deferred with no dependent changeset blocked;
- replaced by a newer changeset with equivalent acceptance criteria.

### 3.5 Universal Changeset Definition Of Done

Every changeset, including plan-hardening changesets, is done only when:

- [ ] the path envelope was respected and no unrelated files were changed;
- [ ] acceptance criteria are satisfied with concrete evidence;
- [ ] the narrowest verification command or review checklist has passed;
- [ ] public contracts, data ownership, telemetry, security, and rollback impacts
      are either updated or explicitly marked not applicable;
- [ ] dependent changesets can consume the result without hidden assumptions;
- [ ] any standard, framework, or external source used by the changeset is
      pinned to a date or version and has a future re-check point before
      production commitment.

## 4. Dependency Graph Summary

```text
Phase A Plan Audit / Roadmap Hardening
  -> P0 Program OS
      -> P1 Platform Factory
          -> P2 Region/Cell Substrate
              -> P3 Trust/Commerce Foundation
                  -> P4 Core IaaS Contracts
                      -> P5 Compute/Network/Storage Data Planes
                          -> P6 Managed Kubernetes
                              -> P7 Observability/Security/DR
                                  -> P8 Internal Preview
                                      -> P9 Customer Preview
                                          -> P10 Public Preview/GA
```

Parallel lanes after P1:

```text
Plan-hardening lane:     audit -> standards hygiene -> traceability -> DoD
Region substrate lane:   region metadata -> cells -> placement -> capacity
Trust lane:              account -> IAM -> audit/metering -> billing
Compute lane:            compute contract -> host agent -> VM lifecycle
Network lane:            VPC contract -> ENI/security groups -> LB/DNS
Storage lane:            object contract -> object metadata/data path
                          block contract -> volume attach/snapshot
Platform lane:           catalog -> CI/provenance -> launch gates -> docs/devex
Ops lane:                telemetry -> SLOs -> incident -> backup/DR
Quality-review lane:     overload -> isolation -> privacy -> canary/PRR -> cost data -> runtime hardening -> abuse/DDoS
```

## 5. Parallel Batch Matrix

| Batch | Can run in parallel | Must be complete before |
|---|---|---|
| A0 | CSA-0001, CSA-0003 | A1 |
| A1 | CSA-0002 | A2 |
| A2 | CSA-0004 | B0 |
| B0 | CS-0001, CS-0002, CS-0003 | B1 |
| B1 | CS-0101, CS-0102, CS-0103, CS-0104 | B2 |
| B2 | CS-0201, CS-0202, CS-0203 | B3 and B4 |
| B3 | CS-0301, CS-0302, CS-0303, CS-0304 | B5 |
| B4 | CS-0401, CS-0501, CS-0601, CS-0602 | B5 |
| B5 | CS-0402, CS-0502, CS-0603, CS-0604 | B6 |
| B6 | CS-0403, CS-0503, CS-0605, CS-0701 | B6b |
| B6b | CS-0404 | B7a |
| B7a | CS-0702, CS-0801, CS-0802, CS-0803, CS-0804 | B7b |
| B7b | CS-0805, CS-0806, CS-0807, CS-0808, CS-0809, CS-0810, CS-0811 | B8 |
| B8 | CS-0901, CS-0902, CS-0903 | B9 |
| B9 | CS-1001, CS-1002, CS-1003 | GA decision |

## 6. Phase A: Plan Audit And Roadmap Hardening

This phase makes the roadmap auditable before implementation begins. It is
small by design: the plan, audit, and changeset plan must agree on scope,
standards, traceability, dependencies, and Definition of Done before agents
start changing implementation paths.

### 6.1 Audit-To-Changeset Traceability

| Audit area | Main plan anchor | Changesets that implement or prove it |
|---|---|---|
| Honest claim boundary | Sections 0, 0.1, 20.2, 22 | CSA-0001, CSA-0002, CS-1003, CS-1102, CS-1103, CS-1104 |
| Product breadth | Sections 4, 5.8, 12, 13 | CS-0301 through CS-1104 by service family |
| Region / zone / cell scaling | Sections 5.2, 5.4, 5.5, 13.2 | CS-0201, CS-0402, CS-0804, CS-1102, CS-1104 |
| Control-plane / data-plane separation | Sections 5.3, 5.4, 5.10.6, 11 | CS-0102, CS-0201, CS-0402, CS-0403, CS-0604, CS-0804 |
| Microservice boundaries | Sections 5.10, 10.1, 11 | CS-0101, CS-0102, CS-0203, service-family contract changesets |
| Clean architecture enforcement | Sections 5.10.2, 10.1.2, 10.1.9 | CS-0101, CS-0102, CS-0404, CS-0502, CS-0603, CS-0702 |
| Platform engineering operating model | Sections 6, 10, 18 | CS-0101, CS-0102, CS-0103, CS-0104, CS-0903 |
| Project/program management | Sections 7, 12, 16, 20 | CS-0001, CS-0002, CS-0003, CS-1001, CS-1003 |
| Development lifecycle | Sections 8, 14, 19 | CS-0104, CS-0802, CS-1003, phase checkpoints |
| PRAOSAO quality bars | Sections 9, 9.9 | CS-0801 through CS-0811, CS-1102, CS-1104 |
| Trust and compliance | Sections 5.7, 8.3, 9.3, 9.9, 16 | CS-0302, CS-0303, CS-0802, CS-0807, CS-0811, CS-1103 |
| Commercial readiness | Sections 4, 5.7, 7.3, 9.9, 16, 20 | CS-0304, CS-0809, CS-0903, CS-1001, CS-1101 |
| Developer experience | Sections 6, 10, 16 | CS-0102, CS-0901, CS-0902, CS-0903 |
| Evidence gates | Sections 5.9, 9.9, 14, 16, 20.1, 22 | CS-0801 through CS-0811, CS-1003, CS-1102 through CS-1104 |
| Current-source hygiene | Sections 2, 9.8 | CSA-0003, CS-0104, CS-0802, CS-1103 |
| Overload and fairness | Section 9.9 | CS-0805, CS-0808, CS-1104 |
| Tenant isolation and shuffle sharding | Sections 5.2, 5.10, 9.9 | CS-0201, CS-0402, CS-0806, CS-1102 |
| Privacy and data governance | Sections 5.7, 9.8, 9.9 | CS-0807, CS-1103 |
| Runtime hardening | Sections 8.3, 9.8, 9.9 | CS-0702, CS-0810, CS-1103 |
| Abuse and DDoS readiness | Sections 5.6, 9.9, 16 | CS-0503, CS-0811, CS-1001, CS-1101 |

### CSA-0001: Main plan audit artifact

**Path envelope:** `plan/hyperscaler-grade-cloud-platform-main-plan-audit.md`

**Description:** Audit the main greenfield plan against hyperscaler roadmap
criteria and record pass/gap/remediation findings without inspecting existing
repository documentation.

**Acceptance criteria:**

- [ ] Audit criteria cover architecture, platform engineering, project
      management, lifecycle, PRAOSAO quality bars, trust, commerce, developer
      experience, evidence gates, and decomposability.
- [ ] Each non-pass finding has an explicit remediation changeset.
- [ ] The verdict preserves the honest boundary between roadmap and achieved
      hyperscaler proof.

**Verification:**

- [ ] `rg -n "F-[0-9]+|Verdict|Remediation" plan/hyperscaler-grade-cloud-platform-main-plan-audit.md`

**Dependencies:** None
**Parallel batch:** A0
**Estimated scope:** S

### CSA-0002: Audit-to-changeset traceability matrix

**Path envelope:** `plan/hyperscaler-grade-cloud-platform-changeset-slices.md`

**Description:** Add a traceability matrix that maps main-plan audit criteria
to implementation and evidence changesets so gaps cannot disappear between
planning and execution.

**Acceptance criteria:**

- [ ] Traceability matrix covers every audit criterion.
- [ ] Every critical roadmap area maps to one or more changesets.
- [ ] Matrix identifies evidence-pack changesets for preview and GA claims.

**Verification:**

- [ ] `rg -n "Audit-To-Changeset Traceability|Current-source hygiene|PRAOSAO" plan/hyperscaler-grade-cloud-platform-changeset-slices.md`

**Dependencies:** CSA-0001
**Parallel batch:** A1
**Estimated scope:** S

### CSA-0003: Current standards hygiene update

**Path envelope:** `plan/hyperscaler-grade-cloud-platform-greenfield-plan.md`

**Description:** Check versioned standards referenced by the main plan against
official/upstream sources and update stale references before implementation
planning consumes them.

**Acceptance criteria:**

- [ ] Versioned standards in the main plan are current as of the audit date or
      explicitly marked for pre-implementation re-check.
- [ ] Any mismatch found by the audit is patched in the main plan.
- [ ] Future production commitments require re-checking official sources.

**Verification:**

- [ ] `rg -n "OpenAPI Specification|OpenTelemetry semantic conventions|SLSA specification v1.2|NIST Cybersecurity Framework 2.0" plan/hyperscaler-grade-cloud-platform-greenfield-plan.md`

**Dependencies:** None
**Parallel batch:** A0
**Estimated scope:** S

### CSA-0004: Universal changeset Definition of Done

**Path envelope:** `plan/hyperscaler-grade-cloud-platform-changeset-slices.md`

**Description:** Add a universal Definition of Done that applies to all future
changesets and prevents incomplete slices from being promoted.

**Acceptance criteria:**

- [ ] Definition of Done covers path envelope discipline, acceptance evidence,
      verification, dependency consumption, and source-version hygiene.
- [ ] Promotion rules require changesets to satisfy the Definition of Done.
- [ ] The change submission template remains compatible with the Definition of
      Done.

**Verification:**

- [ ] `rg -n "Universal Changeset Definition Of Done|Promotion Rule|Change Submission Template" plan/hyperscaler-grade-cloud-platform-changeset-slices.md`

**Dependencies:** CSA-0002
**Parallel batch:** A2
**Estimated scope:** S

### Checkpoint PA

- [ ] Main plan audit exists and contains no unresolved critical blockers.
- [ ] Current versioned standards are patched or tracked.
- [ ] Every audit criterion maps to at least one changeset or evidence gate.
- [ ] Universal changeset Definition of Done is explicit.
- [ ] B0 implementation planning can start without hidden roadmap gaps.

## 7. Phase 0: Program Operating System

### CS-0001: Cloud product charter

**Path envelope:** `cloud/plans/product-charter/`

**Description:** Define target users, first launch geography, first service
families, commercial stage, non-goals, and readiness definition.

**Acceptance criteria:**

- [ ] Charter names first three service families.
- [ ] Charter names launch stages from internal preview to GA.
- [ ] Charter names explicit first-release non-goals.

**Verification:**

- [ ] `make verify-plan PLAN=cloud/plans/product-charter/charter.md`

**Dependencies:** None
**Parallel batch:** B0
**Estimated scope:** S

### CS-0002: Governance and decision gates

**Path envelope:** `cloud/plans/governance/`

**Description:** Define councils, gate owners, concept/architecture/build/preview
/GA/deprecation gate criteria, and risk acceptance rules.

**Acceptance criteria:**

- [ ] Gate matrix exists with owner and required evidence.
- [ ] Risk acceptance requires owner, expiry, severity, and mitigation.
- [ ] Launch review cannot pass without evidence attachments.

**Verification:**

- [ ] `make verify-plan PLAN=cloud/plans/governance/gates.md`

**Dependencies:** None
**Parallel batch:** B0
**Estimated scope:** S

### CS-0003: Service ownership and RACI model

**Path envelope:** `cloud/plans/ownership/`

**Description:** Define service ownership, on-call ownership, directly
responsible individual, RACI model, escalation model, and service lifecycle
states.

**Acceptance criteria:**

- [ ] Every service lifecycle state has required owner metadata.
- [ ] On-call and escalation ownership are separate from product ownership.
- [ ] Unowned service state is invalid.

**Verification:**

- [ ] `make verify-plan PLAN=cloud/plans/ownership/raci.md`

**Dependencies:** None
**Parallel batch:** B0
**Estimated scope:** S

### Checkpoint P0

- [ ] Product charter approved.
- [ ] Governance gates approved.
- [ ] Ownership model approved.
- [ ] No implementation changesets start without P0 approval.

## 8. Phase 1: Platform Factory

### CS-0101: Greenfield repository scaffold

**Path envelope:** `cloud/`

**Description:** Create the top-level greenfield repository structure,
placeholder package manifests, root commands, and directory ownership notes.

**Acceptance criteria:**

- [ ] Top-level directories match the project structure in this document.
- [ ] Repository includes clean-architecture service shape under generated
  service template: `domain`, `ports`, `application`, `adapters`, `api`,
  `worker`, `runtime`, `contracts`, `tests`, `deploy`, `runbooks`,
  `dashboards`, and `evidence`.
- [ ] Repository includes contract/generated-client separation:
  `contracts/` is source of truth and `generated/` is output only.
- [ ] Repository includes `libs/foundation/` and excludes shared business-domain
  packages from the scaffold.
- [ ] Root `make verify` runs without requiring service implementation.
- [ ] Empty directories have explicit purpose files or equivalent metadata.

**Verification:**

- [ ] `make verify`

**Dependencies:** CS-0001, CS-0002, CS-0003
**Parallel batch:** B1
**Estimated scope:** M

### CS-0102: Golden service template

**Path envelope:** `cloud/platform/service-templates/control-plane-service/`

**Description:** Create the first service template with clean architecture
layers, contract folder, tests, deployment metadata, health/readiness, telemetry,
and runbook placeholders.

**Acceptance criteria:**

- [ ] Template includes `domain`, `ports`, `application`, `adapters`, `api`,
  `worker`, `runtime`, `contracts`, `tests`, `runbooks`, and `dashboards`.
- [ ] Generated sample exposes health/readiness and one idempotent example
  mutation.
- [ ] Generated sample has unit and contract test placeholders.

**Verification:**

- [ ] `make generate-service NAME=sample-control-plane`
- [ ] `make verify SERVICE=sample-control-plane`

**Dependencies:** CS-0101
**Parallel batch:** B1 after CS-0101 if generated paths are isolated
**Estimated scope:** M

### CS-0103: Service catalog schema

**Path envelope:** `cloud/platform/service-catalog/`

**Description:** Define service catalog schema, validation rules, ownership
fields, lifecycle fields, SLO references, dependencies, runbook references, and
deployment targets.

**Acceptance criteria:**

- [ ] Catalog schema validates owner, lifecycle, APIs, dependencies, SLOs,
  runbooks, dashboards, and data stores.
- [ ] Invalid production service without owner/SLO/runbook is rejected.
- [ ] Catalog has seed records for sample services only.

**Verification:**

- [ ] `make test SERVICE=service-catalog`
- [ ] `make verify-catalog`

**Dependencies:** CS-0101
**Parallel batch:** B1
**Estimated scope:** M

### CS-0104: CI, provenance, and signed artifact lane

**Path envelope:** `cloud/platform/ci-cd/`

**Description:** Define build, test, contract, scan, SBOM, provenance, signing,
deploy, and rollback pipeline interfaces.

**Acceptance criteria:**

- [ ] Pipeline fails unsigned artifact deployment.
- [ ] Pipeline emits SBOM and provenance placeholders.
- [ ] Pipeline supports service-scoped verification.

**Verification:**

- [ ] `make build SERVICE=sample-control-plane`
- [ ] `make sbom SERVICE=sample-control-plane`
- [ ] `make provenance SERVICE=sample-control-plane`
- [ ] `make sign SERVICE=sample-control-plane`
- [ ] `make verify SERVICE=sample-control-plane`

**Dependencies:** CS-0101
**Parallel batch:** B1
**Estimated scope:** M

### Checkpoint P1

- [ ] Repository scaffold is usable.
- [ ] Golden service template generates a sample.
- [ ] Service catalog validates ownership and lifecycle.
- [ ] CI/provenance/signing lane works against sample service.

## 9. Phase 2: Region, Cell, And Shared Control Substrate

### CS-0201: Region, zone, cell metadata contract

**Path envelope:** `cloud/control-plane/region-metadata/`

**Description:** Implement the contract and domain model for global, region,
zone, cell, rack, host, service shard, capacity pool, and failure boundary.

**Acceptance criteria:**

- [ ] Metadata model represents region, zone, cell, rack, host, capacity, and
  failure boundary.
- [ ] Contract supports create, describe, list, and mark-health for cells.
- [ ] List APIs are paginated.

**Verification:**

- [ ] `make test SERVICE=region-metadata`
- [ ] `make test-contract SERVICE=region-metadata`

**Dependencies:** Checkpoint P1
**Parallel batch:** B2
**Estimated scope:** M

### CS-0202: Quota and rate-limit contract

**Path envelope:** `cloud/control-plane/quota/`

**Description:** Implement the quota service contract and domain model for
account, region, service, resource-type, and operation-level quotas.

**Acceptance criteria:**

- [ ] Quota decisions include allow, deny, reason, current usage, and limit.
- [ ] Quota checks are account/region/service scoped.
- [ ] Rate-limit response uses stable public error shape.

**Verification:**

- [ ] `make test SERVICE=quota`
- [ ] `make test-contract SERVICE=quota`

**Dependencies:** Checkpoint P1
**Parallel batch:** B2
**Estimated scope:** M

### CS-0203: Common public API contract primitives

**Path envelope:** `cloud/contracts/common/`

**Description:** Define shared API primitives: resource identity, pagination,
idempotency key, request ID, error envelope, tags, region/zone/cell references,
and deprecation metadata.

**Acceptance criteria:**

- [ ] Error envelope has stable code, message, request ID, retryability, and
  documentation link fields.
- [ ] Pagination contract supports cursor-based pagination.
- [ ] Idempotency contract is mandatory for mutation examples.

**Verification:**

- [ ] `make test-contract CONTRACT=common`

**Dependencies:** Checkpoint P1
**Parallel batch:** B2
**Estimated scope:** M

### Checkpoint P2

- [ ] Region/cell metadata contract complete.
- [ ] Quota contract complete.
- [ ] Common API primitives complete.
- [ ] P3 and P4 services can depend on these contracts.

## 10. Phase 3: Trust, Commerce, And Global Foundation

### CS-0301: Account and organization service

**Path envelope:** `cloud/services/account/`

**Description:** Implement account, organization, lifecycle state, region opt-in,
default quotas, and tags using the clean architecture service template.

**Acceptance criteria:**

- [ ] Create account mutation is idempotent.
- [ ] Account can opt into a region.
- [ ] Account emits audit and metering placeholder events.

**Verification:**

- [ ] `make test SERVICE=account`
- [ ] `make test-contract SERVICE=account`

**Dependencies:** CS-0202, CS-0203
**Parallel batch:** B3
**Estimated scope:** M

### CS-0302: IAM and STS baseline

**Path envelope:** `cloud/services/iam/`

**Description:** Implement users, roles, policies, role assumption, short-lived
credential model, federation placeholders, and service identity contracts.

**Acceptance criteria:**

- [ ] Actor can assume a role with scoped temporary credentials.
- [ ] Unauthorized resource mutation is denied.
- [ ] Policy evaluation emits audit placeholder event.

**Verification:**

- [ ] `make test SERVICE=iam`
- [ ] `make test-security SERVICE=iam`

**Dependencies:** CS-0301
**Parallel batch:** B3 after CS-0301 contract is stable
**Estimated scope:** M

### CS-0303: Audit service

**Path envelope:** `cloud/services/audit/`

**Description:** Implement append-only audit event contract, regional buffering,
global aggregation placeholder, query API, and export shape.

**Acceptance criteria:**

- [ ] Audit event includes actor, account, region, service, action, resource,
  result, request ID, and timestamp.
- [ ] Regional buffer can accept events when aggregation is unavailable.
- [ ] Query API is paginated and account-scoped.

**Verification:**

- [ ] `make test SERVICE=audit`
- [ ] `make test-integration SERVICE=audit SCENARIO=regional-buffer-replay`

**Dependencies:** CS-0201, CS-0203
**Parallel batch:** B3
**Estimated scope:** M

### CS-0304: Metering and billing baseline

**Path envelope:** `cloud/services/metering-billing/`

**Description:** Implement meter catalog, usage record contract, regional usage
buffer, invoice preview, budgets, and cost allocation tags.

**Acceptance criteria:**

- [ ] Usage record includes account, service, region, resource, dimension,
  quantity, and timestamp.
- [ ] Invoice preview aggregates by account, service, region, and tag.
- [ ] Budget threshold emits alert event.

**Verification:**

- [ ] `make test SERVICE=metering-billing`
- [ ] `make test-integration SERVICE=metering-billing SCENARIO=usage-to-invoice`

**Dependencies:** CS-0301, CS-0303
**Parallel batch:** B3 after CS-0301 API contract is stable
**Estimated scope:** M

### Checkpoint P3

- [ ] Account, IAM, audit, metering, and billing baseline work together.
- [ ] Customer-visible mutations can be authorized, audited, and metered.
- [ ] All trust services have owners, SLO placeholders, dashboards, and runbooks.

## 11. Phase 4: Core IaaS Contracts

### CS-0401: Compute API contract

**Path envelope:** `cloud/services/compute/contracts/`

**Description:** Define VM, image, instance type, network interface reference,
volume reference, lifecycle state, host maintenance, and idempotent mutation API
contracts.

**Acceptance criteria:**

- [ ] API supports create, describe, list, start, stop, reboot, terminate.
- [ ] Create/terminate are idempotent.
- [ ] Lifecycle state machine is explicit.

**Verification:**

- [ ] `make test-contract SERVICE=compute`

**Dependencies:** CS-0203, CS-0302
**Parallel batch:** B4
**Estimated scope:** S

### CS-0501: VPC networking API contract

**Path envelope:** `cloud/services/network/contracts/`

**Description:** Define VPC, subnet, route table, security group, network
interface, IPAM, NAT, load balancer, target group, and DNS contracts.

**Acceptance criteria:**

- [ ] VPC and subnet APIs are account/region scoped.
- [ ] Security group denies inbound traffic by default.
- [ ] Load balancer target health contract is defined.

**Verification:**

- [ ] `make test-contract SERVICE=network`

**Dependencies:** CS-0203, CS-0302
**Parallel batch:** B4
**Estimated scope:** S

### CS-0601: Object storage API contract

**Path envelope:** `cloud/services/object-storage/contracts/`

**Description:** Define bucket, object, versioning, policy, lifecycle,
encryption, replication class, and audit/metering contracts.

**Acceptance criteria:**

- [ ] Bucket/object lifecycle APIs are defined.
- [ ] Bucket policy authorization model is defined.
- [ ] Object operations include metering dimensions.

**Verification:**

- [ ] `make test-contract SERVICE=object-storage`

**Dependencies:** CS-0203, CS-0302
**Parallel batch:** B4
**Estimated scope:** S

### CS-0602: Block storage API contract

**Path envelope:** `cloud/services/block-storage/contracts/`

**Description:** Define volume, attachment, snapshot, encryption, zone
placement, lifecycle, and failure-mode contracts.

**Acceptance criteria:**

- [ ] Volume create/attach/detach/snapshot/delete APIs are defined.
- [ ] Volume placement is zone/cell scoped.
- [ ] Snapshot restore contract is defined.

**Verification:**

- [ ] `make test-contract SERVICE=block-storage`

**Dependencies:** CS-0203, CS-0302
**Parallel batch:** B4
**Estimated scope:** S

### Checkpoint P4

- [ ] Compute, network, object storage, and block storage contracts exist.
- [ ] Contract tests pass.
- [ ] Consumers can start implementation without changing contract envelopes.

## 12. Phase 5: Core IaaS Implementations

### CS-0402: Placement service

**Path envelope:** `cloud/control-plane/placement/`

**Description:** Implement placement rules for host, zone, cell, capacity pool,
anti-affinity, resource shape, and health.

**Acceptance criteria:**

- [ ] Placement excludes unhealthy cells and hosts.
- [ ] Placement respects account, region, zone, capacity, and anti-affinity.
- [ ] Placement returns deterministic reasons for denial.

**Verification:**

- [ ] `make test SERVICE=placement`
- [ ] `make test-integration SERVICE=placement SCENARIO=cell-failure`

**Dependencies:** CS-0201, CS-0401
**Parallel batch:** B5
**Estimated scope:** M

### CS-0403: Compute host agent

**Path envelope:** `cloud/data-plane/host-agent/`

**Description:** Implement host-local capacity reporting, health reporting,
VM lifecycle command interface, metadata attachment, and reconciliation loop.

**Acceptance criteria:**

- [ ] Host agent reports capacity and health.
- [ ] Host agent accepts start/stop/terminate command envelope.
- [ ] Host agent reconciliation is idempotent.

**Verification:**

- [ ] `make test SERVICE=host-agent`
- [ ] `make test-integration SERVICE=host-agent SCENARIO=restart-reconcile`

**Dependencies:** CS-0402
**Parallel batch:** B6
**Estimated scope:** M

### CS-0404: Compute API vertical slice

**Path envelope:** `cloud/services/compute/src/`

**Description:** Implement create/describe/terminate VM control-plane flow using
IAM, quota, placement, audit, metering, and host-agent command interfaces.

**Acceptance criteria:**

- [ ] Create VM validates IAM and quota.
- [ ] Create VM records idempotency key and lifecycle state.
- [ ] Terminate VM is idempotent and emits audit/metering events.

**Verification:**

- [ ] `make test SERVICE=compute`
- [ ] `make test-integration SERVICE=compute SCENARIO=create-describe-terminate`

**Dependencies:** CS-0302, CS-0303, CS-0304, CS-0403
**Parallel batch:** B6b
**Estimated scope:** M

### CS-0502: VPC and subnet vertical slice

**Path envelope:** `cloud/services/network/src/vpc/`

**Description:** Implement VPC, subnet, route table, security group, and network
interface lifecycle with default-deny security groups.

**Acceptance criteria:**

- [ ] Account can create VPC and subnet.
- [ ] Security group denies inbound by default.
- [ ] Network interface can be created in subnet.

**Verification:**

- [ ] `make test SERVICE=network`
- [ ] `make test-integration SERVICE=network SCENARIO=vpc-subnet-eni`

**Dependencies:** CS-0501, CS-0302, CS-0303
**Parallel batch:** B5
**Estimated scope:** M

### CS-0503: Load balancer and DNS vertical slice

**Path envelope:** `cloud/services/network/src/load-balancer-dns/`

**Description:** Implement L4 load balancer, target groups, target health,
regional DNS record, and basic failout of unhealthy targets.

**Acceptance criteria:**

- [ ] Load balancer can register targets.
- [ ] Failed target is removed from rotation.
- [ ] DNS record resolves to load balancer endpoint.

**Verification:**

- [ ] `make test SERVICE=network`
- [ ] `make test-integration SERVICE=network SCENARIO=lb-target-failure`

**Dependencies:** CS-0502
**Parallel batch:** B6
**Estimated scope:** M

### CS-0603: Object storage metadata vertical slice

**Path envelope:** `cloud/services/object-storage/src/metadata/`

**Description:** Implement bucket and object metadata, bucket policy checks,
object listing, versioning placeholder, audit, and metering.

**Acceptance criteria:**

- [ ] Create bucket is idempotent.
- [ ] Put/get/list/delete object metadata works under bucket policy.
- [ ] List objects is paginated.

**Verification:**

- [ ] `make test SERVICE=object-storage`
- [ ] `make test-integration SERVICE=object-storage SCENARIO=bucket-object-lifecycle`

**Dependencies:** CS-0601, CS-0302, CS-0303, CS-0304
**Parallel batch:** B5
**Estimated scope:** M

### CS-0604: Object storage data path slice

**Path envelope:** `cloud/services/object-storage/src/data-path/`

**Description:** Implement local/dev object data path, object checksum,
range-read placeholder, encryption envelope placeholder, and durability hooks.

**Acceptance criteria:**

- [ ] Put object stores bytes and checksum.
- [ ] Get object verifies checksum.
- [ ] Delete object removes data path reference without breaking audit history.

**Verification:**

- [ ] `make test SERVICE=object-storage`
- [ ] `make test-integration SERVICE=object-storage SCENARIO=object-data-path`

**Dependencies:** CS-0603
**Parallel batch:** B6
**Estimated scope:** M

### CS-0605: Block volume vertical slice

**Path envelope:** `cloud/services/block-storage/src/`

**Description:** Implement volume create, attach, detach, snapshot metadata,
restore metadata, encryption placeholder, and VM attachment handshake.

**Acceptance criteria:**

- [ ] Encrypted volume can be created in zone/cell.
- [ ] Volume can attach to and detach from VM.
- [ ] Snapshot can restore into a new volume metadata record.

**Verification:**

- [ ] `make test SERVICE=block-storage`
- [ ] `make test-integration SERVICE=block-storage SCENARIO=attach-snapshot-restore`

**Dependencies:** CS-0602, CS-0404
**Parallel batch:** B6 after CS-0404 API is stable
**Estimated scope:** M

### Checkpoint P5

- [ ] VM create/describe/terminate works in a dev cell.
- [ ] VPC, subnet, ENI, load balancer, and DNS baseline works.
- [ ] Object bucket/object lifecycle works.
- [ ] Block volume attach and snapshot metadata works.
- [ ] All customer-visible mutations emit audit and metering events.

## 13. Phase 6: Managed Kubernetes

### CS-0701: Managed Kubernetes API contract

**Path envelope:** `cloud/services/kubernetes/contracts/`

**Description:** Define managed cluster, node pool, Kubernetes version, add-on,
upgrade, network, IAM, observability, and billing contracts.

**Acceptance criteria:**

- [ ] Create/describe/delete cluster APIs are defined.
- [ ] Node pool lifecycle API is defined.
- [ ] Upgrade and rollback contract is defined.

**Verification:**

- [ ] `make test-contract SERVICE=kubernetes`

**Dependencies:** CS-0401, CS-0501, CS-0602
**Parallel batch:** B6
**Estimated scope:** S

### CS-0702: Managed cluster lifecycle vertical slice

**Path envelope:** `cloud/services/kubernetes/src/`

**Description:** Implement create cluster, create node pool, deploy add-ons,
emit telemetry, and delete cluster using compute, VPC, and block storage
interfaces.

**Acceptance criteria:**

- [ ] Account can create a managed cluster in a VPC.
- [ ] Cluster creates node pool and emits cluster state.
- [ ] Delete cluster drains and releases resources.

**Verification:**

- [ ] `make test SERVICE=kubernetes`
- [ ] `make test-integration SERVICE=kubernetes SCENARIO=create-deploy-delete`

**Dependencies:** CS-0404, CS-0502, CS-0605, CS-0701
**Parallel batch:** B7a
**Estimated scope:** M

### Checkpoint P6

- [ ] Managed cluster create/deploy/delete works in dev cell.
- [ ] Kubernetes service integrates with account, IAM, VPC, compute, block
  storage, audit, and metering.

## 14. Phase 7: Hyperscaler-Class Quality Gates

### CS-0801: Observability and SLO platform

**Path envelope:** `cloud/platform/observability-kit/`

**Description:** Implement telemetry conventions, service dashboards, SLO
definition format, burn-rate alert templates, trace/log/metric correlation, and
customer impact view.

**Acceptance criteria:**

- [ ] Services can declare SLIs and SLOs.
- [ ] Dashboards link service, region, cell, version, and owner.
- [ ] Burn-rate alert template routes to owner.

**Verification:**

- [ ] `make test SERVICE=observability-kit`
- [ ] `make launch-check SERVICE=sample-control-plane REGION=dev-1 CELL=cell-a`

**Dependencies:** CS-0103, CS-0303
**Parallel batch:** B7a
**Estimated scope:** M

### CS-0802: Security and vulnerability program

**Path envelope:** `cloud/platform/security-kit/`

**Description:** Implement vulnerability inventory, dependency inventory,
severity SLA, exception process, secret scanning, policy checks, and emergency
patch workflow.

**Acceptance criteria:**

- [ ] Critical vulnerability maps to service owner and artifact.
- [ ] Exception requires owner, expiry, and mitigation.
- [ ] Secret scan failure blocks promotion.

**Verification:**

- [ ] `make test-security`
- [ ] `make launch-check SERVICE=sample-control-plane REGION=dev-1 CELL=cell-a`

**Dependencies:** CS-0104, CS-0103
**Parallel batch:** B7a
**Estimated scope:** M

### CS-0803: Backup, restore, and DR harness

**Path envelope:** `cloud/platform/dr-harness/`

**Description:** Implement backup policy declarations, restore drill runner,
RTO/RPO evidence format, DR pairing registry, and stateful service drill
templates.

**Acceptance criteria:**

- [ ] Stateful service can declare backup policy and RTO/RPO.
- [ ] Restore drill emits evidence with pass/fail and timing.
- [ ] DR pairing registry validates region/cell pairing.

**Verification:**

- [ ] `make test SERVICE=dr-harness`
- [ ] `make test-integration SERVICE=dr-harness SCENARIO=metadata-restore`

**Dependencies:** CS-0201, CS-0303
**Parallel batch:** B7a
**Estimated scope:** M

### CS-0804: Performance, scalability, and availability harness

**Path envelope:** `cloud/platform/quality-harness/`

**Description:** Implement standard load, stress, soak, chaos, capacity,
cell-addition, shard-addition, and availability drill definitions.

**Acceptance criteria:**

- [ ] Service can declare p50/p90/p99/p999 objectives.
- [ ] Harness supports 1x, 2x, and failure-mode load scenarios.
- [ ] Harness records pass/fail evidence for cell failure and dependency
  impairment.

**Verification:**

- [ ] `make test-load SERVICE=sample-control-plane`
- [ ] `make test-chaos SERVICE=sample-control-plane SCENARIO=dependency-impairment`

**Dependencies:** CS-0201, CS-0104
**Parallel batch:** B7a
**Estimated scope:** M

### CS-0805: Overload, fairness, and load-shedding evidence kit

**Path envelope:** `cloud/platform/overload-kit/`

**Description:** Implement standard overload-test scenarios, request-cost
classification, priority classes, fairness policy, retry-budget checks, bounded
queue checks, and load-shedding evidence format for public APIs and internal
service hops.

**Acceptance criteria:**

- [ ] Services can declare admission-control, fairness, queue, and load-shed
      policies.
- [ ] Overload test proves useful work continues while excess work is rejected
      with observable reason codes.
- [ ] Retry budgets and client backoff behavior are checked during overload.

**Verification:**

- [ ] `make test-overload SERVICE=sample-control-plane`
- [ ] `make launch-check SERVICE=sample-control-plane EVIDENCE=overload-fairness`

**Dependencies:** CS-0202, CS-0801
**Parallel batch:** B7b
**Estimated scope:** M

### CS-0806: Shuffle-sharded tenant isolation kit

**Path envelope:** `cloud/platform/isolation-kit/`

**Description:** Implement tenant/resource assignment policy, shuffle-shard
modeling, correlated-blast-radius analysis, noisy-neighbor drills, and
isolation evidence for shared control-plane queues, workers, cells, and shards.

**Acceptance criteria:**

- [ ] Tenant assignment model can map accounts/resources to cells, shards,
      queues, and workers.
- [ ] Drill proves one synthetic tenant cannot create regional or service-wide
      impairment.
- [ ] Correlated-impact report identifies shared dependencies that defeat
      isolation.

**Verification:**

- [ ] `make test-isolation SERVICE=sample-control-plane SCENARIO=noisy-neighbor`
- [ ] `make launch-check SERVICE=sample-control-plane EVIDENCE=tenant-isolation`

**Dependencies:** CS-0201, CS-0402, CS-0804
**Parallel batch:** B7b
**Estimated scope:** M

### CS-0807: Privacy and data-governance control pack

**Path envelope:** `cloud/platform/privacy-kit/`

**Description:** Implement service-level personal-data inventory, residency,
retention, deletion, export, legal-hold, log/telemetry redaction, and backup
retention declarations with launch-gate validation.

**Acceptance criteria:**

- [ ] Every service can declare personal-data categories, retention, residency,
      deletion, export, and backup behavior.
- [ ] Deletion/export tests produce evidence for at least one sample account.
- [ ] Logs, traces, metrics, and audit events have redaction/classification
      checks.

**Verification:**

- [ ] `make test-privacy SERVICE=sample-control-plane`
- [ ] `make launch-check SERVICE=sample-control-plane EVIDENCE=privacy-data-governance`

**Dependencies:** CS-0301, CS-0303, CS-0802
**Parallel batch:** B7b
**Estimated scope:** M

### CS-0808: Automated canary and production-readiness gate

**Path envelope:** `cloud/platform/release-safety-kit/`

**Description:** Implement design-time production-readiness review templates,
one-cell canary orchestration, automated canary evaluation, rollback triggers,
and post-rollout observation evidence.

**Acceptance criteria:**

- [ ] Service launch requires PRR evidence before private preview.
- [ ] Canary evaluation compares release candidate against control for errors,
      latency, saturation, cost, and customer-impact signals.
- [ ] Failed canary automatically blocks rollout and produces rollback evidence.

**Verification:**

- [ ] `make test-canary SERVICE=sample-control-plane`
- [ ] `make launch-check SERVICE=sample-control-plane EVIDENCE=canary-prr`

**Dependencies:** CS-0104, CS-0801, CS-0804
**Parallel batch:** B7b
**Estimated scope:** M

### CS-0809: FOCUS-compatible cost and usage export

**Path envelope:** `cloud/platform/cost-data-kit/`

**Description:** Implement cost-and-usage export schema, validation, allocation
dimensions, tag normalization, invoice reconciliation hooks, and unit-cost
dashboard feeds aligned to the selected FOCUS version.

**Acceptance criteria:**

- [ ] Export includes account, service, region, resource, SKU, usage unit, tags,
      discounts, credits, and allocation dimensions.
- [ ] Export validation fails incompatible records before billing publication.
- [ ] Unit-cost dashboard can consume the same export used for billing
      reconciliation.

**Verification:**

- [ ] `make test-cost-data SERVICE=metering-billing`
- [ ] `make launch-check SERVICE=metering-billing EVIDENCE=focus-cost-export`

**Dependencies:** CS-0304
**Parallel batch:** B7b
**Estimated scope:** M

### CS-0810: Kubernetes runtime-hardening policy gate

**Path envelope:** `cloud/platform/kubernetes-policy-kit/`

**Description:** Implement managed-Kubernetes and platform-cluster policy gates
for Pod Security Standards, namespace isolation, least-privilege RBAC, network
policy, image provenance, admission controls, and exception expiry.

**Acceptance criteria:**

- [ ] Restricted or baseline Pod Security Standard is enforced according to
      workload class.
- [ ] Privileged workload exceptions require owner, expiry, mitigation, and
      audit event.
- [ ] Non-compliant images, RBAC, or network policies block promotion.

**Verification:**

- [ ] `make test-k8s-policy SERVICE=kubernetes`
- [ ] `make launch-check SERVICE=kubernetes EVIDENCE=runtime-hardening`

**Dependencies:** CS-0702, CS-0802
**Parallel batch:** B7b
**Estimated scope:** M

### CS-0811: Abuse, fraud, and DDoS readiness kit

**Path envelope:** `cloud/platform/abuse-defense-kit/`

**Description:** Implement abuse/fraud/DDoS readiness scenarios, signup and
payment risk hooks, quota throttles, suspension and appeal workflows, ingress
protection evidence, and customer-safe communication templates.

**Acceptance criteria:**

- [ ] Abuse scenario can throttle or suspend synthetic bad actors without
      harming unrelated tenants.
- [ ] DDoS drill records ingress protection, load shedding, customer impact,
      and status communication evidence.
- [ ] Appeals and false-positive review workflow exists for preview customers.

**Verification:**

- [ ] `make launch-check SERVICE=abuse-defense EVIDENCE=abuse-ddos-drill`
- [ ] `make test-integration SERVICE=abuse-defense SCENARIO=synthetic-abuse`

**Dependencies:** CS-0202, CS-0301, CS-0304, CS-0503, CS-0801
**Parallel batch:** B7b
**Estimated scope:** M

### Checkpoint P7

- [ ] Observability kit works.
- [ ] Security kit works.
- [ ] DR harness works.
- [ ] Performance/scalability/availability harness works.
- [ ] Overload/fairness, tenant-isolation, privacy, canary/PRR, FOCUS cost-data,
  Kubernetes runtime-hardening, and abuse/DDoS kits work.
- [ ] Every P5/P6 service has at least placeholder evidence for all seven
  hyperscaler-class bars.

## 15. Phase 8: Developer Experience And Internal Preview

### CS-0901: Cloud console and CLI baseline

**Path envelope:** `cloud/developer-experience/console-cli/`

**Description:** Implement minimal console and CLI flows for account, VPC, VM,
bucket, volume, and cluster lifecycle.

**Acceptance criteria:**

- [ ] CLI can create account-scoped resources through public APIs.
- [ ] Console can list accounts, regions, VMs, VPCs, buckets, volumes, and
  clusters.
- [ ] Console/CLI never bypass service APIs.

**Verification:**

- [ ] `make test SERVICE=console-cli`
- [ ] `make test-integration SERVICE=console-cli SCENARIO=core-resource-flow`

**Dependencies:** Checkpoint P6
**Parallel batch:** B8
**Estimated scope:** M

### CS-0902: SDK and IaC provider baseline

**Path envelope:** `cloud/developer-experience/sdk-iac/`

**Description:** Generate SDK and IaC provider baseline for account, network,
compute, object storage, block storage, and Kubernetes contracts.

**Acceptance criteria:**

- [ ] SDK is generated from contracts, not hand-coded drift.
- [ ] IaC provider can create VPC, VM, bucket, volume, and cluster in dev.
- [ ] Examples compile or validate.

**Verification:**

- [ ] `make test-contract`
- [ ] `make test-integration SERVICE=sdk-iac SCENARIO=iac-core-resources`

**Dependencies:** Checkpoint P6
**Parallel batch:** B8
**Estimated scope:** M

### CS-0903: Internal dogfood workload onboarding

**Path envelope:** `cloud/preview/internal-dogfood/`

**Description:** Define and execute internal workload onboarding plan,
guardrails, quotas, SLOs, support path, incident path, and feedback capture.

**Acceptance criteria:**

- [ ] At least three internal workload templates are defined.
- [ ] Each workload has quota, SLO, owner, cost allocation, and rollback plan.
- [ ] Feedback and incident capture path exists.

**Verification:**

- [ ] `make launch-check SERVICE=internal-dogfood REGION=dev-1 CELL=cell-a`

**Dependencies:** Checkpoint P7, CS-0901, CS-0902
**Parallel batch:** B8 after CS-0901/CS-0902 are stable
**Estimated scope:** M

### Checkpoint P8

- [ ] Console, CLI, SDK, and IaC baseline work against service APIs.
- [ ] Internal dogfood plan is launch-checkable.
- [ ] At least one internal workload can run through onboarding rehearsal.

## 16. Phase 9: Private Customer Preview

### CS-1001: Customer onboarding and support readiness

**Path envelope:** `cloud/preview/customer-onboarding/`

**Description:** Implement private preview onboarding flow, account creation
runbook, quota package, support runbook, incident communication, and customer
responsibility matrix.

**Acceptance criteria:**

- [ ] Preview customer can be onboarded without ad hoc privileged steps.
- [ ] Support path is staffed and documented.
- [ ] Customer responsibility matrix is visible.

**Verification:**

- [ ] `make launch-check SERVICE=customer-onboarding REGION=dev-1 CELL=cell-a`

**Dependencies:** Checkpoint P8
**Parallel batch:** B9
**Estimated scope:** M

### CS-1002: Status page and incident communications

**Path envelope:** `cloud/services/support-status/`

**Description:** Implement customer-visible status, incident timeline,
communication templates, region/service health, and post-incident report
publishing workflow.

**Acceptance criteria:**

- [ ] Status can be scoped by service and region.
- [ ] Incident timeline records updates and resolution.
- [ ] Customer communication template exists for preview incidents.

**Verification:**

- [ ] `make test SERVICE=support-status`
- [ ] `make test-integration SERVICE=support-status SCENARIO=incident-update`

**Dependencies:** CS-0801
**Parallel batch:** B9
**Estimated scope:** M

### CS-1003: Preview launch gate automation

**Path envelope:** `cloud/platform/launch-gates/`

**Description:** Implement evidence-backed launch gate checks for product,
architecture, security, SRE, support, billing, performance, reliability,
accountability, observability, scalability, availability, and optimization.

**Acceptance criteria:**

- [ ] Launch gate checks every required evidence pack.
- [ ] Missing evidence blocks preview.
- [ ] Risk acceptance requires owner and expiry.

**Verification:**

- [ ] `make launch-check SERVICE=compute REGION=dev-1 CELL=cell-a`
- [ ] `make launch-check SERVICE=kubernetes REGION=dev-1 CELL=cell-a`

**Dependencies:** Checkpoint P7, CS-1001
**Parallel batch:** B9 after CS-1001 support requirements are known
**Estimated scope:** M

### Checkpoint P9

- [ ] Private preview onboarding is repeatable.
- [ ] Status and incident communications are ready.
- [ ] Launch gates block missing evidence.
- [ ] Private preview can proceed by explicit approval.

## 17. Phase 10: Public Preview And GA Hardening

### CS-1101: Public preview self-service onboarding

**Path envelope:** `cloud/public-preview/self-service-onboarding/`

**Description:** Implement public-preview account signup, region selection,
default quotas, payment/billing handoff, support routing, and abuse prevention.

**Acceptance criteria:**

- [ ] New customer can create account and enable first region without internal
  manual action.
- [ ] Default quotas and abuse throttles apply.
- [ ] Billing handoff records customer billing state.

**Verification:**

- [ ] `make test-integration SERVICE=self-service-onboarding SCENARIO=new-customer`

**Dependencies:** Checkpoint P9
**Parallel batch:** Post-private-preview
**Estimated scope:** M

### CS-1102: GA reliability evidence pack

**Path envelope:** `cloud/ga/evidence/reliability/`

**Description:** Produce and validate GA reliability evidence: SLO history,
error budget, incidents, game days, RTO/RPO drills, zone/cell failure drills,
and rollback drills.

**Acceptance criteria:**

- [ ] SLO history meets GA threshold.
- [ ] Restore and rollback drills pass.
- [ ] No unowned critical reliability risks remain.

**Verification:**

- [ ] `make launch-check STAGE=ga EVIDENCE=reliability`

**Dependencies:** Checkpoint P9
**Parallel batch:** Post-private-preview
**Estimated scope:** S

### CS-1103: GA security and compliance evidence pack

**Path envelope:** `cloud/ga/evidence/security-compliance/`

**Description:** Produce and validate GA security/compliance evidence: threat
models, vulnerability SLAs, policy checks, supply-chain evidence, audit coverage,
customer responsibility, and risk acceptances.

**Acceptance criteria:**

- [ ] Threat models are complete for GA services.
- [ ] Critical vulnerabilities have no expired exceptions.
- [ ] Audit coverage is complete for customer-visible mutations.

**Verification:**

- [ ] `make launch-check STAGE=ga EVIDENCE=security-compliance`

**Dependencies:** Checkpoint P9
**Parallel batch:** Post-private-preview
**Estimated scope:** S

### CS-1104: GA performance, scalability, and optimization evidence pack

**Path envelope:** `cloud/ga/evidence/performance-scale-optimization/`

**Description:** Produce and validate GA performance, scalability, availability,
cost, and optimization evidence: 2x scale tests, tail latency, shard addition,
cell addition, unit cost, capacity forecast, and optimization backlog.

**Acceptance criteria:**

- [ ] 2x launch-scale test passes.
- [ ] p99/p999 latency targets pass under normal and impaired dependency cases.
- [ ] Unit-cost dashboard and 12-month capacity forecast exist.

**Verification:**

- [ ] `make launch-check STAGE=ga EVIDENCE=performance-scale-optimization`

**Dependencies:** Checkpoint P9
**Parallel batch:** Post-private-preview
**Estimated scope:** S

### Checkpoint P10

- [ ] Public-preview onboarding works.
- [ ] GA reliability evidence passes.
- [ ] GA security/compliance evidence passes.
- [ ] GA performance/scale/optimization evidence passes.
- [ ] Executive launch review can decide based on evidence.

## 18. Disjoint Worktree Assignment Guide

| Agent lane | Recommended changesets | Path envelopes |
|---|---|---|
| Audit/plan hardening | CSA-0001, CSA-0002, CSA-0003, CSA-0004 | `plan/` only |
| Program/TPM | CS-0001, CS-0002, CS-0003, CS-1001, CS-1102, CS-1103, CS-1104 | `cloud/plans/`, `cloud/preview/`, `cloud/ga/` |
| Platform factory | CS-0101, CS-0102, CS-0103, CS-0104 | `cloud/platform/` except subpaths reserved later |
| Region substrate | CS-0201, CS-0402 | `cloud/control-plane/region-metadata/`, `cloud/control-plane/placement/` |
| Trust foundation | CS-0301, CS-0302, CS-0303, CS-0304 | `cloud/services/account/`, `iam/`, `audit/`, `metering-billing/` |
| Compute | CS-0401, CS-0403, CS-0404 | `cloud/services/compute/`, `cloud/data-plane/host-agent/` |
| Networking | CS-0501, CS-0502, CS-0503 | `cloud/services/network/` |
| Storage | CS-0601, CS-0602, CS-0603, CS-0604, CS-0605 | `cloud/services/object-storage/`, `block-storage/` |
| Kubernetes | CS-0701, CS-0702 | `cloud/services/kubernetes/` |
| Ops/quality | CS-0801, CS-0802, CS-0803, CS-0804, CS-1002, CS-1003 | `cloud/platform/*-kit/`, `cloud/services/support-status/`, `cloud/platform/launch-gates/` |
| Quality hardening | CS-0805, CS-0806, CS-0807, CS-0808, CS-0809, CS-0810, CS-0811 | `cloud/platform/overload-kit/`, `isolation-kit/`, `privacy-kit/`, `release-safety-kit/`, `cost-data-kit/`, `kubernetes-policy-kit/`, `abuse-defense-kit/` |
| Developer experience | CS-0901, CS-0902 | `cloud/developer-experience/` |

## 19. Change Submission Template

Each changeset should submit with:

```markdown
Title: <CS-ID>: <imperative summary>

Scope:
- Phase: <phase>
- Path envelope: <path>
- Dependencies completed: <ids>

Acceptance:
- [ ] <criterion>
- [ ] <criterion>

Verification:
- [ ] <command and result>

Non-goals:
- <explicitly not touched>

Parallel safety:
- Does not touch path envelopes owned by: <ids>
```

## 20. Phase-Level Review Gates

| Phase | Review gate |
|---|---|
| PA | Main-plan audit, source-version hygiene, traceability, and Definition of Done are complete |
| P0 | Strategy, ownership, and governance are coherent |
| P1 | New service can be generated, built, tested, cataloged, and signed |
| P2 | Region/cell/quota/common contract foundations are stable |
| P3 | Account/IAM/audit/metering/billing trust loop works |
| P4 | Core IaaS contracts are stable enough for implementations |
| P5 | VM, VPC, LB/DNS, object, and block storage vertical slices work |
| P6 | Managed Kubernetes integrates core IaaS |
| P7 | Seven hyperscaler-class evidence bars plus overload, isolation, privacy, canary/PRR, cost-data, runtime-hardening, and abuse/DDoS kits have harnesses |
| P8 | Internal dogfood can onboard through API/console/CLI/SDK/IaC |
| P9 | Private preview is supportable and evidence-gated |
| P10 | Public preview and GA decision can be made from evidence |

## 21. Open Questions For Human Review

1. Should first implementation standardize on Rust, Go, or a mixed standard by
   service family?
2. Should object storage and block storage be built immediately or wrapped
   behind pluggable backends during first internal preview?
3. Should public-preview self-service onboarding be in scope before first
   private customer preview?
4. What is the first target launch region and regulatory posture?
5. Which evidence packs are mandatory for private preview versus GA?
