---
doc_class: MigrationPlaybook
microservice: community
vendor: Discourse (self-hosted or Discourse Hosting)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Discourse → oyatie community

Audience: a team running Discourse (self-hosted or Discourse Hosting) for their community forum. Drivers: TeamBlind-class verified-anonymous + cross-tenant federation + LinkedIn-class reputation + LLM-assisted moderation + audit-chain non-repudiation + sovereign-pack residency.

## Why this migration matters

Discourse is excellent at:

- Forum + threading + voting + tags + categories.
- Self-host option (zero per-user cost).
- Active plugin ecosystem.

oyatie community adds:

- TeamBlind-class verified-anonymous posting (Discourse has anonymous but not verified-corporate-email anonymous).
- Cross-tenant federation.
- LinkedIn-class reputation + Handshake-class job postings (paid advanced).
- Native LLM moderation via the `intelligence` µservice (Discourse has Perspective API + Akismet plugins; both are external).
- Cryptographic audit-chain integration.
- Sovereign-pack residency.

The trade-off: smaller plugin ecosystem at launch. Discourse's ~ 1 000+ plugins exceeds oyatie's built-in feature set in some niche areas (e.g., specific badges, gamification widgets).

## Step 1 — Inventory the Discourse estate (≤ 1 week)

```bash
# From the Discourse admin panel + the Discourse API:
# 1. Export categories (forum boards)
discourse-export categories > ./discourse-categories.json

# 2. Export topics (= oyatie posts)
discourse-export topics --since 2020-01-01 > ./discourse-topics.json

# 3. Export posts (= oyatie comments + post bodies)
discourse-export posts --since 2020-01-01 > ./discourse-posts.json

# 4. Export users
discourse-export users > ./discourse-users.json

# 5. Export user badges / trust levels
discourse-export user-badges > ./discourse-user-badges.json

# 6. Export tags
discourse-export tags > ./discourse-tags.json
```

Document:

- Category count + per-category visibility (public, private, login-required).
- Topic + post counts; typical depth (2-3 levels).
- User count + trust-level distribution.
- Active plugins + their replacement plan.
- Active moderators + their role mapping.

Typical mid-size Discourse: 30-100 categories, 100k-1M topics, 5M-100M posts, 50k-500k users.

## Step 2 — Map Discourse concepts to oyatie (≤ 1 week)

| Discourse concept | oyatie community equivalent |
|---|---|
| Category | Board |
| Topic | Post |
| Post (reply) | Comment |
| Tag | Tag (1:1) |
| Trust level (0-4) | Reputation score (mapping table) |
| User | User principal |
| Group | Cedar role |
| Badge | Reputation milestone |
| Plugin (Solved, Voting, Polls) | Built-in feature OR custom-node SDK (paid advanced) |

Trust level mapping:

| Discourse trust level | oyatie reputation range |
|---|---|
| TL0 (new) | 0-10 |
| TL1 (basic) | 11-50 |
| TL2 (member) | 51-500 |
| TL3 (regular) | 501-5000 |
| TL4 (leader) | 5001+ |

## Step 3 — Data migration (≤ 2-6 weeks per 50M posts)

```sh
oya community migrate import-discourse \
    --tenant acme-corp \
    --categories-input ./discourse-categories.json \
    --topics-input ./discourse-topics.json \
    --posts-input ./discourse-posts.json \
    --users-input ./discourse-users.json \
    --tags-input ./discourse-tags.json \
    --user-badges-input ./discourse-user-badges.json \
    --throttle-rate 10000-posts-per-sec
```

The migration:

1. Creates oyatie boards from Discourse categories.
2. Creates oyatie users from Discourse users.
3. Imports topics → oyatie posts.
4. Imports posts → oyatie comments (with parent_id mapping).
5. Migrates tags + applies to imported posts.
6. Computes initial reputation scores from Discourse trust levels.

Backfill rate ~ 10k posts/sec on paid. 50M posts → ~ 1.5 hours.

Verify post-import counts:

```sql
SELECT board_id, count(*) AS post_count
FROM tenant_acme_corp.post
GROUP BY board_id
ORDER BY post_count DESC;
```

Cross-check against the Discourse export's per-category counts. Acceptable drift: 0 % (entity-level integrity must match).

## Step 4 — User account migration + SSO (≤ 1-2 weeks)

Discourse users with passwords → oyatie users with SSO + password-reset email.

The migration:

1. Creates oyatie principals from Discourse users (preserve username + email).
2. Sends a password-reset email to each user (mandatory; oyatie doesn't import Discourse password hashes).
3. After reset, the user logs in to oyatie + sees their historic content.

Alternatively, SSO integration: if the tenant has SAML / OIDC IdP, configure the Discourse IdP → oyatie IdP via the `cloud-iam` µservice. Users sign in via IdP; no password reset needed.

## Step 5 — Plugin replacement (≤ 4-12 weeks)

Discourse plugins → oyatie equivalents:

| Discourse plugin | oyatie equivalent |
|---|---|
| Discourse Solved | Built-in (mark-post-as-solved feature) |
| Discourse Voting | Built-in (upvote/downvote) |
| Discourse Polls | Built-in (poll widget) |
| Discourse Chat | `messenger` µservice integration |
| Discourse AI / Perspective API | `intelligence` µservice LLM moderation |
| Discourse Reactions (emoji) | Built-in (reactions feature) |
| Discourse Calendar | `calendar` µservice integration |
| Discourse Sitemap | Built-in |
| Discourse Subscriptions (paid memberships) | Built-in tenant_class-gated boards (paid advanced) |
| Discourse Footnotes | Built-in (markdown extensions) |

Custom plugins → oyatie custom-node SDK (paid advanced) OR tenant-application-layer code.

## Step 6 — Shadow run + cutover (≤ 4-8 weeks)

Run BOTH Discourse + oyatie in parallel. New posts go to oyatie; existing Discourse content remains.

After ≥ 4 weeks of clean parallel operation:

```sh
# Redirect the community URL
# DNS / load-balancer flip from discourse.acme.com → community.acme.com (oyatie)

oya audit emit \
    --tenant acme-corp \
    --event-class governance.community_substrate.cut_over \
    --payload '{"from":"discourse","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

Discourse stays read-only for historical access.

## Step 7 — Discourse decommission (≤ 90-180 d post-cutover)

After ≥ 90 d:

- Export final Discourse state for archival.
- Decommission Discourse hosting + database.
- Cancel Discourse Hosting subscription if applicable.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Discourse plugins have no oyatie equivalent | High | Pre-audit; plan custom-node implementation OR scope-cut |
| User passwords don't migrate (security best practice) | Medium | Send password-reset emails; SSO if available |
| Discourse trust-level mapping not 1:1 | Low | Document the mapping table; users may need to "earn back" their level on the new system (acceptable for most communities) |
| URL changes break inbound links | High | Set up 301 redirects from Discourse URLs to oyatie URLs; preserve URL structure where possible |
| Discourse-specific features (e.g., Solved, polls) need user training | Low | Provide migration FAQ + screenshot tour |
| Search index quality differs (Discourse uses PostgreSQL FTS; oyatie uses Elasticsearch) | Low | Re-build oyatie search index post-import; verify search quality on top-100 historical queries |
| Notification preferences don't 1:1 migrate | Low | Reset all users to default notification preferences; users can re-customize |
| Discourse trust-level-based moderation roles need re-mapping | Medium | Map TL3+ → Cedar `community::moderator` role |
| Plugin-stored data (e.g., subscriptions) requires custom export | Medium | Per-plugin migration path; some plugins lack export APIs |
| Discourse Theme + CSS customizations need re-applying | Medium | oyatie has theme + CSS support; manually re-apply (theme migration not auto) |
| Multi-language content: ensure UTF-8 + collation preserved | Low | Validate Unicode integrity post-import on sampled posts |
