---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P02
title: Identity Kernel + Cedar Policy Substrate
status: stub
purpose: Ship `oya-platform-identity-*` and the Cedar RBAC/ABAC substrate that every capability invocation enforces against.
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
| IP-001 | `oya-platform-identity-kernel` user + region + IdP binding | stub | [`IP-001-identity-kernel.md`](IP-001-identity-kernel.md) |
| IP-002 | `oya-platform-identity-app` STS issuance + rotation | stub | [`IP-002-sts-rotation.md`](IP-002-sts-rotation.md) |
| IP-003 | Cedar policy substrate (`-policy-cedar-*`) + publish + supersession | stub | [`IP-003-cedar-policy-substrate.md`](IP-003-cedar-policy-substrate.md) |

## Estimated parallelism
3 agents in parallel; IP-001/002 share kernel crate (sequence within), IP-003 is disjoint.

## Symbols-touched
`crates/oya-platform-identity-{kernel,domain,app,api}-*`, `crates/oya-platform-policy-cedar-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P02 complete: identity kernel + Cedar substrate; STS ≤1h; policy publish versioned" -i critical -k "M01,P02,identity,cedar,complete"
```
