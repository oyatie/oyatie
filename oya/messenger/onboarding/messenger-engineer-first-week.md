---
doc_class: Onboarding
microservice: messenger
persona: messenger-engineer + e2ee-platform-engineer + trust-and-safety-engineer
related_adrs: [ADR-MSG-001, ADR-MSGR-0001, ADR-MSGR-0002, ADR-MSGR-0003, ADR-MSGR-0004, ADR-0316, ADR-0131, ADR-0254]
date: 2026-05-20
doc_status: published
---

# Messenger Engineer onboarding — first 5 working days on `messenger`

Audience: a new messenger engineer, E2EE platform engineer, or trust-and-safety engineer joining the `messenger` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, exercised MLS RFC 9420 key delivery (KeyPackage upload → Welcome enqueue → Commit accept), joined a 5k-member channel, simulated device-add via external commit, walked the MLS recovery ceremony, and shadowed a huddles SFU drill.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 40 min). Note the five-vendor displacement (Slack/Teams/Discord/Telegram/WhatsApp-Business) + MLS-default doctrine.
2. Read `ARCHITECTURE.md` § mls-key-delivery + § cross-tenant-cohort-isolation + § huddles-sfu-trust-boundary + § eDiscovery-ciphertext-only (∼ 60 min).
3. Read `decisions/ADR-MSG-001-mls-e2ee-key-delivery-architecture.md` end-to-end. This is the binding architecture (∼ 45 min).
4. Read `decisions/ADR-MSGR-0001-huddles-placement.md`, `ADR-MSGR-0002-e2e-personal-dm-key-escrow.md`, `ADR-MSGR-0003-search-backend-selection.md`, `ADR-MSGR-0004-federation-posture.md` (∼ 30 min total).
5. Read RFC 9420 §§ 1-7 (MLS overview, KeyPackage, Welcome, Commit) for protocol grounding (∼ 90 min — yes, all of it; you will need this).
6. Open the Grafana folder `messenger`. Primary boards: `messenger-mls-commit-accept-latency`, `messenger-mls-pending-welcome-age`, `messenger-mls-epoch-reject-rate`, `messenger-key-package-fetch-latency`, `messenger-huddle-join-latency`, `messenger-cross-tenant-grant-active-total`.
7. Walk `runbooks/README.md`. The on-call runbooks: `e2e-encryption-key-rotation.md`, `mls-commit-storm.md`, `pending-welcome-queue-backlog.md`, `huddle-sfu-degraded.md`, `cross-tenant-federation-deny.md`, `key-package-exhaustion.md`, `mls-recovery-failed.md`, `signing-key-rotation-overlap-stuck.md`.
8. Sit in on the Wednesday messenger-substrate handoff. Watch outgoing rotation review the past-week MLS commit-accept p99 + pending-Welcome p95 + cross-tenant-grant audit summary.

Acceptance: you can sketch the send path: client encrypts under MLS group epoch → POST `/v1/messenger/conversations/{id}/mls/commits` → server Cedar `messenger::mls_commit::append` → server validates sequencing + signs Commit → audit-chain `EVT-MSG-MLS-COMMIT-ACCEPTED` → Pulsar fanout to per-device Welcome queue. And the recovery path: lost device → passkey step-up at AAL3 → Cedar `messenger::mls_recovery::request` → server creates external-commit slot → new device generates KeyPackage → MLS external commit advances group epoch → old device leaf revoked.

## Day 2 — demo_trial cell bootstrap + first MLS key delivery

```text
Native operation: messenger bootstrap
Route: cloud control-plane operation ledger (not local retired CLI/raw Cargo)
Required evidence:
- Buck2 target(s) for the changed contract/runtime
- Prow/Kubernetes-native `oya-ci-required` job URL
- operation ledger id and emitted audit-chain event ids
```

Expected runtime: ≤ 14 min. Verify:

```sh
oya messenger health --cell drill-syd-1
# Expected:
#   postgres.messenger_key_delivery: up (lag_ms=18)
#   scylla.messages: up (RF=3)
#   pulsar.messenger-events: connected (geo-rep: disabled)
#   openbao.signing-keys: up (lease_count=4)
#   audit-chain.emit: up
#   mls.key-package-pool: warming (target=10000, current=8214)
```

Create a tenant + a conversation:

```sh
oya messenger tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --display-name "ACME Messenger" \
    --federation-posture isolated \
    --huddles-enabled false

oya messenger conversation create \
    --tenant drill-acme \
    --kind dm \
    --members u-alice@drill.test,u-bob@drill.test \
    --mls-ciphersuite MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519
# Output:
#   conversation_id: c_drill_001
#   mls_group_id: mg_e5a4b3c2d1...
#   epoch: 0
```

Upload a KeyPackage for Alice's device:

```sh
oya messenger keypackage upload \
    --tenant drill-acme \
    --principal u-alice@drill.test \
    --device-id d_alice_macbook_001 \
    --key-package-bytes-path ./alice-kp.bin \
    --attestation-ref webauthn-aaguid:ee882879-721c-4913-9775-3dfcce97072a
# Output:
#   key_package_id: kp_alice_001
#   credential_epoch: 1
#   expires_at: 2026-05-27T14:32:17Z
```

Verify the audit emission:

```sh
oya audit query --tenant drill-acme --event-class "oya.messenger.mls.*" --since 5m
# Expected:
#   oya.messenger.mls.key_package.uploaded.v1 (kp_alice_001)
#   oya.messenger.mls.welcome.enqueued.v1 (for u-bob; epoch 1 join)
```

Acceptance: cell bootstrap; tenant + conversation + KeyPackage round-trip; audit-chain emissions verified.

## Day 3 — MLS Commit + device replacement via external commit

Send the first encrypted Commit (creating epoch 2):

```sh
# In production, this is driven by the client SDK; here we use the drill harness.
oya messenger mls-commit append \
    --tenant drill-acme \
    --conversation c_drill_001 \
    --sender-device d_alice_macbook_001 \
    --commit-bytes-path ./alice-commit-epoch-2.bin
# Output:
#   epoch: 2
#   tree_hash: blake3:7c4a2b8e...
#   audit_event_id: ae_msg_mls_commit_001
```

Now simulate Alice's lost device:

```sh
# 1. Alice authenticates on her new device with passkey step-up to AAL3
oya identity stepup \
    --tenant drill-acme \
    --principal u-alice@drill.test \
    --required-acr aal3_hardware_bound
# Output: stepup_id=su_alice_001, acr=aal3_hardware_bound

# 2. Initiate MLS recovery (external commit)
oya messenger mls-recovery request \
    --tenant drill-acme \
    --principal u-alice@drill.test \
    --replacement-device-id d_alice_iphone_002 \
    --stepup-id su_alice_001 \
    --reason lost_device
# Output:
#   recovery_grant_id: rg_alice_001
#   external_commit_slot: pending
#   status: awaiting_keypackage_upload

# 3. Upload KeyPackage for the new device
oya messenger keypackage upload \
    --tenant drill-acme \
    --principal u-alice@drill.test \
    --device-id d_alice_iphone_002 \
    --key-package-bytes-path ./alice-iphone-kp.bin \
    --attestation-ref webauthn-aaguid:9ddd1817-af5a-4672-a2b9-3e3dd95000a9

# 4. Server constructs external-commit proposal; client merges
oya messenger mls-commit external-commit \
    --tenant drill-acme \
    --conversation c_drill_001 \
    --recovery-grant-id rg_alice_001
# Output:
#   epoch: 3
#   removed_devices: [d_alice_macbook_001]
#   added_devices: [d_alice_iphone_002]
#   audit_event_id: ae_msg_mls_recovery_001
```

Verify the old device is now denied:

```sh
oya messenger mls-commit append \
    --tenant drill-acme \
    --conversation c_drill_001 \
    --sender-device d_alice_macbook_001 \
    --commit-bytes-path ./alice-stale-commit.bin
# Expected: 403 Forbidden; EVT-MSG-MLS-EPOCH-REJECTED
```

Acceptance: MLS Commit accepted; external-commit device replacement verified; revoked-leaf denial verified.

## Day 4 — Huddles SFU + cross-tenant cohort (paid shadow)

paid tier enables huddles. Shadow at demo_trial by enabling for one tenant:

```sh
oya messenger tenant update \
    --tenant drill-acme \
    --huddles-enabled true \
    --huddles-sfu-endpoint livekit://drill-livekit-syd-1:7880
```

Start a huddle in the conversation:

```sh
oya messenger huddle start \
    --tenant drill-acme \
    --conversation c_drill_001 \
    --initiator u-alice@drill.test \
    --mode audio_video
# Output:
#   huddle_id: h_drill_001
#   sfu_url: wss://drill-livekit-syd-1:7880/?room=h_drill_001
#   sfu_token: <ephemeral JWT, 60s TTL>
#   tracks: alice=audio,video; bob=audio,video
```

The SFU never sees plaintext audio/video — clients negotiate SRTP+DTLS keys via the MLS group epoch (per IP-005-huddles-key-derivation).

Walk the huddle-sfu-degraded runbook. Read `runbooks/huddle-sfu-degraded.md`. Scenario: LiveKit SFU node fails mid-call. Runbook covers:

1. Identify from `messenger-huddle-join-latency` panel.
2. Confirm SFU node health via `oya livekit health --cell drill-syd-1`.
3. Trigger SFU rebind: clients reconnect to standby node; MLS group epoch unchanged.
4. Verify call resumed; audit-chain `EVT-MSG-HUDDLE-SFU-REBIND`.
5. Post-incident: SLO impact analysis.

Now simulate cross-tenant cohort (paid feature; per ADR-MSGR-0004 federation posture):

```sh
# 1. drill-acme + drill-betta both grant the tenant-pair federation
oya messenger federation grant \
    --tenant drill-acme \
    --peer-tenant drill-betta \
    --scope channel_membership \
    --verified-corp-email-required true \
    --expires-at 2026-08-20T00:00:00Z

oya messenger federation grant \
    --tenant drill-betta \
    --peer-tenant drill-acme \
    --scope channel_membership \
    --verified-corp-email-required true \
    --expires-at 2026-08-20T00:00:00Z

# 2. Create the cohort channel on drill-acme
oya messenger channel create \
    --tenant drill-acme \
    --channel-id sec-engineering-cohort \
    --cross-tenant-mode federated \
    --federation-allowlist drill-betta \
    --member-eligibility verified-corp-email

# 3. A verified user at drill-betta joins
oya messenger channel join \
    --tenant drill-betta \
    --channel-cross-tenant drill-acme/sec-engineering-cohort \
    --principal u-alice@drill-betta.test
# Expected: Cedar permits; MLS group adds drill-betta's leaf; EVT-MSG-MLS-CROSS-TENANT-JOIN
```

Acceptance: huddle verified; SFU degradation runbook walked; cross-tenant cohort federation verified.

## Day 5 — eDiscovery + key-rotation drill

eDiscovery (ciphertext-only per ADR-MSG-001 Constraint MSG-C7):

```sh
oya messenger ediscovery export \
    --tenant drill-acme \
    --conversation c_drill_001 \
    --from 2026-05-01T00:00:00Z \
    --to 2026-05-20T23:59:59Z \
    --output ./drill-ediscovery-export.tar.gz
# Output:
#   exported_objects:
#     ciphertext_messages: 1284
#     mls_commits: 4
#     membership_snapshots: 4
#     audit_chain_events: 1320
#   plaintext_export: NEVER (per ADR-MSG-001 Constraint MSG-C7)
#   tenant_legal_hold_appliance_endpoint: https://legal-hold.drill.test (tenant-controlled)
```

Server signing key rotation drill (30-day cadence with 48h overlap):

```sh
oya messenger signing-key rotate \
    --tenant drill-acme \
    --cell drill-syd-1 \
    --reason scheduled \
    --overlap-hours 48
# Output:
#   new_key_id: sk_drill_acme_2026_05_20
#   previous_key_id: sk_drill_acme_2026_04_20
#   overlap_until: 2026-05-22T14:32:17Z
#   audit_event_id: ae_msg_signing_key_rotated_001
```

Verify both keys accept signatures during the overlap window:

```sh
oya messenger signing-key verify --cell drill-syd-1 --key-id sk_drill_acme_2026_04_20
# Expected: valid (in overlap window)
oya messenger signing-key verify --cell drill-syd-1 --key-id sk_drill_acme_2026_05_20
# Expected: valid (active)
```

Acceptance: eDiscovery export ciphertext-only verified; signing-key rotation overlap verified.

## What you've learned

- demo_trial bootstrap + MLS KeyPackage upload + Commit accept.
- Device replacement via MLS external commit (passkey-AAL3-gated).
- Huddles SFU trust boundary (never sees plaintext).
- Cross-tenant cohort federation with verified-corp-email gate.
- eDiscovery ciphertext-only export model.
- Server signing key rotation with 48h overlap.

Next week: paid promotion (multi-region MLS Welcome delivery, huddles edge POPs), paid advanced tour (500k-member channels, regulated war rooms with `MLS_256_DHKEMP384_AES256GCM_SHA384_P384`), paid compliance-pack tour (FIPS-140-3 L3 HSM, SecureDrop bridge, sovereign-pack ciphersuite enforcement), and your first production shadow on cross-tenant-grant approval.
