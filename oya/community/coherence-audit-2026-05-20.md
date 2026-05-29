# community µservice ownership-coherence audit — 2026-05-20

Audit owner: solo Codex audit lane.
Target µservice: `community`.
Target path: `/Users/jasonlee/oyatie/microservices/community/`.
Deliverable set: coherence audit, feature parity matrix, performance benchmark numbers.
Retired deliverable: capability-profile deltas, dropped by 2026-05-20 tier-retirement directive.
Current batch counterparts: Discourse, Circle, Vanilla Forums.
Scope caveat: chat history later records a Wave 15K re-audit direction for community after network merge; this audit keeps the explicit current batch scope and flags the later scope as a future re-audit question.
Deployment-context assumption: all six canonical contexts remain in scope until service-local evidence proves a context is not applicable.
Canonical multi-context anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-1750`.
Canonical OCI Always Free anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1846-1848`.
Canonical OpenTofu anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2307`.
Canonical forbidden-IaC anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2464-2494`.
Canonical nine-dimension anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3829-3859`.
Canonical OS anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3950-3999`.
Canonical Rust-strict anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4014-4080`.
Canonical audit stop anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4214-4230`.
Machine-readable deployment anchor: `specs/master-plan-sequencing.json:704-746`.
Machine-readable IaC anchor: `specs/master-plan-sequencing.json:747-775`.
Machine-readable OS anchor: `specs/master-plan-sequencing.json:777-815`.
Machine-readable language anchor: `specs/master-plan-sequencing.json:817-855`.
Machine-readable OCI Always Free anchor: `specs/master-plan-sequencing.json:857-867`.
Brief-template multi-context anchor: `docs/standards/brief-template.md:666-688`.
Brief-template OpenTofu anchor: `docs/standards/brief-template.md:809-835`.
Brief-template OS anchor: `docs/standards/brief-template.md:967-1005`.
Brief-template Rust anchor: `docs/standards/brief-template.md:1125-1163`.
Brief-template anti-scaffold anchor: `docs/standards/brief-template.md:1727-1790`.
Tenant-class adoption memory anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:1-45`.
Tenant-class replacement anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-142`.
Ownership directive anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-18`.
Ownership artifact list anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:18-47`.
Ownership contradiction bar anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:49-63`.
Chat taxonomy anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:3`.
Chat future-reaudit anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16613-16619`.
Inventory command evidence: `find microservices/community -type f | sort` returned 202 files.
Line-read evidence: `wc -l $(find microservices/community -type f | sort)` returned 43266 total file-lines.
Tier scan evidence: `rg -n "\b(demo_trial|paid|paid advanced|paid compliance-pack)\b" microservices/community` returned 56 exact references.
Tenant-class scan evidence: `rg -n "tenant_class|demo_trial|revenue_share|\bpaid\b" microservices/community` found no `tenant_class`, no `demo_trial`, no `revenue_share`, and only product/payment uses of `paid`.
Forbidden-language scan evidence: no backend files matching `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, `*.fs`, or `*.fsx` were present under the service path.

## §1 Purpose

This document audits whether `community` is internally coherent as one owned µservice.
The audit treats the service path, not a single PRD or ADR, as the unit of truth.
The audit reads product purpose, contracts, ADRs, implementation plans, runbooks, policy fragments, SLOs, IaC, onboarding, FAQ, migration, benchmark, and reference-implementation artifacts together.
The audit evaluates the nine required dimensions from ADR-0328 D-20.
The audit adds tier-retirement checks required by the 2026-05-20 amendment.
The audit does not produce the retired fourth deliverable.
The audit does not edit existing service artifacts.
The audit does not touch other µservices.
The audit does not claim runtime readiness.
The audit does produce Wave 14 aggregation-ready findings with severity and citations.
The product purpose is clear in the PRD: a tenant-scoped end-user community surface for spaces, channels, threads, Q&A, KB articles, polls, and events that consumes substrates rather than acting as a substrate.
The direct purpose evidence is `microservices/community/PRD.md:10-15`.
The day-one audience model is B2C plus B2B, with one codebase and audience-mode UX templates rather than separate services.
The direct audience evidence is `microservices/community/PRD.md:84-138`.
The current PRD benchmark set includes Discourse but does not include Circle or Vanilla Forums, even though this batch's counterpart bar requires all three.
The direct benchmark evidence is `microservices/community/PRD.md:187-230`.
The current PRD is broader than classic forums: it names Discord, Reddit, Discourse, Stack Overflow, Notion, GitHub Discussions, Zendesk, ActivityPub, and more.
That breadth is valid product ambition, but it increases the risk that this service becomes a suite unless dependency boundaries stay crisp.
The architecture file classifies `community` as a product consumer and states it may call substrates but must not create product-to-product synchronous dependencies.
The direct architecture evidence is `microservices/community/ARCHITECTURE.md:197-205`.
The manifest dependency list includes several sibling products, including `meet`, `shorts`, `calendar`, `mail`, `messenger`, `drive`, `sites`, and `marketplace`.
The direct manifest evidence is `microservices/community/manifest.json:400-420`.
That creates a coherence question: those dependencies may be event or substrate-like handoffs, but the manifest does not distinguish synchronous product dependency from asynchronous integration.
The service has rich planning depth: 202 files and 43266 file-lines under the service path.
The presence of many files is not enough; the audit focuses on contradiction and canonical alignment.
The service has a strong contract baseline: OpenAPI 3.2.0, AsyncAPI, and proto3 files exist.
The OpenAPI route surface covers spaces, posts, replies, voting, answer acceptance, flags, moderation, and KB articles.
The direct OpenAPI evidence is `microservices/community/contracts/openapi/community.yaml:1-230`.
The service has meaningful SLOs; the post-create SLO defines a 250 ms good threshold and a 99% objective.
The direct SLO evidence is `microservices/community/slos/post-create-latency.openslo.yaml:1-44`.
The service has meaningful moderation architecture; ADR-COMM-0001 fixes a chain-of-responsibility pipeline, Cedar evaluation at every hop, and per-hop audit-chain seals.
The direct moderation evidence is `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:56-91`.
The service has a search ADR that chooses Meilisearch primary and Tantivy fallback and rejects Elasticsearch/OpenSearch at M02.
The direct search evidence is `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:66-89`.
The service also has older capacity, cost, and failure-mode artifacts that still refer to Elasticsearch.
The direct drift evidence is `microservices/community/capacity-model.md:48-58`, `microservices/community/cost-budget.md:29-50`, and `microservices/community/failure-modes.md:30-34`.
The service has no README at the service root.
The missing README is a P2 documentation gap because the ownership directive names README as part of the expected per-service surface.
The service has no `supported-oses.json`.
The missing OS manifest is a P2 canonical-direction gap under ADR-0328 D-20.
The service has IaC files, but not in the six canonical OpenTofu context directories.
The IaC path evidence is `microservices/community/iac/helm/community/`, `microservices/community/iac/kustomize/`, and `microservices/community/iac/terraform/grafana-rbac.tf`.
The Terraform file is active IaC and directly conflicts with OpenTofu-only canonical direction.
The Terraform evidence is `microservices/community/iac/terraform/grafana-rbac.tf:1-12`.
The service has no `iac/oci-guest/always-free/` profile.
The missing profile is important because `demo_trial` infrastructure should fit the OCI Always Free profile.
The tier-retirement directive requires all existing demo_trial/paid/paid advanced/paid compliance-pack language to become Wave 15J retirement candidates.
The service has 56 exact tier references.
The service has no canonical `tenant_class` adoption.
The service uses the word `paid` for paid memberships and paid fan journeys, but those are product/payment semantics, not the replacement tenant-class model.

## §2 Inventory

Inventory count: 202 files under `microservices/community/`.
Inventory file-lines: 43266 total.
Inventory root artifact: `microservices/community/ARCHITECTURE.md`.
Inventory root artifact: `microservices/community/AUDIT-FINDINGS-2026-05-18.json`.
Inventory root artifact: `microservices/community/IP-001-postgres-citus-post-store-iac.md`.
Inventory root artifact: `microservices/community/IP-002-post-store-kernel-domain.md`.
Inventory root artifact: `microservices/community/IP-003-post-store-usecase-api.md`.
Inventory root artifact: `microservices/community/IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md`.
Inventory root artifact: `microservices/community/IP-005-thread-tree-materialised-path.md`.
Inventory root artifact: `microservices/community/IP-006-voting-engine.md`.
Inventory root artifact: `microservices/community/IP-007-moderation-queue.md`.
Inventory root artifact: `microservices/community/IP-008-kb-article-store-s3.md`.
Inventory root artifact: `microservices/community/IP-009-search-index-elasticsearch.md`.
Inventory root artifact: `microservices/community/IP-010-foundry-guardrails-moderation-bridge.md`.
Inventory root artifact: `microservices/community/IP-011-cedar-policy-fragments.md`.
Inventory root artifact: `microservices/community/IP-012-openslo-grafana-dashboards.md`.
Inventory root artifact: `microservices/community/IP-013-oya-vcs-promotion-readiness.md`.
Inventory root artifact: `microservices/community/IP-014-hyperscaler-maturity-gate.md`.
Inventory root artifact: `microservices/community/IP-015-capacity-cost-chaos-drill.md`.
Inventory root artifact: `microservices/community/IP-N-anonymous-fold-extraction.md`.
Inventory root artifact: `microservices/community/IP-journey-j05-whistleblower-intake.md`.
Inventory root artifact: `microservices/community/IP-journey-j06-securedrop-intake.md`.
Inventory root artifact: `microservices/community/IP-journey-j100-pack-rollout-first-action.md`.
Inventory root artifact: `microservices/community/IP-journey-j108-talent-and-trust-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j109-talent-and-trust-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j110-talent-and-trust-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j111-talent-and-trust-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j112-talent-and-trust-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j113-talent-and-trust-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j116-developer-reputation-channel.md`.
Inventory root artifact: `microservices/community/IP-journey-j119-verified-financier-reputation.md`.
Inventory root artifact: `microservices/community/IP-journey-j129-transparency-report.md`.
Inventory root artifact: `microservices/community/IP-journey-j130-whistleblower-channel.md`.
Inventory root artifact: `microservices/community/IP-journey-j132-mass-hiring-posting.md`.
Inventory root artifact: `microservices/community/IP-journey-j133-outplacement-and-cohort-channel.md`.
Inventory root artifact: `microservices/community/IP-journey-j134-cross-tenant-staffing-engagement.md`.
Inventory root artifact: `microservices/community/IP-journey-j135-whistleblower-mode-internal.md`.
Inventory root artifact: `microservices/community/IP-journey-j138-corporate-audit-hr-reporting-channel.md`.
Inventory root artifact: `microservices/community/IP-journey-j145-job-application-cross-tenant.md`.
Inventory root artifact: `microservices/community/IP-journey-j147-cohort-sub-tenant-and-referrals.md`.
Inventory root artifact: `microservices/community/IP-journey-j148-consumer-impact-reputation.md`.
Inventory root artifact: `microservices/community/IP-journey-j149-worker-reputation-and-support.md`.
Inventory root artifact: `microservices/community/IP-journey-j15-responsible-disclosure-intake.md`.
Inventory root artifact: `microservices/community/IP-journey-j150-paid-fan-tier.md`.
Inventory root artifact: `microservices/community/IP-journey-j17-tor-friendly-anonymous-presence.md`.
Inventory root artifact: `microservices/community/IP-journey-j18-child-safety-report-intake.md`.
Inventory root artifact: `microservices/community/IP-journey-j23-seller-reputation.md`.
Inventory root artifact: `microservices/community/IP-journey-j24-buyer-review.md`.
Inventory root artifact: `microservices/community/IP-journey-j30-comments-and-appeals.md`.
Inventory root artifact: `microservices/community/IP-journey-j31-reply-thread-bridge.md`.
Inventory root artifact: `microservices/community/IP-journey-j32-teamblind-anonymous-post.md`.
Inventory root artifact: `microservices/community/IP-journey-j49-review-routing.md`.
Inventory root artifact: `microservices/community/IP-journey-j52-review-and-reputation.md`.
Inventory root artifact: `microservices/community/IP-journey-j56-handshake-application.md`.
Inventory root artifact: `microservices/community/IP-journey-j63-researcher-network.md`.
Inventory root artifact: `microservices/community/IP-journey-j65-community-export.md`.
Inventory root artifact: `microservices/community/IP-journey-j76-community-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j79-community-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j84-community-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j89-community-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j90-community-surface.md`.
Inventory root artifact: `microservices/community/IP-journey-j91-us-msb-mtl-overlay.md`.
Inventory root artifact: `microservices/community/IP-journey-j92-br-lgpd-us-parent-dsar.md`.
Inventory root artifact: `microservices/community/IP-journey-j93-in-dpdpa-rbi-overlay.md`.
Inventory root artifact: `microservices/community/IP-journey-j94-sox404-public-company-controls.md`.
Inventory root artifact: `microservices/community/IP-journey-j95-iso27001-soc2-annual-audit.md`.
Inventory root artifact: `microservices/community/IP-journey-j96-ksa-uae-mena-onboarding.md`.
Inventory root artifact: `microservices/community/IP-journey-j97-sg-pdpa-mas-tenant.md`.
Inventory root artifact: `microservices/community/IP-journey-j98-au-privacy-apra-cps234.md`.
Inventory root artifact: `microservices/community/IP-journey-j99-multi-pack-conflict-resolution.md`.
Inventory root artifact: `microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md`.
Inventory root artifact: `microservices/community/PRD.md`.
Inventory root artifact: `microservices/community/backfill-replay.md`.
Inventory benchmark artifact: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md`.
Inventory capability artifact: `microservices/community/capabilities/bug-bounty-submission.yaml`.
Inventory capability artifact: `microservices/community/capabilities/handshake-mode.yaml`.
Inventory capability artifact: `microservices/community/capabilities/linkedin-mode.yaml`.
Inventory capability artifact: `microservices/community/capabilities/moderate-action.yaml`.
Inventory capability artifact: `microservices/community/capabilities/post-create.yaml`.
Inventory capability artifact: `microservices/community/capabilities/reddit-mode.yaml`.
Inventory capability artifact: `microservices/community/capabilities/securedrop-press-source.yaml`.
Inventory capability artifact: `microservices/community/capabilities/teamblind-mode.yaml`.
Inventory capability artifact: `microservices/community/capabilities/vote-cast.yaml`.
Inventory capability artifact: `microservices/community/capabilities/whistleblower-submission.yaml`.
Inventory retirement artifact: `microservices/community/tenant_class model in ADR-0330`.
Inventory root artifact: `microservices/community/capacity-model.md`.
Inventory catalog group: `microservices/community/catalog/` contains 45 layer/catalog YAML files for KB, moderation, post-store, search, thread-tree, and voting engine slices.
Inventory root artifact: `microservices/community/competitor-parity-matrix.md`.
Inventory root artifact: `microservices/community/compliance.md`.
Inventory contract artifact: `microservices/community/contracts/asyncapi/community-events.yaml`.
Inventory contract artifact: `microservices/community/contracts/openapi/community.yaml`.
Inventory contract artifact: `microservices/community/contracts/proto/community.proto`.
Inventory root artifact: `microservices/community/cost-budget.md`.
Inventory dashboard artifact: `microservices/community/dashboards/moderation-queue-depth.json`.
Inventory dashboard artifact: `microservices/community/dashboards/post-throughput.json`.
Inventory dashboard artifact: `microservices/community/dashboards/vote-rate.json`.
Inventory decision artifact: `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md`.
Inventory decision artifact: `microservices/community/decisions/ADR-COMM-0002-voting-engine-tie-breaking-and-decay.md`.
Inventory decision artifact: `microservices/community/decisions/ADR-COMM-0003-kb-article-versioning-and-fork-merge.md`.
Inventory decision artifact: `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md`.
Inventory decision artifact: `microservices/community/decisions/ADR-COMM-0005-graph-of-discussions-and-replies.md`.
Inventory decision artifact: `microservices/community/decisions/README.md`.
Inventory root artifact: `microservices/community/dpia.md`.
Inventory root artifact: `microservices/community/failure-modes.md`.
Inventory FAQ artifact: `microservices/community/faqs/community-engineer-faq.md`.
Inventory IaC artifact: `microservices/community/iac/helm/community/Chart.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/deployment.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/hpa.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/networkpolicy.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/pdb.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/prometheusrule.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/service.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/templates/servicemonitor.yaml`.
Inventory IaC artifact: `microservices/community/iac/helm/community/values.yaml`.
Inventory IaC artifact: `microservices/community/iac/kustomize/base/kustomization.yaml`.
Inventory IaC artifact: `microservices/community/iac/kustomize/overlays/pack-kr/kustomization.yaml`.
Inventory IaC artifact: `microservices/community/iac/terraform/grafana-rbac.tf`.
Inventory root artifact: `microservices/community/incident-response.md`.
Inventory root artifact: `microservices/community/manifest.json`.
Inventory migration artifact: `microservices/community/migration-playbooks/from-discourse.md`.
Inventory root artifact: `microservices/community/multi-region.md`.
Inventory onboarding artifact: `microservices/community/onboarding/community-engineer-first-week.md`.
Inventory policy artifact: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`.
Inventory policy artifact: `microservices/community/policy/anonymity-mode-identity-anchored.cedar`.
Inventory policy artifact: `microservices/community/policy/anonymity-mode-persona-anchored.cedar`.
Inventory policy artifact: `microservices/community/policy/anonymity-mode-pseudonymous.cedar`.
Inventory policy artifact: `microservices/community/policy/auditor-scope.cedar`.
Inventory policy artifact: `microservices/community/policy/ci-scope.cedar`.
Inventory policy artifact: `microservices/community/policy/community-isolation.md`.
Inventory policy artifact: `microservices/community/policy/data-residency.md`.
Inventory policy artifact: `microservices/community/policy/public-read.cedar`.
Inventory policy artifact: `microservices/community/policy/tenant-scope.cedar`.
Inventory reference artifact: `microservices/community/reference-implementations/post-comment-vote-rust-sdk.md`.
Inventory runbook artifact: `microservices/community/runbooks/coordinated-spam-attack-response.md`.
Inventory runbook artifact: `microservices/community/runbooks/kb-attachment-restore.md`.
Inventory runbook artifact: `microservices/community/runbooks/moderation-queue-clear.md`.
Inventory runbook artifact: `microservices/community/runbooks/moderator-decision-appeal-protocol.md`.
Inventory runbook artifact: `microservices/community/runbooks/post-mass-deletion.md`.
Inventory runbook artifact: `microservices/community/runbooks/search-rebuild.md`.
Inventory runbook artifact: `microservices/community/runbooks/spam-flood-throttle.md`.
Inventory runbook artifact: `microservices/community/runbooks/verified-anonymous-deanonymization-incident.md`.
Inventory runbook artifact: `microservices/community/runbooks/vote-anomaly.md`.
Inventory scorecard artifact: `microservices/community/scorecards/overrides.json`.
Inventory root artifact: `microservices/community/sdk-plan.md`.
Inventory SLO artifact: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`.
Inventory SLO artifact: `microservices/community/slos/feed-render-latency.openslo.yaml`.
Inventory SLO artifact: `microservices/community/slos/kb-article-publish-latency.openslo.yaml`.
Inventory SLO artifact: `microservices/community/slos/moderation-action-latency.openslo.yaml`.
Inventory SLO artifact: `microservices/community/slos/post-create-latency.openslo.yaml`.
Inventory SLO artifact: `microservices/community/slos/search-query-latency.openslo.yaml`.
Inventory SLO artifact: `microservices/community/slos/vote-cast-latency.openslo.yaml`.
Inventory root artifact: `microservices/community/threat-model.md`.
Inventory tutorial artifact: `microservices/community/tutorials/configure-anonymous-board-and-moderation.md`.
Inventory omission: no root `microservices/community/README.md` was present.
Inventory omission: no `microservices/community/supported-oses.json` was present.
Inventory omission: no `microservices/community/iac/oyatie-public-cloud/` directory was present.
Inventory omission: no `microservices/community/iac/guest-on-aws/` directory was present.
Inventory omission: no `microservices/community/iac/oci-guest/` directory was present.
Inventory omission: no `microservices/community/iac/oci-guest/always-free/` directory was present.
Inventory omission: no `microservices/community/iac/on-prem/` directory was present.
Inventory omission: no `microservices/community/iac/colo/` directory was present.
Inventory omission: no `microservices/community/iac/oyatie-iaas/` directory was present.
Inventory omission: no `microservices/community/src/` directory was present.
Inventory omission: no `microservices/community/tests/` directory was present.

## §3 9-dimension audit

### §3.1 Dimension 1 — internal coherence

D1 verdict: partial pass with P1/P2 gaps.
D1 evidence: PRD classifies `community` as a tenant-scoped product surface, not a substrate, at `microservices/community/PRD.md:10-15`.
D1 evidence: PRD says one codebase serves multiple audience modes via policy and UX templates at `microservices/community/PRD.md:122-138`.
D1 evidence: Architecture says the product may call substrates but must not create product-to-product synchronous dependencies at `microservices/community/ARCHITECTURE.md:197-205`.
D1 contradiction: manifest `depends_on_microservices` includes several sibling products without classifying sync versus async handoff at `microservices/community/manifest.json:400-420`.
D1 finding: dependency semantics must be split into substrate dependencies, async product handoffs, and forbidden synchronous product calls.
D1 evidence: PRD performance names Meilisearch and Tantivy at `microservices/community/PRD.md:864-879`.
D1 evidence: ADR-COMM-0004 chooses Meilisearch primary and Tantivy fallback at `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:66-89`.
D1 contradiction: capacity and cost artifacts still size Elasticsearch at `microservices/community/capacity-model.md:48-58` and `microservices/community/cost-budget.md:29-50`.
D1 contradiction: failure-mode FM-08 still names Elasticsearch shard corruption at `microservices/community/failure-modes.md:30-34`.
D1 finding: search backend doctrine is coherent in PRD and ADR, but older operational artifacts are stale.
D1 evidence: ADR-COMM-0001 requires per-hop audit seals and calls missing seals P0 at `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:83-91`.
D1 contradiction: failure-mode FM-12 says the tenant write path stays open during audit-chain seal lag at `microservices/community/failure-modes.md:34`.
D1 finding: seal-lag behavior needs a risk split between low-risk writes, high-risk moderation writes, and P0 stop conditions.
D1 evidence: incident response lists P0 cross-tenant leak and mass deletion response at `microservices/community/incident-response.md:21-26`.
D1 evidence: incident response references missing scenario runbooks at `microservices/community/incident-response.md:61-70`.
D1 finding: P0 scenario names exist, but several referenced runbook files are absent.
D1 evidence: OpenAPI covers create, edit, delete, vote, accepted answer, flag, moderation, and KB routes at `microservices/community/contracts/openapi/community.yaml:36-220`.
D1 finding: the OpenAPI route shape is strong for forum/Q&A/KB scope.
D1 gap: OpenAPI does not expose tenant-class or billing-meter emission semantics; this is consistent with gateway-owned tenant_class request behavior but still needs meter-event documentation.

### §3.2 Dimension 2 — outbound cross-references

D2 verdict: partial pass with stale or unresolvable references.
D2 evidence: PRD cites ADR-0242 through ADR-0246 and ADR-COMM files in frontmatter at `microservices/community/PRD.md:25-52`.
D2 evidence: ARCHITECTURE cites many local capability, contract, Cedar, SLO, and runbook artifacts in its inventory at `microservices/community/ARCHITECTURE.md:22-38`.
D2 evidence: incident response references runbooks by filename at `microservices/community/incident-response.md:61-70`.
D2 gap: `runbooks/cross-tenant-bleed.md` is referenced but absent.
D2 gap: `runbooks/mention-reconcile.md` is referenced but absent.
D2 gap: `runbooks/dsr-cascade-resume.md` is referenced but absent.
D2 evidence: failure modes also reference missing runbooks at `microservices/community/failure-modes.md:35-42`.
D2 evidence: PRD uses `cloud-iac | Helm + Terraform registry` for the IaC handoff at `microservices/community/PRD.md:1038-1039`.
D2 gap: that reference conflicts with the OpenTofu-only direction in ADR-0328 D-16.
D2 evidence: manifest links only one bounded context named `community` at `microservices/community/manifest.json:6-64`.
D2 gap: PRD defines 15 bounded contexts at `microservices/community/PRD.md:986-1002`, so manifest bounded-context coverage is underspecified.
D2 evidence: manifest capabilities list only `moderate-action`, `post-create`, and `vote-cast` at `microservices/community/manifest.json:88-107`.
D2 gap: service capability files include 10 capability YAML files, so manifest capability coverage is incomplete.
D2 finding: outbound references are numerous and often concrete, but several are stale, absent, or too coarse.

### §3.3 Dimension 3 — substance bar

D3 verdict: mostly pass, with targeted P2 substance gaps.
D3 evidence: the PRD has service-specific feature matrices and NFR targets at `microservices/community/PRD.md:187-230` and `microservices/community/PRD.md:864-900`.
D3 evidence: ADR-COMM-0001 gives concrete moderation topology, handler names, Cedar attachment points, and audit events at `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md:56-91`.
D3 evidence: ADR-COMM-0004 gives backend candidates, selection, fallback policy, and residency behavior at `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:51-89`.
D3 evidence: OpenAPI has concrete route shapes for spaces, posts, replies, voting, accepted answer, flags, moderation, and KB articles at `microservices/community/contracts/openapi/community.yaml:19-220`.
D3 evidence: SLO files exist and define measurable Prometheus queries, as shown by `microservices/community/slos/post-create-latency.openslo.yaml:20-44`.
D3 substance gap: ARCHITECTURE begins with a generated-anchor warning saying all stub sections must be expanded during content-pass review at `microservices/community/ARCHITECTURE.md:1-3`.
D3 substance gap: ARCHITECTURE uses placeholder names such as `community.moderate_action_2` through `_5` at `microservices/community/ARCHITECTURE.md:140-144`.
D3 substance gap: IP-journey generated files include repeated buildability rows and should be reviewed for real acceptance criteria rather than line volume.
D3 finding: the service has enough artifacts to be buildable in slices, but some generated surfaces still need human consolidation.
D3 finding: the service should keep the rich PRD/ADR/OpenAPI/SLO content and retire scaffold-like duplicated IP boilerplate in a later cleanup wave.

### §3.4 Dimension 4 — canonical-direction alignment

D4 verdict: fail until tier retirement, tenant-class adoption, OpenTofu context modules, OS manifest, and Terraform drift are corrected.
D4 evidence: canonical contexts are six, per `specs/master-plan-sequencing.json:704-746`.
D4 evidence: every supported context needs `iac/<context>/` or N/A evidence, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2195-2224`.
D4 gap: service has Helm, Kustomize, and Terraform directories but none of the six required context directories.
D4 evidence: canonical IaC engine is OpenTofu, not Terraform, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2249`.
D4 evidence: service has an active Terraform file at `microservices/community/iac/terraform/grafana-rbac.tf:1-12`.
D4 finding: `iac/terraform/grafana-rbac.tf` is a P1 OpenTofu violation, not just a wording issue.
D4 evidence: canonical OS manifest is required at `microservices/<name>/supported-oses.json`, per `docs/standards/brief-template.md:967-1005`.
D4 gap: `microservices/community/supported-oses.json` is absent.
D4 evidence: Rust-strict backend policy allows docs/config/contracts but forbids Python, JavaScript app logic, TypeScript app logic, Ruby, Java, Go, and related backend languages at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4014-4080`.
D4 pass: forbidden-language file scan under this service found no forbidden backend language source files.
D4 evidence: tenant-class replacement model is not feature gating; it gates usage caps, time, support, SLO, and compliance eligibility at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:106-113`.
D4 gap: tenant_class semantics are absent from the service path.
D4 evidence: the no-tier memory says retired customer-class ladders do not exist and must be retired at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:1-45`.
D4 gap: exact tier language remains in 56 service-local references listed in §3.4.T.

### §3.4.T Tenant-class adoption candidates

Tier candidate 001: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:13` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 002: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:21` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 003: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:22` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 004: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:35` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 005: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:36` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 006: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:49` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 007: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:62` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 008: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:75` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 009: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:88` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 010: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:89` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 011: `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md:99` uses `paid`, `paid advanced`, and `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 012: `microservices/community/migration-playbooks/from-discourse.md:25` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 013: `microservices/community/migration-playbooks/from-discourse.md:77` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 014: `microservices/community/migration-playbooks/from-discourse.md:112` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 015: `microservices/community/migration-playbooks/from-discourse.md:151` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 016: `microservices/community/migration-playbooks/from-discourse.md:154` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 017: `microservices/community/onboarding/community-engineer-first-week.md:12` uses `demo_trial` and `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 018: `microservices/community/onboarding/community-engineer-first-week.md:25` uses `demo_trial`; Wave 15J retirement candidate; severity P2.
Tier candidate 019: `microservices/community/onboarding/community-engineer-first-week.md:102` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 020: `microservices/community/onboarding/community-engineer-first-week.md:104` uses `paid` and `demo_trial`; Wave 15J retirement candidate; severity P2.
Tier candidate 021: `microservices/community/onboarding/community-engineer-first-week.md:284` uses `demo_trial`; Wave 15J retirement candidate; severity P2.
Tier candidate 022: `microservices/community/onboarding/community-engineer-first-week.md:290` uses `paid`, `paid advanced`, and `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 023: `microservices/community/tutorials/configure-anonymous-board-and-moderation.md:15` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 024: `microservices/community/tutorials/configure-anonymous-board-and-moderation.md:324` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 025: `microservices/community/faqs/community-engineer-faq.md:40` uses `paid` and `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 026: `microservices/community/faqs/community-engineer-faq.md:44` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 027: `microservices/community/faqs/community-engineer-faq.md:60` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 028: `microservices/community/faqs/community-engineer-faq.md:71` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 029: `microservices/community/faqs/community-engineer-faq.md:105` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 030: `microservices/community/faqs/community-engineer-faq.md:107` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 031: `microservices/community/faqs/community-engineer-faq.md:113` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 032: `microservices/community/faqs/community-engineer-faq.md:141` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 033: `microservices/community/faqs/community-engineer-faq.md:143` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 034: `microservices/community/faqs/community-engineer-faq.md:149` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 035: `microservices/community/tenant_class model in ADR-0330:15` uses `demo_trial`; Wave 15J retirement candidate; severity P2.
Tier candidate 036: `microservices/community/tenant_class model in ADR-0330:50` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 037: `microservices/community/tenant_class model in ADR-0330:52` uses `demo_trial`; Wave 15J retirement candidate; severity P2.
Tier candidate 038: `microservices/community/tenant_class model in ADR-0330:89` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 039: `microservices/community/tenant_class model in ADR-0330:91` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 040: `microservices/community/tenant_class model in ADR-0330:125` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 041: `microservices/community/tenant_class model in ADR-0330:129` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 042: `microservices/community/tenant_class model in ADR-0330:131` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 043: `microservices/community/tenant_class model in ADR-0330:147` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 044: `microservices/community/tenant_class model in ADR-0330:150` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 045: `microservices/community/tenant_class model in ADR-0330:163` uses `demo_trial`, `paid`, `paid advanced`, and `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 046: `microservices/community/tenant_class model in ADR-0330:170` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 047: `microservices/community/tenant_class model in ADR-0330:171` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 048: `microservices/community/tenant_class model in ADR-0330:172` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 049: `microservices/community/tenant_class model in ADR-0330:173` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 050: `microservices/community/tenant_class model in ADR-0330:174` uses `paid`; Wave 15J retirement candidate; severity P2.
Tier candidate 051: `microservices/community/tenant_class model in ADR-0330:175` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 052: `microservices/community/tenant_class model in ADR-0330:176` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 053: `microservices/community/tenant_class model in ADR-0330:177` uses `paid advanced`; Wave 15J retirement candidate; severity P2.
Tier candidate 054: `microservices/community/tenant_class model in ADR-0330:178` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 055: `microservices/community/tenant_class model in ADR-0330:180` uses `paid compliance-pack`; Wave 15J retirement candidate; severity P2.
Tier candidate 056: `microservices/community/reference-implementations/post-comment-vote-rust-sdk.md:148` uses `paid`; Wave 15J retirement candidate; severity P2.
Adjacent tier-vocabulary candidate: `microservices/community/manifest.json:343-347` uses `capability_profiles`; schema-review candidate for Wave 15J even though it does not use the exact retired labels.
Adjacent tier-vocabulary candidate: `microservices/community/manifest.json:423` uses `criticality_tier`; schema-review candidate to ensure this means operational criticality and not capability profileing.
Adjacent tier-vocabulary candidate: `microservices/community/PRD.md:7-22` uses `tier`, `tier_subtype`, `tier_certified_at`, and `tier_promotion_history`; schema-review candidate after platform taxonomy is updated.
Adjacent tier-vocabulary candidate: `microservices/community/capacity-model.md:38-70` uses XS/S/M/L/XL sizing tiers; rename to capacity classes if Wave 15J wants all tier vocabulary scrubbed.
Adjacent tier-vocabulary candidate: `microservices/community/cost-budget.md:15-39` uses sizing and cost tiers; rename to capacity classes if Wave 15J wants all tier vocabulary scrubbed.

### §3.4.C Tenant-class adoption gaps

Tenant-class verdict: absent.
Tenant-class scan result: no `tenant_class` token appears in the service path.
Tenant-class scan result: no `demo_trial` token appears in the service path.
Tenant-class scan result: no `revenue_share` token appears in the service path.
Tenant-class scan result: `paid` appears, but only as product/payment semantics.
Tenant-class evidence: PRD reserves payments and paid memberships to `payments` at `microservices/community/PRD.md:1223-1227`.
Tenant-class evidence: journey `IP-journey-j150-paid-fan-tier.md` owns a paid fan product slice, not tenant-class state.
Tenant-class gap: service does not declare usage meters for demo_trial caps.
Tenant-class gap: service does not declare paid tenant unlimited-with-billing semantics.
Tenant-class gap: service does not declare revenue_share event emission for marketplace/B2C operator surfaces.
Tenant-class gap: service does not declare compliance-pack denial behavior for demo_trial tenants.
Tenant-class gap: service does not declare best-effort SLO behavior for demo_trial tenants versus contractual SLO for paid tenants.
Tenant-class direction: do not add tenant_class as an API request parameter because the memory directive says gateway/IAM enforce tenant-class behavior transparently.
Tenant-class required remediation: document emitted meter events and test scenarios that prove demo_trial cap-hit behavior and paid no-cap behavior.
Tenant-class required remediation: map OCI Always Free profile to demo_trial infrastructure in the missing `iac/oci-guest/always-free/` module.

### §3.5 Dimension 5 — industry-counterpart parity

D5 verdict: partial pass for Discourse-like forum/Q&A surface; undercovered for Circle and Vanilla.
D5 evidence: PRD feature matrix compares against Discord, Reddit, Discourse, Stack Overflow, Notion, GitHub Discussions, and Zendesk at `microservices/community/PRD.md:187-230`.
D5 gap: Circle is not included in the PRD feature matrix.
D5 gap: Vanilla Forums is not included in the PRD feature matrix.
D5 evidence: local benchmark doc compares Discourse, Khoros, Bevy, Mighty, and Oyatie at `microservices/community/benchmarks/discourse-khoros-bevy-mighty-vs-oyatie.md`.
D5 gap: local benchmark doc omits Circle and Vanilla Forums.
D5 evidence: migration playbook exists for Discourse at `microservices/community/migration-playbooks/from-discourse.md`.
D5 gap: no migration playbook exists for Circle.
D5 gap: no migration playbook exists for Vanilla Forums.
D5 evidence: OpenAPI supports posts, replies, voting, accepted answers, moderation, and KB articles at `microservices/community/contracts/openapi/community.yaml:36-220`.
D5 parity: those cover core forum/Q&A/KB flows.
D5 gap: Circle's all-in-one commerce, courses, events, live rooms, and payments positioning is not fully owned by `community`; some are intentionally out of scope or sibling owned.
D5 gap: Vanilla's mature moderation dashboard, analytics dashboard, gamification, and admin taxonomy need explicit parity mapping.
D5 future note: chat history says a later Wave 15K scope may replace the counterpart set after community absorbs network; cite `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16613-16619`.
D5 current-scope decision: this report keeps Discourse/Circle/Vanilla because the current user brief explicitly names them.

### §3.6 Dimension 6 — multi-context deployment

D6 verdict: fail.
D6 evidence: required contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854-3859`.
D6 evidence: master-plan sequencing maps each context to an IaC target at `specs/master-plan-sequencing.json:704-746`.
D6 evidence: service has only Helm, Kustomize, and Terraform IaC under `microservices/community/iac/`.
D6 gap: missing `microservices/community/iac/oyatie-public-cloud/`.
D6 gap: missing `microservices/community/iac/guest-on-aws/`.
D6 gap: missing `microservices/community/iac/oci-guest/`.
D6 gap: missing `microservices/community/iac/on-prem/`.
D6 gap: missing `microservices/community/iac/colo/`.
D6 gap: missing `microservices/community/iac/oyatie-iaas/`.
D6 gap: missing per-context tenant onboarding evidence.
D6 gap: missing deployment-context labels for telemetry and billing in service-local IaC.
D6 gap: no N/A manifest explains why any context is not deployable.
D6 severity: P1 because this is an in-scope product µservice claiming deployability but lacking canonical context modules.

### §3.7 Dimension 7 — OpenTofu IaC

D7 verdict: fail.
D7 evidence: canonical engine is OpenTofu, and the word Terraform may appear only as forbidden/superseded/migrated context per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2249`.
D7 evidence: `specs/master-plan-sequencing.json:747-775` says OpenTofu is the engine and Terraform is forbidden.
D7 evidence: service contains `microservices/community/iac/terraform/grafana-rbac.tf`.
D7 evidence: the file begins `Terraform-managed Grafana folder` and declares a `terraform` block at `microservices/community/iac/terraform/grafana-rbac.tf:1-12`.
D7 gap: no `versions.tf` pins OpenTofu and provider versions for canonical contexts.
D7 gap: no `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and context README exist under the required per-context modules.
D7 gap: PRD says `cloud-iac | Helm + Terraform registry` at `microservices/community/PRD.md:1038-1039`.
D7 finding: Terraform exists both as prose drift and as a real service-local IaC directory.
D7 severity: P1 because the service has active forbidden-engine artifacts.

### §3.8 Dimension 8 — OS support

D8 verdict: fail.
D8 evidence: ADR-0328 D-20 requires `supported-oses.json` at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3950-3952`.
D8 evidence: master-plan sequencing requires the OS matrix and manifest at `specs/master-plan-sequencing.json:777-815`.
D8 gap: `microservices/community/supported-oses.json` is absent.
D8 gap: no local artifact maps Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, or macOS Apple Silicon M5+.
D8 gap: no local artifact marks ppc64le and s390x as test-only.
D8 gap: no local artifact marks Intel macOS, pre-M5 Apple silicon, FreeBSD, OpenBSD, Windows Server, and Solaris as out of scope.
D8 gap: no local CI evidence declares tier-1 blocking or tier-2 soft gates.
D8 severity: P2 because the service has no OS manifest rather than a contradictory unsupported OS claim.

### §3.9 Dimension 9 — Rust-strict language policy

D9 verdict: pass with implementation-depth caveat.
D9 evidence: master-plan language policy requires Rust backend and frontend allowlist at `specs/master-plan-sequencing.json:817-855`.
D9 evidence: ADR-0328 D-20 allows `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, OpenAPI, AsyncAPI, OpenSLO, `.sql`, and `.md` while forbidding backend Python/JS/TS/Ruby/PHP/Java/Scala/Groovy/Go/F# at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4014-4080`.
D9 scan: no forbidden backend language files were found under `microservices/community/`.
D9 evidence: reference implementation is Rust SDK documentation at `microservices/community/reference-implementations/post-comment-vote-rust-sdk.md`.
D9 caveat: there is no `src/` directory under `microservices/community/`, so the audit cannot prove Rust implementation completeness.
D9 caveat: there is no `tests/` directory under `microservices/community/`, so the audit cannot prove build/test execution.
D9 finding: language policy is clean in file types; implementation depth is still a later buildability question.

### §3.10 Remediation sequence and ownership boundaries

R01 immediate owner: community owns local documentation repair for its own artifacts; global policy documents remain outside this audit's write scope.
R02 immediate blocker: six-context deployability cannot be claimed until service-local OpenTofu entrypoints exist or explicit N/A manifests exist; cite D6 evidence.
R03 immediate blocker: the service-local Terraform directory must be migrated or retired because ADR-0328 and master-plan sequencing treat Terraform as superseded; cite D7 evidence.
R04 immediate blocker: OCI Always Free profile evidence must be present before demo-trial infrastructure can be represented as implemented; cite D6 and D7 evidence.
R05 immediate blocker: tenant-class semantics must enter service docs before pricing, SLO, compliance, or provisioning claims can be coherent; cite §3.4.C.
R06 local boundary: demo_trial can cap usage and infrastructure, but it must not lower the product quality bar; cite no-capability-profile memory and tenant-class memory.
R07 local boundary: paid can unlock contract terms, scale, BYOK, and compliance packs, but should not become a feature-quality ladder; cite tenant-class memory.
R08 local boundary: revenue_share needs gross-revenue and at-cost substrate semantics, not a fourth plan level; cite tenant-class memory.
R09 local boundary: historical class language in old docs is evidence for Wave 15J cleanup, not a template for new audit content; cite §3.4.T.
R10 sequencing rule: rewrite retired class labels only after deciding whether each statement refers to tenant class, deployment context, usage cap, or contractual support.
R11 sequencing rule: update PRD language before regenerating manifest capability fields, because manifest claims should reflect product contract language.
R12 sequencing rule: update OpenAPI after PRD tenant-class semantics are accepted, because headers, quotas, and errors depend on that model.
R13 sequencing rule: update capacity-model and cost-budget after OpenTofu context modules exist, because context resources define meaningful capacity bounds.
R14 sequencing rule: update performance targets with one industry-grade metric set plus deployment overlays, matching the companion benchmark report.
R15 sequencing rule: update supported OS manifest before packaging claims, because current docs have no service-local OS eligibility artifact.
R16 sequencing rule: keep Helm and Kustomize as workload packaging surfaces only if OpenTofu becomes the orchestration source of truth.
R17 sequencing rule: remove the generated-stub warning in ARCHITECTURE.md only after its search, IaC, and ownership claims are reconciled.
R18 sequencing rule: repair the search-backend drift before feature-completeness claims, because ARCHITECTURE excludes search while the PRD requires search APIs.
R19 sequencing rule: decide whether search is community-owned, search-service-owned, or shared before implementing counterpart-grade search.
R20 sequencing rule: clarify notification ownership before claiming Discourse-style member notification parity.
R21 sequencing rule: clarify analytics ownership before claiming Vanilla-style dashboard parity.
R22 sequencing rule: clarify payments, courses, events, and live-room ownership before claiming Circle-style platform parity.
R23 sequencing rule: preserve ActivityPub federation as a differentiator, but bind it to moderation, abuse, and incident-response paths.
R24 sequencing rule: add moderation queue SLOs before live operation; post-create latency alone does not cover abuse response.
R25 sequencing rule: add read-path and search-path SLOs before benchmark claims become operational commitments.
R26 sequencing rule: bind OpenAPI error responses to rate limits, quota exhaustion, moderation actions, and tenant-class caps.
R27 sequencing rule: classify forbidden-language examples separately if future SDK docs add examples; backend implementation must remain Rust-strict.
R28 sequencing rule: retain the current clean forbidden-language scan as an admission check for future implementation slices.
R29 sequencing rule: use the companion feature matrix as backlog evidence, not as automatic scope expansion.
R30 sequencing rule: every counterpart gap should be accepted, rejected, or handed off with service ownership citations.
R31 sequencing rule: implementation plans should be deduplicated around acceptance tests, not around generated journey permutations.
R32 sequencing rule: incident-response references must resolve to actual runbooks or be converted into explicit missing-runbook findings.
R33 sequencing rule: audit-chain seal lag must have a precise degraded-mode policy so writes do not halt or continue inconsistently.
R34 sequencing rule: manifest dependencies should distinguish substrates, event producers, event consumers, sync dependencies, and forbidden sync calls.
R35 sequencing rule: root README absence should be resolved by a concise README or by retiring the README expectation for this service.
R36 sequencing rule: cost budgets should express demo-trial caps, paid elasticity, and revenue-share at-cost substrate treatment without feature-quality segmentation.
R37 sequencing rule: on-prem and colo overlays must state customer facility responsibilities and support boundaries.
R38 sequencing rule: guest-on-aws and guest-on-oci overlays must preserve provider-agnostic product behavior.
R39 sequencing rule: oyatie-public-cloud and oyatie-as-cloud-provider overlays must state elasticity and operator responsibility.
R40 sequencing rule: local docs should avoid cloud-provider-native lock-in claims unless a deployment-context overlay explains portability.
R41 sequencing rule: future Wave 15K counterpart changes should not rewrite this batch's evidence; they should produce a new scoped audit.
R42 sequencing rule: Discourse remains a valid baseline for forum, trust, moderation, plugin, and self-hosting expectations in this batch.
R43 sequencing rule: Circle remains a valid baseline for member spaces, courses, events, live experiences, payments, and API limits in this batch.
R44 sequencing rule: Vanilla Forums remains a valid baseline for moderation dashboards, analytics dashboards, gamification, and enterprise community operations.
R45 sequencing rule: any feature not owned by community should become an explicit cross-microservice handoff, not an untracked omission.
R46 sequencing rule: P1 findings must close before the service is used as evidence for deployable-context maturity.
R47 sequencing rule: P2 findings must close before Wave 15J and ownership-coherence claims are treated as complete.
R48 sequencing rule: P3 findings can follow after P1 and P2 closure because they refine evidence and maintainability rather than core doctrine.
R49 verification rule: future remediation should rerun inventory because this audit's 202-file, 43,266-line snapshot is time-bound.
R50 verification rule: future remediation should rerun the retired-label scan and expect zero non-audit matches after Wave 15J cleanup.
R51 verification rule: future remediation should rerun the forbidden-language extension scan after implementation files appear.
R52 verification rule: future remediation should cite line-level evidence, not only line counts, because the verification memory requires substance over volume.
R53 verification rule: future remediation should update the companion benchmark report when capacity-model or cost-budget values change.
R54 verification rule: future remediation should update the feature matrix when counterpart scope changes in a later wave.
R55 stop condition: this audit is complete when three deliverables exist, line floors pass, inventory is reported, tier-retirement candidates are cataloged, tenant-class gaps are stated, and the orchestrator report has final counts.
R56 non-goal: this audit does not implement OpenTofu modules, service code, tests, or Wave 15J cleanup.
R57 non-goal: this audit does not touch other microservices or shared canonical docs.
R58 non-goal: this audit does not create a fourth retired deliverable.
R59 non-goal: this audit does not make commits.
R60 final ownership call: community is product-substantive but not yet deployment-coherent; the service should be treated as a remediation input rather than deployable-proof evidence.

## §4 Findings table

| ID | Sev | Dimension | Finding | Evidence | Recommended remediation |
|---|---|---|---|---|---|
| COMM-P1-001 | P1 | D6 | Six canonical deployment context modules are missing. | `docs/decisions/ADR-0328...:3854-3859`; service IaC inventory only Helm/Kustomize/Terraform. | Add OpenTofu modules or N/A manifests for all six contexts. |
| COMM-P1-002 | P1 | D7 | Active Terraform IaC exists in the service path. | `microservices/community/iac/terraform/grafana-rbac.tf:1-12`; `docs/decisions/ADR-0328...:2243-2249`. | Migrate to OpenTofu-owned context modules and remove Terraform framing. |
| COMM-P1-003 | P1 | D6/D7 | OCI Always Free profile for demo_trial infrastructure is missing. | `specs/master-plan-sequencing.json:857-867`; no `iac/oci-guest/always-free/`. | Add `iac/oci-guest/always-free/` with resource caps and tenant-class semantics. |
| COMM-P1-004 | P1 | D1 | Product-to-product dependency semantics are ambiguous. | `ARCHITECTURE.md:197-205`; `manifest.json:400-420`. | Split dependencies into substrates, async product handoffs, and forbidden sync calls. |
| COMM-P2-001 | P2 | D4 | 56 exact retired tier references remain. | §3.4.T. | Wave 15J retire or rewrite to tenant_class/capacity-class language. |
| COMM-P2-002 | P2 | D4 | Tenant-class semantics are absent. | `rg` scan; memory tenant-class directive lines 101-142. | Document meter events, cap behavior, support/SLO/compliance overlays. |
| COMM-P2-003 | P2 | D8 | `supported-oses.json` is missing. | `docs/decisions/ADR-0328...:3950-3999`; no file present. | Add OS/arch manifest with tier-1, test-only, and out-of-scope entries. |
| COMM-P2-004 | P2 | D2/D5 | Circle and Vanilla Forums are absent from current counterpart matrices. | `PRD.md:187-230`; benchmark file scope. | Add counterpart coverage in parity docs and migration plans. |
| COMM-P2-005 | P2 | D1 | Search backend drift remains in capacity/cost/failure docs. | `capacity-model.md:48-58`; `cost-budget.md:29-50`; `failure-modes.md:30-34`; ADR-COMM-0004 lines 66-89. | Rewrite ops artifacts to Meilisearch/Tantivy. |
| COMM-P2-006 | P2 | D2 | Incident response references missing runbooks. | `incident-response.md:61-70`; `failure-modes.md:35-42`. | Author or redirect cross-tenant bleed, mention reconcile, DSR cascade runbooks. |
| COMM-P2-007 | P2 | D2 | Manifest bounded-context coverage is underspecified. | `manifest.json:6-64`; `PRD.md:986-1002`. | Expand manifest BCs or declare why only one manifest BC remains. |
| COMM-P2-008 | P2 | D2 | Manifest capability coverage is sparse versus capability files. | `manifest.json:88-107`; 10 capability YAML files in inventory. | Register all service capabilities or mark non-runtime docs. |
| COMM-P2-009 | P2 | D3 | ARCHITECTURE still carries generated stub warning. | `ARCHITECTURE.md:1-3`. | Remove warning after human content pass and verify no placeholder anchors remain. |
| COMM-P2-010 | P2 | D1 | Audit-chain lag policy conflicts with missing-seal severity. | `ADR-COMM-0001:83-91`; `failure-modes.md:34`. | Define which writes stop under seal lag and which degrade safely. |
| COMM-P2-011 | P2 | D2 | Root README is missing. | service inventory; ownership directive lines 18-23. | Add concise service README or explicitly retire README requirement for this service. |
| COMM-P2-012 | P2 | D7 | PRD still says Terraform registry. | `PRD.md:1038-1039`; ADR-0328 D-16. | Replace with OpenTofu registry language in a remediation slice. |
| COMM-P3-001 | P3 | D9 | Rust-strict file scan is clean but implementation is not present. | no forbidden files; no `src/`. | Track implementation buildability separately from documentation audit. |
| COMM-P3-002 | P3 | D5 | Later chat history changes future counterpart scope. | chat lines `16613-16619`. | Treat this batch as transitional and schedule Wave 15K re-audit. |
| COMM-P3-003 | P3 | D3 | Generated journey IP rows need deduplication review. | IP inventory and repeated generated row patterns. | Consolidate journey IPs around real acceptance tests. |
| COMM-P3-004 | P3 | D5 | Current Discourse migration playbook has value but uses retired tier language. | `migration-playbooks/from-discourse.md:25,77,112,151,154`. | Preserve Discourse migration mechanics while rewriting class language. |

Finding count by severity: P0 = 0.
Finding count by severity: P1 = 4.
Finding count by severity: P2 = 12.
Finding count by severity: P3 = 4.

## §5 Open questions

Open question 1: Should Wave 15K supersede this batch's Discourse/Circle/Vanilla target set, or should Discourse/Circle/Vanilla remain the forum-subset benchmark after the broader community/network merge?
Open question 2: Should `community` own courses, live events, and revenue surfaces when matching Circle, or should those remain explicit handoffs to `learn`, `meet`, `payments`, and `marketplace` equivalents?
Open question 3: Should all service-local `tier`, `tier_subtype`, and `criticality_tier` fields be renamed, or only capability-pricing tier language?
Open question 4: Should XS/S/M/L/XL capacity classes survive as sizing language, or should Wave 15J rename them to avoid all tier-like vocabulary?
Open question 5: Should `community` keep ActivityPub/fediverse scope after the chat-history network merge, or should federation become a separate substrate handoff?
Open question 6: Should missing P0 runbooks be authored in `community/runbooks/` or owned by security/audit-chain with community-specific links?
Open question 7: Should the service manifest represent the 15 PRD bounded contexts directly, or is the current single-context manifest an intentional flattening?
Open question 8: Should product-to-product handoffs such as messenger, mail, drive, and marketplace be modeled as AsyncAPI events only?
Open question 9: Should tenant-class meter events be added to the current OpenAPI/AsyncAPI/proto contracts or emitted through a cloud-billing-owned envelope?
Open question 10: Should the existing `IP-009-search-index-elasticsearch.md` filename be renamed now that ADR-COMM-0004 rejects Elasticsearch/OpenSearch at M02?

<!-- ORCHESTRATOR REPORT
  µservice: community
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/community/coherence-audit-2026-05-20.md (615 lines)
    - /Users/jasonlee/oyatie/microservices/community/feature-parity-matrix-2026-05-20.md (417 lines)
    - /Users/jasonlee/oyatie/microservices/community/performance-benchmark-numbers-2026-05-20.md (326 lines)
  inventory_files_seen: 202
  inventory_lines_read: 43266
  chat_history_matches_processed: 471 raw community hits scanned; relevant anchors cited
  findings_p0: 0
  findings_p1: 4
  findings_p2: 12
  findings_p3: 4
  customer_class_ladder_retirement_candidates_found: 56 exact demo_trial/paid/paid advanced/paid compliance-pack references; see §3.4.T candidates 001-056
  tenant_class_adoption_gaps: yes; tenant_class/demo_trial/revenue_share absent and paid references are product/payment semantics
  top_3_counterparts_confirmed: Discourse / Circle / Vanilla Forums
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1358
-->
