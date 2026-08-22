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

Dual-homes path trees **only** so freshness/`cargo metadata` can resolve members; forever content owners stay:

| Path | Forever content tip |
| --- | --- |
| `ci/controller/**` | `integ/ci` (#1646) |
| `os/harness/{oci-executor-oracle,attestation-relying-party}/**` | `integ/os` (#1926) |

## Absorbed this land

| Source | Membership / lock action |
| --- | --- |
| `#1646` | Add `ci/controller/{app,github-adapter,k8s-adapter,kernel}` members + lock packages `ci-controller-*` |
| `#1926` | Un-exclude ready: add harness crates + lock packages `os-oci-executor-oracle`, `os-attestation-relying-party` |
| `#1839` | Drop `cloud/cloud-os/crates/oya-*` glob + prune `cloud-os-*-domain` lock packages |
| build tip | Refresh missing `port-engine-*` lock packages for existing `build/port-engine/*/*` members |

## Post-land follow-ups

1. `#1646` strips root `Cargo.lock` + controller **members** (sources stay).
2. `#1839` strips `Cargo.toml` cloud-os glob bridge (build owns it).
3. `#1926` drops harness exclude bridge.
4. Expire `integ-os-cargo-lock-bridge.yaml` on specs tip-free after this lands.
5. `#1647` apex-gist exclude → later build absorb (not this wave).

## Done-when

1. No open non-`integ/build` PR diffs root `Cargo.lock`.
2. Temporary excludes/waivers expired same-wave or immediately after land.
3. Freshness green on `integ/build` after lock absorb.
