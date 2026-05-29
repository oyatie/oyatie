---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-VIRUS-SCAN-ROLLBACK
severity_class: sev-2
related_adrs: [ADR-DRIVE-0005]
related_slos: [dlp-scan-correctness]
owner_team: axis-drive + ops-security
date: 2026-05-17
doc_status: published
---

# Runbook: Virus-scan rollback (ClamAV / OPSWAT signature update regression)

## Symptom

A ClamAV or OPSWAT signature update regresses scan accuracy:
- False-positive flood: large number of legitimate files flagged after signature update.
- False-negative observed: known-malicious sample passes scan after signature update.
- `oya_drive_virus_scan_verdict_total{verdict="malicious"}` step-function rise post-signature-update.

## Severity

**Sev-2** for false-positive flood. **Sev-1** for confirmed false-negative on known-malicious sample reaching durable bucket.

## First responder

ops-security on-call. Engage axis-drive on-call within 15 min.

## Diagnosis

### Step 1 — Confirm signature update is the trigger

```bash
# Verify scan-engine version
kubectl -n drive exec deploy/oya-drive-dlp-virus-scan-worker -- clamscan --version
# (or for OPSWAT) — opswat-mdctl status

# Verdict-rate trend
kubectl -n drive exec deploy/oya-drive-dlp-virus-scan-worker -- \
  curl -s localhost:9090/metrics |
  grep oya_drive_virus_scan_verdict_total
```

### Step 2 — Identify recently-updated signature

```bash
# ClamAV signature DB version
kubectl -n drive exec deploy/oya-drive-dlp-virus-scan-worker -- \
  sigtool --info /var/lib/clamav/main.cvd

# OPSWAT engine bundle version
opswat-mdctl info --engines
```

### Step 3 — Reproduce on a representative sample

```bash
# Pull a flagged file (read-only audit-portal review surface)
cargo run -p oya-dev-cli -- vcs admin drive virus-quarantine-sample \
  --tenant <tenant_id> \
  --recent-flagged 5
```

## Mitigation

### Case A — False-positive flood (signature update over-broad)

1. Pin ClamAV / OPSWAT signature to prior known-good version:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive scan-signature-pin \
     --engine clamav \
     --version <prior-known-good>
   ```
2. Re-scan all files flagged by the over-broad signature:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive dlp-rescan \
     --signature-version <new-version> \
     --auto-release-if-clean
   ```
3. Notify upstream signature provider; track CVE if applicable.
4. Tenant comms via banner if user-facing impact: "Files flagged in the last N hours have been re-scanned and released."

### Case B — Confirmed false-negative (malicious sample passed scan)

1. **Sev-1**. Engage incident-response.md.
2. Quarantine the malicious file:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive quarantine \
     --file-id <file_id> \
     --reason "post-deploy false-negative; engine bypass"
   ```
3. Fan-out scan across all objects ingested since the false-negative window:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive fan-out-rescan \
     --since <bypass_window_start>
   ```
4. Track downloads of the malicious file via audit-chain replay; notify any users who downloaded.
5. Engage OPSWAT multi-engine secondary verdict for affected packs.

### Case C — OPSWAT outage (pack-us-healthcare + pack-eu)

1. Fall back to ClamAV-only verdict.
2. Files in pack-us-healthcare flagged for "single-engine scan only — pending OPSWAT re-scan when service returns".
3. Restart OPSWAT pods + verify connectivity:
   ```bash
   kubectl -n drive rollout restart deploy/oya-drive-dlp-virus-scan-adapter-opswat
   ```

## Verification

```bash
# Scan-verdict distribution back to baseline
kubectl -n drive exec deploy/oya-drive-dlp-virus-scan-worker -- \
  curl -s localhost:9090/metrics | grep oya_drive_virus_scan_verdict_total

# DLP-correctness SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo dlp-scan-correctness

# EICAR test signature still triggers (sanity)
echo 'X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*' |
  curl -F file=@- http://oya-drive-upload-rest.drive.svc.cluster.local/v1/files
# expect: 422 + "malicious"
```

## Post-incident

- Per-tenant comms on impact.
- Tune signature-pinning policy + auto-rollback rules.
- LEAN-check addition: `oya-check-virus-scan-signature-pinned` (enforce pin policy).

## References

- ADR-DRIVE-0005 — preview + scan sandboxing.
- `slos/dlp-scan-correctness.openslo.yaml`.
- `incident-response.md` IR-3.
- ClamAV operator guide.
- OPSWAT MetaDefender operator guide.
- EICAR test signature.
