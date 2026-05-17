---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M06-P01-IP-001
title: Auction kernel + bidding engine
status: stub
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ads auction kernel + bidding engine internal-tenant only.
execution_variant: merge-into-existing-crates
decided_at: 2026-05-17
decided_by: user-directive-option-2
execution_variant_note: "Delta-1 backports Auction + Bid + AuctionState + AuctionError + AuctionId + BidId into existing crates/oya-saas-plugin-marketplace-kernel/src/auction.rs instead of scaffolding a new oya-ads-auction-kernel. Honors no-over-scaffolding rule; tenant-isolation enforced at construct/submit time. Subsequent deltas track second-price settlement + advertiser console + ML isolation under same FixupTask F-M02B-PLAN-LIVE-CRATE-RECONCILIATION (extended scope to M06 phases)."
---

# M06-P01-IP-001 — Auction kernel + bidding engine

## Purpose
Ads auction kernel + bidding engine internal-tenant only.

## Symbols-to-grit-claim
```
crates/oya-saas-plugin-marketplace-kernel/src/auction.rs::Auction
crates/oya-saas-plugin-marketplace-kernel/src/auction.rs::Bid
```
(execution_variant: merge-into-existing-crates; no scaffold needed — symbols live in the existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
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
icm store -t context-oyatie -c 'M06-P01-IP-001 Auction kernel + bidding engine shipped; acceptance commands green' -i high -k 'M06-P01-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: (to be filled at PR time; empty section = fail).
