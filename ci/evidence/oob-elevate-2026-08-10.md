---
doc_class: JudgmentNote
title: Elevate OOB .grok + Cargo.toml off integ/ci
status: Accepted
date: 2026-08-10
ssot_todo: unblocking-caps
---

# Envelope hygiene (#1646)

## Elevated

1. `Cargo.toml` workspace members for `ci/controller/{app,github-adapter,k8s-adapter,kernel}` → forever `integ/build` (`#planes.root_manifests`). Do **not** touch `Cargo.lock` from this tip.
2. Historical note: `.grok/**` was once proposed as process-kit forever under `roots.grok`. **Founder OVERRULE:** BAN agent-dotdirs as forever homes — process-kit forever is `ci/process-kit/**`; daemon/perimeter harness forever is `ci/facade/harness/**` (both `roots.ci` → `integ/ci`). Residual ephemeral `.grok/` (mm-runs/memory) is not forever policy.

Unblocks: hub-exclusivity/Claim for admission producer path; de-conflicts root_manifests sole-owner.

## Follow-up (post-#1644 restack)

- Process-kit + harness forever homes encoded on `integ/ci`: `ci/process-kit/**` + `ci/facade/harness/{daemon-hotset,perimeter}.v1.json` (script → `//ci/process-kit:oya-process-kit-check-daemon`). Shell under `tools/swarm/**` remains aborted (#1644).
- Re-dropped root `Cargo.toml` `ci/controller/**` members + tip `Cargo.lock` churn — forever `#planes.root_manifests` → `integ/build`.
