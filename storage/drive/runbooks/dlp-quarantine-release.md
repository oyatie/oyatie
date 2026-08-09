---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-DLP-QUARANTINE-RELEASE
severity_class: sev-3
related_adrs: [ADR-DRIVE-0005]
related_slos: [dlp-scan-correctness]
owner_team: axis-drive + ops-security + council-privacy
date: 2026-05-17
doc_status: published
---

# Runbook: DLP quarantine release

## Symptom

A file in DLP-quarantine state needs review and either release or permanent block. Triggered by:
- Tenant ticket: "DLP blocked my legitimate file from being shared".
- Per-tenant DLP-flag rate > 5% sustained 24h (likely rule misconfig).
- `oya_drive_dlp_quarantine_pending_review_total` rising.

## Severity

**Sev-3** for single-tenant single-file. **Sev-2** for cross-tenant rule-misconfig storm or DLP-correctness regression.

## First responder

axis-drive on-call. Escalate to council-privacy for content-review decisions.

## Diagnosis

### Step 1 — List quarantined files

```bash
psql "$DRIVE_PG" -c \
  "SELECT file_id, scan_job_id, dlp_rules_matched, flagged_at, flagged_by_rule
   FROM oya_drive_dlp_quarantine
   WHERE tenant_id = '<tenant_id>' AND released_at IS NULL
   ORDER BY flagged_at DESC LIMIT 50;"
```

### Step 2 — Review the matched rule + file content

```bash
# Get rule details
cargo run -p oya-dev-cli -- vcs query --microservice drive \
  --query "dlp_rule_definition --rule-id <rule_id>"

# Get file metadata (NEVER fetch bytes outside the review tool — content review happens
# inside the gVisor sandbox via the auditor-portal review surface)
cargo run -p oya-dev-cli -- vcs query --microservice drive \
  --query "file_metadata --file-id <file_id>"
```

### Step 3 — Decide

Three outcomes per skill `dlp-quarantine-release.md`:

- **Release** — file is legitimate; rule is over-broad or wrong. Release the file + tune the rule.
- **Block** — file is correctly flagged; keep in quarantine; tenant security takes action.
- **Defer** — need more context; tenant policy review.

## Mitigation

### Outcome A — Release single file

1. Two-person rule: ops-security on-call + tenant policy officer co-approve.
2. Release:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive dlp-release \
     --file-id <file_id> \
     --approver-a <ops_user_id> \
     --approver-b <tenant_policy_user_id> \
     --reason "<reason>"
   ```
3. Audit-chain seal emitted.

### Outcome B — Block + add tenant policy entry

1. Mark file as blocked:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive dlp-block-confirmed \
     --file-id <file_id> \
     --tenant-policy-followup-required true
   ```
2. Notify tenant security via tenant-portal ticket.

### Outcome C — Rule misconfig (mass release)

1. Sev-2. Engage incident-response.md.
2. Tune rule:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive dlp-rule-update \
     --rule-id <rule_id> \
     --action disable-or-tune
   ```
3. Re-scan all files flagged by the misconfigured rule:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive dlp-rescan \
     --rule-id <rule_id> \
     --tenant <tenant_id>
   ```
4. Auto-release for files that no longer match.

## Verification

```bash
# Quarantine pending count back to baseline
cargo run -p oya-dev-cli -- vcs query --microservice drive --metric dlp_quarantine_pending_review_total

# DLP-correctness SLO recovering
cargo run -p oya-dev-cli -- gate validate slo --microservice drive --slo dlp-scan-correctness
```

## Post-incident

- Per-tenant communication on release/block decision.
- Tune DLP rules per learnings.
- LEAN-check addition if mitigated by code change.

## References

- ADR-DRIVE-0005 — preview pipeline sandboxing (DLP review surface).
- `slos/dlp-scan-correctness.openslo.yaml`.
- `incident-response.md` IR-6.
- `policy/tenant-scope.cedar` (DLP-flagged share-link forbid clause).
- foundry-runtime DLP rule reference.
