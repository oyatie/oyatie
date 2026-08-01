---
id: ADR-SDK-0007
title: "Developer workflows ship as a module of the first-party portal"
status: Accepted
date: 2026-08-01
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0243
  - ADR-0244
  - ADR-0258
  - ADR-0263
  - ADR-0394
decision_owner: axis-ecosystem + council-architecture
---

# ADR-SDK-0007: Developer workflows ship as a first-party portal module

## Status

**Accepted — 2026-08-01.** This decision conforms the developer-sdk surface to the accepted
first-party portal substrate in ADR-0394. It replaces the unaccepted third-party-plugin proposal in
this decision slot; Git history preserves that proposal.

## Context

Developer-sdk needs governed workflows for API keys, SDK downloads, docs, sandbox tenants, webhook
events, payout status, tax forms, KYC, marketplace submissions, and support cases. A separate web app
would duplicate the application shell, identity, Cedar authorization, catalog, and observability
stack. A third-party plugin host would create the same parallel authority under a different name.

## Decision

- Developer workflows mount as a module under `app/ops-console/developer-portal/`.
- The module uses the canonical Leptos application shell and owned Rust operations BFF.
- Developer-sdk APIs remain authoritative; the portal owns no developer-sdk business state.
- Authentication is resolved server-side and every mutation is Cedar-gated.
- API key, sandbox, webhook, payout, tax, KYC, and marketplace operations call versioned
  developer-sdk APIs through typed clients.
- Long-running work returns a durable operation resource; UI polling or streaming never substitutes
  for operation state.
- The module exposes SDK release manifests, signing keys, sandbox lifecycle, delivery attempts, and
  marketplace status without exposing raw KYC, bank, or tax evidence.
- State-changing actions emit `DeveloperPortalActionInvoked` audit events and OTel spans.
- The module supports WCAG 2.2 AA and degrades individual cards when a downstream capability is
  unavailable.
- The module and APIs must work in self-hosted deployments without a third-party portal runtime.

## Contract and safety requirements

- Browser state must never contain raw KYC evidence, bank details, or unredacted tax identifiers.
- Every request carries explicit developer-account and tenant scope resolved by the server.
- Every generated client pins the requested API version.
- Portal summary endpoints are read-only; state-changing endpoints stay in their owning capability.
- The portal is not an authorization oracle. Cedar decisions are made by the server-side policy
  decision point.
- Partial downstream failure cannot take down unrelated developer workflows.

## Verification

- identity maps to the correct developer account and tenant scope;
- summary endpoints cannot mutate state;
- sandbox creation, webhook replay, payout reads, and marketplace submission invoke the correct
  Cedar action;
- sensitive fields are absent or redacted in browser state, logs, and traces;
- API-version pins are sent on every call;
- degraded downstream cards do not fail the portal shell;
- keyboard, screen-reader, contrast, and responsive browser tests satisfy WCAG 2.2 AA;
- OTel metrics cover page load, API latency, Cedar denials, route errors, and downstream degradation.

## Consequences

- Developer docs and operational workflows share the first-party portal without creating a second
  authority.
- The developer-sdk surface remains modular while using the same shell, identity, policy, and
  observability substrate as other operator workflows.
- Legacy third-party portal charts and plugin packages are deleted during the capability move rather
  than preserved as runtime options.
