---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P01-IP-003
title: Cloud Network VPC + LB + DNS + CDN + interconnect API
status: vpc-dns-second-provider-selfhosted-lb-interconnect-oci-request-contract-green; cdn-remaining-second-provider-live-smoke pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Bring cloud.network.* surfaces to stable; ≥2 provider adapters per surface, with self-hosted/on-prem/colo treated as a first-class cloud target.
---

# M03-P01-IP-003 — Cloud Network VPC + LB + DNS + CDN + interconnect API

## Purpose
Bring cloud.network.* surfaces to stable; ≥2 provider adapters per surface, with self-hosted/on-prem/colo treated as a first-class cloud target.

## Symbols-to-grit-claim
```
crates/oya-cloud-network-vpc-api/src/lib.rs::create
crates/oya-cloud-network-lb-api/src/lib.rs::create
crates/oya-cloud-network-dns-api/src/lib.rs::create_zone
crates/oya-cloud-network-cdn-api/src/lib.rs::create_distribution
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-governance-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- OCI VCN/VPC request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- OCI Load Balancer request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- OCI FastConnect direct interconnect request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- Self-hosted/colo VPC second-provider request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-21).
- Self-hosted/colo DNS second-provider request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-21).
- CDN, remaining second-provider adapters, and credentialed live provider smoke remain required before marking this whole IP complete.
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M03-P01-IP-003 OCI VCN/VPC, Load Balancer, DNS, FastConnect, and self-hosted/colo VPC+DNS request contracts green; CDN, remaining second provider, and live smoke pending' -i high -k 'M03-P01-IP-003,partial,selfhosted-vpc-dns,live-smoke-pending'
```

## Progress ledger

- 2026-05-20 — `cs-m03-p01-network-vpc-oci-adapter-port-2026-05-20` added the provider-neutral VPC create provider port in `oya-cloud-network-domain` and the OCI VCN request-contract adapter crate `oya-cloud-network-adapter-oci`. This proves deterministic OCI VCN create command/receipt shape only; LB, DNS, CDN, interconnect, second-provider adapters, and credentialed live smoke remain pending.
- 2026-05-20 — `cs-m03-p01-network-lb-oci-adapter-port-2026-05-20` extended the provider-neutral Cloud Network port surface to load balancer create and extended `oya-cloud-network-adapter-oci` with deterministic OCI Load Balancer create command/receipt shape. This proves OCI LB request contract only; DNS, CDN, interconnect, second-provider adapters, and credentialed live smoke remain pending.

- 2026-05-20 — `cs-m03-p01-network-dns-oci-adapter-port-2026-05-20` extended the provider-neutral Cloud Network port surface to DNS zone create and extended `oya-cloud-network-adapter-oci` with deterministic OCI DNS CreateZone command/receipt shape. This proves OCI DNS request contract only; CDN, interconnect, second-provider adapters, and credentialed live smoke remain pending.

- 2026-05-20 — `cs-m03-p01-network-interconnect-oci-adapter-port-2026-05-20` extended the provider-neutral Cloud Network port surface to direct interconnect create and extended `oya-cloud-network-adapter-oci` with deterministic OCI FastConnect CreateVirtualCircuit command/receipt shape. This proves OCI FastConnect request contract only; CDN, second-provider adapters, and credentialed live smoke remain pending.

- 2026-05-21 — `cs-m03-p01-network-selfhosted-vpc-adapter-port-2026-05-20` added `oya-cloud-network-adapter-selfhosted` for deterministic self-hosted/colo VPC network-segment command/receipt shape behind the existing provider-neutral VPC port. This proves the VPC second-provider/on-prem target only; CDN, remaining second-provider adapters, and credentialed live smoke remain pending.
- 2026-05-21 — `cs-m03-p01-network-selfhosted-dns-adapter-port-2026-05-21` extended `oya-cloud-network-adapter-selfhosted` with deterministic self-hosted/colo authoritative DNS zone command/receipt shape behind the existing provider-neutral DNS zone port. This proves the DNS second-provider/on-prem target only; CDN, remaining second-provider LB/interconnect adapters, and credentialed live smoke remain pending.

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: OCI VCN, Load Balancer, DNS, and FastConnect compartment/region/path/reference handling stays inside the OCI adapter crate, while self-hosted/colo site/cell/fabric handling for VPC and DNS stays inside the self-hosted adapter crate instead of branching through domain/API crates; no provider SDK or live network call was added to these request-contract slices.


## ChangeSet evidence — cs-m03-p01-network-vpc-oci-adapter-port-2026-05-20
- Added provider-neutral `NetworkProviderVpcPort` plus validated VPC create request and receipt types in `oya-cloud-network-domain`.
- Added `oya-cloud-network-adapter-oci` with deterministic OCI VCN create command shape and provider-VCN drift/config tests.
- Verification: `cargo test -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`; `cargo clippy -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`.
- Status boundary: OCI VCN/VPC request contract is green; LB, DNS, CDN, interconnect, second-provider adapters, and credentialed live provider smoke remain pending.


## ChangeSet evidence — cs-m03-p01-network-lb-oci-adapter-port-2026-05-20
- Added provider-neutral `NetworkProviderLoadBalancerPort` plus validated load balancer create request and receipt types in `oya-cloud-network-domain`.
- Extended `oya-cloud-network-adapter-oci` with deterministic OCI Load Balancer create command shape and provider-load-balancer drift/config tests.
- Verification: `cargo test -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`; `cargo clippy -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`.
- Status boundary: OCI VCN/VPC and Load Balancer request contracts are green; DNS, CDN, interconnect, second-provider adapters, and credentialed live provider smoke remain pending.


## ChangeSet evidence — cs-m03-p01-network-dns-oci-adapter-port-2026-05-20
- Added provider-neutral `NetworkProviderDnsZonePort` plus validated DNS zone create request and receipt types in `oya-cloud-network-domain`.
- Extended `oya-cloud-network-adapter-oci` with deterministic OCI DNS CreateZone command shape and provider-DNS-zone drift/config tests.
- Verification: `cargo test -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`; `cargo clippy -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`.
- Status boundary: OCI VCN/VPC, Load Balancer, and DNS request contracts are green; CDN, interconnect, second-provider adapters, and credentialed live provider smoke remain pending.


## ChangeSet evidence — cs-m03-p01-network-interconnect-oci-adapter-port-2026-05-20
- Added provider-neutral `NetworkProviderDirectInterconnectPort` plus validated direct interconnect create request and receipt types in `oya-cloud-network-domain`.
- Extended `oya-cloud-network-adapter-oci` with deterministic OCI FastConnect CreateVirtualCircuit command shape and provider-virtual-circuit drift/config tests.
- Verification: `cargo test -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`; `cargo clippy -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci`.
- Status boundary: OCI VCN/VPC, Load Balancer, DNS, and FastConnect request contracts are green; CDN, second-provider adapters, and credentialed live provider smoke remain pending.

## ChangeSet evidence — cs-m03-p01-network-selfhosted-vpc-adapter-port-2026-05-20
- Added `NetworkProviderKind::SelfHostedColoVpc` as an additive VPC provider kind without changing the provider-neutral VPC port request/receipt shape.
- Added `oya-cloud-network-adapter-selfhosted` with deterministic self-hosted/colo tenant network segment command shape and provider-VPC drift/config tests.
- Verification: `cargo test -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted`; `cargo clippy -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted`.
- Status boundary: OCI VCN/VPC, Load Balancer, DNS, FastConnect, and self-hosted/colo VPC request contracts are green; CDN, remaining second-provider adapters, and credentialed live provider smoke remain pending.

## ChangeSet evidence — cs-m03-p01-network-selfhosted-dns-adapter-port-2026-05-21
- Added `NetworkProviderKind::SelfHostedColoDnsZone` as an additive DNS provider kind without changing the provider-neutral DNS zone port request/receipt shape.
- Extended `oya-cloud-network-adapter-selfhosted` with deterministic self-hosted/colo authoritative DNS zone command shape and provider-DNS-zone drift/config tests.
- Verification: `cargo test -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted`; `cargo clippy -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-network-domain -p oya-cloud-network-adapter-oci -p oya-cloud-network-adapter-selfhosted`.
- Status boundary: OCI VCN/VPC, Load Balancer, DNS, FastConnect, self-hosted/colo VPC, and self-hosted/colo DNS request contracts are green; CDN, remaining second-provider LB/interconnect adapters, and credentialed live provider smoke remain pending.
