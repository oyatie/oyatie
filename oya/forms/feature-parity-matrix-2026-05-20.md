# Forms Feature-Parity Matrix - 2026-05-20

Scope: `microservices/forms/`.
Counterpart set: Google Forms, Typeform, SurveyMonkey.
Purpose: compare the current forms artifact surface against the union of the assigned industry-counterpart capability surfaces.
Tier posture: no retired four-level capability tier rows are introduced; retired local tier language is treated as a finding in `coherence-audit-2026-05-20.md`.
Evidence policy: local claims cite forms artifacts; counterpart claims cite public product, help, or developer documentation.
Batch boundary: this matrix is an audit artifact, not a remediation plan and not a fourth tier-deltas deliverable.

## Source Anchors

Local product purpose: `microservices/forms/PRD.md:30-38`.
Local functional requirements: `microservices/forms/PRD.md:44-68`.
Local acceptance criteria: `microservices/forms/PRD.md:72-101`.
Local contracts: `microservices/forms/contracts/openapi/forms.openapi.yaml:1`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`, `microservices/forms/contracts/proto/forms.proto:12`.
Local previous parity artifact: `microservices/forms/competitor-parity-matrix.md:15-67`.
Google Forms feature source: `https://workspace.google.com/products/forms/:435-488`.
Google Forms question and upload source: `https://support.google.com/docs/answer/7322334:72-122`.
Google Forms sharing/embed source: `https://support.google.com/docs/answer/2839588:100-138`.
Google Forms quiz source: `https://support.google.com/docs/answer/7032287:87-126`.
Typeform feature and limits source: `https://www.typeform.com/platform-overview:122-210`, `https://www.typeform.com/pricing:389-411`, `https://www.typeform.com/pricing:1928-2048`.
Typeform API and logic source: `https://www.typeform.com/developers/get-started/:155-176`, `https://www.typeform.com/developers/create/logic-jumps/:220-356`.
Typeform response-limit source: `https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:9-45`.
SurveyMonkey creation source: `https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:341-423`.
SurveyMonkey logic source: `https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:336-357`.
SurveyMonkey API/limit source: `https://api.surveymonkey.com/v3/docs:219-239`.
SurveyMonkey response-limit source: `https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:335-355`.

## 1. Counterpart 1 - Google Forms Capability Surface

GF-001. Google Forms positions itself as an online forms and surveys product for gathering data and insights from anywhere (`https://workspace.google.com/products/forms/:401-414`).
GF-002. Google Forms supports multiple question types and drag-and-drop organization (`https://workspace.google.com/products/forms/:435-437`).
GF-003. Google Forms supports easy sharing by email, social media, or website embed (`https://workspace.google.com/products/forms/:435-437`).
GF-004. Google Forms supports templates for surveys and questionnaires (`https://workspace.google.com/products/forms/:449-451`).
GF-005. Google Forms supports custom flows through logic based on previous answers (`https://workspace.google.com/products/forms/:455-458`).
GF-006. Google Forms supports quiz creation for knowledge testing (`https://workspace.google.com/products/forms/:455-458`).
GF-007. Google Forms supports theme and brand customization through colors, images, and fonts (`https://workspace.google.com/products/forms/:460-462`).
GF-008. Google Forms supports responses from any device (`https://workspace.google.com/products/forms/:466-468`).
GF-009. Google Forms supports real-time response visualization (`https://workspace.google.com/products/forms/:472-478`).
GF-010. Google Forms supports raw-data export to Google Sheets (`https://workspace.google.com/products/forms/:481-483`).
GF-011. Google Forms supports collaborative analysis in the same product family (`https://workspace.google.com/products/forms/:486-488`).
GF-012. Google Forms states secure-by-default posture and malware protection (`https://workspace.google.com/products/forms/:497-505`).
GF-013. Google Forms states encryption in transit and at rest for Forms data and uploaded files in Drive (`https://workspace.google.com/products/forms/:508-510`).
GF-014. Google Forms supports file upload questions with Google-account sign-in and Drive storage (`https://support.google.com/docs/answer/7322334:72-77`).
GF-015. Google Forms lets form owners specify file types for upload questions (`https://support.google.com/docs/answer/7322334:78-81`).
GF-016. Google Forms lets form owners set maximum number of uploaded files (`https://support.google.com/docs/answer/7322334:78-81`).
GF-017. Google Forms lets form owners choose maximum upload file size (`https://support.google.com/docs/answer/7322334:78-81`).
GF-018. Google Forms supports linear scale questions (`https://support.google.com/docs/answer/7322334:83-89`).
GF-019. Google Forms supports rating questions (`https://support.google.com/docs/answer/7322334:91-98`).
GF-020. Google Forms supports multiple-choice grid questions (`https://support.google.com/docs/answer/7322334:100-104`).
GF-021. Google Forms supports checkbox grid questions (`https://support.google.com/docs/answer/7322334:106-109`).
GF-022. Google Forms supports date questions (`https://support.google.com/docs/answer/7322334:111-115`).
GF-023. Google Forms supports time and duration questions (`https://support.google.com/docs/answer/7322334:117-122`).
GF-024. Google Forms can email published forms to responders (`https://support.google.com/docs/answer/2839588:98-100`).
GF-025. Google Forms restricts email embed when a form includes file upload, rating, images, or secured quiz elements (`https://support.google.com/docs/answer/2839588:102-108`).
GF-026. Google Forms supports responder links and shortened URLs (`https://support.google.com/docs/answer/2839588:109-119`).
GF-027. Google Forms supports pre-filled answer links (`https://support.google.com/docs/answer/2839588:121-129`).
GF-028. Google Forms supports website/blog embed via generated HTML (`https://support.google.com/docs/answer/2839588:131-138`).
GF-029. Google Forms supports individual quiz grading (`https://support.google.com/docs/answer/7032287:78-86`).
GF-030. Google Forms supports quiz result summaries (`https://support.google.com/docs/answer/7032287:87-92`).
GF-031. Google Forms supports question-by-question quiz grading with full, partial, or no points (`https://support.google.com/docs/answer/7032287:93-103`).
GF-032. Google Forms supports immediate or later grade release (`https://support.google.com/docs/answer/7032287:104-119`).
GF-033. Google Forms supports emailing released scores (`https://support.google.com/docs/answer/7032287:120-126`).
GF-034. Google Forms API quotas include 975 read requests per minute per project and 390 read requests per minute per user per project (`https://developers.google.com/workspace/forms/api/limits:224-228`).
GF-035. Google Forms API quotas include 450 expensive read requests per minute per project and 180 per minute per user per project (`https://developers.google.com/workspace/forms/api/limits:229-235`).
GF-036. Google Forms API quotas include 375 write requests per minute per project and 150 per minute per user per project (`https://developers.google.com/workspace/forms/api/limits:236-238`).
GF-037. Google Forms recommends exponential backoff for quota errors (`https://developers.google.com/workspace/forms/api/limits:239-255`).
GF-038. Google Forms strength: low-friction creation, sharing, Sheets integration, quizzes, and Workspace trust.
GF-039. Google Forms gap against Oyatie ambition: no public evidence of self-hosted, pack-bound, per-question policy-governed deployment.
GF-040. Google Forms gap against Oyatie ambition: no public evidence of OpenTofu deployment to customer-owned AWS, OCI, on-prem, colo, and Oyatie-as-provider contexts.

## 2. Counterpart 2 - Typeform Capability Surface

TF-001. Typeform positions itself around forms, surveys, and quizzes with engagement-focused collection (`https://www.typeform.com/platform-overview:122-125`).
TF-002. Typeform exposes REST APIs for Create, Responses, and Webhooks using JSON (`https://www.typeform.com/developers/get-started/:162-164`).
TF-003. Typeform API base URL is `https://api.typeform.com/`, with EU data-center base URLs for configured accounts (`https://www.typeform.com/developers/get-started/:167-169`).
TF-004. Typeform Create and Responses APIs are limited to two requests per second per account (`https://www.typeform.com/developers/get-started/:172-176`).
TF-005. Typeform account requirements say some features require paid accounts, including hidden fields, webhooks, payments, and some embed modes (`https://www.typeform.com/developers/get-started/:155-160`).
TF-006. Typeform response limits depend on plan and reset monthly (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:9-14`).
TF-007. Typeform response limits apply across all forms in the account, not per form (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:13-14`).
TF-008. Typeform warns at 90 percent and 100 percent of response limit (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:33-37`).
TF-009. Typeform makes forms private after response-limit exhaustion until renewal or upgrade (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:37-38`).
TF-010. Typeform pricing source lists plans with seats and response counts, including 5 seats and 10k+ responses per month in a Growth plan context (`https://www.typeform.com/pricing:389-395`).
TF-011. Typeform pricing source lists custom seats and 20k+ responses per month for Growth Custom (`https://www.typeform.com/pricing:447-453`).
TF-012. Typeform supports response enrichment and AI-assisted insights (`https://www.typeform.com/pricing:397-411`, `https://www.typeform.com/pricing:1928-1930`).
TF-013. Typeform supports video questions and video answers in listed feature surfaces (`https://www.typeform.com/pricing:401-407`).
TF-014. Typeform supports reCAPTCHA (`https://www.typeform.com/pricing:407-410`).
TF-015. Typeform supports file uploads with published storage amounts in plan comparison (`https://www.typeform.com/pricing:1942-1962`).
TF-016. Typeform supports payment questions through Stripe (`https://www.typeform.com/pricing:1963-1966`).
TF-017. Typeform supports partial response collection, with additional constraints when logic or scoring is used (`https://www.typeform.com/pricing:1982-2002`).
TF-018. Typeform supports AI clarification questions (`https://www.typeform.com/pricing:2020-2022`).
TF-019. Typeform supports video answers (`https://www.typeform.com/pricing:2035-2037`).
TF-020. Typeform advertises GDPR compliance in its feature comparison (`https://www.typeform.com/pricing:2045-2048`).
TF-021. Typeform supports SSO with providers such as Okta, OneLogin, Azure, PingFederate, and other SAML/OAuth/OpenID providers (`https://www.typeform.com/pricing:2065-2067`).
TF-022. Typeform Logic Jump definitions are unique per triggering field (`https://www.typeform.com/developers/create/logic-jumps/:220-222`).
TF-023. Typeform Logic Jump operators depend on field type, including text, numeric, choice, date, file upload, and hidden (`https://www.typeform.com/developers/create/logic-jumps/:353-356`).
TF-024. Typeform logic can evaluate hidden values and variables, supporting personalization (`https://www.typeform.com/developers/create/logic-jumps/:353-356`).
TF-025. Typeform strength: polished respondent experience, conversational flow, logic, payments, hidden fields, video, and AI features.
TF-026. Typeform strength: clear external developer API and webhooks surface.
TF-027. Typeform gap against Oyatie ambition: account-level API quotas are much lower than Google Forms developer quotas and below Oyatie service targets.
TF-028. Typeform gap against Oyatie ambition: public docs do not evidence customer-owned deployability.
TF-029. Typeform gap against Oyatie ambition: no public OpenTofu multi-context deployment model is exposed.
TF-030. Typeform gap against Oyatie ambition: plan-based response limits are commercial plan controls, not tenant_class infrastructure profiles.

## 3. Counterpart 3 - SurveyMonkey Capability Surface

SM-001. SurveyMonkey supports starting from templates, including 500+ expert-built templates in the help source (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:341-343`).
SM-002. SurveyMonkey supports a full design editor with features available by plan (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:373-375`).
SM-003. SurveyMonkey supports copying existing surveys (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:386-391`).
SM-004. SurveyMonkey supports AI survey creation from a prompt (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:393-396`).
SM-005. SurveyMonkey supports popular templates and team templates (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:398-401`).
SM-006. SurveyMonkey supports pasted questions as a creation path (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:403-405`).
SM-007. SurveyMonkey supports building target audiences as a creation path (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:407`).
SM-008. SurveyMonkey supports a full editor for custom and complex surveys (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:416-417`).
SM-009. SurveyMonkey supports adding or editing questions, page elements, and survey options in preview (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:418-420`).
SM-010. SurveyMonkey supports a Question Bank (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:418-420`).
SM-011. SurveyMonkey supports logic to personalize survey experience (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:421-423`).
SM-012. SurveyMonkey supports page skip logic (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:336-338`).
SM-013. SurveyMonkey supports question skip logic (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:339-340`).
SM-014. SurveyMonkey supports disqualification logic (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:339-340`).
SM-015. SurveyMonkey supports advanced branching based on answers, contacts custom data, custom variables, or language (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:342-342`).
SM-016. SurveyMonkey supports same-page logic through advanced branching (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:342-355`).
SM-017. SurveyMonkey supports randomization of pages, questions, and blocks (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:342-355`).
SM-018. SurveyMonkey supports carry-forward responses (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:342-355`).
SM-019. SurveyMonkey supports quotas to close surveys at response ratios (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:342-355`).
SM-020. SurveyMonkey supports custom variables (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:355-355`).
SM-021. SurveyMonkey supports piping question or answer text and metadata (`https://help.surveymonkey.com/en/surveymonkey/create/logic-options/:357-357`).
SM-022. SurveyMonkey API basic accounts can retrieve up to 25 responses per survey for detailed response scope (`https://api.surveymonkey.com/v3/docs:206-214`).
SM-023. SurveyMonkey paid plans have unlimited response access for that scope (`https://api.surveymonkey.com/v3/docs:206-214`).
SM-024. SurveyMonkey draft and private apps start with 120 requests per minute and 500 requests per day (`https://api.surveymonkey.com/v3/docs:219-231`).
SM-025. SurveyMonkey API has a maximum page size of 1000 resources (`https://api.surveymonkey.com/v3/docs:233-239`).
SM-026. SurveyMonkey API documents a maximum survey size of 1000 questions before over-limit surveys return 413 (`https://api.surveymonkey.com/v3/docs:233-239`).
SM-027. SurveyMonkey response limits differ by plan and billing cycle (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:335-341`).
SM-028. SurveyMonkey over-limit responses may be deleted after one year if not retained under plan or support exception (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:339-344`).
SM-029. SurveyMonkey self-serve billing can charge per overage response and reset count by billing cycle (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:345-352`).
SM-030. SurveyMonkey enterprise plans can have annual response limits (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:353-355`).
SM-031. SurveyMonkey strength: survey research depth, templates, question bank, audience acquisition, advanced branching, quotas, piping, and enterprise response governance.
SM-032. SurveyMonkey gap against Oyatie ambition: public API limits are materially constrained for high-throughput service integration.
SM-033. SurveyMonkey gap against Oyatie ambition: public docs do not evidence customer-owned deployment contexts.
SM-034. SurveyMonkey gap against Oyatie ambition: self-hosted, pack-bound, per-question data classification is not a public default surface.

## 4. Union-Coverage Matrix

Legend: `Covered` means the forms artifacts contain a direct requirement, contract, runbook, or doc surface. `Partial` means evidence exists but is incomplete or contradicted. `Gap` means the audit found no service-local evidence. `Retire` means coverage depends on retired tier language and must be rewritten.

| # | Union capability | Google Forms | Typeform | SurveyMonkey | Oyatie forms status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 001 | Core form builder | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 002 | Multiple field types | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 003 | Section/page structure | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:45` |
| 004 | Drag/drop builder expectation | Yes | Yes | Yes | Partial | PRD requires builder, UX mechanics under-specified: `microservices/forms/PRD.md:44` |
| 005 | Public respondent renderer | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:47`, `microservices/forms/IP-011-form-renderer.md` |
| 006 | Responsive/mobile access | Yes | Yes | Yes | Partial | Device support implied by web renderer; no OS/device manifest. |
| 007 | Templates | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:64` |
| 008 | AI form generation | No public core default | Yes | Yes | Covered | `microservices/forms/PRD.md:65`, `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md` |
| 009 | Prompt-to-form constraints | Limited | Yes | Yes | Covered with retired vocabulary | `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md:53-57` |
| 010 | Logic jumps | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:45`, `microservices/forms/decisions/ADR-FORMS-0004-conditional-logic-and-branching-engine.md` |
| 011 | Advanced branching | Basic section logic | Yes | Yes | Covered | `microservices/forms/PRD.md:45` |
| 012 | Same-page conditional display | Not primary | Yes | Yes | Partial | `microservices/forms/decisions/ADR-FRM-001-logic-jump-evaluator-with-conditional-cedar-permit-per-question.md` |
| 013 | Hidden fields / URL parameters | Pre-fill links | Yes | Yes | Covered | `microservices/forms/PRD.md:60` |
| 014 | Validation rules | Basic | Yes | Yes | Covered | `microservices/forms/PRD.md:46`, `microservices/forms/IP-004-validation-engine.md` |
| 015 | Calculation fields | Limited | Yes | Yes | Covered but retired wording in migration doc | `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:111` |
| 016 | Quiz mode | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:58` |
| 017 | Quiz answer key | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:58` |
| 018 | Partial credit | Yes | Yes | Yes | Partial | PRD has quiz scoring, not detailed partial-credit rules. |
| 019 | Score release controls | Yes | Yes | Yes | Partial | No explicit score-release contract found. |
| 020 | File upload field | Yes | Yes | Paid plans | Covered | `microservices/forms/PRD.md:50`, `microservices/forms/faqs/forms-engineer-faq.md:92-94` |
| 021 | Upload type limits | Yes | Yes | Yes | Partial | Upload policy is tiered and needs tenant_class rewrite. |
| 022 | Upload malware scanning | Yes via Google security | Not core evidence | Not core evidence | Covered | `microservices/forms/PRD.md:50`, `microservices/forms/IP-001-layer-a-postgres-valkey-meilisearch-clamav-waf-cdn-captcha-iac.md` |
| 023 | Signature field | No | Add-on/integration class | Add-on/integration class | Covered | `microservices/forms/PRD.md:52`, `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md` |
| 024 | Payment field | No native Stripe | Yes | Integrations | Covered but dependency drift | `microservices/forms/PRD.md:53` |
| 025 | Location/address fields | Limited | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 026 | Matrix/grid questions | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 027 | Rating fields | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 028 | Date/time fields | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 029 | Rich content blocks | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 030 | Image choice | Limited | Yes | Yes | Covered | `microservices/forms/PRD.md:44` |
| 031 | Form themes | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:49` |
| 032 | Brand customization | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:49` |
| 033 | Custom domain | Workspace/site path | Yes | Yes | Partial | PRD has embed and CDN; no custom-domain contract in forms. |
| 034 | Email sharing | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:62` |
| 035 | Link sharing | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:62` |
| 036 | Website embed | Yes | Yes | Yes | Covered with missing CSP doc | `microservices/forms/PRD.md:50` |
| 037 | Social sharing | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:62` |
| 038 | SMS distribution | No core | Integrations | Integrations | Covered with retired wording | `microservices/forms/PRD.md:63` |
| 039 | QR/kiosk/offline collection | Limited | Limited | Survey modes | Gap | No local forms evidence found. |
| 040 | Response collection | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:47`, `microservices/forms/IP-012-response-collector-rest.md` |
| 041 | Duplicate prevention | Yes in settings | Yes | Yes | Partial | Not directly evidenced in PRD lines reviewed. |
| 042 | Captcha/anti-spam | Google abuse controls | reCAPTCHA | Anti-fraud controls | Covered with retired wording | `microservices/forms/PRD.md:57`, `microservices/forms/decisions/ADR-FORMS-0002-captcha-and-anti-spam.md` |
| 043 | WAF/CDN edge | Google platform | Typeform platform | SurveyMonkey platform | Partial | Terraform/Kustomize evidence exists but canonical OpenTofu missing. |
| 044 | Realtime charts | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:55` |
| 045 | Cross-tab analytics | Limited | Insights | Strong survey analysis | Covered | `microservices/forms/PRD.md:55` |
| 046 | AI insights | Gemini-adjacent Workspace | Yes | Yes | Covered | `microservices/forms/PRD.md:65` |
| 047 | Export CSV | Yes via Sheets/export | Yes | Yes | Covered | `microservices/forms/PRD.md:56`, `microservices/forms/slos/export-csv-latency.openslo.yaml` |
| 048 | Export XLSX | Yes via Sheets | Yes | Yes | Covered | `microservices/forms/PRD.md:56` |
| 049 | Export PDF | Add-ons | Yes | Yes | Partial | Not primary in PRD excerpt. |
| 050 | Warehouse export | No | Integrations | Enterprise exports | Covered | `microservices/forms/PRD.md:56`, `microservices/forms/IP-014-export-worker.md` |
| 051 | Webhooks/events | Forms API watches | Yes | Yes | Covered | `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`, `microservices/forms/PRD.md:175-185` |
| 052 | REST API | Yes | Yes | Yes | Covered | `microservices/forms/contracts/openapi/forms.openapi.yaml:1` |
| 053 | Bulk response API | Yes | Yes | Yes | Covered | `microservices/forms/PRD.md:56`, `microservices/forms/contracts/openapi/forms.openapi.yaml` |
| 054 | Per-question data class | No public default | No public default | No public default | Covered | `microservices/forms/PRD.md:96` |
| 055 | Consent binding | Basic | GDPR features | Compliance features | Covered | `microservices/forms/PRD.md:97`, `microservices/forms/dpia.md` |
| 056 | DSAR support | Workspace compliance | GDPR tools | Enterprise/legal support | Covered | `microservices/forms/PRD.md:98` |
| 057 | Data residency | Workspace regional controls | EU data center API option | regional API access URL | Covered | `microservices/forms/policy/data-residency.md`, `microservices/forms/PRD.md:94` |
| 058 | Per-pack compliance | No public default | Limited | Enterprise plans | Covered but tier wording | `microservices/forms/compliance.md` |
| 059 | Audit-chain anchoring | Admin/audit logs | Enterprise logs | Enterprise logs | Covered | `microservices/forms/PRD.md:99` |
| 060 | Role/admin scope | Workspace IAM | SSO/roles | team roles | Covered | `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/tenant-scope.cedar` |
| 061 | SSO support | Workspace | Yes | Enterprise | Partial | Identity dependency listed; forms-local SSO flow not explicit. |
| 062 | Team collaboration | Yes | Yes | Yes | Partial | Collaboration implied through Oyatie workspace, not deeply specified. |
| 063 | Admin analytics | Workspace admin | Typeform workspace | SurveyMonkey team admin | Partial | Dashboards exist, admin UX not specified. |
| 064 | Templates marketplace | Google templates | Typeform templates | SurveyMonkey templates | Partial | PRD requires templates; marketplace governance not shown. |
| 065 | Question bank | Limited templates | templates | Yes | Gap | No question-bank artifact found. |
| 066 | Audience panel | No | Limited | Yes | Gap | No audience acquisition surface found. |
| 067 | Quotas | API quotas | response limits | survey quotas | Partial | Usage caps absent for tenant_class. |
| 068 | Response-limit billing | Workspace plan | plan limits | overage billing | Gap | Tenant_class billing model absent. |
| 069 | Per-seat billing support | Workspace seats | seats | team seats | Gap | Forms cost model has old tier language, no tenant_class paid model. |
| 070 | Usage billing support | Workspace quotas | response add-ons | response overages | Gap | No canonical paid usage meter model in forms. |
| 071 | Revenue-share model | No | No | No | Gap | No `revenue_share` adoption in forms. |
| 072 | Demo/trial caps | Free Google account | free/limited plans | free plan | Gap | No `demo_trial` adoption in forms. |
| 073 | OCI Always Free profile | No | No | No | Gap | No `iac/oci-guest/always-free/` in forms. |
| 074 | Public-cloud deployment | Google-hosted | Typeform-hosted | SurveyMonkey-hosted | Gap | No `iac/oyatie-public-cloud/` OpenTofu module. |
| 075 | Guest AWS deployment | No public default | No public default | No public default | Gap | No `iac/guest-on-aws/` OpenTofu module. |
| 076 | Guest OCI deployment | No public default | No public default | No public default | Gap | No `iac/oci-guest/` OpenTofu module. |
| 077 | On-prem deployment | No public default | No public default | No public default | Gap | No `iac/on-prem/` OpenTofu module. |
| 078 | Colo deployment | No public default | No public default | No public default | Gap | No `iac/colo/` OpenTofu module. |
| 079 | Oyatie-as-cloud-provider deployment | No | No | No | Gap | No `iac/oyatie-iaas/` OpenTofu module. |
| 080 | Supported OS manifest | SaaS opaque | SaaS opaque | SaaS opaque | Gap | No `supported-oses.json`. |
| 081 | Rust backend policy | Not applicable | Not applicable | Not applicable | Partial | No forbidden files; no service-local Rust source tree. |
| 082 | Leptos/WASM builder | Not applicable | No public | No public | Covered in plan | `microservices/forms/IP-010-form-builder-leptos-wasm.md` |
| 083 | Accessibility SLO | Product accessibility | product accessibility | product accessibility | Covered | `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml` |
| 084 | Performance SLOs | Google platform | Typeform platform | SurveyMonkey platform | Covered | `microservices/forms/PRD.md:103-115` |
| 085 | Failure-mode docs | SaaS internal | SaaS internal | SaaS internal | Covered | `microservices/forms/failure-modes.md` |
| 086 | Incident runbooks | SaaS internal | SaaS internal | SaaS internal | Covered | `microservices/forms/runbooks/` |
| 087 | Missing referenced runbook check | Not applicable | Not applicable | Not applicable | Gap | `microservices/forms/faqs/forms-engineer-faq.md:75` |
| 088 | Embed CSP policy | Google platform | embed SDK | survey embed | Partial | PRD references absent `policy/embed-csp.md`. |
| 089 | AI conformance doc | Google AI policies | Typeform AI docs | SurveyMonkey AI docs | Gap | PRD references absent `legal/ai-act-conformity.md`. |
| 090 | Async events | API watches | webhooks | webhooks | Covered | `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1` |
| 091 | Contract version consistency | Google docs | Typeform docs | SurveyMonkey docs | Partial | PRD says AsyncAPI 3.0; file declares 3.1.0. |
| 092 | Dependency registry coherence | Workspace suite | Typeform suite | Momentive suite | Partial | PRD, manifest, and architecture disagree. |
| 093 | Ontology projection | No public default | No public default | No public default | Partial | PRD claims writes; manifest has empty projections. |
| 094 | Policy evaluation per field | No public default | No public default | No public default | Covered | `microservices/forms/decisions/ADR-FRM-001-logic-jump-evaluator-with-conditional-cedar-permit-per-question.md` |
| 095 | Pack-bound signature | No public default | No public default | No public default | Covered | `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md` |
| 096 | Payment dependency coherence | No | Stripe | integrations | Partial | PRD references fintech; manifest omits it. |
| 097 | File-storage dependency coherence | Drive | Typeform storage | SurveyMonkey storage | Partial | PRD references drive; manifest omits drive. |
| 098 | Message distribution coherence | email/social | email/embed | email/collector | Partial | PRD references mail/messenger; manifest omits mail/messenger. |
| 099 | Workflow triggers | add-ons/scripts | webhooks | integrations | Covered | `microservices/forms/PRD.md:175-185` |
| 100 | Public content publishing | embed/link | embed/link | web link | Partial | public-content dependency not clearly registered. |
| 101 | Response search | Sheets/filter | dashboards | analytics | Covered | `microservices/forms/IP-008-meilisearch-adapter.md` |
| 102 | Response cache | SaaS internal | SaaS internal | SaaS internal | Covered | `microservices/forms/IP-007-valkey-adapter.md` |
| 103 | Encrypted response store | Google encryption | Typeform security | SurveyMonkey security | Covered | `microservices/forms/IP-006-postgres-citus-adapter-with-column-encryption.md` |
| 104 | Data retention controls | Workspace admin | plan/workspace | plan/compliance | Covered | `microservices/forms/PRD.md:93` |
| 105 | Localization | Google global | Typeform global | SurveyMonkey languages | Partial | not deeply evidenced in forms docs reviewed. |
| 106 | Multi-region posture | Google platform | Typeform platform | SurveyMonkey data centers | Covered but deployment gap | `microservices/forms/multi-region.md` |
| 107 | Cost model | Workspace pricing | Typeform pricing | SurveyMonkey pricing | Partial | `microservices/forms/cost-budget.md` uses retired tier model. |
| 108 | Onboarding guide | help center | help center | help center | Covered | `microservices/forms/onboarding/forms-engineer-first-week.md` |
| 109 | Migration guide | import/manual | import/tools | import/tools | Covered | `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md` |
| 110 | Reference implementation | Apps Script/API docs | API examples | API examples | Covered | `microservices/forms/reference-implementations/submit-form-and-export-rust-sdk.md` |

## 5. Family Summary

Google Forms defines the low-friction baseline: create, customize, share, embed, collect, visualize, export to Sheets, and quiz.
Typeform defines the engagement and workflow baseline: conversational UX, logic, hidden fields, payments, file uploads, partial responses, AI features, API, webhooks, and account-level limits.
SurveyMonkey defines the survey-research baseline: templates, Question Bank, audience acquisition, skip logic, advanced branching, quotas, piping, response governance, and enterprise limits.
Oyatie forms already aims beyond the union on governance: per-question data classes, pack-bound compliance, audit-chain anchoring, policy evaluation, workflow events, ontology writes, and self-hostable deployment contexts.
Oyatie forms is weaker than the union where artifacts are missing rather than where ambition is missing.
The first weakness is canonical deployability: no OpenTofu six-context module set is present.
The second weakness is commercialization semantics: there is no tenant_class model for demo_trial, paid, or revenue_share.
The third weakness is retired tier residue: multiple local files still model feature or performance differences through tiers.
The fourth weakness is response-market maturity: no SurveyMonkey-style audience panel or Question Bank is evidenced.
The fifth weakness is frontend/UX detail: builder UX, collaboration, preview, mobile device expectations, and question-bank ergonomics are not specified to counterpart depth.
The sixth weakness is benchmark provenance: prior numbers are useful dimensions but use retired tiered hardware terms and lack public-methodology disclosure.
The strongest Oyatie differentiator is sovereign/governed deployment, but that differentiator is currently blocked by missing canonical IaC.
The second strongest differentiator is per-question policy/data-class control, but registry and contract evidence still need alignment.
The third strongest differentiator is AI form generation under bounds, but the API vocabulary needs post-tier migration.

## 6. Headline Gap Analysis

Gap 01: all six deployment contexts are unsupported by service-local OpenTofu evidence.
Gap 02: OCI Always Free profile is absent, so demo/trial infrastructure cannot be audited.
Gap 03: tenant_class semantics are absent across manifest, contracts, policy, cost, and capacity.
Gap 04: retired tier language remains in direct user-facing and implementation-facing docs.
Gap 05: Question Bank is not evidenced, leaving a SurveyMonkey parity gap.
Gap 06: audience panel or target audience sourcing is not evidenced, leaving a SurveyMonkey parity gap.
Gap 07: partial-response collection is not clearly specified, leaving a Typeform parity gap.
Gap 08: score release controls are not clearly specified, leaving a Google Forms quiz parity gap.
Gap 09: response-limit billing and overage behavior are not expressed in the replacement tenant_class model.
Gap 10: custom-domain publication is not specified to Typeform/enterprise forms depth.
Gap 11: embed CSP policy is referenced but absent, weakening public embed parity.
Gap 12: AI Act conformity is referenced but absent, weakening AI feature governance.
Gap 13: dependency registry drift weakens integration claims for payments, storage, messaging, and AI providers.
Gap 14: supported OS manifest is absent, blocking the canonical OS dimension.
Gap 15: no service-local Rust source tree proves implementation completeness.
Gap 16: broad local competitor matrix should remain separate from assigned batch top-three matrix.
Gap 17: API version prose drift should be fixed before SDK consumers rely on the docs.
Gap 18: ontology projection registry must match PRD claims before semantic graph features are credible.
Gap 19: upload limits should be rewritten as tenant_class usage/cost/compliance constraints, not capability tiers.
Gap 20: performance targets should be expressed as one industry-leader target set with deployment-context overlays.

## 7. Additive Surface for Oyatie Forms

Additive 01: keep per-question data classification as an Oyatie differentiator.
Additive 02: keep Cedar policy evaluation for conditional logic and response access.
Additive 03: keep audit-chain anchoring for submissions, exports, AI build, and signature flows.
Additive 04: keep pack-bound consent, retention, residency, and DSAR semantics.
Additive 05: keep response-store encryption and field-level PII protection.
Additive 06: keep Rust backend and Leptos/WASM web architecture.
Additive 07: keep self-hostable deployment ambition, but require OpenTofu proof before claiming it.
Additive 08: add a canonical Question Bank surface if SurveyMonkey parity is in scope.
Additive 09: add audience sourcing or explicitly mark it out of scope with customer-impact rationale.
Additive 10: add score-release and quiz-grade workflow controls to match Google Forms depth.
Additive 11: add partial-response capture semantics to match Typeform depth.
Additive 12: add custom-domain publication and embed governance.
Additive 13: add tenant_class usage caps and billing meters.
Additive 14: add revenue_share support for seller/operator/reseller/affiliate use cases.
Additive 15: add OCI Always Free demo/trial deployment profile.
Additive 16: add supported OS evidence for all canonical OS families.
Additive 17: replace retired tier vocabulary in contracts with tenant_class plus policy entitlement semantics.
Additive 18: reconcile manifest dependencies with PRD and architecture before implementation.
Additive 19: add benchmark methodology that cites public counterpart limits separately from estimated latency targets.
Additive 20: preserve the current feature ambition, but move canonical substrate and model alignment ahead of more feature expansion.

## 8. Coverage Notes by Capability Family

CF-001. Builder family status: PRD has builder scope, but implementation evidence is planning-level, not source-level.
CF-002. Builder family evidence: `microservices/forms/PRD.md:44` names form-builder features.
CF-003. Builder family counterpart pressure: Google Forms emphasizes multiple question types and drag-and-drop organization (`https://workspace.google.com/products/forms/:435-437`).
CF-004. Builder family gap: forms needs a builder interaction spec or UI contract to match counterpart ergonomics.
CF-005. Logic family status: core parity is strong on paper.
CF-006. Logic family evidence: `microservices/forms/PRD.md:45` and ADR-FORMS-0004 cover branching and conditional logic.
CF-007. Logic family counterpart pressure: Typeform Logic Jumps support field, hidden, variable, constant, and end conditions (`https://www.typeform.com/developers/create/logic-jumps/:353-356`).
CF-008. Logic family gap: the forms contract should state whether hidden fields, URL parameters, variables, and same-page logic are all supported.
CF-009. Quiz family status: core quiz mode is present but release semantics are thin.
CF-010. Quiz family evidence: `microservices/forms/PRD.md:58` names quiz mode and auto-grading.
CF-011. Quiz family counterpart pressure: Google Forms supports individual grading, summary grading, question-by-question grading, partial points, and score release controls (`https://support.google.com/docs/answer/7032287:78-126`).
CF-012. Quiz family gap: forms needs explicit grade release, manual review, partial credit, and feedback semantics.
CF-013. Upload family status: upload exists but quota model is retired.
CF-014. Upload family evidence: `microservices/forms/PRD.md:50` names file upload, scan, and CSP policy need.
CF-015. Upload family counterpart pressure: Google Forms lets owners set file type, count, and size (`https://support.google.com/docs/answer/7322334:78-81`).
CF-016. Upload family counterpart pressure: Typeform pricing publishes file upload storage variants (`https://www.typeform.com/pricing:1942-1962`).
CF-017. Upload family gap: forms needs tenant_class usage caps and storage meters instead of retired capability tiers.
CF-018. Payment family status: PRD includes payments, but dependency authority is inconsistent.
CF-019. Payment family evidence: `microservices/forms/PRD.md:53` names payment bridge behavior.
CF-020. Payment family counterpart pressure: Typeform supports Stripe payment questions (`https://www.typeform.com/pricing:1963-1966`).
CF-021. Payment family gap: forms manifest should register the payment dependency or route payments through workflow-owned handoff.
CF-022. Signature family status: forms has a deeper governed signature model than the top-three default surfaces.
CF-023. Signature family evidence: `microservices/forms/PRD.md:52` and ADR-FORMS-0006 cover signature intent.
CF-024. Signature family gap: signature eligibility must be rewritten away from tenant-tier language.
CF-025. Distribution family status: email, link, embed, social, QR, and SMS are uneven.
CF-026. Distribution family evidence: `microservices/forms/PRD.md:62-63` names distribution channels.
CF-027. Distribution family counterpart pressure: Google Forms supports email, responder links, pre-filled links, and website embed (`https://support.google.com/docs/answer/2839588:98-138`).
CF-028. Distribution family gap: forms needs explicit QR, SMS provider, custom domain, and embed CSP policy surfaces.
CF-029. Analytics family status: response analytics and dashboards are present.
CF-030. Analytics family evidence: `microservices/forms/PRD.md:55` and dashboard JSON files cover analytics.
CF-031. Analytics family counterpart pressure: Google Forms emphasizes automated real-time charts and Sheets export (`https://workspace.google.com/products/forms/:472-483`).
CF-032. Analytics family gap: forms needs a clearer line between interactive analytics, warehouse export, and governed reporting.
CF-033. Export family status: strong target numbers exist.
CF-034. Export family evidence: `microservices/forms/PRD.md:56`, `microservices/forms/PRD.md:110-111`, and export SLOs cover CSV/XLSX.
CF-035. Export family gap: missing warehouse-export lag runbook weakens operations parity.
CF-036. API family status: strong contract presence with vocabulary drift.
CF-037. API family evidence: OpenAPI, AsyncAPI, and protobuf contracts all exist.
CF-038. API family counterpart pressure: Google Forms publishes per-minute project/user quotas; Typeform and SurveyMonkey publish 2 rps style API limits.
CF-039. API family gap: API model should drop retired tier fields before SDK stabilization.
CF-040. Webhook/event family status: event surface is present.
CF-041. Webhook/event family evidence: `microservices/forms/PRD.md:175-185` and AsyncAPI cover events.
CF-042. Webhook/event family gap: event payloads still need tenant_class and entitlement semantics.
CF-043. Governance family status: Oyatie forms is strongest here.
CF-044. Governance family evidence: `microservices/forms/PRD.md:93-101`, `microservices/forms/dpia.md`, and policy files cover compliance posture.
CF-045. Governance family counterpart pressure: Google, Typeform, and SurveyMonkey expose security/compliance at SaaS/product level, but not the same pack-bound self-hosting model.
CF-046. Governance family gap: local compliance docs still carry retired tier language.
CF-047. Residency family status: present as policy but not deployed.
CF-048. Residency family evidence: `microservices/forms/policy/data-residency.md` exists.
CF-049. Residency family gap: no OpenTofu modules prove region and residency routing in any context.
CF-050. Tenant isolation family status: policy exists but tenant_class does not.
CF-051. Tenant isolation family evidence: `microservices/forms/policy/tenant-scope.cedar` exists.
CF-052. Tenant isolation family gap: tenant scope should distinguish tenant_class from authorization entitlement.
CF-053. Question Bank family status: gap.
CF-054. Question Bank counterpart pressure: SurveyMonkey exposes Question Bank in the survey build path (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:418-420`).
CF-055. Question Bank gap: no forms-local question-bank artifact found.
CF-056. Audience family status: gap.
CF-057. Audience family counterpart pressure: SurveyMonkey exposes target audience as a creation path (`https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:407`).
CF-058. Audience family gap: no panel/audience procurement scope in forms artifacts.
CF-059. Template family status: partial.
CF-060. Template family evidence: `microservices/forms/PRD.md:64` names template marketplace.
CF-061. Template family counterpart pressure: Google and SurveyMonkey both emphasize templates (`https://workspace.google.com/products/forms/:449-451`, `https://help.surveymonkey.com/en/surveymonkey/create/creating-a-survey/:341-343`).
CF-062. Template family gap: marketplace publisher Terraform file exists, but canonical OpenTofu and governance are missing.
CF-063. Performance family status: target-rich but not measured.
CF-064. Performance family evidence: `microservices/forms/PRD.md:103-115` names concrete targets.
CF-065. Performance family gap: prior benchmark artifact must be replaced by non-tiered methodology.
CF-066. Deployment family status: major gap.
CF-067. Deployment family evidence: only Helm, Kustomize, and Terraform paths exist in service inventory.
CF-068. Deployment family gap: no six-context OpenTofu layout exists.
CF-069. OS family status: major gap.
CF-070. OS family evidence: no `supported-oses.json` exists under forms.
CF-071. SDK family status: partial.
CF-072. SDK family evidence: `microservices/forms/reference-implementations/submit-form-and-export-rust-sdk.md` and `microservices/forms/sdk-plan.md` exist.
CF-073. SDK family gap: API tier fields should be replaced before long-lived SDK generation.
CF-074. Migration family status: useful but outdated by tier retirement.
CF-075. Migration family evidence: `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md` exists.
CF-076. Migration family gap: migration acceptance should reference tenant_class and entitlements, not retired tiers.
CF-077. Onboarding family status: present.
CF-078. Onboarding family evidence: `microservices/forms/onboarding/forms-engineer-first-week.md` exists.
CF-079. Onboarding family gap: onboarding must mention canonical OpenTofu/OS/tenant_class blockers before implementation begins.
CF-080. Verdict from capability-family scan: Oyatie forms has broad feature intent, but deployment, tenant model, Question Bank, audience, and benchmark proof are the main parity gaps.

## 9. Family Verdict

Feature breadth verdict: Oyatie forms is already ambitious enough to cover most of the Google Forms, Typeform, and SurveyMonkey union surface on paper.
Feature evidence verdict: several capabilities are represented only as PRD/IP intent, not as coherent contract, runbook, policy, deployment, and OS evidence.
Counterpart parity verdict: product ambition is ahead in governance, behind in Question Bank, audience sourcing, partial-response detail, quiz release controls, and canonical deployment proof.
Canonical fit verdict: the service cannot honestly claim all-context ownership coherence until the OpenTofu, OS, OCI Always Free, tenant_class, and tier-retirement gaps are closed.
Next artifact link: performance numbers are handled separately in `performance-benchmark-numbers-2026-05-20.md` without tier segmentation.
