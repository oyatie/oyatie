---
purpose: |
  Cloud-native infrastructure automation standard: API-shaped Rust components,
  declarative configuration, idempotent reconciliation, observable execution,
  and deployment-compatible delivery; rejects new ad-hoc CLIs and new
  Python/shell core infra automation.
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-07-01
purpose: |
  Defines the allowed and disallowed shapes for cloud-native, API-driven
  infrastructure automation. Reviewers use it to reject new ad-hoc CLIs,
  Python scripts, shell scripts, and host-local workflows for core
  infrastructure behavior, and to steer work toward Rust, APIs, config,
  idempotency, observability, and deployment-compatible controllers/gates.
canonical_authority: docs/decisions/ADR-0700-ci-admission-live-apex.md
enforced_by: presubmit/gate-rust-first-automation-hygiene
companion_docs:
  - docs/AGENTS.md
  - docs/standards/api-design.md
  - docs/standards/openapi-3-2-authoring.md
  - docs/standards/idempotency-keys-canonical.md
  - docs/standards/observability.md
  - specs/deployment-ops-contract.json
related_adrs:
  - ADR-0515
  - ADR-0523
  - ADR-0540
---

# Cloud-Native Infrastructure Automation

## Doctrinal authority

ADR-0515 makes Oyatie cloud-native: infrastructure behavior is pipeline- and
API-shaped, not an operator's collection of local commands. The key words MUST,
MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT
RECOMMENDED, MAY, and OPTIONAL in this document are interpreted as described in
RFC 2119 and RFC 8174 when they appear in all capitals.

## 1. Scope

This standard applies to infrastructure workflows that provision, reconcile,
verify, deploy, roll back, meter, govern, or evidence cloud resources, clusters,
CI/CD gates, GitOps state, tenant cells, quotas, secrets, keys, network policy,
or control-plane resource lifecycle.

It applies to new work and to changes that extend existing infrastructure
behavior. Existing legacy CLIs or scripts are migration/provenance surfaces only;
a change MAY delete, quarantine, or port them, but MUST NOT extend them as the
canonical behavior path.

It does not prohibit test fixtures, one-off read-only audits, or developer
experiments when they are outside the production/control-plane path and cannot
produce merge, deploy, rollback, or governance authority. Those aids MUST still
avoid becoming the only documented acceptance path.

## 2. Required target shape

Core infrastructure behavior MUST be one of these shapes:

1. A Rust API component: library, service, controller, operator, or gate binary
   with typed inputs/outputs and tests.
2. A declarative configuration or manifest consumed by an existing Rust/API
   engine, controller, or GitOps reconciler.
3. A Kubernetes-native controller/CRD/reconciler that exposes desired state,
   observed state, conditions, and events.
4. A pipeline/`presubmit` gate packet backed by Rust logic and policy as
   data, not by a CLI invocation transcript.
5. A documented API contract: OpenAPI, proto, AsyncAPI, JSON Schema, CRD schema,
   or equivalent typed contract that callers and reviewers can inspect.

Every accepted shape MUST be configuration-driven, idempotent, observable, and
compatible with the live deployment path and the planned owned-runner path.

## 3. Disallowed shapes

New infrastructure behavior MUST NOT be implemented as:

1. A new ad-hoc CLI for provisioning, deployment, rollback, gate enforcement,
   evidence production, or tenant/cell/resource lifecycle.
2. A new Python, shell, Groovy, Makefile, or host-local script that owns core
   infrastructure behavior.
3. A README/runbook whose authoritative step is "run this local command" when an
   API, gate, controller, or declarative manifest could own the behavior.
4. A Kubernetes Job, init container, or hook whose core logic is `bash -c`, a
   Python script, or a mutable laptop/runner environment.
5. A workflow that stores hidden state in a developer machine, a CI workspace, or
   an untyped file outside the declared control-plane data model.
6. A manual `kubectl`, cloud-console, or SSH procedure as the normal path for
   provisioning, repair, or rollback.

Thin local inspection tools MAY exist only when they call the same public API or
read the same declarative status as every other actor, carry no exclusive
capability, and are not the acceptance, merge, deploy, or governance authority.
Adding such a bridge still needs an explicit review rationale.

## 4. Required properties

### 4.1 API-shaped Rust component

The implementation SHOULD be Rust-first. It MUST expose a typed boundary rather
than a sequence of process calls. Acceptable boundaries include Rust traits,
OpenAPI/proto surfaces, CRD schemas, JSON Schema policy packets, or pipeline gate
packet schemas.

### 4.2 Configuration-driven behavior

Behavior MUST be controlled by versioned config, policy, or manifest data. New
boolean flags, environment variables, or path conventions MUST have a schema,
default, owner, migration story, and reviewable examples.

### 4.3 Idempotency and convergence

Mutating operations MUST be safe to retry. API mutations use idempotency keys or
stable operation IDs; reconcilers compare desired state to observed state and
emit no duplicate side effects when re-run. A reviewer must be able to answer:
"what happens if this request, gate, sync, or rollout runs twice?"

### 4.4 Observability and auditability

Automation MUST emit structured logs, metrics, traces, and audit events
proportional to its blast radius. Every long-running or asynchronous operation
MUST expose status, failure reason, retry posture, and correlation IDs. Silent
success and silent failure are both defects.

### 4.5 Deployment compatibility

The path MUST work in the current protected pipeline and in the destination
owned-runner/GitOps posture without changing its semantics. It MUST NOT depend on
interactive terminals, local paths, ambient credentials, manually ordered shell
steps, or a single mutable cluster. Rollback and canary/degraded-mode behavior
MUST be part of the same API/config/controller contract.

## 5. Examples

### 5.1 Add a CI policy

Acceptable: a Rust gate binary reads a JSON policy fixture and emits a gate
packet consumed by `presubmit`.

Unacceptable: `scripts/check-policy.sh` exits 0/1 and becomes the only required
check.

### 5.2 Provision a cell resource

Acceptable: a CRD/schema plus Rust reconciler exposes desired/observed state,
conditions, an idempotent operation ID, OTel, and rollback condition.

Unacceptable: `python cloud/create_cell.py --tenant ...` creates resources
directly from a laptop or runner.

### 5.3 Change deployment topology

Acceptable: a declarative GitOps manifest is consumed by ArgoCD/owned CD with
schema validation and status projection.

Unacceptable: a runbook says to run `kubectl apply` commands in order during
every deploy.

### 5.4 Add tenant quota enforcement

Acceptable: an API-shaped Rust component reads versioned quota config and
returns typed decisions with audit events.

Unacceptable: a cron shell script scans tenants and patches quota YAML as the
source of truth.

### 5.5 Debug infrastructure state

Acceptable: a read-only diagnostic view calls the same status API as the console
and cannot mutate state.

Unacceptable: a new `oya infra fix` command performs repair logic unavailable
through the API/controller.

## 6. Review guidance

Reject or request redesign when any answer is "yes":

1. Does the change add a new CLI, Python script, shell script, Makefile target,
   or local command as the core workflow?
2. Is the behavior only described as ordered imperative steps instead of an API,
   controller, gate, or declarative manifest?
3. Can the workflow produce merge, deploy, rollback, evidence, or governance
   authority outside `presubmit`, pipeline gate packets, or the API/control
   plane?
4. Does retrying the operation risk duplicate resources, duplicate events,
   partial rollback, or inconsistent state?
5. Is configuration untyped, undocumented, or hidden in ambient environment?
6. Are logs/traces/metrics/audit events absent from the success and failure
   paths?
7. Does the workflow depend on a laptop, interactive shell, SSH session, manual
   `kubectl`, or mutable CI workspace?
8. Would a future owned runner, GitOps controller, or tenant cell need a second
   implementation to do the same work?

Approve only when the change names its typed surface, config schema, retry model,
observability evidence, deployment/rollback compatibility, and any legacy debt it
retires or deliberately leaves untouched.

## 7. Automated enforcement

The blocking CI backstop is the `presubmit` matrix leg
`gate · rust-first automation hygiene`, backed by the Buck2-native
`//cloud/cloud-ci/gates/pipeline-rust-first-automation-hygiene-app` gate.
The gate fails on unregistered non-Rust automation files, new GitHub workflow
inline shell beyond the frozen baseline, forbidden workflow action bridges,
Rust code that spawns retired interpreters, and new cloud/infra/tooling Cargo
packages shaped as `*-cli`.

Exceptions are data, not tribal knowledge: add them only in
`cloud/cloud-ci/gates/pipeline-rust-first-automation-hygiene-app/rust-first-automation-policy.json`
with `reason`, `replacement`, and `status`. The replacement MUST point toward a
Rust Buck2/pipeline gate, Kubernetes/GitOps/controller path, or equivalent
API-shaped cloud-native surface, and stale exceptions must be removed with the
same change that retires the legacy file or workflow step.

## 8. Sources scanned

- [ADR-0515 — Phase-0 firewall + one-canonical-CI + cloud-native posture](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md).
- [AGENTS.md](../AGENTS.md) agent operating contract and CLI retirement note.
- [API design](api-design.md), [OpenAPI 3.2 authoring](openapi-3-2-authoring.md),
  [idempotency keys](idempotency-keys-canonical.md), and
  [observability](observability.md) standards.
- [Deployment ops contract](../../specs/deployment-ops-contract.json).
- [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) and
  [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174).
