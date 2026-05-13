---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P05
title: Provider-Agnosticism + Adapter Discipline
status: stub
purpose: Provider-specific code lives in `oya-<context>-adapter-<provider>-*` crates only; kernels stay provider-neutral.
---

# M-CC-P05 — Provider-Agnosticism

## Purpose
Per MASTERPLAN §2 Directive 4. The Foundry multi-provider adapter pattern (Claude/OpenAI/Gemini) is the canonical implementation; Cloud/KMS/storage/network/observability/secrets/identity follow.

## Acceptance
- `oya-foundry-fitness-provider-coupling` lane CI-blocks provider-specific imports outside adapter crates.
- Per-kernel ≥ 2 provider adapters (e.g., Cloud KMS ships AWS-KMS + GCP-KMS + OpenBao adapters; Cloud Storage ships S3 + GCS + Azure-Blob adapters).
- Adapter-substitution test: kernel API contract identical regardless of adapter; integration tests run against ≥ 2 adapters.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Provider-coupling lane kernel | stub | [`IP-001-provider-coupling-lane.md`](IP-001-provider-coupling-lane.md) |
| IP-002 | Multi-provider adapter audit + remediation across Cloud kernels | stub | [`IP-002-cloud-multi-provider-audit.md`](IP-002-cloud-multi-provider-audit.md) |
| IP-003 | Adapter-substitution integration-test harness | stub | [`IP-003-adapter-substitution-harness.md`](IP-003-adapter-substitution-harness.md) |

## Estimated parallelism
3 agents.

## Symbols-touched
`crates/oya-foundry-fitness-provider-coupling-kernel`, `crates/oya-cloud-*-adapter-*-*`, `tools/oya-adapter-substitution-test`.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P05 complete: provider-coupling lane green; ≥2 adapters per kernel; substitution harness running" -i critical -k "M-CC,P05,provider-agnosticism,complete"
```
