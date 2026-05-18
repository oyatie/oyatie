---
ip_id: IP-015
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/quarterly-regulator-evidence
related_adrs: [ADR-0162, ADR-0174, ADR-0183, ADR-0197, ADR-0199]
depends_on: [IP-001, IP-002, IP-004, IP-014]
target_lines: 150
---

# IP-015 — Quarterly regulator-evidence emit

## Why this slice

Regulators in each pack (KR FSS, EU DPA, US healthcare CMS, US
financial OCC) require quarterly evidence of:

1. **Per-tenant cost-allocation policy in effect** — every active
   policy is sealed quarterly to the audit-chain.
2. **Per-tenant invoice totals** — quarterly rollup, signed.
3. **Credit-ledger position** — running balance at quarter close.
4. **Anomaly investigation outcomes** — every anomaly opened
   during the quarter has a sealed investigation event.

This slice authors the quarterly emit pipeline. It runs as a
scheduled job triggered by the calendar-quarter close (Mar 31,
Jun 30, Sep 30, Dec 31, 23:59 UTC) + a 5-day cure window for late
adjustments.

The emit is **signed** with the ops-finops quarterly signing key
(rotated quarterly per ADR-0162) and **sealed** to the audit-chain
under class `FinOpsQuarterlyReport`.

## Acceptance criteria

1. New crate `crates/oya-finops-portal-quarterly-emit/`.
2. Public function `emit_quarterly_report`:
   ```rust
   pub async fn emit_quarterly_report<R, L, A, S>(
       quarter: FiscalQuarter,
       repo: &R, ledger: &L, audit: &A, signer: &S,
   ) -> Result<ReportEmitted, EmitError>
   where
       R: TenantInvoiceRepository, L: CreditLedger,
       A: AuditEmitter, S: QuarterlySigner;
   ```
3. The report is a structured JSON envelope:
   - `quarter`, `emitted_at`, `signed_by` (key fingerprint),
     `signature`.
   - `tenants[]` — per-tenant: `tenant_id`, `quarter_total_cents`,
     `credits_applied_cents`, `policies_in_effect[]`,
     `anomalies[]`.
   - `fleet_total_cents`, `fleet_anomaly_count`.
4. Signing uses Ed25519 (per ADR-0162) with a key rotated
   quarterly; the key fingerprint is included in the envelope.
5. Emit lands on audit-chain under class `FinOpsQuarterlyReport`
   AND lands a Parquet copy in SeaweedFS at
   `regulator-evidence/{quarter}/q-report-{pack}.parquet` for
   FOCUS-style download.
6. Per-pack overlays: KR pack emits an additional PIPA-personal-
   data-redaction marker; EU pack emits a GDPR Article 30 record.
7. SEV-2 incident if emit slips > 5 days past quarter-close;
   surfaced via `slos/regulator-emit-availability.openslo.yaml`.
8. ≥ 6 integration tests:
   - happy quarterly emit round-trip.
   - missing tenant invoice → emit fails, alert fires.
   - signing-key rotation mid-emit handled gracefully.
   - re-emission idempotency (audit-chain dedups).
   - per-pack overlay correctness.
   - 5-day cure window allowance.

## File-level work plan

1. `Cargo.toml` — depends on usecase + ledger + audit-chain
   client + signer crate.
2. `src/lib.rs`.
3. `src/emit.rs` — main pipeline.
4. `src/envelope.rs` — JSON envelope shape.
5. `src/sign.rs` — Ed25519 signing wrapper.
6. `src/parquet.rs` — Parquet copy writer.
7. `src/overlay.rs` — per-pack overlay handling.
8. `src/schedule.rs` — calendar-quarter scheduler.

## Schedule

The job runs at quarter-close + 5 days (i.e. Apr 5, Jul 5, Oct 5,
Jan 5 23:59 UTC). The 5-day cure window allows for late
invoice finalization, credit application, and anomaly outcome
recording. The job retries up to 3 times with exponential backoff
on transient failures; final failure pages ops-finops manager.

## Signing key lifecycle (binds to ADR-0162)

- Each quarter has a dedicated Ed25519 key pair.
- Public keys are published to the audit-chain under class
  `FinOpsQuarterlyKeyPublished` at quarter-start.
- Private key is held in the per-pack HSM; signed only by the
  emit job.
- Verification: any consumer can verify the envelope signature
  against the published key. The verification process is
  documented in `compliance-matrix.md`.

## Per-pack overlay specifics

- **KR pack** (PIPA): emits additional marker
  `pii_redaction_applied: true` and includes a redaction-evidence
  hash for the FSS auditor.
- **EU pack** (GDPR): emits additional `gdpr_article_30_record`
  with controller / processor metadata.
- **US-healthcare pack** (HIPAA): redacts every dollar amount
  to per-tenant aggregate; never includes per-individual line.
- **US-financial pack** (SOX): includes a control-attestation
  marker that names the responsible ops-finops manager.
- **US-public-sector pack** (FedRAMP): includes a moderate-
  controls attestation block.

## Risk + mitigation

- **Risk**: signing key compromise. **Mitigation**: keys are
  rotated quarterly; old signatures remain verifiable against the
  published-key audit-chain entries.
- **Risk**: emit slips into the next quarter. **Mitigation**:
  SLO `regulator-emit-availability` fires at >5d slip; on-call
  ops-finops escalates per
  `runbooks/quarterly-regulator-emit-miss.md`.

## Out-of-scope

- The regulator-portal download surface — separate µservice.
- The HSM provisioning — secrets µservice.

## References

- ADR-0162 — per-tenant audit-log slicing + signing.
- ADR-0174 — chargeback formula.
- ADR-0197 — backup substrate (the Parquet copy lives in
  SeaweedFS via the backup-target seeded by cloud-iac).
- ADR-0199 — FinOps canonical.
- `slos/regulator-emit-availability.openslo.yaml`.
- `runbooks/quarterly-regulator-emit-miss.md`.

## Verification

- `cargo test -p oya-finops-portal-quarterly-emit`.
- `oya gate quarterly-emit-correctness --quarter <Q>`.
- Manual quarter-end rehearsal documented in
  `incident-playbook.md`.
