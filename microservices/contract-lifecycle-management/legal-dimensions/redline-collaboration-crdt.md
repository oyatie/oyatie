---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: Migrated-from-tier-matrix (retired-advanced)
related_packs: [sox-404]
date: 2026-05-21
---

# Redline Collaboration — CRDT Model

CLM supports real-time collaborative editing of contract drafts (pre-signature) using a CRDT (Conflict-free Replicated Data Type) approach equivalent to the `docs` µservice. The CRDT is Loro (https://loro.dev) — Rust-native, JSON-CRDT, with rich-text support.

## Constraints

- Pre-signature only. Once a contract enters OutForSignature or beyond, edits are blocked.
- All collaborators must be tenant-scoped (no cross-tenant collaboration).
- Counterparty redlines arrive via the counterparty-redline-provenance pipeline (IP-029), NOT through the CRDT.
- All CRDT operations seal to the audit chain.

## Loro CRDT properties

- Operation-based CRDT (OB-CRDT).
- JSON-CRDT for arbitrary document structure.
- Built-in rich-text type for text editing.
- Compact encoding (binary diff between versions).
- Time-traveling history (peer can rewind).

## Per-tenant deployment

The CRDT runs in the µservice's `crdt-coordinator` worker (per ADR-0105 worker layer). Each contract draft has a per-tenant CRDT room. Peers connect via WebSocket + HTTP/3 (per ADR-0253).

## Audit-chain integration

Every CRDT operation produces an audit-chain event:

- `crdt_operation_applied { contract_id, peer_id, operation_hash, lamport_timestamp }`.

The operation hash is BLAKE3 of the operation payload; replay deterministic from the operation log.

## Concurrency model

Loro provides eventual consistency. The µservice does NOT impose any locking; all peers may edit simultaneously. Conflicts are resolved by Loro's CRDT semantics (preserve all intentions where possible; last-writer-wins on the rare ambiguous merge).

## Snapshot model

The µservice snapshots the CRDT state at:

- Every 100 operations.
- Every 5 minutes (whichever first).
- At every state-machine transition (Review → Approved, etc.).

Snapshots are content-addressed (BLAKE3) and stored alongside the operation log.

## Export

Snapshot export to immutable contract version (for OutForSignature transition):

1. Materialize the current CRDT state into a static JSON+text representation.
2. Hash with BLAKE3.
3. Seal into the signature envelope's `contract_version_hash`.

## Cedar gate

```cedar
forbid (
  principal,
  action == Action::"CrdtEdit",
  resource is Contract
) when {
  resource.state !in ["Draft", "Review", "InternalReview", "CounterRedline"]
};
```

## Audit events

- `oya.contract.lifecycle.management.crdt.peer_joined`
- `oya.contract.lifecycle.management.crdt.operation_applied`
- `oya.contract.lifecycle.management.crdt.snapshot_taken`
- `oya.contract.lifecycle.management.crdt.state_materialized_for_signature`

## Standards references

- Loro CRDT specification (loro.dev).
- Shapiro et al., "Conflict-Free Replicated Data Types" (INRIA, 2011).
- Kleppmann, "Designing Data-Intensive Applications", Ch. 5.
