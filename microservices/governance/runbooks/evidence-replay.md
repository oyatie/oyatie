---
doc_class: Runbook
title: Evidence Replay + Auditor Export
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry
severity_default: Sev-3 (operational; Sev-2 if audit-chain gap detected)
related_failure_modes: [F-03]
related_artifacts:
  - microservices/governance/failure-modes.md
  - microservices/governance/policy/auditor-scope.cedar
review_cadence: quarterly + on every external audit cycle
doc_status: published
---

# Runbook: Evidence Replay + Auditor Export

## When to invoke

- External auditor (SOC 2 / ISO 27001 / SLSA / GDPR DPA) requests evidence within a JIT-scoped audit window.
- Internal SME requests Finding + evidence trail for postmortem / RCA.
- Audit-chain seal verification fails for one or more Findings (F-03).
- Tenant requests own µservice's Finding history (DPIA §3 Right of Access).

## Pre-flight

- You are: ops-security on-call OR axis-foundry on-call.
- You have: `cargo run -p oya-dev-cli` workspace ready; Postgres + S3 + audit-chain credentials in OpenBao; valid JIT token (for auditor requests).
- You verified: requester identity + scope + window.

## Decision tree

```text
                Who is requesting?
                  ├─ External auditor → §A (JIT export)
                  ├─ Internal SME (postmortem) → §B (replay-query)
                  ├─ Tenant operator (own µservice) → §C (tenant self-service)
                  └─ Audit-chain gap detected (F-03) → §D (seal reconciliation)
```

## §A — External auditor JIT export

### Pre-conditions

1. Auditor identity + scope window negotiated in advance.
2. DPA addendum signed (per `compliance.md` Cross-Border Transfer Mechanisms).
3. JIT OIDC token issued via OpenBao: `vault write auth/oidc/role/external-auditor-<auditor-id> ttl=1h scope_microservices=<list> audit_window_start=<unix> audit_window_end=<unix>`.
4. Auditor acknowledges JIT scope acceptance (per `policy/auditor-scope.cedar` P5).

### Steps

1. **Verify scope**:
   ```bash
   cargo run -p oya-dev-cli -- governance auditor-scope verify \
     --auditor-id <id> --window-start <unix> --window-end <unix>
   ```
   Expect: scope_microservices ⊆ auditor's authorized scope.

2. **Trigger export**:
   ```bash
   cargo run -p oya-dev-cli -- governance evidence export \
     --auditor-id <id> \
     --window-start <unix> --window-end <unix> \
     --microservices <list> \
     --output /tmp/audit-export-<id>.tar.zst
   ```

3. **Verify bundle integrity**:
   ```bash
   cargo run -p oya-dev-cli -- governance evidence verify-bundle \
     --path /tmp/audit-export-<id>.tar.zst
   ```
   Expect: all Findings carry valid Ed25519 signatures + audit-chain seal verification passes.

4. **Deliver** via secure channel (per DPA):
   - encrypted file transfer (per-auditor PGP key, in OpenBao);
   - or per-audit S3 prefix with auditor-scoped IAM role.

5. **Log delivery** in `evidence/audits/external-auditor-deliveries/<audit-window-id>.md` with:
   - Auditor identity + scope.
   - Window start + end.
   - Bundle SHA256.
   - Delivery timestamp + channel.
   - Counter-signature from auditor on receipt.

### Bundle format

Canonical structure (per `contracts/openapi/governance.yaml` schema `EvidenceBundle`):

```
audit-export-<id>.tar.zst
├── manifest.json                   # bundle manifest (signed; audit-window metadata; Merkle root)
├── findings/                       # one JSON per Finding
│   ├── <finding-id>.json
│   └── ...
├── lane-runs/                      # one JSON per lane run
│   ├── <lane-run-id>.json
│   └── ...
├── evidence/                       # raw lane output blobs
│   ├── <evidence-blob-sha>.bin
│   └── ...
├── audit-chain-seals/              # Ed25519 seal records
│   ├── <seal-id>.json
│   └── ...
└── attestations/
    ├── slsa-provenance.json
    └── soc2-cc7-4-evidence.json
```

### Size limits

- Per-export size cap: 5 GB (per `auditor-scope.cedar` F6).
- Larger requests split into multiple windows.

## §B — Internal SME replay-query (postmortem / RCA)

1. **Query** Findings for a date range + µservice:
   ```bash
   cargo run -p oya-dev-cli -- governance finding-query \
     --microservice <ms> --window-start <unix> --window-end <unix> \
     --output /tmp/findings-<id>.jsonl
   ```

2. **Fetch** evidence blob for a specific Finding:
   ```bash
   cargo run -p oya-dev-cli -- governance evidence get \
     --finding-id <id> --output /tmp/evidence-<id>.bin
   ```

3. **Verify** signature + seal:
   ```bash
   cargo run -p oya-dev-cli -- governance evidence verify \
     --finding-id <id>
   ```

4. **Document** the SME query in the postmortem at `evidence/audits/postmortems/<incident-id>.md`.

## §C — Tenant operator self-service

1. **Tenant** logs into Application Shell.
2. **Tenant** navigates to "My µservice → Findings → Export".
3. **Shell** calls REST `/findings?microservice=<ms>&author=<self-subject>` per `contracts/openapi/governance.yaml`.
4. **Cedar** policy `tenant-scope.cedar` enforces P1 + P2 + P3 (read own µservice's Findings only).
5. **Tenant** receives JSON or CSV export.

## §D — Audit-chain seal reconciliation (F-03)

(Sev-2)

1. **Confirm**: `cargo run -p oya-dev-cli -- governance audit-chain status` → expect `unsealed_findings_age_seconds_p99 > 300`.
2. **Identify** unsealed Findings: `cargo run -p oya-dev-cli -- governance audit-chain unsealed --since <unix>`.
3. **Check** audit-chain µservice health: `kubectl logs -n audit-chain audit-chain-app | tail -100`.
4. **If audit-chain healthy** → reconciler should drain naturally; wait 5 min; re-check.
5. **If audit-chain unhealthy** → engage `microservices/audit-chain/runbooks/audit-chain-outage.md`; OnCall page if not already.
6. **Once audit-chain recovers** → trigger reconciler back-fill:
   ```bash
   cargo run -p oya-dev-cli -- governance audit-chain reconcile --since <unix>
   ```
7. **Verify** zero unsealed Findings remain: `cargo run -p oya-dev-cli -- governance audit-chain status`.
8. **Post-incident**: RCA; consider increasing reconciler concurrency if reconciliation lag became significant.

## Stand-down criteria

- Auditor confirms receipt + verification (for §A).
- All Findings in the requested window have been delivered with valid signatures (for §A/B).
- Tenant receives export confirmation (for §C).
- Audit-chain reconciler reports zero unsealed Findings older than 5 min (for §D).

## Post-action

- Update `compliance.md` ROPA if new transfer occurred.
- Log delivery in `evidence/audits/external-auditor-deliveries/` (for §A).
- Update postmortem if applicable.

## References

- `microservices/governance/failure-modes.md` F-03.
- `microservices/governance/policy/auditor-scope.cedar`.
- `microservices/governance/dpia.md` §3 Right of Access.
- `microservices/governance/compliance.md` ROPA.
- `microservices/audit-chain/PRD.md` (upstream µservice).
- SOC 2 CC7.4 evidence requirements.
