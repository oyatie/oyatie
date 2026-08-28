---
doc_class: Owner-PLAN
owner: app/payroll
status: Accepted
date: 2026-08-27
---

# Payroll remaining work

This plan begins after the owner-law baseline. Lane names are semantic
operator language. Each lane is authority, structural, behavioral, or
operational; no lane mixes classes.

<dispatch_rules>

## Dispatch rules

- A listed path set is closed. Literal directory retirement or move means the
  named tracked directory as one git rm/git mv target; no wildcard is implied.
- Structural lanes change directories, faces, manifests, Buck targets, package
  identity, or Cargo.lock and do not change business behavior.
- Behavioral lanes change only content paths inside already admitted crates
  and use stable, pre-registered test targets.
- NON-DISPATCHABLE means an owning decision is unresolved. That decision must
  amend this PLAN with literal paths, dependencies, generated outputs, tests,
  and Buck targets before a worker receives it.
- Every Rust lane runs its target-scoped Buck targets and protected Cargo
  package/test command with --locked. Buck never substitutes for protected
  Cargo evidence.
- Disjoint behavioral files may run in parallel after their shared structural
  parent merges. Structural lanes and Cargo.lock writers serialize.
- No lane creates a placeholder process, empty main, compatibility server,
  JSON product surface, hand-written Connect framing, or cloud-core import.

</dispatch_rules>

<dependency_graph>

## Dependency order

    ADR-0710 invariant amendment
      -> atomic IAM tenant-rbac retirement
          -> zero IAM-to-Payroll edges
              -> portable core identity
              -> legacy Payroll cone retirement

    root OpenAPI binding retirement
      -> pipeline fixture-policy retirement
          -> legacy Payroll cone retirement

    local records face
      -> local classification contract
          -> local consumer detachment
              -> core source-budget split
                  -> portable core identity
                      -> calculation / overlay / gate behavior

    records + external port contracts
      -> pack / audit / accounting provider settlements
      -> authorization-evidence provider settlement
      -> core dependency wiring
      -> real port-orchestrating use cases

    (core source-budget completion AND
     accepted record-encryption owner/provider decision AND
     encryption port + adapter structural admission)
      -> SQLite library selection
          -> SQLite adapter face
              -> SQLite adapter dependency wiring
                  -> SQLite records behavior
                      -> SQLite encryption integration

    encryption port + adapter contract behavior
      -> core dependency wiring
      -> SQLite encryption integration

    generated Connect decision
      -> facade staging decision
      -> proto structure
      -> proto conformance behavior
      -> facade behavior
      -> route activation
      -> promotion

</dependency_graph>

<owner_law_baseline>

## Owner law baseline

Class: authority-only; this commit.

    app/payroll/ADR.md
    app/payroll/PRD.md
    app/payroll/SPEC.md
    app/payroll/PLAN.md
    app/payroll/README.md

Success is one current/target truth with no code, manifest, lock, generated,
root-law, or foreign-owner diff. Failure is an unsupported implementation
claim, hidden current gap, stale numeric lane name, fake process, or path
outside this set. Review: Payroll and architecture.

</owner_law_baseline>

<admission_invariant_amendment>

## Preserve the admission invariant before IAM retirement

Class: authority; NON-DISPATCHABLE; IAM + Kubernetes + architecture owner.

The exact path is:

    docs/decisions/ADR-0710-kubernetes-admission-substrate-is-the-api-server.md

The accepted amendment preserves identity-bound, fail-closed in-process
VAP/CEL authorizer/RBAC and PSA for caller-sensitive admission, while removing
the claim that iam/core/tenant-rbac-tenant-admission-policy is its live
instance. That crate is review-only at the current base and records every
runtime attachment/enforcement flag as false. The amendment names the actual
future enforcement owner and evidence without moving or salvaging tenant-rbac
code.

This is not an ADR-0719 amendment. Until the amendment is accepted, neither the
IAM cone retirement nor Payroll collapse may dispatch. Success is an accepted
five-field ADR-0710 amendment with the abstract invariant unchanged. Failure is
deleting the named crate while ADR-0710 still points to it, weakening
fail-closed admission, or claiming review-only Rust is runtime enforcement.

</admission_invariant_amendment>

<iam_cone_retirement>

## Retire the complete false tenant-rbac cone

Class: structural; IAM-owned; depends on the accepted ADR-0710 amendment.

This is one atomic deletion: no moves, salvage, splits, replacements, or
compatibility aliases. The 39 literal crate roots are:

    iam/adapters/tenant-rbac-postgres-rls-storage
    iam/adapters/tenant-rbac-postgres-rls-transaction-contract
    iam/adapters/tenant-rbac-postgres-rls-write-contract
    iam/adapters/tenant-rbac-storage-inmemory
    iam/adapters/tenant-rbac-workflow-inmemory
    iam/core/tenant-rbac-audit-chain-emission
    iam/core/tenant-rbac-auth-app
    iam/core/tenant-rbac-deployment-manifest
    iam/core/tenant-rbac-domain
    iam/core/tenant-rbac-tenant-admission-policy
    iam/core/tenant-rbac-tenant-workload-manifest
    iam/core/tenant-rbac-usecase
    iam/facade/tenant-rbac-app
    iam/facade/tenant-rbac-audit-chain-runtime-evidence
    iam/facade/tenant-rbac-deployment-evidence
    iam/facade/tenant-rbac-disbursement-evidence
    iam/facade/tenant-rbac-erp-parity-map
    iam/facade/tenant-rbac-identity-provider-runtime-evidence
    iam/facade/tenant-rbac-identity-provider-verification
    iam/facade/tenant-rbac-listener-gateway
    iam/facade/tenant-rbac-listener-runtime-evidence
    iam/facade/tenant-rbac-local-inmemory-harness
    iam/facade/tenant-rbac-local-runtime-composition
    iam/facade/tenant-rbac-postgres-rls-runtime-evidence
    iam/facade/tenant-rbac-readiness-gate
    iam/facade/tenant-rbac-slo-evidence
    iam/facade/tenant-rbac-statutory-filing-evidence
    iam/facade/tenant-rbac-tenant-workload-runtime-evidence
    iam/facade/tenant-rbac-workflow-runtime-evidence
    iam/ports/tenant-rbac-api
    iam/ports/tenant-rbac-tenant-autoscaling-contract
    iam/ports/tenant-rbac-tenant-availability-contract
    iam/ports/tenant-rbac-tenant-cost-allocation-contract
    iam/ports/tenant-rbac-tenant-egress-policy-contract
    iam/ports/tenant-rbac-tenant-image-provenance-contract
    iam/ports/tenant-rbac-tenant-residency-contract
    iam/ports/tenant-rbac-tenant-resource-quota-contract
    iam/ports/tenant-rbac-tenant-secret-boundary-contract
    iam/ports/tenant-rbac-tenant-workload-identity-contract

Delete these four hand-authored SLO files and mechanically refresh the lock:

    iam/observability/slos/tenant-rbac/tenant-rbac-audit-emission-lag-p99.openslo.yaml
    iam/observability/slos/tenant-rbac/tenant-rbac-availability.openslo.yaml
    iam/observability/slos/tenant-rbac/tenant-rbac-latency-p99.openslo.yaml
    iam/observability/slos/tenant-rbac/tenant-rbac-readiness-gate-correctness.openslo.yaml
    Cargo.lock

Success is zero Payroll Cargo path dependencies, Buck labels, Rust imports,
route strings, evidence strings, direct edges, and transitive edges anywhere
under iam/. Every IAM Payroll route/evidence reference disappears with the
deleted roots; no replacement path is moved or salvaged. Required verification
includes cargo metadata --locked,
cargo test --locked -p iam-pdp-app -p iam-identity-service, Buck targets
//iam/facade/pdp-app:iam-pdp-app-unittest and
//iam/facade/identity-service:iam-identity-service-tests, workspace reverse
dependency inspection, and repository search for every deleted package/path.
Review: IAM, Kubernetes, Payroll, HR, Accounting, Security, Build, and
architecture.

</iam_cone_retirement>

<root_wire_reference_retirement>

## Retire the root Payroll OpenAPI binding

Class: structural; root/build owner. Exact path:

    .cargo/config.toml

Delete only OYATIE_PAYROLL_OPENAPI_CONTRACT. Do not delete or rewrite other
fixture bindings. Success is a parseable Cargo configuration with no Payroll
OpenAPI value. Failure is a Payroll worker editing this path, unrelated
fixture churn, or deleting the Payroll contract first. Review: Build,
pipeline, Payroll, and architecture. Verification is
`cargo test --locked -p pipeline-admission layout::cargo_config` plus the
repository path-layout app against the PR base/head.

</root_wire_reference_retirement>

<pipeline_wire_reference_retirement>

## Retire pipeline admission's Payroll fixture exception

Class: behavioral; pipeline owner; follows the root binding retirement.

At the current base pipeline/admission and its workspace-members dependency
have no Buck faces. A preceding pipeline structural PR therefore adds exactly:

    pipeline/core/workspace-members-kernel/BUCK
    pipeline/core/admission/BUCK

It registers //pipeline/core/workspace-members-kernel:workspace-members-kernel,
//pipeline/core/admission:pipeline-admission, and
//pipeline/core/admission:pipeline-admission-cargo-config-unittest over the
existing Cargo crates and //third-party:toml; it changes no Rust or policy
behavior. Protected Cargo is
`cargo test --locked -p workspace-members-kernel -p pipeline-admission` and Buck
builds/tests those exact labels. The following behavioral PR changes exactly:

    pipeline/core/admission/src/layout/cargo_config.rs

Delete the Payroll tuple from FIXTURE_BINDINGS, delete the positive Payroll
fixture assertion, and delete the three Payroll-specific malformed-binding
vectors while preserving generic closed-environment tests. Protected Cargo:
`cargo test --locked -p pipeline-admission layout::cargo_config`. Buck:
//pipeline/core/admission:pipeline-admission-cargo-config-unittest. Success is
no OYATIE_PAYROLL_OPENAPI_CONTRACT or Payroll OpenAPI path in pipeline. Review:
pipeline, Build, Payroll, and architecture.

</pipeline_wire_reference_retirement>

<local_records_face>

## Register the Payroll-local records face

Class: structural; executes Payroll row D1b-CA-S already sequenced by
data/PLAN.md. Exact paths:

    app/payroll/ports/draft/records/Cargo.toml
    app/payroll/ports/draft/records/BUCK
    app/payroll/ports/draft/records/build.rs
    app/payroll/ports/draft/records/src/lib.rs
    Cargo.lock

The scanner-only root registers package payroll-records-draft and Buck targets
//app/payroll/ports/draft/records:payroll-records-draft and
//app/payroll/ports/draft/records:payroll-records-draft-unittest. It has no
Data/Gateway/provider dependency and no contract behavior. Protected Cargo is
`cargo test --locked -p payroll-records-draft`; Buck runs both exact targets.
Review: Payroll, Data, Build, and architecture.

</local_records_face>

<local_classification_contract>

## Reproduce classification as an owner-local records contract

Class: behavioral; follows the local records face and executes Payroll row
D1b-CA-C in data/PLAN.md. Exact paths:

    app/payroll/ports/draft/records/src/items/a_data_class.rs
    app/payroll/ports/draft/records/src/items/b_privacy_data_class.rs
    app/payroll/ports/draft/records/src/items/c_classification_axes.rs
    app/payroll/ports/draft/records/src/items/d_parsers.rs
    app/payroll/ports/draft/records/src/items/e_classified.rs
    app/payroll/ports/draft/records/src/test_items/a_labels.rs
    app/payroll/ports/draft/records/src/test_items/b_privacy.rs
    app/payroll/ports/draft/records/src/test_items/c_classified.rs

Preserve the current spelling, derives, constructors, parser trimming, labels,
errors, and Classified<T> shape without sharing Rust type identity with Data.
Protected Cargo is `cargo test --locked -p payroll-records-draft`; Buck runs the
two exact records targets above. Failure is IO, storage behavior, a cloud edge,
or semantic drift. Review: Payroll, Data, Build, and architecture.

</local_classification_contract>

<local_classification_consumer_detachment>

## Detach current Payroll consumers from Data classification

Class: structural; follows the local classification contract and executes
Payroll row D1b-CA-X in data/PLAN.md. Exact paths:

    app/payroll/core/run-domain/Cargo.toml
    app/payroll/core/run-domain/BUCK
    app/payroll/facade/run-app/Cargo.toml
    app/payroll/facade/run-app/BUCK
    Cargo.lock

Cargo aliases and Buck named_deps preserve the existing data_boundary_kernel
extern spelling while resolving to payroll-records-draft; no Rust source or
behavior changes. Protected Cargo is
`cargo test --locked -p payroll-records-draft -p payroll-run-domain -p payroll-run-app`.
The exact Buck build/test set is:

    //app/payroll/ports/draft/records:payroll-records-draft
    //app/payroll/ports/draft/records:payroll-records-draft-unittest
    //app/payroll/core/run-domain:payroll-run-domain
    //app/payroll/core/run-domain:payroll-run-domain-accounting-bridge
    //app/payroll/core/run-domain:payroll-run-domain-filing
    //app/payroll/core/run-domain:payroll-run-domain-group
    //app/payroll/core/run-domain:payroll-run-domain-group-gl-posting
    //app/payroll/core/run-domain:payroll-run-domain-hr-leave
    //app/payroll/core/run-domain:payroll-run-domain-kr-close
    //app/payroll/core/run-domain:payroll-run-domain-retro-adjustment
    //app/payroll/core/run-domain:payroll-run-domain-rollback
    //app/payroll/core/run-domain:payroll-run-domain-rulepack-manifest
    //app/payroll/core/run-domain:payroll-run-domain-support
    //app/payroll/core/run-domain:payroll-run-domain-variance
    //app/payroll/facade/run-app:payroll-run-app
    //app/payroll/facade/run-app:payroll-run-app-app-envelopes
    //app/payroll/facade/run-app:payroll-run-app-hr-leave

Success is identical classification behavior with no Data core/port edge.
Review: Payroll, Data, all reverse consumers, Build, and architecture.

</local_classification_consumer_detachment>

<core_source_budget>

## Split the current core before migration

Class: structural/mechanical; hard prerequisite; no semantic change.

Exact paths:

    app/payroll/core/run-domain/build.rs
    app/payroll/core/run-domain/BUCK
    app/payroll/core/run-domain/src/lib.rs
    app/payroll/core/run-domain/src/items/a_identity.rs
    app/payroll/core/run-domain/src/items/b_money.rs
    app/payroll/core/run-domain/src/items/c_trial_validation.rs
    app/payroll/core/run-domain/src/items/d_hr_leave.rs
    app/payroll/core/run-domain/src/items/e_group_close.rs
    app/payroll/core/run-domain/src/items/f_statutory_evidence.rs
    app/payroll/core/run-domain/src/items/g_accounting.rs
    app/payroll/core/run-domain/src/items/h_promotion.rs
    app/payroll/core/run-domain/src/items/i_variance.rs
    app/payroll/core/run-domain/src/items/j_retro.rs
    app/payroll/core/run-domain/tests/variance.rs
    app/payroll/core/run-domain/tests/variance_edges.rs
    app/payroll/core/run-domain/tests/rulepack_manifest.rs
    app/payroll/core/run-domain/tests/rulepack_manifest_sources.rs

The std-only build script generates module membership in OUT_DIR; no generated
file is committed. BUCK registers one library-unit target that follows the
same source discovery plus the existing integration targets. The 1,933-line
root and two over-budget tests become files of at most 300 lines. Every public
type, function, error, vector, and existing target remains byte/behavior
equivalent.

Protected Cargo is `cargo test --locked -p payroll-run-domain`. Buck builds
//app/payroll/core/run-domain:payroll-run-domain, runs the new
//app/payroll/core/run-domain:payroll-run-domain-unittest, and runs the eleven
exact existing rust_test labels enumerated in the consumer-detachment lane.
Failure is a new invariant, public rename, fixture change, dependency/lock
change, or generated module index in git. Review: Payroll and Build.

</core_source_budget>

<portable_core_identity>

## Move the portable core to its final identity

Class: structural; requires IAM retirement, local consumer detachment, and the
source-budget split.

Move the literal directory:

    app/payroll/core/run-domain -> app/payroll/core/run

The literal move above occupies every tracked source/test file in that root.
Outside the moved root, edit only these consumer files for the mechanical
package/import/label rename:

    app/payroll/facade/run-app/Cargo.toml
    app/payroll/facade/run-app/BUCK
    app/payroll/facade/run-app/src/lib.rs
    app/payroll/facade/run-app/tests/app_envelopes.rs
    app/payroll/facade/run-app/tests/hr_leave.rs
    app/payroll/ports/run-api/Cargo.toml
    app/payroll/ports/run-api/BUCK
    app/payroll/ports/run-api/src/lib.rs
    app/payroll/ports/run-api/tests/contracts.rs
    app/payroll/adapters/run-infrastructure/Cargo.toml
    app/payroll/adapters/run-infrastructure/BUCK
    app/payroll/adapters/run-infrastructure/src/lib.rs
    app/payroll/adapters/run-infrastructure/src/authz.rs
    app/payroll/adapters/run-infrastructure/tests/runtime.rs
    app/payroll/adapters/run-storage-inmemory/Cargo.toml
    app/payroll/adapters/run-storage-inmemory/BUCK
    app/payroll/adapters/run-storage-inmemory/src/lib.rs
    app/payroll/adapters/run-storage-inmemory/tests/storage.rs
    Cargo.lock

Rename payroll-run-domain/payroll_run_domain to payroll-run/payroll_run and
the Buck targets mechanically. No behavior, wrapper, DTO, route, or storage
shape changes. Protected Cargo is
`cargo test --locked -p payroll-run -p payroll-records-draft -p payroll-run-app -p payroll-run-api -p payroll-run-infrastructure -p payroll-run-storage-inmemory`.
The exact Buck build/test set is:

    //app/payroll/core/run:payroll-run
    //app/payroll/core/run:payroll-run-unittest
    //app/payroll/core/run:payroll-run-accounting-bridge
    //app/payroll/core/run:payroll-run-filing
    //app/payroll/core/run:payroll-run-group
    //app/payroll/core/run:payroll-run-group-gl-posting
    //app/payroll/core/run:payroll-run-hr-leave
    //app/payroll/core/run:payroll-run-kr-close
    //app/payroll/core/run:payroll-run-retro-adjustment
    //app/payroll/core/run:payroll-run-rollback
    //app/payroll/core/run:payroll-run-rulepack-manifest
    //app/payroll/core/run:payroll-run-support
    //app/payroll/core/run:payroll-run-variance
    //app/payroll/ports/draft/records:payroll-records-draft
    //app/payroll/ports/draft/records:payroll-records-draft-unittest
    //app/payroll/facade/run-app:payroll-run-app
    //app/payroll/facade/run-app:payroll-run-app-app-envelopes
    //app/payroll/facade/run-app:payroll-run-app-hr-leave
    //app/payroll/ports/run-api:payroll-run-api
    //app/payroll/ports/run-api:payroll-run-api-contracts
    //app/payroll/adapters/run-infrastructure:payroll-run-infrastructure
    //app/payroll/adapters/run-infrastructure:payroll-run-infrastructure-unittest
    //app/payroll/adapters/run-infrastructure:payroll-run-infrastructure-runtime
    //app/payroll/adapters/run-storage-inmemory:payroll-run-storage-inmemory
    //app/payroll/adapters/run-storage-inmemory:payroll-run-storage-inmemory-storage

Failure is behavior drift or an IAM edge. Review: Payroll, Build, and
architecture.

</portable_core_identity>

<legacy_payroll_cone_retirement>

## Delete the false facade, volatile adapter, and legacy product wire

Class: structural; requires zero IAM-to-Payroll edges and both external
OpenAPI binding retirements.

Delete these literal roots/files together:

    app/payroll/facade/run-app
    app/payroll/ports/run-api
    app/payroll/adapters/run-infrastructure
    app/payroll/adapters/run-storage-inmemory
    app/payroll/contracts/openapi-v1.yaml
    app/payroll/contracts/openapi-v1.meta.yaml
    Cargo.lock

No run-app source is moved: all its public functions are no-op wrappers and all
its outcomes/topics/errors are deleted. The storage trait and metadata helpers
are deleted rather than promoted; later records code is written against the
real port. No main.rs, alias, REST codec, JSON product/protobuf-JSON mapping,
static bearer, Gateway import, or compatibility route is added.

Success is the portable core tests passing with no payroll-run-app,
payroll-run-api, payroll-run-infrastructure, payroll-run-storage-inmemory,
Payroll OpenAPI path, REST product route, or JSON product dependency.
Protected Cargo is
`cargo test --locked -p payroll-run -p payroll-records-draft`. Buck runs the
first fifteen exact core/records labels enumerated in the portable-core lane.
Review:
Payroll, IAM, HR, Accounting, Gateway, Data, pipeline, Build, architecture,
and API review.

</legacy_payroll_cone_retirement>

<local_port_faces>

## Register local port faces before contract behavior

Class: one serialized structural PR. It creates this literal path set and
pre-registers each library and unit-test Buck target; lib.rs/build.rs contain
only the stable item/test-item loader.

Pack-install:

    app/payroll/ports/draft/pack-install/Cargo.toml
    app/payroll/ports/draft/pack-install/BUCK
    app/payroll/ports/draft/pack-install/build.rs
    app/payroll/ports/draft/pack-install/src/lib.rs

Authorization-evidence:

    app/payroll/ports/draft/authorization-evidence/Cargo.toml
    app/payroll/ports/draft/authorization-evidence/BUCK
    app/payroll/ports/draft/authorization-evidence/build.rs
    app/payroll/ports/draft/authorization-evidence/src/lib.rs

Audit:

    app/payroll/ports/draft/audit/Cargo.toml
    app/payroll/ports/draft/audit/BUCK
    app/payroll/ports/draft/audit/build.rs
    app/payroll/ports/draft/audit/src/lib.rs

Accounting:

    app/payroll/ports/draft/accounting/Cargo.toml
    app/payroll/ports/draft/accounting/BUCK
    app/payroll/ports/draft/accounting/build.rs
    app/payroll/ports/draft/accounting/src/lib.rs
    Cargo.lock

Package/target names are payroll-pack-install-draft,
payroll-authorization-evidence-draft, payroll-audit-draft, and
payroll-accounting-draft, each with a -unittest target. Cargo.lock changes only
by the four workspace-package records. Protected Cargo is
`cargo test --locked -p payroll-pack-install-draft -p payroll-authorization-evidence-draft -p payroll-audit-draft -p payroll-accounting-draft`.
The exact Buck build/test labels are:

    //app/payroll/ports/draft/pack-install:payroll-pack-install-draft
    //app/payroll/ports/draft/pack-install:payroll-pack-install-draft-unittest
    //app/payroll/ports/draft/authorization-evidence:payroll-authorization-evidence-draft
    //app/payroll/ports/draft/authorization-evidence:payroll-authorization-evidence-draft-unittest
    //app/payroll/ports/draft/audit:payroll-audit-draft
    //app/payroll/ports/draft/audit:payroll-audit-draft-unittest
    //app/payroll/ports/draft/accounting:payroll-accounting-draft
    //app/payroll/ports/draft/accounting:payroll-accounting-draft-unittest

Success is four compile-loaded owner-local draft faces with no foreign
dependency or semantic claim. Review: Payroll and Build.

</local_port_faces>

<local_port_contracts>

## Define records and external-intent contracts

Class: behavioral; the five path sets are disjoint and may run in parallel
after their faces exist.

Records:

    app/payroll/ports/draft/records/src/items/f_canonical_request.rs
    app/payroll/ports/draft/records/src/items/g_records_contract.rs
    app/payroll/ports/draft/records/src/items/h_outbound_intent.rs
    app/payroll/ports/draft/records/src/test_items/d_canonical_request.rs
    app/payroll/ports/draft/records/src/test_items/e_records_contract.rs

Pack install:

    app/payroll/ports/draft/pack-install/src/items/a_contract.rs
    app/payroll/ports/draft/pack-install/src/test_items/a_contract.rs

Authorization evidence:

    app/payroll/ports/draft/authorization-evidence/src/items/a_principal.rs
    app/payroll/ports/draft/authorization-evidence/src/items/b_action_resource.rs
    app/payroll/ports/draft/authorization-evidence/src/items/c_evidence_port.rs
    app/payroll/ports/draft/authorization-evidence/src/test_items/a_contract.rs
    app/payroll/ports/draft/authorization-evidence/src/test_items/b_faults.rs

Audit:

    app/payroll/ports/draft/audit/src/items/a_contract.rs
    app/payroll/ports/draft/audit/src/test_items/a_contract.rs

Accounting:

    app/payroll/ports/draft/accounting/src/items/a_contract.rs
    app/payroll/ports/draft/accounting/src/test_items/a_contract.rs

Each package runs protected Cargo and its two exact Buck targets. Authorization
tests cover absent principal, deny, stale decision, timeout, malformed
evidence, and provider fault. Records tests cover canonical equality/conflict,
expected-version CAS, atomic outcome/evidence/intents, and replay. Review:
Payroll; Security additionally reviews authorization; provider owners join at
settlement, not before.

</local_port_contracts>

<provider_contract_settlement>

## Settle pack, audit, and accounting providers

Class: external contract; NON-DISPATCHABLE.

Packs, Audit, Accounting, and architecture each reconcile the corresponding
local draft onto one provider-owned agreed port/proto. The accepted amendment
to this PLAN must name for each need:

- the literal provider port and proto input paths;
- the generated client Cargo package and Buck target;
- the literal Payroll adapter backend directory;
- the adapter Cargo.toml, BUCK, build.rs, lib.rs, Cargo.lock, item, test-item,
  integration test, and golden paths;
- timeout/staleness/idempotency/fault semantics; and
- provider, Payroll, Security, API, Build, and architecture reviewers.

Adapter face/dependency structure lands before behavior, and behavior changes
only src/test paths loaded by stable pre-registered targets. No lane is
dispatchable while any literal provider/client/adapter path remains unnamed.

</provider_contract_settlement>

<authorization_adapter_settlement>

## Select the authorization-evidence provider and adapter

Class: external contract; NON-DISPATCHABLE; follows the Payroll-owned
authorization-evidence port contract.

Payroll, Policy, IAM, Security, protocol/API, Build, and architecture first
identify which sold facade supplies the decision and durable evidence. IAM
principal verification and Policy authorization are distinct responsibilities;
the settlement may compose their sold contracts but cannot invent an
in-process hybrid or let Payroll import either owner's core or ports. The
accepted PLAN amendment must name:

- exact provider proto and generated-client inputs and outputs;
- exact client Cargo package and Buck target;
- literal Payroll adapter Cargo.toml, BUCK, build.rs, lib.rs, Cargo.lock, item,
  test-item, integration-test, and error/protobuf golden paths;
- separate face, dependency, and behavior PR path sets;
- principal/tenant/action/resource/request-digest/evidence mappings; and
- deny, stale, timeout, malformed evidence, provider fault, reply loss,
  cancellation, saturation, and cross-tenant vectors.

The adapter returns only the Payroll port's typed authorization evidence or
refusal. Success tests prove every refusal causes zero business-core and
records calls and an approved mutation persists the evidence atomically. No
adapter lane is dispatchable until all literal paths, Cargo/Buck commands, and
provider reviews are recorded.

</authorization_adapter_settlement>

<record_encryption_decision>

## Decide record-encryption ownership and provider

Class: external contract; NON-DISPATCHABLE; hard prerequisite to every SQLite
selection, structural, dependency, and behavioral lane.

Payroll, the selected key/protection provider, Security/Privacy, Storage/Data,
Build, and architecture decide whether the agreed port is provider-owned or
begins as a Payroll draft. The decision must amend this PLAN with:

- exact agreed/draft port path and package/target;
- Seal/Open request, purpose, tenant, schema, generation, nonce, authenticated
  context, and typed fault contract;
- exact generated or commodity provider dependency;
- literal Payroll adapter directory, Cargo.lock, and every Cargo/Buck/src/test/
  golden path;
- key-unavailable, stale-generation, tamper, nonce-reuse, wrong-purpose,
  wrong-tenant, and restart vectors; and
- an adapter-structure lane before behavior.

Three conditions are conjunctive hard predecessors of
`<sqlite_dependency_decision>`, both structural PRs in
`<sqlite_adapter_structure>`, `<sqlite_records_behavior>`, and
`<sqlite_encryption_integration>`: `<core_source_budget>` is complete; this
owner/provider decision is accepted; and the decision's exact encryption port
and adapter structural lanes are admitted. No SQLite library selection,
dependency edit, face, schema/repository behavior, fixture, or encryption
integration may dispatch early.

No local record-encryption path from an earlier draft is reserved by this
plan. Failure is inventing provider ownership, putting keys in Payroll,
plaintext fallback, or beginning SQLite protected-record behavior before the
selected adapter structure is admitted.

</record_encryption_decision>

<calculation_behavior>

## Implement deterministic gross-to-net calculation

Class: behavioral. Exact paths:

    app/payroll/core/run/src/items/k_calculation.rs
    app/payroll/core/run/src/test_items/a_calculation.rs

Implement checked fixed-point inputs, canonical ordering, gross/taxable bases,
deductions, withholding, employer contributions, rounding adjustments, net,
and byte-stable calculation evidence. Unit/property tests cover permutation,
rounding boundaries, overflow, currency mismatch, duplicates, negative-net
rules, and deterministic replay.

Protected Cargo is `cargo test --locked -p payroll-run`. Buck is
//app/payroll/core/run:payroll-run-unittest. Success is a pure calculation
engine parameterized by a typed overlay; no current trial-close state or SLO is
promoted. Review: Payroll, Security, and architecture.

</calculation_behavior>

<jurisdiction_overlay_behavior>

## Implement certified owner-local jurisdiction overlays

Class: behavioral; follows calculation types. Exact paths:

    app/payroll/core/run/src/items/l_jurisdiction_overlay.rs
    app/payroll/core/run/src/test_items/b_jurisdiction_overlay.rs
    app/payroll/core/run/tests/fixtures/us_federal_2026_gross_to_net.txt
    app/payroll/core/run/tests/fixtures/kr_2026_gross_to_net.txt
    app/payroll/core/run/tests/fixtures/jp_2026_gross_to_net.txt
    app/payroll/core/run/tests/fixtures/eu_de_2026_gross_to_net.txt

Create independently reviewed official-source vectors only when each
version's evidence and effective window are accepted. Bind one
Packs-installed id to one local overlay; reject eu without a member, missing,
expired, mismatched, or uncertified versions. Tests cover bracket/rate
boundaries, rounding, and source digest. Protected Cargo is
`cargo test --locked -p payroll-run`; Buck is
//app/payroll/core/run:payroll-run-unittest. Review: Payroll, Compliance/Legal
for each jurisdiction, Packs, Security, and architecture.

</jurisdiction_overlay_behavior>

<anomaly_resolution_behavior>

## Gate variance and retro outcomes with typed resolution

Class: behavioral; may run parallel with overlay content after calculation
types exist. Exact paths:

    app/payroll/core/run/src/items/i_variance.rs
    app/payroll/core/run/src/items/j_retro.rs
    app/payroll/core/run/src/items/m_anomaly_resolution.rs
    app/payroll/core/run/src/test_items/c_anomaly_resolution.rs

Give findings stable ids and typed blocking reasons; implement Corrected,
AcceptedWithEvidence, and Rejected resolution, expected input digest/version,
authorization evidence reference, invalidation after correction, and no
free-form bypass. Retro outputs remain linked adjustments, not history edits.
Protected Cargo is `cargo test --locked -p payroll-run`; Buck is
//app/payroll/core/run:payroll-run-unittest. Failure is allowing close with an
unresolved/currently invalid resolution. Review: Payroll, HR, Security, Audit,
and architecture.

</anomaly_resolution_behavior>

<group_workflow_behavior>

## Implement entity and group close workflow

Class: behavioral; follows calculation and anomaly resolution. Exact paths:

    app/payroll/core/run/src/items/e_group_close.rs
    app/payroll/core/run/src/items/n_group_workflow.rs
    app/payroll/core/run/src/test_items/d_group_workflow.rs

Extend the existing pure rollup validation into typed entity-close and
group-close decisions over immutable digests. Reject missing/duplicate
entities, tenant/period/currency mismatch, stale overlay eligibility,
unresolved anomalies, and unbalanced aggregate intent. Protected Cargo is
`cargo test --locked -p payroll-run`; Buck is
//app/payroll/core/run:payroll-run-unittest. Review: Payroll, HR, Accounting,
and architecture.

</group_workflow_behavior>

<production_close_behavior>

## Implement production close as a distinct transition

Class: behavioral; follows group workflow and authorization evidence contract.
Exact paths:

    app/payroll/core/run/src/items/h_promotion.rs
    app/payroll/core/run/src/items/o_production_close.rs
    app/payroll/core/run/src/test_items/e_production_close.rs

Require eligible entity/group close, expected version, step-up authorization
evidence, certified overlay, resolved anomalies, balanced accounting intent,
and privileged audit intent. Emit a new immutable outcome; retry returns it,
and retro begins a new linked adjustment. Protected Cargo is
`cargo test --locked -p payroll-run`; Buck is
//app/payroll/core/run:payroll-run-unittest. Review: Payroll, Security, Audit,
Accounting, and architecture.

</production_close_behavior>

<core_port_dependency_wiring>

## Wire admitted ports into portable core

Class: structural; NON-DISPATCHABLE; serialized after local port contracts and
all promoted provider paths are frozen. Exact Payroll paths:

    app/payroll/core/run/Cargo.toml
    app/payroll/core/run/BUCK
    Cargo.lock

Add only the final records, pack-install, authorization-evidence, audit,
accounting, and record-encryption port packages/targets named by settlement.
Do not add adapters, generated transport, cloud core, cloud internal ports,
serde, SQL, or SQLite. Because provider package/target identities are not yet
accepted, this lane remains NON-DISPATCHABLE; the settlement amendments must
replace this paragraph with exact protected Cargo packages and Buck labels.
Failure is an unresolved draft imported by another owner or any
provider-internal edge.

</core_port_dependency_wiring>

<orchestrating_use_cases>

## Implement real port-orchestrating use cases

Class: behavioral; follows port dependency wiring and the matching domain
behavior. Exact paths:

    app/payroll/core/run/src/items/p_use_cases.rs
    app/payroll/core/run/src/test_items/f_use_cases.rs

Implement the eight SPEC use cases with authorization/pack/idempotency order,
atomic records writes, encryption request, and audit/accounting intents.
Deleted close_trial_run, prepare_accounting_dispatch, and
prepare_hr_leave_impact_intake names never return. Tests use deterministic
port fakes and assert zero records/core work on authority failure, same-key
replay, changed-key conflict, expected-version conflict, and durable intents.
Protected Cargo is `cargo test --locked -p payroll-run`; Buck is
//app/payroll/core/run:payroll-run-unittest. Review: Payroll, Packs, Policy/IAM,
Audit, Accounting, Security, and architecture.

</orchestrating_use_cases>

<records_inmemory_adapter>

## Build an in-memory records conformance oracle

Structure PR exact paths:

    app/payroll/adapters/draft/records-inmemory/Cargo.toml
    app/payroll/adapters/draft/records-inmemory/BUCK
    app/payroll/adapters/draft/records-inmemory/build.rs
    app/payroll/adapters/draft/records-inmemory/src/lib.rs
    Cargo.lock

Dependency structural PR:

    app/payroll/adapters/draft/records-inmemory/Cargo.toml
    app/payroll/adapters/draft/records-inmemory/BUCK
    Cargo.lock

Behavioral PR:

    app/payroll/adapters/draft/records-inmemory/src/items/a_repository.rs
    app/payroll/adapters/draft/records-inmemory/src/test_items/a_conformance.rs
    app/payroll/adapters/draft/records-inmemory/src/test_items/b_faults.rs

The face PR adds only the workspace package record to Cargo.lock and registers
//app/payroll/adapters/draft/records-inmemory:payroll-records-inmemory-draft plus
//app/payroll/adapters/draft/records-inmemory:payroll-records-inmemory-draft-unittest.
The dependency PR adds only payroll-records-draft. Behavior implements atomic
semantic outcomes in memory as a reference oracle and makes no durability
claim. Every head runs
`cargo test --locked -p payroll-records-inmemory-draft`; Buck builds the package
target and runs that exact unittest.

</records_inmemory_adapter>

<sqlite_dependency_decision>

## Select the SQLite library without ad-hoc dependency drift

Class: dependency authority; NON-DISPATCHABLE; hard-follows completion of
`<core_source_budget>` plus the accepted encryption decision and its admitted
port/adapter structure.

The current workspace has sqlx 0.8.6 configured only for Postgres and no
accepted SQLite library. Payroll, Build, Security, and architecture select the
MIT/Apache transient library and features behind the records adapter, then
amend this PLAN with the exact root `Cargo.toml` dependency/version/features,
`Cargo.lock` effect, adapter manifest entry, Buck third-party label/provenance,
license evidence, and protected Cargo/Buck commands. This decision does not
change the records port or business model.

</sqlite_dependency_decision>

<sqlite_adapter_structure>

## Register the SQLite adapter face and dependencies

Class: two structural PRs; hard-follows completion of `<core_source_budget>`,
the accepted encryption decision and its admitted port/adapter structure, the
records contract, and the SQLite dependency decision.

Face paths:

    app/payroll/adapters/draft/records-sqlite/Cargo.toml
    app/payroll/adapters/draft/records-sqlite/BUCK
    app/payroll/adapters/draft/records-sqlite/build.rs
    app/payroll/adapters/draft/records-sqlite/src/lib.rs
    Cargo.lock

Dependency paths:

    app/payroll/adapters/draft/records-sqlite/Cargo.toml
    app/payroll/adapters/draft/records-sqlite/BUCK
    Cargo.lock

The face adds only the workspace package record to Cargo.lock and registers
//app/payroll/adapters/draft/records-sqlite:payroll-records-sqlite-draft and
//app/payroll/adapters/draft/records-sqlite:payroll-records-sqlite-draft-unittest.
The dependency PR adds only payroll-records-draft, the accepted SQLite library,
and the exact encryption port named by its provider decision. It adds no
behavior. Protected Cargo is
`cargo test --locked -p payroll-records-sqlite-draft`; Buck builds/tests those
two exact labels after each structural head.

</sqlite_adapter_structure>

<sqlite_records_behavior>

## Implement atomic SQLite records over opaque protected values

Class: behavioral; hard-follows completion of `<core_source_budget>`, the
accepted encryption decision and its admitted port/adapter structure, and both
SQLite structural PRs. Exact paths:

    app/payroll/adapters/draft/records-sqlite/src/items/a_schema.rs
    app/payroll/adapters/draft/records-sqlite/src/items/b_repository.rs
    app/payroll/adapters/draft/records-sqlite/src/items/c_delivery_claim.rs
    app/payroll/adapters/draft/records-sqlite/src/test_items/a_adapter_parity.rs
    app/payroll/adapters/draft/records-sqlite/src/test_items/b_restart_replay.rs
    app/payroll/adapters/draft/records-sqlite/src/test_items/c_failure_injection.rs
    app/payroll/adapters/draft/records-sqlite/tests/fixtures/schema_v1.sql
    app/payroll/adapters/draft/records-sqlite/tests/fixtures/corrupt_schema_v1.bin

Persist only opaque protected-record values plus non-sensitive indexes,
canonical request digest, typed outcome metadata, authorization evidence
reference, and outbound intents in one writer transaction. Tests use a real
temporary file, hard-close every connection before reopen, and inject
before/during/after commit, reply loss, busy/full/corrupt schema, duplicate
delivery, migration, and expected-version conflict. In-memory and SQLite
contract outcomes match.

Protected Cargo is `cargo test --locked -p payroll-records-sqlite-draft`. Buck:
//app/payroll/adapters/draft/records-sqlite:payroll-records-sqlite-draft and
//app/payroll/adapters/draft/records-sqlite:payroll-records-sqlite-draft-unittest.
Failure is :memory: recovery evidence, plaintext sensitive
fields, acknowledgement before commit, or dual-write.

</sqlite_records_behavior>

<sqlite_encryption_integration>

## Integrate selected record encryption before production use

Class: behavioral; NON-DISPATCHABLE; hard-follows completion of
`<core_source_budget>`, the accepted encryption decision, its admitted
port/adapter structure and contract behavior, both SQLite structural PRs, and
SQLite records behavior.

The encryption decision must amend this lane with exact existing SQLite item,
test-item, provider adapter, Cargo package, Buck target, and golden paths.
Behavior proves seal-before-write, authenticated open-after-read,
wrong-tenant/purpose/schema refusal, tamper detection, key outage, stale
generation, nonce uniqueness, restart, and no plaintext fallback. Only after
this lane passes may SQLite be called the v1 durable production adapter.

</sqlite_encryption_integration>

<connect_toolchain_decision>

## Select one generated Connect toolchain

Class: repository protocol authority; NON-DISPATCHABLE.

Protocol/API, Build, Gateway, Security, and architecture name an accepted
generated Connect runtime/codegen. The decision must amend this PLAN with:

- exact root/workspace Cargo dependency and Buck rule paths;
- exact protobuf compiler/plugin inputs and pinned versions;
- exact OUT_DIR/Buck generated output identities and freshness checks, none
  committed;
- generated client/server/error types and framing;
- cancellation, deadline, size, field, concurrency, queue, and byte limits;
- exact conformance test and protobuf/error golden paths; and
- license/provenance and rejection of REST, protobuf JSON mapping, gRPC
  trailers/content types, handwritten framing, and a second SDK.

The generated runtime's protocol-defined error framing is permitted. A
hand-authored JSON product error is not. This selects an implementation under
ADR-0719 and does not amend it.

</connect_toolchain_decision>

<facade_staging_decision>

## Resolve structural staging without a fake facade

Class: architecture/build decision; NON-DISPATCHABLE; follows the Connect
toolchain decision and complete core/adapters.

ADR-0719 requires structure and behavior to separate, while owner law forbids
an empty, inert, or permanently-not-ready main. Before dispatch, architecture
must amend this PLAN with a green sequence in which the facade structural head
contains a real bounded boot/configuration/shutdown process and cannot be
mistaken for a served Payroll API. The amendment names every literal Cargo,
Buck, main/lib/module/test path and the exact non-promotion guard. A red
intermediate commit, empty fn main, boot marker, or compatibility listener is
not an admissible solution.

</facade_staging_decision>

<payroll_proto_structure>

## Land the sold schema only

Class: structural; NON-DISPATCHABLE until Connect and facade staging decisions
amend exact generated/test paths.

The fixed schema inputs are:

    app/payroll/facade/proto/payroll/run/v1/payroll_run_service.proto
    app/payroll/facade/proto/payroll/run/v1/BUCK
    app/payroll/facade/proto/payroll/run/v1/OWNERS

The proto defines all eight SPEC RPCs, bounded nested messages, expected
versions, idempotency, semantic error details, and package payroll.run.v1.
BUCK exposes payroll-run-v1-proto. This structural lane contains no handler or
route. The Connect decision must add exact protected Cargo codegen input and
generated-output verification before dispatch. Review: Payroll, HR,
Accounting, Packs, IAM/Policy, Audit, Gateway, protocol/API, Security, Build,
and architecture.

</payroll_proto_structure>

<payroll_proto_conformance>

## Prove schema bytes and generated error framing

Class: behavioral; exact paths are supplied by the Connect/facade staging
amendments before dispatch.

Tests and goldens cover each request/response, field bounds, unknown fields,
same semantic value encoding, generated Connect error details, malformed
frames, bound-plus-one, JSON product body and protobuf-JSON rejection, gRPC
content-type/trailer rejection, deadline, cancellation, and saturation. Cargo
uses the named proto input and generated OUT_DIR only; Buck uses
payroll-run-v1-proto plus the exact contract-test target. No handwritten
framer or checked-in generated output is accepted.

</payroll_proto_conformance>

<genuine_process_behavior>

## Implement the genuine generated-Connect process

Class: behavioral; follows the amended facade structural head, full use cases,
SQLite encryption, selected provider adapters, and proto conformance.

The facade staging amendment supplies literal pre-registered source and test
paths. Behavior implements generated handlers, bounded admission,
identity-to-authorization-evidence ordering, exact adapter selection,
readiness, telemetry, drain/shutdown, replay after reply loss, and all eight
SPEC methods. Tests prove zero core/records calls on identity, tenant,
authorization, pack, overlay, or encryption refusal; success only after
commit; dependency outage/readiness withdrawal burns budget; and only
protobuf Connect succeeds.

Protected Cargo runs the exact facade package and integration tests. Buck runs
the registered binary, unit, boot, wire, authorization, recovery, and
saturation targets. Failure is cloud-core import, static bearer, JSON product
payload, gRPC, success before commit, unbounded work, or fake readiness.

</genuine_process_behavior>

<route_activation>

## Activate the platform route separately

Class: operational/structural; NON-DISPATCHABLE until the repository IR and
Gateway owners name the exact generated route/SLO input and output paths.

Route activation is a separate owner/cross-owner PR after the real process and
failure suites pass. It exposes the same Connect service through the one
platform door; it adds no REST alias, transcode, second listener, or gRPC
route. The amended lane names literal app/payroll/iac IR,
Gateway-consumed generated output, rollback switch, route probe, Cargo/Buck
inputs, and reviewers. Payroll does not edit Gateway-owned files without
occupancy and review.

</route_activation>

<promotion>

## Promote only the complete Payroll product

Class: no-write operational evidence gate after route activation.

Generated SLO outputs, once their accepted IR exists, are exactly:

    app/payroll/observability/slos/authorized-mutation-availability.generated.openslo.yaml
    app/payroll/observability/slos/mutation-latency.generated.openslo.yaml
    app/payroll/observability/slos/trial-calculation-latency.generated.openslo.yaml
    app/payroll/observability/slos/crash-recovery.generated.openslo.yaml
    app/payroll/observability/slos/outbound-intent-delivery.generated.openslo.yaml

Promotion runs calculation goldens/properties, overlay certification, anomaly
resolution, entity/group/production close, every port and adapter, encrypted
SQLite restart/replay, proto/generated Connect, authorization evidence,
delivery, dependency-outage, readiness-withdrawal, saturation, and
failure-injection suites in protected Cargo and target-scoped Buck. It
measures every PRD SLO on the reference cell.

Success is independent APPROVE, green presubmit, full lifecycle evidence, zero
legacy Payroll REST/JSON/OpenAPI paths, RPO-zero acknowledged mutations, and
offered-load SLO accounting that includes dependency/readiness failures.
Failure is promoting today's validation foundation, a docs-only SLO,
hand-authored OpenSLO, skipped fault, hidden compatibility codec, or a current
non-claim promoted by narration. Rollback stops or routes away from the
Connect process without reopening legacy wire or weakening authorization.

</promotion>
