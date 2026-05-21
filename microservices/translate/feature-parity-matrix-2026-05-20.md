# translate feature-parity matrix - 2026-05-20

Service: translate.
Audit batch: Wave 3 Batch 3.2.
Counterpart 1: Google Translate / Google Cloud Translation.
Counterpart 2: DeepL.
Counterpart 3: Amazon Translate.
Purpose: compare translate against the union of the three required industry counterparts.
Retired framing: no commercial capability ladder rows are used.
Forward framing: uniform quality target with tenant-class and deployment-context overlays handled outside this feature matrix.
Local product anchor: `PRD.md:22-34`.
Local contract anchor: `contracts/openapi/translate.yaml:37-381`.
Local event anchor: `contracts/asyncapi/translate-events.yaml:1-231`.
Local Proto anchor: `contracts/proto/translate.proto:23-67`.
Local parity caveat: existing `competitor-parity-matrix.md:18-40` is broader than this top-three scope and must be refreshed.
External Google quota source: https://docs.cloud.google.com/translate/quotas
External Google language source: https://docs.cloud.google.com/translate/docs/languages
External Google pricing source: https://cloud.google.com/translate/pricing
External DeepL usage source: https://developers.deepl.com/docs/resources/usage-limits
External DeepL language source: https://developers.deepl.com/docs/getting-started/supported-languages
External DeepL glossary source: https://developers.deepl.com/api-reference/multilingual-glossaries
External Amazon quota source: https://docs.aws.amazon.com/translate/latest/dg/what-is-limits.html
External Amazon feature source: https://aws.amazon.com/translate/details/

## 1. Counterpart 1 - Google Translate surface

1. Google offers hosted neural machine translation through Cloud Translation.
2. Google exposes synchronous text translation.
3. Google exposes language detection.
4. Google exposes document translation.
5. Google exposes batch translation for larger asynchronous workloads.
6. Google exposes supported-language APIs.
7. Google publishes quota controls and rate limits.
8. Google content quota for general model text is 6,000,000 characters per project per minute.
9. Google content quota for general model text is also 6,000,000 characters per project per minute per user.
10. Google content quota for custom model text is 100,000 characters per project per minute.
11. Google document translation quota is 2,400 pages per project per minute.
12. Google request quota for v3 is 6,000 requests per project per minute.
13. Google recommends 5,000 code points per text request for latency.
14. Google Cloud Translation Advanced allows up to 30,000 code points per single request.
15. Google Cloud Translation Basic allows 100,000 bytes per request.
16. Google pricing uses character-based billing for text translation.
17. Google pricing currently includes a free monthly credit for the first 500,000 characters.
18. Google document translation pricing uses per-page billing for DOCX, PPT, and PDF formats.
19. Google Translation LLM and adaptive translation support a published language list.
20. Google supports official and experimental language categories.
21. Google has strong global cloud substrate and quota adjustability.
22. Google has a mature console and quota administration model.
23. Google has glossary-like customization through Advanced APIs and custom models.
24. Google has batch and document translation in official docs.
25. Google does not provide Oyatie-specific tenant isolation, Cedar policy, or audit-chain evidence.
26. Google does not expose Oyatie pack-level data-residency policy in local artifacts.
27. Google sets the high-scale quota bar for translate.
28. Google sets the managed-cloud operational bar for translate.
29. Google sets the language breadth bar for translate.
30. Google sets a public price-transparency bar for text and document usage.
31. Translate local PRD covers text translation (`PRD.md:24`).
32. Translate local PRD covers language detection (`PRD.md:54`).
33. Translate local PRD covers document translation (`PRD.md:65-69`).
34. Translate local PRD covers bulk API (`PRD.md:34`, `PRD.md:69-70`).
35. Translate local PRD covers data-residency language packs (`PRD.md:33`, `PRD.md:121-130`).
36. Translate local PRD covers engine routing and fallback (`PRD.md:27`, `PRD.md:132-147`).
37. Translate OpenAPI covers `/translate` (`contracts/openapi/translate.yaml:37-58`).
38. Translate OpenAPI covers `/detect-language` (`contracts/openapi/translate.yaml:80-99`).
39. Translate OpenAPI covers `/document-translate` (`contracts/openapi/translate.yaml:138-175`).
40. Translate OpenAPI covers `/bulk-translate` (`contracts/openapi/translate.yaml:197-231`).
41. Translate gap versus Google: no OpenTofu public-cloud module proves Google-scale elasticity.
42. Translate gap versus Google: no OS support manifest proves multi-OS SDK/runtime posture.
43. Translate gap versus Google: no runtime code proves v3/v2-compatible behavior.
44. Translate gap versus Google: no published language list artifact in translate path was found.
45. Translate advantage versus Google: stronger explicit data-residency pack design.
46. Translate advantage versus Google: explicit audit-chain and policy intent.
47. Translate parity posture versus Google: product surface is planned, implementation proof is absent.
48. Google coverage status: partial planned parity, not deployable parity.

## 2. Counterpart 2 - DeepL surface

1. DeepL offers high-quality machine translation APIs.
2. DeepL offers text translation.
3. DeepL offers document translation.
4. DeepL offers glossaries.
5. DeepL offers multilingual glossary APIs.
6. DeepL supports language retrieval through API endpoints.
7. DeepL supports formality and style-related features for supported languages.
8. DeepL usage limits document a 16 KiB header-size maximum.
9. DeepL usage limits document a 128 KiB total request-size maximum.
10. DeepL usage limits document 500,000 characters per month for DeepL API Free.
11. DeepL document limits include DOC/DOCX at 10 MB and 500,000 characters for Free.
12. DeepL document limits include DOC/DOCX at 30 MB and 1,000,000 characters for Pro.
13. DeepL document limits include PPTX at 10 MB and 500,000 characters for Free.
14. DeepL document limits include PPTX at 30 MB and 1,000,000 characters for Pro.
15. DeepL document limits include XLSX at 10 MB and 500,000 characters for Free.
16. DeepL document limits include XLSX at 30 MB and 1,000,000 characters for Pro.
17. DeepL document limits include PDF at 10 MB and 500,000 characters for Free.
18. DeepL document limits include PDF at 30 MB and 1,000,000 characters for Pro.
19. DeepL document limits include text files at 1 MB.
20. DeepL document limits include HTML files at 5 MB.
21. DeepL document limits include XLIFF 2.1 files at 10 MB for Free and Pro.
22. DeepL document limits include SRT files at 150 KB.
23. DeepL document limits include image translation in beta at 3 MB.
24. DeepL tracks character usage by source text Unicode code points.
25. DeepL text API supports a `context` parameter constrained by the same 128 KiB request body limit.
26. DeepL glossary entries can be formatted as CSV or TSV.
27. DeepL glossary APIs list, retrieve, add, replace, patch, and delete dictionaries.
28. DeepL currently has stronger public product emphasis on translation quality and localization writing style than Amazon.
29. DeepL has stronger glossary and document workflow parity relevance than a raw text-only MT engine.
30. Translate local PRD covers termbase and glossary semantics (`PRD.md:26`, `PRD.md:56-57`).
31. Translate OpenAPI covers glossary CRUD (`contracts/openapi/translate.yaml:233-301`).
32. Translate OpenAPI covers termbase CRUD and import/export (`contracts/openapi/translate.yaml:302-395`).
33. Translate ADR-TRANSLATE-0001 gives DeepL preference for some European legal language pairs through routing logic (`decisions/ADR-TRANSLATE-0001-mt-engine-routing-and-fallback.md:49-78`).
34. Translate ADR-TRANSLATE-0004 makes DeepL EU endpoints usable only when allowed by pack policy (`decisions/ADR-TRANSLATE-0004-data-residency-bound-inference.md:63-119`).
35. Translate local benchmark ranks DeepL strongest among external vendors on EN-FR BLEU/COMET (`benchmarks/translate-vs-deepl-google-smartling-crowdin.md:89-100`).
36. Translate local tutorial seeds TM using a workflow that must use tenant_class language (`tutorials/first-translation-with-tm-seed.md:24-33`).
37. Translate gap versus DeepL: no implementation path proves XLIFF, SRT, PDF, DOCX, PPTX, or XLSX round-trip behavior.
38. Translate gap versus DeepL: no public language list artifact maps formality, style, or glossary feature availability.
39. Translate gap versus DeepL: no tenant-class entitlement model replaces the old feature gating docs.
40. Translate advantage versus DeepL: stronger explicit audit, residency, and policy-pack framing in local docs.
41. Translate advantage versus DeepL: TM, QE, human review, and engine-router substrate are designed as integrated Oyatie services.
42. Translate parity posture versus DeepL: planned feature breadth exceeds DeepL in governance, but proof is weaker.
43. DeepL coverage status: substantial planned parity, proof gaps around formats, runtime, and entitlement rewrite.

## 3. Counterpart 3 - Amazon Translate surface

1. Amazon Translate offers synchronous real-time translation.
2. Amazon Translate offers asynchronous batch translation.
3. Amazon Translate supports UTF-8 text input.
4. Amazon Translate synchronous maximum input text is 10,000 bytes.
5. Amazon Translate real-time document maximum number of characters per document is 100,000.
6. Amazon Translate real-time document maximum document size is 100,000 bytes.
7. Amazon Translate asynchronous batch maximum characters per document is 1,000,000.
8. Amazon Translate asynchronous batch maximum size per document is 20 MB.
9. Amazon Translate asynchronous batch maximum translatable text per document is 1 MB.
10. Amazon Translate asynchronous batch maximum target languages per batch request is 10.
11. Amazon Translate asynchronous batch maximum documents per batch is 1,000,000.
12. Amazon Translate asynchronous batch maximum total document size is 5 GB.
13. Amazon Translate asynchronous batch concurrent translation jobs limit is 10.
14. Amazon Translate asynchronous batch queued jobs limit is 1,000.
15. Amazon Translate StartTextTranslationJob TPS limit is 5.
16. Amazon Translate DescribeTextTranslationJob TPS limit is 10.
17. Amazon Translate ListTextTranslationJobs TPS limit is 10.
18. Amazon Translate StopTextTranslationJob TPS limit is 5.
19. Amazon Translate custom terminology file size limit is 10 MB.
20. Amazon Translate custom terminology files per account per region limit is 100.
21. Amazon Translate custom terminology target languages per terminology file limit is 10.
22. Amazon Translate source/target text length per terminology term limit is 200 bytes.
23. Amazon Translate terminology files per TranslateText or StartTextTranslationJob request limit is 1.
24. Amazon Translate parallel data resources per account per region limit is 1,000.
25. Amazon Translate parallel data input file size limit is 5 GB.
26. Amazon Translate parallel data source language count is 1.
27. Amazon Translate parallel data segment size limit is 1,000 bytes.
28. Amazon Translate supports 75 languages according to the AWS feature page.
29. Amazon has strong AWS-region integration and enterprise procurement posture.
30. Amazon has weaker local prominence in the current translate parity files than Google or DeepL.
31. Translate local PRD lists Amazon as a benchmark counterpart (`PRD.md:211-242`).
32. Translate local benchmark includes Amazon in text-latency and quality comparisons (`benchmarks/translate-vs-deepl-google-smartling-crowdin.md:18-31`, `benchmarks/translate-vs-deepl-google-smartling-crowdin.md:89-100`).
33. Translate OpenAPI covers batch and bulk operations (`contracts/openapi/translate.yaml:59-79`, `contracts/openapi/translate.yaml:197-231`).
34. Translate OpenAPI covers glossary and termbase operations (`contracts/openapi/translate.yaml:233-395`).
35. Translate PRD covers external engine adapter routing and fallback (`PRD.md:132-147`).
36. Translate PRD covers Amazon-style batch and bulk jobs (`PRD.md:65-70`).
37. Translate gap versus Amazon: local parity matrix does not center Amazon in the required top three (`competitor-parity-matrix.md:18-40`).
38. Translate gap versus Amazon: adapter docs need explicit Amazon Translate, custom terminology, and parallel-data treatment.
39. Translate gap versus Amazon: no AWS OpenTofu guest module proves deployability inside a customer AWS account.
40. Translate gap versus Amazon: no per-region quota overlay maps Amazon service limits to Oyatie context limits.
41. Translate advantage versus Amazon: stronger explicit multi-engine route selection design.
42. Translate advantage versus Amazon: stronger multi-pack data-residency policy story.
43. Translate advantage versus Amazon: integrated TM/QE/human review surface is broader than raw Amazon Translate.
44. Amazon coverage status: planned baseline coverage, but first-class adapter and AWS-context proof are missing.

## 4. UNION-coverage matrix

1. Capability: text translation API; Google yes; DeepL yes; Amazon yes; translate planned and contracted; evidence `PRD.md:24`, `contracts/openapi/translate.yaml:37-58`; status planned parity.
2. Capability: language detection; Google yes; DeepL auto-detect yes; Amazon supports source auto-detection in service behavior; translate planned; evidence `PRD.md:54`, `contracts/openapi/translate.yaml:80-99`; status planned parity.
3. Capability: batch text translation; Google yes; DeepL via document/text batching constraints; Amazon yes; translate planned; evidence `contracts/openapi/translate.yaml:59-79`; status planned parity.
4. Capability: large bulk localization jobs; Google batch yes; DeepL document/file yes; Amazon batch yes; translate planned; evidence `PRD.md:69-70`, `contracts/openapi/translate.yaml:197-231`; status planned parity.
5. Capability: document translation; Google yes; DeepL yes; Amazon yes; translate planned; evidence `PRD.md:65-69`, `contracts/openapi/translate.yaml:138-175`; status planned parity.
6. Capability: PDF translation; Google yes; DeepL yes; Amazon real-time document yes; translate planned; evidence `ADR-TRANSLATE-0005:69-122`; status planned but unproven.
7. Capability: DOCX translation; Google yes; DeepL yes; Amazon document support; translate planned; evidence `ADR-TRANSLATE-0005:69-122`; status planned but unproven.
8. Capability: PPTX translation; Google yes; DeepL yes; Amazon batch support; translate planned; evidence `ADR-TRANSLATE-0005:84-118`; status planned but unproven.
9. Capability: XLSX translation; Google yes; DeepL yes; Amazon batch support; translate planned; evidence `ADR-TRANSLATE-0005:84-118`; status planned but unproven.
10. Capability: XLIFF; Google not primary; DeepL supports XLIFF 2.1; Amazon batch text can process files; translate planned strongly; evidence `PRD.md:69`, `benchmarks/translate-vs-deepl-google-smartling-crowdin.md:35-45`; status planned strength.
11. Capability: SRT/captions; Google not a core Cloud Translation feature; DeepL supports SRT upload; Amazon raw translation can support text; translate planned; evidence `PRD.md:63`, `contracts/openapi/translate.yaml:176-196`; status planned differentiator.
12. Capability: real-time streaming captions; Google raw API no; DeepL Voice adjacent; Amazon raw API no; translate planned; evidence `PRD.md:63`, `ADR-TRANSLATE-0006`; status planned differentiator.
13. Capability: translation memory; Google no native TMS; DeepL supports translation memories through product/docs ecosystem; Amazon parallel data approximates adaptation; translate planned; evidence `PRD.md:25`, `IP-005-translation-memory-stack.md:19-23`; status planned differentiator.
14. Capability: fuzzy TM leverage; Google no; DeepL product-adjacent; Amazon parallel data no interactive TM; translate planned; evidence `PRD.md:55`, `IP-005-translation-memory-stack.md:80-92`; status planned differentiator.
15. Capability: termbase; Google glossary/custom models; DeepL glossaries; Amazon custom terminology; translate planned; evidence `PRD.md:26`, `contracts/openapi/translate.yaml:302-395`; status planned parity.
16. Capability: glossary CRUD; Google glossary APIs; DeepL glossary APIs; Amazon terminology import/list/delete APIs; translate planned; evidence `contracts/openapi/translate.yaml:233-301`; status planned parity.
17. Capability: custom terminology; Google glossary; DeepL glossary; Amazon custom terminology; translate planned; evidence `IP-006-termbase-and-glossary-stack.md:51-69`; status planned parity.
18. Capability: parallel data; Google custom/adaptive paths; DeepL glossary/context; Amazon explicit parallel data; translate not explicitly named; evidence Amazon quota source; status gap.
19. Capability: custom model routing; Google custom models; DeepL next-gen model choices; Amazon custom terminology/parallel data; translate planned; evidence `ADR-TRANSLATE-0001:49-78`; status planned parity.
20. Capability: multi-engine routing; Google single-provider; DeepL single-provider; Amazon single-provider; translate planned; evidence `ADR-TRANSLATE-0001:49-78`; status differentiator.
21. Capability: fallback routing; Google cloud HA; DeepL hosted HA; Amazon hosted HA; translate planned; evidence `ADR-TRANSLATE-0001:76-78`; status differentiator.
22. Capability: residency-bound inference; Google regional controls vary; DeepL has EU endpoint options; Amazon regional service; translate planned strongly; evidence `ADR-TRANSLATE-0004:63-119`; status differentiator.
23. Capability: audit-chain events; Google logs but not Oyatie chain; DeepL API usage logs; Amazon CloudTrail; translate planned; evidence `PRD.md:104-113`, `contracts/asyncapi/translate-events.yaml:165-231`; status planned differentiator.
24. Capability: EU AI Act QE disclosure; Google not service-native; DeepL not service-native; Amazon not service-native; translate planned; evidence `ADR-TRANSLATE-0003:61-87`; status differentiator.
25. Capability: quality estimation API; Google not public core API; DeepL quality inferred but no QE endpoint; Amazon no QE endpoint; translate planned; evidence `PRD.md:58-60`, `contracts/openapi/translate.yaml:119-137`; status differentiator.
26. Capability: human review workflow; Google no; DeepL workflow through integrations; Amazon no; translate planned; evidence `PRD.md:30`, `ADR-TRANSLATE-0001:141-145`; status planned differentiator.
27. Capability: language-pair routing policy; Google no multi-vendor; DeepL no multi-vendor; Amazon no multi-vendor; translate planned; evidence `ADR-TRANSLATE-0001:61-72`; status differentiator.
28. Capability: policy pack overlays; Google no Oyatie packs; DeepL no Oyatie packs; Amazon no Oyatie packs; translate planned; evidence `policy/data-residency.md`, `PRD.md:121-130`; status differentiator.
29. Capability: Cedar tenant policy; counterparts no; translate planned; evidence `policy/translate-tenant-scope.cedar`; status differentiator but unproven.
30. Capability: OpenAPI contract; Google API docs yes; DeepL OpenAPI yes; Amazon SDK/API docs yes; translate yes; evidence `contracts/openapi/translate.yaml:1-395`; status parity.
31. Capability: AsyncAPI events; counterparts no comparable public event model; translate yes; evidence `contracts/asyncapi/translate-events.yaml:1-231`; status differentiator.
32. Capability: Proto/gRPC; Google gRPC yes in cloud libraries; DeepL HTTP primary; Amazon SDK APIs; translate yes; evidence `contracts/proto/translate.proto:23-67`; status planned parity/differentiator.
33. Capability: SDK plan; all counterparts have SDKs; translate has plan; evidence `sdk-plan.md`; status planned but unproven.
34. Capability: reference implementation; all counterparts have examples; translate has Markdown reference; evidence `reference-implementations/translate-with-router-fallback-rust-sdk.md`; status partial.
35. Capability: production quotas; Google public; DeepL public limits; Amazon public quotas; translate capacity model exists; evidence `capacity-model.md`; status partial.
36. Capability: usage/cost budgets; Google public pricing; DeepL plan usage; Amazon pricing/quotas; translate cost doc exists; evidence `cost-budget.md`; status partial.
37. Capability: deployment to public cloud; counterparts hosted; translate lacks required context module; evidence inventory; status gap.
38. Capability: deploy into customer AWS; Amazon native; Google/DeepL external; translate lacks `iac/guest-aws/`; status P1 gap.
39. Capability: deploy into customer OCI; counterparts external; translate lacks `iac/oci-guest/`; status P1 gap.
40. Capability: on-prem deployment; counterparts limited/enterprise-specific; translate planned by platform; missing `iac/on-prem/`; status P1 gap.
41. Capability: colo deployment; counterparts hosted; translate planned by platform; missing `iac/colo/`; status P1 gap.
42. Capability: Oyatie-as-cloud-provider; counterparts hosted; translate planned by platform; missing `iac/oyatie-iaas/`; status P1 gap.
43. Capability: OCI Always Free demo infrastructure; counterparts have free usage but not Oyatie profile; translate missing module; status P1 gap.
44. Capability: OS support manifest; counterparts SDKs vary; translate missing `supported-oses.json`; status P1 gap.
45. Capability: Rust backend implementation; counterparts opaque; translate required; no `src/`; status P2 gap.
46. Capability: conformance tests; counterparts docs/sdks; translate references tests but no `tests/`; status P2 gap.
47. Capability: SLO docs; counterparts publish quotas not always SLOs; translate has SLO files; status planned strength.
48. Capability: dashboards; counterparts cloud consoles; translate has dashboard JSON; status planned strength.
49. Capability: incident runbooks; counterparts support docs; translate has runbooks; status planned strength.
50. Capability: data protection impact assessment; counterparts publish compliance docs; translate has DPIA; status planned strength.
51. Capability: threat model; counterparts internal; translate has threat model; status planned strength.
52. Capability: compliance policy docs; counterparts compliance portals; translate has compliance docs; status planned strength.
53. Capability: migration from Smartling; counterparts not required; translate has playbook; status additive.
54. Capability: migration from Amazon Translate; Google/DeepL not; translate lacks specific playbook; status gap.
55. Capability: migration from Google Translate; DeepL/Amazon not; translate lacks specific playbook; status gap.
56. Capability: migration from DeepL; Google/Amazon not; translate lacks specific playbook; status gap.
57. Capability: source-language auto detection billing; Google documents no extra detection charge in pricing source; translate does not document billing; status gap.
58. Capability: document page counting; Google documents page billing; translate cost model should map pages and segments; status gap.
59. Capability: source code language compliance; counterparts irrelevant; translate scan found no forbidden files; status clean but absent implementation.
60. Capability: tenant-class semantics; counterparts have plans/accounts; translate old entitlement docs remain; status P2 gap.
61. Capability: revenue-share economics; counterparts standard billing; translate forward model requires it; status missing.
62. Capability: paid per-seat and usage billing; counterparts usage billing; translate old pricing ladder; status rewrite needed.
63. Capability: demo-trial caps; counterparts free tiers; translate missing tenant-class profile; status rewrite needed.
64. Capability: compliance packs allowed for paid; counterparts enterprise contracts; translate policy packs planned; status partial.
65. Capability: BYOK allowance; counterparts enterprise-specific; translate not clearly expressed in translate path; status gap.
66. Capability: best-effort demo SLO; counterparts free limits; translate old plan SLOs; status rewrite needed.
67. Capability: contractual paid SLO; counterparts enterprise contracts; translate SLO docs exist; status partial.
68. Capability: at-cost revenue-share substrate; counterparts not similar; translate missing; status gap.
69. Capability: region-aware vendor selection; Google/DeepL/Amazon have regional surfaces; translate ADR-0004 has pack policy; status planned strength.
70. Capability: legal content routing; counterparts raw APIs; translate ADR-0001/ADR-0004 planned; status planned strength.
71. Capability: medical content routing; counterparts raw APIs; translate policy docs likely cover content classes; status partial.
72. Capability: financial content routing; counterparts raw APIs; translate policy docs likely cover content classes; status partial.
73. Capability: unsupported-format rejection; counterparts document format limits; translate ADR-0005 planned; status partial.
74. Capability: conversion sandboxing; counterparts internal; translate ADR-0005 planned; status strength.
75. Capability: layout round-trip fidelity; DeepL/Google document surfaces; translate benchmark claims metadata preservation; status planned strength.
76. Capability: request size guardrails; Google/DeepL/Amazon publish; translate contract should declare; status needs explicit limits.
77. Capability: per-tenant rate limits; counterparts account quotas; translate old plan docs; status rewrite needed.
78. Capability: circuit breakers; counterparts hosted; translate manifest cites invariant; status partial.
79. Capability: bulkhead isolation; counterparts hosted; translate manifest cites invariant; status partial.
80. Capability: tenant rate-limit metrics; counterparts cloud metrics; translate dashboards likely; status partial.
81. Capability: audit event replay; counterparts not exposed; translate backfill-replay exists; status planned strength.
82. Capability: backfill replay; counterparts internal; translate doc exists; status planned strength.
83. Capability: model rollback; counterparts internal; translate QE rollback runbook exists; status planned strength.
84. Capability: TM corruption restore; counterparts not public; translate runbook exists; status planned strength.
85. Capability: caption stream stall response; counterparts not public; translate runbook exists; status planned strength.
86. Capability: sovereign cross-region incident response; counterparts compliance portals; translate runbook exists; status planned strength.
87. Capability: glossary conflict resolution; DeepL/Amazon terminology conflict customer-managed; translate runbook exists; status planned strength.
88. Capability: tenant data isolation; counterparts account/project isolation; translate IP-005 HMAC/RLS design; status planned strength.
89. Capability: cross-tenant TM sharing consent; counterparts enterprise-specific; translate old entitlement docs mention it; status needs tenant-class rewrite.
90. Capability: pack-local model registry; counterparts not public; translate old plan docs mention; status needs tenant-class rewrite.
91. Capability: pack-local audit signing; counterparts cloud logs; translate old plan docs mention; status needs tenant-class rewrite.
92. Capability: language pack whitelist; counterparts region/language controls; translate PRD and ADR; status planned strength.
93. Capability: content egress denial; counterparts enterprise policies; translate ADR-0004; status planned strength.
94. Capability: service catalog entries; counterparts product catalog; translate has catalog; status strength.
95. Capability: manifest; counterparts APIs; translate manifest exists; status strength but contains adjacent tenant-class terms.
96. Capability: deployment manifest consistency; counterparts owned; translate manifest and iac diverge; status gap.
97. Capability: benchmark methodology; counterparts provide docs/quotas; translate benchmark exists; status needs top-three refresh.
98. Capability: quality metrics BLEU/COMET; counterparts do not publish broad current numbers; translate local benchmark has numbers; status partial.
99. Capability: latency measurements; counterparts rarely publish public p99; translate local benchmark has numbers; status partial.
100. Capability: cost per million characters; Google public; DeepL plan-specific; Amazon public pricing outside this matrix; translate cost docs; status partial.
101. Capability: custom usage dashboards; counterparts have consoles; translate dashboards exist; status planned strength.
102. Capability: customer onboarding doc; counterparts docs; translate onboarding exists but uses retired terms; status rewrite needed.
103. Capability: engineer FAQ; counterparts docs; translate FAQ exists but uses retired terms; status rewrite needed.
104. Capability: tutorial; counterparts docs; translate tutorial exists but uses retired terms; status rewrite needed.
105. Capability: migration playbook; counterparts docs; translate only Smartling; status partial.
106. Capability: developer SDK generation; counterparts have SDKs; translate SDK plan exists; status partial.
107. Capability: API versioning; counterparts version APIs; translate OpenAPI version exists; status partial.
108. Capability: event versioning; counterparts internal; translate AsyncAPI exists; status partial.
109. Capability: schema evolution; counterparts SDK/API docs; translate Proto exists; status partial.
110. Capability: per-context OpenTofu module; counterparts not comparable; translate required; status P1 gap.
111. Capability: on-device/mobile translation client; Google/DeepL consumer apps; Amazon API only; translate not evidenced; status outside current docs unless frontend scope expands.
112. Capability: web UI translation widget; Google consumer/product; DeepL web; Amazon console; translate not evidenced; status possible gap.
113. Capability: admin console for termbase/TM; DeepL web, Google console, Amazon console; translate not evidenced in frontend; status gap if product requires UI.
114. Capability: API-only substrate; all counterparts yes; translate strong contracts; status planned parity.
115. Capability: marketplace/embed economics; counterparts standard billing; translate revenue-share model required by directive; status missing.
116. Capability: paid license + usage; counterparts usage/subscription; translate old tiers; status rewrite needed.
117. Capability: free demo limits; counterparts free credits/usage; translate missing demo-trial semantics; status rewrite needed.
118. Capability: compliance packs; counterparts enterprise compliance; translate policy docs; status partial.
119. Capability: BYOK; counterparts enterprise-specific; translate not explicit; status gap.
120. Capability: data deletion/customer training controls; counterparts have policies; translate not fully verified; status gap.
121. Capability: model-training consent; translate old docs mention; needs tenant-class/policy rewrite; status partial.
122. Capability: government/regulated mode; counterparts enterprise/regional; translate ADR/policy; status planned strength.
123. Capability: edge rate limits; counterparts account quotas; translate architecture mentions tenant_class; status rewrite needed.
124. Capability: quota self-service; counterparts consoles; translate not evidenced; status gap.
125. Capability: quota override workflow; counterparts support/console; translate not evidenced; status gap.
126. Capability: SLA dashboard; counterparts enterprise support; translate dashboard/SLO; status partial.
127. Capability: localization project workflow; DeepL/Smartling-style; translate planned human review; status partial.
128. Capability: file-format matrix; DeepL and Google publish; translate old docs include; status needs tenant-class rewrite.
129. Capability: language pair matrix; Google/DeepL/Amazon publish; translate not found as service artifact; status gap.
130. Capability: exact public product docs; counterparts have docs; translate docs are internal; status acceptable for microservice but external docs gap remains.

## 5. Family summary

1. API family: translate has planned parity through OpenAPI, Proto, and AsyncAPI.
2. API family: proof gap is runtime code and conformance tests.
3. Translation quality family: translate is ambitious through engine routing, TM, QE, and human review.
4. Translation quality family: public counterpart quality numbers are sparse, so local benchmark claims must remain reproducible.
5. Document family: translate intends to match or beat Google/DeepL/Amazon on file workflows.
6. Document family: translate needs executable round-trip tests before claiming parity.
7. Glossary/terminology family: translate matches the union target on paper.
8. Glossary/terminology family: Amazon custom terminology and parallel data need explicit mapping.
9. Residency/compliance family: translate is stronger on paper than the counterpart APIs.
10. Residency/compliance family: missing OpenTofu context modules weaken real deployability.
11. Streaming family: translate has a differentiating caption-stream plan.
12. Streaming family: stream stall runbook and AsyncAPI eventing are good supporting artifacts.
13. Batch/bulk family: translate covers batch and bulk API surfaces.
14. Batch/bulk family: Amazon quotas should be explicitly mapped because Amazon is now required top-three.
15. SDK/developer family: translate has SDK planning and a reference implementation.
16. SDK/developer family: no executable SDK conformance evidence was found.
17. Operations family: translate has SLO, dashboard, runbook, and incident docs.
18. Operations family: operations cannot be accepted without deployable OpenTofu and runtime code.
19. Commercial/entitlement family: translate is currently incoherent because retired entitlement docs remain.
20. Commercial/entitlement family: tenant-class adoption is missing.

## 6. Headline gap analysis

1. Gap A: Required top-three parity needs Amazon promoted to first-class in docs and adapter plans.
2. Gap B: Required deployment-context parity needs six OpenTofu modules.
3. Gap C: Required OCI demo-trial posture needs an OCI Always Free profile.
4. Gap D: Required OS posture needs `supported-oses.json`.
5. Gap E: Required Rust posture needs actual Rust backend files or canonical external implementation pointer.
6. Gap F: Required test posture needs local conformance tests for contracts and SLOs.
7. Gap G: Required tenant-class posture needs `demo_trial`, `paid`, and `revenue_share` semantics.
8. Gap H: Retired entitlement language must be removed from onboarding, tutorial, FAQ, benchmark, and deleted capability ladder docs.
9. Gap I: Translate needs a language-pair support matrix comparable to counterpart docs.
10. Gap J: Translate needs request-size and quota docs comparable to counterpart public limits.
11. Gap K: Translate needs migration guides from Google, DeepL, and Amazon, not only Smartling.
12. Gap L: Translate needs a root README and cross-microservice handoff doc or machine-readable equivalent.
13. Gap M: Translate needs UI/admin-console stance if it is a standalone localization product, not only API substrate.
14. Gap N: Translate needs explicit BYOK and compliance-pack allowance behavior under tenant classes.
15. Gap O: Translate needs revenue-share economics and cost guards for high-volume embedded/B2C tenants.

## 7. Additive surface to preserve

1. Preserve multi-engine routing because it is a translate differentiator.
2. Preserve residency-bound inference because it is stronger than raw vendor APIs.
3. Preserve audit-chain events because they connect translate to Oyatie governance.
4. Preserve TM isolation and consent semantics while rewriting commercial terminology.
5. Preserve termbase/glossary CRUD because it is required for DeepL/Amazon parity.
6. Preserve QE endpoint and EU AI Act trace design because it differentiates translate.
7. Preserve document sandboxing and round-trip fidelity goals.
8. Preserve real-time caption streaming because it is an additive product surface.
9. Preserve bulk and batch job APIs because they are required for Google/Amazon parity.
10. Preserve service catalog granularity because it gives component ownership clarity.
11. Preserve SLO files and dashboards because they are stronger than many product-only docs.
12. Preserve runbooks because they are operationally substantive.
13. Preserve DPIA, compliance, policy, and threat-model docs.
14. Preserve Smartling migration playbook as one migration family member.
15. Add Google migration playbook.
16. Add DeepL migration playbook.
17. Add Amazon Translate migration playbook.
18. Add language-pair support matrix.
19. Add request-limit and file-format matrix.
20. Add Amazon custom terminology and parallel-data mapping.
21. Add Google document/page quota mapping.
22. Add DeepL style/formality/glossary availability mapping.
23. Add tenant-class entitlement matrix.
24. Add demo-trial OCI Always Free infrastructure profile.
25. Add paid contractual scaling model.
26. Add revenue-share cost-control model.
27. Add all six OpenTofu deployment contexts.
28. Add OS support manifest.
29. Add Rust implementation or canonical pointer.
30. Add conformance tests.

## 8. Decision

1. Feature parity conclusion: translate is a strong planned product but not a deployable parity package.
2. Google parity is mostly planned, with gaps in scale proof, language matrix, and quota mapping.
3. DeepL parity is mostly planned, with gaps in document-format proof, style/formality mapping, and entitlement rewrite.
4. Amazon parity is under-emphasized and must be promoted in docs, adapter plans, and AWS context proof.
5. Oyatie differentiators should be kept, but they need implementation and platform proof.
6. The next remediation should not author another parity doc first.
7. The next remediation should create the missing canonical surfaces: tenant-class matrix, OpenTofu contexts, OS manifest, and implementation/test pointer.
8. After those surfaces exist, parity docs can be refreshed without reusing retired commercial plan language.

## 9. Corrective feature worklist

1. Add a Google-specific request-limit table covering 5,000-code-point recommendation, 30,000-code-point Advanced limit, 100,000-byte Basic limit, 6,000 v3 requests/min, and 2,400 document pages/min.
2. Add a DeepL-specific request-limit table covering 128 KiB text body, 16 KiB headers, 30 MB Pro documents, 1,000,000-character Pro documents, 10 MB XLIFF, and 150 KB SRT.
3. Add an Amazon-specific request-limit table covering 10,000-byte sync input, 100,000-byte real-time document cap, 20 MB async document cap, 5 GB batch cap, 10 concurrent batch jobs, and 1,000 queued jobs.
4. Add a language-pair matrix with source, target, route preference, fallback order, residency pack, and unsupported-pair response.
5. Add an Amazon custom terminology mapping to the termbase/glossary model.
6. Add an Amazon parallel-data mapping to the TM/model adaptation model.
7. Add a Google glossary/custom-model mapping to the termbase/model routing model.
8. Add a DeepL formality/style mapping to content class and locale policy.
9. Add route-specific payload validation tests for Google, DeepL, and Amazon.
10. Add document round-trip tests for PDF, DOCX, PPTX, XLSX, XLIFF, HTML, TXT, and SRT.
11. Add real-time caption stream tests for chunk order, correction events, backpressure, and reconnect replay.
12. Add TM isolation tests for tenant boundary, HMAC lookup, exact match, fuzzy match, and consented cross-tenant sharing.
13. Add QE tests for score calibration, bias drift, model rollback, and EU AI Act trace completeness.
14. Add deployment-context feature proofs for all six contexts after OpenTofu modules exist.
15. Add demo-trial cap behavior without changing translation quality.
16. Add paid contract behavior with scale, compliance packs, and BYOK allowance.
17. Add revenue-share behavior with at-cost substrate and gross-revenue share guardrails.
18. Rewrite onboarding so the first-week exercise uses tenant class and policy pack configuration.
19. Rewrite tutorial commands so they set `tenant_class`.
20. Rewrite FAQ concurrency answers around tenant class, purchased capacity, and deployment context.
21. Rewrite benchmark language around single targets plus overlays.
22. Rewrite migration playbooks to include Google Translate, DeepL, and Amazon Translate.
23. Add a root README that points to PRD, architecture, contracts, deploy contexts, SLOs, and validation commands.
24. Add a cross-microservice handoff artifact or a machine-readable equivalent in the manifest.
25. Refresh the local competitor matrix so the first comparison family is exactly Google Translate, DeepL, and Amazon Translate.
