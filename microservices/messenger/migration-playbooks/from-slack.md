---
doc_class: MigrationPlaybook
microservice: messenger
vendor: Slack (Standard, Plus, Business+, Enterprise Grid)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Slack → oyatie messenger

Audience: a team running Slack (Standard, Plus, Business+, or Enterprise Grid) for company messaging. Drivers: MLS RFC 9420 E2EE-by-default + per-tenant cross-tenant disclosure policy + audit-chain non-repudiation + sovereign-pack residency + 7.5× TCO reduction at Enterprise scale.

## Why this migration matters

Slack is excellent at:

- Best-in-class UX for casual + structured workplace chat.
- Massive integration ecosystem (1 500+ apps in the Slack App Directory).
- Channel-based mental model that scales from team-level to org-wide.
- Slack Connect for cross-organization workflows.
- Slack Enterprise Grid for multi-workspace organizations.

oyatie messenger adds:

- **MLS RFC 9420 E2EE-by-default for all DMs, channels, huddles** (Slack uses TLS-in-transit + at-rest only).
- **Cryptographic audit-chain non-repudiation** (Slack's audit log is server-mutable).
- **Per-tenant Cedar-gated cross-tenant disclosure** (Slack Connect's controls are coarser).
- **FIPS-140-3 L3 HSM key custody at paid compliance-pack** for healthcare/finance/public-sector.
- **eDiscovery without server-side plaintext export** — tenant owns decryption keys.
- **No per-seat licensing** — self-hosted paid at $1.0M/year for 50k seats vs Slack Enterprise Grid at $7.6M/year.
- **Sovereign-pack residency** (KR-PIPA, EU-GDPR Art 9, US-HIPAA, FedRAMP-High, CN-PIPL).

The trade-off: Slack's app integration ecosystem is mature (~ 1 500 apps). oyatie's plugin SDK supports custom integrations but the breadth of pre-built integrations is smaller at launch (~ 80 first-party + community). Plan for parallel integration porting during shadow phase.

## Step 1 — Inventory the Slack estate (≤ 1-2 weeks)

```bash
# Use Slack's Data Loss Prevention export (Enterprise Grid required for full corporate export)
slack-cli export \
    --workspace acme-corp \
    --include channels,users,dms,private-channels,files,reactions,pins,bookmarks \
    --since 2020-01-01 \
    --output ./slack-export/

# Or use the Slack Admin Export API for Enterprise Grid:
curl -X POST "https://slack.com/api/admin.exports.export" \
    -H "Authorization: Bearer $SLACK_ADMIN_TOKEN" \
    -d '{"date_range":{"start":"2020-01-01","end":"2026-05-20"},"export_type":"all"}'
```

Document:

- Workspace count (Enterprise Grid only).
- Channel count + per-channel member count distribution.
- DM count + cross-workspace DM count.
- User count + Slack Connect external user count.
- Active App integrations (and their replacement plan).
- Active Workflow Builder workflows.
- Slack Connect partnerships (counterparty orgs + scopes).
- Slack Enterprise Key Management (EKM) configuration (if applicable).
- Compliance exports + retention settings.

Typical mid-size Enterprise Grid: 1 workspace per business unit (5-20 workspaces), 5k-50k users, 5k-50k channels, 100M-10B messages.

## Step 2 — Map Slack concepts to oyatie messenger (≤ 1 week)

| Slack concept | oyatie messenger equivalent |
|---|---|
| Workspace (Standard/Plus) | Tenant |
| Enterprise Grid Org | Conglomerate parent tenant (per `ADR-TEN-001`) |
| Enterprise Grid Workspace | Child tenant in conglomerate |
| Channel (public/private) | Channel (per-channel ACL via Cedar) |
| Direct Message (DM) | Conversation kind=`dm` |
| Multi-Party DM (MPDM) | Conversation kind=`group_dm` |
| Slack Connect (cross-org DM) | Cross-tenant federation grant + cohort channel |
| Slack Huddle | oyatie huddle (LiveKit SFU; MLS-derived SRTP) |
| Slack Threads | MLS sub-conversation |
| Slack Reactions | Per-message reaction (encrypted under MLS group epoch) |
| Slack Pinned messages | Pinned messages (server-side metadata; ciphertext) |
| Slack App (Bot) | oyatie messenger plugin via Plugin SDK |
| Slack Workflow Builder | `workflow-engine` µservice flow |
| Slack Enterprise Key Management (EKM) | OpenBao + per-tenant HSM (default in oyatie) |
| Slack Compliance Export | Ciphertext + membership + audit-chain export |
| Slack Discovery API | oyatie eDiscovery API + tenant legal-hold appliance |

## Step 3 — Data migration (≤ 4-12 weeks per 1B messages)

```sh
oya messenger migrate import-slack \
    --tenant acme-corp \
    --slack-export-dir ./slack-export/ \
    --map-workspace-to-tenant acme-engineering=acme-corp,acme-sales=acme-corp-sales \
    --conglomerate-parent acme-corp \
    --convert-app-mentions plugin-sdk \
    --preserve-message-timestamps true \
    --throttle-rate 5000-msgs-per-sec \
    --mls-ciphersuite MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519
```

The migration:

1. Creates oyatie tenants from Slack workspaces (one tenant per workspace in Enterprise Grid; conglomerate hierarchy via parent tenant).
2. Creates oyatie principals from Slack users (preserve email + display name).
3. Imports channels → oyatie channels (preserve channel slug, description, member list).
4. Imports DMs → oyatie conversations (kind=dm + group_dm).
5. **Re-encrypts message history under MLS group epoch 0** — Slack server-stored plaintext is encrypted under fresh per-conversation MLS groups during import. Subsequent epochs are advanced as devices join.
6. Imports reactions + pins + threads.
7. Replays Slack-Connect partnerships → oyatie tenant-pair federation grants (requires counterparty consent).

Backfill rate ~ 5k msgs/sec at paid. 1B messages → ~ 56 hours.

Verify post-import counts:

```sql
SELECT tenant_id, count(*) AS msg_count
FROM messenger.messages
WHERE imported_from = 'slack'
GROUP BY tenant_id
ORDER BY msg_count DESC;
```

Cross-check against the Slack export's per-channel counts. Acceptable drift: 0 % (entity-level integrity must match).

## Step 4 — User identity migration + SSO (≤ 1-2 weeks)

Slack users have an email + password OR SSO. Map to oyatie identity:

1. Create oyatie principals from Slack users (preserve email).
2. **No password import** — passwords are never migrated (security best practice + identity µservice is passkey-primary per ADR-ID-001).
3. **SSO integration**: if tenant uses Okta/Microsoft Entra ID/Google Workspace as Slack IdP, configure same IdP for oyatie via `identity` µservice OIDC federation.
4. **Passkey enrollment**: users enroll passkeys (WebAuthn Level 3) on first login. Default acr=`aal2_passkey_uv`.
5. **Corporate-email verification**: required for cross-tenant federation per ADR-MSG-001. Verified at first login.

```sh
oya identity import-from-slack-users \
    --tenant acme-corp \
    --slack-users ./slack-export/users.json \
    --sso-idp okta-acme-corp \
    --auto-enroll-passkey-on-first-login true
```

## Step 5 — App integration replacement (≤ 4-16 weeks)

Slack apps → oyatie equivalents:

| Slack App | oyatie equivalent |
|---|---|
| Slack Workflows | `workflow-engine` µservice flow |
| Slack incoming webhooks | oyatie webhook subscription |
| Slack outgoing webhooks (slash commands) | oyatie messenger plugin SDK |
| GitHub for Slack | `gh-app` plugin (oyatie-native) |
| Jira / Linear for Slack | `linear-app` / `jira-app` plugin |
| PagerDuty for Slack | `pagerduty-bridge` plugin |
| Zoom for Slack | Replace with native huddles |
| Polly polls | Built-in poll widget |
| Donut (1:1 matching) | `donut-bridge` plugin (community-µservice integration) |
| Custom Slack apps (internal) | Port to oyatie Plugin SDK (Rust + Wit) |

Custom Slack apps → port to oyatie Plugin SDK. The SDK provides equivalent primitives: slash commands, message actions, view modals, OAuth + webhook events.

## Step 6 — Shadow run + cutover (≤ 8-16 weeks)

Run BOTH Slack + oyatie in parallel. New conversations go to oyatie; existing Slack channels remain read-only or read-write per phase:

Phase 1 (weeks 1-4): Read-only Slack; new conversations on oyatie. Users have both clients.
Phase 2 (weeks 5-8): DMs migrate to oyatie. Channels still on Slack.
Phase 3 (weeks 9-12): Channels migrate one team at a time.
Phase 4 (week 13+): Full cutover; Slack → read-only archive.

After phase 4 begins:

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.messenger_substrate.cut_over \
    --payload '{"from":"slack","to":"oyatie","cutover_at":"2026-08-15T14:00:00Z","preserved_archive_url":"slack.acme-archive.local"}'
```

## Step 7 — Slack decommission (≤ 90-180 d post-cutover)

After ≥ 90 d:

- Export final Slack state for archival (Compliance Export + Discovery API).
- Decommission Slack Enterprise Grid contract.
- Cancel paid integrations.
- Retain archived export for legal-hold duration (typically 7 y for finance, 6 y for HIPAA).

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Slack EKM tenant key extraction not possible (Slack holds the key) | High | Plaintext re-encryption under oyatie MLS during import; old Slack export remains decryptable until decommission |
| User adoption: Slack mental model is sticky | High | Pre-launch training; preserve channel slug + threading UX; provide migration FAQ |
| App integration gap | High | Pre-audit; port top-20 apps to plugin SDK before cutover; defer long-tail apps to community |
| Slack Workflow Builder workflows | Medium | Map to workflow-engine µservice flows; some 1:1 mapping, some require redesign |
| Slack Connect partnerships | Medium | Replay as tenant-pair federation grants; require counterparty consent (may take weeks) |
| Slack EKM compliance certifications | Medium | oyatie paid compliance-pack provides FIPS-140-3 L3 + SOC2 + HIPAA + FedRAMP-High equivalents |
| User mention conventions (`@here`, `@channel`) | Low | Direct 1:1 mapping in oyatie |
| Slack emoji + custom emoji | Low | Custom emoji exported + imported; standard Unicode emoji unchanged |
| Slack file uploads | Medium | Files migrate to `drive` µservice with per-file DEK envelope (per ADR-DRIVE-001) |
| Slack message permalinks | High | Old `https://acme.slack.com/archives/C01/p1234567` permalinks need redirect; provide 301-redirect service for 180 d |
| Slack threads + reply patterns | Low | 1:1 mapping (oyatie thread = Slack thread) |
| Custom Slack themes / branding | Low | oyatie theme support; brand assets re-applied |
| Slack DLP integrations (Nightfall, Polymer, etc.) | Medium | Replace with oyatie's intelligence µservice DLP scanner pre-encryption |
| Slack retention policies | Low | Map to oyatie pack retention class via `compliance` µservice |
| Slack Connect external counterparties (across orgs) | High | Require counterparty buy-in; some may not migrate; provide bridge mode (oyatie ↔ Slack via SCIM + webhook) for transition period |
| Mobile client transition (Slack mobile vs oyatie mobile) | Medium | Side-by-side iOS/Android available; rolling cutover by device |
| User-installed apps (per-user OAuth) | Medium | User-by-user re-auth on oyatie equivalent plugin |
| Slack Search index quality | Low | oyatie search is metadata + client-side for E2EE channels; full-text on cleartext channels matches Slack quality after index warm-up |
