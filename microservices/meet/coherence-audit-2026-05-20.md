# meet ownership-coherence audit - 2026-05-20

Report owner: Codex single-agent audit lane.
Target microservice: `microservices/meet/`.
Counterpart bar: Zoom, Google Meet, Microsoft Teams Meetings.
Deliverable set: three reports only; capability-tier-deltas deliverable is retired.
Line floor for this file: 600 substantive lines.
Inventory count: 139 files under `microservices/meet/`.
Measured line inventory: 24040 lines under the service path.
Chat-history source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
Target deployable contexts from current instruction: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
Canonical deployable context paths from ADR-0328 and sequencing: `iac/oyatie-public-cloud`, `iac/guest-on-aws`, `iac/oci-guest`, `iac/on-prem`, `iac/colo`, `iac/oyatie-iaas`.
Tenant-class model to adopt: `demo_trial`, `paid`, `revenue_share`.
Retired model to remove from product docs: the four named product capability tiers.
Severity legend: P0 blocks product truth; P1 blocks canonical readiness; P2 blocks documentation coherence; P3 is cleanup risk.
Primary canonical anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` D-15 through D-20.
Primary sequencing anchor: `specs/master-plan-sequencing.json`.
Primary template anchor: `docs/standards/brief-template.md`.
Primary memory anchors: the 2026-05-20 constraint memory files listed in the batch directive.
Audit mode: read existing artifacts, compare to canonical direction, write only these three audit files.

## 1. Purpose

1. This audit determines whether `meet` is internally coherent as a single product microservice.
2. The audit treats `meet` as Oyatie's video-meeting product rather than a generic real-time substrate.
3. The service PRD says `meet` owns named rooms, lobby, calendar-bound instances, breakouts, screen share, recording, transcription, summary, interpretation, webinar mode, RTMP egress, and large-audience broadcast [PRD.md:21-40].
4. The product distinction from `messenger` huddles is explicit: huddles are ad-hoc and capped at 30, while `meet` owns scheduled rooms, lobby, recording, transcription, webinars, and broadcast scale [PRD.md:23-38].
5. The target industry set for this batch is Zoom, Google Meet, and Microsoft Teams Meetings.
6. The service's own PRD already uses those products as the main meeting-class comparison set [PRD.md:247-252].
7. Chat history confirms the rolling audit queue mapped `meet` to Zoom, Google Meet, and Microsoft Teams Meetings [8f603fc7...jsonl:16290].
8. A later queue line repeats the same top-three mapping for `meet` [8f603fc7...jsonl:16311].
9. Chat history also says the user selected full per-microservice ownership-coherence audit before remediation, so this report is a gate artifact rather than a remediation patch [8f603fc7...jsonl:13945-13947].
10. The audit evaluates product purpose, artifact completeness, interface completeness, canonical-direction alignment, counterpart coverage, deployment portability, OpenTofu IaC posture, OS support, language-policy posture, and documentation substance.
11. The audit does not grade four feature tiers because the 2026-05-20 directive says those tiers are retired.
12. The replacement commercial model is tenant-class based, with uniform industry-leader quality across the three tenant classes.
13. The audit therefore treats old product-tier language as a Wave 15J retirement candidate, not as a valid current product model.
14. The audit does not modify service implementation, policies, contracts, runbooks, or shared documents.
15. The only files authored are this report, the feature-parity matrix, and the performance-benchmark numbers report.
16. The audit accepts existing untracked or modified `meet` files as user/workspace state and does not overwrite them.
17. The audit uses local file evidence first, then memory and chat evidence, then public counterpart sources for benchmark context.
18. The audit uses public vendor limits for counterpart numbers where the vendors publish product limits.
19. The audit avoids claiming vendor latency numbers that vendors do not publish as audited benchmarks.
20. The audit flags vendor-latency rows in existing docs as unsupported when they are not tied to reproducible harness output or public sources.
21. The audit's stop condition is satisfied when the three deliverables exist, line floors are met, and the reports name material findings with provenance.
22. The audit's non-goal is landing the remediation work.
23. The audit's non-goal is creating a fourth tier-deltas document.
24. The audit's non-goal is touching another microservice.
25. The audit's non-goal is re-running the whole platform gate suite.
26. The audit's key pass signal is that `meet` has a rich product surface and deep contract/runbook/policy material.
27. The audit's key fail signal is that deployment-context, OpenTofu, OS-manifest, tenant-class, and retired-tier alignment are not coherent yet.
28. The product is directionally strong enough to preserve.
29. The canonical-direction gaps are large enough to block "ready to move on" claims.
30. The immediate remediation shape should be narrow and mechanical: add missing context matrices and OpenTofu modules, retire the old tier surface, add tenant-class semantics, and reconcile docs that claim artifacts not present.

## 2. Inventory

1. Inventory method: `find microservices/meet -type f | sort | nl -ba`.
2. Inventory result: 139 files.
3. Line inventory method: recursive line count over the same file set.
4. Line inventory result: 24040 lines.
5. Required root `README.md`: absent; only `microservices/meet/decisions/README.md` was found by the file search.
6. Required `cross-microservice-handoffs.md`: absent from the service path.
7. Required `supported-oses.json`: absent from the service path.
8. Required all-context OpenTofu directories: absent from the service path.
9. Existing IaC files are Helm and Kustomize only [iac listing command].
10. Existing top-level architecture file is present but is an untracked workspace file.
11. Existing architecture file starts with an anchor-sweep warning saying stub sections must be expanded [ARCHITECTURE.md:1-4].
12. Inventory file 001: `microservices/meet/ARCHITECTURE.md`.
13. Inventory file 002: `microservices/meet/AUDIT-FINDINGS-2026-05-18.json`.
14. Inventory file 003: `microservices/meet/IP-001-iac-bootstrap.md`.
15. Inventory file 004: `microservices/meet/IP-002-cargo-workspace-bootstrap.md`.
16. Inventory file 005: `microservices/meet/IP-003-meeting-room-kernel-domain.md`.
17. Inventory file 006: `microservices/meet/IP-004-meeting-room-adapter-postgres.md`.
18. Inventory file 007: `microservices/meet/IP-005-meeting-instance-and-livekit.md`.
19. Inventory file 008: `microservices/meet/IP-006-participant-and-lobby.md`.
20. Inventory file 009: `microservices/meet/IP-007-screen-share-and-tracks.md`.
21. Inventory file 010: `microservices/meet/IP-008-recording-pipeline.md`.
22. Inventory file 011: `microservices/meet/IP-009-transcription-pipeline.md`.
23. Inventory file 012: `microservices/meet/IP-010-webinar-and-breakouts.md`.
24. Inventory file 013: `microservices/meet/IP-011-live-stream-egress.md`.
25. Inventory file 014: `microservices/meet/IP-012-e2e-encryption-mls.md`.
26. Inventory file 015: `microservices/meet/IP-013-contracts-openapi-asyncapi-proto.md`.
27. Inventory file 016: `microservices/meet/IP-014-cedar-policies-and-data-residency.md`.
28. Inventory file 017: `microservices/meet/IP-015-hg-meet-registration-and-branch-protection.md`.
29. Inventory file 018: `microservices/meet/IP-journey-j100-pack-rollout-first-action.md`.
30. Inventory file 019: `microservices/meet/IP-journey-j132-interview-rooms.md`.
31. Inventory file 020: `microservices/meet/IP-journey-j142-layoff-room-and-hr-witness-badge.md`.
32. Inventory file 021: `microservices/meet/IP-journey-j145-cross-tenant-interview-room.md`.
33. Inventory file 022: `microservices/meet/IP-journey-j28-family-call-adaptation.md`.
34. Inventory file 023: `microservices/meet/IP-journey-j39-quarterly-review-room.md`.
35. Inventory file 024: `microservices/meet/IP-journey-j44-telemedicine-room.md`.
36. Inventory file 025: `microservices/meet/IP-journey-j56-interview-room.md`.
37. Inventory file 026: `microservices/meet/IP-journey-j57-orientation-session.md`.
38. Inventory file 027: `microservices/meet/IP-journey-j58-review-recording.md`.
39. Inventory file 028: `microservices/meet/IP-journey-j61-telehealth-consult.md`.
40. Inventory file 029: `microservices/meet/IP-journey-j72-live-translation.md`.
41. Inventory file 030: `microservices/meet/IP-journey-j91-us-msb-mtl-overlay.md`.
42. Inventory file 031: `microservices/meet/IP-journey-j92-br-lgpd-us-parent-dsar.md`.
43. Inventory file 032: `microservices/meet/IP-journey-j93-in-dpdpa-rbi-overlay.md`.
44. Inventory file 033: `microservices/meet/IP-journey-j94-sox404-public-company-controls.md`.
45. Inventory file 034: `microservices/meet/IP-journey-j95-iso27001-soc2-annual-audit.md`.
46. Inventory file 035: `microservices/meet/IP-journey-j96-ksa-uae-mena-onboarding.md`.
47. Inventory file 036: `microservices/meet/IP-journey-j97-sg-pdpa-mas-tenant.md`.
48. Inventory file 037: `microservices/meet/IP-journey-j98-au-privacy-apra-cps234.md`.
49. Inventory file 038: `microservices/meet/IP-journey-j99-multi-pack-conflict-resolution.md`.
50. Inventory file 039: `microservices/meet/PHASE-01-MEET-FOUNDATION.md`.
51. Inventory file 040: `microservices/meet/PRD.md`.
52. Inventory file 041: `microservices/meet/backfill-replay.md`.
53. Inventory file 042: `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md`.
54. Inventory file 043: `microservices/meet/capabilities/T0-suggest.yaml`.
55. Inventory file 044: `microservices/meet/capabilities/T1-assist.yaml`.
56. Inventory file 045: `microservices/meet/capabilities/T2-auto.yaml`.
57. Inventory file 046: `microservices/meet/capability-tiers/tier-matrix.md`.
58. Inventory file 047: `microservices/meet/capacity-model.md`.
59. Inventory file 048: `microservices/meet/catalog/oya-meet-audio-kernel.yaml`.
60. Inventory file 049: `microservices/meet/catalog/oya-meet-e2e-encryption-kernel.yaml`.
61. Inventory file 050: `microservices/meet/catalog/oya-meet-live-stream-egress-adapter-srs-rtmp.yaml`.
62. Inventory file 051: `microservices/meet/catalog/oya-meet-meeting-instance-adapter-coturn.yaml`.
63. Inventory file 052: `microservices/meet/catalog/oya-meet-meeting-instance-adapter-livekit.yaml`.
64. Inventory file 053: `microservices/meet/catalog/oya-meet-meeting-room-adapter-postgres.yaml`.
65. Inventory file 054: `microservices/meet/catalog/oya-meet-meeting-room-domain.yaml`.
66. Inventory file 055: `microservices/meet/catalog/oya-meet-meeting-room-kernel.yaml`.
67. Inventory file 056: `microservices/meet/catalog/oya-meet-meeting-room-rest.yaml`.
68. Inventory file 057: `microservices/meet/catalog/oya-meet-meeting-room-usecase.yaml`.
69. Inventory file 058: `microservices/meet/catalog/oya-meet-participant-adapter-valkey.yaml`.
70. Inventory file 059: `microservices/meet/catalog/oya-meet-participant-domain.yaml`.
71. Inventory file 060: `microservices/meet/catalog/oya-meet-participant-kernel.yaml`.
72. Inventory file 061: `microservices/meet/catalog/oya-meet-participant-rest.yaml`.
73. Inventory file 062: `microservices/meet/catalog/oya-meet-participant-usecase.yaml`.
74. Inventory file 063: `microservices/meet/catalog/oya-meet-recording-bridge-adapter-ffmpeg.yaml`.
75. Inventory file 064: `microservices/meet/catalog/oya-meet-recording-bridge-adapter-s3.yaml`.
76. Inventory file 065: `microservices/meet/catalog/oya-meet-recording-bridge-kernel.yaml`.
77. Inventory file 066: `microservices/meet/catalog/oya-meet-screen-share-kernel.yaml`.
78. Inventory file 067: `microservices/meet/catalog/oya-meet-transcription-adapter-whisper.yaml`.
79. Inventory file 068: `microservices/meet/catalog/oya-meet-transcription-kernel.yaml`.
80. Inventory file 069: `microservices/meet/catalog/oya-meet-video-kernel.yaml`.
81. Inventory file 070: `microservices/meet/catalog/oya-meet-webinar-mode-kernel.yaml`.
82. Inventory file 071: `microservices/meet/competitor-parity-matrix.md`.
83. Inventory file 072: `microservices/meet/compliance.md`.
84. Inventory file 073: `microservices/meet/contracts/asyncapi/meet-events.yaml`.
85. Inventory file 074: `microservices/meet/contracts/openapi/meet.yaml`.
86. Inventory file 075: `microservices/meet/contracts/proto/meet.proto`.
87. Inventory file 076: `microservices/meet/cost-budget.md`.
88. Inventory file 077: `microservices/meet/dashboards/ai-features-quality.json`.
89. Inventory file 078: `microservices/meet/dashboards/meeting-quality-mos.json`.
90. Inventory file 079: `microservices/meet/dashboards/recording-pipeline.json`.
91. Inventory file 080: `microservices/meet/decisions/ADR-MEE-001-sfu-vs-mcu-vs-mesh-topology.md`.
92. Inventory file 081: `microservices/meet/decisions/ADR-MEET-0001-sfu-substrate-selection.md`.
93. Inventory file 082: `microservices/meet/decisions/ADR-MEET-0002-recording-and-transcription-pipeline.md`.
94. Inventory file 083: `microservices/meet/decisions/ADR-MEET-0003-e2e-encryption-for-meetings.md`.
95. Inventory file 084: `microservices/meet/decisions/ADR-MEET-0004-live-streaming-egress-policy.md`.
96. Inventory file 085: `microservices/meet/decisions/ADR-MEET-0005-large-audience-and-webinar-architecture.md`.
97. Inventory file 086: `microservices/meet/decisions/ADR-MEET-0006-ai-feature-bounds.md`.
98. Inventory file 087: `microservices/meet/decisions/README.md`.
99. Inventory file 088: `microservices/meet/dpia.md`.
100. Inventory file 089: `microservices/meet/failure-modes.md`.
101. Inventory file 090: `microservices/meet/faqs/realtime-engineer-faq.md`.
102. Inventory file 091: `microservices/meet/iac/helm/meet/Chart.yaml`.
103. Inventory file 092: `microservices/meet/iac/helm/meet/templates/deployment.yaml`.
104. Inventory file 093: `microservices/meet/iac/helm/meet/templates/hpa.yaml`.
105. Inventory file 094: `microservices/meet/iac/helm/meet/templates/networkpolicy.yaml`.
106. Inventory file 095: `microservices/meet/iac/helm/meet/templates/pdb.yaml`.
107. Inventory file 096: `microservices/meet/iac/helm/meet/templates/prometheusrule.yaml`.
108. Inventory file 097: `microservices/meet/iac/helm/meet/templates/service.yaml`.
109. Inventory file 098: `microservices/meet/iac/helm/meet/templates/servicemonitor.yaml`.
110. Inventory file 099: `microservices/meet/iac/helm/meet/values.yaml`.
111. Inventory file 100: `microservices/meet/iac/kustomize/base/kustomization.yaml`.
112. Inventory file 101: `microservices/meet/iac/kustomize/base/namespace.yaml`.
113. Inventory file 102: `microservices/meet/iac/kustomize/overlays/pack-eu/kustomization.yaml`.
114. Inventory file 103: `microservices/meet/iac/kustomize/overlays/pack-kr/kustomization.yaml`.
115. Inventory file 104: `microservices/meet/incident-response.md`.
116. Inventory file 105: `microservices/meet/manifest.json`.
117. Inventory file 106: `microservices/meet/migration-playbooks/from-zoom-and-google-meet.md`.
118. Inventory file 107: `microservices/meet/multi-region.md`.
119. Inventory file 108: `microservices/meet/onboarding/realtime-engineer-first-week.md`.
120. Inventory file 109: `microservices/meet/policy/auditor-scope.cedar`.
121. Inventory file 110: `microservices/meet/policy/ci-scope.cedar`.
122. Inventory file 111: `microservices/meet/policy/data-residency.md`.
123. Inventory file 112: `microservices/meet/policy/meeting-scope.cedar`.
124. Inventory file 113: `microservices/meet/policy/public-read.cedar`.
125. Inventory file 114: `microservices/meet/policy/recording-consent.md`.
126. Inventory file 115: `microservices/meet/policy/redaction-phi.md`.
127. Inventory file 116: `microservices/meet/policy/tenant-scope.cedar`.
128. Inventory file 117: `microservices/meet/reference-implementations/join-room-and-stream-rust-sdk.md`.
129. Inventory file 118: `microservices/meet/runbooks/coturn-key-rotation.md`.
130. Inventory file 119: `microservices/meet/runbooks/live-caption-stalled.md`.
131. Inventory file 120: `microservices/meet/runbooks/lobby-bypass-incident.md`.
132. Inventory file 121: `microservices/meet/runbooks/recording-storage-degraded.md`.
133. Inventory file 122: `microservices/meet/runbooks/sfu-degraded.md`.
134. Inventory file 123: `microservices/meet/runbooks/transcription-classifier-rollback.md`.
135. Inventory file 124: `microservices/meet/runbooks/webinar-overload-throttle.md`.
136. Inventory file 125: `microservices/meet/scorecards/overrides.json`.
137. Inventory file 126: `microservices/meet/sdk-plan.md`.
138. Inventory file 127: `microservices/meet/slos/availability.openslo.yaml`.
139. Inventory file 128: `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`.
140. Inventory file 129: `microservices/meet/slos/live-caption-latency.openslo.yaml`.
141. Inventory file 130: `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`.
142. Inventory file 131: `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`.
143. Inventory file 132: `microservices/meet/slos/participant-join-latency.openslo.yaml`.
144. Inventory file 133: `microservices/meet/slos/recording-start-latency.openslo.yaml`.
145. Inventory file 134: `microservices/meet/slos/room-create-latency.openslo.yaml`.
146. Inventory file 135: `microservices/meet/slos/screen-share-start-latency.openslo.yaml`.
147. Inventory file 136: `microservices/meet/slos/transcription-correctness-bound.openslo.yaml`.
148. Inventory file 137: `microservices/meet/slos/webinar-fanout-latency.openslo.yaml`.
149. Inventory file 138: `microservices/meet/threat-model.md`.
150. Inventory file 139: `microservices/meet/tutorials/host-100-person-webinar-with-recording-transcription-translation.md`.
151. Top-level product docs seen: `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, `PHASE-01-MEET-FOUNDATION.md`, `sdk-plan.md`, `multi-region.md`, `backfill-replay.md`.
152. Service ADR docs seen: `decisions/README.md` plus seven ADR-like files.
153. Implementation plans seen: fifteen foundation IP files and twenty-one journey IP files.
154. Contract files seen: OpenAPI, AsyncAPI, and Protobuf.
155. SLO files seen: eleven OpenSLO manifests.
156. Capability risk files seen: T0, T1, T2 capability YAML files.
157. Retired product-tier surface seen: `capability-tiers/tier-matrix.md`.
158. Runtime policy files seen: eight policy and Cedar files.
159. Operational docs seen: runbooks, incident response, failure modes, capacity, cost, compliance, DPIA, threat model.
160. Developer docs seen: onboarding, tutorial, FAQ, reference implementation, migration playbook.
161. Missing source directory: no `src/` directory exists under the service path.
162. Missing test directory: no `tests/` directory exists under the service path.
163. The PRD nevertheless names test paths under `microservices/meet/tests/e2e/*` [PRD.md:313-326].
164. That mismatch means acceptance criteria cite tests that are not in the current service inventory.
165. The service has enough design documentation for a serious audit.
166. The service does not have enough deployment or verification artifacts for a readiness claim.

## 3. Nine-Dimension Audit

### 3.1 Dimension 1 - Product ownership and purpose

1. Verdict: strong product definition with one naming drift and one implementation-evidence gap.
2. `meet` is explicitly "oyatie's dedicated video-meeting product" [PRD.md:21-24].
3. The product surface includes rooms, lobby, scheduled links, breakouts, screen-share, recording, transcription, AI summary, interpretation, webinar mode, streaming egress, and broadcast [PRD.md:23-24].
4. The boundary against `messenger` huddles is crisp in the PRD table [PRD.md:25-38].
5. The service owner is `axis-meet` in PRD frontmatter and manifest [PRD.md:14-16] [manifest.json:3-5].
6. The bounded contexts cover meeting-room, meeting-instance, participant, audio, video, screen-share, recording, transcription, webinar, live-stream-egress, and e2e-encryption [PRD.md:129-145].
7. The crate family list in the PRD is large and matches the product's surface [PRD.md:133-145].
8. The manifest lists a smaller but consistent crate set around those same concerns [manifest.json:6-35].
9. The product claims desktop, web, iOS, Android, and dial-in in requirements [PRD.md:59-60].
10. The language policy requires native clients to use Swift, Kotlin, WinUI3, and Leptos/WASM for web, with Rust backend [specs/master-plan-sequencing.json:817-856].
11. The service has no local frontend client source to prove those client claims.
12. The service has no local Rust source to prove backend crate implementation.
13. The product purpose is therefore coherent at PRD/contract level but not implementation-proven.
14. The `ARCHITECTURE.md` file begins with a warning that anchor-sweep stub sections must be expanded [ARCHITECTURE.md:1-4].
15. The architecture file therefore cannot be treated as fully authored architecture despite its line count.
16. Chat history supports the broader product role of meetings in workplace integration: clocking in, approvals, signing, meetings, and related enterprise workflows [8f603fc7...jsonl:5318-5320].
17. The service fits that workplace integration vision through calendar binding and workflow events [PRD.md:195-224].
18. Dimension 1 severity: P2 for implementation-evidence gap; no P0 product contradiction.

### 3.2 Dimension 2 - Artifact completeness and internal navigation

1. Verdict: rich surface, incomplete root navigation, several required files absent.
2. The service has a PRD, architecture file, ADRs, implementation plans, contracts, SLOs, dashboards, policies, runbooks, capacity, cost, compliance, DPIA, threat model, onboarding, tutorial, FAQ, reference implementation, and migration playbook.
3. The service lacks root `README.md`.
4. The service lacks `cross-microservice-handoffs.md`.
5. The service lacks `supported-oses.json`.
6. The service lacks `tests/`.
7. The service lacks `src/`.
8. The service lacks per-context OpenTofu modules.
9. The service has `decisions/README.md`, but that does not replace a root service README for cold-start navigation.
10. The PRD names acceptance tests under `microservices/meet/tests/e2e/*` [PRD.md:313-326].
11. The inventory shows no such test files.
12. The ADR README requires each ADR to carry context, decision, alternatives, consequences, and references [decisions/README.md:26-29].
13. The ADR index is useful and maps six meet ADRs to product anchors [decisions/README.md:19-24].
14. A seventh ADR-like file, `ADR-MEE-001-sfu-vs-mcu-vs-mesh-topology.md`, is present with a different prefix pattern than `ADR-MEET-*`.
15. The user asked for `decisions/ADR-MS-*.md`; this service uses `ADR-MEET-*` and `ADR-MEE-*`.
16. The naming drift is minor but should be normalized.
17. Dimension 2 severity: P2 for missing root README, cross-handoff, OS manifest, and test/source evidence.

### 3.3 Dimension 3 - Interface, contract, and event coherence

1. Verdict: strong contract coverage across REST, event bus, and protobuf.
2. OpenAPI summary covers meeting-room CRUD, meeting-instance lifecycle, participants, lobby, recording, transcription, webinar, and live-stream egress [contracts/openapi/meet.yaml:5].
3. OpenAPI tenant scoping is declared through JWT tenant claims and `X-Scope-OrgID` [contracts/openapi/meet.yaml:9-17].
4. OpenAPI defines `MeetingRoom`, `MeetingInstance`, `Recording`, `Transcript`, and `WebinarRegistration` schemas [contracts/openapi/meet.yaml:66-157].
5. OpenAPI exposes room list/create/read/archive operations [contracts/openapi/meet.yaml:190-251].
6. OpenAPI exposes start/read/end/join meeting instance operations [contracts/openapi/meet.yaml:258-327].
7. OpenAPI exposes lobby approval/denial [contracts/openapi/meet.yaml:339-361].
8. OpenAPI exposes recording start/stop and metadata read [contracts/openapi/meet.yaml:378-481].
9. OpenAPI exposes transcription start, transcript read, summary read, and transcript search [contracts/openapi/meet.yaml:400-518].
10. OpenAPI exposes breakout, egress, webinar registration, eDiscovery hold, disclosure, health, and readiness [contracts/openapi/meet.yaml:424-646].
11. AsyncAPI declares client-facing signaling and workflow event surfaces [contracts/asyncapi/meet-events.yaml:3-18].
12. AsyncAPI channels include meeting-room created, meeting-instance started/ended, participant joined/left, recording started/finalized, transcript sealed, summary produced, breakout events, live-stream events, webinar registration, MLS epoch, eDiscovery hold, and four-eyes disclosure [contracts/asyncapi/meet-events.yaml:35-102].
13. AsyncAPI consumed events include calendar created/updated, messenger huddle graduation, ontology changed, retention-policy updated, and audit-chain sealed [contracts/asyncapi/meet-events.yaml:172-200].
14. Protobuf services cover MeetingRoom, MeetingInstance, Participant, Recording, Transcription, Webinar, and LiveStreamEgress [contracts/proto/meet.proto:13-90].
15. Protobuf entities include descriptors, frames, lobby, presence, egress, legal hold, disclosure, registration, and attendee reports [contracts/proto/meet.proto:135-419].
16. The PRD's workflow events line up with AsyncAPI channel names [PRD.md:197-224] [contracts/asyncapi/meet-events.yaml:35-102].
17. The contract surface covers the top competitor product families at a feature level.
18. The contract surface does not include a tenant-class field.
19. The contract surface includes a `broadcast_tier` enum in OpenAPI [contracts/openapi/meet.yaml:91].
20. The word "tier" there is a technical interactive-vs-broadcast state, not the retired commercial product-tier model.
21. The current directive prefers avoiding tier language where possible; `broadcast_mode` or `audience_mode` would be clearer in a cleanup.
22. Dimension 3 severity: P3 for terminology cleanup; otherwise strong.

### 3.4 Dimension 4 - Canonical-direction alignment

1. Verdict: blocked until deployment-context, OpenTofu, OS, tenant-class, and retired-tier corrections land.
2. ADR-0328 D-15 requires microservices to declare support for all relevant deployment contexts or give explicit N/A evidence [ADR-0328:1730-2240].
3. ADR-0328 D-16 requires OpenTofu as the IaC substrate and forbids Terraform, Pulumi, CloudFormation, ARM, ad-hoc shell, and console-click provisioning [ADR-0328:2241-2645].
4. ADR-0328 D-17 requires a service OS manifest where the service ships binary/container/daemon/installable artifacts [ADR-0328:2646-3044].
5. ADR-0328 D-18 requires Rust-strict backend/runtime/tooling with only authorized non-Rust extensions [ADR-0328:3045-3490].
6. ADR-0328 D-19 requires the OCI Always Free profile under `iac/oci-guest/always-free/` for the guest-on-OCI path [ADR-0328:3491-3828].
7. ADR-0328 D-20 requires every audit to include the new constraint dimensions [ADR-0328:3829-4235].
8. The master plan lists the six deployment contexts [specs/master-plan-sequencing.json:704-746].
9. The master plan names OpenTofu as substrate and forbids Terraform and other cloud-specific engines [specs/master-plan-sequencing.json:747-776].
10. The master plan names Rust as backend language and defines the frontend allowlist [specs/master-plan-sequencing.json:817-856].
11. The master plan has legacy `demo_trial_tier_oci` wording in the OCI section [specs/master-plan-sequencing.json:857-868].
12. The user prompt supersedes that legacy wording for this audit: use OCI Always Free profile, not the retired product-tier label.
13. The multi-context memory says absence of multi-context deployment spec is at least P1 for microservice PRDs/ADRs/IPs [feedback_multi_context_provider_agnostic_2026_05_20.md:32-38].
14. The OpenTofu memory says OpenTofu is mandatory and Terraform/cloud-specific IaC are forbidden [feedback_zero_handroll_opentofu_only_2026_05_20.md:10-18].
15. The OS memory says each microservice needs a supported OS manifest and CI/package coverage [feedback_os_support_matrix_2026_05_20.md:56-76].
16. The Rust-strict memory says Rust is the only production backend language and lists the forbidden languages to scan [feedback_rust_strict_only_no_python_2026_05_20.md:10-18] [feedback_rust_strict_only_no_python_2026_05_20.md:51-66].
17. The OCI memory requires Always Free posture and the relevant modules/tests [feedback_oci_always_free_maximization_2026_05_20.md:65-82].
18. The no-tier memory says "we don't have tiers" and directs retirement of the old tier corpus [feedback_no_capability_tracks_2026_05_20.md:10-45].
19. The tenant-class memory requires tenant-class, cloud billing, Cedar, contracts, and UI copy to move away from tiers [feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-142].
20. The ownership memory requires a full microservice inventory and chat-history search [feedback_microservice_ownership_coherence_2026_05_20.md:18-48].
21. The verify-deliverables memory warns not to trust line count or self-report alone [feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-54].
22. The substance memory says docs must not be scaffold line-padding [feedback_docs_substance_not_scaffold_2026_05_20.md:10-21].
23. Current `meet` fails the OpenTofu substrate evidence requirement.
24. Current `meet` fails the OS manifest evidence requirement.
25. Current `meet` fails tenant-class evidence.
26. Current `meet` contains retired product-tier content.
27. Current `meet` passes the immediate forbidden-backend-language file scan because no prohibited source files were found.
28. Current `meet` cannot prove Rust implementation because no source directory exists.
29. Current `meet` cannot prove tests because no test directory exists.
30. Dimension 4 severity: P1 for canonical-direction blockers; P2 for retired-tier and tenant-class docs.

#### 3.4.T Tier retirement candidates

1. `microservices/meet/capability-tiers/tier-matrix.md:9` names the file as a Capability Tier Matrix tied to ADR-0316; Wave 15J retirement candidate, severity P2.
2. `microservices/meet/capability-tiers/tier-matrix.md:11` says tiers differ on participant caps and features; Wave 15J retirement candidate, severity P2.
3. `microservices/meet/capability-tiers/tier-matrix.md:13` contains `demo_trial` preview-tier heading; Wave 15J retirement candidate, severity P2.
4. `microservices/meet/capability-tiers/tier-matrix.md:49` contains `paid` production-default heading; Wave 15J retirement candidate, severity P2.
5. `microservices/meet/capability-tiers/tier-matrix.md:51` says "Adds to demo_trial"; Wave 15J retirement candidate, severity P2.
6. `microservices/meet/capability-tiers/tier-matrix.md:81` contains `paid` multi-region heading; Wave 15J retirement candidate, severity P2.
7. `microservices/meet/capability-tiers/tier-matrix.md:83` says "Adds to paid"; Wave 15J retirement candidate, severity P2.
8. `microservices/meet/capability-tiers/tier-matrix.md:114` says cost delta from paid and cost for paid; Wave 15J retirement candidate, severity P2.
9. `microservices/meet/capability-tiers/tier-matrix.md:118` contains `compliance_pack-bound paid` heading; Wave 15J retirement candidate, severity P2.
10. `microservices/meet/capability-tiers/tier-matrix.md:120` says "Adds to paid"; Wave 15J retirement candidate, severity P2.
11. `microservices/meet/capability-tiers/tier-matrix.md:133` says same operational latency as paid; Wave 15J retirement candidate, severity P2.
12. `microservices/meet/capability-tiers/tier-matrix.md:135` says same as paid plus pack-bound availability; Wave 15J retirement candidate, severity P2.
13. `microservices/meet/capability-tiers/tier-matrix.md:147` contains the old promotion path across all four named tiers; Wave 15J retirement candidate, severity P2.
14. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:13` labels hardware as oyatie paid; Wave 15J retirement candidate, severity P2.
15. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:21` labels benchmark row as oyatie meet paid; Wave 15J retirement candidate, severity P2.
16. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:29` says oyatie paid is competitive; Wave 15J retirement candidate, severity P2.
17. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:31` says target is at paid; Wave 15J retirement candidate, severity P2.
18. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:37` labels audio row as oyatie meet paid; Wave 15J retirement candidate, severity P2.
19. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:50` labels video row as oyatie meet paid; Wave 15J retirement candidate, severity P2.
20. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:63` labels MOS/VMAF row as oyatie meet paid; Wave 15J retirement candidate, severity P2.
21. `microservices/meet/benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:90` labels TCO row as oyatie meet paid; Wave 15J retirement candidate, severity P2.
22. `microservices/meet/tutorials/host-100-person-webinar-with-recording-transcription-translation.md:15` requires paid cell; Wave 15J retirement candidate, severity P2.
23. `microservices/meet/faqs/realtime-engineer-faq.md:43` says Deepgram and AWS Transcribe are options at paid with usage-sensitive billing_components; Wave 15J retirement candidate, severity P2.
24. `microservices/meet/faqs/realtime-engineer-faq.md:81` refers to compliance_pack-bound paid tier; Wave 15J retirement candidate, severity P2.
25. `microservices/meet/reference-implementations/join-room-and-stream-rust-sdk.md:291` says a full MLS group error is compliance_pack-bound paid; Wave 15J retirement candidate, severity P2.
26. `microservices/meet/manifest.json:365` has a `capability_tracks` manifest field; Wave 15J structural retirement candidate, severity P2.
27. `microservices/meet/capability-tiers/` exists as a directory; Wave 15J structural retirement candidate, severity P2.
28. The explicit retired-word search returned 23 lines for the four product-tier words.
29. The structural-retirement count adds the directory and manifest field.
30. Recommended replacement: move entitlement, usage, support, SLO, and infrastructure distinctions to `tenant_class`, billing policy, and deployment-context overlays.

#### 3.4.C Tenant-class adoption gaps

1. Search evidence: `rg -n "tenant_class|demo_trial|revenue_share|OCI Always Free|always-free|paid" microservices/meet` returned no tenant-class hits.
2. The only `paid` hit was a competitor row saying Zoom has 1000 paid participants [competitor-parity-matrix.md:104].
3. The other hits were unrelated RBI/DPDPA wording in a journey IP [IP-journey-j93-in-dpdpa-rbi-overlay.md:37-70].
4. No file currently declares `demo_trial`.
5. No file currently declares `revenue_share`.
6. No file currently declares `tenant_class`.
7. No contract schema exposes tenant-class.
8. No OpenSLO manifest is parameterized by tenant-class.
9. No onboarding doc explains demo trial usage caps.
10. No cost-budget section maps paid tenants to per-seat plus usage billing.
11. No cost-budget section maps revenue-share tenants to at-cost or zero-margin substrate operation.
12. No OCI Always Free profile path exists for demo trial infrastructure.
13. Gap severity: P2 because the current service docs still work as product specs but do not express the replacement commercial model.
14. The remediation should not reintroduce feature-quality tiers.
15. The remediation should state uniform quality bar across tenant classes.
16. The remediation should put constraints on duration, usage, compliance packs, BYOK, and SLO entitlements.
17. The remediation should reference `demo_trial` as a usage-limited tenant class.
18. The remediation should reference `paid` as per-seat plus usage.
19. The remediation should reference `revenue_share` as gross-revenue-share billing.
20. The remediation should preserve product feature parity across all tenant classes.

### 3.5 Dimension 5 - Counterpart parity and product surface

1. Verdict: broad parity surface, but public proof and claim boundaries need cleanup.
2. The PRD identifies Google Meet, Zoom, Microsoft Teams Meetings, Webex, Whereby, and Jitsi-class products [PRD.md:19].
3. The batch target narrows the top-three union bar to Zoom, Google Meet, and Microsoft Teams Meetings.
4. The existing competitor parity matrix includes those three plus many secondary products [competitor-parity-matrix.md:24-40].
5. The parity matrix covers core meetings, media quality, collaboration, recording/transcription/AI, webinar/large audience, security/compliance, substrate/ops, quantitative parity, gaps, differentiators, and claim boundaries [competitor-parity-matrix.md:42-204].
6. Core meeting rows include named room, lobby, calendar binding, guest join, mobile, web, desktop, and PSTN dial-in [competitor-parity-matrix.md:44-55].
7. Collaboration rows include screen-share, remote-control, chat, reactions, polls, Q&A, whiteboard, breakouts, and hand raise [competitor-parity-matrix.md:69-81].
8. Recording/transcription/AI rows include cloud recording, captions, translation, transcript, summary, action items, interpretation, and streaming egress [competitor-parity-matrix.md:83-95].
9. Webinar rows include mode, registration, practice, interactive scale, broadcast scale, and analytics [competitor-parity-matrix.md:97-106].
10. Security rows include residency, HIPAA, KR PIPA, SEC/FINRA, MiFID II, E2E, four-eyes, Cedar, audit-chain, consent, and lobby [competitor-parity-matrix.md:108-122].
11. The existing matrix has a useful forbidden-claim section [competitor-parity-matrix.md:170-183].
12. The matrix still asserts several approximate competitor latency numbers without public proof [competitor-parity-matrix.md:135-146].
13. The new benchmark report should treat those as internal estimates unless a harness and evidence file are provided.
14. Public vendor sources support scale limits better than latency limits.
15. Zoom publicly documents meeting participant limits by plan and large-meeting add-ons up to 500, 1000, 3000, or 5000 participants [Zoom KB0068002] [Zoom KB0065323].
16. Zoom publicly documents webinars as view-only broadcasts up to 100000 attendees [Zoom KB0065323].
17. Google publicly documents Enterprise Standard and Plus Meet limits as 500 and 1000 participants, and live streaming up to 10000 and 100000 viewers [Google Workspace Admin Help 10037875].
18. Microsoft publicly documents Teams meetings at 1000 interactive participants, 10000 view-only expansion, and Teams Premium town halls up to 50000 [Microsoft Learn plan-meetings] [Microsoft Learn feature comparison].
19. `meet`'s PRD target of 1000 interactive and 100000 broadcast is in the class of the published vendor limits [PRD.md:30] [PRD.md:48].
20. Feature parity is directionally strong.
21. Evidence parity is weaker because the benchmark report cites non-public "engineering blog" and "quality guide" sources without URLs [benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:15].
22. Dimension 5 severity: P2 for benchmark-source hygiene.

### 3.6 Dimension 6 - Multi-context deployment coverage

1. Verdict: P1 gap.
2. The batch instruction says all six deployable contexts are in scope unless this audit finds otherwise.
3. ADR-0328 D-15 says Phase 3 collaboration services default to public cloud, AWS guest, OCI guest, and Oyatie-as-cloud-provider, with on-prem/colo requiring service-local review for email, meet, recordings, and public-content dependencies [ADR-0328:1730-2240].
4. The service has no `iac/oyatie-public-cloud`.
5. The service has no `iac/guest-on-aws`.
6. The service has no `iac/oci-guest`.
7. The service has no `iac/oci-guest/always-free`.
8. The service has no `iac/on-prem`.
9. The service has no `iac/colo`.
10. The service has no `iac/oyatie-iaas`.
11. The service has Helm chart files under `iac/helm/meet`.
12. The service has Kustomize base and pack overlays under `iac/kustomize`.
13. Helm and Kustomize are not the canonical per-context OpenTofu evidence required by D-15/D-16.
14. `IP-001` claims the deployment substrate is Helm, Kustomize, and OpenTofu [IP-001-iac-bootstrap.md:16-24].
15. `IP-001` file targets include Terraform-managed Grafana RBAC [IP-001-iac-bootstrap.md:35].
16. `IP-001` acceptance gates call `terraform -chdir=... validate` [IP-001-iac-bootstrap.md:43-49].
17. No `iac/terraform` or `iac/tofu` directory appears in the inventory.
18. The service therefore has a double mismatch: forbidden wording and absent canonical module evidence.
19. No service-local N/A manifest explains why any of the six contexts would be impossible.
20. Because Meet has WebRTC, TURN, GPU transcription, recording, and streaming egress, context-specific constraints must be explicit rather than assumed.
21. On-prem and colo need NAT/TURN, GPU, storage retention, and egress handling statements.
22. OCI Always Free needs smaller caps and no-retired-tier naming.
23. AWS guest and OCI guest need provider-specific OpenTofu variables without provider lock-in in product logic.
24. Oyatie public cloud and Oyatie-as-cloud-provider need elasticity guarantees.
25. Dimension 6 severity: P1.

### 3.7 Dimension 7 - OpenTofu IaC posture

1. Verdict: P1 gap.
2. Canonical direction requires OpenTofu, not Terraform [feedback_zero_handroll_opentofu_only_2026_05_20.md:10-18].
3. Canonical direction forbids hand-rolled shell bootstrap and cloud-specific IaC engines [feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35].
4. The service inventory has no `.tf` files under the expected context paths.
5. The service inventory has no OpenTofu context module directories.
6. `IP-001` says "OpenTofu" in its title [IP-001-iac-bootstrap.md:16].
7. `IP-001` then says "Terraform-managed Grafana RBAC" [IP-001-iac-bootstrap.md:20].
8. `IP-001` file target names `iac/terraform/grafana-rbac.tf` [IP-001-iac-bootstrap.md:35].
9. `IP-001` acceptance gate uses the Terraform binary [IP-001-iac-bootstrap.md:47].
10. This is a direct contradiction against the OpenTofu-only doctrine.
11. This is not only wording: the real file tree contains Helm and Kustomize instead of OpenTofu modules.
12. The current Helm chart may be useful as Kubernetes packaging, but it is not the canonical infrastructure substrate.
13. The OpenTofu remediation should create each context directory with `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and README.
14. The remediation should use `tofu init`, `tofu plan`, and `tofu apply`, not Terraform commands.
15. The remediation should keep Helm/Kustomize as generated application manifests only when wrapped by the OpenTofu context modules.
16. Dimension 7 severity: P1.

### 3.8 Dimension 8 - OS support coverage

1. Verdict: P1/P2 gap.
2. ADR-0328 D-17 requires OS support manifests for microservices that ship binary/container/daemon/installable artifacts [ADR-0328:2646-3044].
3. The OS memory says missing manifest and package docs are P1 when implementation appears portable [feedback_os_support_matrix_2026_05_20.md:56-76].
4. `meet` claims desktop, web, iOS, Android, and dial-in clients [PRD.md:59-60].
5. `meet` claims web, desktop, iOS, and Android standalone meet clients [PRD.md:40].
6. `meet` has no `supported-oses.json`.
7. `meet` has no local CI matrix proving supported Linux, macOS, Windows, iOS, Android, or browser lanes.
8. `meet` has no client source tree under this service path.
9. `meet` has no package or installer docs under this service path.
10. The PRD's client claims are plausible but not supported by OS-matrix evidence.
11. The service should explicitly split backend/container OS support from client OS support.
12. Backend: Rust services, LiveKit, coturn, SRS, ffmpeg, Whisper, Postgres, Valkey, S3-compatible storage, Meilisearch.
13. Web client: Leptos/WASM SSR plus selective island hydration per language policy.
14. Native clients: Swift for Apple platforms, Kotlin for Android, WinUI3 for Windows.
15. The OS manifest must include the canonical Tier-1 OS set where applicable, but the audit avoids using tier terminology as a commercial model.
16. Dimension 8 severity: P1 for absent manifest; P2 for missing package/client evidence.

### 3.9 Dimension 9 - Rust-strict language policy and implementation evidence

1. Verdict: no forbidden-language source files found, but implementation evidence is absent.
2. The forbidden file scan for `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, and `.fsx` returned zero files under `microservices/meet`.
3. The Rust-strict memory authorizes Markdown, YAML, JSON, Proto, OpenSLO, SQL, Cedar, OpenTofu HCL, Swift, Kotlin, WinUI3, and Leptos web where appropriate [feedback_rust_strict_only_no_python_2026_05_20.md:38-49].
4. The same memory forbids Python, JavaScript app logic, TypeScript app logic, Ruby, Perl, PHP, Java, Scala, Groovy, Go, and F# for backend/runtime/tooling [feedback_rust_strict_only_no_python_2026_05_20.md:51-60].
5. The service contains YAML, JSON, Proto, OpenSLO, Cedar, and Markdown artifacts.
6. Those artifact types are allowed.
7. The service contains no local Rust source files.
8. The service contains no `Cargo.toml` under `microservices/meet`.
9. `IP-002` claims cargo workspace bootstrap, but source evidence is not present in the inspected inventory.
10. Catalog YAML lists crate names but not implementations [manifest.json:6-35].
11. The PRD lists approximately 80 crates across 11 bounded contexts [PRD.md:129-160].
12. The manifest lists 23 crate names [manifest.json:10-34].
13. The service should reconcile PRD crate count and manifest crate count.
14. The service should add Rust crates or explicitly state that the current repo state is doc-only.
15. Dimension 9 severity: P2 for absent implementation evidence; no P1 forbidden-language violation found.

## 4. Findings Table

1. F-001 | P1 | Missing all six per-context OpenTofu modules | Evidence: inventory has only Helm/Kustomize under `iac/`; no expected context paths; ADR D-15/D-16 require context modules [ADR-0328:1730-2645].
2. F-002 | P1 | `IP-001` contradicts OpenTofu doctrine with Terraform wording and command | Evidence: Terraform target and command in IP-001 [IP-001-iac-bootstrap.md:35] [IP-001-iac-bootstrap.md:47].
3. F-003 | P1 | OS support manifest absent despite client and container claims | Evidence: no `supported-oses.json`; PRD claims web, desktop, iOS, Android clients [PRD.md:40] [PRD.md:59-60].
4. F-004 | P1 | Acceptance criteria cite tests that do not exist under service path | Evidence: PRD AC paths [PRD.md:313-326]; inventory has no `tests/`.
5. F-005 | P2 | Retired product-tier surface remains in service path | Evidence: 23 explicit references plus structural `capability-tiers/` and manifest field [capability-tiers/tier-matrix.md:13-147] [manifest.json:365].
6. F-006 | P2 | Tenant-class model absent | Evidence: no tenant-class search hits; replacement model required by current directive and memory [feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-142].
7. F-007 | P2 | Root README absent | Evidence: file search found only `decisions/README.md`.
8. F-008 | P2 | Cross-microservice handoff doc absent | Evidence: file search found no `cross-microservice-handoffs.md`.
9. F-009 | P2 | Architecture document begins as anchor-sweep stub | Evidence: stub warning [ARCHITECTURE.md:1-4].
10. F-010 | P2 | Benchmark doc uses unsupported public-source language for latency estimates | Evidence: benchmark says comparators measured against broad docs without URLs [benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:15].
11. F-011 | P2 | Journey IP `j93` imports payment/RBI PPI concepts into a meeting service | Evidence: per-transaction RBI threshold lines [IP-journey-j93-in-dpdpa-rbi-overlay.md:37-70] [IP-journey-j93-in-dpdpa-rbi-overlay.md:190-300].
12. F-012 | P2 | PRD and manifest crate surfaces are not reconciled | Evidence: PRD says ~80 crates [PRD.md:160], manifest lists 23 crate entries [manifest.json:10-34].
13. F-013 | P3 | ADR prefix drift between `ADR-MEE` and `ADR-MEET` | Evidence: inventory has both naming forms.
14. F-014 | P3 | OpenAPI uses `broadcast_tier` terminology, which can be confused with retired commercial tiers | Evidence: schema field [contracts/openapi/meet.yaml:91].
15. F-015 | P3 | Existing competitor matrix includes Webex and secondary competitors beyond the batch top-three bar | Evidence: competitor set [competitor-parity-matrix.md:24-40].
16. P0 count: 0.
17. P1 count: 4.
18. P2 count: 8.
19. P3 count: 3.
20. The P1 findings block canonical readiness.
21. The P2 findings block docs coherence and commercial-model alignment.
22. The P3 findings are cleanup targets after P1/P2 remediation.

## 5. Open Questions

1. Should `meet` retain `broadcast_tier` as a technical enum, or rename it to `audience_mode` to avoid policy confusion?
2. Should `ADR-MEE-001` be renamed into the `ADR-MEET-*` series, or is `MEE` an accepted historical prefix?
3. Should `meet` own PSTN dial-in directly, or should dial-in be delegated to `connect` or another telephony substrate?
4. Should whiteboard remain a `meet` feature surface, or should it bind to `slides`/future whiteboard service through Workflow events?
5. Should AI summary and live translation be implemented in `meet` adapters, or delegated to `translate` and `intelligence` with strict event boundaries?
6. Should all six deployment contexts be required immediately for `meet`, or should on-prem and colo begin with a service-local review/N/A matrix?
7. What is the minimum viable OCI Always Free profile for demo trial meetings: room-only, no recording, no transcription, or small recording/transcription quotas?
8. What is the uniform industry-leader quality bar for demo trial when infrastructure is capped by usage rather than feature quality?
9. Which tenant-class field owns entitlement: `tenant_class`, `billing_model`, or both?
10. Should revenue-share tenants get the same contractual SLO as paid tenants by default, or a separate contract term tied to at-cost substrate?
11. Where should client OS claims live: `supported-oses.json`, a client manifest, or both?
12. Should the reference implementation SDK stay in `reference-implementations/` or move into a generated SDK package once the developer-SDK pipeline lands?
13. Should the benchmark harness path `benchmarks/meetbench/` be added, or should the existing benchmark doc stop claiming reproducibility until harness exists?
14. Should the migration playbook cover Microsoft Teams migration alongside Zoom and Google Meet?
15. Should counterpart refresh cadence remain biannual for fast-moving AI meeting features?
16. Should the retired product-tier corpus be removed before or after tenant-class contracts are added?
17. Should `capabilities/T0/T1/T2` be renamed if the term "tier" in capability metadata causes confusion with retired product tiers?
18. Should journey IP boilerplate be rewritten in one cleanup pass or only when each journey is implemented?
19. Should `IP-journey-j93` be moved out of `meet` or rewritten as meeting-specific Indian DPDPA consent language only?
20. Should compliance docs add a tenant-class matrix for BYOK, compliance pack eligibility, and SLO terms?

### §5.1 Evidence Crosswalk For Remediation

1. First remediation target: add canonical OpenTofu context modules because D-15 and D-16 make deployment evidence a prerequisite, not optional prose (`ADR-0328:1730-2645`).
2. Required evidence: `iac/oyatie-public-cloud/` should exist with reusable OpenTofu modules, variable contracts, state wiring, and plan/apply validation.
3. Required evidence: `iac/guest-on-aws/` should exist or be explicitly marked N/A with a service-local decision.
4. Required evidence: `iac/oci-guest/` should exist, including `iac/oci-guest/always-free/` for demo_trial infrastructure.
5. Required evidence: `iac/on-prem/` should exist or cite a service-local N/A decision for meeting media and storage placement.
6. Required evidence: `iac/colo/` should exist or cite a service-local N/A decision for colocated SFU and TURN placement.
7. Required evidence: `iac/oyatie-iaas/` should exist for Oyatie-as-cloud-provider deployment.
8. Current counter-evidence: only Helm and Kustomize files were found under `microservices/meet/iac/`.
9. Current counter-evidence: `IP-001` mentions Terraform in the observability plan, which conflicts with the zero-handroll OpenTofu directive.
10. Second remediation target: add a service-local OS support manifest because D-17 requires OS support claims to be explicit (`ADR-0328:2646-3044`).
11. Required evidence: `supported-oses.json` should enumerate server runtime OS, client OS, architecture, packaging, and test status.
12. Required evidence: Meet should distinguish server-side Linux targets from desktop/mobile/web client targets.
13. Current counter-evidence: no `supported-oses.json` exists under `microservices/meet/`.
14. Third remediation target: add tenant_class semantics because the retired commercial model is no longer authoritative.
15. Required evidence: contracts or manifests should include `tenant_class` values `demo_trial`, `paid`, and `revenue_share`.
16. Required evidence: demo_trial should define usage caps, OCI Always Free profile constraints, best-effort SLO, no compliance packs, and no BYOK.
17. Required evidence: paid should define per-seat plus usage billing, contractual SLO, compliance pack eligibility, BYOK eligibility, and scale-with-payment behavior.
18. Required evidence: revenue_share should define gross-revenue percentage economics, at-cost or zero-margin substrate, and the same admitted-workload quality target.
19. Current counter-evidence: no `tenant_class`, `demo_trial`, or `revenue_share` strings were found in the meet path.
20. Fourth remediation target: replace retired commercial-language artifacts with tenant-class and deployment-context overlays.
21. Required evidence: `capability-tiers/` should be removed or retired in Wave 15J, with any useful operational content migrated to current terminology.
22. Required evidence: benchmark docs should express one industry-leader target set and context/class overlays.
23. Current counter-evidence: the retired-language catalog in §3.4.T contains 25 line-specific candidates plus the structural directory.
24. Fifth remediation target: add cross-service handoffs because Teams-class and Google Meet-class parity require Calendar, Files, Records, Search, Workflow, Messenger, Identity, Billing, Compliance, Translate, Intelligence, and Notifications ownership.
25. Required evidence: either `cross-microservice-handoffs.md` or a machine-readable equivalent should define producer, consumer, event, API, storage, retention, and failure responsibilities.
26. Current counter-evidence: no `cross-microservice-handoffs.md` file exists under meet.
27. Sixth remediation target: add implementation/test evidence or point to the owning implementation package with a checked contract.
28. Required evidence: PRD acceptance criteria should map to colocated tests or documented cross-repo test owners.
29. Current counter-evidence: no `src/` and no `tests/` directories were found under meet.
30. Seventh remediation target: harden the counterpart parity matrix around Microsoft Teams Meetings as well as Zoom and Google Meet.
31. Required evidence: migration playbooks should include Teams, not only Zoom and Google Meet.
32. Current counter-evidence: Zoom and Google Meet migration playbooks exist in inventory, but no Teams migration playbook was found.
33. Eighth remediation target: define meeting artifact storage and retention.
34. Required evidence: recording, transcript, summary, chat metadata, and audit-event storage should identify owner service, retention rule, legal hold behavior, export path, and deletion path.
35. Current evidence: OpenAPI has legal hold and disclosure endpoints, but storage ownership is not fully traced (`contracts/openapi/meet.yaml:558-620`).
36. Ninth remediation target: define room and interop surface.
37. Required evidence: room hardware, SIP/H.323/CVI, PSTN, NDI, RTMP, eCDN, and unmanaged guest browser paths should each have either a support plan or a rejection rationale.
38. Current evidence: the PRD covers broad meeting scenarios, but the inspected contracts do not expose all room and interop surfaces.
39. Tenth remediation target: replace architecture stub content with complete service evidence.
40. Current counter-evidence: `ARCHITECTURE.md:1-4` marks the architecture as an anchor-sweep scaffold, which is insufficient for industry-leader proof.
41. Eleventh remediation target: correct journey drift that imports unrelated payments/regulatory language into a meeting service.
42. Current counter-evidence: `IP-journey-j93-in-dpdpa-rbi-overlay.md` contains RBI prepaid/payment threshold language in a Meet path.
43. Twelfth remediation target: build a measured benchmark harness rather than relying on asserted benchmark tables.
44. Required evidence: room create, join, media latency, caption latency, recording readiness, broadcast fanout, failover, compliance hold, and disclosure export should be measured by deployment context.
45. Current counter-evidence: the benchmark doc references a harness path that was not present in the inventory.
46. Stop condition for remediation: do not claim completion until OpenTofu context modules, OS manifest, tenant_class contracts, handoffs, implementation/test evidence, and benchmark harness evidence are either present or explicitly scoped out with approved service-local decisions.
47. Acceptance posture: after remediation, the correct claim is not "feature parity by prose"; it is "audited parity evidence exists for named contexts, clients, workloads, and tenant classes."
48. Risk posture: without those artifacts, Meet remains a strong product concept with incomplete ownership proof.

<!-- ORCHESTRATOR REPORT
  µservice: meet
  deliverables_landed:
    - microservices/meet/coherence-audit-2026-05-20.md: 608 lines
    - microservices/meet/feature-parity-matrix-2026-05-20.md: 467 lines
    - microservices/meet/performance-benchmark-numbers-2026-05-20.md: 309 lines
  inventory_files_seen: 139
  inventory_lines_read: 24040
  chat_history_matches_processed: 44 targeted matches, 283 broad meet matches counted
  findings_p0: 0
  findings_p1: 4
  findings_p2: 8
  findings_p3: 3
  tier_retirement_candidates_found: 25 plus structural capability-tiers directory; cites: capability-tiers/tier-matrix.md:13,49,51,81,83,114,118,120,133,135,147; benchmarks/meet-vs-zoom-vs-google-meet-vs-teams-vs-webex.md:13,21,29,31,37,50,63,90; tutorials/host-100-person-webinar-with-recording-transcription-translation.md:15; faqs/realtime-engineer-faq.md:43,81; reference-implementations/join-room-and-stream-rust-sdk.md:291; manifest.json:365
  tenant_class_adoption_gaps: yes - no tenant_class, demo_trial, paid, or revenue_share contract semantics found in meet artifacts
  top_3_counterparts_confirmed: Zoom / Google Meet / Microsoft Teams Meetings
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1384
-->
