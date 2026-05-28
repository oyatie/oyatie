---
doc_class: Onboarding
microservice: audit-chain
persona: Diana-as-auditor / Dimitri-as-auditor-for-client-A / Hyo-Jin-as-auditor-A / Jakub-as-IT-auditor
related_adrs: [ADR-0028, ADR-0003, ADR-0263, ADR-0296, ADR-0251, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# Compliance Officer onboarding — first 5 working days on `audit-chain`

Audience: a compliance officer, auditor, audit-platform engineer, or security-substrate engineer joining the `audit-chain` rotation. By Day-5 they will have: stood up a demo_trial chain instance, emitted + sealed real events, exercised the cross-µservice emission adapter, replayed a Merkle verification proof, run a regulator-evidence export, and walked the HSM key-rotation / Merkle recovery runbooks.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the inheritance from Bominal ADR-0028 + ADR-0003 (Merkle + Ed25519; emission contract).
2. Read `ARCHITECTURE.md` § per-tenant-isolation + § seal-pipeline + § Merkle-tree-shape (∼ 60 min).
3. Open the Grafana folder `audit-chain`. Identify the published boards — `dashboards/emission-rate.json`, `dashboards/seal-latency.json`, and `dashboards/verification-failure-rate.json`.
4. Walk the on-call runbooks: `audit-chain-restart.md`, `audit-export.md`, `chain-replay-from-snapshot-protocol.md`, `hsm-key-rotation.md`, `merkle-root-discrepancy-investigation.md`, `merkle-seal-recovery.md`, `regulator-evidence-export-failure.md`, `retention-cascade.md`, and `signature-verification-failure.md`.
5. Sit in on the Wednesday audit-substrate handoff. Watch how the outgoing rotation walks the past-week chain-coverage ledger (every µservice's emission-rate, gap count, late-arrival count) and hands the pager.

Acceptance: you can sketch on a whiteboard the seal path: emitter µservice → Pulsar `audit.event.<class>` → ingest pod → batch buffer → Merkle builder → HSM signer → PostgreSQL chain-head write + SeaweedFS-S3 batch tarball.

## Day 2 — demo_trial audit-chain cell bootstrap

```sh
cargo run -p oya-dev-cli -- audit-chain bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --pulsar-endpoint pulsar://drill-pulsar-syd-1:6650 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/audit_chain \
    --seaweed-s3-endpoint http://drill-seaweed-syd-1:8333 \
    --signing-key-mode sealed-secret \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 15 min. Watch the bootstrap log for the phases (in order):

1. Pulsar topic family `audit.event.*` created (one topic per event class, partitioned by `tenant_id % 32`).
2. PostgreSQL schema applied (`event`, `chain_head`, `merkle_batch`, `seal_certificate` tables — see `iac/postgres/schema.sql`).
3. SeaweedFS-S3 bucket `audit-chain-batches-drill-syd-1` created with object-lock OFF (demo_trial tenant_class).
4. Ed25519 signing keypair generated; sealed-secret applied to namespace.
5. Sealer worker pods scheduled (`audit-chain-sealer-1`, `audit-chain-sealer-2`).
6. First sealed batch produced (empty batch with `chain_head_seq = 0`).

After bootstrap, verify:

```sh
kubectl -n audit-chain get pods
# Expected: audit-chain-ingest-{0,1,2}, audit-chain-sealer-{0,1}, audit-chain-verifier-0

cargo run -p oya-dev-cli -- audit-chain head --cell drill-syd-1
# Expected output:
# chain_head_seq=0 root_hash=sha256:c2c7d553b16112a279535b2f012840cf21ed851372a7bef12070e16e40365619 sealed_at=<bootstrap timestamp> signature_ed25519=...
```

Acceptance: chain is live, you can describe the role of each pod and explain why the empty seal at `seq=0` matters (it anchors the chain genesis).

## Day 3 — Emit real events + verify a Merkle proof

Emit synthetic events from a tenant-emulator:

```sh
oya synthetic audit-emit \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --event-class workflow.step.completed \
    --principal-id u-12345 \
    --count 10000 \
    --rate 500-per-sec
```

The emitter publishes 10 000 events at 500/sec over ~ 20 s. After the next 1-second seal period, verify the chain:

```sh
oya audit query --cell drill-syd-1 --tenant drill-acme --since 5m
# Returns the 10 000 events, each with event_id, prev_hash, signature, sealed_in_batch_id.

oya audit verify-chain --cell drill-syd-1 --tenant drill-acme --since 5m
# Output:
#   chain verified: 10000 events
#   batches inspected: 4 (2500 events each)
#   Merkle roots: 4 (all matched per-batch signatures)
#   signature gaps: 0
#   prev_hash discontinuities: 0
#   verification time: 312 ms
```

Inspect a single Merkle proof:

```sh
oya audit prove --cell drill-syd-1 --event-id <one_event_id> --output proof.json
cat proof.json | jq '.'
```

The proof has shape:

```json
{
  "event_id": "01HZ...",
  "merkle_path": [
    {"position": "left",  "hash": "sha256:abc123..."},
    {"position": "right", "hash": "sha256:def456..."},
    ...
  ],
  "batch_root": "sha256:fedcba...",
  "batch_id": "ulid:01HZ...",
  "signature_ed25519": "ed25519:...",
  "signing_key_id": "audit-chain-demo_trial-drill-syd-1-2026-05-20"
}
```

The proof is independently verifiable against the public signing key (advertised on the chain's `/keys` endpoint).

Acceptance: 10 000 events emitted, chain verified, single-event proof generated and externally verified via `oya audit external-verify proof.json --pubkey <key>`.

## Day 4 — Cross-µservice emission + dual-tenant seal walk

Cross-µservice emission: every other oyatie µservice emits to audit-chain via the `oya-audit-emission-adapter` kernel. Walk the path with the `workflow-engine` integration:

```sh
oya synthetic workflow-engine emit \
    --tenant drill-acme \
    --workflow-id wf-001 \
    --step-completion-count 50
```

This triggers `workflow-engine` to emit 50 `workflow.step.completed` events to its local emit-adapter, which forwards to audit-chain. Verify:

```sh
oya audit query --cell drill-syd-1 --tenant drill-acme \
    --event-class workflow.step.completed \
    --workflow-id wf-001 \
    --since 1m
# Expected: 50 events, all with consistent prev_hash chain.
```

Now exercise the dual-tenant boundary. Per IP-journey-j101 (dual-seal events), some events are emitted with two tenant IDs (e.g., a marketplace transaction has `buyer_tenant` and `seller_tenant`). Verify the dual-seal:

```sh
oya synthetic audit-emit \
    --cell drill-syd-1 \
    --tenant-pair drill-acme,drill-vendor \
    --event-class marketplace.transaction.settled \
    --count 1 \
    --dual-seal
```

The single event lands in BOTH tenants' query views without duplication of the underlying Merkle leaf (the leaf is shared; the query view projects per-tenant by indexed tenant-ID set).

```sh
oya audit query --tenant drill-acme  --event-class marketplace.transaction.settled --since 1m
oya audit query --tenant drill-vendor --event-class marketplace.transaction.settled --since 1m
# Both queries return the same event_id but with respective tenant_id projection.
```

Acceptance: cross-µservice emit flows; dual-tenant seal projects correctly to both tenants without re-sealing the underlying leaf.

## Day 5 — Sealer restart drill + regulator-evidence export

Read `runbooks/hsm-key-rotation.md`, `runbooks/merkle-seal-recovery.md`, and `runbooks/audit-chain-restart.md` end-to-end (the demo_trial tenant_class uses sealed-secret so the drill is a sealer-restart drill; promote to paid before drilling HSM key custody proper).

Run the sealer-restart drill:

```sh
oya audit drill sealer-restart --cell drill-syd-1 --duration 5m
```

The drill stops the active sealer leader, observes the failover (the standby takes the lease in ≤ 30 s), continues emission throughout, then re-attaches the original sealer. Verify the chain has no signature gap:

```sh
oya audit verify-chain --cell drill-syd-1 --since 10m
# Expected: 0 signature gaps even across the failover window.
```

Now run a regulator-evidence export. The export gathers a tenant's full chain (with Merkle proofs + signing-key history) into a tarball that an external auditor can verify without oyatie's cooperation:

```sh
oya audit regulator-export \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --since 2026-05-13T00:00:00Z \
    --until 2026-05-20T23:59:59Z \
    --regulator-class SOC2 \
    --output ./drill-acme-evidence-2026-05.tar.gz
```

The tarball contains:

- `events.jsonl` — every event with `event_id, tenant_id, event_class, principal_id, payload_hash, prev_hash, signature, sealed_in_batch_id`.
- `merkle_batches/` — per-batch root hashes + signatures + Merkle paths.
- `signing_keys.jsonl` — every signing-key public component active during the window + key-rotation timestamps.
- `verification.sh` — a stand-alone Bash script that re-verifies the entire chain using only `openssl`, `sha256sum`, and `jq` (no oyatie tooling required).

Test the standalone verification:

```sh
tar -xzf ./drill-acme-evidence-2026-05.tar.gz -C /tmp/auditor-replay/
cd /tmp/auditor-replay && ./verification.sh
# Expected: PASS — chain verified end-to-end without oyatie tooling.
```

Target end-to-end recovery for sealer-restart drill: ≤ 5 min (production target ≤ 10 min per `slos/sealer-failover.openslo.yaml`).

Acceptance: drill executed without chain gap; regulator-export verified by the standalone script.

## What you've learned

- The demo_trial bootstrap profile end-to-end + the sealer/ingest/verifier pod topology.
- The Merkle proof shape + standalone verification path (no oyatie tooling required).
- The cross-µservice emission adapter + dual-tenant seal projection.
- The sealer-restart drill (the most-likely page on demo_trial) + the regulator-evidence export shape.

Next week: paid tenant_class promotion drill (HSM enrollment + shadow-sign verification), paid tenant_class multi-region restore drill, paid tenant_class dual-control quorum tabletop, and your first production shadow rotation.
