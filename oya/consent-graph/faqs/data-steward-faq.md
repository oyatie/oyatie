---
doc_class: FAQ
microservice: consent-graph
persona: data-steward + partnership-manager
date: 2026-05-20
doc_status: published
---

# Data Steward FAQ — consent-graph

## Why bilateral Merkle chain and not blockchain?

Per ADR-0214 § 3.5. Blockchain (Bitcoin / Ethereum) requires consensus + mining + the operational + reputational baggage. We need:

- Tamper-evidence (hash chaining).
- Bilateral attribution (both grantor + grantee sign each event).
- Audit-defensibility (any third-party can verify the chain).

A bilateral Merkle chain with Ed25519 signatures gives us all three at orders-of-magnitude less complexity than a public blockchain. No mining, no consensus, no token economics. The trade-off: we're not "trustless" — both parties must trust each other's signatures + the central authority that publishes the chain root. For B2B-partner data sharing, that's the right trust model.

## When is Projection-mode the right sharing mode vs Aggregate?

- Projection: real-time operational use; the grantee needs the row-level data to make per-entity decisions (e.g., a logistics partner needs the actual shipment record).
- Aggregate: analytical use; the grantee needs aggregate statistics, never per-row data (e.g., a market-research partner needs "average order value per region", not per-order rows).
- AttestedQuery: ad-hoc but auditable; the grantee submits a one-off query (e.g., a compliance officer needs "how many cross-border transactions in 2026 above $10k threshold").

Choose Aggregate by default for partners who don't need row-level data. The k-anonymity threshold protects against re-identification at small group sizes.

## A grantor revoked an agreement; how long until the grantee can no longer access data?

Per PRD goal §4. Target: p99 ≤ 1 s, p100 ≤ 3 s.

Mechanism (per ADR-SVC-CG-002):

1. Revocation event emitted to Pulsar.
2. Pulsar fans out to all Cedar evaluator workers (typical ≤ 200 ms).
3. Each worker invalidates the local policy cache (≤ 100 ms).
4. Next projection-read by the grantee → Cedar denies.

The boundary: if a grantee's app has buffered rows in its OWN local cache, revocation does not reach back to that buffer. The agreement terms must specify the grantee's local-cache TTL (typically ≤ 30 min).

## A grantee tries to use data after revocation — is this a contract breach?

Generally yes. The DataSharingAgreement specifies acceptable use; using data after revocation typically violates the agreement.

What we GUARANTEE:

- The grantee CANNOT pull new data from the grantor after revocation (Cedar denies).
- The grantee CANNOT see new updates in the projection.

What we DO NOT GUARANTEE:

- We don't enforce in-grantee-database deletion of historically-projected data unless the agreement specifies + the grantee implements local-erasure-on-revocation.
- A GDPR right-to-erasure request to the grantee is the legal remedy if the grantor learns of in-grantee retention.

## What's the difference between scope narrowing and predicate narrowing?

- **Scope narrowing** = which Ontology entity-types + which fields can the agreement permit.
- **Predicate narrowing** = which rows of the entity-type satisfy a predicate.

Example: an `Order` agreement might be scope-narrowed to `{id, total_amount, created_at}` (NOT `customer_pii`) AND predicate-narrowed to `status = 'shipped'`. Both layers apply per row.

## We have a B2C use case where a consumer can self-revoke. How does that work?

Per ADR-SVC-CG-005. The B2C variant of the agreement is between the consumer (the data subject) + the platform's tenant (the grantor) — with the platform-tenant agreeing to share consumer data with a specified grantee.

The consumer has the right to self-revoke via:

```
oya consent-graph self-revocation invoke \
    --agreement-id ag-active-2026 \
    --consumer drill-consumer-x \
    --reason consumer-self-revoke \
    --revocation-evidence ./self-attest.json
```

This emits `consumer_self_revoked`; both grantor + grantee receive notification + propagation per the standard p99 ≤ 1 s.

## Cross-border transfer is explicit per agreement; what packs forbid it?

Per agreement-level `geographic-constraint`. Defaults:

- KR-PIPA: cross-border transfer requires explicit consent + additional safeguards.
- EU-GDPR: cross-border transfer outside EU/EEA requires Art. 46 safeguards (SCCs, BCRs, adequacy decision, etc.).
- CN-PIPL: cross-border transfer requires CAC security assessment.
- KSA-PDPL: cross-border transfer requires KSA NDMO approval.

The agreement's `geographic-constraint` can be: `same-region` (no cross-border), `eu-eea-only`, `pack-specific-approved-list`, `none` (no constraint).

The projection-gateway enforces at READ-time, not just at agreement-creation; if a grantee's request comes from a region not approved, the read is denied.

## Why is the projection gateway a separate service from the agreement state machine?

Per ADR-SVC-CG-001 + ADR-SVC-CG-003. The agreement state machine handles slow operations (state transitions; ~ 100/sec sustained); the projection gateway handles fast operations (per-row Cedar evaluation; ~ 50 k/sec sustained at tenant_class paid tier).

Separating gives us:

- Independent capacity scaling (the gateway scales with projection volume; the state machine scales with agreement count).
- Failure isolation (a slow agreement state-machine doesn't starve projections).
- Different SLO posture (state-machine p99 ≤ 1 s; gateway p99 ≤ 50 ms).

## A bilateral chain integrity failure was alerted. What do I do?

Per `runbooks/bilateral-chain-integrity-failure.md`. The most likely cause is a chain-link signature corruption (someone replayed an event or one side's Ed25519 key was rotated improperly).

The runbook walks:

1. Identify the offending chain link (the one whose signature does not verify against the prior root + the prior signer's key).
2. Cross-reference both sides' chains. If only one side has the corruption, the corruption is local to that side.
3. Suspend new state transitions for the agreement until cleared.
4. Notify the joint DPO meeting; escalate per the master agreement.
5. Once root cause identified, re-establish the chain from the last verified root + add a `chain_repair` audit event noting the repair (the chain is never "rewritten"; only extended with a chain_repair marker).

## Why not give grantees raw cross-tenant access tokens?

Per PRD goal #1 + #2 + #4. Token-based access:

- Has no scope narrowing beyond the token's coarse permissions.
- Has no real-time revocation (tokens are valid until expiry).
- Has no per-read Cedar evaluation (the token grants blanket access).
- Has no audit-chain-bound attribution per read.

The agreement-based model gives us all four. The grantee uses an mTLS-bound agreement-id-scoped JWT; the JWT does NOT itself grant access — the projection-gateway evaluates Cedar per read against the agreement state at read-time.

## How does partner-directory work?

Per IP-014. Each tenant publishes a partner-directory entry: their tenant_id, trust-anchor public key, pack-id, supported sharing-modes, contact-data-steward.

When a tenant wants to invite a partner:

1. Both tenants exchange directory entries (out-of-band or via a discovery service).
2. The grantor offers an agreement to the grantee's directory entry.
3. The grantee's data-steward reviews + accepts.
4. The trust-anchor public keys are used to verify per-event signatures from that point.

The trust-anchor is rotated periodically (per the master-agreement; typically annual). Rotation requires both parties to re-sync directory entries.

## What's the difference between this and Snowflake Secure Data Share?

Snowflake passes guarantees 1 (audit defensibility) + 3 (sovereignty) of the PRD §1. It fails:

- (2) Revocability: revoking a Snowflake Secure Data Share is a Snowflake-internal operation; the grantee may have already pulled data into their Snowflake-private space; revocation doesn't reach back.
- (4) Scope narrowness: Snowflake shares an entire view; row/field-level narrowing isn't enforced cryptographically.
- (5) Real-time: Snowflake's data-share refresh is ETL-tier; not sub-second.

oyatie consent-graph passes all 5. The trade-off: Snowflake has the global warehouse ecosystem; we're agreement-based + per-tenant. Use Snowflake for ad-hoc analytics across snowflake-already-tenants; use consent-graph for B2B-partner operational + governance use.
