# Social Microservice Ownership-Coherence Audit - 2026-05-20

Audit owner: single-agent social audit lane.
Target microservice: `microservices/social/`.
Counterpart bar: TikTok / Instagram / Snapchat.
Deployable-context presumption: all six canonical contexts unless this audit flags a gap.
Directive boundary: the retired fourth deliverable remains intentionally absent.
Audit date context: Wave 3 Batch 3.2 with 2026-05-20 tier-retirement amendment and 2026-05-21 mobile-bundle amendment.
Read-only investigation source root: `/Users/jasonlee/oyatie/microservices/social/`.
Canonical sequence source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:124-146`.
Canonical six-context source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1994`.
Canonical OpenTofu source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2309`.
Canonical OS source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2648-2927`.
Canonical Rust/source policy source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3285`.
Canonical OCI Always Free source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3493-3790`.
Nine-dimension audit source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3831-3852`.
Severity rubric source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4094-4118`.
Substance bar source: `docs/standards/brief-template.md:1720-1854`.
Microservice ownership source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-92`.
Mobile-bundle amendment source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-191`.

## 1. Purpose

1. This report audits whether `social` owns a coherent, current product boundary.
2. The current canonical product target is visual and short-video social, not a generic text broadcast network.
3. The confirmed competitor union bar is TikTok, Instagram, and Snapchat.
4. The active 2026-05-21 directive says social carries the Instagram/TikTok-class visual and short-video flavor; source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-134`.
5. The same directive forbids LinkedIn-style engagement feed, follower monetization, sponsored-post promotion, and algorithmic For-You-feed behavior in this service boundary; source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:136-147`.
6. The unified mobile-app bundle directive places social beside messenger, mail, and community inside one binary per platform while preserving distinct backend services; source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:165-191`.
7. The audit compares current artifacts to that direction without editing product specs in this pass.
8. The audit scope is documentation and artifact coherence only.
9. The audit does not authorize implementation.
10. The audit does not create a fourth retired-tier delta document because that deliverable is retired by directive.
11. The audit treats current tier-language artifacts as findings when they conflict with Wave 15J retirement.
12. The audit reads current service files, canonical sequencing, constraint memory, and chat history.
13. The audit lists the full service inventory.
14. The audit evaluates nine dimensions from the ADR-0328 audit shape.
15. The audit adds explicit checks for the five cross-cutting constraints named in the prompt.
16. The audit records finding severities using the ADR-0328 severity rubric.
17. P0 means an immediate safety, security, or execution halt issue.
18. P1 means a major coherence or launch-readiness blocker.
19. P2 means a required correction that can be sequenced after P1 blockers.
20. P3 means a cleanup, hygiene, or clarification item.
21. The current conclusion is that social is not coherent enough to advance as-is.
22. The main blocker is product-purpose drift: current artifacts describe Twitter/X-style broadcast social; current doctrine requires visual and short-video social.
23. A second blocker is platform-readiness drift: the service lacks canonical six-context OpenTofu IaC.
24. A third blocker is mobile-bundle drift: messenger/mail/community coupling is partial and not expressed as a unified app-bundle backend contract.
25. A fourth blocker is tenant-class drift: current artifacts use older class and tier terms, not the three active tenant classes.
26. A fifth blocker is evidence drift: the PRD names test files and source layout that are not present under the service path.
27. The audit finds no exact demo_trial/paid/paid advanced/paid compliance-pack references in the service path.
28. The audit finds one exact metal-label false-positive: `reference-set` in an ADR, which is not a retired feature tier.
29. The audit finds broad non-metal tier semantics that should be Wave 15J cleanup candidates.
30. The audit cites every finding to file, memory, chat, or canonical-direction line references.
31. The audit remains read-only with respect to existing service artifacts.
32. The only write in this pass is this report plus the two required companion reports.

## 2. Inventory

### 2.1 Inventory Method

1. Command used for file inventory: `rg --files microservices/social | sort`.
2. Inventory file count observed: 144 files.
3. Total line count observed under the service path: 22,228 lines.
4. Empty required-documentation directories observed: `benchmarks/`, `capability-profiles/`, `faqs/`, `migration-playbooks/`, `onboarding/`, `reference-implementations/`, and `tutorials/`.
5. No `src/` directory exists under `microservices/social/`.
6. No `tests/` directory exists under `microservices/social/`.
7. No `supported-oses.json` exists under `microservices/social/`.
8. No context-specific `iac/oyatie-public-cloud/` directory exists.
9. No context-specific `iac/guest-on-aws/` directory exists.
10. No context-specific `iac/oci-guest/` directory exists.
11. No context-specific `iac/oci-guest/always-free/` directory exists.
12. No context-specific `iac/on-prem/` directory exists.
13. No context-specific `iac/colo/` directory exists.
14. No context-specific `iac/oyatie-iaas/` directory exists.
15. Current IaC inventory is Helm, Kustomize, YAML config, and policy-adjacent YAML rather than OpenTofu context modules.
16. Contract inventory includes OpenAPI, AsyncAPI, and proto files.
17. SLO inventory includes OpenSLO YAML files.
18. ADR inventory includes six `ADR-SOC-*` records.
19. Implementation-plan inventory includes baseline IPs and journey-specific IPs.
20. Runbook inventory includes operational and trust-safety runbooks.

### 2.2 Complete File Inventory

1. `microservices/social/ARCHITECTURE.md`
2. `microservices/social/AUDIT-FINDINGS-2026-05-18.json`
3. `microservices/social/CHANGELOG.md`
4. `microservices/social/IP-001-iac-bootstrap.md`
5. `microservices/social/IP-002-cargo-workspace-bootstrap.md`
6. `microservices/social/IP-003-user-profile-bc.md`
7. `microservices/social/IP-004-follow-graph-bc.md`
8. `microservices/social/IP-005-post-composition-bc.md`
9. `microservices/social/IP-006-feed-timeline-bc.md`
10. `microservices/social/IP-007-reactions-bc.md`
11. `microservices/social/IP-008-mentions-and-hashtags-bc.md`
12. `microservices/social/IP-009-trending-topics-bc.md`
13. `microservices/social/IP-010-notifications-bc.md`
14. `microservices/social/IP-011-content-moderation-bc.md`
15. `microservices/social/IP-012-search-and-cedar-filter.md`
16. `microservices/social/IP-013-age-verification-and-profile-verification.md`
17. `microservices/social/IP-014-observability-slo.md`
18. `microservices/social/IP-015-hg-social-registration-and-branch-protection.md`
19. `microservices/social/IP-016-minor-protection-strict-defaults.md`
20. `microservices/social/IP-017-abuse-defence-edge-and-cedar.md`
21. `microservices/social/IP-018-dsa-compliance-overlay.md`
22. `microservices/social/IP-journey-j100-pack-rollout-first-action.md`
23. `microservices/social/IP-journey-j31-broadcast-context.md`
24. `microservices/social/IP-journey-j79-social-moderation-surface.md`
25. `microservices/social/IP-journey-j89-social-moderation-surface.md`
26. `microservices/social/IP-journey-j90-social-moderation-surface.md`
27. `microservices/social/IP-journey-j91-us-msb-mtl-overlay.md`
28. `microservices/social/IP-journey-j92-br-lgpd-us-parent-dsar.md`
29. `microservices/social/IP-journey-j93-in-dpdpa-rbi-overlay.md`
30. `microservices/social/IP-journey-j94-sox404-public-company-controls.md`
31. `microservices/social/IP-journey-j95-iso27001-soc2-annual-audit.md`
32. `microservices/social/IP-journey-j96-ksa-uae-mena-onboarding.md`
33. `microservices/social/IP-journey-j97-sg-pdpa-mas-tenant.md`
34. `microservices/social/IP-journey-j98-au-privacy-apra-cps234.md`
35. `microservices/social/IP-journey-j99-multi-pack-conflict-resolution.md`
36. `microservices/social/PHASE-01-SOCIAL-FOUNDATION.md`
37. `microservices/social/PRD.md`
38. `microservices/social/README.md`
39. `microservices/social/backfill-replay.md`
40. `microservices/social/capabilities/T0-suggest.yaml`
41. `microservices/social/capabilities/T1-assist.yaml`
42. `microservices/social/capabilities/T2-auto.yaml`
43. `microservices/social/capacity-model.md`
44. `microservices/social/catalog/oya-social-app.yaml`
45. `microservices/social/catalog/oya-social-content-moderation-adapter-clamav.yaml`
46. `microservices/social/catalog/oya-social-content-moderation-adapter-opswat.yaml`
47. `microservices/social/catalog/oya-social-content-moderation-kernel.yaml`
48. `microservices/social/catalog/oya-social-csam-classifier-adapter-photodna.yaml`
49. `microservices/social/catalog/oya-social-dsa-transparency-worker.yaml`
50. `microservices/social/catalog/oya-social-federation-gateway-adapter-activitypub.yaml`
51. `microservices/social/catalog/oya-social-feed-timeline-adapter-valkey.yaml`
52. `microservices/social/catalog/oya-social-feed-timeline-kernel.yaml`
53. `microservices/social/catalog/oya-social-follow-graph-adapter-postgres.yaml`
54. `microservices/social/catalog/oya-social-follow-graph-kernel.yaml`
55. `microservices/social/catalog/oya-social-post-composition-adapter-ffmpeg.yaml`
56. `microservices/social/catalog/oya-social-post-composition-adapter-imagemagick.yaml`
57. `microservices/social/catalog/oya-social-post-composition-adapter-postgres.yaml`
58. `microservices/social/catalog/oya-social-post-composition-adapter-s3.yaml`
59. `microservices/social/catalog/oya-social-post-composition-kernel.yaml`
60. `microservices/social/catalog/oya-social-profile-verification-adapter-idv.yaml`
61. `microservices/social/catalog/oya-social-search-adapter-meilisearch.yaml`
62. `microservices/social/catalog/oya-social-sock-puppet-detector-kernel.yaml`
63. `microservices/social/catalog/oya-social-user-profile-adapter-postgres.yaml`
64. `microservices/social/catalog/oya-social-user-profile-domain.yaml`
65. `microservices/social/catalog/oya-social-user-profile-kernel.yaml`
66. `microservices/social/catalog/oya-social-user-profile-rest.yaml`
67. `microservices/social/competitor-parity-matrix.md`
68. `microservices/social/compliance.md`
69. `microservices/social/contracts/asyncapi/social-events.yaml`
70. `microservices/social/contracts/openapi/social.yaml`
71. `microservices/social/contracts/proto/social.proto`
72. `microservices/social/cost-budget.md`
73. `microservices/social/dashboards/abuse-defence-outcomes.json`
74. `microservices/social/dashboards/csam-and-trust-safety.json`
75. `microservices/social/dashboards/federation-and-cross-context.json`
76. `microservices/social/dashboards/feed-experience.json`
77. `microservices/social/dashboards/minor-protection-health.json`
78. `microservices/social/dashboards/moderation-and-safety.json`
79. `microservices/social/decisions/ADR-SOC-0001-feed-ranking-algorithm.md`
80. `microservices/social/decisions/ADR-SOC-0002-follow-graph-storage.md`
81. `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md`
82. `microservices/social/decisions/ADR-SOC-0004-federation-posture.md`
83. `microservices/social/decisions/ADR-SOC-0005-dual-context-feed-isolation.md`
84. `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md`
85. `microservices/social/decisions/README.md`
86. `microservices/social/dpia.md`
87. `microservices/social/failure-modes.md`
88. `microservices/social/iac/ech-config.yaml`
89. `microservices/social/iac/edge-waf.yaml`
90. `microservices/social/iac/helm/social/Chart.yaml`
91. `microservices/social/iac/helm/social/templates/deployment.yaml`
92. `microservices/social/iac/helm/social/templates/hpa.yaml`
93. `microservices/social/iac/helm/social/templates/networkpolicy.yaml`
94. `microservices/social/iac/helm/social/templates/pdb.yaml`
95. `microservices/social/iac/helm/social/templates/prometheusrule.yaml`
96. `microservices/social/iac/helm/social/templates/service.yaml`
97. `microservices/social/iac/helm/social/templates/servicemonitor.yaml`
98. `microservices/social/iac/helm/social/values.yaml`
99. `microservices/social/iac/kustomize/base/kustomization.yaml`
100. `microservices/social/iac/kustomize/overlays/pack-kr/kustomization.yaml`
101. `microservices/social/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml`
102. `microservices/social/iac/openbao-policy.yaml`
103. `microservices/social/iac/pqc-cert.yaml`
104. `microservices/social/iac/secret-bindings.yaml`
105. `microservices/social/incident-response.md`
106. `microservices/social/manifest.json`
107. `microservices/social/multi-region.md`
108. `microservices/social/policy/abuse-defence.cedar`
109. `microservices/social/policy/auditor-scope.cedar`
110. `microservices/social/policy/ci-scope.cedar`
111. `microservices/social/policy/content-policy.cedar`
112. `microservices/social/policy/data-residency.md`
113. `microservices/social/policy/dm-scope.cedar`
114. `microservices/social/policy/dual-context-isolation.md`
115. `microservices/social/policy/federation-egress.cedar`
116. `microservices/social/policy/minor-protection.cedar`
117. `microservices/social/policy/profile-verification.cedar`
118. `microservices/social/policy/public-read.cedar`
119. `microservices/social/policy/tenant-scope.cedar`
120. `microservices/social/runbooks/abuse-report-backlog-drain.md`
121. `microservices/social/runbooks/content-moderation-rollback.md`
122. `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`
123. `microservices/social/runbooks/csam-detect-and-ncmec-report.md`
124. `microservices/social/runbooks/dsa-transparency-report-generation.md`
125. `microservices/social/runbooks/federation-bridge-degraded.md`
126. `microservices/social/runbooks/feed-cache-rebuild.md`
127. `microservices/social/runbooks/follow-graph-corruption.md`
128. `microservices/social/runbooks/mention-storm-throttle.md`
129. `microservices/social/runbooks/social-bot-score-recalibration.md`
130. `microservices/social/runbooks/sock-puppet-cluster-takedown.md`
131. `microservices/social/runbooks/trending-topic-poisoning.md`
132. `microservices/social/scorecards/overrides.json`
133. `microservices/social/sdk-plan.md`
134. `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`
135. `microservices/social/slos/csam-classifier-latency.openslo.yaml`
136. `microservices/social/slos/feed-render-latency.openslo.yaml`
137. `microservices/social/slos/follow-action-latency.openslo.yaml`
138. `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`
139. `microservices/social/slos/moderation-classifier-latency.openslo.yaml`
140. `microservices/social/slos/notification-fanout-latency.openslo.yaml`
141. `microservices/social/slos/post-create-latency.openslo.yaml`
142. `microservices/social/slos/profile-render-availability.openslo.yaml`
143. `microservices/social/slos/search-people-latency.openslo.yaml`
144. `microservices/social/threat-model.md`

### 2.3 Artifact Family Inventory

1. Product baseline docs present: `PRD.md`, `ARCHITECTURE.md`, `README.md`, `PHASE-01-SOCIAL-FOUNDATION.md`, and `competitor-parity-matrix.md`.
2. Operational docs present: `capacity-model.md`, `cost-budget.md`, `failure-modes.md`, `incident-response.md`, `multi-region.md`, `threat-model.md`, `dpia.md`, and `compliance.md`.
3. Contract docs present: OpenAPI, AsyncAPI, and proto.
4. SLO docs present: nine OpenSLO YAML files.
5. Policy docs present: Cedar files plus policy markdown.
6. Dashboard docs present: six JSON dashboards.
7. Catalog docs present: 23 component catalog YAML files.
8. Baseline implementation plans present: IP-001 through IP-018.
9. Journey implementation plans present: J31, J79, J89, J90, J91 through J100.
10. Runbooks present: 12 runbooks.
11. Capability documents present: `capabilities/T0-suggest.yaml`, `capabilities/T1-assist.yaml`, and `capabilities/T2-auto.yaml`.
12. Capability-tier directory present but empty: `microservices/social/capability-profiles/`.
13. Benchmark directory present but empty: `microservices/social/benchmarks/`.
14. FAQ directory present but empty: `microservices/social/faqs/`.
15. Onboarding directory present but empty: `microservices/social/onboarding/`.
16. Migration-playbook directory present but empty: `microservices/social/migration-playbooks/`.
17. Reference-implementation directory present but empty: `microservices/social/reference-implementations/`.
18. Tutorial directory present but empty: `microservices/social/tutorials/`.
19. Source implementation not present: no `microservices/social/src/`.
20. Test implementation not present: no `microservices/social/tests/`.

## 3. Nine-Dimension Audit

### 3.1 Dimension 1 - Product Purpose and Boundary

1. Finding: current product purpose is stale relative to the 2026-05-21 direction.
2. Current PRD describes social as "Twitter/X-class first-party social platform"; evidence: `microservices/social/PRD.md:22`.
3. Current README describes broadcast-shape social with Twitter/X, Facebook, Instagram, LinkedIn, Threads, Bluesky, and Mastodon precedents; evidence: `microservices/social/README.md:18`.
4. Current architecture describes short-form posts, follow graph, ranked feed, engagement, and federation; evidence: `microservices/social/ARCHITECTURE.md:48`.
5. Current competitor matrix includes X, Bluesky, Mastodon, Threads, Instagram, LinkedIn, TikTok, Reddit, Tumblr, and Hive Social; evidence: `microservices/social/competitor-parity-matrix.md:28-40`.
6. Current canonical directive narrows social to visual posts and TikTok-class short video; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-134`.
7. Current canonical directive confirms TikTok, Instagram, and Snapchat as the top three counterparts; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:157-163`.
8. Current chat dispatch confirms `social TikTok Instagram Snapchat MOBILE_BUNDLE`; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17079`.
9. Severity: P1 because the root product category drives every downstream artifact.
10. Impact: feature parity, performance targets, contracts, safety policies, runbooks, and mobile-bundle seams are being measured against the wrong product family.
11. Required correction: rebase social around visual posts, short video, ephemeral/social-camera surfaces, creator safety, media graph, and direct sharing.
12. Required non-goal: do not reintroduce LinkedIn/X/Threads style text-feed positioning.
13. Required non-goal: do not make follower monetization or sponsored-post promotion a social-service surface.
14. Required non-goal: do not treat generic trending text topics as a primary product axis.
15. Acceptance shape: PRD, architecture, contracts, competitor matrix, runbooks, and performance targets all name TikTok/Instagram/Snapchat as the union bar.
16. Acceptance shape: Twitter/X, LinkedIn, Threads, Bluesky, and Mastodon appear only as explicit non-goals, migration context, or historical retirement notes.
17. Residual useful asset: the current moderation, privacy, age-protection, residency, and safety work remains relevant.
18. Residual useful asset: the current media transcode ADR is a starting point for visual/short-video scope.
19. Residual risk: the current text-feed contract shape may bias implementers into building a prohibited broadcast surface.
20. Audit disposition: block product-coherence signoff until purpose is corrected.

### 3.2 Dimension 2 - Ownership Boundaries and Bounded Contexts

1. Finding: bounded contexts are expansive but not aligned to the current visual/short-video target.
2. PRD bounded contexts include user-profile, follow-graph, post-composition, feed-timeline, reactions, mentions, hashtags, trending-topics, notifications, moderation, bookmarks, lists, search, federation, and verification; evidence: `microservices/social/PRD.md:127-145`.
3. Post-composition context is text-post shaped and owns repost, quote-post, comment-reply, link-preview, and cross-link to messenger; evidence: `microservices/social/PRD.md:131`.
4. Feed-timeline owns chronological plus algorithmic feed materialization; evidence: `microservices/social/PRD.md:132`.
5. Trending-topics owns windowed rank over hashtags and entities; evidence: `microservices/social/PRD.md:136`.
6. Current directive places heavy visual/video social in a separate social service, with images and short video as the defining flavor; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-134`.
7. Contract `PostKind` includes post, repost, quote, and comment but has no reels, story, spotlight, snap, lens, remix, or ephemeral visual object; evidence: `microservices/social/contracts/proto/social.proto:142-148`.
8. OpenAPI `Post` schema has `kind` values post, repost, quote_post, and comment; evidence: `microservices/social/contracts/openapi/social.yaml:103-104`.
9. OpenAPI media variants support thumbnails and HLS variants but not first-class visual product objects; evidence: `microservices/social/contracts/openapi/social.yaml:150-167`.
10. ADR-SOC-0006 supports images and short videos up to 200 MB with HLS; evidence: `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:32-35`.
11. ADR-SOC-0006 is therefore useful but underspecified for TikTok/Instagram/Snapchat parity.
12. Severity: P1 because the bounded context map tells implementers what to build.
13. Required correction: replace or demote text-feed contexts that are not needed for visual/short-video social.
14. Required correction: add explicit visual content contexts such as clip composition, story lifecycle, remix/stitch policy, media safety, creator profile display, and share-to-message handoff.
15. Required correction: ensure any "discovery" context is tag/search/relationship-guided and not an algorithmic For-You clone.
16. Required correction: keep DMs owned by messenger, not social.
17. Required correction: keep group discussion owned by community, not social.
18. Required correction: keep mail action cards owned by mail, not social.
19. Required correction: keep cloud identity sessions owned by cloud-iam/identity, not social.
20. Audit disposition: ownership boundaries need a corrective PRD and contract pass before implementation.

### 3.3 Dimension 3 - Artifact Completeness and Substance

1. Finding: social has extensive narrative artifacts but weak executable coverage.
2. The service contains 144 files and 22,228 lines of current artifacts.
3. The service contains a large architecture document and many journey IPs.
4. The service lacks `src/`, so the implementation-plan references to source crates are not backed by present code.
5. IP-002 says it will create `microservices/social/src/crates/`; evidence: `microservices/social/IP-002-cargo-workspace-bootstrap.md:20`.
6. IP-002 says it will create many Cargo child crates; evidence: `microservices/social/IP-002-cargo-workspace-bootstrap.md:28`.
7. No `microservices/social/src/` directory was present in the inventory.
8. The service lacks `tests/`, so PRD acceptance references are not backed by present test files.
9. PRD acceptance criteria name tests under `microservices/social/tests/...`; evidence: `microservices/social/PRD.md:321-339`.
10. IP-003 names `tests/dual_context_invariant_profile.rs`; evidence: `microservices/social/IP-003-user-profile-bc.md:48`.
11. IP-010 names `tests/notifications_fanout_e2e.rs`; evidence: `microservices/social/IP-010-notifications-bc.md:39`.
12. No `microservices/social/tests/` directory was present in the inventory.
13. Empty benchmark, FAQ, onboarding, migration, reference implementation, and tutorial directories exist.
14. The substance-bar standard forbids scaffold without meaningful content; evidence: `docs/standards/brief-template.md:1720-1854`.
15. Severity: P1 for source/test mismatch because artifacts claim implementation evidence that is absent.
16. Severity: P2 for empty supporting directories because they look like planned surfaces but do not carry audit substance.
17. Required correction: align PRD and IP acceptance gates with real source/test files once implementation exists.
18. Required correction: until implementation exists, mark acceptance gates as planned evidence rather than present evidence.
19. Required correction: fill empty support directories only when they contain service-specific substance.
20. Audit disposition: documentation maturity is not enough to claim executable readiness.

### 3.4 Dimension 4 - Canonical-Direction Alignment

1. Finding: social diverges from canonical direction in product target, tenant model, IaC substrate, OS evidence, and retired tier semantics.
2. Canonical master plan names six deployment contexts; evidence: `specs/master-plan-sequencing.json:704-745`.
3. Canonical master plan names OpenTofu as the IaC substrate and forbids Terraform, Pulumi, CloudFormation, and ARM/Bicep; evidence: `specs/master-plan-sequencing.json:747-775`.
4. Canonical master plan names the OS support matrix; evidence: `specs/master-plan-sequencing.json:777-815`.
5. Canonical master plan names Rust backend and restricted frontend allowlist; evidence: `specs/master-plan-sequencing.json:817-855`.
6. Canonical master plan names the OCI Always Free profile; evidence: `specs/master-plan-sequencing.json:857-867`.
7. Social manifest does not declare `deployment_contexts`; evidence: absence in `microservices/social/manifest.json` and canonical requirement in `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2084`.
8. Social manifest does not declare `supported_oses`; evidence: absence in `microservices/social/manifest.json` and canonical requirement in `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2927`.
9. Social manifest declares capability levels T0/T1/T2; evidence: `microservices/social/manifest.json:53-67` and `microservices/social/manifest.json:318-322`.
10. Social manifest uses `service_classification` and `criticality_tier`; evidence: `microservices/social/manifest.json:349` and `microservices/social/manifest.json:386`.
11. Tenant-class adoption memory retires the prior capability-profile model; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:10-43`.
12. Three-class tenant replacement is the prompt-level controlling model for this audit.
13. Existing file-level tenant model does not express `demo_trial`, `paid`, and `revenue_share` as tenant classes.
14. Severity: P1 for canonical direction drift that blocks shared planning.
15. Severity: P2 for stale tier-language cleanup after product and IaC blockers.

#### 3.4.B Mobile App Bundle Coordination

1. Mobile-bundle source states the app is one binary per platform, with messenger, mail, social, and community as distinct backends; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:165-191`.
2. Mobile-bundle source requires Swift for iOS/macOS, Kotlin for Android, WinUI for Windows, and Leptos for web; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:165-191`.
3. Current PRD says posts cross-link to messenger DMs, community discussions, ontology entities, and workflow events; evidence: `microservices/social/PRD.md:34`.
4. Current PRD has a messenger deep-link FR; evidence: `microservices/social/PRD.md:57`.
5. Current PRD has messenger handle resolution; evidence: `microservices/social/PRD.md:66`.
6. Current AsyncAPI says events are consumed by ontology, audit-chain, messenger, mail, workflow-engine, and foundry-runtime; evidence: `microservices/social/contracts/asyncapi/social-events.yaml:7-13`.
7. Current AsyncAPI includes messenger deep-link request consumption; evidence: `microservices/social/contracts/asyncapi/social-events.yaml:299-300`.
8. Current AsyncAPI includes mail action card consumption; evidence: `microservices/social/contracts/asyncapi/social-events.yaml:317`.
9. Current manifest dependencies include `messenger` but do not list mail or community; evidence: `microservices/social/manifest.json:375-383`.
10. Current architecture cross-service links do not include mail or community in the core list; evidence: `microservices/social/ARCHITECTURE.md:1121-1128`.
11. Current proto is a gRPC contract for social itself; evidence: `microservices/social/contracts/proto/social.proto:1-2`.
12. Gap: there is no explicit gRPC handoff contract to messenger, mail, and community together.
13. Gap: there is no explicit shared cloud-iam session contract in the social artifacts.
14. Gap: there is no unified push-notification stream contract across the four mobile-bundle backends.
15. Gap: mobile app bundle is not visible in `manifest.json` dependencies.
16. Gap: current docs do not state that social backends remain distinct from messenger/mail/community.
17. Required correction: add a social-owned handoff matrix for visual post to messenger share, mail action card, community discussion attach, and cloud-iam session propagation.
18. Required correction: declare that social consumes a cloud-iam/identity session and does not mint its own mobile session.
19. Required correction: declare push notification event ownership and deduplication across social, messenger, mail, and community.
20. Required correction: declare that direct messages remain messenger-owned.
21. Required correction: declare that email-style notification/action cards remain mail-owned.
22. Required correction: declare that groups/forums remain community-owned.
23. Forbidden anti-pattern: LinkedIn-style engagement-feed.
24. Forbidden anti-pattern: influencer-monetization-via-followers.
25. Forbidden anti-pattern: sponsored-post-promotion.
26. Forbidden anti-pattern: algorithmic For-You-feed.
27. Existing drift: `ARCHITECTURE.md` exposes creator monetization and branded content templates; evidence: `microservices/social/ARCHITECTURE.md:612-614`.
28. Existing drift: PRD includes an ads-substrate-stub; evidence: `microservices/social/PRD.md:70`.
29. Existing drift: runbook names paid influencers as an astroturfing campaign; evidence: `microservices/social/runbooks/trending-topic-poisoning.md:61`.
30. Existing drift: current feed docs expose algorithmic feeds; evidence: `microservices/social/PRD.md:47`, `microservices/social/contracts/openapi/social.yaml:452-479`, and `microservices/social/contracts/proto/social.proto:399`.
31. Severity: P1 because mobile-bundle behavior is a current directive and affects interface contracts.
32. Acceptance shape: gRPC and event contracts show social-to-messenger, social-to-mail, and social-to-community flows as bounded handoffs.
33. Acceptance shape: all four backends rely on shared cloud-iam session identity.
34. Acceptance shape: social notification events are routed through a unified push stream with owner, dedupe key, priority, and mobile presentation policy.
35. Acceptance shape: social remains visual/short-video and never expands into LinkedIn/X/Threads or sponsored promotion.

#### 3.4.C Tenant-Class Adoption Gaps

1. Required tenant classes for this audit: `demo_trial`, `paid`, and `revenue_share`.
2. Search result: no `demo_trial` string was found under `microservices/social/`.
3. Search result: no `tenant_class` string was found under `microservices/social/`.
4. Search result: no `revenue_share` string was found under `microservices/social/`.
5. Search result: `paid` appears in competitor and policy contexts but not as the active three-class tenant model.
6. Capacity model uses `trial`, `sandbox`, `production`, and `internal`; evidence: `microservices/social/capacity-model.md:183-195`.
7. Policy uses `paid_api_tier`; evidence: `microservices/social/policy/abuse-defence.cedar:133-134`.
8. Edge WAF uses `paid_api_tier_only`; evidence: `microservices/social/iac/edge-waf.yaml:75`.
9. Tenant-class memory requires per-service contract surfaces to adopt tenant class semantics; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:128-143`.
10. Gap classification: yes, tenant-class adoption gap exists.
11. Severity: P2 because this is a mandatory documentation/model correction but can follow P1 product and infra blockers.
12. Required correction: replace old service-local class terms with `tenant_class`.
13. Required correction: map `demo_trial` to usage caps and OCI Always Free profile infrastructure, not a lower quality feature class.
14. Required correction: map `paid` to per-seat plus usage billing with contractual SLO and permitted compliance packs.
15. Required correction: map `revenue_share` to at-cost or zero-margin substrate and gross-revenue share economics.
16. Required correction: keep quality bar uniform across tenant classes.
17. Required correction: make the tenant class visible in OpenAPI headers or claims only if cloud-iam/tenancy owns the source of truth.
18. Required correction: update capacity and cost model overlays to use tenant classes.
19. Required correction: remove `paid_api_tier` naming from policy and WAF unless it is clearly reclassified as an entitlement, not a tenant class.
20. Audit disposition: tenant-class adoption is not present.

#### 3.4.T Tier Retirement Candidates

1. Exact search pattern used: `demo_trial|paid|paid advanced|paid compliance-pack`.
2. Exact retired metal-label references found under `microservices/social/`: 0.
3. False-positive reference: `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md:178` uses `reference-set eval`, which is an evaluation corpus term, not a feature-tier label.
4. Wave 15J exact metal-label retirement candidate count: 0.
5. Broader stale tier semantics remain and should be separately scrubbed.
6. Stale tier semantic: PRD front matter says `tier: hero-product`; evidence: `microservices/social/PRD.md:8`.
7. Stale tier semantic: PRD names Personal-tier and Professional-tier behavior; evidence: `microservices/social/PRD.md:31`, `microservices/social/PRD.md:35`, `microservices/social/PRD.md:95-101`, and `microservices/social/PRD.md:120-121`.
8. Stale tier semantic: manifest defines T0/T1/T2 capability entries; evidence: `microservices/social/manifest.json:53-67`.
9. Stale tier semantic: manifest defines `capability_profiles`; evidence: `microservices/social/manifest.json:318-322`.
10. Stale tier semantic: manifest defines `service_classification`; evidence: `microservices/social/manifest.json:349`.
11. Stale tier semantic: manifest defines `criticality_tier`; evidence: `microservices/social/manifest.json:386`.
12. Stale tier semantic: competitor matrix names Professional-tier and Personal-tier rows; evidence: `microservices/social/competitor-parity-matrix.md:91`, `microservices/social/competitor-parity-matrix.md:152`, and `microservices/social/competitor-parity-matrix.md:169`.
13. Stale tier semantic: architecture uses follower-count tiering; evidence: `microservices/social/ARCHITECTURE.md:56`.
14. Stale tier semantic: architecture uses tenant-tier-adaptive wording; evidence: `microservices/social/ARCHITECTURE.md:835-869`.
15. Stale tier semantic: policy uses `paid_api_tier`; evidence: `microservices/social/policy/abuse-defence.cedar:133-134`.
16. Stale tier semantic: edge WAF uses `per_tenant_per_minute_tier_1`, `tier_2`, `paid_api_tier_only`, and `tenant_tier_adaptive_sensitivity`; evidence: `microservices/social/iac/edge-waf.yaml:18-20`, `microservices/social/iac/edge-waf.yaml:75`, and `microservices/social/iac/edge-waf.yaml:139`.
17. Stale tier semantic: capacity model uses XS/S/M/L scale labels; evidence: `microservices/social/capacity-model.md:54-59`.
18. Stale tier semantic: cost model uses XS scale component costs and scale-tier forecast; evidence: `microservices/social/cost-budget.md:44-85`.
19. Default severity for broad stale tier semantics: P2.
20. Required correction: avoid replacing old tiers with new tiers.
21. Required correction: translate capability and pricing semantics into tenant class, entitlement, scale profile, or deployment-context overlay.
22. Required correction: do not call OCI Always Free a feature tier.
23. Audit disposition: exact retired metal labels are absent, but broader tier semantics remain a Wave 15J cleanup queue.

### 3.5 Dimension 5 - Cross-Cutting Constraint Compliance

1. Multi-context constraint status: fail.
2. Canonical contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`; evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1994`.
3. Required context IaC directories are named in ADR-0328; evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2294`.
4. Social lacks the six context directories under `iac/`.
5. Current `iac/` has Helm, Kustomize, and YAML config, not context modules.
6. Manifest does not declare the six contexts.
7. Severity: P1.
8. OpenTofu constraint status: fail.
9. IP-001 title says Helm + Kustomize + OpenTofu; evidence: `microservices/social/IP-001-iac-bootstrap.md:16`.
10. IP-001 body still says Terraform-managed Grafana RBAC; evidence: `microservices/social/IP-001-iac-bootstrap.md:20-27`.
11. IP-001 target table names `microservices/social/iac/terraform/grafana-rbac.tf`; evidence: `microservices/social/IP-001-iac-bootstrap.md:44`.
12. IP-001 acceptance gates name Helm/Kubectl/Cargo but not `tofu`; evidence: `microservices/social/IP-001-iac-bootstrap.md:52-58`.
13. Canonical zero-handroll doctrine requires OpenTofu-only durable infrastructure; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35`.
14. Severity: P1.
15. OS support constraint status: fail.
16. Social lacks `supported-oses.json`.
17. Canonical OS matrix requires Tier-1 OSes and service manifests; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-76`.
18. Severity: P1.
19. Rust-strict constraint status: pass for present files, incomplete for missing implementation.
20. Search for forbidden backend/source extensions under social returned no source/package hits for Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, or F#.
21. Rust-strict memory forbids those languages for backend/runtime/durable behavior; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-66`.
22. Absence of forbidden source files is not evidence that the Rust implementation is present, because no `src/` directory exists.
23. Severity: P3 for present-language scan, P1 for missing implementation if readiness is claimed.
24. OCI Always Free constraint status: fail.
25. Social lacks `iac/oci-guest/always-free/`.
26. Canonical OCI Always Free doctrine includes Ampere A1 4 OCPU / 24 GB and other profile limits; evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514-3571`.
27. OCI Always Free memory says AWS/GCP do not have equivalent long-term persistent free doctrine; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:86-98`.
28. Social cost model uses XS/S/M/L and does not express demo_trial OCI Always Free infrastructure; evidence: `microservices/social/cost-budget.md:44-94`.
29. Severity: P1.
30. Audit disposition: all five cross-cutting dimensions were evaluated; four fail and one passes only as an absence-of-forbidden-files scan.

### 3.6 Dimension 6 - API, Contract, and Integration Coherence

1. Finding: contracts are substantial but oriented toward text-post social and event bridge, not the mobile-bundle visual/short-video contract.
2. OpenAPI covers profile, post, follow, feed, reactions, comments, mentions, hashtags, search, notifications, moderation, bookmarks, and lists; evidence: `microservices/social/contracts/openapi/social.yaml:5`.
3. OpenAPI security and headers include OIDC bearer and dual-context headers; evidence: `microservices/social/contracts/openapi/social.yaml:14-19`.
4. OpenAPI Post schema allows body max 4096 and media refs up to 10; evidence: `microservices/social/contracts/openapi/social.yaml:95-138`.
5. OpenAPI `/feed` renders chronological default with algorithmic available; evidence: `microservices/social/contracts/openapi/social.yaml:450-479`.
6. OpenAPI `/trending` returns trending topics; evidence: `microservices/social/contracts/openapi/social.yaml:521-523`.
7. OpenAPI `/media` allows up to 209,715,200 bytes; evidence: `microservices/social/contracts/openapi/social.yaml:545-572`.
8. Proto is explicitly the gRPC contract for clients that prefer gRPC; evidence: `microservices/social/contracts/proto/social.proto:1-2`.
9. Proto `PostKind` is text-feed shaped; evidence: `microservices/social/contracts/proto/social.proto:142-148`.
10. Proto feed mode is chronological or algorithmic; evidence: `microservices/social/contracts/proto/social.proto:399`.
11. AsyncAPI includes social post/repost/quote/follow/reaction/comment/moderation events; evidence: `microservices/social/contracts/asyncapi/social-events.yaml:49-56` and `microservices/social/contracts/asyncapi/social-events.yaml:224-227`.
12. AsyncAPI consumer list includes messenger and mail but not community; evidence: `microservices/social/contracts/asyncapi/social-events.yaml:7-13`.
13. Severity: P1 for missing visual/short-video object contract.
14. Severity: P1 for algorithmic feed contract drift against the current no-For-You anti-pattern.
15. Severity: P2 for missing community and unified-push handoff contract.
16. Required correction: add first-class visual post, clip, story, spotlight-style, remix/stitch, lens/effect reference, and safety-review states where product direction requires them.
17. Required correction: clarify discovery as tag/search/relationship-guided, not engagement-optimized For-You.
18. Required correction: represent mobile-bundle backend handoffs in both proto and AsyncAPI.
19. Required correction: centralize identity/session in cloud-iam and identity claims.
20. Audit disposition: current contracts are not ready for TikTok/Instagram/Snapchat parity.

### 3.7 Dimension 7 - Data, Privacy, Compliance, and Safety

1. Finding: social has strong safety documentation, but some compliance and monetization surfaces reflect the old product target.
2. PRD names Cedar enforcement for profile, post, and follow-graph reads; evidence: `microservices/social/PRD.md:94`.
3. PRD names Personal-tier and Professional-tier admin-read constraints; evidence: `microservices/social/PRD.md:95-101`.
4. PRD names EU AI Act obligations for moderation classifier and ranking model; evidence: `microservices/social/PRD.md:109`.
5. Architecture includes OpenBao credentials and push tokens; evidence: `microservices/social/ARCHITECTURE.md:906`.
6. Runbook set includes CSAM detection, coordinated inauthentic behavior, sock-puppet takedown, moderation rollback, and DSA reporting.
7. OpenSLO set includes content policy correctness, CSAM classifier latency, minor-protection engagement correctness, and moderation classifier latency.
8. Current product-direction risk: ad substrate and monetization templates conflict with the 2026-05-21 forbidden anti-patterns.
9. PRD names ads-substrate-stub as a capability; evidence: `microservices/social/PRD.md:70`.
10. Architecture names `creator-monetization-template`, `engagement-bot-stub`, and `branded-content-template`; evidence: `microservices/social/ARCHITECTURE.md:612-614`.
11. Current trust-safety risk: trending-topic poisoning runbook assumes public trending visibility as a core feature; evidence: `microservices/social/runbooks/trending-topic-poisoning.md:21-38`.
12. Current visual-safety gap: image/video abuse scanning exists, but visual-first product safety objects are not first-class in contracts.
13. ADR-SOC-0006 names OPSWAT/ClamAV scanning and HLS storage; evidence: `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:68-110`.
14. Severity: P1 for monetization and algorithmic-discovery drift.
15. Severity: P2 for stale class-language in compliance docs.
16. Required correction: preserve safety depth but align it to visual clips, stories, remixes, and camera/effects surfaces.
17. Required correction: define minor-protection limits for remix, duet/stitch-like interactions, public profile exposure, and messaging handoff.
18. Required correction: remove or quarantine monetization templates that imply follower monetization or sponsored social promotion.
19. Required correction: map policy gates to tenant classes and entitlements without feature quality stratification.
20. Audit disposition: safety substance is valuable but not product-aligned.

### 3.8 Dimension 8 - Performance, Capacity, Cost, and SLO Coherence

1. Finding: performance artifacts are detailed for text-feed operations but not benchmarked against TikTok/Instagram/Snapchat visual surfaces.
2. PRD target latency for feed render is p50 60 ms, p95 200 ms, p99 400 ms; evidence: `microservices/social/PRD.md:76-83`.
3. PRD target latency for post create is p50 30 ms, p95 100 ms, p99 250 ms; evidence: `microservices/social/PRD.md:76-83`.
4. PRD target for image transcode is p95 2 seconds; evidence: `microservices/social/PRD.md:76-90`.
5. PRD target for video transcode is p95 90 seconds; evidence: `microservices/social/PRD.md:76-90`.
6. PRD capacity says 500k active users per cell and 5M in large public pack; evidence: `microservices/social/PRD.md:297-307`.
7. PRD capacity says 1k post writes/sec per cell and 25k/sec for large public pack; evidence: `microservices/social/PRD.md:297-307`.
8. PRD capacity says 100k media uploads/day per cell and 5M/day for large public pack; evidence: `microservices/social/PRD.md:297-307`.
9. Capacity model sizes WebSocket, Postgres, Valkey, S3, Meilisearch, Layer-B Rust services, and classifier calls; evidence: `microservices/social/capacity-model.md:23`.
10. Capacity model formulas include trending ops; evidence: `microservices/social/capacity-model.md:85-87`.
11. Cost model uses XS/S/M/L infrastructure shape; evidence: `microservices/social/cost-budget.md:44-94`.
12. Competitor matrix performance numbers are old and aimed at X/Bluesky/Mastodon/Threads; evidence: `microservices/social/competitor-parity-matrix.md:116-127`.
13. Competitor matrix says video-first content type was out-of-scope or separate; evidence: `microservices/social/competitor-parity-matrix.md:133-139`.
14. Severity: P1 because performance targets for a visual short-video product need media ingest, playback, feed/discovery, push, story expiration, and edge-cache metrics.
15. Required correction: create single industry-leader targets with deployment-context overlays.
16. Required correction: replace old text-feed benchmarks with TikTok/Instagram/Snapchat union coverage.
17. Required correction: map demo_trial caps to OCI Always Free profile, not a lower feature quality target.
18. Required correction: map paid and revenue_share tenants to scale and billing overlays.
19. Required correction: define mobile-bundle push and session performance budgets.
20. Audit disposition: current SLOs are useful primitives but not sufficient for the confirmed product family.

### 3.9 Dimension 9 - Verification, Evidence, and Delivery Readiness

1. Finding: current artifacts have many planned acceptance gates but limited present verification evidence.
2. PRD acceptance criteria list expected tests; evidence: `microservices/social/PRD.md:321-339`.
3. IP-001 acceptance gates include `helm lint`, `helm template`, `kubectl kustomize`, and Cargo, but do not include `tofu` validation; evidence: `microservices/social/IP-001-iac-bootstrap.md:52-58`.
4. IP-015 says non-cited Twitter-compatible claims are refused; evidence: `microservices/social/IP-015-hg-social-registration-and-branch-protection.md:57`.
5. Chat history shows prior social/network confusion and dispatch correction into TikTok/Instagram/Snapchat mobile-bundle lane; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16530`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16535`, and `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17079`.
6. Verification memory says deliverables must be checked for scope, quality, and chat-history contradictions, not just line count; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-64`.
7. Documentation-substance memory forbids scaffold and recycled boilerplate; evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-21`.
8. Current benchmark directory is empty.
9. Current implementation source directory is absent.
10. Current implementation test directory is absent.
11. Current context IaC directories are absent.
12. Current OS matrix file is absent.
13. Current tenant-class contract is absent.
14. Severity: P1 because readiness claims would be unsupported.
15. Required correction: make each readiness claim point to executable evidence.
16. Required correction: require `tofu validate` or equivalent OpenTofu validation for context modules when they exist.
17. Required correction: require contract validation for OpenAPI, AsyncAPI, and proto after product realignment.
18. Required correction: require SLO verification with realistic media workloads.
19. Required correction: require visual/short-video smoke workloads against mobile app bundle handoffs.
20. Audit disposition: do not advance social as implementation-ready without a corrective plan.

## 4. Findings Table

| ID | Severity | Finding | Evidence | Required Action |
|---|---:|---|---|---|
| SOC-AUD-001 | P1 | Product purpose is stale: current docs define Twitter/X-class broadcast social, not visual/short-video social. | `microservices/social/PRD.md:22`; `microservices/social/README.md:18`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-134` | Rebase PRD, architecture, contracts, and parity matrix around TikTok/Instagram/Snapchat visual and short-video scope. |
| SOC-AUD-002 | P1 | Competitor set is stale and still centered on X, Bluesky, Mastodon, Threads, and LinkedIn. | `microservices/social/competitor-parity-matrix.md:28-40`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17079` | Replace counterpart analysis with TikTok / Instagram / Snapchat union coverage. |
| SOC-AUD-003 | P1 | Algorithmic feed contract conflicts with the explicit no algorithmic For-You anti-pattern. | `microservices/social/PRD.md:47`; `microservices/social/contracts/openapi/social.yaml:452-479`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:136-147` | Replace engagement-optimized For-You-style feed semantics with explicit non-goal and safer discovery model. |
| SOC-AUD-004 | P1 | Text-feed post/repost/quote-post model is not enough for visual/short-video product parity. | `microservices/social/contracts/proto/social.proto:142-148`; `microservices/social/contracts/openapi/social.yaml:103-104`; `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:32-35` | Add first-class clip/story/visual-object lifecycle contracts. |
| SOC-AUD-005 | P1 | Mobile bundle coordination is partial and lacks explicit gRPC handoffs to messenger, mail, and community. | `microservices/social/manifest.json:375-383`; `microservices/social/contracts/asyncapi/social-events.yaml:7-13`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:165-191` | Add mobile-bundle backend handoff contract and dependency surface. |
| SOC-AUD-006 | P1 | Shared cloud-iam session and unified push stream are absent from social artifacts. | `microservices/social/ARCHITECTURE.md:1121-1128`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:165-191` | Declare cloud-iam session dependency and push dedupe/presentation handoff. |
| SOC-AUD-007 | P1 | Six deployment-context IaC directories are absent. | `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1994`; service inventory | Add context modules or documented N/A fields under canonical names. |
| SOC-AUD-008 | P1 | OpenTofu substrate is missing and IP-001 still names Terraform-managed Grafana RBAC. | `microservices/social/IP-001-iac-bootstrap.md:20-27`; `microservices/social/IP-001-iac-bootstrap.md:44`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35` | Replace Terraform references and add OpenTofu context modules with validation gates. |
| SOC-AUD-009 | P1 | OS support matrix is absent. | no `microservices/social/supported-oses.json`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-76` | Add service OS support manifest and test/package assertions. |
| SOC-AUD-010 | P1 | OCI Always Free profile is absent. | no `microservices/social/iac/oci-guest/always-free/`; `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3493-3790` | Add OCI Always Free demo_trial infrastructure profile and outputs. |
| SOC-AUD-011 | P1 | PRD acceptance references source/tests that are not present. | `microservices/social/PRD.md:321-339`; no `src/`; no `tests/` | Stop treating planned acceptance gates as present evidence. |
| SOC-AUD-012 | P1 | Monetization and promotion surfaces conflict with current forbidden social anti-patterns. | `microservices/social/PRD.md:70`; `microservices/social/ARCHITECTURE.md:612-614`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:136-147` | Remove or quarantine follower monetization, sponsored promotion, and engagement-template surfaces from social. |
| SOC-AUD-013 | P2 | Tenant-class adoption is absent. | no `demo_trial`, no `tenant_class`, no `revenue_share`; `microservices/social/capacity-model.md:183-195`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:128-143` | Adopt `demo_trial`, `paid`, and `revenue_share` semantics with uniform quality bar. |
| SOC-AUD-014 | P2 | Broad stale tier semantics remain despite no exact retired metal labels. | `microservices/social/manifest.json:318-322`; `microservices/social/PRD.md:8`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:10-43` | Queue Wave 15J cleanup of stale tier words into tenant classes, entitlements, or scale profiles. |
| SOC-AUD-015 | P2 | Empty support directories imply scaffold without substance. | empty `benchmarks/`, `faqs/`, `onboarding/`, `migration-playbooks/`, `reference-implementations/`, `tutorials/`; `docs/standards/brief-template.md:1720-1854` | Fill with service-specific content or remove until needed. |
| SOC-AUD-016 | P2 | Current performance targets are text-feed oriented and not visual/short-video benchmarked. | `microservices/social/PRD.md:76-90`; `microservices/social/competitor-parity-matrix.md:116-139` | Replace with industry-leader target set and deployment/tenant overlays. |
| SOC-AUD-017 | P2 | AsyncAPI includes messenger and mail but not community in consumed-event surface. | `microservices/social/contracts/asyncapi/social-events.yaml:7-13`; `microservices/social/contracts/asyncapi/social-events.yaml:295-318` | Add community handoff and mobile-bundle event ownership. |
| SOC-AUD-018 | P2 | Policy and WAF use `paid_api_tier` rather than current tenant-class or entitlement language. | `microservices/social/policy/abuse-defence.cedar:133-134`; `microservices/social/iac/edge-waf.yaml:75` | Rename as entitlement or map to `tenant_class` without feature quality stratification. |
| SOC-AUD-019 | P2 | Federation posture is old product-family behavior and not currently justified for visual/short-video scope. | `microservices/social/decisions/ADR-SOC-0004-federation-posture.md:35-91`; `microservices/social/PRD.md:101` | Reassess ActivityPub federation against current social scope. |
| SOC-AUD-020 | P2 | Public trending-topic surface is overemphasized for a product now constrained away from engagement-optimized feeds. | `microservices/social/PRD.md:50`; `microservices/social/runbooks/trending-topic-poisoning.md:21-38` | Reframe discovery around tags/search/context without trending engagement loops. |
| SOC-AUD-021 | P2 | Exact retired metal-label search found no candidates, but the service still has a `capability-profiles/` directory. | empty `microservices/social/capability-profiles/`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:28-43` | Retire the directory or replace with tenant-class artifacts. |
| SOC-AUD-022 | P3 | Rust-strict scan found no forbidden backend language files, but implementation absence limits confidence. | no forbidden source extension hits; no `src/`; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-66` | Preserve Rust-only rule when code is created. |
| SOC-AUD-023 | P3 | False-positive `reference-set` contains a metal-like word but is not tier scaffolding. | `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md:178` | No product retirement action needed; leave as evaluation terminology. |
| SOC-AUD-024 | P3 | README says Product GA despite missing implementation and context evidence. | `microservices/social/README.md:53`; no `src/`, no `tests/`, no context IaC | Downgrade readiness wording until executable evidence exists. |
| SOC-AUD-025 | P3 | IP-001 lacks OpenTofu validation in acceptance gates. | `microservices/social/IP-001-iac-bootstrap.md:52-58` | Add OpenTofu validation once modules exist. |

## 5. Open Questions

1. Should historical Twitter/X, Threads, Bluesky, Mastodon, and LinkedIn references be removed entirely or kept in an explicit "retired product framing" appendix?
2. Which service owns media CDN orchestration for short video after social hands off encoding/CDN/view metrics to the shorts/video substrate named in the mobile-bundle directive?
3. Should social expose a separate `clip` object, or should the current `Post` object be split into `VisualPost`, `ShortVideoClip`, and `StoryItem`?
4. What exact gRPC contract should social expose to messenger for share-to-DM, reaction-to-message, and comment-thread deep links?
5. What exact gRPC contract should social expose to mail for action cards, digest inserts, and compliance notices?
6. What exact gRPC contract should social expose to community for discussion attach, community-native visual post reference, and group safety state?
7. Does cloud-iam provide a single mobile app session claim that includes tenant class, deployment context, pack, and user safety age band?
8. Which service owns unified push notification dedupe keys across messenger, mail, social, and community?
9. Should `paid_api_tier` policy language be renamed to a billing entitlement, an API quota class, or a tenant-class overlay?
10. Should current Personal/Professional context language survive as account context, or be replaced with tenant-class and actor-context terminology?
11. Should ActivityPub federation remain part of social after the product narrows to visual/short-video, or should it become community-owned?
12. Should `trending-topics` remain as a safe tag surface, or should it be retired because of the no algorithmic For-You-feed anti-pattern?
13. What is the canonical maximum short-video duration and upload size for social after comparing TikTok, Instagram, and Snapchat?
14. Should Stories-like ephemeral visual content be required for parity with Instagram and Snapchat?
15. Should Lenses/effects be in social scope, delegated to a media/effects service, or treated as post-MVP?
16. Which deployment contexts must be implemented first for social: all six simultaneously, or OCI Always Free plus one paid elastic context as a staged proof?
17. What minimum OpenTofu module set is required before social can claim guest-on-oci readiness?
18. What OS validation level is expected before any mobile-bundle social backend claim can be made?
19. Which existing SLOs remain valid after replacing text-feed assumptions with visual/short-video workloads?
20. What evidence gate should block future GA wording in README until source, tests, context IaC, and tenant classes exist?

<!-- ORCHESTRATOR REPORT
  µservice: social
  deliverables_landed: microservices/social/coherence-audit-2026-05-20.md (622 lines); microservices/social/feature-parity-matrix-2026-05-20.md (425 lines); microservices/social/performance-benchmark-numbers-2026-05-20.md (319 lines)
  inventory_files_seen: 144
  inventory_lines_read: 22228
  chat_history_matches_processed: 7
  findings_p0: 0
  findings_p1: 12
  findings_p2: 9
  findings_p3: 4
  customer_class_ladder_retirement_candidates_found: 0 exact demo_trial/paid/paid advanced/paid compliance-pack references; false-positive only microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md:178
  tenant_class_adoption_gaps: yes - no demo_trial, tenant_class, or revenue_share usage found; legacy trial/sandbox/production/internal and paid_api_tier semantics remain
  top_3_counterparts_confirmed: TikTok / Instagram / Snapchat
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1366
-->
