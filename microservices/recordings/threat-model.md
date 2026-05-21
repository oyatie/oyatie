---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-recordings + ops-security
deciders: council-architecture, ops-security, axis-recordings, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154 + NIST SP 800-86 (forensic-integrity)
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145), ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005, ADR-RECORDINGS-0006, ADR-RECORDINGS-0007]
related_specs: [/specs/microservices/recordings.json]
review_cadence: quarterly + on every architecture or substrate change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44-50"
  - "EU AI Act Art. 50 (transparency) + Annex III (high-risk if employment/legal-context)"
  - "ePrivacy Directive 2002/58 Art. 5(3)"
  - "ISO 27037:2012 — guidelines for identification, collection, acquisition, preservation of digital evidence"
  - "NIST SP 800-86 — guide to integrating forensic techniques into incident response"
  - "OWASP ASVS v4.0.3 — application security verification standard"
  - "SLSA L3 — supply-chain levels for software artefacts"
  - "NIST SP 800-218 (SSDF) — secure software development framework"
  - "CIS Kubernetes Benchmark v1.9"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/22-2/23/28/29", "KR-ISMS-P §2.1-2.12", "KR 전자문서법 Art. 5 (electronic-doc retention)", "KR 통신비밀보호법 (recording-consent + intercept restrictions)"]
  pack-us-healthcare: ["HIPAA 45 CFR §§164.308/312/316/502/514", "HITECH Act breach-notification"]
  pack-us-financial: ["SEC Rule 17a-4(f) — WORM-class for recorded communications", "FINRA Rule 4511 (book + record retention)", "MiFID II Art. 16(7) — recording-of-communications retention (5y + on-request)", "CFTC Rule 1.31"]
  pack-eu: ["GDPR Arts. 5/9/22/25/30/32/35/44-50", "ePrivacy Art. 5(3)", "EU AI Act (transcription + summary + auto-publish — Annex III/transparency)", "NIS2 2022/2555 (when thresholds engaged)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "TIA Act + Surveillance Devices Act (intercept + recording-consent)"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: recordings µservice

## Purpose

Identify, classify, and mitigate threats to the recordings µservice's
confidentiality, integrity, availability, privacy, and **forensic-integrity**
posture. The recordings µservice is the centralised audit-grade store for
every audio/video recording across oyatie; a compromise leaks meeting
content, identity graph, eDiscovery evidence chains, and (for pack-us-
healthcare / pack-us-financial) PHI / SEC-regulated communications. This
document is reviewed by SOC 2 examiners, ISO 27001 auditors, GDPR DPAs, KR
PIPC, HIPAA OCR, SEC examiners, FINRA, EU AI Act notified bodies, and ISO
27037 digital-evidence reviewers at first-tenant onboarding per pack.

## Scope

### In-scope

All components introduced by ADR-0135 (Connect dual-context inherited) and
ADR-0132 (suite dissolution into recordings surface). Deployed in the
dedicated recordings Kubernetes namespace.

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 (recording metadata + transcript JSON + redaction overlay + retention + legal-hold + ediscovery) | `oya-recordings-recording-*` (10 crates) |
| Valkey 8.1 (RESP3 wire-compatible) (share-link signed-URL cache + playback session) | `oya-recordings-media-segment-*` (8 crates) |
| S3-compatible (media — hot tier) + S3-Glacier-class (cold tier) | `oya-recordings-transcript-*` (10 crates) |
| CloudFront (primary) / Bunny + Fastly + nginx-vod (self-host pack-cn / pack-ksa) | `oya-recordings-redaction-*` (9 crates) |
| Meilisearch 0.10.0 LTS (search index) | `oya-recordings-retention-policy-*` (9 crates) |
| Whisper-large via foundry-runtime gVisor (transcription) | `oya-recordings-legal-hold-*` (9 crates) |
| pyannote 3.x via foundry-runtime gVisor (speaker diarization) | `oya-recordings-export-*` (8 crates) |
| ffmpeg 7.x in gVisor sandbox (transcode + thumbnail + loudness norm) | `oya-recordings-share-link-*` (9 crates) |
| Pandoc 3.x (transcript-to-PDF/DOCX) | `oya-recordings-playback-*` (8 crates) |
| OPSWAT MetaDefender / ClamAV (upload scan) | `oya-recordings-ediscovery-*` (8 crates) |
| Cedar v4.2 policy evaluator | `oya-recordings-watermarking-*` (7 crates) |
| OpenBao (signed-URL HMAC secret + tenant DEK) | `oya-recordings-recording-ingest-*` (9 crates) + 10 other BC families |

### Out-of-scope

- Threats to the underlying Kubernetes / hyperscaler — owned by `cloud-k8s`.
- Threats to OpenBao — owned by `cloud-secrets`.
- Threats to audit-chain µservice — owned by its own threat model; inherited.
- Threats to Ontology — owned by `ontology` µservice; inherited.
- Threats to foundry-runtime — owned by its own threat model; inherited.
- Threats to the producing µservices (meet / messenger / live-broadcast) —
  owned by their own threat models; inherited via the ingest contract.
- Threats to GitHub Actions — owned by `governance`.

## Data Inventory

| Asset | Sensitivity | Pack scope |
|---|---|---|
| Source media (audio/video) | PII_IDENTIFYING (always); PHI (pack-us-healthcare); regulated-comm (pack-us-financial) | every pack |
| Transcript JSON | PII_IDENTIFYING + same overlay as media | every pack |
| Redaction overlay | INTERNAL_ONLY (metadata only — no media) | every pack |
| Retention policy + KMS-shred-key-ref | AUDIT | every pack |
| Legal-hold engagement + chain-of-custody event | AUDIT (load-bearing) | every pack |
| Share-link HMAC secret | INTERNAL_ONLY (resolved per-request from OpenBao `${openbao:secret/recordings/<tenant>/share-link-hmac}`) | every pack |
| Per-viewer watermark key | INTERNAL_ONLY | every pack |
| eDiscovery export bundle (signed manifest + Merkle root) | AUDIT + the bundle carries PII | every pack |
| Court-order reference + engagement letter | AUDIT + PII | every pack |

## Threat Catalog (STRIDE × LINDDUN)

### Spoofing

| ID | Threat | Vector | Mitigation | Residual |
|---|---|---|---|---|
| T-S-01 | Producer µservice (meet/messenger/live-broadcast) spoofed during ingest | adversary impersonates a producer SPIFFE identity | SPIFFE mTLS on ingest endpoint; producer-allowlist per `policy/cedar/ingest-allowlist.cedar`; per-request audit-chain entry; Cedar default-deny | low — mTLS + audit |
| T-S-02 | Tenant-admin impersonation on legal-hold engagement | session-fixation; stolen cookie | mTLS + WebAuthn step-up for legal-hold; four-eyes pair approval per ADR-RECORDINGS-0002 | low |
| T-S-03 | Share-link forged by attacker | HMAC secret leak | OpenBao 30d rotation; per-tenant HMAC; revocation list checked on every playback | low |

### Tampering

| ID | Threat | Vector | Mitigation | Residual |
|---|---|---|---|---|
| T-T-01 | Source media silently mutated after ingest | adversary writes to S3 directly | object-lock (WORM) for pack-us-financial + pack-us-healthcare; content-hash recorded at ingest; periodic audit re-verify hash | low |
| T-T-02 | Transcript JSON silently rewritten | adversary writes Postgres directly | row-level append-only audit; content_hash verified against audit-chain seal; redaction overlay model means transcript writes are immutable post-finalisation (only overlay grows) | low |
| T-T-03 | Redaction overlay coordinates tampered to un-redact | adversary edits overlay row | overlay rows are insert-only with audit-chain seal on every insert; un-redact requires a new compensating overlay row with reason + four-eyes | low |
| T-T-04 | Retention purge bypasses legal hold | race between purge worker and hold engagement | retention worker uses pessimistic read-lock on `legal_hold_table`; legal-hold engagement Sev-1 if any purge executes against a held row; **load-bearing 100 % invariant** | very low — load-bearing SLO |
| T-T-05 | eDiscovery export Merkle root forged | adversary swaps a manifest before counsel download | manifest signed by export-worker SPIFFE identity (Ed25519); root committed to audit-chain at export time; counsel verifies signature against published public key per ISO 27037:2012 §5.4 | low |

### Repudiation

| ID | Threat | Vector | Mitigation | Residual |
|---|---|---|---|---|
| T-R-01 | Tenant denies engaging legal hold | no audit trail | every hold engagement emits `LegalHoldEngaged` event with engaging-principal + paired-approver + reason; audit-chain seal | very low |
| T-R-02 | Counsel denies receiving export bundle | no download proof | signed-URL download emits audit-chain event with counsel IP + UA + signed-URL ID; bundle Merkle root in audit-chain | very low |
| T-R-03 | Tenant denies a recording was published | no published-event audit | every publish emits `RecordingPublished` with tenant-id + recording-id + content_hash | very low |

### Information disclosure

| ID | Threat | Vector | Mitigation | Residual |
|---|---|---|---|---|
| T-I-01 | Cross-tenant recording read via Cedar policy bypass | malformed query parameters | Cedar policy default-deny + per-resource `tenant_id` predicate + Cedar evaluator runs server-side; LEAN-A2 lane forbids client-trust patterns | low |
| T-I-02 | Cross-pack media replication | misconfigured S3 cross-region replication | replication rules per-pack-only; periodic S3 audit; `oya-check-recordings-pack-residency` lane | low |
| T-I-03 | Transcript leaks via search-index residual data | redaction overlay applied but search index not re-emitted | search-index re-emit triggered on every redaction insert; periodic search-index reconciliation worker | low |
| T-I-04 | Playback leak via screen capture | attacker records the playback screen | per-viewer dynamic visible + steganographic watermark per ADR-RECORDINGS-0004; identifies leaker post-hoc | accept — visible watermark deters; steganographic enables post-hoc attribution |
| T-I-05 | Transcript model (Whisper) leaks training data | residual training data in model outputs | only the open-weights Whisper-large model is used; foundry-runtime gVisor sandbox isolates; no cross-tenant model fine-tuning | low |
| T-I-06 | Share-link guessable | weak HMAC seed | per-tenant 256-bit HMAC seed from OpenBao; share-link IDs are 128-bit UUID-v4; signed-URL TTL 24h default | very low |

### Denial of service

| ID | Threat | Vector | Mitigation | Residual |
|---|---|---|---|---|
| T-D-01 | Transcription queue flooded | adversary uploads many empty recordings | per-tenant ingest rate-limit; ingest-side OPSWAT scan filters obviously-bogus uploads; foundry-runtime queue with priority lane | low |
| T-D-02 | Playback CDN cache cascade failure | cold cache; high QPS | multi-tier CDN; per-pack origin shielding; `runbooks/playback-cdn-cache-cascade.md` | medium — see runbook |
| T-D-03 | Export bomb (a tenant requests 10k exports in parallel) | adversary scripts exports | per-tenant export queue depth cap; export worker concurrency cap | low |
| T-D-04 | Search index DoS via heavy query | adversary issues regex-of-doom | Meilisearch query timeout + per-tenant search QPS cap | low |

### Elevation of privilege

| ID | Threat | Vector | Mitigation | Residual |
|---|---|---|---|---|
| T-E-01 | Tenant-admin → other-tenant data via Cedar bypass | malformed `tenant_id` query | Cedar default-deny + tenant_id-derived-from-session (not request) | very low |
| T-E-02 | End-user → tenant-admin via privilege escalation | role-table direct write | tenancy µservice owns roles; recordings reads only; LEAN-A2 lane enforces no role-table writes | low |
| T-E-03 | ffmpeg sandbox escape | malicious media file triggers ffmpeg CVE | ffmpeg 7.x in gVisor sandbox; quarterly upstream-CVE lane; pinned per ADR-RECORDINGS-0004 | low — gVisor + LTS pin |
| T-E-04 | Pandoc sandbox escape | malicious transcript payload triggers Pandoc CVE | Pandoc 3.x in gVisor sandbox; pinned; quarterly CVE review | low |

### LINDDUN privacy threats

| ID | Threat | Mitigation |
|---|---|---|
| T-L-01 (Linkability) | Cross-recording linkage via diarization | speaker clusters per-recording by default; cross-recording linkage gated by tenant-admin opt-in + Cedar policy |
| T-L-02 (Identifiability) | Voice-print used to identify speaker without consent | diarization emits cluster labels only; speaker-naming requires explicit user binding |
| T-L-03 (Non-repudiation) | Transcript publish forces user repudiation cost | per ePrivacy Art. 5(3): user opt-in mandatory; recording-consent banner on producer (meet/messenger) |
| T-L-04 (Detectability) | Hidden recording-going-on | producing µservice (meet/messenger) emits in-session indicator; recordings µservice refuses ingest without consent-banner-confirmed flag |
| T-L-05 (Disclosure of info) | Cf. T-I-* above |  |
| T-L-06 (Unawareness) | User unaware their voice was diarised | tenant-side consent banner + post-recording disclosure of transcription / diarization |
| T-L-07 (Non-compliance) | Pack regulatory mismatch (e.g., MiFID II 5y retention violated) | per-pack retention defaults enforced; CI lane refuses tenant overrides outside pack ceiling |

## Per-Pack Regulatory Threats

### pack-us-financial (SEC Rule 17a-4(f) + FINRA 4511 + MiFID II 16(7))

- Recordings of business-communications retained ≥ 36 months in non-rewriteable
  / non-erasable form per SEC 17a-4(f). Pack-overlay enforces S3 object-lock +
  legal-hold-default-on for new tenants in the pack until tenant-admin opts out.
- WORM-attestation included in eDiscovery export bundle.

### pack-us-healthcare (HIPAA 45 CFR §164.308-316)

- Recording bodies decrypted ONLY if counsel has BAA on file.
- Per-recording access audit + four-eyes for clinical-recording disclosure.
- Auto-PII redaction at transcription time required (PHI-aware Whisper post-
  processing).

### pack-eu (GDPR + ePrivacy)

- Recording-consent banner required per ePrivacy Art. 5(3).
- DSR cascade: right-to-erasure → recording purge + transcript purge +
  redaction overlay erasure + audit-chain notes the erasure event.
- Cross-border transfer forbidden by default (Arts. 44-50).
- EU AI Act Art. 50 transparency: every transcription / summary / auto-
  translate output is labelled `ai-generated`; evidence_topic record per
  Art. 13.

### pack-kr (KR PIPA + 전자문서법 + 통신비밀보호법)

- 통신비밀보호법 forbids non-consensual recording of communications; ingest
  refuses recordings without `consent_banner_confirmed: true`.
- 전자문서법 Art. 5 — electronic-document retention with integrity attestation;
  audit-chain Merkle seal satisfies.

### pack-au (Privacy Act + TIA Act + Surveillance Devices Act)

- TIA Act + state Surveillance Devices Acts require explicit consent for
  voice recording; ingest refuses without per-jurisdiction consent flag.

## Verification (CI-enforced)

- `oya gate validate retention-policy-correctness --microservice recordings` —
  100 % invariant; any breach Sev-1.
- `oya gate validate legal-hold-chain-of-custody-correctness --microservice recordings` —
  100 % invariant; any breach Sev-1.
- `oya gate validate authority-cohesion --microservice recordings` (HG-RECORDINGS).
- `oya gate validate lean-a*` lanes.
- `oya gate validate version-pinning-conformance` — Whisper, pyannote, ffmpeg,
  Pandoc, Postgres, Valkey, Meilisearch, Cedar pins.
- `oya gate validate cve-freshness --microservice recordings` — quarterly.

## Residual Risk

After mitigations, the residual risk is **low for confidentiality + integrity
+ legal-hold-correctness + retention-correctness** and **medium for playback
cache-cascade availability** (see `runbooks/playback-cdn-cache-cascade.md`).

## References

- STRIDE (Microsoft Threat Modeling).
- LINDDUN (KU Leuven).
- OWASP Top 10 (2021), OWASP API Top 10 (2023).
- NIST SP 800-154 (Data-centric threat modeling).
- NIST SP 800-86 (forensic-integrity).
- ISO 27037:2012 (digital evidence handling).
- SOC 2 / ISO 27001:2022 / GDPR / HIPAA / SEC 17a-4 / FINRA 4511 / MiFID II /
  KR PIPA / KR 전자문서법 / KR 통신비밀보호법 / APPI / PDPA SG-AU / DPDPA / LGPD /
  UAE PDPL / KSA PDPL.
- ADR-RECORDINGS-0001..0007.
- Bominal ADR-0028 (audit-chain Merkle + Ed25519).
- Bominal ADR-0111 (ciphertext property type).
