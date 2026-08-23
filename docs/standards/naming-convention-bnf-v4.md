---
doc_class: Standard
title: Naming Convention BNF v4 Standard
status: Accepted
date: 2026-05-20
owner: council-architecture + axis-foundry
related_oyatie_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0115
  - ADR-0131
enforced_by:
  - governance-naming-convention
  - governance-layered-architecture
  - governance-flat-crates
canonical_paths:
  - docs/standards/crate-naming-convention.md
  - docs/standards/layer-enum-adr-0105.md
  - Cargo.toml
  - registry/catalog/
---

# Naming Convention BNF v4 Standard

This standard is the canonical authoring rule for names that must be parsed by
agents, retired VCS ratchet, CI lanes, catalog validators, and future automation. It expands
the older crate-specific grammar in `docs/standards/crate-naming-convention.md`
into a cross-artifact BNF v4 discipline for crates, capability records, contract
files, policies, events, SLOs, runbooks, dashboards, and implementation plans.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every durable identifier that crosses a repository,
runtime, audit, or documentation boundary.

It covers Rust crates under `crates/`.

It covers catalog records under `registry/catalog/`.

It covers capability tier records under `registry/capability-tiers/`.

It covers microservice-local catalogs under `microservices/<ms>/catalog/`.

It covers OpenAPI files under `contracts/` and `microservices/<ms>/contracts/`.

It covers AsyncAPI files under `microservices/<ms>/contracts/`.

It covers proto3 files under `microservices/<ms>/contracts/`.

It covers Cedar files under `microservices/<ms>/policy/`.

It covers OpenSLO files under `microservices/<ms>/slos/`.

It covers implementation plans under `microservices/<ms>/IP-*.md`.

It covers ADR filenames under `docs/decisions/` and `microservices/<ms>/decisions/`.

It does not rename a legacy artifact by itself.

It does not authorize blanket search-and-replace migrations.

It does not override ADR-0105 layer semantics.

It does not relax the `oyatie-` crate prefix.

It does not permit compatibility aliases without a tombstone or redirect entry.

## Normative Requirements

N-001. Every durable identifier MUST be lowercase kebab-case unless the host format requires another separator.

N-002. Every repository-owned crate MUST start with `oyatie-`.

N-003. Every repository-owned checker crate MUST start with `check-`.

N-004. Every non-checker crate MUST include a registered microservice token immediately after `oyatie-`.

N-005. Every non-checker crate MUST end with an ADR-0105 layer token.

N-006. Every crate layer token MUST be one of the values named in `layer-enum-adr-0105.md`.

N-007. Every crate directory name MUST match `[package].name`.

N-008. Every library target name MUST equal the package name with hyphens converted to underscores.

N-009. Every catalog record filename MUST equal its declared artifact id plus `.yaml`.

N-010. Every microservice directory token MUST match the microservice registry spelling.

N-011. Every bounded-context token SHOULD be present when a microservice has more than one domain concept at the same layer.

N-012. A bounded-context token MUST NOT smuggle a provider name into a kernel crate.

N-013. Provider names MUST appear only in adapter, contract, policy, or deployment artifacts.

N-014. Capability tier identifiers MUST use `<microservice>.<capability>.<tier>`.

N-015. Capability tier file names MUST use `<microservice>-<capability>-<tier>.yaml`.

N-016. Public REST contract files MUST use `<surface>-v<major>.openapi.yaml`.

N-017. Async contract files MUST use `<surface>-v<major>.asyncapi.yaml`.

N-018. Proto contract files MUST use `<bounded-context>-v<major>.proto`.

N-019. OpenSLO files MUST use `<sli>.openslo.yaml`.

N-020. Cedar files MUST use the archetype names required by `cedar-policy-authoring.md`.

N-021. Audit event names MUST use `EVT-<DOMAIN>-<ACTION>-V<MAJOR>`.

N-022. Event schema ids MUST use `<domain>.<event>.v<major>`.

N-023. Runbook names MUST describe the failure or operation, not the owning team.

N-024. Dashboard names MUST include the microservice and signal family.

N-025. Implementation plan names MUST start with `IP-` or `IP-journey-`.

N-026. ADR filenames MUST start with `ADR-` followed by a zero-padded id or local service prefix.

N-027. Human display names MAY use title case but MUST map to one machine id.

N-028. Machine ids MUST NOT include spaces.

N-029. Machine ids MUST NOT include underscores unless the host format forbids hyphens.

N-030. Machine ids MUST NOT include organizational names that ADR-0284 says are indirected.

N-031. Machine ids MUST NOT encode temporary phase names.

N-032. Machine ids MUST NOT encode implementation language except in toolchain artifacts.

N-033. Machine ids MUST NOT encode cloud vendor unless the artifact is a vendor adapter.

N-034. Machine ids MUST NOT use `shared` as a microservice token.

N-035. Machine ids MUST NOT use `platform` as a microservice token.

N-036. Machine ids MUST NOT use `vertical` as a microservice token.

N-037. Machine ids MUST use `workflow-engine` for the runtime service and `workflow-studio` for the UX builder.

N-038. Machine ids MUST use `ontology` for the projection substrate.

N-039. Machine ids MUST use `policy-engine` for Cedar substrate services.

N-040. Machine ids MUST use `regional-pack` for jurisdiction overlay services.

N-041. A rename MUST include an ADR or explicit migration playbook.

N-042. A rename MUST include old-to-new mapping evidence.

N-043. A rename MUST include catalog record updates.

N-044. A rename MUST include CI lane fixture updates.

N-045. A rename MUST include documentation cross-reference updates.

N-046. A rename MUST NOT silently recreate retired roots such as `services/` or `modules/`.

N-047. A compatibility alias MAY exist only with an expiration date.

N-048. A compatibility alias MUST name the successor id.

N-049. A compatibility alias MUST be validated by `governance-retired-paths`.

N-050. A compatibility alias MUST NOT be used by new artifacts.

N-051. Each id grammar MUST be documented before enforcement promotes to BLOCKER.

N-052. Each id grammar MUST include accepted and rejected examples.

N-053. Each id grammar MUST cite the owning ADR.

N-054. Each id grammar MUST cite the owning checker.

N-055. Each id grammar MUST include a reverse-dependency search pattern.

N-056. Every checker diagnostic MUST print the invalid id, expected grammar, and owning standard.

N-057. Every checker diagnostic SHOULD suggest a valid replacement.

N-058. Every checker diagnostic MUST avoid guessing intent across bounded contexts.

N-059. Names used in code comments MUST match the canonical id spelling.

N-060. Names used in docs MUST match the canonical id spelling unless explicitly discussing legacy.

N-061. Names used in diagrams MUST match the canonical id spelling.

N-062. Names used in telemetry attributes MUST match the canonical id spelling.

N-063. Names used in Cedar entities MUST match the canonical id spelling.

N-064. Names used in OpenAPI tags MUST match the canonical id spelling.

N-065. Names used in AsyncAPI channel addresses MUST match the canonical id spelling.

N-066. Names used in proto package names MUST use dot-separated equivalents of canonical ids.

N-067. Names used in Prometheus labels MUST use underscore-separated equivalents only where Prometheus requires it.

N-068. Names used in Grafana dashboard uids MUST use the canonical id without punctuation drift.

N-069. Names used in OpenBao paths MUST mirror microservice and capability tier ids.

N-070. Names used in Kubernetes labels MUST use canonical ids and DNS-safe compression.

N-071. A generated artifact MUST identify the generator and source id.

N-072. A generated artifact MUST NOT invent a new id namespace.

N-073. A human-authored artifact MUST NOT depend on unstated generator magic.

N-074. A product label MUST map to a capability tier id before it appears in a public contract.

N-075. A tenant-visible label MUST be localizable without changing the machine id.

N-076. A region or pack suffix MUST appear only when the artifact is actually pack-specific.

N-077. A date suffix SHOULD appear only on snapshots, audits, or evidence files.

N-078. A version suffix MUST use `v<major>` for contracts and `-<semver>` only for release bundles.

N-079. An id that is serialized into audit history MUST be immutable.

N-080. An id that changes behavior MUST receive a new major version or migration record.

## Worked Examples

### Example 1: Valid crate name

Input:

```toml
[package]
name = "workflow-engine-state-machine-domain"

[lib]
name = "workflow_engine_state_machine_domain"

[package.metadata.oya]
microservice = "workflow-engine"
bounded_context = "state-machine"
layer = "domain"
```

Why it passes:

The prefix is `oya`.

The microservice token is `workflow-engine`.

The bounded context is `state-machine`.

The layer token is `domain`.

The lib target maps hyphen to underscore.

The dependency graph is checked by `governance-layered-architecture`.

### Example 2: Invalid provider leak

Input:

```toml
[package]
name = "cloud-aws-kms-kernel"
```

Why it fails:

`aws` is a provider token.

Provider tokens do not belong in kernel crates.

The valid kernel is `cloud-kms-kernel`.

The AWS provider implementation belongs in `cloud-kms-adapter-aws`.

The catalog record belongs at `registry/catalog/cloud-kms-adapter-aws.yaml`.

### Example 3: Contract file naming

Valid files:

```text
microservices/workflow-engine/contracts/state-machine-v1.openapi.yaml
microservices/workflow-engine/contracts/state-machine-v1.asyncapi.yaml
microservices/workflow-engine/contracts/state-machine-v1.proto
```

Invalid files:

```text
microservices/workflow-engine/contracts/api.yaml
microservices/workflow-engine/contracts/events.yaml
microservices/workflow-engine/contracts/workflow.proto
```

The invalid files hide version and bounded-context ownership.

### Example 4: Cedar archetype naming

Valid policy files:

```text
microservices/tenancy/policy/tenant-scope.cedar
microservices/tenancy/policy/ci-scope.cedar
microservices/tenancy/policy/auditor-scope.cedar
microservices/tenancy/policy/public-read.cedar
```

Invalid policy files:

```text
microservices/tenancy/policy/auth.cedar
microservices/tenancy/policy/rbac.cedar
microservices/tenancy/policy/main.cedar
```

The invalid names conceal the evaluation context.

### Example 5: Capability tier identifier

Valid record:

```yaml
id: workflow-engine.approval-routing.professional
file_name: workflow-engine-approval-routing-professional.yaml
contributes:
  ontology_projection: ApprovalRoute
  workflow_template: approval-routing-v1
  cedar_policy: workflow-engine-approval-routing-professional.cedar
```

The id states service, capability, and tier without creating a product silo.

## Verification

The primary checker is `governance-naming-convention`.

The crate layer checker is `governance-layered-architecture`.

The catalog checker is `governance-catalog-id-discipline`.

The contract checker is `governance-contract-name-shape`.

The Cedar checker is `governance-cedar-file-shape`.

The SLO checker is `governance-openslo-conformance`.

The ADR checker is `governance-adr-shape`.

The retired path checker is `governance-retired-paths`.

The naming checker MUST scan `Cargo.toml`.

The naming checker MUST scan `crates/*/Cargo.toml`.

The naming checker MUST scan `registry/catalog/*.yaml`.

The naming checker MUST scan `registry/capability-tiers/*.yaml`.

The naming checker MUST scan `contracts/*.openapi.yaml`.

The naming checker MUST scan `microservices/*/contracts/*.openapi.yaml`.

The naming checker MUST scan `microservices/*/contracts/*.asyncapi.yaml`.

The naming checker MUST scan `microservices/*/contracts/*.proto`.

The naming checker MUST scan `microservices/*/policy/*.cedar`.

The naming checker MUST scan `microservices/*/slos/*.openslo.yaml`.

The naming checker MUST scan `docs/decisions/ADR-*.md`.

The naming checker MUST scan `microservices/*/decisions/ADR-*.md`.

The naming checker SHOULD report all failures in one run.

The naming checker SHOULD include a remediation class for each failure.

The naming checker MUST fail on newly introduced invalid names.

The naming checker MAY remain advisory for legacy names only when a migration ledger row exists.

Verification command:

```bash
presubmit (retired CLI gate validate) naming-convention --scope repo
presubmit (retired CLI gate validate) layered-architecture --scope crates
presubmit (retired CLI gate validate) catalog-id-discipline --scope registry
```

CI evidence MUST include the count of scanned artifacts.

CI evidence MUST include the count of invalid ids.

CI evidence MUST include the migration ledger ids for any tolerated legacy rows.

## Common Anti-Patterns

Using `shared` as a crate token is an anti-pattern.

Using `platform` as a crate token is an anti-pattern.

Using `vertical` as a crate token is an anti-pattern.

Using a cloud provider in a kernel crate is an anti-pattern.

Using a product marketing name as a service id is an anti-pattern.

Using a temporary roadmap phase in a durable id is an anti-pattern.

Using a team name in a runtime artifact id is an anti-pattern.

Using an acronym before it is in `docs/GLOSSARY.md` is an anti-pattern.

Using `api.yaml` for a public contract is an anti-pattern.

Using `events.yaml` for an AsyncAPI contract is an anti-pattern.

Using `schema.proto` for a proto3 contract is an anti-pattern.

Using `main.cedar` for authorization policy is an anti-pattern.

Using `slo.yaml` without an SLI name is an anti-pattern.

Using `dashboard.json` without service and signal family is an anti-pattern.

Using unversioned public contracts is an anti-pattern.

Using a date suffix for a non-snapshot contract is an anti-pattern.

Using a region suffix for a global artifact is an anti-pattern.

Using a pack suffix for a pack-agnostic artifact is an anti-pattern.

Using a compatibility alias without a sunset is an anti-pattern.

Using a checker exception without an ADR is an anti-pattern.

## Cross-References

`docs/standards/crate-naming-convention.md` remains the crate-specific precedent.

`docs/standards/layer-enum-adr-0105.md` owns the layer vocabulary.

`docs/standards/clean-architecture.md` owns dependency direction.

`docs/standards/openapi-3-2-authoring.md` owns REST contract file rules.

`docs/standards/asyncapi-3-1-authoring.md` owns event contract file rules.

`docs/standards/proto3-authoring.md` owns proto package and file rules.

`docs/standards/cedar-policy-authoring.md` owns Cedar archetype rules.

`docs/standards/openslo-authoring.md` owns SLO manifest rules.

`docs/decisions/ADR-0700-ci-admission-live-apex.md` records the BNF decision.

`docs/decisions/ADR-0709-general-live-apex.md` records layer enumeration.

`docs/decisions/ADR-0703-cas-cache-live-apex.md` records role transition.

`docs/decisions/ADR-0701-monorepo-capability-live-apex.md` records microservice layout.

`docs/decisions/ADR-0709-general-live-apex.md` records capability tier naming.

## Substance Bar Compliance Checklist

BNF-SB-001. Verify `oyatie-` prefix on every non-checker crate.

BNF-SB-002. Verify `check-` prefix on every checker crate.

BNF-SB-003. Verify microservice token against workspace metadata.

BNF-SB-004. Verify bounded-context token against service manifest.

BNF-SB-005. Verify terminal token against ADR-0105 layer enum.

BNF-SB-006. Verify `[lib].name` hyphen-to-underscore mapping.

BNF-SB-007. Verify registry catalog id equals filename stem.

BNF-SB-008. Verify capability tier id uses service-capability-tier shape.

BNF-SB-009. Verify OpenAPI filename includes bounded context and major version.

BNF-SB-010. Verify AsyncAPI filename includes bounded context and major version.

BNF-SB-011. Verify proto filename includes bounded context and major version.

BNF-SB-012. Verify Cedar filename matches canonical archetype.

BNF-SB-013. Verify OpenSLO filename names the SLI.

BNF-SB-014. Verify ADR filename uses zero-padded id.

BNF-SB-015. Verify local microservice ADR filename uses service prefix.

BNF-SB-016. Verify implementation plan filename starts with `IP-`.

BNF-SB-017. Verify journey plan filename starts with `IP-journey-`.

BNF-SB-018. Verify runbook filename names operation or failure mode.

BNF-SB-019. Verify dashboard filename names service and signal family.

BNF-SB-020. Verify audit event id uses `EVT-` grammar.

BNF-SB-021. Reject provider token in kernel crate names.

BNF-SB-022. Reject product SKU token in service names.

BNF-SB-023. Reject team name token in runtime artifact ids.

BNF-SB-024. Reject temporary milestone token in durable ids.

BNF-SB-025. Reject region token on global artifacts.

BNF-SB-026. Reject pack token on pack-agnostic artifacts.

BNF-SB-027. Reject unversioned public contract filenames.

BNF-SB-028. Reject `api.yaml` in contract directories.

BNF-SB-029. Reject `events.yaml` in contract directories.

BNF-SB-030. Reject `schema.proto` in contract directories.

BNF-SB-031. Reject `main.cedar` in policy directories.

BNF-SB-032. Reject `slo.yaml` in SLO directories.

BNF-SB-033. Reject `dashboard.json` in dashboard directories.

BNF-SB-034. Reject `shared` as slot-two crate token.

BNF-SB-035. Reject `platform` as slot-two crate token.

BNF-SB-036. Reject `vertical` as slot-two crate token.

BNF-SB-037. Require migration ledger row for aliases.

BNF-SB-038. Require sunset date for aliases.

BNF-SB-039. Require successor id for aliases.

BNF-SB-040. Require ADR citation for new namespace.

BNF-SB-041. Require glossary citation for new acronym.

BNF-SB-042. Require catalog update for renamed crate.

BNF-SB-043. Require route parity update for renamed contract.

BNF-SB-044. Require policy reference update for renamed Cedar file.

BNF-SB-045. Require dashboard reference update for renamed SLO.

BNF-SB-046. Require audit event migration for renamed event id.

BNF-SB-047. Require fixture update for checker grammar change.

BNF-SB-048. Require docs cross-reference sweep for renamed standard.

BNF-SB-049. Require `docs/GLOSSARY.md` entry for repeated new term.

BNF-SB-050. Require tenant-visible label indirection for display text.

BNF-SB-051. Check `crates/workflow-engine-state-machine-domain`.

BNF-SB-052. Check `crates/tenancy-sub-scope-registry-kernel`.

BNF-SB-053. Check `crates/policy-cedar-domain`.

BNF-SB-054. Check `crates/cloud-compute-vm-api`.

BNF-SB-055. Check `crates/intelligence-evidence-domain`.

BNF-SB-056. Check `microservices/workflow-engine/contracts/state-machine-v1.openapi.yaml`.

BNF-SB-057. Check `microservices/workflow-engine/contracts/state-machine-v1.asyncapi.yaml`.

BNF-SB-058. Check `microservices/workflow-engine/contracts/state-machine-v1.proto`.

BNF-SB-059. Check `microservices/tenancy/policy/tenant-scope.cedar`.

BNF-SB-060. Check `microservices/observability/slos/availability.openslo.yaml`.

BNF-SB-061. Emit count of crate ids scanned.

BNF-SB-062. Emit count of contract ids scanned.

BNF-SB-063. Emit count of policy ids scanned.

BNF-SB-064. Emit count of SLO ids scanned.

BNF-SB-065. Emit count of dashboard ids scanned.

BNF-SB-066. Emit count of runbook ids scanned.

BNF-SB-067. Emit count of ADR ids scanned.

BNF-SB-068. Emit count of migration aliases scanned.

BNF-SB-069. Emit invalid id diagnostics with expected grammar.

BNF-SB-070. Emit replacement suggestion only when deterministic.

BNF-SB-071. Map naming failures to ADR-0056 when crate grammar fails.

BNF-SB-072. Map naming failures to ADR-0105 when layer token fails.

BNF-SB-073. Map naming failures to ADR-0131 when layout fails.

BNF-SB-074. Map naming failures to ADR-0316 when capability tier id fails.

BNF-SB-075. Map naming failures to ADR-0258 when contract version fails.

BNF-SB-076. Map naming failures to ADR-0263 when audit event id fails.

BNF-SB-077. Preserve old names in tombstone rows only.

BNF-SB-078. Preserve generated names only when generator version is declared.

BNF-SB-079. Preserve pack suffixes only for pack-specific artifacts.

BNF-SB-080. Preserve region suffixes only for region-specific artifacts.

## Extended Worked Example: Cross-Surface Naming Registry Migration

This example shows a safe rename of a workflow-engine capability from an
informal local name to a BNF-v4 conformant name. The migration is intentionally
multi-surface because naming is a contract, not a cosmetic label.

```yaml
change_id: rename-workflow-template-cancel-v1
governing_standard: docs/standards/naming-convention-bnf-v4.md
related_adrs:
  - docs/adr-archive/ADR-0056-rust-clean-architecture-bnf.md
  - docs/adr-archive/ADR-0105-13-layer-enum-and-check-family-patterns.md
  - docs/adr-archive/ADR-0131-per-microservice-flat-layout.md
  - docs/adr-archive/ADR-0258-api-versioning-model.md
old_names:
  crate: workflow_cancel
  openapi_operation: cancel
  asyncapi_message: cancelTask
  proto_rpc: Cancel
  cedar_action: Cancel
new_names:
  crate: workflow-template-cancel-usecase
  openapi_operation: workflowTemplateCancelV1
  asyncapi_message: WorkflowTemplateCancelRequestedV1
  proto_rpc: WorkflowTemplateCancelV1
  cedar_action: Action::"workflow.template.cancel.v1"
required_alias_rows:
  - registry: registry/naming/renames.jsonl
    from: workflow_cancel
    to: workflow-template-cancel-usecase
    sunset_after: 2026-08-20
  - registry: registry/naming/renames.jsonl
    from: cancelTask
    to: WorkflowTemplateCancelRequestedV1
    sunset_after: 2026-08-20
required_paths:
  - crates/workflow-template-cancel-usecase/Cargo.toml
  - microservices/workflow-engine/contracts/state-machine-v1.openapi.yaml
  - microservices/workflow-engine/contracts/state-machine-v1.asyncapi.yaml
  - microservices/workflow-engine/contracts/state-machine-v1.proto
  - microservices/workflow-engine/policy/workflow-template.cedar
  - docs/adr-archive/ADR-0258-api-versioning-model.md
  - docs/standards/capability-tier-matrix.md
verification:
  commands:
    - cargo run -p check-naming-bnf-v4 --quiet
    - cargo run -p check-contract-name-parity --quiet
    - cargo run -p check-cedar-action-names --quiet
  expected:
    invalid_identifiers: 0
    unresolved_aliases: 0
    missing_rename_tombstones: 0
```

Reviewers MUST reject the migration if any downstream surface still uses the
old informal name after the tombstone row is introduced. Tombstones are for
compatibility and audit traversal only; they are not permission to keep writing
new artifacts with the retired name.

## Extended Worked Example: BNF Tokens by Artifact Surface

| ID | Surface | Valid example | Invalid example | Enforced by |
|---|---|---|---|---|
| BNF-SURF-001 | Rust kernel crate | `tenancy-tenant-registry-kernel` | `tenant_registry` | `check-naming-bnf-v4` |
| BNF-SURF-002 | Rust domain crate | `policy-cedar-domain` | `cedarDomain` | `check-crate-name-parity` |
| BNF-SURF-003 | Rust usecase crate | `workflow-template-start-usecase` | `start_workflow` | `check-layer-token` |
| BNF-SURF-004 | Rust adapter crate | `mail-smtp-adapter-aws-ses` | `ses-mailer` | `check-provider-suffix` |
| BNF-SURF-005 | Rust runtime crate | `intelligence-provider-runtime` | `foundry_service` | `check-runtime-token` |
| BNF-SURF-006 | API contract file | `tenant-registry-v1.openapi.yaml` | `api.yaml` | `check-openapi-names` |
| BNF-SURF-007 | Event contract file | `workflow-state-v1.asyncapi.yaml` | `events.yaml` | `check-asyncapi-names` |
| BNF-SURF-008 | Proto contract file | `workflow_state_v1.proto` | `workflow.proto` | `check-proto-names` |
| BNF-SURF-009 | Cedar policy file | `tenant-scope.cedar` | `authz.cedar` | `check-cedar-policy-names` |
| BNF-SURF-010 | OpenSLO file | `workflow-engine-availability.openslo.yaml` | `slo.yaml` | `check-openslo-names` |
| BNF-SURF-011 | ADR file | `ADR-0105-layer-enum-ratchet.md` | `layers.md` | `check-adr-name-shape` |
| BNF-SURF-012 | Standard file | `proto3-authoring.md` | `protobuf.md` | `check-standard-name-shape` |
| BNF-SURF-013 | Evidence file | `standards-completeness-w1-2026-05-20.json` | `evidence.json` | `check-evidence-names` |
| BNF-SURF-014 | Runbook file | `workflow-engine-stuck-execution.md` | `how-to-fix-workflows.md` | `check-runbook-names` |
| BNF-SURF-015 | Dashboard id | `workflow-engine-latency-p99` | `Latency Dashboard` | `check-dashboard-ids` |
| BNF-SURF-016 | Metric name | `oyatie_workflow_execution_duration_seconds` | `workflowTime` | `check-metric-names` |
| BNF-SURF-017 | Trace span | `workflow.engine.transition.apply` | `do transition` | `check-trace-span-names` |
| BNF-SURF-018 | Audit event | `EVT-WORKFLOW-TRANSITION-APPLIED` | `workflow changed` | `check-audit-event-names` |
| BNF-SURF-019 | Capability id | `cap.workflow.template.start.t2` | `starter` | `check-capability-tier-matrix` |
| BNF-SURF-020 | Tenant pack id | `pack-kr-fintech` | `Korea Finance` | `check-pack-id-names` |
| BNF-SURF-021 | Cell id | `cell-kr-csap-seoul-1` | `seoul-prod` | `check-cell-names` |
| BNF-SURF-022 | RuntimeClass id | `kata-clh-sev-snp` | `secure runtime` | `check-runtimeclass-names` |
| BNF-SURF-023 | Helm chart id | `workflow-engine` | `workflowEngine` | `check-helm-chart-names` |
| BNF-SURF-024 | Kustomize component | `istio-waypoint-policies` | `waypointStuff` | `check-kustomize-names` |
| BNF-SURF-025 | Secret reference | `workflow-engine/provider/aws-ses/api-key` | `sesKey` | `check-secret-ref-names` |
| BNF-SURF-026 | Config key | `workflow.execution.max_attempts` | `maxAttempts` | `check-config-key-names` |
| BNF-SURF-027 | Env var | `OYATIE_WORKFLOW_ENGINE_BIND_ADDR` | `PORT` | `check-env-var-names` |
| BNF-SURF-028 | Queue name | `workflow-engine.execution.requested.v1` | `jobs` | `check-queue-names` |
| BNF-SURF-029 | Topic name | `tenant.scope.updated.v1` | `tenantUpdates` | `check-topic-names` |
| BNF-SURF-030 | Table name | `workflow_execution` | `WorkflowExecution` | `check-schema-names` |
| BNF-SURF-031 | Column name | `tenant_id` | `tenantId` | `check-schema-names` |
| BNF-SURF-032 | Index name | `idx_workflow_execution_tenant_state` | `tenantStateIdx` | `check-schema-names` |
| BNF-SURF-033 | Migration file | `20260520_workflow_execution_state.sql` | `fix.sql` | `check-migration-names` |
| BNF-SURF-034 | Fixture file | `workflow-execution-requested-v1.json` | `sample.json` | `check-fixture-names` |
| BNF-SURF-035 | SDK package | `@oyatie/workflow-engine-v1` | `workflow-sdk` | `check-sdk-package-names` |
| BNF-SURF-036 | Generated module | `workflow_engine_v1` | `WorkflowEngine` | `check-generated-module-names` |
| BNF-SURF-037 | Feature flag | `workflow_template_cancel_v1_enabled` | `cancelFlow` | `check-feature-flag-names` |
| BNF-SURF-038 | Experiment id | `exp-workflow-cancel-copy-2026q2` | `buttonTest` | `check-experiment-names` |
| BNF-SURF-039 | Rollout id | `rollout-workflow-cancel-v1-dev` | `newCancel` | `check-rollout-names` |
| BNF-SURF-040 | Alert id | `alert-workflow-engine-error-budget-fast-burn` | `workflowBad` | `check-alert-names` |

## Extended Compliance Checklist for Naming Reviews

BNF-REV-001. Confirm the name includes the owning domain token.

BNF-REV-002. Confirm the layer token matches ADR-0105.

BNF-REV-003. Confirm the service token matches the µservice directory.

BNF-REV-004. Confirm the version token appears only on versioned contracts.

BNF-REV-005. Confirm the provider token appears only in adapter surfaces.

BNF-REV-006. Confirm the region token appears only in region-bound artifacts.

BNF-REV-007. Confirm display labels are not used as machine identifiers.

BNF-REV-008. Confirm deprecated aliases have sunset dates.

BNF-REV-009. Confirm the ADR that created the name is cited.

BNF-REV-010. Confirm generated SDK identifiers map back to the contract name.

BNF-REV-011. Confirm audit-event names use the `EVT-*` grammar.

BNF-REV-012. Confirm metric names use Prometheus-compatible snake case.

BNF-REV-013. Confirm trace spans use dotted lower-case semantic names.

BNF-REV-014. Confirm Cedar actions use dotted action ids.

BNF-REV-015. Confirm OpenAPI operation ids are stable and versioned.

BNF-REV-016. Confirm AsyncAPI message ids include event tense.

BNF-REV-017. Confirm Proto packages do not embed deployment environment.

BNF-REV-018. Confirm queue names do not expose tenant names.

BNF-REV-019. Confirm table names do not expose product marketing language.

BNF-REV-020. Confirm environment variables use the `OYATIE_` prefix.

BNF-REV-021. Confirm root-hub pointers reference the canonical file name.

BNF-REV-022. Confirm docs cross-references use the new canonical path.

BNF-REV-023. Confirm dashboards use canonical metric ids.

BNF-REV-024. Confirm runbooks use incident-oriented names.

BNF-REV-025. Confirm test fixtures state the contract version in the file name.

BNF-REV-026. Confirm evidence bundles include agent and date tokens.

BNF-REV-027. Confirm migration files include sortable dates.

BNF-REV-028. Confirm temporary scratch files do not graduate into canonical paths.

BNF-REV-029. Confirm no name has unexplained acronym expansion.

BNF-REV-030. Confirm the glossary carries repeated new terminology.

BNF-REV-031. Confirm tenant-visible copy is isolated from machine ids.

BNF-REV-032. Confirm regulatory pack ids match pack registry rows.

BNF-REV-033. Confirm sovereign cell ids include jurisdiction and cell ordinal.

BNF-REV-034. Confirm feature flags include a retirement owner.

BNF-REV-035. Confirm rollout ids include environment and capability tokens.

BNF-REV-036. Confirm alert ids include symptom, not implementation rumor.

BNF-REV-037. Confirm external provider ids are not used as domain names.

BNF-REV-038. Confirm service aliases are not treated as canonical names.

BNF-REV-039. Confirm renamed public names have compatibility tests.

BNF-REV-040. Confirm the VCS promote evidence includes naming checker output.
