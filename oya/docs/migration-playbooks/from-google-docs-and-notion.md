# Migration playbook — Google Docs or Notion → Oyatie `docs`

Audience: an org running its collaborative documentation on Google Docs (Workspace), Notion, or a mix, who wants to migrate to
Oyatie `docs` without losing version history, permissions, or comments.

> Phase budget: 60 days for ≤ 1,000 docs; 180 days for ≤ 100k docs; 365 days for ≥ 1M docs (large Notion workspaces).

## Phase 0 — Inventory (Day 0…14)

### From Google Workspace
1. Export drive contents:
   ```bash
   # Use Google Takeout for full export OR Drive API for selective
   curl -H "Authorization: Bearer $GW_TOKEN" \
     "https://www.googleapis.com/drive/v3/files?q=mimeType='application/vnd.google-apps.document'&pageSize=1000" \
     > gdoc-list.json
   ```
2. For each doc:
   - Doc ID + last modified.
   - Permission grants (per-user + per-group).
   - Comments (resolved + unresolved).
   - Revision history (full).

### From Notion
1. Use Notion's API:
   ```bash
   curl -X POST 'https://api.notion.com/v1/search' \
     -H "Authorization: Bearer $NOTION_TOKEN" \
     -H 'Notion-Version: 2026-02-22' \
     -H 'Content-Type: application/json' \
     -d '{"filter":{"property":"object","value":"page"}}' \
     > notion-pages.json
   ```
2. For each page:
   - Page ID + database parent (if any).
   - Block hierarchy.
   - Comments.
   - Permission grants (workspace + selective).

## Phase 1 — Tenant + tier provisioning (Day 14…21)

```bash
./bin/oya tenant create \
  --id oyatie.b2b.smb.<your-org>.docs-migration \
  --tenant-class paid \
  --region us-east-2 \
  --pack-set "soc2-type-ii-v2017,gdpr-eu-v2018"
```

Provision users (mirroring your Google/Notion user list):
```bash
for user in $(jq -r '.users[] | .email' user-list.json); do
  ./bin/oya identity user-create \
    --tenant oyatie.b2b.smb.<your-org>.docs-migration \
    --email "$user"
done
```

## Phase 2 — Permission group migration (Day 21…30)

Google Workspace groups + Notion teamspaces translate to Oyatie identity groups:
```bash
./bin/oya identity group-create \
  --tenant oyatie.b2b.smb.<your-org>.docs-migration \
  --group engineering \
  --members "$(jq -r '.engineering[] | .email' user-list.json | paste -sd, -)"
```

Cedar permits per group:
```cedar
permit (
  principal in oyatie.b2b.smb.<your-org>.docs-migration::Group::"engineering",
  action in [docs::Action::Read, docs::Action::Edit, docs::Action::Comment],
  resource is docs::Document
)
when {
  resource.tags.contains("engineering")
};
```

## Phase 3 — Document migration (Day 30…60)

### Google Docs
```bash
./bin/oya docs migrate import \
  --tenant oyatie.b2b.smb.<your-org>.docs-migration \
  --source-format google-docs \
  --source-token $GW_TOKEN \
  --doc-list gdoc-list.json \
  --include-history true \
  --include-comments true \
  --dry-run
```

Review the dry-run output. Then:
```bash
./bin/oya docs migrate import ... --confirm
```

The migrator:
- Renders each Google Doc as a sequence of block ops.
- Re-creates the version history as CRDT op history (best-effort; not all GD operations have CRDT counterparts).
- Migrates comments preserving author + thread structure.
- Maps permissions to Cedar grants.

### Notion
```bash
./bin/oya docs migrate import \
  --tenant oyatie.b2b.smb.<your-org>.docs-migration \
  --source-format notion \
  --source-token $NOTION_TOKEN \
  --page-list notion-pages.json \
  --workspace-mapping notion-mapping.yaml
```

Notion mapping is more complex because Notion's block model is richer; the `notion-mapping.yaml` declares how Notion-specific
blocks (toggles, callouts, columns) map to Oyatie blocks. The default mapping is shipped at
`microservices/docs/migrators/notion-default-mapping.yaml`.

## Phase 4 — Dual-run + parity check (Day 60…80)

For 20 days, run both systems. New docs created in Google/Notion get shadow-replicated to Oyatie:
```bash
./bin/oya docs migrate watcher \
  --tenant oyatie.b2b.smb.<your-org>.docs-migration \
  --source-format google-docs \
  --source-token $GW_TOKEN \
  --since-date 2026-06-01 \
  --interval 1h
```

Parity check daily:
```bash
./bin/oya docs migrate parity-check --tenant oyatie.b2b.smb.<your-org>.docs-migration --window 24h
```

Target: 99.5 % content parity (text + block structure); 99 % comment parity.

## Phase 5 — Cut-over (Day 80…95)

1. Disable new document creation in Google/Notion.
2. Update employee bookmarks / SSO to point at Oyatie `docs`.
3. Set up redirect from Google Docs URLs to Oyatie equivalents (a small Cloudflare Worker can handle this).

## Phase 6 — Decommission (Day 95+)

After 30 d clean run:
- Set old Google Docs to view-only.
- After 90 d: archive entire Google Drive `Documents` folder.
- For Notion: export final archive; cancel subscription.

## Rollback

Within the 20-day dual-run + 15-day cutover window:
1. Re-enable new document creation in Google/Notion.
2. Reverse the SSO redirect.
3. Pause Oyatie `docs` for new edits; mark migrated docs as read-only.
4. Migrate any net-new Oyatie content back via the reverse migrator (re-creating Google Docs / Notion pages).

After 30 d on Oyatie: rollback requires re-creating docs in Google/Notion, which is a major manual effort.

## What you gain

- 2-7x lower keystroke latency.
- 10k concurrent editors per doc at compliance_pack (vs 50-100 vendor max).
- First-class branching + merge workflow.
- BLAKE3 audit chain (tamper-evident).
- EU AI Act ready (for AI-assisted authoring in regulated industries).
- E-sign at every level including FDA 21 CFR Part 11 + KR PKI.

## What you give up

- Google Drive file-system metaphor (Oyatie uses tag-based + folder hybrid).
- Notion's database-as-page-property model (Oyatie embeds `sheets` instead).
- Vendor desktop apps (Word, Google Docs offline) — Oyatie has PWA + native desktop apps but less mature.
- The Notion plugin ecosystem.
