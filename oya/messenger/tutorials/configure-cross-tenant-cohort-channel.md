---
doc_class: Tutorial
microservice: messenger
persona: messenger-engineer + tenant-admin
related_adrs: [ADR-MSG-001, ADR-MSGR-0004]
date: 2026-05-20
doc_status: published
---

# Tutorial — Configure a cross-tenant cohort channel with MLS E2EE + verified-corp-email gate

You will: grant tenant-pair federation between two tenants, create an MLS-protected cross-tenant cohort channel, join members from both tenants with verified-corporate-email gates, send an encrypted message, exercise a member-removal Commit, and verify the audit-chain trail. Total time ≤ 75 minutes.

## Pre-requisites

- Two tenants (`acme-corp` + `betacorp`) on paid tenant_class (`tenant_class model in ADR-0330`).
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `messenger_admin` Cedar role for each tenant.
- Each user has completed identity verification (corporate-email) via the `community` µservice flow.
- The `intelligence` µservice configured for moderation (optional but recommended).

## Step 1 — Mutual tenant-pair federation grants (≤ 10 min)

Both tenants must explicitly grant. Federation is symmetric:

```sh
# acme-corp grants betacorp
oya messenger federation grant \
    --tenant acme-corp \
    --peer-tenant betacorp \
    --scope channel_membership \
    --verified-corp-email-required true \
    --pack-residency-must-match false \
    --expires-at 2026-08-20T00:00:00Z \
    --justification "Joint Q3 incident-response retainer"

# betacorp grants acme-corp (mutual)
oya messenger federation grant \
    --tenant betacorp \
    --peer-tenant acme-corp \
    --scope channel_membership \
    --verified-corp-email-required true \
    --pack-residency-must-match false \
    --expires-at 2026-08-20T00:00:00Z \
    --justification "Joint Q3 incident-response retainer (matching grant)"
```

Verify both grants are active:

```sh
oya messenger federation list --tenant acme-corp
# Output:
#   peer_tenant: betacorp
#   scope: channel_membership
#   verified_corp_email_required: true
#   state: active
#   expires_at: 2026-08-20T00:00:00Z
#   audit_event_id: ae_msg_federation_grant_001

oya messenger federation list --tenant betacorp
# Output: mirror of the above
```

## Step 2 — Create the cohort channel on acme-corp (≤ 10 min)

```sh
oya messenger channel create \
    --tenant acme-corp \
    --channel-id ir-cohort-2026q3 \
    --display-name "Incident Response Cohort 2026-Q3" \
    --description "Joint IR exercises between Acme + Beta security teams" \
    --cross-tenant-mode federated \
    --federation-allowlist betacorp \
    --member-eligibility verified-corp-email \
    --mls-ciphersuite MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519 \
    --huddles-enabled true \
    --moderation-policy default
# Output:
#   channel_id: ch_acme_ir_cohort_2026q3
#   mls_group_id: mg_d8e7f6a5b4c3...
#   initial_epoch: 0
#   audit_event_id: ae_msg_channel_created_001
```

Verify the channel:

```sh
oya messenger channel show --tenant acme-corp --channel ch_acme_ir_cohort_2026q3 | jq '.cross_tenant_mode, .federation_allowlist, .member_eligibility, .mls_group_id'
# Output:
#   "federated"
#   ["betacorp"]
#   "verified-corp-email"
#   "mg_d8e7f6a5b4c3..."
```

## Step 3 — Members join from both tenants (≤ 15 min)

Acme user joins:

```sh
oya messenger channel join \
    --tenant acme-corp \
    --channel ch_acme_ir_cohort_2026q3 \
    --principal u-alice@acme-corp.com
# Cedar evaluates:
#   - principal.tenant_id == channel.tenant_id ✓
#   - principal has verified-corp-email claim ✓
#   - permits messenger::channel::join
# Output:
#   joined_at_epoch: 1
#   leaf_index: 0
```

Beta user joins (cross-tenant):

```sh
oya messenger channel join \
    --tenant betacorp \
    --channel-cross-tenant acme-corp/ch_acme_ir_cohort_2026q3 \
    --principal u-bob@betacorp.com
# Cedar evaluates:
#   - principal.tenant_id (betacorp) is in channel.federation_allowlist ✓
#   - tenant-pair federation grant active ✓
#   - principal has verified-corp-email claim ✓
#   - permits messenger::channel::join (cross-tenant)
# Output:
#   joined_at_epoch: 2
#   leaf_index: 1
#   audit_event_id: ae_msg_mls_cross_tenant_join_001
```

Try joining without verified-corp-email (should fail):

```sh
oya messenger channel join \
    --tenant betacorp \
    --channel-cross-tenant acme-corp/ch_acme_ir_cohort_2026q3 \
    --principal u-charlie@betacorp.com   # has no verified claim
# Expected: 403 Forbidden
# Cedar deny reason: "messenger::channel::join requires verified-corp-email claim for federated channels"
```

## Step 4 — Send the first encrypted message (≤ 10 min)

In production, this is driven by the client SDK. For the drill:

```sh
# Alice's client generates plaintext, encrypts under MLS group epoch 2, signs, and POSTs
oya messenger message send \
    --tenant acme-corp \
    --conversation ch_acme_ir_cohort_2026q3 \
    --sender u-alice@acme-corp.com \
    --sender-device d_alice_macbook_001 \
    --plaintext "Welcome to the IR cohort! First exercise scheduled for Wed 14:00 UTC." \
    --content-type text/plain
# Output:
#   message_id: m_acme_ir_001
#   sent_at_epoch: 2
#   ciphertext_size: 142 bytes
#   fanout_recipients: 2 (alice's leaf for delivery confirmation; bob's leaf for read)
#   audit_event_id: ae_msg_sent_001
```

Bob's client decrypts (under the MLS group epoch 2 key) and renders. Verify from the server side what the server saw:

```sh
oya messenger message show --tenant acme-corp --message m_acme_ir_001
# Output:
#   message_id: m_acme_ir_001
#   ciphertext: <base64-encoded ciphertext; opaque to server>
#   sender_device: d_alice_macbook_001
#   sent_at_epoch: 2
#   mls_group_id: mg_d8e7f6a5b4c3...
#   plaintext: NEVER (E2EE; server cannot decrypt)
```

## Step 5 — Member removal Commit (≤ 10 min)

Suppose betacorp wants to remove Bob from the cohort (he left the team). Per ADR-MSG-001, this is an MLS Remove proposal + Commit advancing epoch:

```sh
# betacorp messenger admin initiates removal
oya messenger channel remove-member \
    --tenant betacorp \
    --channel-cross-tenant acme-corp/ch_acme_ir_cohort_2026q3 \
    --principal u-bob@betacorp.com \
    --requester u-bobs-manager@betacorp.com \
    --reason "Left the team; access revoked per offboarding policy"
# Cedar evaluates:
#   - requester has messenger::channel::remove_member ✓
#   - principal is a member of the channel ✓
#   - tenant scope: betacorp can remove its own members from cross-tenant channels
# Output:
#   epoch_advanced_to: 3
#   removed_leaf: bob (leaf_index=1)
#   audit_event_id: ae_msg_mls_member_removed_001
```

Verify Bob can no longer send:

```sh
oya messenger message send \
    --tenant betacorp \
    --conversation ch_acme_ir_cohort_2026q3 \
    --sender u-bob@betacorp.com \
    --sender-device d_bob_iphone_001 \
    --plaintext "Test"
# Expected: 403 Forbidden
# EVT-MSG-MLS-EPOCH-REJECTED: device d_bob_iphone_001 leaf revoked at epoch 3
```

Per RFC 9420 forward-secrecy: Bob's client can still decrypt messages sent BEFORE epoch 3 (he had access then), but cannot read messages from epoch 3 onward.

## Step 6 — Huddles join (≤ 10 min)

Start a huddle in the channel:

```sh
oya messenger huddle start \
    --tenant acme-corp \
    --channel ch_acme_ir_cohort_2026q3 \
    --initiator u-alice@acme-corp.com \
    --mode audio_video
# Output:
#   huddle_id: h_ir_001
#   sfu_url: wss://livekit-syd-1.oyatie.local:7880/?room=h_ir_001
#   sfu_token: <ephemeral JWT, 60s TTL>
#   mls_subgroup_id: mg_h_ir_001_subgroup
#   ciphersuite: MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519
```

Members of the channel can join. Cedar `messenger::huddle::join` permits members. The SFU mixes SRTP streams whose keys are derived from `mls_subgroup_id` epoch — the SFU never sees plaintext audio/video.

## Step 7 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "oya.messenger.*" --since 60m
```

Expected events for our flow:

- `oya.messenger.federation.grant.created.v1` (× 2; mutual grants)
- `oya.messenger.channel.created.v1`
- `oya.messenger.channel.member_joined.v1` (× 2; alice + bob cross-tenant)
- `oya.messenger.channel.member_join_denied.v1` (× 1; charlie missing verified-corp-email)
- `oya.messenger.mls.commit.accepted.v1` (× ≥ 3; channel-create + member-joins + remove)
- `oya.messenger.mls.welcome.enqueued.v1` (× 2; for alice + bob's devices)
- `oya.messenger.message.sent.v1` (× 1; alice's message)
- `oya.messenger.channel.member_removed.v1` (× 1; bob)
- `oya.messenger.mls.epoch.rejected.v1` (× 1; bob's post-removal send attempt)
- `oya.messenger.huddle.started.v1` (× 1)

All Ed25519-signed; chain verifies:

```sh
oya audit verify-chain --tenant acme-corp --since 60m
# Output: chain verified, all events signed, signature_gaps: 0
```

## Step 8 — Mock cross-tenant disclosure request (≤ 10 min)

Suppose betacorp legal-counsel needs the channel's ciphertext + membership trail for an eDiscovery matter (Bob's removal context):

```sh
oya messenger ediscovery export \
    --tenant betacorp \
    --channel-cross-tenant acme-corp/ch_acme_ir_cohort_2026q3 \
    --scope membership-and-ciphertext \
    --requester u-legal-counsel@betacorp.com \
    --court-order-evidence ./court-order-2026-05-20.pdf \
    --justification "Subpoena for IR cohort communications" \
    --from 2026-05-15T00:00:00Z \
    --to 2026-05-20T23:59:59Z \
    --output ./ir-cohort-ediscovery.tar.gz
# Cedar evaluates:
#   - requester has messenger::ediscovery::export ✓
#   - court-order evidence present ✓
#   - cross-tenant: betacorp can export its own members' membership trail + the ciphertext THEY had access to
#   - PLAINTEXT EXPORT: denied unconditionally (per ADR-MSG-001 § Cedar)
# Output:
#   exported:
#     membership_snapshots: 4 (epochs 0-3)
#     ciphertext_messages: 1 (m_acme_ir_001)
#     audit_chain_events: 12
#   plaintext: NOT EXPORTED (use betacorp legal-hold appliance to decrypt with bob's recovered keys)
```

The export contains ciphertext + membership; to decrypt, betacorp uses its legal-hold appliance (deployed in betacorp's compliance boundary) which holds Bob's device keys per Mode-B tenant escrow (if enabled per ADR-MSGR-0002) or requires Bob's voluntary key export.

## What you've learned

- Mutual tenant-pair federation grant model.
- Cross-tenant cohort channel with verified-corp-email gate.
- MLS group joining across tenants with Cedar enforcement.
- Encrypted message send with server never seeing plaintext.
- Member removal via MLS Remove + Commit (epoch advancement).
- Huddles with SFU-blind audio/video.
- Audit-chain verification of the full flow.
- eDiscovery ciphertext-only export model.

Next tutorial: `tutorials/upgrade-conversation-ciphersuite-to-fips-l3.md` — migrate a conversation from default to `MLS_256_DHKEMP384_AES256GCM_SHA384_P384` for pack compliance (paid advanced capability).
