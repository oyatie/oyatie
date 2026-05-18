---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-011
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

# IP-011 — Cedar policy fragments

## Intent

Land Cedar fragments at `policy/*.cedar` and the supporting `schema.cedarschema`. Wire fragment coverage CI gate.

## Scope

- `policy/tenant-scope.cedar`
- `policy/ci-scope.cedar`
- `policy/auditor-scope.cedar`
- `policy/public-read.cedar`
- `policy/schema.cedarschema`
- CI lane `cedar-fragment-coverage-community` (lean-a7 family).

## Deliverables

- Fragments authored.
- Schema authored.
- CI lane added.

## Acceptance

- Cedar compile green.
- Every action declared in `community.proto` has either a `permit` or explicit `forbid` clause.
- Coverage CI lane green.
- Negative-test suite green (cross-tenant attempt → deny + audit event).

## Owner

ops-security.
