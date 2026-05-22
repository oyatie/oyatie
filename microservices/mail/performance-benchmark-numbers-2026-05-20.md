# Mail Performance Benchmark Numbers

Audit date: 2026-05-20
Target microservice: `mail`
Counterparts: Gmail, Microsoft Outlook, Proton Mail
Methodology status: public counterpart limits are sourced from official public docs where available; latency numbers for counterparts are either product-limit numbers or estimates from the existing mail benchmark artifact, explicitly marked.
Target model: one industry-leader-grade Oyatie target set with deployment-context overlays and tenant-class overlays.
Forbidden model not used: no new capability rank rows or headings are introduced here.

## Citation Anchor Block

1. Oyatie PRD latency and SLO targets: `microservices/mail/PRD.md:952-968`.
2. Oyatie availability and recovery targets: `microservices/mail/PRD.md:972-982`.
3. Existing internal counterpart benchmark artifact: `microservices/mail/benchmarks/gmail-m365-proton-vs-oyatie.md:11-121`.
4. Google Workspace Gmail sending limits: `https://support.google.com/a/answer/166852?hl=en`.
5. Google Workspace Gmail bandwidth limits: `https://support.google.com/a/answer/1071518?hl=en-GB`.
6. Microsoft Exchange Online limits: `https://learn.microsoft.com/en-us/office365/servicedescriptions/exchange-online-service-description/exchange-online-limits`.
7. Proton Mail sending limits: `https://proton.me/support/email-sending-limits`.
8. Proton Mail attachment limits: `https://proton.me/support/attaching-multiple-documents-to-a-message`.
9. Proton Mail storage support: `https://proton.me/support/increase-storage-space`.

## 1. Methodology

1. Benchmark dimension A: user-perceived latency for compose, send submit, inbox open, mailbox fetch, thread render, search, and workflow handoff.
2. Benchmark dimension B: protocol throughput for SMTP submit, inbound SMTP receive, IMAP/JMAP fetch, mailbox search, and event emission.
3. Benchmark dimension C: deliverability and trust latency for DKIM signing, SPF/DMARC evaluation, MTA-STS/TLSRPT processing, bounce handling, and reputation feedback.
4. Benchmark dimension D: compliance latency for DLP classification, legal-hold engagement, eDiscovery export freshness, and audit-chain sealing.
5. Benchmark dimension E: storage and recovery for mailbox quota, attachment size, object-store retrieval, backup restore, and mailbox rebuild.
6. Benchmark dimension F: anti-abuse quality for spam precision, spam recall, phishing precision, false positive rate, and classifier latency.
7. Benchmark dimension G: scale ceiling for sustained inbound mail, sustained outbound mail, mailbox count per cell, concurrent protocol sessions, and search corpus size.
8. Benchmark dimension H: resilience for monthly availability, RPO, RTO, queue drain, and region or cell failover.
9. Test workload W1: personal mailbox with 25k messages, 10 GB mail corpus, 10 active folders/labels, 20 saved searches, and moderate attachment traffic.
10. Test workload W2: work tenant with 10k users, 100M messages, 1 TB historical import, 20 custom domains, legal hold active for 2% of mailboxes.
11. Test workload W3: regulated work tenant with HIPAA/GDPR/KR pack overlays, BYOK, audit-chain sealing, DLP, and eDiscovery export.
12. Test workload W4: high-volume marketplace tenant with transactional mail, receipts, shipping notices, support bridges, bounce handling, and abuse feedback.
13. Test workload W5: demo_trial tenant on OCI Always Free profile with time and usage caps, small mailbox set, and best-effort SLO.
14. OS disclosure: current mail path lacks `supported-oses.json`; targets assume the canonical OS matrix once implementation evidence exists.
15. Architecture disclosure: current mail path lacks `src/` and `tests/`; Oyatie target numbers are design targets until Rust benchmark harnesses exist.
16. Deployment context disclosure: all six contexts remain in scope, but current mail path lacks context-specific OpenTofu modules.
17. Tenant class disclosure: this benchmark uses `demo_trial`, `paid`, and `revenue_share` as business overlays, not feature-quality segments.
18. Methodology disclosure: public SaaS providers do not publish most internal p50/p95/p99 latency numbers for mail operations.
19. Methodology disclosure: public SaaS provider rows therefore use official public limits where available and source-limited estimates where latency is not exposed.
20. Methodology disclosure: internal Oyatie benchmark artifact rows are treated as draft design evidence because the referenced harness is not present in inventory.
21. Methodology disclosure: the current document avoids copying the retired rank rows from the older benchmark file.
22. Methodology disclosure: "source-limited estimate" means the number is useful for target-setting but not a verified public SLA from the counterpart.
23. Methodology disclosure: "canonical target" means the desired Oyatie production target after the missing Rust, OpenTofu, OS, and test evidence exists.
24. Methodology disclosure: "context overlay" means infrastructure limits can cap throughput or elasticity without lowering product-quality standards.
25. Methodology disclosure: "tenant_class overlay" means billing, usage cap, compliance eligibility, and support terms differ while the correctness bar stays uniform.

## 2. Counterpart Numbers

### 2.1 Gmail Numbers

1. Gmail daily sending limit for paid Google Workspace user accounts: 2,000 messages per day; source: Google Workspace Gmail sending limits.
2. Gmail mail merge sending limit for paid accounts: 1,500 messages per day; source: Google Workspace Gmail sending limits.
3. Gmail trial-account sending limit: 500 messages per day; source: Google Workspace Gmail sending limits.
4. Gmail total recipients per message: 2,000 total, with a 500 external-recipient maximum; source: Google Workspace Gmail sending limits.
5. Gmail SMTP by POP/IMAP recipients per message: 100; source: Google Workspace Gmail sending limits.
6. Gmail API recipients per message: 500; source: Google Workspace Gmail sending limits.
7. Gmail total recipients per day: 10,000; source: Google Workspace Gmail sending limits.
8. Gmail external recipients per day: 3,000; source: Google Workspace Gmail sending limits.
9. Gmail unique recipients per day: 3,000 total and 2,000 external; source: Google Workspace Gmail sending limits.
10. Gmail auto-forwarded messages per account: 10,000 per day; source: Google Workspace Gmail sending limits.
11. Gmail auto-forward mail filters per account: 40; source: Google Workspace Gmail sending limits.
12. Gmail web client download bandwidth: 750 MB/hour and 1,250 MB/day; source: Google Workspace Gmail bandwidth limits.
13. Gmail web client upload bandwidth: 300 MB/hour and 1,500 MB/day; source: Google Workspace Gmail bandwidth limits.
14. Gmail IMAP download bandwidth: 2,500 MB/day; source: Google Workspace Gmail bandwidth limits.
15. Gmail IMAP upload bandwidth: 500 MB/day; source: Google Workspace Gmail bandwidth limits.
16. Gmail POP download bandwidth: 1,250 MB/day; source: Google Workspace Gmail bandwidth limits.
17. Gmail outgoing attachment limit: commonly 25 MB before Drive-style large-file handoff; source: Gmail product support and user documentation.
18. Gmail DKIM signing p95 reference in local benchmark: source-limited estimate from `benchmarks/gmail-m365-proton-vs-oyatie.md:19-30`.
19. Gmail spam classifier F1 reference in local benchmark: source-limited estimate from `benchmarks/gmail-m365-proton-vs-oyatie.md:58-73`.
20. Gmail JMAP latency row: not applicable because Gmail does not expose JMAP as its primary public mail API.

### 2.2 Microsoft Outlook and Exchange Online Numbers

1. Exchange Online message rate limit: 30 messages per minute; source: Microsoft Exchange Online limits.
2. Exchange Online recipient rate limit: 10,000 recipients per day; source: Microsoft Exchange Online limits and Microsoft SMTP submission guidance.
3. Exchange Online meeting invitation recipient limit: 5,000 recipients; source: Microsoft Exchange Online limits.
4. Exchange Online message size limit for Outlook in Plan 1/Plan 2: up to 150 MB; source: Microsoft Exchange Online limits.
5. Exchange Online OWA message size limit in Plan 1/Plan 2: 112 MB in the published table; source: Microsoft Exchange Online limits.
6. Exchange Online Outlook for iOS and Android message size limit: 33 MB for Online plans in the published table; source: Microsoft Exchange Online limits.
7. Exchange Online encrypted message size limit with Microsoft Purview Message Encryption: 100 MB; source: Microsoft Exchange Online limits.
8. Exchange Online legacy encrypted message size limit: 25 MB; source: Microsoft Exchange Online limits.
9. Exchange Online file attachment count limit: 250 attachments; source: Microsoft Exchange Online limits.
10. Exchange Online Outlook file attachment size limit: up to 150 MB for Online plans; source: Microsoft Exchange Online limits.
11. Exchange Online OWA file attachment size limit: 112 MB for Online plans in the published table; source: Microsoft Exchange Online limits.
12. Exchange Online mobile file attachment size limit: 33 MB for Online plans in the published table; source: Microsoft Exchange Online limits.
13. Exchange Online multipart message limit: 250 parts; source: Microsoft Exchange Online limits.
14. Exchange Online embedded message depth limit: 30 embedded messages; source: Microsoft Exchange Online limits.
15. Outlook mailbox storage common ceiling: up to 100 GB per mailbox in current Outlook/Exchange Online support surfaces; source: Microsoft mailbox storage support and Exchange Online service descriptions.
16. Outlook DKIM signing p95 reference in local benchmark: source-limited estimate from `benchmarks/gmail-m365-proton-vs-oyatie.md:19-30`.
17. Outlook spam classifier F1 reference in local benchmark: source-limited estimate from `benchmarks/gmail-m365-proton-vs-oyatie.md:58-73`.
18. Outlook JMAP latency row: not applicable because Outlook/Exchange does not expose JMAP as the primary public protocol.

### 2.3 Proton Mail Numbers

1. Proton Mail Free sending limit: 50 emails per hour; source: Proton Mail sending limits.
2. Proton Mail Free daily sending limit: 150 emails per day; source: Proton Mail sending limits.
3. Proton Mail paid-plan recipient maximum per email: 100 recipients; source: Proton Mail sending limits.
4. Proton Mail inbound mail limit: unlimited according to the sending-limits support page; source: Proton Mail sending limits.
5. Proton Mail suspicious sending block duration: up to 48 hours at Proton discretion; source: Proton Mail sending limits.
6. Proton Mail outgoing attachment limit: 25 MB; source: Proton Mail attachment limits.
7. Proton Mail incoming attachment limit: 50 MB; source: Proton Mail attachment limits.
8. Proton Mail maximum attachments per email: 100 files; source: Proton Mail attachment limits.
9. Proton Free mail storage: up to 1 GB Mail storage with free-plan storage context; source: Proton storage support.
10. Proton Mail Plus storage: 15 GB Mail storage; source: Proton storage support.
11. Proton account free bundle storage: up to 6 GB total, split between Mail and Drive in support docs; source: Proton storage support.
12. Proton automatic encryption between Proton users: qualitative security number, not a latency metric; source: Proton Mail support.
13. Proton password-protected external messages: qualitative security surface, not a public throughput metric; source: Proton Mail support.
14. Proton DKIM signing p95 reference in local benchmark: source-limited estimate from `benchmarks/gmail-m365-proton-vs-oyatie.md:19-30`.
15. Proton spam classifier F1 reference in local benchmark: source-limited estimate from `benchmarks/gmail-m365-proton-vs-oyatie.md:58-73`.
16. Proton JMAP latency row: not applicable because Proton does not expose JMAP as its primary public protocol.

## 3. Oyatie Target Numbers: Single Industry-Leader Target Set

### 3.1 Canonical User-Visible Latency Targets

1. Compose draft open p50: 50 ms; canonical target from `PRD.md:952-968`.
2. Compose draft open p95: 200 ms; canonical target from `PRD.md:952-968`.
3. Compose draft open p99: 500 ms; canonical target from `PRD.md:952-968`.
4. Send submit p50: 100 ms; canonical target from `PRD.md:952-968`.
5. Send submit p95: 200 ms; canonical target from `PRD.md:952-968`.
6. Send submit p99: 500 ms; canonical target from `PRD.md:952-968`.
7. Mailbox fetch p50: 40 ms; canonical target from `PRD.md:952-968`.
8. Mailbox fetch p95: 100 ms; canonical target from `PRD.md:952-968`.
9. Mailbox fetch p99: 300 ms; canonical target from `PRD.md:952-968`.
10. Search p50: 100 ms; canonical target from `PRD.md:952-968`.
11. Search p95: 500 ms; canonical target from `PRD.md:952-968`.
12. Search p99: 2 seconds; canonical target from `PRD.md:952-968`.
13. Workflow handoff p50: 200 ms; canonical target from `PRD.md:952-968`.
14. Workflow handoff p95: 500 ms; canonical target from `PRD.md:952-968`.
15. Workflow handoff p99: 2 seconds; canonical target from `PRD.md:952-968`.
16. Smart compose suggestion p50: 50 ms; canonical target from `PRD.md:952-968`.
17. Smart compose suggestion p95: 150 ms; canonical target from `PRD.md:952-968`.
18. Smart compose suggestion p99: 500 ms; canonical target from `PRD.md:952-968`.
19. Smart reply p50: 100 ms; canonical target from `PRD.md:952-968`.
20. Smart reply p95: 300 ms; canonical target from `PRD.md:952-968`.
21. Smart reply p99: 1 second; canonical target from `PRD.md:952-968`.

### 3.2 Canonical Mail-Transport Targets

1. Inbound SMTP accept p50: 200 ms; canonical target from `PRD.md:952-968`.
2. Inbound SMTP accept p95: 1 second; canonical target from `PRD.md:952-968`.
3. Inbound SMTP accept p99: 3 seconds; canonical target from `PRD.md:952-968`.
4. Outbound MX delivery p50: 5 seconds; canonical target from `PRD.md:952-968`.
5. Outbound MX delivery p95: 30 seconds; canonical target from `PRD.md:952-968`.
6. Outbound MX delivery p99: 5 minutes; canonical target from `PRD.md:952-968`.
7. Sustained ingestion per cell: 100,000 mails/sec; canonical target from `PRD.md:952-968`.
8. DKIM signing p95: less than 10 ms; source: benchmark target reading and ADR-MAIL-001 intent.
9. DMARC policy evaluation p95: less than 25 ms; target derived from mail authentication needing to stay below SMTP accept p95.
10. SPF lookup budget p95: less than 100 ms including DNS cache; target derived from SMTP accept budget.
11. MTA-STS policy fetch/cache p95: less than 100 ms on cache hit and less than 1 second on cold fetch; target derived from delivery budget.
12. TLSRPT ingestion p95: less than 1 second for report event acceptance; target derived from observability pipeline.
13. Bounce classification p95: less than 500 ms; target derived from queue and deliverability operations.
14. Queue drain after downstream outage: under 15 minutes for normal backlog and under 2 hours for regional incident backlog; target requires future runbook proof.
15. IP warm-up safety: no automated jump above tenant-approved recipient velocity; target is correctness-first rather than max throughput.

### 3.3 Canonical Compliance and Recovery Targets

1. DLP scan p50: 50 ms; canonical target from `PRD.md:952-968`.
2. DLP scan p95: 150 ms; canonical target from `PRD.md:952-968`.
3. DLP scan p99: 500 ms; canonical target from `PRD.md:952-968`.
4. Legal-hold engage p50: 500 ms; canonical target from `PRD.md:952-968`.
5. Legal-hold engage p95: 2 seconds; canonical target from `PRD.md:952-968`.
6. eDiscovery export completion: under 24 hours for 10 TB tenant corpus; canonical target from `PRD.md:952-968`.
7. Mailbox restore RTO: under 15 minutes for a single mailbox; canonical target from `PRD.md:952-968`.
8. Mail RPO: 0 for accepted messages; canonical target from `PRD.md:972-982`.
9. Regional RTO: under 15 minutes for failover; canonical target from `PRD.md:972-982`.
10. Personal Mail availability: 99.95% monthly; canonical target from `PRD.md:972-982`.
11. Work Mail availability: 99.99% monthly; canonical target from `PRD.md:972-982`.
12. Dual-context isolation error budget: zero tolerated cross-context leakage; source: `slos/dual-context-correctness.openslo.yaml`.
13. Audit-chain sealing p95: less than 500 ms per compliance event; target derived from legal hold and workflow handoff budgets.
14. Recovery trustee action audit latency p95: less than 1 second; target derived from ADR-MAIL-0001 recovery flow.
15. Key rotation propagation p95: less than 10 minutes for DKIM selector readiness after DNS propagation is observed; target requires future OpenTofu/DNS proof.

### 3.4 Canonical Storage and Client Limits

1. Mailbox default production quota target: 100 GB per active work mailbox, matching Outlook-class expectations.
2. Mailbox high-volume tenant quota target: scale by contract and object-store policy, not by reduced feature quality.
3. Personal mailbox default target: 15 GB minimum to match Proton Mail Plus class storage; paid and revenue-share contexts may scale higher.
4. Demo_trial mailbox quota target: capped by OCI Always Free profile budget; recommend 1 GB to 5 GB depending pack-free trial duration.
5. Attachment outgoing target: 25 MB direct SMTP-compatible baseline to match Gmail and Proton.
6. Attachment large-file target: object-store link handoff for files above 25 MB, with policy and expiration controls.
7. Attachment incoming target: at least 50 MB to match Proton incoming support and Outlook mobile/web ranges.
8. Attachment enterprise target: up to 150 MB where both sender and recipient context can safely support it, matching Exchange Online upper bound.
9. IMAP download target: at least 2,500 MB/day per account to meet Gmail documented IMAP download limit.
10. IMAP upload target: at least 500 MB/day per account to meet Gmail documented IMAP upload limit.
11. JMAP fetch p95 target: 100 ms or better for hot mailbox list fetch, from PRD fetch target.
12. Concurrent IMAP/JMAP sessions per mailbox: target 10 active sessions before throttling; final number needs Rust harness proof.
13. Concurrent tenant protocol sessions per cell: target 100k concurrent sessions for production cells; final number requires implementation benchmark.
14. Search corpus target: 100M messages per work tenant within p95 500 ms for common queries; derived from W2 workload and PRD search target.
15. Backfill import target: 1,000 messages/sec per tenant during controlled migration, matching existing migration-playbook ambition while avoiding old model language.

### 3.5 Deployment-Context Overlays

1. `oyatie-public-cloud` overlay: full canonical target set applies with elastic capacity and Oyatie-managed reputation, DNS, OpenBao, storage, queue, and observability.
2. `oyatie-public-cloud` overlay: public cloud must publish SLO evidence before using 99.99% Work Mail claim.
3. `guest-on-aws` overlay: full canonical target set applies if tenant account quotas, SES or egress posture, DNS, and storage are explicitly modeled in OpenTofu.
4. `guest-on-aws` overlay: throughput may be capped by customer account limits until quota increases are approved.
5. `guest-on-oci` overlay: full target set applies on paid OCI substrate when quotas, VCN, egress, DNS, and mail reputation controls are modeled.
6. `guest-on-oci` overlay: OCI Always Free profile is a demo_trial infrastructure profile, not a production-capacity proof.
7. `guest-on-oci` overlay: OCI Always Free profile must respect the canonical 4 OCPU and 24 GB Ampere ceiling from ADR-0328.
8. `guest-on-oci` overlay: OCI Always Free profile must cap outbound mail and storage aggressively because free email-delivery allotments are small.
9. `on-prem` overlay: targets apply only when the facility supplies sufficient compute, storage, DNS authority, egress reputation, monitoring, and OpenBao/HSM posture.
10. `on-prem` overlay: deliverability target is facility-constrained because IP reputation is customer or operator dependent.
11. `colo` overlay: targets apply when colo facility and network posture satisfy redundancy, egress, physical access, and reputation constraints.
12. `colo` overlay: queue drain and failover targets need facility-specific topology evidence.
13. `oyatie-as-cloud-provider` overlay: full target set applies with native Oyatie provider primitives and should be the cleanest reference deployment.
14. `oyatie-as-cloud-provider` overlay: this context should become the benchmark control plane once implementation exists.
15. Cross-context overlay: no context may claim production readiness until OpenTofu modules, OS manifest, Rust source, tests, and runbooks are present.

### 3.6 Tenant-Class Overlays

1. `demo_trial` overlay: same correctness and privacy bar as other classes.
2. `demo_trial` overlay: free, time-capped, and usage-capped.
3. `demo_trial` overlay: uses OCI Always Free profile where possible.
4. `demo_trial` overlay: best-effort SLO, no compliance packs, and no BYOK.
5. `demo_trial` overlay: recommend 150 outgoing messages/day initial cap to align with privacy-provider free-account norms and protect reputation.
6. `demo_trial` overlay: recommend 100 recipients/message cap until tenant reputation exists.
7. `demo_trial` overlay: recommend 1 GB to 5 GB mailbox storage cap depending trial length and Always Free storage budget.
8. `demo_trial` overlay: no bulk send, no high-volume automation, and no custom reputation pool until conversion.
9. `paid` overlay: per-seat plus usage billing.
10. `paid` overlay: any deployment context can be used if context prerequisites are met.
11. `paid` overlay: contractual SLO allowed.
12. `paid` overlay: compliance packs allowed.
13. `paid` overlay: BYOK allowed.
14. `paid` overlay: throughput scales with paid capacity and approved sender reputation.
15. `paid` overlay: large mailbox and eDiscovery workloads scale with contracted storage and compute.
16. `revenue_share` overlay: Oyatie takes a percentage of customer gross revenue.
17. `revenue_share` overlay: substrate runs at-cost or zero-margin.
18. `revenue_share` overlay: same production quality bar as paid.
19. `revenue_share` overlay: throughput scales to marketplace, B2C operator, embedded SaaS reseller, or affiliate partner business volume.
20. `revenue_share` overlay: hard cost guardrails must be visible because the substrate is intentionally low-margin.
21. Cross-class overlay: no class may receive weaker privacy, security, or protocol correctness.
22. Cross-class overlay: only quotas, economics, compliance eligibility, support terms, and substrate budget differ.

## 4. Comparison Narrative

1. Daily send volume: Gmail's paid Workspace 2,000 messages/day and Outlook's 10,000 recipients/day are public guardrails; Oyatie paid/revenue-share should support contract-driven higher sender volume after warm-up, while demo_trial should stay capped.
2. Attachment size: Gmail and Proton converge around 25 MB outgoing; Outlook supports larger enterprise attachments up to 150 MB in some clients, so Oyatie should support 25 MB direct mail and large-file object handoff, with enterprise contexts optionally supporting larger messages.
3. IMAP bandwidth: Gmail publishes 2,500 MB/day IMAP download and 500 MB/day upload; Oyatie should meet or exceed those numbers for paid/revenue-share tenants and cap demo_trial by profile.
4. Mailbox storage: Outlook reaches 100 GB mailbox expectations, Proton Mail Plus publishes 15 GB, and Gmail varies by Workspace storage pool; Oyatie should target at least 100 GB work mailbox capability where paid economics support it.
5. Proton free sending: Proton's 150/day free cap is a useful demo_trial guardrail because it protects reputation; Oyatie should not let demo_trial become an abusive bulk sender.
6. User-visible latency: public counterparts do not publish complete p50/p95/p99 mail UI latency, so Oyatie's PRD targets become the internal bar.
7. Compose latency: Oyatie p95 200 ms is industry-leader grade if implemented and measured.
8. Send submit latency: Oyatie p95 200 ms is aggressive and needs queue architecture proof.
9. Mailbox fetch latency: Oyatie p95 100 ms is strong against Gmail/Outlook perceived expectations and especially important for JMAP-first UX.
10. Search latency: Oyatie p95 500 ms and p99 2 seconds are credible if Tantivy index and corpus partitioning are proven.
11. Inbound SMTP accept: Oyatie p95 1 second is reasonable but must include DKIM/SPF/DMARC and anti-abuse path costs.
12. Outbound MX delivery: Oyatie p95 30 seconds is competitive, but real-world delivery is receiver-dependent.
13. Ingestion scale: 100,000 mails/sec per cell is above ordinary public-account limits and must be proven with Rust benchmarks before claims.
14. Availability: 99.99% monthly for Work Mail is enterprise-grade but requires context OpenTofu, multi-region evidence, and runbook integrity.
15. RPO: zero for accepted messages is an appropriate mail invariant and must be tested with queue/store crash recovery.
16. RTO: 15 minutes is plausible for mailbox restore and regional failover only if backup, object-store, DNS, and queue recovery evidence is present.
17. Anti-spam accuracy: Gmail remains the catch-up bar because public and internal benchmark context identify Gmail as strongest.
18. Privacy: Proton remains the catch-up bar for user trust and E2EE semantics; Oyatie can move ahead if operator non-access and recovery are proven.
19. Enterprise compliance: Outlook remains the catch-up bar for eDiscovery, retention, DLP, and admin operations.
20. JMAP: Oyatie can be ahead because Gmail, Outlook, and Proton do not use JMAP as the primary public API surface.
21. Self-hostable contexts: Oyatie can be ahead because the counterpart SaaS products do not generally offer the same six-context provider-agnostic deployment target.
22. OCI Always Free: Oyatie should treat this as a constrained demo_trial profile, not as evidence of production scale.
23. OpenTofu evidence: current Oyatie is behind its own canonical bar because the mail path lacks context modules.
24. OS evidence: current Oyatie is behind its own canonical bar because `supported-oses.json` is absent.
25. Rust evidence: current Oyatie is behind its own canonical bar because no `src/` or `tests/` path exists.
26. Benchmark evidence: current Oyatie is behind its own canonical bar because the old benchmark file references a harness not present in inventory.
27. Tenant-class evidence: current Oyatie is behind its own canonical bar because `tenant_class` does not appear in the mail path.
28. Overall performance position: targets are industry-leader grade on paper, but implementation evidence is insufficient for production claims.
29. Ahead areas after proof: JMAP-first API, provider-agnostic deployment, dual-context isolation, Cedar policies, and privacy plus enterprise unification.
30. Parity areas after proof: compose, fetch, search, attachment, admin, DKIM/SPF/DMARC, DLP, legal hold, and mailbox restore.
31. Catch-up areas after proof begins: spam classifier quality, Outlook-grade transport rules/shared mailboxes, Proton-grade E2EE UX, and migration from Outlook/Proton.
32. Source-limited areas: counterpart UI latency, internal classifier quality, exact search latency, exact DKIM signing latency, and provider-internal queue behavior.
33. Required next benchmark action: create a Rust benchmark harness for compose, send, fetch, search, DKIM, SMTP accept, DLP, legal hold, restore, and ingestion.
34. Required next benchmark action: run harness across at least one reference context and one constrained OCI Always Free profile.
35. Required next benchmark action: publish results by deployment context and tenant class overlay without reducing the quality bar.
36. Required next benchmark action: bind benchmark runs to `supported-oses.json` once the OS manifest exists.
37. Required next benchmark action: remove retired rank flags from benchmark commands and replace them with `--tenant-class` and `--deployment-context`.
38. Final benchmark verdict: Oyatie mail targets can meet or beat the top-three union bar, but current artifacts only prove target ambition, not measured readiness.

## 5. Benchmark Acceptance Ledger

1. Compose benchmark acceptance: Rust harness opens a representative compose draft and records p50, p95, and p99 against the PRD target.
2. Send-submit benchmark acceptance: Rust harness submits signed outbound mail and records API latency separately from downstream MX delivery.
3. Inbound SMTP benchmark acceptance: Rust harness injects valid and invalid SMTP sessions and records accept latency, reject correctness, and queue persistence.
4. Outbound MX benchmark acceptance: harness separates queue admission, DNS/MX lookup, TLS negotiation, remote acceptance, bounce, and retry latency.
5. DKIM benchmark acceptance: harness signs RSA and Ed25519 messages and records signing latency, selector lookup latency, and verification correctness.
6. SPF benchmark acceptance: harness uses warm and cold DNS cache cases and records policy evaluation under lookup-budget constraints.
7. DMARC benchmark acceptance: harness records quarantine/reject decision latency and false-positive handling for aligned and non-aligned mail.
8. MTA-STS benchmark acceptance: harness records policy cache hit, policy cache miss, and TLS failure behavior.
9. TLSRPT benchmark acceptance: harness ingests report batches and records event acceptance plus dashboard availability.
10. JMAP benchmark acceptance: harness fetches mailbox lists, message bodies, thread views, attachments, and changes using realistic mailbox sizes.
11. IMAP benchmark acceptance: harness runs compatibility sessions and records bandwidth throttling, concurrent session behavior, and mailbox consistency.
12. Search benchmark acceptance: harness indexes and queries at 25k, 1M, 10M, and 100M message corpus sizes.
13. Attachment benchmark acceptance: harness sends 25 MB direct messages, large-file handoff messages, and incoming 50 MB messages.
14. DLP benchmark acceptance: harness scans PHI, PII, secrets, harmless attachments, and adversarial boundary cases.
15. Legal-hold benchmark acceptance: harness engages hold on active and archived mailboxes and records audit-chain sealing latency.
16. eDiscovery benchmark acceptance: harness exports tenant corpora at 100 GB, 1 TB, and 10 TB scale and records completion time.
17. Mailbox restore benchmark acceptance: harness restores single mailbox, folder subset, and point-in-time message set cases.
18. Queue-drain benchmark acceptance: harness simulates receiver outage and measures backlog drain without message loss.
19. Abuse benchmark acceptance: harness feeds spam, phishing, bulk legitimate, and transactional mail and records precision, recall, and false-positive rate.
20. Privacy benchmark acceptance: harness proves operator non-access paths for personal mail and BYOK-controlled work mail.
21. Cross-context benchmark acceptance: run at least one benchmark slice in each deployment context after OpenTofu modules exist.
22. OCI Always Free benchmark acceptance: run constrained demo_trial workload separately and report profile caps without production overclaim.
23. Tenant-class benchmark acceptance: report `demo_trial`, `paid`, and `revenue_share` overlays as cap/economics differences, not target-quality differences.
24. OS benchmark acceptance: bind every benchmark run to OS, architecture, kernel/runtime profile, and container base.
25. Regression benchmark acceptance: every future performance target change must include old number, new number, rationale, and affected contexts.
26. Evidence acceptance: raw benchmark output, harness version, commit SHA, context, tenant class, OS, and architecture must be captured together.
27. Counterpart acceptance: public counterpart numbers must cite official support or service-description sources when available.
28. Estimate acceptance: estimates must be labeled source-limited and must not be promoted to counterpart SLA facts.
29. Readiness acceptance: no production performance claim is valid until source, tests, OpenTofu, OS manifest, and runbook evidence are present.
30. Final acceptance rule: performance reporting remains a single industry-leader target set with overlays until a newer directive changes the model.
