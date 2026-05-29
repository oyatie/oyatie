# Slides feature parity matrix, 2026-05-20

Audited microservice: `microservices/slides/`.
Counterpart 1: Google Slides.
Counterpart 2: Microsoft PowerPoint Online.
Counterpart 3: Pitch.
Union coverage bar: Oyatie slides should meet or beat the combined product surface where the feature belongs to a presentation authoring, collaboration, presenting, sharing, import/export, or analytics service.
Tier-retirement note: this matrix does not introduce demo_trial, paid, paid, or compliance_pack feature levels.
Tenant-class note: quality bar is uniform across `demo_trial`, `paid`, and `revenue_share`; this matrix records capability presence, not tenant-class entitlement policy.

Source A: `microservices/slides/PRD.md:24-87` defines the Oyatie slides product surface and functional requirements.
Source B: `microservices/slides/competitor-parity-matrix.md:15-130` records the existing local counterpart comparison.
Source C: Google Slides official usage and feature sources include `https://developers.google.com/workspace/slides/api/limits`, `https://support.google.com/docs/answer/6386827`, and `https://support.google.com/docs/answer/7009814`.
Source D: Microsoft official sources include `https://learn.microsoft.com/en-au/office365/servicedescriptions/office-online-service-description/powerpoint-online`, `https://support.microsoft.com/en-us/office/quick-tips-share-and-collaborate-in-powerpoint-for-the-web-cf06708d-3494-4a9c-b5fd-6eac06cafecc`, and `https://support.microsoft.com/en-us/office/present-live-engage-your-audience-with-live-presentations-039aa2cc-67fa-4fb5-9677-46ed8a060c8c`.
Source E: Pitch official sources include `https://help.pitch.com/en/articles/4615453-import-a-presentation`, `https://help.pitch.com/en/articles/6713988-export-a-presentation-to-power-point`, `https://help.pitch.com/en/articles/5592127-view-presentation-analytics`, `https://help.pitch.com/en/articles/8541722-start-a-new-presentation-with-ai`, and `https://help.pitch.com/en/articles/5671537-work-offline-in-pitch`.

## Counterpart 1 - Google Slides capability surface

1. Google surface G-001: browser-based slide authoring with decks, slides, layouts, themes, text, shapes, images, tables, charts, video embedding, and presenter view; source: Google product/help surface and local matrix `competitor-parity-matrix.md:17-36`.
2. Google surface G-002: collaborative editing, comments, sharing, revision history, and Drive-backed permissions; source: local matrix `competitor-parity-matrix.md:38-49`.
3. Google surface G-003: linked Google Sheets charts and linked slides/tables that can be updated; source: `https://support.google.com/docs/answer/7009814`.
4. Google surface G-004: Slides API chart insertion from Google Sheets; source: `https://developers.google.com/slides/api/guides/add-chart`.
5. Google surface G-005: audience Q&A during presentation, with presenter tools and viewer questions; source: `https://support.google.com/docs/answer/6386827`.
6. Google surface G-006: API quotas expose service-scale signals, including read, expensive read, and write request limits; source: `https://developers.google.com/workspace/slides/api/limits`.
7. Google surface G-007: import/export parity with PowerPoint exists but local Oyatie docs classify Google PPTX export as less faithful than Microsoft native; source: `competitor-parity-matrix.md:67-72` and `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:61-65`.
8. Google surface G-008: design suggestions and AI-adjacent layout help are present in the market, but Google does not expose Oyatie-style EU AI Act risk stamps in the official sources reviewed.
9. Google surface G-009: sharing and embedding are mature, but per-slide named-block ACL is not evidenced; local matrix marks Oyatie unique at `competitor-parity-matrix.md:48-49`.
10. Google surface G-010: presentation collaboration is mainstream, but the service does not publish customer-visible p95/p99 latency objectives for deck open, cursor sync, or save latency in the sources reviewed.
11. Google surface G-011: Google has strong Workspace integration; Oyatie must match the cross-product path through sheets/docs/forms/messenger/mail/drive dependencies in `PRD.md:287-302`.
12. Google surface G-012: Google supports large shared documents through Workspace infrastructure, but API quotas show fair-use constraints that Oyatie should reflect in demo-trial usage caps rather than product-quality tiers.
13. Google surface G-013: Google Q&A is a direct parity requirement for Oyatie audience engagement; Oyatie maps this through present/broadcast requirements in `PRD.md:69-70`.
14. Google surface G-014: Google chart linking is a direct parity requirement; Oyatie maps this through `ADR-SLIDES-0008-chart-live-link-to-sheets.md` and `PRD.md:67`.
15. Google surface G-015: Google lacks Oyatie's explicit audit-chain Ed25519 seal target; source: `PRD.md:127-133` and local matrix `competitor-parity-matrix.md:112`.

## Counterpart 2 - Microsoft PowerPoint Online capability surface

16. Microsoft surface M-001: web PowerPoint editing works in browser against presentations stored on OneDrive or SharePoint; source: Microsoft Learn PowerPoint for the web service description.
17. Microsoft surface M-002: PowerPoint for the web is a browser editing surface designed to preserve presentation fidelity; source: `https://learn.microsoft.com/en-au/office365/servicedescriptions/office-online-service-description/powerpoint-online`.
18. Microsoft surface M-003: real-time collaboration shows who else is editing and enables shared work; source: Microsoft Support quick tips for PowerPoint for the web.
19. Microsoft surface M-004: PowerPoint Live and Live Presentations offer audience device viewing, subtitles, and presenter engagement; source: Microsoft Support Present Live and PowerPoint Live documentation.
20. Microsoft surface M-005: PowerPoint Online has native OOXML lineage and therefore sets the highest bar for PPTX fidelity; source: `ADR-SLIDES-0003:36-43`, `ADR-SLIDES-0003:150-165`, and local matrix `competitor-parity-matrix.md:67-72`.
21. Microsoft surface M-006: PowerPoint for the web supports a bounded set of transitions and animations; source: Microsoft Learn service description.
22. Microsoft surface M-007: Microsoft 365 integration with Teams and Forms creates strong audience feedback and meeting workflows; source: Microsoft Support PowerPoint Live and Forms-in-PowerPoint sources.
23. Microsoft surface M-008: Designer/Copilot surfaces set a high AI-assist expectation; local matrix maps this at `competitor-parity-matrix.md:91-99`.
24. Microsoft surface M-009: Office coauthoring and revision behavior is mature; local matrix treats Microsoft as YES for multi-user editing, cursor presence, comments, version restore, and suggestion-mode at `competitor-parity-matrix.md:42-47`.
25. Microsoft surface M-010: PowerPoint Online has limitations versus desktop PowerPoint; this matters because Oyatie compares against the web product, not the desktop-only superset.
26. Microsoft surface M-011: Microsoft has strong import/export expectations, so Oyatie's accepted subset strategy must be clearly documented.
27. Microsoft surface M-012: Microsoft does not evidence per-slide named-block ACL in the reviewed sources; Oyatie marks that as unique at `competitor-parity-matrix.md:48-49`.
28. Microsoft surface M-013: Microsoft Graph exposes broad throttling concepts, but not customer-facing presentation latency targets; source: `https://learn.microsoft.com/en-us/graph/throttling-limits`.
29. Microsoft surface M-014: Microsoft audience experience includes reactions, chat, subtitles, and meeting context; Oyatie maps this through LiveKit reuse in `ADR-SLIDES-0005`.
30. Microsoft surface M-015: Microsoft native PPTX editing remains the hardest fidelity benchmark for import/export.

## Counterpart 3 - Pitch capability surface

31. Pitch surface P-001: collaborative presentation creation with team comments and slide-level feedback; source: `https://help.pitch.com/en/articles/4318672-collaborate-with-comments`.
32. Pitch surface P-002: PowerPoint import is supported for `.pptx` files; source: `https://help.pitch.com/en/articles/4615453-import-a-presentation`.
33. Pitch surface P-003: PowerPoint export to `.pptx` is supported for supported elements; source: `https://help.pitch.com/en/articles/6713988-export-a-presentation-to-power-point`.
34. Pitch surface P-004: analytics links track presentation visits, viewed slides, visit length, device/browser, and location where known; source: `https://help.pitch.com/en/articles/5592127-view-presentation-analytics`.
35. Pitch surface P-005: external presentation links can include passcodes, visitor email collection, engagement analytics, embed code, duplication controls, and PDF download controls; source: `https://help.pitch.com/en/articles/3748926-share-a-presentation-with-others/`.
36. Pitch surface P-006: guests can be invited to view, comment, or edit presentations; source: `https://help.pitch.com/en/articles/8575905-invite-guests-to-collaborate-on-presentations`.
37. Pitch surface P-007: AI presentation generation starts from prompts with constraints and sensitive-data warnings; source: `https://help.pitch.com/en/articles/8541722-start-a-new-presentation-with-ai`.
38. Pitch surface P-008: offline desktop/browser work covers many editing tasks, but offline export and some integrations are unavailable; source: `https://help.pitch.com/en/articles/5671537-work-offline-in-pitch`.
39. Pitch surface P-009: batch creation supports up to 50 presentations in one bulk run; source: `https://help.pitch.com/en/articles/12006264-batch-create-presentations`.
40. Pitch surface P-010: Pitch emphasizes modern presentation sharing and analytics more than enterprise-grade sovereignty or per-slide policy.
41. Pitch surface P-011: Pitch's analytics surface is a parity gap if Oyatie treats slides as only live presentation and not post-send engagement analytics.
42. Pitch surface P-012: Pitch's external-link controls map to Oyatie ACL/share requirements in `PRD.md:77-82`.
43. Pitch surface P-013: Pitch's AI generator maps to Oyatie full-deck generation in `contracts/openapi/slides.yaml:339-345`.
44. Pitch surface P-014: Pitch's offline editing highlights a gap in Oyatie docs; the current PRD does not make offline deck authoring a first-class requirement.
45. Pitch surface P-015: Pitch's batch creation suggests a template/data-merge surface that Oyatie should consider through sheets/forms integration.

## Union-coverage matrix

| # | Capability | Google Slides | PowerPoint Online | Pitch | Oyatie evidence | Gap verdict |
|---|---|---|---|---|---|---|
| 46 | Create deck | yes | yes | yes | `PRD.md:51`; OpenAPI deck create path | covered |
| 47 | Open deck | yes | yes | yes | `PRD.md:90-93`; deck-open SLO | covered |
| 48 | Edit slide text | yes | yes | yes | `PRD.md:54-56` | covered |
| 49 | Rich text formatting | yes | yes | yes | `competitor-parity-matrix.md:22` | covered |
| 50 | Vector shapes | yes | yes | limited | `competitor-parity-matrix.md:23` | covered |
| 51 | Freeform drawing | yes | yes | limited | `competitor-parity-matrix.md:23` | covered |
| 52 | Images | yes | yes | yes | `PRD.md:59`; `competitor-parity-matrix.md:24` | covered |
| 53 | Image crop/filter | yes | yes | yes | `competitor-parity-matrix.md:24` | covered |
| 54 | Video embed | yes | yes | yes | `PRD.md:60`; Pitch offline limitation source | covered |
| 55 | Audio embed | yes | desktop stronger | limited | `competitor-parity-matrix.md:26` | partial proof |
| 56 | Tables | yes | yes | yes | `PRD.md:58`; Pitch offline source | covered |
| 57 | Chart blocks | yes | yes | limited | `PRD.md:67`; `ADR-SLIDES-0008` | covered |
| 58 | Live Sheets chart link | yes | Excel link equivalent | limited | `ADR-SLIDES-0008`; Google chart link source | covered |
| 59 | Link revocation cascade | unclear | unclear | unclear | `competitor-parity-matrix.md:138` | Oyatie differentiator |
| 60 | Equation support | limited | yes | no | `competitor-parity-matrix.md:29` | covered |
| 61 | Themes | yes | yes | yes | `PRD.md:61`; matrix row 32 | covered |
| 62 | Template gallery | yes | yes | yes | `PRD.md:61`; Pitch template docs | covered |
| 63 | Tenant custom templates | yes | yes | library templates | `PRD.md:61`; `IP-008` | covered |
| 64 | Master slide editing | yes | yes | limited | matrix row 21 | covered |
| 65 | Slide sorter | yes | yes | yes | matrix row 34 | covered |
| 66 | Speaker notes | yes | yes | yes | `PRD.md:62`; Pitch offline source | covered |
| 67 | Presenter view | yes | yes | yes | `PRD.md:69`; matrix row 55 | covered |
| 68 | Audience view | yes | yes | yes | `PRD.md:70`; `ADR-SLIDES-0005` | covered |
| 69 | Audience Q&A | yes | yes | yes | `PRD.md:70`; Google Q&A source | covered |
| 70 | Audience polls | limited | Forms-linked | yes | `PRD.md:70`; forms embed dependency | covered |
| 71 | Audience reactions | limited | yes | yes | `PRD.md:70`; Microsoft Live source | covered |
| 72 | Broadcast mode | Meet-bridged | PowerPoint Live | yes | `ADR-SLIDES-0005`; `PRD.md:70` | covered |
| 73 | Live subtitles | not core Slides | yes | not primary | not explicit in PRD | gap |
| 74 | External public link | yes | yes | yes | `PRD.md:77-82`; Pitch share source | covered |
| 75 | External link passcode | yes via Drive controls | yes via M365 controls | yes | `PRD.md:77-82` | needs explicit contract |
| 76 | Link analytics | basic Drive/activity | Microsoft audit/usage | yes | no explicit slides analytics PRD row | gap |
| 77 | Viewed-slide analytics | not primary | not primary | yes | no explicit event in contracts | gap |
| 78 | Visitor consent for analytics | privacy controls | compliance controls | yes | compliance docs broad only | gap |
| 79 | Public embed | yes | yes | yes | `PRD.md:82`; Pitch embed source | covered |
| 80 | Download controls | yes | yes | yes | `PRD.md:82` | needs link-policy details |
| 81 | Invite external guests | yes | yes | yes | `PRD.md:77-82`; Pitch guest source | covered |
| 82 | Can view/comment/edit roles | yes | yes | yes | `PRD.md:77-82`; OpenAPI ACL schemas | covered |
| 83 | Per-deck ACL | yes | yes | yes | `PRD.md:77`; matrix row 108 | covered |
| 84 | Per-slide ACL | no | no | no | `PRD.md:78`; `ADR-SLIDES-0007` | Oyatie differentiator |
| 85 | Named-block ACL | no | no | no | `PRD.md:79`; `ADR-SLIDES-0007` | Oyatie differentiator |
| 86 | Comments | yes | yes | yes | `PRD.md:83`; Pitch comments source | covered |
| 87 | Threaded comments | yes | yes | yes | `PRD.md:83`; matrix row 45 | covered |
| 88 | Suggestion mode | yes | yes | yes | matrix row 46 | covered |
| 89 | Version history | yes | yes | yes | `PRD.md:84`; matrix row 47 | covered |
| 90 | Restore version | yes | yes | yes | `PRD.md:84` | covered |
| 91 | Offline editing | browser limited | desktop app | yes | not explicit in PRD | gap |
| 92 | Offline sync conflict policy | Google internal | Microsoft internal | slide assignment guidance | not explicit in PRD | gap |
| 93 | CRDT no-silent-loss invariant | internal | internal | internal | `PRD.md:63`; `ADR-SLIDES-0001` | covered as target |
| 94 | Cursor presence | yes | yes | yes | `PRD.md:64`; cursor SLO | covered |
| 95 | Merge conflict surfacing | yes | yes | yes | `contracts/asyncapi/slides-events.yaml:57-58` | covered |
| 96 | Per-slide assignment/status | no strong evidence | no strong evidence | yes | not explicit | gap |
| 97 | Master layout governance | yes | yes | yes | `IP-008`; matrix row 21 | covered |
| 98 | Brand kit | yes | yes | yes | `tenant-class-adoption/tenant-class-adoption-record.md:93`; needs no-tenant-class-drift rewrite | covered but stale wording |
| 99 | Bulk creation from template variables | Apps Script/API | M365 automation | yes | not explicit | gap |
| 100 | API creation/update | yes | Graph/Office APIs | limited | OpenAPI and proto contracts | covered |
| 101 | API quota disclosure | yes | Graph throttling | not public | no service quota table by tenant class | gap |
| 102 | PPTX import | yes | native | yes | `PRD.md:72`; `ADR-SLIDES-0003` | covered |
| 103 | PPTX export | yes | native | yes | `PRD.md:72`; `ADR-SLIDES-0003` | covered |
| 104 | PPTX round-trip invariant | limited | best | partial | `ADR-SLIDES-0003:43-66` | covered as subset |
| 105 | ODP import/export | limited | limited | no evidence | `PRD.md:72`; `ADR-SLIDES-0003:68-70` | Oyatie strong |
| 106 | PDF export | yes | yes | yes | `PRD.md:73`; export SLO | covered |
| 107 | PDF/A export | no common web claim | no common web claim | no evidence | `ADR-SLIDES-0003:72-76` | Oyatie differentiator |
| 108 | Keynote import | no | no | PPTX route | `PRD.md:72`; `ADR-SLIDES-0003:78-80` | covered best-effort |
| 109 | MP4 export | limited | limited | no primary | `PRD.md:75`; `ADR-SLIDES-0003:82-86` | Oyatie strong |
| 110 | PNG-per-slide export | yes | yes | yes | `ADR-SLIDES-0003:88-90` | covered |
| 111 | Deterministic MP4 export | no customer proof | no customer proof | no evidence | `ADR-SLIDES-0003:82-86` | Oyatie differentiator |
| 112 | Import malware scanning | Drive backend | M365 backend | no public details | `ADR-SLIDES-0003:45`; threat model | covered |
| 113 | Import unsupported-feature diagnostics | partial | native warnings | partial | `ADR-SLIDES-0003:66` | covered |
| 114 | Export worker sandbox | backend internal | backend internal | unknown | `ADR-SLIDES-0003:45` | covered target |
| 115 | Accessibility checker | limited | yes | limited | `PRD.md:76`; matrix row 81 | covered |
| 116 | Manual alt text | yes | yes | yes | `PRD.md:76`; matrix row 79 | covered |
| 117 | AI alt text | yes | yes | limited | `contracts/openapi/slides.yaml:354-360` | covered |
| 118 | Color contrast validation | limited | yes | limited | `PRD.md:76` | covered |
| 119 | Reduced motion fallback | limited | limited | no evidence | `ADR-SLIDES-0004`; matrix row 61 | Oyatie strong |
| 120 | Keyboard-only authoring | yes | yes | yes | `PRD.md:76`; matrix row 83 | covered |
| 121 | Screen reader canvas semantics | yes | yes | unknown | `ADR-SLIDES-0002:66-68` | covered target |
| 122 | AI layout suggestions | yes | Designer/Copilot | yes | `contracts/openapi/slides.yaml:327-338` | covered |
| 123 | AI copy refinement | yes | Copilot | yes | `capabilities/T1-assist.yaml` | covered, terminology risk |
| 124 | Full-deck AI generation | yes | Copilot | yes | `contracts/openapi/slides.yaml:339-352`; Pitch AI source | covered |
| 125 | AI sensitive-data warning | Gemini policies | Microsoft policies | yes | `ADR-SLIDES-0006` | covered target |
| 126 | EU AI Act risk stamp | no | no | no | `competitor-parity-matrix.md:100`; `ADR-SLIDES-0006` | Oyatie differentiator |
| 127 | AI provenance watermark | limited | limited | no evidence | `competitor-parity-matrix.md:101` | covered target |
| 128 | High-risk refusal default | no public equivalent | no public equivalent | no evidence | `competitor-parity-matrix.md:102` | Oyatie differentiator |
| 129 | Deck-open p95 target | no published target | no published target | no published target | `PRD.md:90-93` | Oyatie target only |
| 130 | Cursor sync p99 target | no published target | no published target | no published target | `PRD.md:97` | Oyatie target only |
| 131 | Save latency target | no published target | no published target | no published target | `PRD.md:98` | Oyatie target only |
| 132 | Export latency target | no published target | no published target | no published target | `PRD.md:99-101` | Oyatie target only |
| 133 | Active editor session ceiling | no public target | no public target | no public target | `PRD.md:105` | needs no-tenant-class-drift context overlay |
| 134 | Broadcast viewers per deck | Meet scale dependent | Teams dependent | sharing dependent | `PRD.md:108`; `PRD.md:433-434` | target covered |
| 135 | API usage throttles | public quotas | public Graph throttles | no public comparable | no tenant-class quota table | gap |
| 136 | Service-level OpenAPI | Google API | Graph/Office APIs | no broad public API | `contracts/openapi/slides.yaml` | covered |
| 137 | Async event contracts | internal | internal | no public | `contracts/asyncapi/slides-events.yaml` | covered |
| 138 | gRPC/proto surface | no public | no public | no public | `contracts/proto/slides.proto` | covered |
| 139 | Audit-chain seals | no public equivalent | compliance/audit logs | analytics logs | `PRD.md:127-133` | Oyatie differentiator |
| 140 | Per-pack residency | Workspace regions | M365 regions | limited | `PRD.md:145-148`; compliance docs | covered target |
| 141 | HIPAA/BAA support | enterprise | enterprise | no broad evidence | `competitor-parity-matrix.md:111` | covered target |
| 142 | SLSA L3 provenance | no public Slides claim | no public PowerPoint claim | no public Pitch claim | `competitor-parity-matrix.md:114` | Oyatie differentiator |
| 143 | WASM SRI | not applicable | not applicable | unknown | `PRD.md:116`; `ADR-SLIDES-0002:64-65` | Oyatie differentiator |
| 144 | Tenant-class usage caps | Workspace plans | M365 plans | paid plans | absent in slides docs | gap |
| 145 | Revenue-share partner mode | no evidence | no evidence | no evidence | absent in slides docs | gap |
| 146 | Demo-trial OCI profile | no equivalent | no equivalent | no equivalent | absent in slides docs | gap |
| 147 | OpenTofu deployability | no equivalent | no equivalent | no equivalent | absent in slides `iac/<context>` | canonical gap |
| 148 | Six deployment contexts | no equivalent | no equivalent | no equivalent | absent in slides IaC | canonical gap |
| 149 | OS support manifest | no comparable | no comparable | desktop/browser support | absent in slides docs | canonical gap |
| 150 | Root README | not counterpart | not counterpart | not counterpart | absent | hygiene gap |
| 151 | Architecture doc substance | not counterpart | not counterpart | not counterpart | `ARCHITECTURE.md:1-3` | gap |
| 152 | Product PRD substance | not counterpart | not counterpart | not counterpart | `PRD.md:24-518` | strong |
| 153 | Accepted ADR suite | not counterpart | not counterpart | not counterpart | ADR-SLIDES-0001..0008 | strong |
| 154 | Runbook coverage | backend internal | backend internal | no public | seven runbooks | strong |
| 155 | SLO coverage | no public | no public | no public | nine OpenSLO files | strong |
| 156 | Dashboard coverage | no public | no public | no public | three dashboard JSON files | strong |
| 157 | Data residency policy | Drive/Workspace admin | M365 admin | limited | `policy/data-residency.md`; compliance docs | covered target |
| 158 | Cedar policy | no public equivalent | no public equivalent | no public equivalent | `policy/*.cedar` | Oyatie differentiator |
| 159 | Editor isolation policy | no public equivalent | no public equivalent | no public equivalent | `policy/editor-isolation.md` | covered target |
| 160 | Public-read policy | link sharing | link sharing | external links | `policy/public-read.cedar`; Pitch share source | covered |
| 161 | Share ACL drift runbook | backend internal | backend internal | no public | `runbooks/share-acl-drift.md` | covered target |
| 162 | CRDT conflict runbook | backend internal | backend internal | no public | `runbooks/collab-conflict-resolution-crdt.md` | covered target |
| 163 | Export failure runbook | backend internal | backend internal | no public | `runbooks/export-pipeline-failure-pptx.md` | covered target |
| 164 | Broadcast degraded runbook | backend internal | backend internal | no public | `runbooks/broadcast-mode-degraded.md` | covered target |
| 165 | Theme corruption runbook | backend internal | backend internal | no public | `runbooks/theme-corruption.md` | covered target |
| 166 | Attachment restore runbook | backend internal | backend internal | no public | `runbooks/attachment-restore.md` | covered target |
| 167 | Animation rollback runbook | backend internal | backend internal | no public | `runbooks/animation-engine-rollback.md` | covered target |
| 168 | Security threat model | backend internal | backend internal | no public | `threat-model.md` | covered target |
| 169 | DPIA | enterprise docs | enterprise docs | privacy docs | `dpia.md` | covered target |
| 170 | Compliance pack mapping | Workspace/M365 compliance | M365 compliance | limited | `compliance.md` | covered target |
| 171 | Cost budget | pricing pages | M365 pricing | Pitch pricing | `cost-budget.md`; needs tenant-class rewrite | partial |
| 172 | Capacity model | no public target | no public target | no public target | `capacity-model.md`; `PRD.md:426-438` | covered target |
| 173 | Migration playbook | import docs | import docs | import docs | `migration-playbooks/from-google-slides-and-powerpoint.md` | covered but stale wording |
| 174 | Engineer onboarding | no counterpart | no counterpart | no counterpart | `onboarding/slides-engineer-first-week.md` | present but stale wording |
| 175 | User tutorial | docs/help | docs/help | help center | `tutorials/build-investor-deck-with-charts-and-collab.md` | present but stale wording |
| 176 | FAQ | docs/help | docs/help | help center | `faqs/slides-engineer-faq.md` | present but stale wording |
| 177 | Rust SDK reference | Google client libs | Graph SDKs | no public broad SDK | `reference-implementations/create-deck-and-export-rust-sdk.md` | covered |
| 178 | SDK plan | Google APIs | Microsoft Graph | limited public API | `sdk-plan.md` | covered, billing terms drift |
| 179 | Manifest | service registry | service registry | service registry | `manifest.json` | covered, tenant class gap |
| 180 | Catalog components | internal registry | internal registry | internal registry | `catalog/*.yaml` | covered |

## Family summary

181. Authoring family verdict: Oyatie target covers the baseline of all three counterparts.
182. Authoring family evidence: `PRD.md:51-62` and `competitor-parity-matrix.md:15-36`.
183. Authoring family gap: offline editing is not first-class, while Pitch documents offline editing.
184. Collaboration family verdict: Oyatie target covers mainstream coauthoring and adds CRDT/no-silent-loss ambition.
185. Collaboration family evidence: `PRD.md:63-66`, `ADR-SLIDES-0001`, and cursor SLO.
186. Collaboration family gap: no runtime source/tests in service path prove the CRDT claims.
187. Present/broadcast family verdict: Oyatie target covers Google Q&A, Microsoft Live-style audience experience, and Pitch presentation sharing.
188. Present/broadcast family evidence: `PRD.md:69-70`, `ADR-SLIDES-0005`, and broadcast SLO.
189. Present/broadcast family gap: live subtitles are not explicit in PRD.
190. Sharing/analytics family verdict: sharing is covered, analytics is under-specified.
191. Sharing/analytics evidence: `PRD.md:77-82`, Pitch analytics source.
192. Sharing/analytics gap: Pitch-like viewed-slide analytics and visitor-consent analytics are not explicit.
193. Import/export family verdict: Oyatie target is stronger than Pitch and Google for ODP/PDF-A/MP4, but Microsoft remains hardest for PPTX.
194. Import/export family evidence: `PRD.md:72-75`, `ADR-SLIDES-0003`.
195. Import/export family gap: subset limitations need customer-facing clarity.
196. Accessibility family verdict: Oyatie target meets or exceeds counterparts, especially reduced-motion policy.
197. Accessibility evidence: `PRD.md:76`, `ADR-SLIDES-0004`, matrix rows 79-85.
198. Accessibility gap: AsyncAPI event drift around `AltTextSuggested` should be fixed.
199. AI family verdict: Oyatie target meets or exceeds all three, with stronger governance claims.
200. AI evidence: `ADR-SLIDES-0006`, `contracts/openapi/slides.yaml:327-360`.
201. AI gap: T0/T1/T2 vocabulary needs policy review because capability vocabulary is being retired.
202. Governance family verdict: Oyatie target is stronger than visible counterpart surfaces.
203. Governance evidence: audit-chain, Cedar, per-pack residency, SLSA, WASM SRI.
204. Governance gap: tenant-class semantics and deployment-context overlays are absent.
205. Deployment family verdict: not counterpart-driven, but canonical Oyatie evidence is insufficient.
206. Deployment evidence: Helm/Kustomize exists.
207. Deployment gap: OpenTofu and six-context directories are missing.

## Headline gap analysis

208. Headline gap H-001: Pitch analytics are not matched by explicit Oyatie link analytics.
209. Evidence H-001: Pitch analytics source; no matching PRD event beyond generic sharing and audit events.
210. Impact H-001: sales and investor-deck workflows may trail Pitch even if authoring parity is strong.
211. Recommended H-001: add viewed-slide, visit-duration, device/browser, link-consent, and external-link analytics events.
212. Headline gap H-002: Pitch offline editing is not represented in Oyatie requirements.
213. Evidence H-002: Pitch offline source; no offline requirement in `PRD.md:51-87`.
214. Impact H-002: field-sales and travel workflows may be weaker than Pitch desktop.
215. Recommended H-002: decide whether offline editing is out of scope or add sync/conflict requirements.
216. Headline gap H-003: Microsoft PPTX fidelity remains the top import/export bar.
217. Evidence H-003: `ADR-SLIDES-0003:150-165` accepts subset round-trip rather than full feature parity.
218. Impact H-003: migrations from complex PowerPoint decks need visible limitations.
219. Recommended H-003: publish the supported PPTX subset and warnings.
220. Headline gap H-004: Google linked charts are a mature parity baseline.
221. Evidence H-004: Google linked chart source; `ADR-SLIDES-0008`.
222. Impact H-004: chart-update latency, revocation cascade, and permission propagation must be product-visible.
223. Recommended H-004: ensure Sheets ACL revocation events are contract-tested against slides embeds.
224. Headline gap H-005: live subtitles are explicit in Microsoft Live Presentations but not in Oyatie PRD.
225. Evidence H-005: Microsoft Present Live source; `PRD.md:69-70` lacks subtitle language.
226. Impact H-005: accessibility and international presentation experience may lag Microsoft.
227. Recommended H-005: add live captions/subtitles if they belong in broadcast scope.
228. Headline gap H-006: API quotas and tenant-class usage caps are absent.
229. Evidence H-006: Google Slides API limits source; no `tenant_class` search hits.
230. Impact H-006: demo-trial infrastructure cannot be bounded cleanly.
231. Recommended H-006: add quota tables by operation, deployment context, and tenant class.
232. Headline gap H-007: service deployability is not product-parity but is canonical Oyatie parity.
233. Evidence H-007: missing OpenTofu context dirs; ADR-0328 OpenTofu requirement.
234. Impact H-007: the service is not coherently deployable across the promised contexts.
235. Recommended H-007: author OpenTofu modules before claiming all-six deployability.

## Additive Oyatie surface

236. Additive A-001: per-slide ACL is a differentiator over all three counterparts.
237. Evidence A-001: `PRD.md:78`; `ADR-SLIDES-0007`; matrix row 48.
238. Additive A-002: named-block ACL is a stronger governance model than deck-level permissions.
239. Evidence A-002: `PRD.md:79`; `ADR-SLIDES-0007`.
240. Additive A-003: audit-chain Ed25519 seals on deck saves, imports, exports, and share changes are stronger than visible counterpart audit surfaces.
241. Evidence A-003: `PRD.md:127-133`; `ADR-SLIDES-0003:181-182`.
242. Additive A-004: EU AI Act risk-class stamping is a unique AI governance surface.
243. Evidence A-004: `competitor-parity-matrix.md:100-102`; `ADR-SLIDES-0006`.
244. Additive A-005: reduced-motion default policy makes accessibility a default rather than an optional preference.
245. Evidence A-005: `ADR-SLIDES-0004`; `competitor-parity-matrix.md:61`.
246. Additive A-006: deterministic MP4 export adds reproducibility beyond normal video export.
247. Evidence A-006: `ADR-SLIDES-0003:82-86`.
248. Additive A-007: PDF/A and PAdES handling targets archival/legal workflows.
249. Evidence A-007: `ADR-SLIDES-0003:72-76`.
250. Additive A-008: per-pack residency and Cedar policies align slides with regulated tenant packs.
251. Evidence A-008: `PRD.md:145-148`; `policy/*.cedar`.
252. Additive A-009: WASM SRI and cargo-leptos chunk signing harden browser delivery.
253. Evidence A-009: `ADR-SLIDES-0002:64-65`; `PRD.md:116`.
254. Additive A-010: chart-link revocation cascade is stronger than standard linked-chart behavior if implemented.
255. Evidence A-010: `competitor-parity-matrix.md:138`; `contracts/asyncapi/slides-events.yaml:72-73`.

## Product families needing explicit no-tenant-class-drift rewrite

256. Rewrite R-001: benchmark family needs no-tenant-class-drift target phrasing.
257. Evidence R-001: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:13-31` uses retired target labels.
258. Rewrite R-002: onboarding family needs no-tenant-class-drift expectations.
259. Evidence R-002: `onboarding/slides-engineer-first-week.md:45-71` uses retired labels.
260. Rewrite R-003: migration family needs tenant-class wording instead of capability-level migration gates.
261. Evidence R-003: `migration-playbooks/from-google-slides-and-powerpoint.md:89-99`.
262. Rewrite R-004: FAQ family needs neutral hardware and capacity wording.
263. Evidence R-004: `faqs/slides-engineer-faq.md:22-74`.
264. Rewrite R-005: tutorial family needs uniform quality and usage-cap wording.
265. Evidence R-005: `tutorials/build-investor-deck-with-charts-and-collab.md:15-164`.
266. Rewrite R-006: old tenant_class adoption record should be retired rather than renamed in place.
267. Evidence R-006: `tenant-class-adoption/tenant-class-adoption-record.md:9-150`.
268. Rewrite R-007: cost budget needs `demo_trial`, `paid`, and `revenue_share` overlays.
269. Evidence R-007: `cost-budget.md:74`; tenant-class directive in current prompt.
270. Rewrite R-008: SDK plan should map per-seat terms to `paid` and revenue-share billing semantics.
271. Evidence R-008: `sdk-plan.md:71`.
272. Rewrite R-009: manifest should carry tenant-class and context-support metadata after schema lands.
273. Evidence R-009: no tenant-class search hits in `manifest.json`.
274. Rewrite R-010: PRD should remove stale open questions already resolved by accepted ADRs.
275. Evidence R-010: `PRD.md:485-487`; `ADR-SLIDES-0002`; `ADR-SLIDES-0003`.

## Final matrix verdict

276. Verdict V-001: Oyatie slides has a broad enough intended feature surface to compete with Google Slides, PowerPoint Online, and Pitch.
277. Verdict V-002: The service exceeds visible counterparts in policy/governance ambitions such as per-slide ACL, named-block ACL, audit-chain seals, AI risk stamps, reduced-motion defaulting, deterministic MP4, and PDF/A/PAdES.
278. Verdict V-003: The service trails Pitch unless link analytics, visitor tracking consent, and viewed-slide engagement analytics become explicit product requirements.
279. Verdict V-004: The service trails Pitch on offline editing unless offline is intentionally out of scope.
280. Verdict V-005: The service trails Microsoft native PowerPoint on full PPTX round-trip by design, and must communicate its supported subset.
281. Verdict V-006: The service can meet Google linked-chart parity only if Sheets ACL revocation and update propagation are contract-tested.
282. Verdict V-007: The service's feature parity is much stronger than its deployability parity.
283. Verdict V-008: The no-tenant-class-drift rewrite is mandatory before existing benchmark/onboarding/tutorial/migration docs are used as current guidance.
284. Verdict V-009: Tenant-class adoption is a product and platform gap, not a counterpart-driven gap.
285. Verdict V-010: The next parity pass should add evidence URLs to every counterpart row, because `competitor-parity-matrix.md:142-146` currently says links are inline but many rows use label-only sources.

## Detailed additive backlog candidates

286. Candidate AB-001: Add `ExternalLinkViewed` event with link id, deck id, slide id, viewer class, consent state, and pack.
287. Candidate AB-002: Add `ExternalLinkEngagementRecorded` event with duration, viewed slide ordinals, device class, and consent status.
288. Candidate AB-003: Add `ExternalLinkConsentChanged` event for Pitch-like privacy controls.
289. Candidate AB-004: Add `DeckGuestInvited` event to distinguish guest collaborator invitations from internal ACL changes.
290. Candidate AB-005: Add `DeckGuestPermissionChanged` event for view/comment/edit role changes.
291. Candidate AB-006: Add `DeckOfflineReplicaCreated` only if offline editing becomes in scope.
292. Candidate AB-007: Add `OfflineReplicaSynced` with conflict count if offline editing becomes in scope.
293. Candidate AB-008: Add `LiveSubtitleStarted` if Microsoft Live subtitle parity enters scope.
294. Candidate AB-009: Add `LiveSubtitleLanguageChanged` for multilingual audience mode.
295. Candidate AB-010: Add `AudienceQuestionHighlighted` for Google Q&A parity.
296. Candidate AB-011: Add `AudienceReactionRecorded` for broadcast reaction telemetry.
297. Candidate AB-012: Add `AudiencePollLinked` for forms-backed poll embedding.
298. Candidate AB-013: Add `ChartLinkRefreshFailed` so Sheets link parity is diagnosable.
299. Candidate AB-014: Add `ChartLinkPermissionDenied` for ACL-driven revocation cascade.
300. Candidate AB-015: Add `PptxUnsupportedFeatureDetected` for import diagnostics.
301. Candidate AB-016: Add `PptxRoundTripSubsetValidated` for fidelity evidence.
302. Candidate AB-017: Add `ExportArtifactSigned` for audit-chain and provenance.
303. Candidate AB-018: Add `AiRiskClassStamped` for AI governance auditability.
304. Candidate AB-019: Add `AiHighRiskRequestRejected` for Annex III refusal evidence.
305. Candidate AB-020: Add `ReducedMotionFallbackEngaged` for accessibility telemetry.
306. Candidate AB-021: Add `TenantUsageCapApproached` for demo-trial usage caps.
307. Candidate AB-022: Add `TenantUsageCapExceeded` for quota enforcement.
308. Candidate AB-023: Add `RevenueShareUsageAccrued` if revenue-share tenants are first-class.
309. Candidate AB-024: Add `PaidSeatEntitlementChanged` to map existing per-seat checks into the new model.
310. Candidate AB-025: Add `BroadcastViewerCapApplied` for constrained deployment contexts.

## Requirement trace against PRD

311. PRD FR-01 deck create/open is matched by matrix rows 46-47.
312. PRD FR-02 slide CRUD is matched by rows 48-50.
313. PRD FR-03 placeholders and layout are matched by rows 61-65.
314. PRD FR-04 text, shape, image, video, and audio are matched by rows 48-55.
315. PRD FR-05 tables and equations are matched by rows 56 and 60.
316. PRD FR-06 charts are matched by rows 57-59.
317. PRD FR-07 themes/templates are matched by rows 61-64.
318. PRD FR-08 speaker notes are matched by row 66.
319. PRD FR-09 real-time collaboration is matched by rows 93-96.
320. PRD FR-10 cursor presence is matched by row 94.
321. PRD FR-11 conflict surfacing is matched by row 95.
322. PRD FR-12 CRDT no-silent-loss is matched by row 93.
323. PRD FR-13 autosave and version history are matched by rows 89-90.
324. PRD FR-14 comments are matched by rows 86-87.
325. PRD FR-15 present mode is matched by rows 67-72.
326. PRD FR-16 animation is matched by rows 21, 119, and 167.
327. PRD FR-17 audience view is matched by row 68.
328. PRD FR-18 Q&A/polls/reactions are matched by rows 69-71.
329. PRD FR-19 broadcast is matched by row 72.
330. PRD FR-20 export is matched by rows 102-110.
331. PRD FR-21 import is matched by rows 102 and 108.
332. PRD FR-22 chart live-link is matched by rows 57-59.
333. PRD FR-23 AI design assist is matched by row 122.
334. PRD FR-24 AI full-deck generation is matched by row 124.
335. PRD FR-25 AI alt-text is matched by row 117.
336. PRD FR-26 accessibility is matched by rows 115-121.
337. PRD FR-27 ACL is matched by rows 83-85.
338. PRD FR-28 share/embed is matched by rows 74-80.
339. PRD FR-29 audit is matched by rows 139 and 240-241.
340. PRD FR-30 residency is matched by rows 140 and 250-251.
341. PRD FR-31 SDK/API is matched by rows 136-138 and 177-178.
342. PRD FR-32 runbooks are matched by rows 161-167.
343. PRD FR-33 cost/capacity is matched by rows 171-172.
344. PRD FR-34 deployment is not matched by evidence, as rows 146-149 remain canonical gaps.

## Closing assessment

345. The feature surface is credible against the three named counterparts.
346. The current artifacts already identify most high-value presentation capabilities.
347. The strongest business gap is Pitch-style analytics and link engagement.
348. The strongest technical parity gap is Microsoft-native PPTX fidelity.
349. The strongest collaboration gap is lack of runtime proof for CRDT claims.
350. The strongest platform gap is lack of OpenTofu context deployability.
351. The strongest doctrine gap is retired vocabulary.
352. The strongest tenant model gap is total absence of `demo_trial`, `paid`, and `revenue_share` semantics.
353. This report should be paired with the performance report for numeric target decisions.
354. This report should be paired with the coherence audit for canonical-direction blocker severity.
355. No capability-tier-delta deliverable is produced.

