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
2. `.grok/**` swarm kit adds/edits → forever `integ/grok` (envelope `roots.grok`); transitional tools vacate separate.

Unblocks: hub-exclusivity/Claim for admission producer path; de-conflicts root_manifests sole-owner.
