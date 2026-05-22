---
doc_class: MigrationPlaybook
microservice: translate
vendor: Smartling
date: 2026-05-20
doc_status: published
---

# Migration playbook — Smartling → oyatie translate

Audience: a localization team currently using Smartling for their TMS who is consolidating onto oyatie. Smartling supports a wide surface (TMS + MT + glossary + workflow); the playbook handles each surface.

## Mapping at-a-glance

| Smartling object | oyatie translate object | Field-level deltas |
|---|---|---|
| Project | Project (per tenant) | Smartling's "Project ID" → oyatie's `project_id`; "Source Locale" → `source_lang`; "Target Locale(s)" → `target_langs[]` (array; Smartling has one target per project; oyatie supports multi-target per project) |
| Smartling File (string-resource) | Source bundle | Smartling tracks per-file translation state; oyatie tracks per-segment in TM with file-namespace tag |
| Smartling Job | Bulk-translate task | Smartling Jobs bundle strings for human-review; oyatie's `bulk-translate` returns instantly with QE scoring |
| Smartling TM | TM | Direct TMX export from Smartling; import to oyatie |
| Smartling Glossary | Termbase | TBX export from Smartling; import to oyatie |
| Smartling Translator user | Translator role | Mapped per email |
| Smartling Reviewer user | Reviewer role | Mapped per email |
| Smartling MT Profile | Engine routing config | Smartling MT Profile maps to oyatie engine-routing-matrix per content-class |
| Smartling Quality Check | QE rule | Smartling Quality Checks (terminology, regex, etc.) map to oyatie post-translation QE rules |
| Smartling Context (URL, screenshot) | Context (URL, screenshot) | Direct map; preserves visual context for translators |
| Smartling Workflow | Workflow-engine workflow | Smartling's translator → reviewer → approver workflow → oyatie's `workflow-engine` template `translate.three-stage-review` |

## Step 1 — Inventory Smartling source (≤ 60 min for 500K-segment TM)

```sh
oya translate import inventory \
    --vendor smartling \
    --account-id <smartling-account-id> \
    --user-id <smartling-api-user-id> \
    --secret <smartling-api-secret> \
    --project <smartling-project-id> \
    --out inventory/smartling-<project-id>.yaml
```

The inventory enumerates: projects, files, jobs, TM entries (paged), glossary entries, translators, reviewers, workflows, MT profiles, quality checks.

Smartling's API is rate-limited at 60 req/min per account; the importer respects this (you'll see "rate-limit pause 60s" entries in the log if your TM is large).

Review the inventory for:

- Custom MT profiles: Smartling lets you mix MT vendors per (source-lang, target-lang, content-class); the importer converts these to oyatie engine-routing-matrix rows. Verify the conversion table.
- Smartling-specific quality checks: spell-check, leading/trailing whitespace, double-spaces, etc. — these become oyatie post-translation QE rules; some require manual config in oyatie because oyatie's QE rule grammar differs (oyatie uses a regex+predicate model; Smartling uses keyword-based).

## Step 2 — TM export from Smartling (≤ 30 min per 500K-segment TM)

Smartling's TMX export is the canonical way to extract the TM. Run from the Smartling web UI: Settings → TM → Export. Alternatively via API:

```sh
oya translate import smartling-tm-export \
    --project <smartling-project-id> \
    --target-locale <locale> \
    --out tm-export-<locale>.tmx
```

The export file is ~ 200 MB per 500K segments. Each file has one target locale; export per (source, target) pair.

## Step 3 — TM import to oyatie (≤ 20 min per 500K-segment file)

```sh
oya translate tm import \
    --tenant-id my-tenant \
    --source-lang en \
    --target-lang fr \
    --file tm-export-fr.tmx \
    --merge-strategy preserve-most-recent
```

`--merge-strategy preserve-most-recent` keeps the most-recent TM entry per source segment if duplicates exist. Other strategies: `preserve-highest-rated` (uses Smartling's confidence score), `preserve-all-as-variants` (keeps all variants; useful for tenants with multiple registers).

The import runs ~ 25K segments/min. Audit-chain emits `tm_bulk_import_completed` with the segment count + tenant + source + target.

## Step 4 — Glossary / termbase import (≤ 5 min)

```sh
oya translate import smartling-glossary-export \
    --project <smartling-project-id> \
    --out glossary-export.tbx

oya translate termbase import \
    --tenant-id my-tenant \
    --file glossary-export.tbx
```

Smartling's glossary export is TBX (ISO 30042) — same as oyatie's termbase format. Direct import; no conversion needed.

## Step 5 — User mapping (≤ 10 min)

```sh
oya translate import map-users \
    --inventory inventory/smartling-<project-id>.yaml \
    --strategy by-email \
    --out mapping/smartling-users-<project-id>.yaml
```

Edit the YAML to fix any unmapped users. Smartling supports per-language permission scoping (a user can be a translator for en→fr but reviewer for en→es); oyatie preserves this in the role mapping.

## Step 6 — Workflow mapping (≤ 30 min for typical custom workflows)

Smartling workflows are a state machine with stages: translator → reviewer → approver → published. oyatie's `workflow-engine` template `translate.three-stage-review` matches this exactly.

If your Smartling workflow has custom stages (e.g., "subject-matter-expert review between reviewer and approver"), you need to author a custom `workflow-engine` template. Use `oya workflow-engine template generate --base translate.three-stage-review --add-stage smr` and edit the generated template.

## Step 7 — MT profile / engine-routing conversion (≤ 20 min)

```sh
oya translate import convert-mt-profiles \
    --inventory inventory/smartling-<project-id>.yaml \
    --tenant-id my-tenant \
    --out engine-routing-<tenant>.yaml
```

The output is an oyatie engine-routing-matrix override per the migrated project's expectations. Review carefully — Smartling's "preferred MT" for a (pair, content-class) becomes oyatie's "preferred engine" for the same (pair, content-class). If Smartling preferred DeepL for en→fr legal, oyatie's routing should agree (and does by default since DeepL is en→fr legal's top engine).

Apply the override:

```sh
oya translate routing apply \
    --tenant-id my-tenant \
    --file engine-routing-<tenant>.yaml
```

## Step 8 — Shadow operation (≤ 14 days)

Before cutting over: run oyatie in shadow mode against Smartling-routed live traffic:

1. Smartling continues to serve translations in production.
2. oyatie receives the same source segments via a shadow webhook.
3. oyatie produces a translation but does not return it to the consumer.
4. The audit-chain emits `shadow_translation_executed` events with the side-by-side oyatie vs Smartling results.
5. The lane `oya-governance-translate-shadow-delta` analyses the delta and reports per-pair quality gaps.

After 14 days of green shadow: cut over.

## Step 9 — Cutover (≤ 7 days)

Per project:

1. Stop Smartling job-creation (no new strings sent to Smartling for translation).
2. Drain pending Smartling jobs (translator + reviewer + approver).
3. Switch the publishing pipeline to source translations from oyatie.
4. Run for 7 days; monitor for surprises (translation quality, latency, audit-chain coverage).
5. Decommission Smartling project after 30 days of green operation.

## Step 10 — Sunset evidence

```sh
oya translate import sunset-evidence \
    --vendor smartling \
    --project <smartling-project-id> \
    --target-tenant my-tenant \
    --out evidence/migrations/smartling-<project-id>.json
```

The evidence file is referenced by the `oya-governance-migration-evidence` lane and forms the auditor-visible record.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| TM segment quality differs after import | High | Shadow-mode review reveals delta; if quality regresses, adjust merge-strategy |
| Smartling-specific quality checks not auto-convertible | Medium | Manual QE rule authoring; allocate ~ 1 day per 20 unique quality checks |
| Custom Smartling workflow stages | High | Author custom workflow-engine template; pair-review before activating |
| Translator/reviewer email mismatch | Medium | User mapping edit; allocate ~ 30 min per 50 unmapped users |
| In-flight Smartling jobs at cutover | High | Drain Smartling jobs before cutover; do not cut over with active jobs |
| Smartling's "Style Guides" (free-text reference) | Medium | Copy into oyatie's termbase notes per term; not a 1:1 mapping |
| MT vendor cost increase post-migration | Low-Medium | Oyatie's engine-routing chooses cost-optimal; expect ~ 15-30% cost reduction for paid tenants with per_usage billing |
