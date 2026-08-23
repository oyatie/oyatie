---
purpose: Oyatie — Commit Message Standard
doc_status: published
---

# Oyatie — Commit Message Standard

> **Owner:** `council-architecture`. Validator: retired `./bin/oya verify` pre-push hook + per-PR CI lane.

## 1. Format (Conventional Commits)

```
<type>(<scope>): <subject>

<body>

<footer>
```

## 2. Type (closed enumeration)

| Type | Use |
|---|---|
| `feat` | New feature / capability |
| `fix` | Bug fix |
| `refactor` | Internal restructure (no behavior change) |
| `perf` | Performance improvement |
| `docs` | Documentation only |
| `test` | Test additions / fixes |
| `chore` | Tooling / build / no product impact |
| `ci` | CI lane changes |
| `revert` | Revert prior commit |
| `migrate` | Flat-crates migration phase per ADR-0015 |
| `rebrand` | Brand-consolidation batch for Oyatie per ADR-0017 |

## 3. Scope (closed enumeration)

Per axis or cross-cutting:

| Scope | Use |
|---|---|
| `tenant` / `identity` / `audit` / `eventing` / `og` / `policy` / `dub` / `cell` / `pack` | Foundation (per ADR-0001..0010) |
| `foundry` / `foundry-mcp` / `foundry-adapter-{anthropic,openai,gemini}-{api,subscription}` / `foundry-eval` / `foundry-sandbox` / `foundry-rag` / `foundry-engineering-platform` / `foundry-model-{train,serve,vision,speech}` / `foundry-robotics` | Foundry (ADR-0020..0027) |
| `saas` / `workspace-{mail,calendar,docs,sheets,slides,drive,meet,chat,forms,sites,tasks}` | SaaS + Workspace |
| `vertical-<name>` (corporate / healthcare / industrial / logistics / fintech / legal / retail / education / public-sector / hospitality / construction / real-estate / agriculture / food) | Vertical |
| `cloud-{compute-{vm,k8s,functions},storage-{object,block,file,archive},network-{vpc,lb,dns,cdn},iam,kms,billing,observability,finops,dcops}` | Cloud |
| `search-{crawler,parser,index-inverted,index-vector,rank,query,serp,rag,safety}` | Search |
| `ads-{auction,targeting,attribution,console,publisher}` / `analytics-{event,warehouse,dp,streaming}` | Ads + Analytics |
| `pack-{kr,jp,us,eu,in,br,ksa,ae,au,sg,...}` | Regional packs |
| `docs-consolidated` / `docs-standards` / `docs-runbooks` / `docs-decisions` | Doc-only |
| `repo` | Repo-wide / cross-cutting |

## 4. Subject

- ≤ 72 chars
- Imperative mood ("add" not "adds" or "added")
- No trailing period
- Lowercase first letter

## 5. Body

- 72-char wrap
- "What" + "why" (not "how" — code shows that)
- Reference ADRs from new pack (e.g. "Per ADR-0008 §2.2.1, ...")
- Reference incident / mistake-and-fixes ledger entry if applicable

## 6. Footer

- `Refs #<issue>` — soft reference
- `Closes #<issue>` — auto-close on merge
- `Blocks #<issue>` — this PR blocks linked issue
- `Blocked-by #<issue>` — cannot proceed until resolved
- `BREAKING CHANGE: <description>` — for major-bump per ADR-0037
- `Co-Authored-By: <name> <email>` — for paired work / agent attribution
- `Signed-off-by: <committer>` — per signed-commits ADR

## 7. Examples

```
feat(foundry-adapter-anthropic-api): add Anthropic API provider adapter

Implements the ProviderAdapter trait for Anthropic Claude API
mode. Supports streaming, tool-use, prompt-caching, and cost
ceiling enforcement per ADR-0020.

Refs #1234
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
```

```
fix(workspace-mail): correct DKIM signature on outbound mail in pack-kr

Outbound IP binding was using global pool instead of pack-specific
warm pool, causing reputation degradation. Per ADR-0049
cross-region replication and ADR-0010 regional pack architecture.

Closes #5678
```

```
migrate(repo): phase 5 service-runtime crate moves

Moves runtimes from services/* to crates/oyatie-*-runtime per ADR-0015
flat-crates target.

Refs #1458
```

## 8. Anti-patterns

- "Fix bug" / "Update code" / "WIP" — never (per CI gate)
- Trailing period in subject — never
- Past-tense imperative ("added" / "fixed") — never
- Multi-axis scope — split commit
- Mixed types (feat + fix in one commit) — split

## 9. Sources
Conventional Commits 1.0.0; CLAUDE.md commit-protocol; ADR-0019 doc-catalog protocol.
