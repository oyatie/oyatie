---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-001
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-iac
related_adrs: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001 — Postgres + Citus post-store IaC

## Intent

Provision the Citus-on-Postgres cluster that backs `post-store`, `voting-engine`, `moderation-queue`, and `kb-article-store`. Per-tenant distribution column `tenant_id`; RLS forced; Patroni for HA.

## Scope

- 1 coordinator + N workers per region.
- Replication factor 2; sync standby; WAL retention 7 d (14 d L/XL).
- pgvector + pg_partman extensions enabled.
- CIS-Postgres benchmark applied.
- Per-tenant schema `community`; tables: `posts`, `revisions`, `threads`, `votes`, `moderation_actions`, `flags`, `kb_articles`, `kb_article_revisions`, `kb_attachments`, `subscriptions`, `mentions`.

## Deliverables

- `iac/helm/postgres/Chart.yaml` + `values.yaml` (this IP)
- `iac/kustomize/base/kustomization.yaml` overlay reference
- Citus schema migration in `src/migrations/0001_initial.sql`
- RLS policy per table
- Backup CronJob (WAL-G to S3)

## Acceptance

- Cluster up; coordinator + 2 workers minimum.
- RLS enforced on every table.
- WAL backup runs hourly; restore drill green.
- Citus distribution column = `tenant_id` on every sharded table.
- Patroni leader election green.

## Risks

- Citus rebalancer storm during onboarding spike → throttle.
- WAL-G S3 backpressure → multi-region S3 endpoint.

## Owner

axis-community + ops-iac.
