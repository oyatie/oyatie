---
doc_class: MigrationPlaybook
microservice: audit-chain
vendor: Splunk Audit (Enterprise + Audit entitlement)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Splunk Audit → oyatie audit-chain

Audience: an oyatie tenant or internal substrate team moving cryptographic audit evidence from Splunk Audit (the "Enterprise + Audit" entitlement; sometimes branded "Splunk Audit Manager") to oyatie's native `audit-chain` µservice. The driving reasons are usually: (a) per-GiB-indexed Splunk pricing scaling unsustainably at > 50 k events/sec sustained, (b) lack of cryptographic non-repudiation in Splunk's HMAC-based event signing, and (c) sovereignty / residency requirements that Splunk's regional-cloud topology can't satisfy.

## Why this migration matters

Splunk Audit's tamper-evidence is HMAC-based with a shared key per index. This means:

- An insider with HMAC-key access can forge events that look authentic — Splunk does not satisfy NIST SP 800-92's "non-repudiation" requirement strictly.
- The HMAC key is rotatable but the rotation is not Merkle-anchored, so verifying events older than the most-recent rotation requires Splunk's cooperation.
- Regulator-evidence export from Splunk is CSV; no auditor can independently verify the chain without trusting Splunk's process.

oyatie audit-chain replaces this with RFC 6962-shaped SHA-256 Merkle proofs + Ed25519-HSM signatures. The signing key never leaves the HSM at paid tenant_class; verification is standalone (no oyatie tooling needed).

## Step 1 — Inventory the Splunk audit indexes (≤ 1 day)

```sh
# From a Splunk search head with the audit_admin role:
splunk search "| metadata type=sourcetypes index=audit*" -auth admin:<password>
```

Document, for each audit index:

- Event count (last 12 months) — `| eventcount summarize=false index=audit_<name>`.
- Daily-ingest GiB and 12-month total — useful for sizing the oyatie cold-tier.
- HMAC key rotation history — `| rest /services/audit/keys` returns active + retired keys.
- Cross-index dependencies (saved searches, dashboards that join two audit indexes).

Typical inventory at a mid-size tenant: 4-12 audit indexes, 1.5-8 TiB/day ingest, 6-18 months retention, 4-8 active HMAC keys.

## Step 2 — Plan the source-of-truth cutover model (≤ 1 week)

Three patterns. Pick one per audit class:

1. **Dual-emit during shadow** (recommended for first 4-6 weeks): each emitting µservice writes to BOTH Splunk Audit (via Splunk HEC) AND oyatie audit-chain. Compare daily reconciliation reports. Cut over once drift is < 0.01 % for 14 d.
2. **Splunk-historical, oyatie-forward** (for tenants who can't dual-emit): freeze Splunk on cutover day, redirect new events to oyatie. Historical Splunk events remain queryable in Splunk's archive for the legal retention period (typically 7 y); new events live in oyatie.
3. **Backfill + retire Splunk** (only for tenants whose retention requirement permits): export all Splunk historical events, backfill into oyatie as "imported" events (clearly marked as not natively oyatie-emitted; carry a Splunk source attestation), retire Splunk.

Pattern (3) is heaviest because Splunk export is per-event CSV and oyatie's import flow re-anchors with the oyatie signing key (since the Splunk HMAC is not transferable). The "imported" attestation preserves the Splunk-era HMAC + key-rotation history alongside the new oyatie Merkle anchor, but auditors must understand the dual-attestation model.

## Step 3 — Wire emitter µservices to dual-emit (≤ 5-10 days)

For each emitting µservice (`iam`, `payments`, `workflow-engine`, `intelligence`, etc.), add the oyatie emission adapter alongside the existing Splunk emitter:

```rust
// Existing Splunk emitter (kept during shadow):
splunk_audit_emitter.emit(SplunkAuditEvent {
    index: "audit_iam",
    event_type: "role_assigned",
    payload: json!({"principal_id": ..., "role": ...}),
}).await?;

// New oyatie audit-chain emitter (in parallel):
audit_chain_emitter.emit(AuditEvent {
    event_class: "iam.role.assigned".into(),
    tenant_ids: vec![tenant_id.clone()],
    principal_id: principal_id.clone(),
    payload: serde_json::to_vec(&payload)?,
    ..Default::default()
}).await?;
```

Critical: dual-emit must be **best-effort dual** — if one side fails, the other still emits, and an `audit_chain.emit_drift_detected` event is recorded so the reconciliation report flags the discrepancy.

The emit-adapter ships in `oya-audit-emission-adapter` ≥ 1.18.0 and includes a `dual_emit_mode = ("splunk-primary" | "oyatie-primary" | "best-effort-both")` knob — set to `"best-effort-both"` during shadow.

## Step 4 — Reconciliation report (daily; for the full shadow window) (≤ 14-42 d)

```sh
oya audit reconcile \
    --source-a splunk-audit \
    --source-b oyatie-audit-chain \
    --tenant acme-corp \
    --window-day 2026-05-20 \
    --report ./reconciliation-2026-05-20.json
```

The report compares per-event-class counts + samples mismatched events:

```json
{
  "window": "2026-05-20",
  "event_classes": {
    "iam.role.assigned": {
      "splunk_count": 1428,
      "oyatie_count": 1428,
      "drift_pct": 0.00,
      "mismatch_sample": []
    },
    "payments.invoice.issued": {
      "splunk_count": 421,
      "oyatie_count": 423,
      "drift_pct": 0.47,
      "mismatch_sample": [
        {"event_id_oyatie": "01HZX2K3...", "side": "oyatie-only", "reason": "late-arriving event past splunk_search_window"},
        {"event_id_oyatie": "01HZX2K4...", "side": "oyatie-only", "reason": "late-arriving event past splunk_search_window"}
      ]
    }
  },
  "total_drift_pct": 0.02
}
```

Acceptable cutover drift: < 0.01 % across all classes for ≥ 14 consecutive days. Common drift sources we tolerate:

- Late-arriving events (Splunk's search window typically truncates at 24 h; oyatie has no such truncation).
- Events > 32 KiB (Splunk's per-event soft limit; oyatie has no per-event size cap).

Common drift sources we DO NOT tolerate (block cutover):

- Missing event classes (oyatie not emitting at all for a particular class).
- HMAC verification failures on Splunk side (indicates Splunk-side compromise — separate incident).
- Cedar denials on oyatie side (the emit adapter should be in a permitted Cedar role; investigate).

## Step 5 — Cut over (one-time, ≤ 4 h scheduled window)

```sh
# Step 5a: flip the emit-adapter flag to oyatie-primary
oya governance set-config \
    --µservice all \
    --key audit_dual_emit_mode \
    --value oyatie-primary

# Step 5b: verify next-hour reconciliation report still under 0.01 % drift
oya audit reconcile --window-hour now-1h

# Step 5c: stop the Splunk HEC writers
oya governance set-config \
    --µservice all \
    --key audit_dual_emit_mode \
    --value oyatie-only

# Step 5d: emit the cutover audit event
oya audit emit \
    --tenant acme-corp \
    --event-class governance.audit_substrate.cut_over \
    --payload '{"from":"splunk-audit","to":"oyatie-audit-chain","cutover_at":"2026-05-20T14:00:00Z"}'
```

After Step 5d, every µservice emits ONLY to oyatie audit-chain.

## Step 6 — Splunk decommission + historical-access retention (≤ 30 d post-cutover)

Don't delete the Splunk audit indexes immediately. Run for ≥ 30 d post-cutover:

- Splunk indexes remain read-only for historical query.
- Auditors with active engagements continue to query Splunk for pre-cutover periods.
- New oyatie audit-chain becomes the source of truth for post-cutover periods.

At day +30, evaluate:

- Are any active audit engagements still querying Splunk for pre-cutover data? If yes, extend Splunk retention for the engagement's duration.
- Is the Splunk-retention regulatory minimum (e.g., SEC 17a-4 7 y) still in force? If yes, archive Splunk events to cold storage (S3 Glacier Deep Archive) and decommission the Splunk Audit entitlement.

The decommission emits one final audit event:

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.audit_substrate.decommissioned \
    --payload '{"vendor":"splunk-audit","decommission_at":"2026-06-20T14:00:00Z","historical_archive":"s3://acme-corp-audit-archive/splunk/"}'
```

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Drift > 0.01 % during shadow | High | Block cutover; investigate root cause; common: missed event class |
| Splunk HEC backpressure causes drift | Medium | Pre-stage the migration with HEC capacity expansion |
| HMAC-key transfer not legally permitted (Splunk SaaS contract) | Low | Pattern (1) or (2) — don't try to re-anchor Splunk events with oyatie signing |
| Auditors expect Splunk-shaped CSV exports | Medium | The `regulator-export` script ships a Splunk-compatible CSV view alongside the verifiable JSONL |
| Per-event-size > 32 KiB events break dual-emit | Low | Splunk-side: truncate with a pointer to the full payload (already a Splunk convention) |
| Cedar denies the new oyatie emit-adapter principal | Low | Pre-deploy the Cedar permits + smoke-test the emit path before flipping |
| Cross-µservice emission-adapter version skew | Medium | Roll out the adapter version pin from 1.18.0 to all µservices BEFORE enabling dual-emit |
| Splunk historical-search dependency surfaces post-cutover | Medium | Extend Splunk read-only retention until all dependent searches migrate |
| Regulator audit period spans cutover | High | Document the cutover in the regulator-evidence narrative; provide both Splunk + oyatie exports for the period |
