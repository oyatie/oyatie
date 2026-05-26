---
doc_class: DPIA
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: council-privacy + axis-foundry
jurisdiction: pack-kr (M01 launch); pack-eu (subsequent-to-M01-completion); pack-us / pack-us-healthcare (subsequent-to-M01-completion)
related_adrs: [ADR-0117, ADR-0136, ADR-0137]
---

# Data Protection Impact Assessment — foundry (consolidated)

## Scope

This DPIA covers the foundry µservice end-to-end. Per-BC DPIAs are preserved
at `bc-sources/<bc>/dpia.md` and remain authoritative for BC-internal data
flows. This document covers cross-BC data flows and aggregate risk.

## Personal data inventory

| Data category | BC of residence | Lawful basis | Retention | Cross-pack flow |
|---|---|---|---|---|
| Session conversation content (may incl. tenant-supplied PII) | runtime | Tenant contract (Art.6(1)(b)) + tenant data-processing agreement | 30d hot (Valkey) + per-tenant cold retention (Postgres) | Forbidden (per-pack) |
| Invocation metadata (tenant_id, capability_id, started_at, status) | runtime + evidence | Legitimate interest (operations + audit) | 1y baseline; 6y for pack-us-healthcare (HIPAA) | Forbidden (per-pack) |
| Supervision command log | supervisor + evidence | Legal obligation (audit) | 1y baseline | Forbidden (per-pack) |
| Guardrail decision record (hash(prompt), decision, version) | guardrails + evidence | Legitimate interest (safety) | 1y baseline | Forbidden (per-pack) |
| Provider receipt (provider, model, tokens, cost) | providers + evidence | Legitimate interest (billing + ops) | 1y baseline | Forbidden (per-pack) |
| Eval-run record (capability, baseline_id, parity_outcome) | eval | Legitimate interest (quality) | 1y baseline | Forbidden (per-pack) |
| Evidence pack (sealed bundle) | evidence | Legal obligation (audit-chain) | 6y default; per-pack overlays | Forbidden (per-pack) |

## Cross-BC data flows

1. **Invocation flow**: Tenant SDK → runtime → guardrails (pre) → providers
   → guardrails (post) → evidence → runtime → Tenant SDK. Session content
   touches: runtime, guardrails (hash only), providers (transient pass-
   through), evidence (sealed).
2. **Eval replay**: eval reads a sealed invocation record from evidence,
   replays through runtime sandbox pool. Sandbox refuses production session
   binding.
3. **Regulator export**: evidence builds a pack per (tenant, period, scope)
   and exports through a Cedar-scoped, signed envelope. Cross-tenant
   leakage refused by Cedar.

## Data Subject Rights (DSR)

- **Access**: Tenant operators read their own invocation history via runtime
  SDK + evidence query endpoint.
- **Rectification**: Capability descriptors are tenant-owned; tenant edits
  through supervisor → propagate to runtime cache within ≤30s.
- **Erasure (right-to-be-forgotten)**: TenantDsrCascade event scans all 6
  BCs for affected subject identifiers; soft-deletes within 30 days per
  Bominal ADR-NNN. Audit-chain retention exception per Art.17(3)(b)
  (compliance with legal obligation).
- **Restriction**: Per-tenant kill-switch (supervisor) halts invocation
  while preserving session-state.
- **Portability**: Tenant invocation history exportable as JSON via
  evidence/regulator-export endpoint scoped to tenant.
- **Object**: Per ADR-0117, tenant may revoke jurisdiction binding;
  per-pack flow forbids cross-pack so revocation is a tenant-side delete.

## Privacy by design

- **Minimisation**: Guardrail decisions store hash(prompt), not prompt.
  Provider receipts store metadata, not request/response bodies. Evidence
  packs link by hash to runtime session blobs.
- **Purpose limitation**: Each BC's port traits declare data classes;
  cross-BC traffic refuses unsupported classes (Cedar + data-class lane).
- **Storage limitation**: Retention windows per data category enforced by
  TTL on Valkey + lifecycle policies on Postgres + S3.
- **Integrity**: Audit-chain (Ed25519+Merkle) seals every cross-BC state
  transition.
- **Confidentiality**: mTLS internal; TLS external; Redis/Postgres/S3
  encryption at rest with per-pack KMS keys.
- **Accountability**: Per-BC owner_team declared in `bc-sources/<bc>/PRD.md`
  frontmatter; foundry top-level owner: axis-foundry.

## Residual risks (cross-BC)

| # | Risk | Likelihood | Impact | Mitigation in place | Residual |
|---|---|---|---|---|---|
| 1 | Session content leak via cross-tenant Valkey collision | Low | High | Per-tenant key prefix + Cedar | Low |
| 2 | Evidence pack assembly omits a BC's contribution | Low | Medium (audit incompleteness) | AC-X7 cross-BC pack e2e | Low |
| 3 | DSR cascade misses a BC | Medium | High | Per-BC `bc-sources/<bc>/dpia.md` enumerates DSR endpoints; cascade event consumed by all 6 BCs | Low |
| 4 | Cross-pack migration via misconfigured kustomize | Low | High | Per-pack overlay validated by `oya gate validate per-pack-residency` | Low |
| 5 | Provider receipt leaks token-count metadata across tenants | Low | Low | Per-tenant scope at provider router | Very low |

## Per-BC DPIA archives

- `bc-sources/runtime/dpia.md` — session-state + invocation DSR cascade.
- `bc-sources/supervisor/dpia.md` — supervision command log DSR cascade.
- `bc-sources/eval/dpia.md` — synthetic-only PHI rule + replay sandbox DSR.
- `bc-sources/evidence/dpia.md` — evidence-pack DSR + retention exceptions.
- `bc-sources/guardrails/dpia.md` — guardrail-decision-log DSR; hash-only
  prompt storage.
- `bc-sources/providers/dpia.md` — provider receipt DSR + credential
  isolation invariant.

## References

- ADR-0117: Data-residency + jurisdiction codes.
- ADR-0136 / ADR-0137: foundry topology authority.
- GDPR Art.6, 17, 30, 35.
- HIPAA §164.316.
- PIPA Art.23.
