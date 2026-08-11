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

## Follow-up (post-#1644 restack)

- Restored `.grok/harness/daemon-hotset.v1.json` + `perimeter.v1.json` on `integ/ci` (envelope `roots.grok` → `integ/ci`; binding mirrors). Shell process-kit remains deferred Rust-first (automation-language; #1644 aborted `tools/swarm/**`).
- Re-dropped root `Cargo.toml` `ci/controller/**` members + tip `Cargo.lock` churn — forever `#planes.root_manifests` → `integ/build`.
