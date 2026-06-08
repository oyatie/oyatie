---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M02-P00
title: Account-Auth Contracts (Phase 00 lift)
status: complete
purpose: Lift Foundry Phase 00 account-auth contract surface from foundry-salvage into `oyatie/docs/products/foundry/PHASE-00-SPEC.md` and ship the seven `oya-intelligence-account-*` crates under Clean Architecture.
---

# M02-P00 — Account-Auth Contracts

## Purpose
Per [`../../../../../.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md`](../../../../../.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md) §A-B. Foundation for Foundry's multi-provider runtime.

## Acceptance
- P00-01 architecture skeleton: 7 crates green (`kernel`, `domain`, `app`, `adapter-{codex-cli,claude-code,gemini-cli,openbao}`, `runtime`); boundary check passes.
- P00-02 domain types: 40+ unit tests; state machine `Draft → Verified → Active → Degraded → Disabled → Revoked` (terminal); allowlist enforced; silent-account-switch detection.
- P00-03 secret persistence: SecretStorePort + local OpenBao adapter (provider-agnostic); secret redaction tests.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Clean Architecture skeleton + 7-crate scaffold-claim (ADR-0054 path) | complete | [`IP-001-clean-arch-skeleton.md`](IP-001-clean-arch-skeleton.md) |
| IP-002 | Domain types + state machine + 40+ unit tests (P00-02) | complete | [`IP-002-domain-types-state-machine.md`](IP-002-domain-types-state-machine.md) |
| IP-003 | SecretStorePort + OpenBao adapter (provider-agnostic) | complete | [`IP-003-secret-store-port.md`](IP-003-secret-store-port.md) |

## Estimated parallelism
IP-001 serializes (scaffold-claim under `scaffold-locks-oyatie`); IP-002 + IP-003 fan out to 2-3 agents after IP-001 merge.

## Symbols-touched
`crates/oya-intelligence-account-{kernel,domain,app,adapter-codex-cli,adapter-claude-code,adapter-gemini-cli,adapter-openbao,runtime}-*`, `docs/products/foundry/PHASE-00-SPEC.md` (new), ADR-0XXX local-OpenBao-as-default-SecretStorePort.

## Agent-handoff
```
icm store -t context-oyatie -c "M02-P00 complete: Phase 00 account-auth contracts shipped; state machine + secret persistence verified" -i critical -k "M02,P00,foundry,account-auth,phase-00,complete"
```
