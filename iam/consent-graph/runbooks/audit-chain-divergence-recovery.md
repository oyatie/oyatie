# Runbook: audit-chain-divergence-recovery

- Severity: P0 (chain integrity)
- Trigger: IP-013 reconciler emits MissingGrantor / MissingGrantee / OrphanCrossPointer / HmacMismatch /
  SeqMismatch / LateArrival(>1h).

## Step 1 — Acknowledge (≤5min)

1. Page on-call + audit-officer.
2. Open incident; halt new agreement-create for affected pair via `oya consent-graph partner suspend
   <peer>`.

## Step 2 — Classify divergence (≤15min)

`oya consent-graph reconcile report --window 24h --pair <grantor>,<grantee>` outputs detailed report.

Match to one of:

### 2.1 MissingGrantor
- Grantee chain has the event, grantor does not.
- Likely: grantor seal failed mid-bilateral emit; rollback didn't fire OR rollback fired but logged
  oddly.
- Action: examine grantor's audit-chain outbox for the event_id; if present, replay; if absent,
  delete grantee-side entry (rare; emit "rollback-late" audit class).

### 2.2 MissingGrantee
- Mirror of 2.1.

### 2.3 OrphanCrossPointer
- Cross-pointer row exists; one or both chain entries missing.
- Action: delete orphan row; emit `cross-pointer-cleanup` audit event.

### 2.4 HmacMismatch
- Cross-pointer's `paired_hmac` doesn't recompute from current key.
- Likely: OpenBao key rotation; OR tampering.
- Action:
  1. Try recomputation with previous key version.
  2. If verifies with prev key: legitimate rotation; backfill recomputed HMACs with current key.
  3. If still mismatches: escalate to consent-forgery-detected.md.

### 2.5 SeqMismatch
- Cross-pointer claims (chain, seq) but audit-chain reports different.
- Action: re-query audit-chain; if seq differs, audit-chain re-indexed; update cross-pointer.

### 2.6 LateArrival(>1h)
- Bilateral seals > 1h apart.
- Likely: cross-region pulsar lag during outage.
- Action: log; tag affected entries with `late_arrival=true`; no remediation needed unless > 24h.

## Step 3 — Replay missing entries (≤30min)

For 2.1 / 2.2:
1. Pull event from outbox: `SELECT * FROM consent_graph_<bc>_outbox WHERE event_id = ...`.
2. Re-emit via `oya consent-graph audit-bridge replay-event <event-id>`.
3. Verify both sides now have entries.
4. Recompute paired_hmac; update cross_pointers row.

## Step 4 — Validate (≤15min)

1. Run IP-013 reconciler again on the same window: expect zero divergences.
2. Spot-check 10 random other agreements for the pair: confirm no collateral damage.
3. Restore service (lift partner suspension):
   `oya consent-graph partner resume <peer>`.

## Step 5 — Regulatory disclosure

Only if HmacMismatch + tampering confirmed (Step 2.4 → consent-forgery-detected.md path).

Routine divergences (rollback bugs, late arrival) do NOT trigger external disclosure.

## Step 6 — Post-mortem (≤48h)

- Capture all timestamps + actions.
- Code fix if rollback logic faulty.
- ADR-SVC-CG-* if structural.

## Verification

- Reconciliation report shows zero divergences in window.
- Sample test: emit a deliberately-failing bilateral event; verify rollback fires; verify no orphan.

## Audit evidence

- All replay actions sealed.
- Recomputed HMACs include `regenerated_at` field.
- Post-mortem doc sealed.
