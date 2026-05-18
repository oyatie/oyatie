# consent-graph partnership onboarding

- Owner: axis-consent-graph + partnerships
- Date: 2026-05-18
- Authority: IP-014 partner-directory handshake.

## 1. End-to-end onboarding flow

```
[Partner Manager A]  → finds Partner B via marketplace or out-of-band introduction
       │
       ▼
[Partnership UI]     → "Invite Partner B"
       │
       ▼
[partner-directory] → POST /handshake/initiate { peer_tenant_id: B }
       │
       ▼
[Three-leg handshake]  → mTLS + anchor exchange + audit-chain root proof (≤30s)
       │
       ▼
[Verified state]     → A's directory has B as Verified
       │
       ▼
[Compliance review]  → audit-officer approves Verified→Active transition (manual day-1)
       │
       ▼
[Active state]       → A may now draft agreements with B
       │
       ▼
[First agreement]    → A drafts → offers → B accepts → Active
       │
       ▼
[Operational]        → projection flowing; revocations real-time; bilateral audit live
```

## 2. Pre-handshake prerequisites

Partner B must already be:
- An oyatie tenant (B has signed up).
- Running consent-graph in their region (this is the canonical EaaS substrate — required).
- Running audit-chain (required for bilateral chain).

A pre-flight check via the partner-directory API confirms these capabilities before initiating
handshake.

## 3. Handshake operations

### 3.1 Leg 1 — A → B initiate
- A's partner-directory-app opens HTTPS+mTLS connection to B's `/v1/partner-directory/handshake/respond`.
- Includes A's nonce + claimed anchors (A's X.509 SPKI, Pulsar JWT issuer SPKI, A's audit-chain root).

### 3.2 Leg 2 — B → A respond
- B verifies A's anchors (X.509 chain to trusted root or pre-trusted; audit-chain root has valid
  inclusion proof from A's audit-chain).
- B replies with B's nonce + B's anchors + HMAC over (A-nonce, B-anchors).

### 3.3 Leg 3 — A → B finalize
- A verifies B's anchors symmetrically.
- A replies with HMAC over (B-nonce, A-anchors).
- Both sides commit: persist `consent_graph_partner_tenants` row.

### 3.4 Commit + audit
- Both sides emit `oya.consent-graph.partner-handshake-completed` to audit-chain.
- Both sides emit `oya.consent-graph.partner-state-changed` (∅ → Verified).

## 4. Verified → Active transition

PHASE-01 requires audit-officer manual approval. Procedure:
1. Audit-officer reviews handshake evidence (audit-chain entries + Postgres partner row).
2. Cross-checks B's organizational identity via out-of-band channel (legal entity verification).
3. Approves via `POST /v1/partner-directory/{B}/approve` (RLS: only audit-officer role).
4. State transitions Verified → Active; audit-chain emission.

PHASE-02 introduces automated approval for pre-validated tenants (e.g., enterprise tier).

## 5. Failure modes during onboarding

| Failure | Class | Resolution |
|---------|-------|-----------|
| Peer not running consent-graph | pre-flight | block; require peer to deploy |
| Peer's audit-chain root proof invalid | handshake | abort handshake; alert security review |
| Schema version mismatch | handshake | abort; require upgrade |
| mTLS cert chain not trusted | handshake | offer pre-trust option (out-of-band) |
| Handshake times out | handshake | retry up to 3× over 24h |
| Audit-officer rejects | post-handshake | rollback to ∅; reasons recorded |

## 6. UX

Partner-Manager UI (in ops-portal):
1. **Invite partner**: search marketplace or enter tenant-id directly.
2. **Handshake progress**: live indicator of Leg 1/2/3.
3. **Anchor verification**: side-by-side comparison of claimed vs verified anchors.
4. **Status**: Onboarding → Verified → Active progression.
5. **Audit officer queue**: pending Verified→Active approvals.

## 7. Agreement template assignment

Once Active, Partner-Manager may issue agreements. PHASE-01 ships 5 templates (see IP-002 §7).
Workflow:
1. Select template.
2. Override defaults (scope, terms, expiration, etc.).
3. Data-steward review (mandatory before Offer).
4. Send to grantee.

## 8. Partner offboarding

Either side may offboard. Procedure in `runbooks/partner-offboarding.md`:
1. Initiator calls `POST /v1/partner-directory/{peer}/offboard`.
2. All active agreements transitioned to `Revoked{PartnerOffboarded}`.
3. All projection topics destroyed.
4. Partner state → Offboarded (terminal).
5. Bilateral audit emission.

Offboarding is mutually visible; the other side cannot resist offboarding (cross-tenant trust is
unilaterally revocable).

## 9. Tooling

`oya consent-graph partner <subcommand>`:
- `oya consent-graph partner invite <peer>` — initiate handshake.
- `oya consent-graph partner list` — show all partners + state.
- `oya consent-graph partner show <peer>` — show anchors + audit-chain root.
- `oya consent-graph partner offboard <peer> --reason <r>` — terminal offboard.
- `oya consent-graph partner suspend <peer> --reason <r>` — temporary suspend.

## 10. Metrics + SLO

- `partner-handshake-latency` SLO p95 ≤30s.
- Onboarding funnel dashboard: count per stage (Onboarding / Verified / Active / Offboarded).
- Audit-officer approval-pending count.

## 11. PHASE-02 follow-ups

- Marketplace discovery µservice (find partners without out-of-band intro).
- Automated approval for pre-validated tenants.
- Bulk-onboarding API (enterprise rolling out 100+ partners at once).
- Partner self-service portal (B's side initiates handshake first).
