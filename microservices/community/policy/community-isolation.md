---
doc_class: Policy
template_id: TPL-POLICY
microservice: community
status: Accepted
classification: INTERNAL_ONLY
policy_class: tenant-isolation
date: 2026-05-17
owner_team: ops-security + axis-community
related_adrs: [ADR-0018, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/community/policy/tenant-scope.cedar
  - microservices/community/policy/data-residency.md
  - microservices/community/threat-model.md
doc_status: published
---

# Community-isolation policy: per-tenant community boundary

## Purpose

Enforce that every community surface object (post, reply, vote, KB article, attachment, flag, moderation action, search index, mention resolution) is bound to a single tenant, and that no path — read or write — can cross tenant boundaries.

## Invariants

### Invariant CI-01 — Single tenant_id per row

Every table in the `community.*` Postgres schema has a non-null `tenant_id` column. Citus distribution column is `tenant_id` for all sharded tables.

### Invariant CI-02 — RLS deny-by-default

Every table has Row-Level Security enabled with a `FORCE` policy that denies all access unless the session variable `app.current_tenant_id` matches the row's `tenant_id`. The session variable is set by the adapter-postgres layer at connection acquisition time from the JWT claim — never from request body.

```sql
ALTER TABLE community.posts ENABLE ROW LEVEL SECURITY;
ALTER TABLE community.posts FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON community.posts
  USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
```

Same pattern applies to: `posts`, `revisions`, `threads`, `votes`, `moderation_actions`, `flags`, `kb_articles`, `kb_article_revisions`, `kb_attachments`, `subscriptions`, `mentions`.

### Invariant CI-03 — Per-tenant Elasticsearch index

Each tenant gets a dedicated index name: `community-<tenant_id_short>-<bc>`. The gateway binds the index name to the JWT tenant claim; query templates never accept a tenant_id from the request body.

### Invariant CI-04 — Per-tenant Redis namespace

Redis keys are prefixed with `c:<tenant_id_short>:`. Redis ACLs scope clients to `~c:<tenant_id_short>:*` per tenant connection pool.

### Invariant CI-05 — Per-tenant S3 prefix + bucket

KB attachments live at `s3://oya-community-attachments-<region>/<tenant_id>/<article_id>/<sha256>`. IAM role per tenant scopes prefix.

### Invariant CI-06 — Cedar deny on cross-tenant

`policy/tenant-scope.cedar` has an explicit `forbid` block (defence-in-depth) for every action when `principal.tenant_id != resource.tenant_id`.

### Invariant CI-07 — Mention resolution scoped

The mention-resolver searches only within the JWT-bound tenant. A `@user` mention to a non-tenant-member returns "unresolved", never resolves to a cross-tenant identity.

### Invariant CI-08 — Audit-chain seal carries tenant_id

Every sealed event includes `tenant_id`; replay verification asserts the seal matches the asserting tenant.

### Invariant CI-09 — Moderation actions never cross tenants

A moderator can only act on resources within their own tenant. Cross-tenant moderation is impossible at the Cedar layer.

### Invariant CI-10 — Worker per-tenant fair-share

Worker pools (search-reindex, foundry-guardrails-bridge, audit-chain-seal) use per-tenant token-bucket so one noisy tenant cannot starve others.

## Verification

- Daily CI gate runs `community-isolation-check` which:
  - Asserts every table has RLS enabled + FORCE'd.
  - Asserts every ES index follows `community-<tenant_id_short>-<bc>`.
  - Asserts Redis ACL scope matches namespace pattern.
  - Asserts S3 bucket policy includes per-tenant prefix conditions.
- Quarterly red-team: attempt cross-tenant read with crafted JWT; expected outcome is Cedar deny + audit-chain alert.

## Breach Response

Any cross-tenant access incident is a P0 per `incident-response.md`. Mandatory steps:
1. Page on-call within 1 min.
2. Disable affected gateway path within 5 min.
3. Per-pack regulator notification within 72 h (GDPR Art. 33), 24 h (KR PIPA), 60 days (HIPAA).
4. Post-mortem within 5 business days; ADR for any structural change.

## Related Policies

- `policy/tenant-scope.cedar` (read scope)
- `policy/ci-scope.cedar` (CI agent scope)
- `policy/auditor-scope.cedar` (auditor read scope)
- `policy/public-read.cedar` (anonymous-read for public spaces)
- `policy/data-residency.md` (per-region binding)
