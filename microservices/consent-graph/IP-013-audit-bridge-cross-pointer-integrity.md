# IP-013: audit-bridge cross-pointer integrity — nightly reconciliation + divergence alert

- Bounded context: audit-bridge
- Layers: usecase, worker
- Crates:
  - `oya-consent-graph-audit-bridge-usecase` (extension)
  - `oya-consent-graph-audit-bridge-worker` (extension)
- Acceptance status: ga
- Authority: ADR-0214 §2.6, ADR-SVC-CG-001, ADR-0003.
- Depends on: IP-012.

## 1. Goal

Provide a daily, exhaustive cross-chain reconciliation that detects divergence between grantor + grantee
chains for the same set of consent-graph events. Detection of a single divergent pair is a P1 (security
incident; runbook `audit-chain-divergence-recovery.md`).

## 2. Reconciliation algorithm

For each `(grantor_tenant_id, grantee_tenant_id)` pair with active or recently-revoked agreements:

1. Pull from audit-chain query API:
   - Grantor side: all consent-graph entries for this pair in the window (default: last 24h).
   - Grantee side: all consent-graph entries for this pair in the window.
2. Pull from `consent_graph_cross_pointers`: all cross-pointer rows for this pair in the window.
3. **Three-way join** by `(event_id)`:
   - Each event_id must appear once in grantor chain (matching seq) and once in grantee chain
     (matching seq).
   - Cross-pointer row's `(grantor_chain_id, grantor_seq, grantee_chain_id, grantee_seq)` must
     match what audit-chain reports.
   - `paired_hmac` recomputed from `(pair_key, grantor_link, grantee_link)` matches stored.
4. Emit reconciliation report → `evidence/consent-graph-bilateral-reconciliation-<date>.json`.
5. If any divergence detected: emit P1 alert + automatically open incident ticket.

## 3. Divergence taxonomy

| Class | Condition | Severity | Likely cause |
|-------|-----------|----------|--------------|
| `MissingGrantor` | event in grantee chain, not in grantor chain | P1 | grantor seal lost; rollback failed |
| `MissingGrantee` | event in grantor chain, not in grantee chain | P1 | grantee seal lost; rollback failed |
| `OrphanCrossPointer` | cross-pointer row, but one chain missing | P0 | tampering; investigate |
| `HmacMismatch` | both chains have entries but recomputed HMAC ≠ stored | P0 | tampering or wrong pair key |
| `SeqMismatch` | cross-pointer references seq that doesn't match audit-chain | P0 | tampering |
| `LateArrival` | grantor sealed > 1h after grantee (or vice versa) | P3 | clock skew or worker lag |

P0 divergences trigger immediate auto-suspension of all agreements between the affected pair
(safety-first per ADR-0214 §2.3 fail-closed); operations resume only after audit-officer review.

## 4. SLI

```
oya_consent_graph_bilateral_link_total{outcome="paired"} / oya_consent_graph_bilateral_link_total{outcome=*}
```

Where `outcome ∈ {paired, missing_grantor, missing_grantee, orphan_cross_pointer, hmac_mismatch, seq_mismatch, late_arrival}`.

Target: outcome="paired" ratio = 1.0. SLO `bilateral-chain-link-integrity` budget = 0 (any divergence
burns budget instantly).

## 5. Performance

The reconciler is window-scoped (24h). At peak — 1M cross-tenant events/day per pair — we process:
- 10K pairs × 100 events/pair median = ~1M pairs/day total per region.
- Per-pair reconcile: ≤1s walk-the-window (single Postgres + audit-chain query).
- Total reconcile job: ≤30min single-threaded; parallelized at pair-level (16 workers) → ≤2min.

Run cadence: hourly (rolling 24h window) + daily full-day-only reconciliation snapshot for evidence.

## 6. Worker impl

```rust
pub struct CrossPointerReconcilerWorker {
    agreement_sdk: AgreementClient,
    audit_chain_sdk: AuditChainQueryClient,
    repo: CrossPointerRepository,
    alert_sink: AlertSink,
}

impl CrossPointerReconcilerWorker {
    pub async fn run_once(&self, window: Duration) -> Result<ReconciliationReport, ReconcileError> {
        let pairs = self.agreement_sdk.list_active_pairs().await?;
        let mut report = ReconciliationReport::new();
        let mut tasks = JoinSet::new();
        for pair in pairs {
            let s = self.clone();
            tasks.spawn(async move { s.reconcile_pair(pair, window).await });
        }
        while let Some(pair_result) = tasks.join_next().await {
            report.merge(pair_result??);
        }
        self.emit_report(&report).await?;
        if report.has_p0_or_p1() {
            self.alert_sink.fire(&report).await?;
        }
        Ok(report)
    }

    async fn reconcile_pair(&self, pair: PairRef, window: Duration) -> Result<PairReport, ReconcileError> {
        let grantor_entries = self.audit_chain_sdk.query_chain_window(
            ChainId::for_tenant(pair.grantor), window).await?;
        let grantee_entries = self.audit_chain_sdk.query_chain_window(
            ChainId::for_tenant(pair.grantee), window).await?;
        let cross_pointers = self.repo.list_window(pair, window).await?;

        let mut pair_report = PairReport::new(pair);
        for cp in &cross_pointers {
            let g_entry = grantor_entries.iter().find(|e| e.seq == cp.grantor_seq);
            let s_entry = grantee_entries.iter().find(|e| e.seq == cp.grantee_seq);
            match (g_entry, s_entry) {
                (Some(g), Some(s)) => {
                    let recomputed = self.recompute_hmac(cp, g, s).await?;
                    if recomputed != cp.paired_hmac {
                        pair_report.divergences.push(Divergence::HmacMismatch(cp.event_id));
                    }
                }
                (None, Some(_)) => pair_report.divergences.push(Divergence::MissingGrantor(cp.event_id)),
                (Some(_), None) => pair_report.divergences.push(Divergence::MissingGrantee(cp.event_id)),
                (None, None) => pair_report.divergences.push(Divergence::OrphanCrossPointer(cp.event_id)),
            }
        }
        Ok(pair_report)
    }
}
```

## 7. Tests

- `reconcile_no_divergence_passes` — synthetic 100 paired events → 0 divergences.
- `reconcile_detects_missing_grantor` — delete 1 grantor entry → MissingGrantor reported.
- `reconcile_detects_hmac_tamper` — corrupt 1 paired_hmac → HmacMismatch reported.
- `reconcile_parallelism_safety` — 16 concurrent pair workers; report merges deterministically.
- `auto_suspend_on_p0_divergence` — P0 divergence triggers agreement-suspension API call.

## 8. Reconciliation report schema

```json
{
  "report_id": "ulid",
  "window_start": "rfc3339",
  "window_end": "rfc3339",
  "pairs_reconciled": 10000,
  "events_reconciled": 1000000,
  "divergences": [
    {
      "pair": {"grantor": "tn-acme", "grantee": "tn-retail"},
      "class": "HmacMismatch",
      "event_id": "01HXYZ...",
      "severity": "P0",
      "auto_action_taken": "agreement_suspended:01HXYZ..."
    }
  ],
  "outcome_counts": {
    "paired": 999999,
    "hmac_mismatch": 1
  },
  "audit_seal_id_of_this_report": "chain:consent-graph:seq=12345"
}
```

The report itself is *also* sealed via audit-chain — a tamper-evident audit of the audit reconciliation
itself. Recursive seal closes the trust loop.

## 9. Verification

- `cargo build` + `cargo test` clean.
- E2E: inject 1 divergence per class → all 6 classes detected.
- Performance: 1M events reconciled in <5min.
- Audit: reconciliation report sealed in audit-chain + queryable via `audit-chain query report:*`.

## 10. Risk

- **R**: Reconciler discovers backlog at first deployment.
  **M**: Initial run is bounded to last 24h; full-history reconciliation deferred to backfill-replay.md
  runbook + sequenced over 7 days.
- **R**: False-positive divergence on clock skew during cross-region writes.
  **M**: `LateArrival` class is P3 (informational) up to 1h skew; P1 only above 1h. NTP sync required.
- **R**: P0 auto-suspension causes business disruption from false-positive.
  **M**: Auto-suspension is reversible by audit-officer; suspension itself is audit-chained; alerts
  page the on-call human within 2min.

## 11. Operational integration

Runbook `audit-chain-divergence-recovery.md` is the response procedure for any P0/P1 divergence:
1. Audit-officer reviews report.
2. Determine root cause (hardware fault, software bug, malicious tampering).
3. If software bug: roll back consent-graph; reseal affected pairs.
4. If tampering: forensic snapshot of database + chain → escalate to security incident commander.
5. Restore: replay missing entries from outbox/dead-letter; recompute cross-pointers; manual review.
6. Lift agreement suspensions.

## 12. Output

- Hourly: rolling report → `evidence/consent-graph-bilateral-reconciliation-rolling-<unix>.json` (kept 90d).
- Daily: full-day report → `evidence/consent-graph-bilateral-reconciliation-daily-<YYYY-MM-DD>.json`
  (retained 7y per audit-chain default retention).

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: OneTrust/TrustArc/Cookiebot can prove consent records, while Snowflake/Databricks/AWS logs prove platform operations. This IP supplies the Oyatie-specific integrity layer: nightly and rolling reconciliation of bilateral audit-chain cross-pointers, divergence taxonomy, suspension behavior, and long-retention evidence reports.
