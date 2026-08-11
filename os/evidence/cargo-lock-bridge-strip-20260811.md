---
doc_class: JudgmentNote
title: Strip root Cargo.lock from integ/os (#1926)
status: Accepted
date: 2026-08-11
ssot_todo: cargo-lock-sole-owner
---

Root `Cargo.lock` restored to `origin/dev` (non-owner). New harness crates
`os/harness/{oci-executor-oracle,attestation-relying-party}` excluded from the
workspace until `integ/build` absorbs membership + lock.

Specs waiver `integ-os-cargo-lock-bridge.yaml` remains the expire record —
expires on integ/build first lock-owning land (see
`build/evidence/cargo-lock-sole-owner-absorb-path.md`).
