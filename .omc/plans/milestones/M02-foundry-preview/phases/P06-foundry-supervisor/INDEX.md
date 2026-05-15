---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P06
title: Foundry Supervisor
status: complete
plan_refs:
- v4: /Users/jasonlee/oyatie/.omc/plans/ralplan-foundry-supervisor-simple-v4-2026-05-14.md
- v5: /Users/jasonlee/oyatie/.omc/plans/ralplan-foundry-supervisor-simple-v5-delta-settings-template-2026-05-15.md
- v6: /Users/jasonlee/oyatie/.omc/plans/ralplan-foundry-supervisor-simple-v6-amendments-2026-05-15.md
synthesis_evidence: evidence/debate/M02-P06-FOUNDRY-SUPERVISOR-2026-05-15-synthesis.json
purpose: "Multi-account, multi-provider session supervisor implementing the hook + inbox/outbox JSONL pattern (Option D: host-injected policy ports)."
---
# M02-P06 — Foundry Supervisor

Multi-account, multi-provider session supervisor implementing the hook + inbox/outbox JSONL pattern
(Option D: host-injected policy ports).

## Crates ✅

| Crate | Layer | Purpose |
|-------|-------|---------|
| `oya-foundry-supervisor-kernel` | kernel (L1) | Value types, port traits, pure decision logic |
| `oya-foundry-supervisor-app` | app (L5) | Daemon binary, call chain, hyper webhook |
| `oya-foundry-jsonl-supervisor-adapter` | adapter (L4) | File-backed InboxStore + OutboxSink |
| `oya-foundry-supervisor-conformance` | conformance | build.rs seed + read-back tests |
| `oya-foundry-settings-template-kernel` | kernel (L1) | Settings template value types (v5 delta) |
| `oya-foundry-settings-template-adapter` | adapter (L4) | Per-provider settings renderer (v5 delta) |
| `oya-foundry-claude-account-adapter` | adapter (L4) | Claude CLI driver |
| `oya-foundry-codex-account-adapter` | adapter (L4) | Codex CLI driver |
| `oya-foundry-gemini-account-adapter` | adapter (L4) | Gemini CLI driver |
| `oya-foundry-supervisor-security-adapter` | adapter (L4) | OpenBao + Cedar bridge |

## Wave sequence ✅

- **Wave 1**: scaffold — 10 crate skeletons + workspace registration ✅
- **Wave 2**: core implementation — kernel types, JSONL adapter, daemon, benchmarks ✅
- **Wave 3**: driver integration — Claude, Codex, Gemini CLI subprocesses ✅
- **Wave 4**: security integration — OpenBao secrets + Cedar autonomy ceilings ✅
- **Safety Blockers**: atomic writes, symlink defense, audit-chain (ADR-0003) ✅

## Acceptance

- All 10 crates build and verify via `cargo build --workspace`
- 22+ supervisor-specific unit and integration tests passing
- `RB-SUPERVISOR-001` runbook authored
- 7 new ADRs (0096-0102) ratified and indexed
- Multispectrum synthesis complete: `M02-P06-FOUNDRY-SUPERVISOR-2026-05-15`
