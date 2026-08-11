---
doc_class: JudgmentNote
title: Cargo.lock sole-owner absorb path (integ/build)
status: Accepted
date: 2026-08-11
ssot_todo: cargo-lock-sole-owner
---

# Forever owner

`#planes.root_manifests` → `integ/build` owns root `Cargo.toml` / `Cargo.lock`.

# First lock-owning land (this tip)

Do **not** tip-heroics lock updates on `integ/os`, `integ/cloud`, `integ/ci`, `integ/governance`, or lane PRs.

## Pending absorbs (after non-owner strips)

| Source tip | Membership delta | Lock action on integ/build | Gate |
| --- | --- | --- | --- |
| `#1839` integ/cloud | Remove `cloud/cloud-os/crates/oya-*` glob (burn) | Drop `oya-cloud-os-*-domain` packages from lock after crates gone from `dev` **or** same-wave as burn land | expire cloud toml bridge |
| `#1926` integ/os | Un-exclude `os/harness/{oci-executor-oracle,attestation-relying-party}` | Add `os-oci-executor-oracle` + `os-attestation-relying-party` via `cargo metadata` with crates present | expire `integ-os-cargo-lock-bridge` waiver |
| `#1647` integ/governance | Un-exclude `governance/check/apex-gist-integrity` | Add `check-apex-gist-integrity` | drop gov exclude bridge |
| `#1931` lane | Superseded into `#1926` harness | none (no third writer) | close/strip |

## Done-when

1. No open non-`integ/build` PR diffs root `Cargo.lock`.
2. Temporary excludes/waivers expired in the same wave as the build lock land.
3. Freshness green on `integ/build` after lock absorb.
