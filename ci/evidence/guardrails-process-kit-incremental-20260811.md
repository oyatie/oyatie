---
doc_class: JudgmentNote
title: guardrails-live process-kit incremental
status: Accepted
date: 2026-08-11
ssot_todo: guardrails-live
---

# Shipped on integ/ci

- `.grok/process-kit` modules: `git_shim`, `toolguard`, `claim_push` (+ prior env-escape / orchestrator gate)
- `BUCK` targets: `//.grok/process-kit:oya-process-kit`, `-unittest`, `-check-daemon`
- `daemon-hotset.v1.json` script → `//.grok/process-kit:oya-process-kit-check-daemon` (corrects mistaken `.grok/swarm/*` pointer; swarm tree must not be reborn)
- **BAN** upheld: no `tools/swarm` / `.grok/swarm` shell rebirth
- **BAN** upheld: root `Cargo.lock` / workspace membership untouched (integ/build #1662); no process-kit `Cargo.toml`

# Done-when residual

- Runtime install of git-shim/toolguard into every agent lane-shell
- integ/build lock absorb so membership is sole-owner green
- Real `//...[check]` hot-set fan-out in check-daemon
