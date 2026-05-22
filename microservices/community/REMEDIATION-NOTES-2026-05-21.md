---
doc_class: RemediationNotes
microservice: community
status: Accepted
date: 2026-05-21
owner_team: axis-community
wave: 15K
---

# Community Remediation Notes — 2026-05-21

## Wave 15K Network Retirement Merge

Directive source: `feedback_cell_standalone_network_merges_community_2026_05_21.md`.

Outcome: `network` retires as a standalone µservice. Its professional product
content is now owned by `community`.

## Migration Log

| Source responsibility from retired network | Community destination | Status |
|---|---|---|
| Resume / profile aggregates, export, verification | `professional-profile` BC in `PRD.md` + `ARCHITECTURE.md` | migrated |
| Connections graph + connection request lifecycle | `professional-graph` + `connection-request` BCs | migrated |
| InMail bridge | `inmail-bridge` BC through messenger | migrated |
| Endorsements + recommendations | `endorsement-engine` BC | migrated |
| Jobs, applications, recruiter-stub, ATS handoff | `jobs-recruiter` BC | migrated |
| Skill assessments | `skill-assessments` BC | migrated |
| Pages / events | `pages-events` BC and existing community events surface | migrated |
| LinkedIn-style engagement feed, sponsored posts, follower monetization | Explicitly forbidden in `PRD.md`, `ARCHITECTURE.md`, and `competitor-parity-matrix.md` | rejected |
| Wrong VPC Lattice / Cross-Cloud Network / Azure VWAN counterpart set | Excluded from community; infrastructure networking belongs to `cloud-network` | rehomed by doctrine |

## Counterpart Reset

Top-3 community counterparts are now:

1. Reddit
2. Teamblind
3. Handshake

Secondary anchors: LinkedIn Jobs, LinkedIn Profile, LinkedIn Recruiter. These
anchors apply only to profile, jobs, recruiter, InMail, endorsements,
recommendations, and professional graph. They do not authorize a LinkedIn-style
feed.

## Files Updated In This Pass

- `microservices/community/PRD.md`
- `microservices/community/ARCHITECTURE.md`
- `microservices/community/manifest.json`
- `microservices/community/competitor-parity-matrix.md`
- `microservices/community/REMEDIATION-NOTES-2026-05-21.md`
- `microservices/network/RETIRED.md`

## Cross-Reference Cleanup

Active references in `docs/decisions/`, `docs/operators/`, `docs/architecture/diagrams/`,
`docs/user-journeys/`, `specs/root-hub-pointers.json`, `specs/design-system/`, and
`specs/microservices/` were moved to either `microservices/community/` for the
professional-network product surface or `microservices/cloud-network/` for
infrastructure networking. Historical audit snapshots under `docs/architecture/`
retain old `microservices/network/` evidence paths as provenance, not current
navigation.

<!-- WAVE-15K-COMPLETION-REPORT
status: complete
scope: network-retired-into-community
directive_date: 2026-05-21
retired_path: microservices/network/
successor_path: microservices/community/
top_3_counterparts: Reddit / Teamblind / Handshake
secondary_counterparts: LinkedIn Jobs / LinkedIn Profile / LinkedIn Recruiter
migrated_content: profile, resume, connections, InMail, endorsements, recommendations, jobs, recruiter-stub, skill-assessments, pages, events
rejected_content: LinkedIn engagement feed, For-You-style algorithmic attention feed, sponsored-post promotion, influencer monetization via followers
retirement_protocol: ADR-0138 atomic-consolidation variant with retained RETIRED.md marker
commits: none
-->

## Wave 15-IP-substance scrub (2026-05-21)

Bucket: IP-BUCKET-G.

Scope: `community`.

Scrub finding: `IP-001` through `IP-015` were accepted implementation plans but still carried the mechanical short-form `Intent / Scope / Deliverables / Acceptance / Owner` stamp shape. They had useful seed nouns, yet lacked enough service-specific binding to the Wave 15K community product pillars, real contract files, policy files, SLO files, dashboards, runbooks, counterpart anchors, and ADR-0328 evidence boundaries.

Rewritten in place:

- `IP-001-postgres-citus-post-store-iac.md` — expanded with tenant-sharded Citus/Postgres schema, OpenTofu/ADR-0328 boundary, real OpenAPI/proto route and entity mappings, restore/runbook evidence, and Reddit/Teamblind/Handshake/LinkedIn subset storage closure.
- `IP-002-post-store-kernel-domain.md` — expanded with post kernel/domain types, anonymity-mode author reference, forbidden engagement-feed fields, policy fragment binding, and counterpart-domain closure.
- `IP-003-post-store-usecase-api.md` — expanded with concrete command/query/API surfaces, AsyncAPI event mapping, tenant/anonymity context, and REST/proto operation evidence.
- `IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md` — expanded with adapter/rest/worker/sdk/app boundaries, RLS session setup, NATS/audit/search events, SDK evidence, and executable counterpart closure.
- `IP-005-thread-tree-materialised-path.md` — expanded with LTREE path model, `ThreadTreeService`, reply/accepted-answer event mapping, deep-tree tests, and Reddit/Discourse/GitHub/Teamblind closure.
- `IP-006-voting-engine.md` — expanded with Postgres/Valkey source-of-truth split, idempotent votes, anti-brigade checks, ranking boundary, and vote/answer events.
- `IP-007-moderation-queue.md` — expanded with append-only moderation actions, Cedar decision capture, audit-chain seal reference, two-eyes behavior, and deanonymization incident boundary.
- `IP-008-kb-article-store-s3.md` — expanded with immutable KB revisions, attachment checksum/scan/object-lock flow, public-read policy, restore drill, and Notion/Zendesk/GitHub/Confluence closure.
- `IP-009-search-index-elasticsearch.md` — expanded with stale Elasticsearch correction toward ADR-COMM-0004 Meilisearch/Tantivy, indexed document kinds, redaction, reindex worker, and search counterpart closure.
- `IP-010-foundry-guardrails-moderation-bridge.md` — expanded with classifier advisory bridge, redacted features, fallback, DLQ/backlog behavior, and moderation runbook evidence.
- `IP-011-cedar-policy-fragments.md` — expanded with concrete action taxonomy, entity fields, anonymity/public/auditor/CI scope tests, and AWS Verified Permissions-style closure.
- `IP-012-openslo-grafana-dashboards.md` — expanded with seven service SLOs, dashboard bindings, low-cardinality labels, Alertmanager/Grafana OnCall routing, and performance counterpart closure.
- `IP-013-oya-vcs-promotion-readiness.md` — expanded with community-specific promotion blockers, observability eligibility verdict binding, rollback runbooks, and signed evidence requirements.
- `IP-014-hyperscaler-maturity-gate.md` — expanded with Reddit/Teamblind/Handshake/LinkedIn subset claim rules, Big-8/P0 elevation, forbidden feed claims, and evidence-vs-design separation.
- `IP-015-capacity-cost-chaos-drill.md` — expanded with product-semantic load/chaos/cost drills, runbook-linked scenarios, anonymity checks, and measured-evidence requirements.

Deleted as duplicative: none. The 15 files cover distinct substrate, policy, observability, promotion, maturity, and drill slices.

Preserved as already-substantive: all long `IP-journey-*` files, `IP-N-anonymous-fold-extraction.md`, and non-stamped service artifacts were not edited in this scrub.

Verification notes:

- Rewritten files now reference real service artifacts under `contracts/`, `policy/`, `slos/`, `dashboards/`, `runbooks/`, `catalog/`, `manifest.json`, `PRD.md`, `ARCHITECTURE.md`, and counterpart matrices.
- Rewritten files now name counterpart anchors including Reddit, Teamblind, Handshake, LinkedIn, GitHub Discussions, Zendesk Help Center, Notion, Confluence, Discourse, Grafana, Datadog, and AWS Verified Permissions where relevant.
- No duplicate IP was deleted because no two `community/IP-001..015` slices were 80 percent identical after conversion; their implementation ownership differs.

## Wave 15-journey-IP substance pass

Scope: `microservices/community/IP-journey-*.md` files over 200 lines.

- Journey IPs inventoried: 50
- Template-loop IPs detected: 50
- Rows rewritten into substantive, evidence-bound journey rows: 300
- Rows deleted as un-grounded generated loop residue: 3994
- Counterpart references added: 300
- Grounding artifacts: `contracts/openapi/community.yaml`, `contracts/asyncapi/community-events.yaml`, `contracts/proto/community.proto`, `policy/*.cedar`, `capabilities/*.yaml`, `slos/*.openslo.yaml`, `dashboards/*.json`.
- Follow-up: planned journey-specific endpoints/policies named inside the rewritten IPs still require contract-test implementation before promotion.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/community/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/community/IP-N-anonymous-fold-extraction.md`
- `microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md`
- `microservices/community/PRD.md`
- `microservices/community/decisions/ADR-COMM-0002-voting-engine-tie-breaking-and-decay.md`
- `microservices/community/feature-parity-matrix-2026-05-20.md`
- `microservices/community/iac/helm/community/templates/networkpolicy.yaml`
- `microservices/community/iac/helm/community/values.yaml`
- `microservices/community/performance-benchmark-numbers-2026-05-20.md`

Counterpart-fact preservations:

None.

Files renamed (git mv):

None.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set PRD target to manifest `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, and `runbooks/dr-failover.md` per ADR-0343, with HIPAA/KR-PIPA/SOC2/ISO27001/KR-CSAP floors cited. Rejected read-cache-only recovery because moderation, KB, and post authorship must preserve write evidence. Cost: active-active post/moderation replication, DR runbook ownership, and attachment restore drills.
- Capacity model: declared manifest `0.12 vCPU`, `256 MiB RAM`, `12 GiB storage`, Valkey/Postgres/outbound baselines, `per_request` scaling, Tier-3 placement, `pod_runtime_tier=2`, and `2..48` replica bounds per ADR-0340. Rejected a forum-read-only model because spam floods and moderation queues drive capacity under incident load. Cost: queue isolation and Citus/search shard split automation.
- Sustainability + cost attribution: required `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on posts, votes, moderation, KB, search, attachments, and federation audit rows per ADR-0344. Rejected the older ADR-0174 aggregate sustainability wording alone because regulatory reporting needs per-capability cost and emissions. Cost: metering on classifier/search/storage/federation paths.
- API versioning posture: adopted `YYYY-MM-DD` carrier triplet, SDK semver, N=3/180-day support, tenant pinning, and ADR-0145 internal-mesh exemption per ADR-0342. Rejected ad hoc ActivityPub/REST version splits because tenant moderation and help-center integrations need one public carrier policy. Cost: versioned gateway compatibility tests.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345. ADR-0337 was not added because community has search/federation evidence, not an Iceberg warehouse write path.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.12 vCPU, 256 MiB RAM, 12 GB storage per active tenant; connections valkey=3, postgres=3, outbound_http=4; scaling_dimension=per_request; cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity declaration plus ADR-0248 cellular class.
- Rejected: template-stamped values copied from another service; community is sized around request and moderation spikes; Tier-1 was rejected because moderation data is product data, not a shared tenant-data substrate.
- Cost: cell sizing and autoscaler budgets must reserve this per-tenant baseline before admitting more tenants.

### Block 2: dr
- Values: rto_p99_seconds=3600, rpo_p99_seconds=300, multi_region_active_active=true, backup_substrate=postgres_wal_g, object_storage_versioned, valkey, audit_chain_merkle_seal, failover_runbook=runbooks/dr-failover.md.
- ADR: ADR-0343 plus compliance-pack floors; HIPAA/us-healthcare floors drive the 1h/5m baseline where applicable.
- Rejected: looser 24h PCI-only recovery because this service can serve healthcare or sensitive tenant workflows.
- Cost: warm cross-region replication and quarterly drill evidence are required for the declared runbook.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; evidence=microservices/community/PRD.md, microservices/community/IP-005-thread-tree-materialised-path.md, microservices/community/IP-007-moderation-queue.md, microservices/community/runbooks/moderation-queue-clear.md.
- ADR: ADR-0338 runtime tiering; ADR-0340/ADR-0248 co-variance with cell_placement_class=Tier-3.
- Rejected: weaker runtime class that would contradict the documented tenant-data or first-party-app surface.
- Cost: runtime placement, nodepool capacity, and incident severity inherit this tier.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; supports_per_tenant_pinning=true.
- ADR: ADR-0342 date-versioned public APIs with per-tenant pinning.
- Rejected: internal-only exemption because this service has public OpenAPI, AsyncAPI, and proto surfaces.
- Cost: at least three supported public API windows and migration docs for any future breaking change.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=postgresql, valkey, cedar, kafka, opentelemetry, opensearch; oss_stewardship_class_overrides=[].
- ADR: ADR-0345 and /specs/oss-stewardship-registry.json registry authority.
- Rejected: local stewardship overrides because the registry default class is sufficient for each declared upstream.
- Cost: SBOM and CVE-response evidence must trace this service to each upstream owner team.

### Block 6: iac_module_invocations
- Values: oyatie-as-cloud-provider/k8s-namespace-bootstrap@v1, oyatie-as-cloud-provider/secrets-bootstrap@v1.
- ADR: ADR-0339 shared IaC module library.
- Rejected: unpinned local wrapper-only IaC because module reuse and pinning are the admission surface.
- Cost: module pins must be advanced deliberately when cloud-iac publishes new primitives.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: `D4-BUCKET-2`.
- Doctrine source: ADR-0337..0345 selective propagation by trigger match; this section records only matched IPs.
- Manifest gap: `manifest.json#dr` is absent, so DR sections preserve compliance-pack floors without inventing service RTO/RPO targets.

| IP | Trigger(s) | Required sections | Source evidence | Manifest gaps |
| --- | --- | --- | --- | --- |
| `microservices/community/IP-001-postgres-citus-post-store-iac.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-002-post-store-kernel-domain.md` | A | API Versioning (per ADR-0342) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | none |
| `microservices/community/IP-003-post-store-usecase-api.md` | A, C | API Versioning (per ADR-0342); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | none |
| `microservices/community/IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-005-thread-tree-materialised-path.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-006-voting-engine.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-007-moderation-queue.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-008-kb-article-store-s3.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-009-search-index-elasticsearch.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-010-foundry-guardrails-moderation-bridge.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-011-cedar-policy-fragments.md` | A | API Versioning (per ADR-0342) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | none |
| `microservices/community/IP-012-openslo-grafana-dashboards.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-013-oya-vcs-promotion-readiness.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-014-hyperscaler-maturity-gate.md` | B | DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-015-capacity-cost-chaos-drill.md` | B, C | DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-N-anonymous-fold-extraction.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j05-whistleblower-intake.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j06-securedrop-intake.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j100-pack-rollout-first-action.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j108-talent-and-trust-surface.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j109-talent-and-trust-surface.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j110-talent-and-trust-surface.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j111-talent-and-trust-surface.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j112-talent-and-trust-surface.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j113-talent-and-trust-surface.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j116-developer-reputation-channel.md` | A, B, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/community/IP-journey-j119-verified-financier-reputation.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j129-transparency-report.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j130-whistleblower-channel.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j132-mass-hiring-posting.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j133-outplacement-and-cohort-channel.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j134-cross-tenant-staffing-engagement.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j135-whistleblower-mode-internal.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j138-corporate-audit-hr-reporting-channel.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j145-job-application-cross-tenant.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j147-cohort-sub-tenant-and-referrals.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j148-consumer-impact-reputation.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j149-worker-reputation-and-support.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j15-responsible-disclosure-intake.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j150-paid-fan-tier.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j17-tor-friendly-anonymous-presence.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j18-child-safety-report-intake.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j23-seller-reputation.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j24-buyer-review.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j30-comments-and-appeals.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j31-reply-thread-bridge.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j32-teamblind-anonymous-post.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j49-review-routing.md` | A, B, D | API Versioning (per ADR-0342); DR posture (per ADR-0343); Pod runtime tier (per ADR-0338) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing; manifest.json#pod_runtime_tier missing |
| `microservices/community/IP-journey-j52-review-and-reputation.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j56-handshake-application.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j63-researcher-network.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j65-community-export.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j76-community-surface.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j79-community-surface.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j84-community-surface.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j89-community-surface.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j90-community-surface.md` | A, B | API Versioning (per ADR-0342); DR posture (per ADR-0343) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j91-us-msb-mtl-overlay.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j92-br-lgpd-us-parent-dsar.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j93-in-dpdpa-rbi-overlay.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j94-sox404-public-company-controls.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j95-iso27001-soc2-annual-audit.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j96-ksa-uae-mena-onboarding.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j97-sg-pdpa-mas-tenant.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j98-au-privacy-apra-cps234.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
| `microservices/community/IP-journey-j99-multi-pack-conflict-resolution.md` | A, B, C | API Versioning (per ADR-0342); DR posture (per ADR-0343); Sustainability emission (per ADR-0344) | microservices/community/contracts/openapi/community.yaml, microservices/community/manifest.json | manifest.json#dr missing |
