---
id: ADR-0394
title: "First-party Rust internal developer platform (Leptos portal + ops BFF)"
status: Accepted
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0482]
related: [ADR-0001, ADR-0011, ADR-0067, ADR-0090, ADR-0130, ADR-0131, ADR-0132, ADR-0203, ADR-0209, ADR-0213, ADR-0372, ADR-0393, ADR-0482, ADR-0509, ADR-0515, ADR-0615]
related_specs:
  - /specs/http-stack-policy.json
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/platform-architecture.json
  - /specs/masterplan.json
milestone: M-IDP-CENTRAL-HUB
depends_on: [ADR-0393, ADR-0515, ADR-0615]
door: two-way
affected_surfaces:
  crates: [oya-application-shell-frontend-prototype, oya-ops-workspace-shell-kernel, oya-ops-workspace-shell-rest, oya-ops-workspace-shell-app, oya-ops-docs-portal-rest]
  products: [ops-console, developer-portal]
  specs: [/specs/http-stack-policy.json, /specs/hyperscaler-architecture-invariants.json, /specs/masterplan.json]
---

# ADR-0394: First-party Rust internal developer platform

## Status

**Accepted — 2026-08-01.** The founder selected the first-party Rust portal after a deep interview
that reduced residual product ambiguity below five percent. This acceptance closes the portal
substrate choice only. It does not claim rollout, production readiness, or completion, and it does
not by itself lift `HOLD(Planning)` or authorize an execution wave.

The earlier Backstage substrate decision is removed from the live ADR corpus under ADR-0515's
current-truth rule. Git history is the sole archive. This record therefore states the current
decision without carrying a live citation or reciprocal edge to the retired file. This record
explicitly amends ADR-0482's generic parallel-bridge doctrine for the portal: Backstage is limited
to a bounded one-way import source and is never a live or parallel runtime.

## Context

Oyatie needs one discoverable surface for capability ownership, API contracts, SLOs, runbooks,
release state, security posture, cost, developer provisioning, and agent-consumable operations. The
repository already owns the underlying catalog, policy, identity, CI, observability, and application
shell primitives in Rust. A third-party portal runtime would create a parallel authority, duplicate
authentication and authorization, add a Node/React exception to the owned stack, and prevent the
portal from dogfooding the platform it is meant to expose.

The portal is both human- and machine-facing. Every operation available in the UI must resolve to a
stable, policy-gated product API; the UI is not an alternate mutation path and is never the source of
truth.

## Decision

### 1. One first-party portal

Oyatie builds and operates a **first-party Rust internal developer platform** consisting of:

1. a Leptos SSR + hydration portal shell;
2. an owned Rust operations BFF that composes capability APIs without becoming a domain owner;
3. catalog, documentation, scorecard, SLO, runbook, release, incident, cost, and provisioning
   modules over existing canonical sources;
4. stable machine-consumable APIs for every portal workflow;
5. Cedar-gated mutations and server-side identity resolution;
6. OpenTelemetry traces, metrics, and logs for every module and workflow.

Backstage may be consulted as a feature reference or used as a bounded one-way import source during
migration. It is not a runtime dependency, plugin host, catalog authority, deployment substrate, or
supported extension point.

### 2. Capability-first placement

Per ADR-0615, the multi-capability composition lives at:

```text
app/ops-console/developer-portal/
```

Single-capability read and mutation surfaces remain in that capability's `facade/`; reusable
business rules remain in `core/`; the portal shell composes those public ports and owns no duplicate
domain model. The portal does not create a new top-level capability.

### 3. Authority and mutation model

- Repository and service control-plane sources remain authoritative; portal indexes are projections.
- UI mutations call the same versioned APIs used by automation and external clients.
- The portal never writes directly to a capability database, Git repository, Kubernetes API, or
  secret store.
- Every mutation is authenticated, Cedar-authorized, idempotency-keyed where applicable,
  audit-emitting, observable, and represented as a durable operation when it is long-running.
- Agent consumers receive the same contract and authorization semantics as human users.
- No repo-local harness state, hidden CLI, or local-only file becomes a portal authority.

### 4. Initial modules

The first-party portal eventually composes at least:

- capability and API catalog;
- documentation and ADR/search projection;
- ownership, maturity, scorecard, and SLO views;
- CI/CD, release, rollout, and incident views;
- runbooks, observability links, and cost allocation;
- developer accounts, credentials, SDKs, sandbox tenants, webhooks, and marketplace submissions;
- agent mission, provider health, dispatch, and replay views through governed platform APIs.

Each module must ship independently behind an owned port and may be disabled without breaking the
rest of the portal. Partial downstream failure degrades the affected module rather than the shell.

### 5. Dependent decision retargeting

This acceptance atomically retargets the load-bearing dependents:

- ADR-0203: federated documentation is rendered by the first-party docs-portal module;
- ADR-0209: the read-only auditor surface is a first-party portal module over compliance and audit
  APIs;
- ADR-0213 and its developer-sdk decision: developer workflows are first-party portal modules, not
  third-party plugins;
- any legacy Backstage chart or plugin skeleton is nonbinding migration inventory and must be
  deleted rather than moved during its capability batch.

## Non-goals

- Reimplementing capability business logic in the portal.
- Building a second policy engine, identity provider, service catalog, or workflow engine.
- Treating a UI route, local CLI, or generated projection as an authority.
- Claiming all modules are already implemented or production-promoted.

## Verification obligations

1. Portal placement satisfies capability-membership and facade/core layering gates.
2. Every mutation path proves server-side authn/authz, audit emission, idempotency or durable-operation
   semantics, and rollback behavior.
3. Contract tests prove UI and automation use the same versioned API surface.
4. Browser evidence covers keyboard navigation, WCAG 2.2 AA, responsive layouts, degraded modules,
   and the critical developer provisioning journey.
5. OTel evidence covers latency, errors, saturation, authorization denials, and downstream
   degradation.
6. Repository scans prove no Backstage runtime, plugin package, or deployment chart is promoted as a
   live authority.

## Alternatives considered

### Backstage runtime

Rejected. It introduces a parallel Node/React/plugin substrate, duplicates owned platform
capabilities, and cannot be the dogfood path for the Rust application shell and control plane.

### Per-product portals

Rejected. They fragment discovery, authorization, and operational workflows. Product-specific UX
belongs in modules mounted into the shared shell.

### Static documentation only

Rejected. It cannot support governed provisioning, credentials, sandbox lifecycle, incident, or
release operations.

## Consequences

- The platform owns the portal lifecycle, reliability, accessibility, and security posture.
- The portal remains a projection-and-composition layer, keeping capability boundaries intact.
- Existing Rust catalog, identity, policy, SLO, CI, and observability primitives are reused rather
  than wrapped behind a third-party authority.
- Legacy Backstage artifacts are removal candidates in their capability move batches; acceptance is
  not evidence that those batches have completed.
