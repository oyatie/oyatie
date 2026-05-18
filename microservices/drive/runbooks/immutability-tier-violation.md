---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-IMMUTABILITY-VIOLATION
severity_class: sev-1
related_adrs: [ADR-DRIVE-0006]
related_slos: [immutability-tier-correctness]
owner_team: axis-drive + ops-security + compliance + council-architecture
date: 2026-05-17
doc_status: published
---

# Runbook: WORM immutability tier violation

## Symptom

One or more of:
- WORM-tier object deleted before retention floor.
- Periodic integrity scan reports hold-set vs storage layer mismatch.
- `oya_drive_immutability_tier_violation_total` non-zero.
- Tenant compliance officer reports missing WORM-tier object.

## Severity

**Sev-1**. WORM violation is zero-tolerance per ADR-DRIVE-0006. SEC 17a-4(f) / FINRA 4511 / HIPAA §164.316 compliance breach.

## First responder

ops-security on-call. **Immediately** engage axis-drive on-call + compliance + council-architecture. Engage tenant compliance officer within 1h.

## Diagnosis

### Step 1 — Confirm violation

```bash
# Integrity-scan output
cargo run -p oya-dev-cli -- vcs query --microservice drive \
  --query "immutability-integrity-scan --tenant <tenant_id>"

# Audit-chain replay for the file
cargo run -p oya-dev-cli -- vcs query --microservice drive \
  --query "audit-chain-replay --file-id <file_id>"
```

### Step 2 — Determine breach path

Three plausible paths per ADR-DRIVE-0006 § Threat Model:

- **Path A — Application-layer bypass** (LEAN-check regression; usecase didn't honour WORM check).
- **Path B — Postgres-layer bypass** (application role had UPDATE/DELETE on `immutability_record` it shouldn't have had).
- **Path C — Object-store-layer bypass** (object-store object-lock was disabled at bucket creation).
- **Path D — Insider tenant-root manual deletion** (despite Cedar refusal; would require multiple-layer bypass).

### Step 3 — Confirm scope

- Single file? Multiple files? Cross-tenant?

## Mitigation

### Immediate (within 15 min of detection)

1. **Suspend all delete paths** for the affected tenant:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive suspend-deletes \
     --tenant <tenant_id> \
     --reason "WORM-violation-investigation"
   ```
2. **Forensic snapshot** of object-store bucket + Postgres `immutability_record` table + audit-chain.
3. Engage tenant compliance officer + tenant security contact.

### Path A — Application-layer bypass (regression)

1. Roll back the calendar µservice canary:
   ```bash
   cargo run -p oya-dev-cli -- vcs canary rollback --microservice drive --to-stable
   ```
2. Page council-architecture. The regression is a code-level bypass of the WORM check.
3. Patch + regression test: file a same-day fix-up ChangeSet against `dev` with a test in `tests/worm-refuses-purge.rs`.

### Path B — Postgres role escalation

1. Page ops-security. Audit OpenBao role assignments.
2. Revoke any DELETE / UPDATE permission on `immutability_record` from application role.
3. Forensic deep-dive: who/what escalated the role.

### Path C — Object-store-lock disabled at bucket creation

1. Page ops-security. Audit object-store bucket configuration:
   ```bash
   garage bucket info <bucket> | grep object-lock
   ```
2. Re-enable object-lock compliance mode.
3. Forensic deep-dive: who/what disabled it.

### Path D — Manual insider deletion

1. Sev-1 insider incident. Engage council-architecture + legal.
2. Identity of the actor from audit-chain replay.

### Per-pack notification

- pack-us SEC 17a-4: SEC notification immediate.
- pack-us-healthcare HIPAA: HHS OCR 60-day notification chain.
- pack-eu GDPR: 72h supervisory-authority notification.
- pack-kr KR PIPA: 72h PIPC notification.
- pack-jp APPI: 3 business day notification.

### Possible restoration

If WORM-violation deletion path detected within RPO (≤ 60s), restore from secondary-region replica before crystallisation:

```bash
cargo run -p oya-dev-cli -- vcs admin drive worm-restore-from-replica \
  --tenant <tenant_id> \
  --file-id <file_id> \
  --approver-a <ops_security_user_id> \
  --approver-b <compliance_user_id>
```

Restoration emits audit-chain seal documenting the restore as a recovery action, NOT a normal write.

## Verification

```bash
# Immutability-correctness SLO at 100%
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo immutability-tier-correctness

# Integrity-scan delta = 0
cargo run -p oya-dev-cli -- vcs query --microservice drive --query "immutability-integrity-scan-delta-total" # expect 0

# Audit-chain seal for restoration present
cargo run -p oya-dev-cli -- vcs query --microservice drive --audit-event worm-violation-restored --file-id <file_id>
```

## Post-incident

- Sev-1 post-mortem within 5 business days.
- Cause analysis: which layer bypassed, why.
- Regression test in `tests/worm-refuses-purge.rs` + multiple layers.
- LEAN-check additions: `oya-check-worm-enforcement-multi-layer`.
- Tenant compliance officer comms with concrete remediation chain.
- Regulatory comms per pack timeline.
- Public status page update.

## References

- ADR-DRIVE-0006 — immutability + WORM policy.
- `slos/immutability-tier-correctness.openslo.yaml`.
- `incident-response.md` IR-2.
- `threat-model.md` T-T-03 + T-E-04.
- SEC 17a-4(f); FINRA Rule 4511; HIPAA §164.316; GDPR Art. 33; KR PIPA Art. 34; APPI Art. 22.
- AWS S3 Object Lock compliance mode reference.
- Garage object-lock reference.
- SeaweedFS object-lock reference.
