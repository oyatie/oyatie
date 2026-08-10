---
doc_class: JudgmentNote
title: integ/build root_manifests hygiene + registry elevate
status: Accepted
date: 2026-08-10
ssot_todo: unblocking-caps
---

# Envelope hygiene (#1662)

1. Stripped `registry/catalog/port-engine-*.yaml` OOB → forever `integ/registry` (#1707 / tip-free).
2. Absorbed `ci/controller/**` workspace membership into root `Cargo.toml` (forever `#planes.root_manifests` owner). **No `Cargo.lock` edit** this commit — lock refresh is a separate sole-owner step if cargo membership requires it.

Unblocks: every membership tip blocked on root_manifests sole-owner; ci tip no longer needs Cargo.toml.
