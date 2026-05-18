---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: drive
runbook_id: RB-DRIVE-SHARE-LINK-TAKEOVER
severity_class: sev-1
related_adrs: [ADR-DRIVE-0003]
related_slos: [share-link-generation-latency]
owner_team: axis-drive + ops-security
date: 2026-05-17
doc_status: published
---

# Runbook: Share-link takeover incident

## Symptom

One or more of:
- `oya_drive_share_link_verify_total{verdict="signature_invalid"}` rises > 100× baseline.
- `oya_drive_share_link_enumeration_pattern_detected_total` non-zero.
- Audit-chain forensic replay shows access pattern consistent with mass enumeration or signing-key compromise.
- Tenant report of accessed links the tenant did not mint.

## Severity

**Sev-1**. Share-link signing-key compromise affects the tenant's entire link space.

## First responder

ops-security on-call. Engage axis-drive on-call within 15 min. Engage council-architecture if cross-tenant scope.

## Diagnosis

### Step 1 — Determine scope

```bash
# Count of failed verifications by IP last 1h
kubectl -n drive logs deploy/oya-drive-share-link-rest --since=1h |
  grep verify_signature_invalid |
  jq -r '.client_ip_hash' | sort | uniq -c | sort -rn | head -20

# Count of successful accesses with unusual IP patterns last 1h
kubectl -n drive logs deploy/oya-drive-share-link-rest --since=1h |
  grep verify_signature_valid |
  jq -r '.client_ip_hash' | sort | uniq -c | sort -rn | head -20
```

### Step 2 — Confirm signing-key compromise vs enumeration

- If `signature_invalid` count >> `signature_valid` → enumeration (Case A).
- If `signature_valid` accesses appear from anomalous IPs / regions → key compromise suspected (Case B).
- If accesses use links that were never minted (per audit-chain replay) → signing-key compromise confirmed (Case C).

## Mitigation

### Case A — Enumeration (no key compromise yet)

1. Engage WAF + per-IP rate limit aggressive throttle (1 req/sec/IP).
2. Block source ASN if patterns concentrate.
3. Monitor; if pattern subsides → return to normal posture.

### Case B — Suspected key compromise

1. Rotate the per-tenant share-link signing key immediately:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive rotate-share-link-signing-key \
     --tenant <tenant_id> \
     --reason "suspected-compromise" \
     --emergency
   ```
2. Revoke all extant share-links for tenant:
   ```bash
   cargo run -p oya-dev-cli -- vcs admin drive revoke-all-share-links \
     --tenant <tenant_id> \
     --reason "key-rotation-cascade"
   ```
3. Tenant comms within 1h: "Share links revoked due to security incident; please re-mint."
4. Forensic audit-chain replay to determine what was accessed during the suspect window.

### Case C — Confirmed key compromise (signed links never minted by tenant)

1. All Case B steps.
2. Sev-1 ICR; engage council-architecture + ops-security incident commander.
3. Per GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.404 / APPI Art. 22 — engage breach-notification chain (see `incident-response.md`).
4. Forensic deep-dive: source of key compromise (OpenBao Transit log, K8s SA leak, supply-chain).
5. Post-mortem within 5 business days.

## Verification

```bash
# Failed-verify rate back to baseline
kubectl -n drive exec deploy/oya-drive-share-link-rest -- \
  curl -s localhost:9090/metrics |
  grep oya_drive_share_link_verify_total

# Old key revoked; no extant links signed by old key resolve
cargo nextest run -p oya-drive-share-link-domain -- old_key_rejected

# Audit-chain seal for rotation + revocation present
cargo run -p oya-dev-cli -- vcs query --microservice drive --audit-event share-link-signing-key-rotated --tenant <tenant_id>
```

## Post-incident

- Per-pack notification within timelines (GDPR 72h / KR PIPA 72h / HIPAA 60d / APPI 3 business days).
- Public status page update for Sev-1.
- Post-mortem at `evidence/postmortem/<incident_id>.md`.
- Update this runbook if signature was new.
- LEAN-check addition if mitigated by code change.

## References

- ADR-DRIVE-0003 — share-link security model (Ed25519 + HKDF + Argon2id).
- `slos/share-link-generation-latency.openslo.yaml`.
- `policy/public-read.cedar`.
- `incident-response.md`.
- `threat-model.md` T-S-02 + T-I-01 + T-D-02.
- GDPR Art. 33; KR PIPA Art. 34; HIPAA 45 CFR §164.404; APPI Art. 22.
- OpenBao Transit operator guide.
