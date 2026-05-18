# IP-014: partner-directory — peer handshake + trust anchor verification + audit-chain root proof

- Bounded context: partner-directory
- Layers: kernel, domain, usecase, api, rest, adapter, app
- Crates:
  - `oya-consent-graph-partner-directory-kernel`
  - `oya-consent-graph-partner-directory-domain`
  - `oya-consent-graph-partner-directory-usecase`
  - `oya-consent-graph-partner-directory-api`
  - `oya-consent-graph-partner-directory-rest`
  - `oya-consent-graph-partner-directory-adapter`
  - `oya-consent-graph-partner-directory-app`
- Acceptance status: ga
- Authority: ADR-0214 §1 (HIE/Open Banking parity), §2 (partner-directory bounded context),
  ADR-0072 (OpenBao secret substrate), ADR-0003 (audit-chain root proof).
- Depends on: `oya-consent-graph-audit-bridge-sdk`, `oya-audit-chain-verification-sdk`,
  `oya-shared-{mtls, jwt-issuer, x509-spki}`.

## 1. Goal

Before two oyatie tenants can exchange a `DataSharingAgreement`, they must perform a one-time
*handshake* that:
1. Mutually authenticates via mTLS.
2. Exchanges trust anchors (X.509 cert + Pulsar JWT issuer pub-key + audit-chain Merkle root).
3. **Verifies the peer's audit-chain Merkle root** — proves the peer actually runs an audit-chain
   µservice with intact chain. This is the trust-but-verify primitive that makes bilateral chain
   audit defensible.
4. Records the peer as a `PartnerTenant` in directory state.

Subsequent agreements bypass the handshake (peer is already `Verified`).

## 2. Why root proof matters

A bilateral audit chain only has integrity if *both* sides have working chains. If grantee runs an
empty/fake audit-chain, the bilateral entries are useless for forensics. The handshake's audit-chain
root verification step gives both sides cryptographic assurance that the peer:
- Has an audit-chain µservice deployed (signs the proof with audit-chain HSM key).
- Has chain entries (Merkle root is non-empty).
- Has the same audit-chain schema version (forward-compat).

This is novel — no existing inter-tenant data-sharing protocol (Plaid, FAPI, HL7 Direct, Ariba) does
this. It's part of the EaaS moat.

## 3. Handshake protocol

Three-leg handshake (think Diffie-Hellman style mutual proof):

```
Tenant A initiates handshake to Tenant B:
  Leg 1: A → B  [mTLS handshake, A presents cert chain]
  Leg 2: B → A  Handshake proof:
            - B's X.509 SPKI fingerprint
            - B's Pulsar JWT issuer pub-key fingerprint
            - B's audit-chain Merkle root (signed by B's audit-chain HSM key)
            - B's audit-chain schema version
            - Nonce-N from B
            - HMAC(nonce-A || B-anchors, key=B-handshake-secret)
  Leg 3: A → B  Reciprocal proof (same shape, A's anchors).
  Verify: each side verifies the other's proof signatures + Merkle root + schema version.
  Commit: each side persists the partner record + emits audit-chain entry.
```

All three legs over a single HTTPS+mTLS connection; total wall-clock budget ≤30s p95 (per SLO
`partner-handshake-latency`).

## 4. Trust anchor verification

After exchange, A verifies B's anchors:
1. X.509: cert chain reaches a root-of-trust in A's pinned root store. Self-signed certs only valid
   if A explicitly pre-trusts B (out-of-band agreement).
2. Pulsar JWT issuer pub-key: SPKI fingerprint matches what B claims; subsequent JWT tokens issued by
   B for projection topic subscriptions verify against this pub-key.
3. Audit-chain Merkle root: A queries B's audit-chain-verification API with the claimed root; B's
   audit-chain returns an inclusion proof; A verifies the proof; A also independently re-hashes the
   most recent published epoch root from B's audit-chain (defense against B's verification API lying).
4. Schema version: compatibility check; mismatch ≥ 2 versions → reject handshake (incompatible audit
   formats; both sides must upgrade).

## 5. Partner state machine

```
∅ → Onboarding   (initiate)
Onboarding → Verified    (handshake complete + anchors verified)
Verified → Active        (first agreement issued/accepted)
Active → Suspended       (security incident, schema mismatch detected post-handshake, etc.)
Suspended → Active       (re-verify + clearance)
Verified/Active/Suspended → Offboarded  (terminal; all agreements revoked)
```

`Suspended` partner: existing agreements automatically suspended (not revoked); resumption pending
clearance.

## 6. REST surface

| Route | Direction | Body |
|-------|-----------|------|
| `POST /v1/partner-directory/handshake/initiate` | A→B | A's identity + proposed anchors |
| `POST /v1/partner-directory/handshake/respond` | B→A | B's anchors + proof (Leg 2) |
| `POST /v1/partner-directory/handshake/finalize` | A→B | A's reciprocal proof (Leg 3) |
| `GET /v1/partner-directory/{tenant_id}` | * | partner record (RLS-tenant-scoped) |
| `POST /v1/partner-directory/{tenant_id}/suspend` | local-only | suspension (audit-officer auth) |
| `POST /v1/partner-directory/{tenant_id}/offboard` | local-only | offboarding (cascades agreement revoke) |

OpenAPI spec: `contracts/openapi/consent-graph.yaml` (covers all partner-directory routes).

## 7. Schema

```sql
CREATE TABLE consent_graph_partner_tenants (
    local_tenant_id uuid NOT NULL,
    peer_tenant_id uuid NOT NULL,
    state text NOT NULL,
    peer_x509_spki bytea NOT NULL,
    peer_pulsar_jwt_pub bytea NOT NULL,
    peer_audit_chain_merkle_root bytea NOT NULL,
    peer_audit_chain_schema_version smallint NOT NULL,
    peer_audit_chain_proof jsonb NOT NULL,
    handshake_initiated_at timestamptz NOT NULL,
    handshake_completed_at timestamptz,
    state_changed_at timestamptz NOT NULL,
    state_payload jsonb,
    PRIMARY KEY (local_tenant_id, peer_tenant_id)
);
SELECT create_distributed_table('consent_graph_partner_tenants', 'local_tenant_id');

ALTER TABLE consent_graph_partner_tenants ENABLE ROW LEVEL SECURITY;
CREATE POLICY partner_read ON consent_graph_partner_tenants FOR SELECT
  USING (local_tenant_id = current_tenant_id());
```

Note this is **symmetric**: A's view has `(A, B)` row; B's view has `(B, A)` row. Each side
independently maintains its own partner record.

## 8. Tests

| Test | Assertion |
|------|-----------|
| `handshake_three_leg_completes_under_30s` | full handshake p95 ≤30s on dev |
| `handshake_rejects_invalid_x509_chain` | self-signed cert without pre-trust → rejected |
| `handshake_rejects_audit_chain_proof_failure` | tampered Merkle root → rejected + audit emission |
| `handshake_rejects_schema_mismatch_too_old` | peer audit-chain v0 vs local v2 → rejected |
| `handshake_idempotent` | re-handshake with same peer → state stays Verified, anchors updated |
| `suspend_cascades_agreement_suspend` | suspending peer suspends all agreements with them |
| `offboard_cascades_agreement_revoke` | offboarding peer revokes all agreements with them |
| `rls_blocks_cross_tenant_read` | tenant C cannot read A-B partner record |

## 9. Verification

- `cargo build` + `cargo test` clean.
- E2E: spin up two consent-graph instances on separate tenants; perform handshake; both record peer
  as Verified.
- Negative test: corrupt peer's audit-chain proof mid-handshake → rejection + audit entry.
- Latency: p95 ≤30s on dev (matches SLO).

## 10. Risk

- **R**: Peer's audit-chain HSM key rotated mid-relationship → proof verification fails on new entries.
  **M**: Handshake captures peer's HSM key version + chain schema version; per-key-version verification;
  re-handshake protocol auto-triggered on first verification failure.
- **R**: Peer's audit-chain compromised post-handshake (i.e., they're now lying).
  **M**: IP-013 daily reconciliation catches divergence; partner auto-suspended on P0.
- **R**: Onboarding bottleneck for first 100 partners (each handshake requires audit-chain query).
  **M**: Handshake is bilateral but stateless server-side; horizontal scale via partner-directory-app
  pods.
- **R**: mTLS cert renewal breaks handshake replay.
  **M**: SPKI fingerprint pinning (not full cert pinning); cert renewal preserves SPKI; rare key-rotation
  triggers re-handshake.

## 11. Onboarding UX

Real-world partner onboarding is human-mediated initially:
1. Partner-Manager A finds Partner B via marketplace (future µservice; manual lookup for now).
2. A's UI initiates handshake; system performs Legs 1-3 automatically.
3. On Verified: A can now draft agreements with B.

Runbook `partner-onboarding.md` covers the operational checklist for the audit-officer review at
Verified→Active transition.

## 12. Self-revocation of partnership

Either side may unilaterally offboard the other. Offboarding cascades:
1. All active agreements with the peer transition to `Revoked{PartnerOffboarded}`.
2. All projection topics with the peer destroyed.
3. All Pulsar JWT tokens for this pair revoked.
4. Partner state → `Offboarded` (terminal).
5. Audit-chain bilateral entries on both sides.

Runbook `partner-offboarding.md` is the procedure.
