---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P12
title: Provider-Agnosticism + Adapter Discipline
status: in-progress
purpose: Provider-specific code lives in `oya-<context>-adapter-<provider>-*` crates only; kernels stay provider-neutral.
---

# M01-P12 — Provider-Agnosticism

## Purpose
Per MASTERPLAN §2 Directive 4. The Foundry multi-provider adapter pattern (Claude/OpenAI/Gemini) is the canonical implementation; Cloud/KMS/storage/network/observability/secrets/identity follow.

## Acceptance
- `oya-governance-provider-coupling` lane CI-blocks provider-specific imports outside adapter crates.
- Per-kernel ≥ 2 provider adapters (e.g., Cloud KMS ships AWS-KMS + GCP-KMS + OpenBao adapters; Cloud Storage ships S3 + GCS + Azure-Blob adapters).
- Adapter-substitution test: kernel API contract identical regardless of adapter; integration tests run against ≥ 2 adapters.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Provider-coupling lane kernel | complete | [`IP-001-provider-coupling-lane.md`](IP-001-provider-coupling-lane.md) |
| IP-002 | Multi-provider adapter audit + remediation across Cloud kernels | split-required-too-broad-for-single-changeset | [`IP-002-cloud-multi-provider-audit.md`](IP-002-cloud-multi-provider-audit.md) |
| IP-002.1 | Multi-provider adapter audit (Part 1: Billing & Marketplace) | complete | [`IP-002.1-audit-billing-marketplace.md`](IP-002.1-audit-billing-marketplace.md) |
| IP-002.2 | Multi-provider adapter audit (Part 2: Capacity & Data) | complete | [`IP-002.2-audit-capacity-data.md`](IP-002.2-audit-capacity-data.md) |
| IP-002.3 | Multi-provider adapter audit (Part 3: Finops & Observability) | complete | [`IP-002.3-audit-finops-observability.md`](IP-002.3-audit-finops-observability.md) |
| IP-003 | Adapter-substitution integration-test harness | complete | [`IP-003-adapter-substitution-harness.md`](IP-003-adapter-substitution-harness.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-governance-provider-coupling-kernel`, `crates/oya-cloud-*-adapter-*-*`, `tools/oya-adapter-substitution-test`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P12 complete: provider-coupling lane green; ≥2 adapters per kernel; substitution harness running" -i critical -k "M-CC,P05,provider-agnosticism,complete"
```
