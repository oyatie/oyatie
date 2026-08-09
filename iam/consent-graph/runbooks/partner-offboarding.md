# Runbook: partner-offboarding

- Severity: routine (P2 if cascade times out)
- Trigger: either party initiates `POST /v1/partner-directory/{peer}/offboard` OR
  audit-officer flags partner for security incident.
- Authority: IP-014 §12, partnership-onboarding.md §9.

## Step 1 — Initiate (≤5min)

1. UI or CLI: `oya consent-graph partner offboard <peer> --reason "<reason>"`.
2. Partner record state transitions Active|Suspended|Verified → Offboarded.
3. Bilateral audit emission `oya.consent-graph.partner-offboarded`.
4. Both sides update their partner record symmetrically.

## Step 2 — Cascade revoke all agreements (≤5min)

Automatic worker action:
1. Enumerate active agreements where (grantor=this OR grantee=peer) AND (grantee=this OR grantor=peer).
2. For each, originate revocation with reason `PartnerOffboarded`.
3. Standard revocation propagation: ≤1s p99 per agreement.
4. Bulk-rate-limited to 1K revocations/min to protect Pulsar.

## Step 3 — Destroy projection topics (≤10min)

For each revoked agreement:
1. `projection-gateway::destroy` triggered.
2. Pulsar topic ACL revoked; topic unloaded; deletion scheduled (1h grace period).
3. Bilateral audit emission `projection-destroy`.

## Step 4 — Revoke OpenBao tokens (≤5min)

1. All projection JWT tokens for the (grantor, grantee) pair added to revocation list.
2. Per-pair HMAC key marked for destruction (90d retention for forensic).
3. Audit emission `secrets-rotated`.

## Step 5 — Bilateral audit reconciliation (within 24h)

1. IP-013 reconciler runs over offboarding-day window.
2. Confirm zero divergence on offboarding cascade.
3. Final reconciliation report sealed.

## Step 6 — Operational cleanup (within 7d)

1. Stop billing relationship for the pair (out-of-band; finops-portal).
2. Archive partner record + bilateral chain pointers (keep retained for 7y audit).
3. Cross-tenant subscribers in ontology/analytics/observability auto-clean their grantee-side caches
   on receipt of revocation events; verify via metrics dashboard.

## Step 7 — Cannot-undo

Offboarding is **terminal**. To re-establish relationship:
1. Treat as fresh onboarding (new handshake).
2. Bilateral chains restart from scratch (no carry-over).
3. Prior offboarding event remains in audit-chain history (audit-citable).

## Verification

- Sample-read attempt by grantee after offboarding: receives Deny{PartnerSuspended} within 1.5s.
- IP-013 reconciliation post-offboarding: zero divergences.
- No projection events emitted post-offboarding for the pair.

## Audit evidence

- Offboarding event sealed bilaterally.
- Cascade revocation events sealed (one per agreement, bilateral).
- Final reconciliation report sealed.
- All retained 7y.

## Edge cases

- **Peer simultaneously offboards**: idempotent; both sides land in Offboarded; no conflict.
- **Peer is unreachable** (disaster scenario): offboarding proceeds unilaterally; bilateral chain entry
  on peer side becomes "missing"; flagged for IP-013 review on recovery.
- **In-flight grant offer during offboarding**: discarded; new draft on peer side cannot proceed
  (Offboarded gate).

## Escalation

- Cascade fails to complete within 1h → P2 alert; investigate Pulsar / Postgres.
- Cascade fails to complete within 24h → P1 alert; manual revocation tooling.
- Bilateral reconciliation shows divergence post-cascade → escalate to
  audit-chain-divergence-recovery.md.
