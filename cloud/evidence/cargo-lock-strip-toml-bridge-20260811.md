---
doc_class: JudgmentNote
title: Strip root Cargo.lock from integ/cloud (#1839); Cargo.toml burn bridge
status: Accepted
date: 2026-08-11
ssot_todo: cargo-lock-sole-owner
---

Root `Cargo.lock` restored to `origin/dev` (non-owner). `Cargo.toml` still
drops `cloud/cloud-os/crates/oya-*` so burned crates do not break cargo
resolution — temporary root_manifests bridge until `integ/build` lands the
same membership + lock absorb (see
`build/evidence/cargo-lock-sole-owner-absorb-path.md`).
