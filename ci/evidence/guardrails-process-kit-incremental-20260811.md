---
doc_class: JudgmentNote
title: guardrails-live process-kit incremental
status: Accepted
date: 2026-08-11
ssot_todo: guardrails-live
---

# Shipped on integ/ci

- Forever home (founder OVERRULE): `ci/process-kit/**` (Buck `//ci/process-kit:…`) — **BAN** agent-dotdirs as forever homes
- Modules: `git_shim`, `toolguard`, `claim_push` (+ prior env-escape / orchestrator gate)
- `BUCK` targets: `//ci/process-kit:oya-process-kit`, `-unittest`, `-check-daemon`
- Policy mirrors: `ci/facade/harness/daemon-hotset.v1.json` + `perimeter.v1.json` (script → `//ci/process-kit:oya-process-kit-check-daemon`)
- **BAN** upheld: no `tools/swarm` / `.grok/swarm` shell rebirth
- **BAN** upheld: root `Cargo.lock` / workspace membership untouched (integ/build #1662); no process-kit `Cargo.toml`
- Ephemeral `.grok/mm-runs` + `.grok/memory` left alone (not forever policy)

# Done-when residual

- Runtime install of git-shim/toolguard into every agent lane-shell
- integ/build lock absorb so membership is sole-owner green
- Real `//...[check]` hot-set fan-out in check-daemon
- Profile A tip-entitlement skip (`tip_class=idle` OR missing preflight receipt → skip expensive jobs, not CODE fail) — see `ci/REORG-DRAIN.md`
