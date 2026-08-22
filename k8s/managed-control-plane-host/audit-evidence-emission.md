# Audit / evidence emission — `managed-k8s-control-plane-host`

**Authority:** ADR-0376 (audit_chain seal_events for provision/teardown/
datastore-bound/tier-selected), ADR-0263 observability emission contract,
ADR-0154 event-schema versioning. non_claim: the emitter WIRING lands with
`kamaji-provider-live-integration`; this lane contracts the seal-event schema +
the manifest declaration so the audit-chain + sla-observability consumers can
design against it.

## Seal events (manifest.json#audit_chain.seal_events)

Every state-changing control-plane decision emits ONE immutable, hash-chained
record into `evidence/audit-chain.jsonl`:

| Event | Emitted when |
|-------|--------------|
| `oya.managed-k8s.control-plane.tier-selected` | a provision request resolves a tier (hosted vs dedicated) |
| `oya.managed-k8s.control-plane.datastore-bound` | a hosted-tier control plane binds its datastore (etcd-per-tenant or pooled-relational) |
| `oya.managed-k8s.control-plane.provisioned` | a control plane reaches `active` |
| `oya.managed-k8s.control-plane.torn-down` | a control plane reaches `deleted` |

Schema: `contracts/asyncapi.yaml#ControlPlaneSealEvent` (carries the ADR-0154
`version: "1"` header, `tenant_id`, `cluster_name`, `tier`, `datastore_class`
where relevant, `status`, `occurred_at`).

## Cedar enforcement

A provision/teardown is DENIED unless `context.audit_chain_emit == true`
(`cedar/policies.cedar`). This makes the audit emission a precondition of the
mutation, not a best-effort afterthought.

## Evidence chain

The records are hash-chained (Merkle seal per the audit-chain microservice's
Ed25519 surface) so a regulator/operator can prove the complete provision/teardown
history of any tenant control plane. The status state machine in the kernel makes
the event ordering well-defined (one seal per legal transition into a
state-changing status).

## What is NOT emitted in this lane

No live audit records are written yet — the CAPI adapter performs no reconcile.
The in-memory adapter exercises the lifecycle deterministically in tests but does
not write to the global `evidence/audit-chain.jsonl` (it is a test/bring-up
reference). The honest posture: the contract is fixed; the emitter activates with
the live integration without a caller change.
