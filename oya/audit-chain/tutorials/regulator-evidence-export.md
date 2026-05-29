---
doc_class: Tutorial
microservice: audit-chain
persona: compliance-officer + audit-platform-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Generate a SOC 2 + SEC 17a-4(f) regulator-evidence bundle for a tenant

You will: configure a regulator-export run, generate a self-contained verification bundle for a 90-day window, hand the bundle to an external auditor (simulated), have the auditor verify the chain WITHOUT oyatie tooling, and file the export evidence to the `oya-governance-evidence` lane. Total time ≤ 45 minutes.

## Pre-requisites

- A tenant cell on paid tenant_class (`tenant_class policy`).
- `oya-dev-cli` ≥ 1.42.0.
- A tenant with ≥ 30 days of emission history.
- Compliance officer Cedar principal in the tenant's `compliance_admin` role.
- An external auditor email address (for the bundle handoff).

## Step 1 — Identify the evidence window and event classes (≤ 5 min)

For SOC 2 Type II + SEC 17a-4(f), regulators typically request:

- All `workflow.*` events for the audit period.
- All `iam.*` events (role assignments, principal mutations).
- All `data_subject_request.*` events (GDPR / KR PIPA / CCPA DSARs).
- All `policy.*` events (Cedar permit changes).
- All `payments.*` events (for fin-relevant tenants).

List the events for one window:

```sh
oya audit query \
    --cell prod-syd-1 \
    --tenant acme-corp \
    --since 2026-02-20T00:00:00Z \
    --until 2026-05-20T23:59:59Z \
    --event-class-prefix workflow,iam,data_subject_request,policy,payments \
    --count-only
```

Expected output:

```
Event class breakdown (2026-02-20 .. 2026-05-20):
  workflow.step.started                812 432
  workflow.step.completed              812 401
  workflow.step.failed                     31
  iam.role.assigned                    14 218
  iam.principal.created                 1 824
  iam.principal.deactivated               412
  data_subject_request.received           18
  data_subject_request.fulfilled          17
  data_subject_request.rejected            1
  policy.permit.updated                    8
  payments.invoice.issued               4 218
  payments.invoice.settled              3 992
  payments.refund.executed                 87
Total: 1 649 959 events
```

Sanity-check this against the tenant's own ledger (e.g., your billing system should report ~ 4 200 invoices for the period; payment-engine `payments.invoice.issued` count should match within 0.1 %).

## Step 2 — Generate the regulator-export bundle (≤ 15 min)

```sh
oya audit regulator-export \
    --cell prod-syd-1 \
    --tenant acme-corp \
    --since 2026-02-20T00:00:00Z \
    --until 2026-05-20T23:59:59Z \
    --event-class-prefix workflow,iam,data_subject_request,policy,payments \
    --regulator-class SOC2-TypeII,SEC-17a-4f \
    --include-merkle-proofs \
    --include-signing-key-history \
    --output ./acme-evidence-2026q1.tar.gz
```

Expected runtime: ~ 12 min for 1.6 M events (compute is gated by HSM throughput for signature batch re-verification + S3 fetch for any cold-tier batches).

The bundle structure:

```
acme-evidence-2026q1.tar.gz
├── README.md                             # Auditor instructions
├── manifest.json                         # Event counts + checksums + verification metadata
├── events/
│   ├── events.jsonl                      # One JSON object per event (sorted by chain_seq)
│   └── events.jsonl.sha256               # SHA-256 hash of events.jsonl
├── merkle_batches/
│   ├── batch-000001.json                 # Each batch with root, leaves, signature
│   ├── batch-000002.json
│   └── ...
├── signing_keys/
│   ├── signing_keys.jsonl                # Every signing-key public component active in window
│   └── key_rotation_events.jsonl         # Key rotation events with prev-key signature attestation
├── proofs/
│   ├── proof-by-event-id/                # Optional: one proof per event_id (large; opt-in via --include-merkle-proofs)
│   └── ...
└── verification.sh                       # Standalone re-verification script
```

The `manifest.json` shape:

```json
{
  "bundle_version": "audit-chain-export-v3",
  "cell": "prod-syd-1",
  "tenant": "acme-corp",
  "window": {
    "since": "2026-02-20T00:00:00Z",
    "until": "2026-05-20T23:59:59Z"
  },
  "regulator_class": ["SOC2-TypeII", "SEC-17a-4f"],
  "event_count": 1649959,
  "batch_count": 6420,
  "signing_key_count": 4,
  "events_jsonl_sha256": "sha256:abc123def456...",
  "bundle_sha256": "sha256:fedcba987654...",
  "generated_at": "2026-05-20T14:32:17Z",
  "generated_by": "u-compliance-jane@acme-corp",
  "export_event_id": "01HZX2K3...",
  "cedar_decision": "allow",
  "cedar_policy_id": "audit-chain-regulator-export-v1"
}
```

The export itself emits an audit event (`audit_chain.regulator_export.emitted`) — exports are traceable.

## Step 3 — Verify the bundle locally before handoff (≤ 5 min)

```sh
mkdir -p /tmp/auditor-replay && tar -xzf ./acme-evidence-2026q1.tar.gz -C /tmp/auditor-replay/
cd /tmp/auditor-replay
./verification.sh
```

Expected output:

```
[1/6] Bundle checksum verified                                        ... OK
[2/6] Events JSONL checksum verified                                  ... OK (1 649 959 events)
[3/6] Per-batch Merkle root recomputation                             ... OK (6 420 batches)
[4/6] Per-batch Ed25519 signature verification                        ... OK (6 420 signatures, 4 signing keys)
[5/6] Chain prev_hash continuity                                      ... OK (no gaps)
[6/6] Signing-key history continuity                                  ... OK (4 keys, 3 rotation events)

PASS: chain verified end-to-end.
Verification took 47.2 s (using openssl, sha256sum, jq; no oyatie tooling).
```

The `verification.sh` script depends ONLY on:

- `bash` ≥ 4.0
- `sha256sum` (or `shasum -a 256` on macOS)
- `openssl` ≥ 3.0 (for Ed25519 verification)
- `jq` ≥ 1.6
- `sha256sum`, `gzip`, `tar` (standard Unix)

The script does NOT require Rust, oyatie CLI, network access, or any pinned oyatie versions.

## Step 4 — Sign and hand off to the external auditor (≤ 10 min)

Sign the bundle with your compliance-admin GPG key for tamper-evidence in transit:

```sh
gpg --detach-sign --armor --output acme-evidence-2026q1.tar.gz.asc acme-evidence-2026q1.tar.gz
```

Upload to the auditor's secure handoff portal (most audit firms use one of: Kiteworks, ShareFile, Box for Compliance, AuditBoard secure exchange). Or, for self-hosted auditors, dispatch via SFTP with the bundle hash communicated out-of-band.

The auditor will:

1. Verify the GPG signature against your published key.
2. Run `./verification.sh` from inside the extracted bundle.
3. Inspect specific events of interest (e.g., for SOC 2 CC6.1 they'll sample N=25 `iam.role.assigned` events and trace each to the source-of-truth in IAM ledgers).

## Step 5 — File the export evidence to the governance lane (≤ 5 min)

```sh
oya governance file-evidence \
    --lane oya-governance-evidence \
    --evidence-class regulator-export \
    --tenant acme-corp \
    --regulator SOC2-TypeII,SEC-17a-4f \
    --bundle-hash sha256:fedcba987654... \
    --auditor-organization "Big4Audit LLP" \
    --auditor-engagement "ACME-CORP-2026-Q1" \
    --export-event-id 01HZX2K3...
```

This emits:

- `oya-governance-evidence` lane row tying the export to the regulator + auditor + bundle hash.
- A second `audit_chain.evidence_filed` event into the chain itself (recursive non-repudiation).

The evidence row will surface in next quarter's audit prep dashboard automatically.

## Step 6 — Audit-chain verification of the export itself (≤ 5 min)

The export operation emitted `audit_chain.regulator_export.emitted`. Verify it landed:

```sh
oya audit query --cell prod-syd-1 --tenant acme-corp \
    --event-class audit_chain.regulator_export.emitted \
    --since 1h
```

Expected: 1 event with bundle hash, regulator class, auditor, principal who exported.

This event is itself part of next quarter's evidence bundle — so the export chain is recursive (auditors verifying Q2 will see the Q1 export evidence inside the Q2 chain).

## What you've learned

- The regulator-export shape + the standalone verification path.
- The handoff envelope (GPG signature + secure portal) + auditor expectations.
- The governance-evidence filing path + the recursive nature of the audit chain.

Next tutorial: `tutorials/cross-tenant-audit-investigation.md` — investigating a Cedar-permit misuse across tenants while preserving cross-tenant isolation invariants.
