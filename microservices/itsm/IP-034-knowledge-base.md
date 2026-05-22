# IP-034 ITSM knowledge-base

Service: itsm
ChangeSet scope: microservices/itsm/IP-034-knowledge-base.md
Counterparts displaced: ServiceNow Knowledge Management (KCS v6), Jira Service Management Knowledge Base, Freshservice Solutions
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0255, ADR-0263, ADR-0328

## Objective
- O-001: Author and serve tenant-isolated knowledge articles with KCS v6 conformance (Knowledge-Centered Service); each problem-record close emits a candidate article when knowledge_gap is recorded.
- O-002: Serve RAG-powered retrieval via the intelligence µservice substrate; per-tenant fine-tuning is the default.
- O-003: Surface articles to the self-service portal (IP-031), agent workspace (IP-033), AI virtual agent (IP-035), and mobile ITSM (IP-032).

## Article model
- AM-001: Article ID is `tenant_id` + monotonic seq; the slug is tenant-scoped only.
- AM-002: Article kinds: `how_to`, `known_error`, `faq`, `runbook_excerpt`.
- AM-003: Lifecycle states: `draft → review → published → retired`.
- AM-004: KCS evolve-loop: every viewed article tracks `useful_to_close` for downstream coverage scoring.

## Authoring flow
- AF-001: Draft author = agent or problem-record closer.
- AF-002: Peer review = a different agent in the same support group.
- AF-003: Publish event emits `kb.article.published` to audit-chain.

## RAG retrieval
- RR-001: Per-tenant embedding model lives in the intelligence µservice; the ITSM KB does not embed.
- RR-002: Retrieval gateway returns `top_k` results with relevance score and per-article provenance.
- RR-003: Re-ranker is intelligence-µservice-side; ITSM only consumes ranked results.

## Tenant invariants
- T-001: Article reads are scoped to `principal.tenant_id == resource.tenant_id`.
- T-002: Cross-tenant search is permanently forbidden; even oyatie internal tenant cannot cross over.

## Cedar policy
- C-001: `policy/knowledge-base-authorization.cedar` default-denies; permits for `kb.draft`, `kb.review`, `kb.publish`, `kb.retire`, `kb.read`.

## Tenant-class behavior
- TC-001: demo_trial: KB cap 50 articles per ADR-0331.
- TC-002: paid: KB cap unlimited; per-usage meter is `kb_articles_active`.

## Acceptance evidence
- E-001: openslo: kb_retrieval_p95_ms ≤ 600 (paid); ≤ 1500 (demo).
- E-002: cargo test for the article lifecycle state machine.
- E-003: replay test for the publish event.

## Out of scope
- OoS-001: External public KB (marketing-blog-style content) is owned by the brand surface µservices, not ITSM.

## Wave 15-IP-substance addendum
This addendum converts the short prior capability stub into a cold-start buildable IP without changing the original capability intent.

### Real source anchors
- Primary capability: knowledge base.
- REST/API anchor: kb article search/publish route.
- Policy anchor: policies/local-knowledge-publish-approval.cedar.
- SLO/dashboard anchor: kb search latency and publish audit.
- Counterpart pressure: ServiceNow ITSM, Jira Service Management, and Freshservice all expose this class of ITSM surface; Oyatie closes the gap with tenant scope, Cedar, audit-chain evidence, and pack overlays.

### Implementation detail that must exist before promotion
- Define the command DTO with tenant_id, principal_id, audience_type, purpose, data_class, and audit_event_class fields.
- Bind the command to a Capability or an adjacent bounded-context action instead of adding a free-form route.
- Evaluate Cedar before any repository write, external provider call, workflow-engine dispatch, or audit success event.
- Emit an ADR-0263 audit event for success and a distinct denial event for policy, budget, residency, or capacity refusal.
- Carry home_cell, jurisdiction_code, and pack ids through the request context before data leaves the home cell.
- Use existing ITSM source files as the first implementation surface: src/domain/mod.rs, src/usecase/mod.rs, src/adapter/mod.rs, and tests/integration.rs.
- Keep source-system identifiers from ServiceNow, Jira, or Freshservice as aliases only; they cannot authorize Oyatie actions.
- Preserve demo_trial and paid behavior from manifest.json; demo caps must be tested separately from paid behavior.
- Add dashboard evidence before calling the feature production-ready.
- Add rollback that disables this capability without disabling incident open, change approval, SLA recompute, or audit publication.

### Acceptance evidence to add
- Unit or integration test proving the clean allow path succeeds for a synthetic tenant.
- Negative test proving cross-tenant access is denied before mutation.
- Negative test proving missing pack/residency context fails closed where the capability touches protected data.
- Contract test or schema validation for the REST/event/RPC surface used by this capability.
- Audit replay check proving one success event is emitted for each successful command.
- Dashboard or OpenSLO check proving latency/error-budget evidence is available.
- Counterpart parity row explaining the ServiceNow/Jira/Freshservice behavior being displaced.
- Residual-risk note if a referenced runtime module, route, or Cedar entity is not yet implemented.

### Counterpart comparison
| Counterpart | Why this IP is not a clone |
|---|---|
| ServiceNow ITSM | knowledge article management under ServiceNow is replaced by Oyatie tenant-scoped policy and audit evidence. |
| Jira Service Management | The JSM equivalent is treated as capability pressure, not as project-key authority. |
| Freshservice | Freshservice-style convenience remains gated by pack residency, DealSet where applicable, and explicit rollback. |

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-034-knowledge-base.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/itsm/IP-034-knowledge-base.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
