---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-012-search-and-cedar-filter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-nextest, search-policy-test, search-people-slo]
---

# IP-012: Search and Cedar filter

## A. Problem
People and content search must be useful without exposing blocked, deleted, moderated, minor-protected, or cross-context content.

## B. Approach
Implement the cataloged Meilisearch adapter plus planned search kernel/domain/usecase/worker/sdk layers. Index only permitted fields, re-check Cedar at query time, and record redaction/suppression evidence.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-community-social-search-adapter-meilisearch.yaml` | Existing adapter anchor. |
| `src/crates/oya-community-social-search-{kernel,domain,usecase,api,adapter-meilisearch,worker,sdk}/` | Planned family named by PRD/IP/catalog. |
| `policy/public-read.cedar`, `policy/content-policy.cedar`, `policy/minor-protection.cedar` | Search visibility controls. |
| `slos/search-people-latency.openslo.yaml` | Search latency SLO. |

## D. Ordered implementation steps
1. Define search document, query, result, redaction, and index-version types.
2. Build people and content indexing workers from profile/post events.
3. Exclude PHI, redacted PII, moderated, and non-visible content from index documents.
4. Re-run Cedar filters on every result page.
5. Test blocked author, muted relation, minor profile, deleted post, and tenant-context cases.
6. Add index rebuild and drift detection hooks.
7. Wire search SLO and dashboard evidence.

## E. Acceptance
- `cargo nextest run -p oya-community-social-search-adapter-meilisearch` passes.
- Search tests prove hidden content is neither indexed nor returned.
- `slos/search-people-latency.openslo.yaml` resolves.
- `buck2 build //:quality-lane-registry-authority-check # lane=data-residency --microservice social` passes.
- Query examples in `contracts/openapi/social.yaml` match implementation behavior.

## F. Evidence
- PRD FR-11: `PRD.md`.
- Catalog: `catalog/oya-community-social-search-adapter-meilisearch.yaml`.
- Policies: `policy/public-read.cedar`, `policy/content-policy.cedar`, `policy/minor-protection.cedar`.
- Contracts: `contracts/openapi/social.yaml`.

## G. Counterpart comparison
X, Instagram, TikTok, Snapchat, Threads, Bluesky, Mastodon, and LinkedIn all make search/discovery a core surface. Oyatie must match discoverability while applying Cedar filtering and redaction where counterparts rely on opaque ranking or admin policy.

## H. Foundation delivery expansion
- Deliverable detail: search documents separate public fields, tenant-visible fields, redacted fields, and index metadata.
- Deliverable detail: index workers consume profile, post, moderation, deletion, and visibility events.
- Deliverable detail: query-time Cedar checks run after index lookup and before result emission.
- Deliverable detail: result pages record redaction and suppression counts.
- Deliverable detail: index version tracks schema, policy version, tenant, and pack.
- Deliverable detail: rebuild path can drop and recreate tenant/context indexes safely.
- Deliverable detail: search contract examples include people, posts, tags, and restricted result cases.
- Deliverable detail: Slack workspace search is direct counterpart pressure for policy-filtered community search.

## I. Acceptance expansion
- Acceptance detail: indexing tests must exclude PHI, redacted PII, deleted posts, moderated posts, and hidden profiles.
- Acceptance detail: query tests must rerun Cedar filters for every page.
- Acceptance detail: blocked author tests must return no content and no existence leak.
- Acceptance detail: minor-profile tests must suppress protected users from inappropriate discovery.
- Acceptance detail: index rebuild tests must preserve audit evidence and version bumps.
- Acceptance detail: OpenAPI examples must match returned redaction fields.
- Acceptance detail: search latency SLO must resolve if the file exists.
- Acceptance detail: Slack, LinkedIn, X, and Mastodon comparisons must map to search behavior and visibility evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for search adapter and workers.
- Evidence detail: capture data-residency and content-policy gate outputs.
- Evidence detail: capture OpenAPI validation for search examples.
- Evidence detail: cite `catalog/oya-community-social-search-adapter-meilisearch.yaml`.
- Evidence detail: cite `policy/public-read.cedar`, `policy/content-policy.cedar`, and `policy/minor-protection.cedar`.
- Evidence detail: cite `contracts/openapi/social.yaml`.
- Evidence detail: cite Slack as workplace/community search pressure requiring strict Cedar filtering.
