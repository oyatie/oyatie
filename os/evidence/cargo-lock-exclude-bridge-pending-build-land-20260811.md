---
doc_class: JudgmentNote
title: Harness exclude bridge remains until #1662 lands
status: Accepted
date: 2026-08-11
ssot_todo: cargo-lock-sole-owner
---

`integ/build` #1662@`ce9735104` absorbed harness crates + lock packages.
This tip **keeps** `os/harness/{oci-executor-oracle,attestation-relying-party}`
workspace excludes until that land reaches `origin/dev` — dropping excludes
here without the lock packages would fail freshness, and editing root
`Cargo.lock` on integ/os is banned (non-owner).

After #1662 squash-lands: drop excludes (no lock edit); expire
`integ-os-cargo-lock-bridge.yaml` on specs.
