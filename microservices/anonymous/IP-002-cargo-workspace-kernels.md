---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-002-cargo-workspace-kernels
status: pending
execution_unit: ChangeSet
owner: axis-anonymous
acceptance_lanes: [cargo-check, cargo-build, oya-governance-per-microservice-layout, oya-governance-layer-purity]
---

# IP-002: Cargo workspace + kernel crates (per BC, zero-I/O)

## Intent

Establish the Cargo workspace under `microservices/anonymous/src/` and create 11 kernel crates (one per BC). Kernels are pure types + traits with zero I/O — they define the port-trait surface the adapters will implement.

## ChangeSet boundary

| BC | kernel crate |
|---|---|
| pseudonymous-identity | `oya-anonymous-pseudonymous-identity-kernel` |
| affinity-attestation | `oya-anonymous-affinity-attestation-kernel` |
| blind-signatures | `oya-anonymous-blind-signatures-kernel` |
| post-thread | `oya-anonymous-post-thread-kernel` |
| feed-timeline | `oya-anonymous-feed-timeline-kernel` |
| vote-engine | `oya-anonymous-vote-engine-kernel` |
| content-moderation | `oya-anonymous-content-moderation-kernel` |
| legal-process-disclosure | `oya-anonymous-legal-process-disclosure-kernel` |
| anonymous-dm | `oya-anonymous-anonymous-dm-kernel` |
| retention-policy | `oya-anonymous-retention-policy-kernel` |
| report-and-moderate | `oya-anonymous-report-and-moderate-kernel` |

## Kernel-trait sketch

Each kernel exports:
- entity structs (e.g., `Post`, `BlindedCredential`, `AffinityAttestationBinding`)
- port traits (e.g., `PostStore`, `BlindSignatureIssuer`)
- error enums
- `data_class` annotations per Bominal ADR-0028

## Acceptance

- 11 kernel crates compile under `cargo check --workspace`
- `oya-check-layer-purity` lane verifies kernels have zero project-internal deps
- `oya-check-data-class` verifies every entity field is annotated
