---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297, ADR-0273, ADR-0243, ADR-0263]
acceptance_status: draft
companion_docs:
  - microservices/mail/policy/anti-phishing.cedar
  - microservices/mail/policy/abuse-defence.cedar
  - microservices/mail/runbooks/account-compromise-recovery.md
  - microservices/mail/dashboards/abuse-defence-outcomes.json
inbound_citations: [microservices/mail/manifest.json]
---

# IP-017: Anti-phishing edge wiring

## A. Problem

Inbound mail is an adversarial edge. The architecture walkthrough says the SMTP path must verify DKIM, SPF, DMARC, and ARC before storage, then evaluate `policy/abuse-defence.cedar` and `policy/anti-phishing.cedar`. The stamped version of this IP named those checks but did not define where the verdict is produced, how borderline messages avoid user friction, or how quarantine and audit evidence prove a policy decision.

This IP closes that gap for `oya-mail-inbound-smtp-adapter-smtp`, `oya-mail-inbound-smtp-app`, and `oya-mail-anti-phishing-kernel`. It turns phishing detection into a typed edge decision instead of a generic "spam filter" note.

## B. Approach

Split the path into deterministic authentication, enrichment, classifier scoring, Cedar authorization, and delivery action. The SMTP adapter parses message metadata and authentication results; the anti-phishing kernel produces `phishing_score`, lookalike-domain findings, URL reputation findings, and attachment risk hints; Cedar converts those facts into allow, quarantine, or reject; the inbound app emits `oya.mail.anti-phishing-block` or `oya.mail.deliver` audit events.

Borderline scores use quarantine rather than hard reject so legitimate transactional mail is not silently lost. Clean mail sees zero extra UI friction. Quarantine release is a Cedar-governed workflow that records who released, why, and which message digest was released.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/mail/catalog/oya-mail-anti-phishing-kernel.yaml` | bind the catalog row to phishing score, URL reputation, impersonation, and quarantine verdicts |
| `microservices/mail/policy/anti-phishing.cedar` | evaluate sender authentication, lookalike-domain risk, URL risk, and release authority |
| `microservices/mail/policy/abuse-defence.cedar` | keep bot/reputation denial separate from phishing content verdicts |
| `microservices/mail/contracts/proto/mail.proto` | use `Dkim`, `Spf`, `Dmarc`, `Arc`, and `AbuseVerdict` as the transport facts |
| `microservices/mail/dashboards/abuse-defence-outcomes.json` | show clean/quarantine/reject counts, false-positive release rate, and provider degradation |
| `microservices/mail/runbooks/account-compromise-recovery.md` | include phishing-triggered account recovery and DKIM/key rotation handoff |

## D. Implementation

1. Add `PhishingAssessment { message_digest, dkim, spf, dmarc, arc, sender_domain, display_name_similarity, url_risk[], attachment_risk[], phishing_score }` to the anti-phishing kernel plan.
2. Wire `oya-mail-inbound-smtp-adapter-smtp` to compute RFC authentication facts before message persistence; never persist unauthenticated body content before policy outcome.
3. Add URL reputation enrichment with an OpenBao-backed credential reference and a hard 500ms enrichment budget; provider timeout falls back to local heuristics plus degraded-mode audit evidence.
4. Update `policy/anti-phishing.cedar` to classify high scores as reject, middle scores as quarantine, known-trusted domain mismatches as review, and explicit `MailQuarantineRelease` as a separate action.
5. Emit ADR-0263 events for block, quarantine, release, enrichment-degraded, and false-positive-release.
6. Bind dashboard panels to false-positive rate, p99 enrichment latency, per-tenant quarantine volume, and DMARC alignment failures.
7. Add synthetic corpora: legitimate transactional mail, lookalike-domain phishing, credential-harvest URL, ARC-forwarded legitimate mail, and URL-provider outage.
8. Add runbook steps for account compromise recovery when outbound phishing is detected from a tenant sender.

## E. Acceptance

- `policy/anti-phishing.cedar` can be evaluated from facts already present in the inbound SMTP path and `contracts/proto/mail.proto`; no invented entity names are required.
- Messages with hard DMARC fail plus high lookalike score reject with SMTP 550 5.7.x and emit `oya.mail.anti-phishing-block`.
- Borderline messages quarantine with a user-visible queue and a Cedar-gated release path.
- URL reputation outage produces degraded-mode audit evidence and does not create a blanket allow.
- False-positive rate target is <=0.5% on the synthetic legitimate-mail corpus; clean mail adds no extra user action.

## F. Evidence

- `microservices/mail/ARCHITECTURE.md` cold-start route names DKIM/SPF/DMARC/ARC, abuse defence, anti-phishing, and delivery events.
- `microservices/mail/contracts/proto/mail.proto` already defines auth-result and `AbuseVerdict` enums.
- `microservices/mail/competitor-parity-matrix.md` compares anti-phishing, quarantine, tracker blocking, and DKIM/SPF/DMARC coverage across Gmail, Exchange, Proton, Fastmail, Naver, and Stalwart.
- ADR-0273 and ADR-0297 provide the mail-authentication and abuse-edge doctrine.

## G. Counterparts

| Counterpart | Gap closed by this IP |
|---|---|
| Gmail | Narrows phishing and malware filtering parity while adding tenant-visible audit evidence that Gmail keeps opaque. |
| Microsoft Exchange Online | Matches enterprise quarantine expectations but keeps decisions Cedar-replayable and tied to Oyatie audit-chain events. |
| Proton / Fastmail | Preserves privacy-first and standards-first expectations while adding work-tenant release governance. |

## H. Non-goals and handoff boundaries

- Do not train a new phishing model in this IP; the IP defines the scoring contract and edge wiring.
- Do not hard-reject borderline messages when quarantine can preserve deliverability and user review.
- Do not store provider API keys in chart values; enrichment credentials use OpenBao secret references only.
- Do not collapse spam, phishing, malware, and DLP into one verdict; `AbuseVerdict` keeps these outcomes reviewable.
- Do not apply tenant-wide friction to clean mail; the PRD's UX floor requires zero friction on clean default paths.

## I. Fixture set

- `dmarc_fail_lookalike_reject.eml` proves SMTP 550 path.
- `arc_forwarded_legitimate_quarantine.eml` proves ARC-aware borderline handling.
- `trusted_domain_clean.eml` proves no-friction delivery.
- `url_provider_timeout.eml` proves degraded-mode evidence and fallback heuristics.
- `quarantine_release_wrong_role.json` proves Cedar release denial.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-017-anti-phishing-edge-wiring.md` matched `.proto`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-017-anti-phishing-edge-wiring.md` matched `p99`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
