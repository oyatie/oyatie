---
purpose: Auto-backfilled purpose for README.md
---

# M01-foundation acceptance evidence

Per ADR-0063 §3 every milestone has an acceptance-evidence directory.

## Current acceptance record

- Evidence JSON: `/evidence/foundation/m01-foundation-acceptance-audit-2026-05-14.json`
- Status: M01 Foundation acceptance complete for G1/G2 contracts.
- G1: P01 data-use-boundary-tenancy, P02 identity-cedar, P03 audit-chain-evidence.
- G2: P04 eventing+ontology, P05 cell+plane, P06 regional-pack+flattening.
- Sequencing: G2 acceptance is valid because G1 contracts were already complete and usable.
- M-CC-P01: foundation-cleared/P5+; banned-primitives, archive-orphan, and authoritative-tracked lanes pass.
- M-CC-P00: accepted-for-masterplan-P00 but not ready. Waiver scope is acceptance reconciliation only; broad fanout, new foundation scaffolds, and M02/M03 implementation remain blocked until P00 readiness or an explicit waiver.
- Validation: focused G1/G2 package tests pass 65/65; full `./scripts/check.sh` passes end-to-end after the Rust 1.95.0 / edition 2024 / rustfmt 2024 closeout.
