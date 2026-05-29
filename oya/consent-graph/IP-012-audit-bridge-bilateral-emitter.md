# IP-012: audit-bridge — bilateral audit-chain emitter (grantor + grantee chains)

- Bounded context: audit-bridge
- Layers: kernel, domain, usecase, api, adapter, app, worker
- Crates:
  - `oya-consent-graph-audit-bridge-kernel`
  - `oya-consent-graph-audit-bridge-domain`
  - `oya-consent-graph-audit-bridge-usecase`
  - `oya-consent-graph-audit-bridge-api`
  - `oya-consent-graph-audit-bridge-adapter`
  - `oya-consent-graph-audit-bridge-app`
  - `oya-consent-graph-audit-bridge-worker`
- Acceptance status: ga
- Authority: ADR-0214 §2.6 (bilateral chain entries), ADR-0003 (audit-chain emission), ADR-SVC-CG-001
  (bilateral chain link schema).
- Depends on: `oya-consent-graph-agreement-kernel`, `oya-audit-chain-emission-sdk` (cross-µservice).

## 1. Goal

For every consent-graph event (grant, accept, amend, revoke, projection-mint, projection-emit,
projection-read, enforcement-permit, enforcement-deny, etc.), emit **two** audit-chain entries —
one on the grantor's chain and one on the grantee's chain — with a cross-pointer linking them. Both
must seal within the audit-chain seal budget (≤500ms per ADR-0003).

This is the substrate that makes Open-Banking-grade + HIE-grade audit defensibility achievable.

## 2. Why bilateral

Either party in a cross-tenant exchange must be able to *independently* prove the other party's
actions, without trusting the other party's chain. The grantor proves "I granted X access to Y at
time T"; the grantee proves "I read X at time T1 under grant Y." A reconciler walks both chains and
verifies every pairing.

Failure modes that bilateral chains catch:
- Grantor erases their copy: grantee's chain stands.
- Grantee erases their copy: grantor's chain stands.
- Forged grant attempt: cannot fabricate paired entry on grantor side without grantor's signing key.

## 3. Cross-pointer schema (per ADR-SVC-CG-001)

```rust
pub struct BilateralAuditEvent {
    pub event_id: Ulid,
    pub agreement_id: AgreementId,
    pub event_type: ConsentGraphEventType,    // 17 types per manifest seal_events
    pub timestamp: Timestamp,
    pub grantor_payload: GrantorChainPayload, // canonical view from grantor perspective
    pub grantee_payload: GranteeChainPayload, // canonical view from grantee perspective
    pub cross_pointer: CrossPointer,          // populated post-seal
}

pub struct CrossPointer {
    pub grantor: ChainLink,                   // {chain_id, seq, sealed_at, merkle_root}
    pub grantee: ChainLink,
    pub paired_hmac: Hmac256Bytes,            // HMAC(grantor||grantee, key=per-pair-pair-secret)
}
```

The `paired_hmac` is computed by the audit-bridge worker once *both* seals arrive; it's signed with
a per-(grantor, grantee) HMAC key stored in OpenBao. This prevents tampering: even if both chains
are compromised, the HMAC verifies pairing integrity.

## 4. Emission flow

```rust
async fn emit_bilateral(&self, event: BilateralAuditEvent) -> Result<EmissionReceipt, EmitError> {
    // 4.1: build per-side payloads
    let grantor_seal_request = AuditChainSealRequest {
        chain_id: ChainId::for_tenant(event.grantor_payload.grantor_tenant),
        payload: event.grantor_payload.canonicalize()?,
        event_class: event.event_type.to_audit_class(),
    };
    let grantee_seal_request = AuditChainSealRequest {
        chain_id: ChainId::for_tenant(event.grantee_payload.grantee_tenant),
        payload: event.grantee_payload.canonicalize()?,
        event_class: event.event_type.to_audit_class(),
    };

    // 4.2: parallel seal (both chains seal independently)
    let (grantor_link, grantee_link) = tokio::try_join!(
        self.audit_chain_sdk.seal(grantor_seal_request),
        self.audit_chain_sdk.seal(grantee_seal_request),
    )?;

    // 4.3: compute paired HMAC
    let pair_key = self.openbao.read_pair_hmac_key(&event.agreement_id).await?;
    let paired_hmac = hmac_sha256(&pair_key, &concat(&grantor_link, &grantee_link));

    // 4.4: persist cross-pointer
    let cross_pointer = CrossPointer { grantor: grantor_link, grantee: grantee_link, paired_hmac };
    self.repo.persist_cross_pointer(event.event_id, &cross_pointer).await?;

    // 4.5: emit confirmation events on both chains (small "I-was-paired" entry)
    // (this is a second, smaller pair of audit entries that proves the cross-pointer existed at this time)
    let confirm_grantor = self.audit_chain_sdk.seal(AuditChainSealRequest::pair_confirmation(...)).await?;
    let confirm_grantee = self.audit_chain_sdk.seal(AuditChainSealRequest::pair_confirmation(...)).await?;

    Ok(EmissionReceipt { event_id, cross_pointer, confirm_grantor, confirm_grantee })
}
```

The double-emission (4.2 primary, 4.5 pair-confirmation) is deliberate: the primary entry records
the consent-graph event; the confirmation entry records the cross-pointer itself. Without 4.5, an
attacker tampering with the cross-pointer storage could go undetected; with 4.5, the pairing fact
itself is sealed.

## 5. Canonicalization (`canonicalize`)

Both grantor and grantee payloads must produce byte-identical canonical JSON for any equivalent
event. This is required so the HMAC and Merkle hashes are reproducible during verification.

Canonicalization rules (per ADR-0003 chain-of-custody):
- Field order: alphabetical.
- Timestamps: RFC3339 with `Z` suffix, ms precision.
- Numbers: integer where representable, else `<value>` with no trailing zeros.
- Booleans: lowercase.
- Null: omit field.
- Strings: NFC normalized.

Implemented via `oya-shared-canonical-json` crate (already shipped).

## 6. Schema versioning

Bilateral chain entries are versioned; `schema_version` field on the event allows downstream
verifiers to know which canonicalization rules + which payload shape to expect.

Adding a new event type:
1. Increment `schema_version`.
2. Add to `ConsentGraphEventType` enum.
3. Add to `manifest.json` seal_events list.
4. Update audit-chain's `EventClass` enum (cross-µservice coordination via `axis-audit-chain`).
5. ADR-SVC-CG-* if the new type alters cross-pointer semantics.

## 7. Tests

| Test | Assertion |
|------|-----------|
| `emit_bilateral_both_chains_seal` | grantor + grantee both receive one entry each |
| `paired_hmac_verifies_with_pair_key` | independent verifier reproduces HMAC from pair_key + links |
| `paired_hmac_fails_with_wrong_key` | wrong pair_key → HMAC mismatch |
| `canonicalization_identical_across_sides` | same event from grantor vs grantee perspective canonicalizes to same bytes |
| `parallel_seal_partial_failure` | grantor seal fails, grantee succeeds → entire emission rolls back |
| `seal_latency_p99_under_500ms` | 1K bilateral emits, p99 ≤500ms |
| `pair_confirmation_emitted_after_primary` | both chains receive 2 entries (primary + confirmation) |

## 8. Adapter (`audit-bridge-adapter`)

Postgres tables:
```sql
CREATE TABLE consent_graph_cross_pointers (
    event_id ulid PRIMARY KEY,
    agreement_id ulid NOT NULL,
    event_type text NOT NULL,
    grantor_chain_id text NOT NULL,
    grantor_seq bigint NOT NULL,
    grantee_chain_id text NOT NULL,
    grantee_seq bigint NOT NULL,
    paired_hmac bytea NOT NULL,
    created_at timestamptz NOT NULL,
    UNIQUE (grantor_chain_id, grantor_seq),
    UNIQUE (grantee_chain_id, grantee_seq)
);
SELECT create_distributed_table('consent_graph_cross_pointers', 'agreement_id');
```

The double-unique-constraint ensures one cross-pointer per chain entry on either side.

## 9. Worker

`audit-bridge-worker`:
- Drains a Pulsar `oya.consent-graph.audit-bridge.v1` topic (events queued by usecases that don't
  inline-await audit emission for latency reasons — e.g., enforcement hot path).
- Performs the bilateral emit flow.
- Retries up to 5x with exp-backoff on transient audit-chain SDK errors.
- Sends to dead-letter table on permanent failure → runbook `audit-chain-divergence-recovery.md`.

## 10. App composition

`audit-bridge-app` wires:
- gRPC server (port 9446) for direct bilateral emit calls (used by usecases that inline-await).
- Pulsar subscriber for async path.
- Audit-chain SDK.
- OpenBao client for pair-HMAC keys.
- Health probes.

## 11. SLO wiring

This IP feeds `audit-chain-coverage-completeness` SLO:
- Numerator: `oya_consent_graph_audit_emit_total{outcome="sealed"}` (both sides sealed + paired).
- Denominator: `oya_consent_graph_audit_emit_total` (all attempts).
- Target: 1.0 (no event un-sealed).

Page on 99.9% (10 PPM drop) — coverage drift is a P1.

## 12. Verification

- `cargo build` + `cargo test` clean.
- E2E: emit 1K bilateral events; query both chains; verify pairing via HMAC + cross-pointer table.
- Chaos: kill grantor chain mid-emit → rollback fires; no orphan grantee entry.
- Performance: 10K bilateral emit/s sustained; p99 ≤500ms.

## 13. Risk

- **R**: Audit-chain µservice outage halts cross-tenant operations.
  **M**: Audit-bridge worker queues to Pulsar; usecases that inline-await (e.g., grant/revoke) fall
  back to async + return success with `audit_pending=true`; reconciliation completes within minutes.
  Enforcement hot-path *never* inline-awaits audit emission — uses outbox.
- **R**: Pair-HMAC key compromise.
  **M**: Per-pair key (not global); compromise of one pair doesn't affect others. Key rotation via
  OpenBao key-version; rotation requires re-pairing nightly reconciliation.
- **R**: Cross-pointer table sharding hot-spot on heavy-use pair.
  **M**: Citus distributes on `agreement_id` (ULID-monotonic); no hotspot at agreement level.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: TrustArc and OneTrust provide audit trails and reports, Cookiebot keeps consent logs, and warehouse/data-exchange tools provide platform audit logs. This IP is narrower and stronger: every cross-tenant consent event produces paired grantor/grantee audit-chain entries with cross-pointers, signatures, and reconciliation semantics.

Grep-recognized counterpart anchor: Snowflake and Databricks data-sharing logs are relevant as clean-room audit counterparts, while Salesforce and HubSpot are relevant when consent state propagates into CRM audit trails. The primary comparator remains consent-platform audit semantics plus Oyatie bilateral audit-chain guarantees.
