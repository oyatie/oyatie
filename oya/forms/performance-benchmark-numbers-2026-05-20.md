# Forms Performance Benchmark Numbers - 2026-05-20

Scope: `microservices/forms/`.
Counterparts: Google Forms, Typeform, SurveyMonkey.
Tier posture: no retired four-level capability tier headings, rows, or target segmentation are used.
Target model: one industry-leader target set, then deployment-context overlays and tenant_class overlays.
Methodology disclosure: public counterpart sources publish quotas, limits, feature caps, and commercial response controls more readily than latency benchmarks; latency values below are Oyatie target numbers or explicitly marked estimates from public limits.

## Five-Citation Anchor Block

Anchor 1 - Local PRD target numbers: `microservices/forms/PRD.md:103-115`.
Anchor 2 - Google Forms API quota numbers: `https://developers.google.com/workspace/forms/api/limits:224-238`.
Anchor 3 - Typeform API and response-limit numbers: `https://www.typeform.com/developers/get-started/:172-176`, `https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:9-45`.
Anchor 4 - SurveyMonkey API and response-limit numbers: `https://api.surveymonkey.com/v3/docs:219-239`, `https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:335-355`.
Anchor 5 - OCI Always Free profile resource numbers: `https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm:37-88`.

## 1. Methodology

M-001. Benchmark dimensions: render latency, validation latency, submission latency, analytics latency, export latency, AI build latency, upload scan latency, API quota, response ingestion throughput, concurrent responders, form size, and deployment cap.
M-002. Test workload A: simple form with 10 questions, no upload, no conditional logic, anonymous respondent.
M-003. Test workload B: standard business intake with 30 questions, 5 conditional branches, one file upload field, and authenticated respondent.
M-004. Test workload C: governed compliance intake with 60 questions, per-question data class, consent binding, file upload scan, signature field, and workflow trigger.
M-005. Test workload D: public campaign form with 100k expected responses, bot pressure, captcha path, CDN/cache path, and export workload.
M-006. Test workload E: AI form build from prompt plus uploaded source document, including guardrails and policy classification.
M-007. Test workload F: analytics dashboard refresh over 100k responses.
M-008. Test workload G: CSV export for 100k responses.
M-009. Test workload H: bulk distribute for 10k recipients.
M-010. OS disclosure: current forms artifacts lack `supported-oses.json`, so OS-specific benchmark validity is blocked until that manifest exists.
M-011. Architecture disclosure: no service-local Rust source tree exists under `microservices/forms/`, so targets are design targets rather than measured code results.
M-012. Deployment-context disclosure: canonical contexts are oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, and oyatie-as-cloud-provider (`specs/master-plan-sequencing.json:704-745`).
M-013. Deployment-context evidence gap: forms has no canonical `iac/<context>/` OpenTofu modules, so context overlays are target constraints, not measured deployments.
M-014. Tenant_class disclosure: this report uses `demo_trial`, `paid`, and `revenue_share`.
M-015. Tenant_class evidence gap: forms has no existing `tenant_class`, `demo_trial`, or `revenue_share` terms in local artifacts.
M-016. Public source limitation: Google publishes Forms API request quotas, but not form-render p95/p99 latency in the cited source.
M-017. Public source limitation: Typeform publishes API rate limits, response limits, file-upload storage amounts, and plan feature caps, but not p95/p99 respondent render latency in the cited source.
M-018. Public source limitation: SurveyMonkey publishes API limits, page size, max survey size, response limits, and response overage rules, but not p95/p99 respondent render latency in the cited source.
M-019. Estimation rule: latency estimates for counterparts are labeled estimated and are used only to size Oyatie targets, not as sourced measured facts.
M-020. Comparison rule: where a counterpart publishes a hard quota, Oyatie target should meet or exceed the strongest published number unless constrained by demo_trial infrastructure.
M-021. Comparison rule: where a counterpart publishes only plan/account limits, Oyatie target uses product design target plus explicit billing and deployment caps.
M-022. Comparison rule: demo_trial constrains volume and infrastructure, not quality of correctness, security, accessibility, or compliance behavior.
M-023. Comparison rule: paid scales with per-seat and usage-based payment, not with feature-quality degradation.
M-024. Comparison rule: revenue_share runs at cost or zero-margin substrate with capacity tied to gross-revenue economics, not feature-quality degradation.
M-025. Stop condition for future measurement: replace estimates with controlled browser/API/load-test results once Rust services, Leptos renderer, OpenTofu modules, and OS manifest exist.

## 2. Counterpart Numbers

### 2.1 Google Forms Numbers

GF-N001. Read request quota: 975 requests per minute per project, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:224-228`).
GF-N002. Read request quota converted: 16.25 requests per second per project, calculated from GF-N001.
GF-N003. Read request user quota: 390 requests per minute per user per project, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:224-228`).
GF-N004. Read request user quota converted: 6.5 requests per second per user per project, calculated from GF-N003.
GF-N005. Expensive read quota: 450 requests per minute per project for `forms.responses.list`, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:229-235`).
GF-N006. Expensive read quota converted: 7.5 requests per second per project, calculated from GF-N005.
GF-N007. Expensive read user quota: 180 requests per minute per user per project, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:229-235`).
GF-N008. Expensive read user quota converted: 3 requests per second per user per project, calculated from GF-N007.
GF-N009. Write request quota: 375 requests per minute per project, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:236-238`).
GF-N010. Write request quota converted: 6.25 requests per second per project, calculated from GF-N009.
GF-N011. Write request user quota: 150 requests per minute per user per project, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:236-238`).
GF-N012. Write request user quota converted: 2.5 requests per second per user per project, calculated from GF-N011.
GF-N013. Daily request limit: unlimited per day if per-minute quotas are respected, source: Google Forms API usage limits (`https://developers.google.com/workspace/forms/api/limits:219-220`).
GF-N014. Linear-scale answer range: scale may start at 0 or 1 and end at an integer from 2 to 10, source: Google Docs Editors Help (`https://support.google.com/docs/answer/7322334:83-89`).
GF-N015. Rating answer range: rating can be whole number from 3 to 10, source: Google Docs Editors Help (`https://support.google.com/docs/answer/7322334:91-98`).
GF-N016. File-upload controls: owner can set file type, maximum file count, and maximum file size, source: Google Docs Editors Help (`https://support.google.com/docs/answer/7322334:78-81`).
GF-N017. Google Forms render p95: estimated 200-450 ms for simple cached public form under good network; source basis: public product positioning plus absence of official latency benchmark, not a measured Google SLA.
GF-N018. Google Forms submission p95: estimated 250-700 ms for normal public submission under good network; source basis: estimate, not a published Google number.

### 2.2 Typeform Numbers

TF-N001. Create API rate limit: 2 requests per second per Typeform account, source: Typeform developer guide (`https://www.typeform.com/developers/get-started/:172-176`).
TF-N002. Responses API rate limit: 2 requests per second per Typeform account, source: Typeform developer guide (`https://www.typeform.com/developers/get-started/:172-176`).
TF-N003. API rate limit converted: 120 requests per minute per account, calculated from TF-N001.
TF-N004. Response-limit reset period: monthly, source: Typeform response limits (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:9-14`).
TF-N005. Response-limit scope: across all forms in the account, source: Typeform response limits (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:13-14`).
TF-N006. Response-limit warning threshold: 90 percent, source: Typeform response limits (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:33-37`).
TF-N007. Response-limit hard threshold: 100 percent, source: Typeform response limits (`https://help.typeform.com/hc/en-us/articles/360040197372-Response-limits:33-38`).
TF-N008. Growth plan response floor in cited pricing surface: 10k+ responses per month, source: Typeform pricing (`https://www.typeform.com/pricing:389-395`).
TF-N009. Growth Custom response floor in cited pricing surface: 20k+ responses per month, source: Typeform pricing (`https://www.typeform.com/pricing:447-453`).
TF-N010. Response enrichment amount in cited Growth plan context: 1,500 responses per month, source: Typeform pricing (`https://www.typeform.com/pricing:397-399`).
TF-N011. Response enrichment amount in cited Growth Custom context: 3,000 responses per month, source: Typeform pricing (`https://www.typeform.com/pricing:455-457`).
TF-N012. File-upload storage amounts shown in pricing comparison: 1 GB, 2 GB, and 4 GB variants, source: Typeform pricing (`https://www.typeform.com/pricing:1942-1962`).
TF-N013. Partial response submit points in cited comparison: 1 and 3 submit-point variants, source: Typeform pricing (`https://www.typeform.com/pricing:2001-2018`).
TF-N014. Typeform render p95: estimated 300-900 ms for media-rich conversational forms; source basis: estimate from product shape and absence of public p95 benchmark.
TF-N015. Typeform submission p95: estimated 350-1000 ms for standard form submission; source basis: estimate, not a published Typeform number.

### 2.3 SurveyMonkey Numbers

SM-N001. Draft/private app API rate limit: 120 requests per minute, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:219-231`).
SM-N002. API rate limit converted: 2 requests per second, calculated from SM-N001.
SM-N003. Starting daily app request limit: 500 requests per day, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:219-231`).
SM-N004. Allowed temporary violations: three violations up to 150 percent within a 30-day window before stricter enforcement, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:226-227`).
SM-N005. Limit review response time: within 5 business days for increased rate requests, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:230-232`).
SM-N006. Max API page size: 1000 resources unless otherwise specified, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:233-239`).
SM-N007. Max survey size: 1000 questions before over-limit survey returns 413, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:233-239`).
SM-N008. Basic plan detailed response access: up to 25 responses per survey, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:206-214`).
SM-N009. Paid plan detailed response access: unlimited for that scope, source: SurveyMonkey API docs (`https://api.surveymonkey.com/v3/docs:206-214`).
SM-N010. Response-limit overage charge in cited help source: 0.15 USD per response when applicable, source: SurveyMonkey Help (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:345-352`).
SM-N011. Over-limit response deletion window: automatically deleted 1 year after collection if not retained, source: SurveyMonkey Help (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:339-344`).
SM-N012. Account response count reset: reset to 0 at billing-cycle start or renewal invoice payment, source: SurveyMonkey Help (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:347-352`).
SM-N013. Enterprise response limits: annual limits can apply, source: SurveyMonkey Help (`https://help.surveymonkey.com/en/surveymonkey/billing/response-limits/:353-355`).
SM-N014. SurveyMonkey render p95: estimated 300-800 ms for normal surveys; source basis: estimate from product class and absence of public p95 benchmark.
SM-N015. SurveyMonkey submission p95: estimated 350-900 ms for normal response submission; source basis: estimate, not a published SurveyMonkey number.

## 3. Oyatie Target Numbers

### 3.1 Single Industry-Leader Target Set

OT-N001. Simple form render p50 target: 75 ms server-side first-byte plus cached shell path.
OT-N002. Simple form render p95 target: 200 ms, aligned with PRD render target (`microservices/forms/PRD.md:103-105`).
OT-N003. Simple form render p99 target: 350 ms for public-cloud and paid elastic contexts.
OT-N004. Standard business intake render p95 target: 250 ms with five conditional branches and no upload transfer.
OT-N005. Governed compliance intake render p95 target: 350 ms with policy/data-class metadata loaded.
OT-N006. Field validation p50 target: 15 ms.
OT-N007. Field validation p95 target: 35 ms.
OT-N008. Field validation p99 target: 50 ms, matching PRD p99 target (`microservices/forms/PRD.md:106`).
OT-N009. Submission p50 target: 60 ms for non-upload submission after connection establishment.
OT-N010. Submission p95 target: 150 ms, matching PRD target (`microservices/forms/PRD.md:107`).
OT-N011. Submission p99 target: 300 ms for non-upload submission.
OT-N012. Analytics refresh p50 target: 180 ms over cached aggregate.
OT-N013. Analytics refresh p95 target: 500 ms, matching PRD target (`microservices/forms/PRD.md:108`).
OT-N014. Analytics refresh p99 target: 900 ms over 100k-response aggregate.
OT-N015. Bulk distribute target: 10k recipients in 30 seconds, matching PRD target (`microservices/forms/PRD.md:109`).
OT-N016. CSV export target: 100k responses in 5 seconds, matching PRD target (`microservices/forms/PRD.md:110`).
OT-N017. XLSX export target: 100k responses in 10 seconds, matching PRD target (`microservices/forms/PRD.md:111`).
OT-N018. AI build p95 target: 8 seconds for prompt-to-draft, matching PRD target (`microservices/forms/PRD.md:112`).
OT-N019. Upload scan target: 100 MB in 5 seconds, matching PRD target (`microservices/forms/PRD.md:113`).
OT-N020. Accessibility target: WCAG 2.2 AA correctness with zero known critical violations, tied to `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`.
OT-N021. API read quota target: 2,000 reads per minute per tenant by default paid/public-cloud envelope, beating Google's 975 per minute per project quota.
OT-N022. API expensive-read quota target: 1,000 response-list reads per minute per tenant by default paid/public-cloud envelope, beating Google's 450 per minute project expensive-read quota.
OT-N023. API write quota target: 750 writes per minute per tenant by default paid/public-cloud envelope, beating Google's 375 per minute project write quota.
OT-N024. API burst target: 2x per-minute quota for 60 seconds with queueing and backoff rather than immediate hard failure.
OT-N025. Public respondent submission throughput target: 500 non-upload submissions per second per provisioned cell.
OT-N026. Public respondent read/render throughput target: 2,000 renders per second per provisioned CDN-backed cell for cacheable form shells.
OT-N027. Concurrent respondent target: 50,000 active public respondents per paid elastic tenant before sharding.
OT-N028. Form definition size target: 1,000 questions, matching SurveyMonkey's published max survey size before 413 behavior.
OT-N029. Response export page size target: 5,000 responses per API page for paid tenants, exceeding SurveyMonkey's 1,000-resource page-size limit.
OT-N030. Response retention lookup target: 100k responses queryable under 500 ms p95 for indexed filters.
OT-N031. PII encryption correctness target: 100 percent encrypted configured PII columns, tied to `microservices/forms/slos/pii-encryption-correctness.openslo.yaml`.
OT-N032. Spam-flood throttle target: absorb 10x normal submission pressure for 10 minutes with degraded acceptance only at configured abuse thresholds.
OT-N033. Captcha challenge latency target: p95 under 700 ms additional time for challenged sessions.
OT-N034. Signature initiation target: p95 under 800 ms excluding external signature-provider round trip.
OT-N035. Payment field initiation target: p95 under 900 ms excluding external payment-provider authorization.
OT-N036. Webhook/event publish target: p95 under 250 ms after durable submission write.
OT-N037. Async export completion target: p95 under 30 seconds for 1M-response CSV when running on paid elastic substrate.
OT-N038. Dashboard freshness target: p95 aggregate lag under 5 seconds for standard forms and under 30 seconds for public campaign spikes.
OT-N039. Data-residency routing target: 100 percent writes routed to declared residency region for pack-bound forms.
OT-N040. Audit-chain append target: p95 under 100 ms after primary event commit.

### 3.2 Deployment-Context Overlay

DC-001. `oyatie-public-cloud`: full target set applies with elastic cell scaling and managed edge/WAF capacity.
DC-002. `oyatie-public-cloud`: read/write API quotas can be raised by adding cells; target remains 2,000 reads/minute, 1,000 expensive reads/minute, and 750 writes/minute per tenant before custom uplift.
DC-003. `guest-on-aws`: full target set applies only after customer account OpenTofu module provisions equivalent compute, storage, cache, search, WAF, and CDN capacity.
DC-004. `guest-on-aws`: if customer account limits block capacity, published target remains canonical but deployment overlay must declare the account quota cap.
DC-005. `guest-on-oci`: full paid target applies with sufficient paid OCI resources.
DC-006. `guest-on-oci`: demo_trial OCI Always Free profile is constrained by Oracle-published 4 OCPU and 24 GB memory total for Ampere A1 (`https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm:71-74`).
DC-007. `guest-on-oci`: Always Free storage overlay is constrained by 200 GB Always Free block volume storage (`https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm:37-40`, `https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm:88-88`).
DC-008. `guest-on-oci`: Always Free idle reclamation risk must be modeled because idle instances may be reclaimed under Oracle's published idle criteria (`https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm:45-56`).
DC-009. `on-prem`: full target set applies only when facility networking, storage, compute, and edge security meet the capacity profile.
DC-010. `on-prem`: facility-specific latency to external email, SMS, payment, signature, and AI providers must be excluded or separately measured.
DC-011. `colo`: full target set applies only when colo facility and transit have enough headroom for campaign spikes.
DC-012. `colo`: CDN/WAF equivalence must be declared in the OpenTofu context module.
DC-013. `oyatie-as-cloud-provider`: full target set applies with Oyatie-controlled substrate and customer tenancy isolation.
DC-014. All contexts: OpenTofu modules are missing today, so every context overlay is a target constraint rather than validated runtime evidence.
DC-015. All contexts: `supported-oses.json` is missing today, so OS/architecture overlays cannot be certified.

### 3.3 Tenant-Class Overlay

TC-001. `demo_trial`: correctness, security, privacy, and accessibility targets remain the same as paid.
TC-002. `demo_trial`: infrastructure must fit the OCI Always Free profile when the context is guest-on-oci.
TC-003. `demo_trial`: recommended initial cap is 25 active forms per tenant.
TC-004. `demo_trial`: recommended initial cap is 10,000 submissions per tenant per month.
TC-005. `demo_trial`: recommended initial cap is 5 GB uploaded files per tenant per month.
TC-006. `demo_trial`: recommended initial cap is 50 AI form-build requests per tenant per month.
TC-007. `demo_trial`: recommended initial cap is 5 concurrent exports per tenant.
TC-008. `demo_trial`: recommended API cap is 120 reads per minute, 60 expensive reads per minute, and 60 writes per minute until measured capacity proves higher.
TC-009. `demo_trial`: recommended public submission cap is 50 non-upload submissions per second per tenant on Always Free profile.
TC-010. `demo_trial`: usage caps must be explicit in contracts and UI; hidden feature-quality degradation is not allowed.
TC-011. `paid`: default target is the full industry-leader set in Section 3.1.
TC-012. `paid`: quotas scale by per-seat contract, response volume, file storage, export usage, and AI build usage.
TC-013. `paid`: custom quota uplift can exceed the default target with provisioned cell capacity and billing approval.
TC-014. `paid`: compliance packs and BYOK may be enabled subject to policy and deployment context.
TC-015. `revenue_share`: default target is the full industry-leader set unless commercial agreement sets an at-cost substrate cap.
TC-016. `revenue_share`: capacity planning should bind gross-revenue share, expected transaction volume, campaign seasonality, and subsidy tolerance.
TC-017. `revenue_share`: no feature-quality degradation is allowed; only volume, concurrency, and subsidy economics are constrained.
TC-018. `revenue_share`: marketplace sellers need public campaign burst budgets.
TC-019. `revenue_share`: B2C operators need anti-abuse and consent budgets aligned to conversion funnel value.
TC-020. `revenue_share`: embedded SaaS resellers need per-reseller tenant isolation and export quotas.
TC-021. `revenue_share`: affiliate partners need attribution fields and abuse throttles measured separately from core submission latency.

## 4. Comparison Narrative

CN-001. API reads: Oyatie target of 2,000 reads/minute per tenant is ahead of Google Forms published 975 reads/minute per project, Typeform's 120/minute account conversion, and SurveyMonkey's 120/minute draft/private app limit.
CN-002. API expensive reads: Oyatie target of 1,000 response-list reads/minute per tenant is ahead of Google's 450/minute project quota and SurveyMonkey's 1000-resource page-size cap.
CN-003. API writes: Oyatie target of 750 writes/minute per tenant is ahead of Google's 375/minute project quota and the 120/minute Typeform/SurveyMonkey public API numbers.
CN-004. Daily API quota: Oyatie should match Google's unlimited-daily posture for paid tenants as long as per-minute quotas and abuse controls are respected.
CN-005. Demo_trial API quota: Oyatie demo_trial starts at parity with Typeform/SurveyMonkey public API rates, then can rise only after Always Free load evidence.
CN-006. Render latency: Oyatie p95 200 ms target is aggressive and ahead of the estimated public SaaS baseline, but it remains unproven without implementation and browser tests.
CN-007. Submission latency: Oyatie p95 150 ms target is ahead of estimated counterpart submission latency, but external payment/signature/upload provider time must be excluded or separately measured.
CN-008. Field validation latency: Oyatie p99 50 ms is an industry-leader target and should be locally measurable once Rust validation code exists.
CN-009. Analytics latency: Oyatie p95 500 ms for response analytics is parity-to-ahead for 100k-response dashboards, but query-shape evidence is absent today.
CN-010. CSV export: Oyatie 100k responses in 5 seconds is ahead of typical SaaS export expectations and needs warehouse/export worker proof.
CN-011. XLSX export: Oyatie 100k responses in 10 seconds is an aggressive target because spreadsheet serialization is often slower than CSV.
CN-012. Bulk distribute: 10k recipients in 30 seconds is plausible only when mail/SMS provider rate limits are modeled outside forms core latency.
CN-013. AI build: 8 seconds p95 is competitive with Typeform and SurveyMonkey AI creation surfaces, but local AI dependency ownership is inconsistent across PRD and manifest.
CN-014. Upload scan: 100 MB in 5 seconds is aggressive and must be measured on every supported deployment context.
CN-015. File storage: Typeform publishes 1 GB, 2 GB, and 4 GB storage variants in pricing comparison; Oyatie's target should be tenant_class usage caps rather than feature tiers.
CN-016. Survey size: matching 1,000 questions gives parity with SurveyMonkey's published max survey-size number.
CN-017. Response page size: 5,000 responses per API page puts Oyatie ahead of SurveyMonkey's 1,000-resource page-size limit if memory and timeout bounds hold.
CN-018. OCI Always Free: demo_trial cannot claim paid-level throughput because 4 OCPU, 24 GB memory, and 200 GB block storage are hard infrastructure constraints.
CN-019. On-prem and colo: latency can beat cloud for local respondents but may trail for global public campaigns without edge/CDN equivalence.
CN-020. All contexts: current lack of OpenTofu modules means these numbers are target commitments, not validated deployment evidence.
CN-021. Product quality: tenant_class overlays constrain volume and economics, not correctness, accessibility, security, privacy, or compliance quality.
CN-022. Final benchmark stance: forms has a strong target set, but measured evidence is blocked by missing canonical deployment modules, OS manifest, and implementation source.

## 5. Workload Acceptance Register

WA-001. Workload A simple-render pass target: p95 <= 200 ms and p99 <= 350 ms in paid public-cloud context.
WA-002. Workload A demo_trial overlay: p95 <= 250 ms and p99 <= 500 ms on OCI Always Free profile if active respondent concurrency is capped.
WA-003. Workload A evidence needed: browser timing, server trace, CDN cache status, and form schema size.
WA-004. Workload B standard-intake pass target: p95 <= 250 ms render and p95 <= 150 ms submission without upload transfer time.
WA-005. Workload B demo_trial overlay: same latency target at lower concurrency; cap active submissions before latency violation.
WA-006. Workload B evidence needed: branch-count telemetry, validation trace, and response-write trace.
WA-007. Workload C governed-intake pass target: p95 <= 350 ms render and p95 <= 300 ms non-upload submission.
WA-008. Workload C policy overlay: Cedar/policy evaluation must be measured separately and included in p99.
WA-009. Workload C evidence needed: data-class load time, policy decision time, consent write time, and audit append time.
WA-010. Workload D public-campaign pass target: 500 non-upload submissions per second per provisioned paid cell.
WA-011. Workload D burst target: 10x normal submission pressure for 10 minutes with abuse controls, not data loss.
WA-012. Workload D demo_trial overlay: cap at 50 non-upload submissions per second until Always Free measurements prove more.
WA-013. Workload D evidence needed: WAF events, captcha challenge rate, queue depth, reject reason counts, and durable-write latency.
WA-014. Workload E AI-build pass target: p95 <= 8 seconds for prompt-to-draft.
WA-015. Workload E quality target: policy-bounded draft must include field labels, validation hints, and unsafe-content rejection path.
WA-016. Workload E evidence needed: prompt size, provider latency, guardrail latency, draft field count, and rejection reason.
WA-017. Workload F analytics pass target: p95 <= 500 ms for cached aggregate and p99 <= 900 ms for 100k-response dashboard.
WA-018. Workload F freshness target: p95 lag <= 5 seconds for standard forms and <= 30 seconds for campaign spikes.
WA-019. Workload F evidence needed: query plan, cache hit ratio, aggregate-lag metric, and dashboard render timing.
WA-020. Workload G CSV export pass target: 100k responses in <= 5 seconds.
WA-021. Workload G XLSX export pass target: 100k responses in <= 10 seconds.
WA-022. Workload G large export pass target: 1M-response CSV p95 <= 30 seconds on paid elastic substrate.
WA-023. Workload G evidence needed: row count, column count, PII redaction mode, stream chunk size, and object-store write time.
WA-024. Workload H bulk-distribute pass target: 10k recipients in <= 30 seconds excluding external provider throttles.
WA-025. Workload H evidence needed: recipient count, provider rate limits, queue dispatch rate, bounce/error counters, and retry policy.
WA-026. File-upload scan pass target: 100 MB in <= 5 seconds.
WA-027. File-upload scan p99 guardrail: p99 <= 10 seconds for 100 MB if CPU is not oversubscribed.
WA-028. File-upload demo_trial overlay: cap total monthly upload bytes before reducing scan correctness.
WA-029. File-upload evidence needed: file type, byte size, scan engine version, CPU allocation, and malware verdict time.
WA-030. Signature initiation pass target: p95 <= 800 ms excluding external provider round trip.
WA-031. Signature evidence needed: policy eligibility time, pack conformance check time, provider handoff time, and audit append time.
WA-032. Payment initiation pass target: p95 <= 900 ms excluding external payment authorization.
WA-033. Payment evidence needed: payment-field validation time, provider handoff time, idempotency key creation, and audit append time.
WA-034. API read paid target: 2,000 reads per minute per tenant.
WA-035. API read demo_trial target: 120 reads per minute per tenant until measured OCI profile allows uplift.
WA-036. API read comparison: paid target is ahead of Google Forms published 975 reads per minute per project.
WA-037. API expensive-read paid target: 1,000 response-list reads per minute per tenant.
WA-038. API expensive-read comparison: paid target is ahead of Google Forms published 450 expensive reads per minute per project.
WA-039. API write paid target: 750 writes per minute per tenant.
WA-040. API write comparison: paid target is ahead of Google Forms published 375 writes per minute per project.
WA-041. API page target: 5,000 responses per API page for paid tenants.
WA-042. API page comparison: target is ahead of SurveyMonkey's published 1,000-resource max page size.
WA-043. Form size target: 1,000 questions supported.
WA-044. Form size comparison: target matches SurveyMonkey's published max survey size before 413.
WA-045. Concurrency paid target: 50,000 active public respondents per tenant before sharding.
WA-046. Concurrency demo_trial target: cap to measured Always Free safe point, initial assumption 500 active respondents.
WA-047. Concurrency evidence needed: active sessions, socket count, render queue, submission queue, memory, CPU, and network saturation.
WA-048. Public-cloud context acceptance: all Workloads A-H meet paid targets with autoscaling enabled.
WA-049. AWS guest context acceptance: all Workloads A-H meet paid targets after customer account quota check.
WA-050. OCI paid guest acceptance: all Workloads A-H meet paid targets after paid OCI quota check.
WA-051. OCI demo_trial acceptance: Workloads A-H meet demo_trial caps under 4 OCPU, 24 GB memory, and 200 GB storage.
WA-052. On-prem acceptance: all Workloads A-H meet target only when facility profile matches required compute, storage, network, and edge controls.
WA-053. Colo acceptance: all Workloads A-H meet target only when transit, edge, and storage profile are validated.
WA-054. Oyatie provider acceptance: all Workloads A-H meet paid targets under Oyatie-controlled substrate.
WA-055. Tenant_class acceptance: benchmark output must print tenant_class and deployment_context for every run.
WA-056. Tenant_class acceptance: output must show quality targets unchanged across tenant classes.
WA-057. Tenant_class acceptance: output must show volume caps for demo_trial and revenue_share when applicable.
WA-058. Evidence acceptance: every benchmark run must record git commit, build profile, OS, arch, OpenTofu plan id, tenant_class, and deployment_context.
WA-059. Evidence acceptance: every latency run must record p50, p95, p99, max, error rate, and sample count.
WA-060. Evidence acceptance: every throughput run must record steady-state duration, warm-up duration, concurrency, and backpressure behavior.
WA-061. Evidence acceptance: every export run must record response count, column count, byte size, redaction mode, and destination storage.
WA-062. Evidence acceptance: every AI run must record prompt bytes, source attachment bytes, generated field count, guardrail actions, and provider class.
WA-063. Evidence acceptance: every upload run must record file bytes, file type, scan version, verdict, and quarantine behavior.
WA-064. Evidence acceptance: every public-campaign run must record bot ratio, captcha challenge ratio, allowed submissions, rejected submissions, and reject reasons.
WA-065. Evidence acceptance: every event run must record publish latency, delivery latency, retry count, and dead-letter count.
WA-066. Failure acceptance: benchmark fails closed if contracts still expose retired tier fields in test payloads.
WA-067. Failure acceptance: benchmark fails closed if tenant_class is absent from test environment metadata.
WA-068. Failure acceptance: benchmark fails closed if deployment_context is absent from test environment metadata.
WA-069. Failure acceptance: benchmark fails closed if OpenTofu plan evidence is missing for a claimed context.
WA-070. Failure acceptance: benchmark fails closed if OS/arch metadata cannot map to the future `supported-oses.json`.
WA-071. Reporting acceptance: counterpart table must distinguish sourced numbers from estimates.
WA-072. Reporting acceptance: latency estimates must not be presented as public counterpart benchmarks.
WA-073. Reporting acceptance: OCI Always Free overlay must mention infrastructure cap, not feature-quality degradation.
WA-074. Reporting acceptance: paid overlay must mention per-seat plus usage scaling.
WA-075. Reporting acceptance: revenue_share overlay must mention at-cost or zero-margin substrate economics.
WA-076. Reporting acceptance: no benchmark section may reintroduce retired tenant-class segmentation.
WA-077. Reporting acceptance: public-cloud, AWS guest, OCI guest, on-prem, colo, and Oyatie provider contexts must all appear.
WA-078. Reporting acceptance: missing context evidence must be reported as blocked, not silently assumed.
WA-079. Reporting acceptance: any provider-specific limit must be separated from forms-core latency.
WA-080. Reporting acceptance: any third-party payment, signature, email, SMS, or AI latency must be tagged external.
WA-081. Correctness acceptance: performance pass cannot override PII encryption, data residency, accessibility, or audit-chain correctness.
WA-082. Correctness acceptance: abuse throttling must prefer explicit rejection over silent data loss.
WA-083. Correctness acceptance: export speed must not bypass redaction or residency rules.
WA-084. Correctness acceptance: AI build speed must not bypass prompt safety or policy classification.
WA-085. Correctness acceptance: upload scan speed must not bypass malware verdict or quarantine policy.
WA-086. Final acceptance: these numbers become shippable only after implementation, OpenTofu contexts, OS manifest, and measured runs exist.
