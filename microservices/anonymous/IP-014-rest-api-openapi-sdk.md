---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-014-rest-api-openapi-sdk
status: pending
execution_unit: ChangeSet
owner: axis-anonymous
acceptance_lanes: [cargo-check, openapi-lint, sdk-build, contract-conformance]
---

# IP-014: REST API surface + OpenAPI 3.2.0 contract + SDK generation

## Intent

Generate REST handlers + tenant SDK from the canonical OpenAPI 3.2.0 contract at `contracts/openapi/anonymous.yaml`. Author rest + sdk crates per BC. SDK generated for: Rust (native), TypeScript, Python.

## Acceptance

- OpenAPI lint passes (Spectral)
- Contract conformance test (handlers match contract; SDK matches contract)
- SDK builds for all 3 target languages
- BBS+ proof header convention enforced
