# Foundry README

Service: foundry
Business capability: hosted-agent platform
Date: 2026-05-21
Doc class: README

## Tenant Class And Billing
Foundry adopts the ADR-0330 tenant_class model. The service is available to `demo_trial` and `paid` tenants with a uniform runtime, evaluation, guardrails, provider-routing, supervision, and evidence surface. Demo_trial tenants are constrained by usage and time caps. Paid tenants use composable `billing_components`; Foundry emits `per_seat` for operator and developer seats plus `per_usage` for runtime invocation, evaluation, evidence, vector-search, and provider-routing consumption.

## Scope
Foundry remains one flat µservice with internal bounded contexts for runtime, supervisor, eval, evidence, guardrails, providers, and vector substrate. Tenant_class is consumed from principal claims at the request boundary and enforced by Cedar fragments; per-µservice contracts do not introduce a customer-facing capability ladder.
