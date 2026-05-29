---
doc_class: MigrationPlaybook
microservice: connector
vendor: Slack + Microsoft Teams External Access + Discord servers + Matrix homeservers
date: 2026-05-20
doc_status: published
---

# Migration playbook — Slack / Microsoft Teams External Access / Discord servers / Matrix → oyatie connect

Audience: an oyatie tenant migrating their cross-organisation collaboration substrate from Slack Connect, Microsoft Teams External Access, Discord servers, or Matrix federation to oyatie's `connector` µservice.

## Why this migration is non-trivial

- **Slack Connect** is bilateral but proprietary: shared channels lock both tenants into Slack. Migration requires BOTH tenants to migrate (or operate in dual-state).
- **Microsoft Teams External Access** is tied to Azure AD B2B / B2C; identity portability is the gating concern.
- **Discord** doesn't have native federation; tenants use bots-as-bridges, which carries no compliance posture.
- **Matrix** is open-protocol; federation can be bridged via libraries but disclosure rules + Cedar gates must be added at the bridge.

The 80/20: channel substrate ports cleanly via the auto-converter; the 20 % needing care is bilateral migration coordination (you can't migrate Slack alone) + identity portability.

## Step 1 — Identify the source + audit federation peers (≤ 1 week)

For Slack Connect:

```sh
oya connect migrate inventory \
    --source slack-connect \
    --slack-admin-token "$SLACK_ADMIN_TOKEN" \
    --workspace-id "$WORKSPACE_ID" \
    --out inventory/slack-connect.yaml
```

Captures: shared channels, peer workspaces, channel members, channel disclosure (PUBLIC / PRIVATE), DLP policies, retention policies, app-integrations.

For Teams External Access:

```sh
oya connect migrate inventory \
    --source teams-external-access \
    --tenant-id "$M365_TENANT_ID" \
    --graph-token "$GRAPH_TOKEN" \
    --out inventory/teams-external.yaml
```

Captures: external-collaboration tenants, allowed-domain list, B2B guest accounts, shared channels (Teams Connect), inbound/outbound communication policies.

For Discord servers:

```sh
oya connect migrate inventory \
    --source discord \
    --bot-token "$DISCORD_BOT_TOKEN" \
    --out inventory/discord.yaml
```

Captures: server-IDs, channel-IDs, roles, bot integrations.

For Matrix:

```sh
oya connect migrate inventory \
    --source matrix \
    --homeserver-url https://matrix.acme.example \
    --admin-access-token "$MATRIX_ADMIN_TOKEN" \
    --out inventory/matrix.yaml
```

Captures: rooms, federated homeservers, user-IDs, history-visibility setting, end-to-end-encryption setting (Olm/Megolm).

## Step 2 — Coordinate with peer tenants (≤ 4-12 weeks)

This is the long pole. Each external organisation in the federation must agree on:

- Both/each migrate to oyatie (most clean).
- One migrates; one stays on source; we run a one-way bridge (more complex, supported for 6-12 mo).
- One migrates; one becomes unreachable (acceptable if relationship is winding down).

Document the agreement per peer:

```sh
oya connect migrate peer-agreement add \
    --tenant drill-acme \
    --peer drill-beta-vendor \
    --source-platform slack-connect \
    --agreement-type both-migrate \
    --target-date 2026-08-15 \
    --justification "joint-cutover-agreement-sent-2026-05-15"
```

## Step 3 — Provision oyatie tenant + federation peer-requests (≤ 1-2 weeks)

Per the tutorial `tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md`. For each peer:

```sh
oya connect federation peer-request \
    --tenant drill-acme \
    --peer-tenant drill-beta-vendor \
    --intent "cutover-from-slack-connect-2026-08-15" \
    --proposed-channels $(yq '.shared_channels[].name' inventory/slack-connect.yaml) \
    --disclosure-baseline TENANT-ONLY \
    --duration 365d
```

## Step 4 — Channel-by-channel migration (≤ 2-8 weeks)

For each shared channel, port:

```sh
oya connect migrate convert-channel \
    --source slack-connect \
    --source-channel-id C012ABCDEF \
    --target-tenant drill-acme \
    --target-channel-name supplier-status \
    --include-history true \
    --history-window 365d
```

Mapping:

| Slack concept | oyatie connect equivalent |
|---|---|
| Shared channel | Federated channel |
| Channel-level "share with another organisation" | Federation peer + channel-bridge |
| `usergroups@` (per-tenant) | Tenant group (Cedar group) |
| External member badge | Peer-tenant member indicator |
| Posts (free-form) | Messages |
| Threads | Threaded messages |
| Reactions | Reactions |
| Pinned messages | Pinned messages |
| Slash commands | Bot integrations (via bot bridge per IP-011) |
| Apps + integrations | Re-author per oyatie's app pattern |
| Files (uploaded) | Drive µservice references |
| DLP rules | Cedar disclosure rules |
| Retention | Cedar pack retention overlay |

For Microsoft Teams External Access: similar mapping; the Teams "Channels" concept maps; "Tenant" maps to oyatie tenant; "Guest" maps to peer-tenant principal.

For Discord: most concepts don't map cleanly; the migration is a re-platform rather than 1:1.

For Matrix: rooms map 1:1; federated rooms map to oyatie federated channels; Megolm group keys are not preserved (new MLS group at oyatie).

## Step 5 — Dual-run period (≤ 2-4 weeks)

For each peer, run the channels in dual-state: source + oyatie. Members on both sides see two channels. Forwarding bridge:

```sh
oya connect migrate dual-run-bridge start \
    --tenant drill-acme \
    --source-channel-id C012ABCDEF \
    --target-channel supplier-status \
    --direction bidirectional
```

The bridge forwards messages between source + oyatie until cutover. Watch for divergence:

```sh
oya connect migrate dual-run-divergence --tenant drill-acme --since 24h
```

Common divergence:

- File attachments differ in source vs oyatie URL.
- Threads order subtly differently (if Slack thread-reply happens before bridge syncs).

## Step 6 — Cutover + decommission source (≤ 2 weeks)

```sh
oya connect migrate cutover \
    --tenant drill-acme \
    --source slack-connect \
    --target oyatie-connect \
    --peer drill-beta-vendor \
    --cutover-time 2026-08-15T09:00:00-04:00
```

At cutover:

1. Source channel is set to read-only.
2. New messages go ONLY to oyatie.
3. Dual-run bridge is decommissioned.
4. Source workspace can be cancelled after 30-day rollback buffer.

## Step 7 — Audit + decommission (≤ 1 month)

```sh
oya connect migrate decommission \
    --tenant drill-acme \
    --source slack-connect \
    --evidence-out evidence/migrations/slack-connect-to-oyatie-drill-acme.json
```

Evidence file includes:

- Per-peer migration timeline.
- Channels migrated.
- Message counts ported.
- File attachments migrated (or referenced).
- Bot integrations re-authored.
- DLP / retention policies migrated.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Peer tenants don't agree to migrate | Critical | Per-peer agreement before migration; dual-run bridge if one stays |
| Channel history not portable | High | Decide per-channel; some history kept on source (read-only); some ported |
| Bot integrations are platform-specific | High | Re-author per oyatie's bot pattern; budget engineer time per bot |
| DLP rules differ in semantics | High | Map source DLP → Cedar policy; test in dual-run period |
| Slack Connect's "Direct Messages with external users" → oyatie has no direct DM-cross-tenant (uses bridged channel) | Medium | Use bridged 2-person channels as DM substitute |
| Teams external access tied to Azure AD B2B → oyatie identity has no B2B (tenant boundary) | High | Migrate B2B users to oyatie identity µservice; budget 4-8 wk |
| Matrix users keep their `@user:matrix.acme.example` identity | Medium | Provision oyatie identity for users; gradual identity-swap during dual-run |
| Slack Apps marketplace integrations not portable | High | Re-author OR find oyatie marketplace equivalent OR sunset |
| Cross-tenant search differs (Slack indexes all external) | Medium | Document oyatie scoped-search; user expectation reset |
| Pack-bound migration (BAA / NDA) | Critical | Validate pack compliance pre-migration; legal review per peer |
