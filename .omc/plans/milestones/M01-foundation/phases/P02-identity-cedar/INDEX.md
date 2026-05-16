---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P02
title: Identity Kernel + Cedar Policy Substrate
status: complete
purpose: Ship `oya-identity-*`, `oya-platform-identity-api`, and the Cedar RBAC/ABAC substrate that every capability invocation enforces against.
phase_evidence_refs:
  - /evidence/foundation/m01-p02-ip-001-identity-kernel.json
  - /evidence/foundation/m01-p02-ip-002-sts-rotation.json
  - /evidence/foundation/m01-p02-ip-003-cedar-policy-substrate.json
---

# M01-P02 — Identity Kernel + Cedar Policy Substrate

## Purpose
Per-tenant user upsert, STS short-lived credentials, Cedar policy publish surface — the substrate every other axis's authorization decision binds to.

## Acceptance
- `identity.user.upsert` and `identity.token.issue` SPEC §2 rows green at `stable`.
- `cedar.policy.publish` rows green; versioned + semver + per-tenant-or-global scope + supersession chain.
- STS tokens always ≤ 1h purpose-bound; no long-lived API keys.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | `oya-identity-domain` user + region + IdP binding | complete | [`IP-001-identity-kernel.md`](IP-001-identity-kernel.md) |
| IP-002 | `oya-identity-application` STS issuance + rotation | complete | [`IP-002-sts-rotation.md`](IP-002-sts-rotation.md) |
| IP-003 | Cedar policy substrate (`oya-policy-cedar-domain` + `oya-platform-policy-cedar-api`) + publish + supersession | complete | [`IP-003-cedar-policy-substrate.md`](IP-003-cedar-policy-substrate.md) |

## Estimated parallelism
IP-002 and IP-003 are unblocked after IP-001, but shared workspace/doc/Cargo surfaces stay serialized by the owning ChangeSet.

## Symbols-touched
`crates/oya-identity-domain`, `crates/oya-identity-application`, `crates/oya-platform-identity-api`, `crates/oya-policy-cedar-domain`, `crates/oya-platform-policy-cedar-api`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P02 complete: identity kernel + Cedar substrate; STS ≤1h; policy publish versioned" -i critical -k "M01,P02,identity,cedar,complete"
```
