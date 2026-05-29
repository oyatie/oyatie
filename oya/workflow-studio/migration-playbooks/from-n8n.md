---
doc_class: MigrationPlaybook
microservice: workflow-studio
vendor: n8n (self-hosted or n8n Cloud)
date: 2026-05-20
doc_status: published
---

# Migration playbook — n8n → oyatie workflow-studio

Audience: a team running n8n (self-hosted or n8n Cloud) for visual workflow automation. Drivers: tighter integration with oyatie µservices, sovereign-pack residency, audit-chain non-repudiation, real-time collaborative editing at scale, AI-assisted workflow generation.

## Why this migration matters

n8n is excellent at:

- 600+ third-party integrations.
- Self-host option (zero per-tenant cost).
- Active open-source community.

oyatie workflow-studio adds:

- Native oyatie µservice nodes (auto-generated from each µservice's contract).
- Yjs-based CRDT real-time collab at scale (5-20 concurrent editors).
- Cryptographic audit-chain integration (every editor action emitted).
- Sovereign-pack residency.
- AI-assisted workflow generation (paid tenant_class with per_usage billing component).
- Time-travel debugging.
- Per-tenant custom-node SDK with typed I/O contracts.

## Step 1 — Inventory the n8n estate (≤ 1 week)

```bash
# From n8n admin (n8n CLI):
n8n list:workflow --output=./n8n-workflows.json
n8n export:credentials --output=./n8n-credentials.json  # (encrypted; keep secure)
n8n list:user --output=./n8n-users.json
```

Document:

- Active workflows + their tags / categories.
- Integrations in use (Stripe, Slack, OpenAI, Shopify, etc.).
- Custom-code nodes (n8n's "Code" node + custom community nodes).
- Workflow trigger types (webhook, cron, manual, polling).
- User count + their roles.
- Self-hosted infra: n8n instance count, PostgreSQL backing, Redis backing (external-redis: existing n8n estate).

Typical mid-size n8n: 100-500 active workflows, 10-30 integrations, 5-20 custom code nodes, 20-100 users.

## Step 2 — Map n8n node types to oyatie node types (≤ 1-2 weeks)

The translation tool:

```sh
oya workflow-studio migrate convert-n8n \
    --input ./n8n-workflows.json \
    --output-dir ./oyatie-workflows/ \
    --node-mapping ./node-mapping.yaml
```

Mapping file (`./node-mapping.yaml`):

```yaml
# Direct 1:1 mappings
n8n.nodes-base.webhook: oyatie.triggers.http_webhook
n8n.nodes-base.cron: oyatie.triggers.cron
n8n.nodes-base.httpRequest: oyatie.http.request
n8n.nodes-base.set: oyatie.data.set_variable
n8n.nodes-base.if: oyatie.flow.conditional
n8n.nodes-base.merge: oyatie.flow.merge
n8n.nodes-base.splitInBatches: oyatie.flow.split_for_each
n8n.nodes-base.code: oyatie.code.javascript  # n8n Code node → oyatie JS code node
n8n.nodes-base.emailSend: oyatie.mail.send_smtp
n8n.nodes-base.gmail: oyatie.mail.send_gmail
n8n.nodes-base.slack: oyatie.slack.message_post
n8n.nodes-base.discord: oyatie.discord.message_post
n8n.nodes-base.openAi: oyatie.intelligence.request_create  # routed via intelligence µservice
n8n.nodes-base.stripe: oyatie.payments.charge_create  # routed via payments µservice
n8n.nodes-base.googleSheets: oyatie.sheets.cell_write
n8n.nodes-base.airtable: oyatie.airtable.record_create
n8n.nodes-base.postgres: oyatie.database.postgres_query
n8n.nodes-base.mysql: oyatie.database.mysql_query
n8n.nodes-base.mongodb: oyatie.database.mongodb_query
n8n.nodes-base.s3: oyatie.cloud_data_s3.put_object
# ... 350+ mappings auto-generated
```

The translator handles ~ 85-90 % of n8n nodes 1:1. Remaining 10-15 % require manual review:

- n8n-specific nodes (e.g., `Item Lists`, `Compare Datasets`) → typically replaceable by oyatie's `flow.merge` + custom JS code.
- Custom community nodes → require re-implementation as oyatie custom nodes (paid tenant_class).
- n8n Function / FunctionItem nodes (deprecated; many migrations have them) → convert to `oyatie.code.javascript`.

## Step 3 — Migrate credentials securely (≤ 3 days)

n8n credentials are encrypted with n8n's encryption key. Decrypt + re-encrypt with oyatie's secrets µservice.

```sh
# Decrypt n8n credentials (requires n8n encryption key)
n8n decrypt:credentials --input ./n8n-credentials.json --output ./decrypted.json --encryption-key $N8N_KEY

# Convert + import to oyatie secrets µservice
oya secrets migrate import-n8n \
    --input ./decrypted.json \
    --tenant acme-corp \
    --secrets-prefix migrated_n8n_

# Securely shred the plaintext file
shred -uv ./decrypted.json
```

The credentials are re-encrypted with oyatie's per-tenant KMS-resident keys. Workflows reference secrets by name (`{{secrets.migrated_n8n_stripe_api_key}}`).

## Step 4 — Migrate custom Code nodes (≤ 1-2 weeks)

n8n Code nodes contain JS that executes in a sandboxed VM. oyatie's `oyatie.code.javascript` node has a similar VM but slightly different I/O conventions.

n8n Code node:
```js
// n8n
return items.map(item => ({
  json: { ...item.json, transformed: true }
}));
```

oyatie equivalent:
```js
// oyatie
return inputs.items.map(item => ({
  ...item,
  transformed: true
}));
```

The differences:

- n8n's `items` is global; oyatie's is `inputs.items`.
- n8n wraps output in `{ json: ... }`; oyatie returns the JSON directly.
- n8n's `$node` global is replaced by oyatie's `context.nodes.<id>` accessor.

The translator auto-rewrites ~ 80 % of Code nodes. Complex code (with $node references, custom imports) requires manual review.

## Step 5 — Migrate trigger URLs (≤ 1 day)

n8n webhooks have URLs like `https://n8n.acme.com/webhook/abc123`. oyatie's are `https://workflow-engine.prod-syd-1.oyatie.local/v1/webhooks/<workflow-id>`.

Update upstream systems:

- Stripe webhook URLs → oyatie URLs.
- Shopify webhook subscriptions → oyatie URLs.
- Any custom integration that POSTs to n8n → oyatie URLs.

The migration tool generates an upstream-update checklist:

```sh
oya workflow-studio migrate generate-webhook-update-checklist \
    --tenant acme-corp \
    --output ./webhook-updates.md
```

The checklist lists every external system that needs updating + the old URL + the new URL.

## Step 6 — Test in shadow (≤ 2-4 weeks)

Run both n8n + oyatie in parallel:

- For each new webhook: dispatch to BOTH n8n + oyatie. Compare results.
- For each scheduled workflow: run on both schedules. Compare outputs.

Reconciliation report nightly:

```sh
oya workflow-studio reconcile \
    --source-a n8n \
    --source-b oyatie \
    --tenant acme-corp \
    --window-day 2026-05-20 \
    --report ./reconcile.json
```

Acceptable cutover drift: < 0.5 % per workflow type. Some drift is expected (timestamps, IDs, transient PSP latency).

## Step 7 — Cutover (≤ 1 d)

1. Update all upstream webhook URLs to oyatie.
2. Switch scheduled workflows from n8n to oyatie.
3. Disable n8n workflows (set them to inactive).
4. Emit cutover audit event.

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.workflow_studio.cut_over \
    --payload '{"from":"n8n","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

## Step 8 — n8n decommission (≤ 30-60 d post-cutover)

Keep n8n in read-only mode for ≥ 30 d for historical inspection. After 30 d:

- Export n8n execution history for archival.
- Decommission n8n instances + their PostgreSQL.
- Cancel n8n Cloud subscription if applicable.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Custom community nodes have no oyatie equivalent | High | Pre-audit community nodes; plan custom-node SDK implementation OR scope-cut |
| Code node rewrites introduce regressions | High | Shadow-run for ≥ 14 d; compare outputs |
| Webhook URL updates miss an upstream system | High | Generate checklist; verify all webhooks fire on oyatie before disabling n8n |
| n8n's "execution history" doesn't 1:1 migrate | Medium | Export to JSON archive; not queryable via oyatie tooling |
| Trigger schedules differ in timezone interpretation | Low | Validate every cron expression against both systems |
| Self-hosted n8n's PostgreSQL has tenant-specific schema | Low | n8n schemas are standard; backup before migration |
| User accounts + roles don't 1:1 migrate | Medium | Map n8n's role model to Cedar roles; document migration table |
| Workflow tags / folder organization is n8n-specific | Low | Re-create in oyatie's workflow library; ~ 1-2 days of curation |
| Cross-workflow dependencies (workflow A calling workflow B) | Medium | n8n's "Execute Workflow" node → oyatie's sub-workflow invocation pattern; rewrite |
| Encrypted credentials require careful handoff | High | Use the shred + re-encrypt flow; never log plaintext credentials |
