---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: slides
status: Accepted
methodology: STRIDE + LINDDUN
date: 2026-05-17
owner_team: axis-workspace + ops-security
related_artifacts:
  - microservices/slides/PRD.md
  - microservices/slides/dpia.md
  - microservices/slides/compliance.md
  - microservices/slides/policy/tenant-scope.cedar
  - microservices/slides/runbooks/broadcast-mode-degraded.md
  - microservices/slides/runbooks/export-pipeline-failure-pptx.md
doc_status: published
---

# Threat model — slides µservice

## Scope

In-scope: every slides BC + its REST + WS surface; collab CRDT layer; broadcast-mode LiveKit signaling; import/export gVisor sandbox; AI-design + AI-content-generation foundry-runtime bridge; per-slide Cedar ACL; per-pack residency overlay.

Out-of-scope: messenger LiveKit cluster operation (covered by messenger threat-model); foundry-runtime LLM (covered by foundry threat-model); sheets data plane (consumed via SDK; threat-modeled in sheets); CDN provider operation (CSP threat covered here, provider compromise covered by cloud-iac).

## Trust boundaries

1. **Browser ↔ slides-rest** — OIDC tenant boundary; WASM bundle SRI; CSP enforced.
2. **Browser ↔ real-time-collaboration-worker (WS)** — OIDC token rebound per WS message; per-tenant session isolation.
3. **slides-rest ↔ Postgres** — RLS per tenant_id; Cedar enforcement at usecase layer.
4. **slides-rest ↔ Valkey** — per-cell cluster; cell-local CRDT state; cross-tenant key namespace.
5. **slides-rest ↔ S3** — per-tenant prefix; SSE-KMS per-pack key.
6. **slides ↔ sheets (chart-live-link)** — SDK boundary; sheets ACL is authority; cross-µservice direct DB read forbidden by LEAN-A2.
7. **slides ↔ messenger (broadcast-mode)** — SDK boundary; LiveKit room created by messenger; slides holds the lease, never the LiveKit credentials.
8. **slides ↔ foundry-runtime (AI)** — SDK boundary; prompt + completion sealed by audit-chain; T2 risk-class evaluated by foundry-runtime, stamped by slides.
9. **Import workers ↔ untrusted PPTX/Keynote/ODP file** — gVisor sandbox; ClamAV + OPSWAT scan pre-parse.
10. **Export workers ↔ deck content** — read-only fetch + gVisor sandbox for WeasyPrint/Chromium-headless/ffmpeg.

## STRIDE matrix

### S — Spoofing

| ID | Threat | Surface | Mitigation | Verification |
|---|---|---|---|---|
| T-S-01 | Attacker submits CRDT op claiming another user's `author_oidc_sub` | WS dispatch | server rebinds `author_oidc_sub` from validated WS-upgrade OIDC at every op; client-supplied claim discarded | property test `tests/security/op_author_rebind.rs` |
| T-S-02 | Attacker spoofs broadcast presenter identity | broadcast-mode worker | LiveKit room ACL bound to OIDC sub + Cedar evaluation; messenger-issued one-time signed token | unit `oya-slides-broadcast-mode-domain::test_presenter_token_bound` |
| T-S-03 | Attacker forges chart-live-link to a sheet they cannot read | chart bind action | server re-verifies sheets-side read grant on bind; periodic re-verification on refresh | integration `tests/integration/chart_acl_reverify.rs` |
| T-S-04 | Attacker submits theme/template with spoofed signature | themes/templates upload | Ed25519 signature verified at upload; tampered assets refused; revocation propagation ≤ 60s | unit `oya-slides-themes-domain::test_signature_verify` |

### T — Tampering

| ID | Threat | Surface | Mitigation | Verification |
|---|---|---|---|---|
| T-T-01 | Attacker tampers Loro CRDT op payload in transit | WS frame | HMAC-SHA-256 over op envelope using per-session key; mismatched HMAC → drop + Sev-1 alarm | property test `tests/security/op_hmac.rs` |
| T-T-02 | Attacker tampers WASM bundle on CDN | WASM load | SHA-384 SRI hash in HTML; mismatch refuses load + audit row + Sev-1 alarm | AC-12 + ADR-SLIDES-0002 |
| T-T-03 | Attacker tampers PPTX during upload (malformed OOXML to crash parser) | import-export worker | gVisor sandbox isolates parser; ClamAV + OPSWAT pre-parse; parser memory + CPU budget | runbook `runbooks/export-pipeline-failure-pptx.md` + unit `oya-slides-import-export-domain::test_pptx_malformed_isolated` |
| T-T-04 | Attacker tampers MP4 export ffmpeg input | export worker | deterministic ffmpeg flags; gVisor sandbox; output sha256 logged | runbook + property test |
| T-T-05 | Attacker tampers per-pack theme/template signed bundle on CDN | themes/templates load | Ed25519 verification at load; revocation propagation; CRL polling | unit `oya-slides-themes-domain::test_revocation` |

### R — Repudiation

| ID | Threat | Surface | Mitigation | Verification |
|---|---|---|---|---|
| T-R-01 | Author denies authoring a deck edit | save | every save emits `slides_deck_saved` Ed25519 seal with `(tenant_id, deck_id, version_sha, author_identity, parent_version_sha, timestamp)` | audit-chain verify routine |
| T-R-02 | Presenter denies running a broadcast | broadcast-mode | `BroadcastStarted` + `BroadcastEnded` Ed25519 seal with attendee count + duration; LiveKit signaling logs cross-verified | audit-chain + messenger logs |
| T-R-03 | Attacker tampers ACL change attribution | acl | `slides_acl_changed` Ed25519 seal with `(tenant_id, deck_id, slide_id, principal, change_type, before_acl_sha, after_acl_sha)` | audit-chain |
| T-R-04 | T2 AI-content-generation denies risk-class | ai-content-generation | every T2 invocation emits `AiContentGenerated` audit row with `risk_class`, `prompt_hash`, `completion_hash`, `pack` | audit-chain + ADR-SLIDES-0006 |

### I — Information Disclosure

| ID | Threat | Surface | Mitigation | Verification |
|---|---|---|---|---|
| T-I-01 | Cross-tenant deck content leakage via Postgres | slides-rest | RLS on tenant_id; Cedar evaluation at usecase; LEAN-A2 cross-product refusal | unit `oya-slides-presentation-adapter-postgres::test_rls` |
| T-I-02 | Cross-tenant CDN cache pollution | CDN | per-tenant CDN cache keys; `(tenant_hash, pack, version)` partitioning; verified at edge | edge config test |
| T-I-03 | Cross-tenant Valkey CRDT state leakage | real-time-collaboration | per-cell cluster; cell-local CRDT state; tenant_id prefix in Valkey key; pod-level isolation; HMAC binding | integration `tests/security/valkey_isolation.rs` |
| T-I-04 | Cross-tenant S3 asset leakage | image / video-embed | per-tenant prefix; SSE-KMS per-pack key; tenant_id in IAM condition; cross-tenant read fail-closed | unit + integration |
| T-I-05 | XSS via rich-text or embed-bridge | text-box / embed-bridge | virtual-DOM text nodes; sanitization at embed-bridge boundary; CSP strict | unit `tests/security/xss.rs` |
| T-I-06 | Chart-live-link reads beyond sheet-side ACL | chart | sheets SDK enforces ACL; cell-range bound at bind-time; revocation cascade | ADR-SLIDES-0008 + integration |
| T-I-07 | Speaker-notes leak to audience-view during broadcast | broadcast-mode | speaker-notes scope = presenter-view only; broadcast frame stream excludes notes layer | unit `oya-slides-broadcast-mode-domain::test_notes_excluded_from_broadcast` |
| T-I-08 | PHI in slide content leaks to T2 AI-content-generation | ai-content-generation | per-pack `phi_redaction_required` flag (us-healthcare); pre-flight redaction; refusal if PHI detected without redaction consent | ADR-SLIDES-0006 + DPIA |
| T-I-09 | Comment thread visible to revoked viewer | comments | per-comment Cedar evaluation at fetch; revocation invalidates cache | integration |
| T-I-10 | Version-history reveals deleted slide content beyond retention | version-history | per-pack retention enforced; cryptographic delete on retention expiry; un-restorable past retention | unit |

### D — Denial of Service

| ID | Threat | Surface | Mitigation | Verification |
|---|---|---|---|---|
| T-D-01 | Per-tenant session-storm (open many editor sessions) | slides-rest | per-tenant session cap (50 default; configurable per-pack); 429 + audit beyond cap | helm value `perTenantSessionCap` |
| T-D-02 | CRDT op-flood from compromised client | WS gateway | per-session op rate limit (100 ops/sec sustained, 1k burst); HMAC-bound; offender disconnect + audit | property + integration |
| T-D-03 | Broadcast-mode LiveKit cluster overload (single deck) | broadcast-mode | per-deck viewer cap (5000 default); LiveKit SFU cascade; admission throttle | runbook `broadcast-mode-degraded.md` |
| T-D-04 | PPTX/PDF/MP4 export-flood | import-export worker | job-queue depth limit per-tenant; per-pack daily export quota; gVisor worker pool finite | helm value `exportQuotaPerPackPerDay` |
| T-D-05 | AI-content-generation T2 flood | ai-content-generation | per-tenant T2 rate limit (10/hour default); per-pack daily T2 quota; foundry-runtime backpressure | foundry-runtime SDK |
| T-D-06 | Chart-live-link refresh flood | chart | per-chart refresh debounce (1 refresh / 2s baseline); per-deck refresh budget | unit |
| T-D-07 | Cedar evaluator crash | acl | fail-closed (helm `cedar.failClosed: true`); fallback denies; 503 + Sev-1 alarm; never fail-open | helm + property test |
| T-D-08 | gVisor sandbox out-of-memory (malicious PPTX) | import-export worker | per-job memory budget (2 GiB import; 4 GiB MP4 export); OOM-kill + audit; tenant op-rate cooldown after 3 OOMs | helm + observability alert |

### E — Elevation of Privilege

| ID | Threat | Surface | Mitigation | Verification |
|---|---|---|---|---|
| T-E-01 | WS message with client-supplied tenant_id mid-stream | real-time-collaboration | server rebinds tenant_id from WS-upgrade OIDC; client-supplied tenant_id discarded | property test |
| T-E-02 | Per-slide ACL bypass via deck-level grant | acl | deck-level grant is necessary but not sufficient; per-slide ACL evaluated additionally per ADR-SLIDES-0007 | unit `oya-slides-acl-domain::test_per_slide_overrides_deck` |
| T-E-03 | Broadcast presenter elevates to room admin | broadcast-mode | LiveKit room ACL distinguishes presenter (publish) vs admin (terminate); messenger SDK issues presenter-scoped tokens only | integration |
| T-E-04 | Sandbox escape from gVisor in PPTX import | import-export worker | gVisor + seccomp; user namespace; no host network; no host filesystem mount beyond input/output dirs; gVisor advisory feed monitored | runbook + supply-chain advisory subscription |
| T-E-05 | T2 AI-content-generation bypasses risk-class | ai-content-generation | foundry-runtime is the risk-class authority; slides forwards verdict; slides cannot stamp risk-class itself | ADR-SLIDES-0006 + unit |

## LINDDUN (privacy)

| ID | Privacy threat | Mitigation | Compliance reference |
|---|---|---|---|
| P-L-01 | Linkability — author identity across decks via CRDT op timestamps | OIDC sub used as identity; per-deck pseudonym available (pack default in us-healthcare) | GDPR Art. 5(1)(c) data minimization |
| P-L-02 | Identifiability — sensitive content in alt-text suggestions exposed to AI provider | T1 alt-text via foundry-runtime; per-pack `phi_redaction_required`; consent gate in us-healthcare | HIPAA + GDPR Art. 9 |
| P-L-03 | Non-repudiation harm — audit seal reveals participation in confidential deck | per-pack audit seal retention; cryptographic delete on retention expiry | GDPR Art. 17 |
| P-L-04 | Detectability — broadcast-mode attendee count leaks viewer presence | attendee count is aggregate-only; per-viewer identities not stored beyond 24h unless tenant opts in | DPIA §"Broadcast attendee data" |
| P-L-05 | Disclosure of information — chart-live-link from sheet with stale ACL leaks values | revocation cascade ≤ 5s; ADR-SLIDES-0008 | GDPR Art. 32 |
| P-L-06 | Unawareness — tenant unaware deck content used for T2 training | T2 invocations never used for training without explicit opt-in; foundry-runtime contract | EU AI Act + GDPR Art. 13 |
| P-L-07 | Non-compliance — cross-pack data residency violation | overlay enforces per-pack region pin; cross-pack collab forbidden | GDPR Art. 44 + per-jurisdiction laws |

## Attack trees (selected)

### A-T-1 — Silent CRDT loss

Goal: cause a tenant edit to be silently dropped during merge.

```
Goal: Silent CRDT loss (T-T-01, AC-06)
├── 1. Tamper op in transit
│   ├── Strip HMAC (mitigated by HMAC + Sev-1 alarm)
│   └── Replay stale op (mitigated by sequence_num monotonicity + dedup)
├── 2. Exploit merge-algebra bug in Loro
│   ├── Find non-commutative merge edge case
│   │   └── (mitigated by proptest + property `no_silent_overwrite`)
│   └── Library-level vulnerability
│       └── (mitigated by Loro version pin + advisory feed; supersession path in ADR-SLIDES-0001)
└── 3. Bypass conflict surfacer
    └── Trigger code-path that calls .last_writer_wins() (mitigated: no such code-path exists; grep-gate refuses introduction)
```

### A-T-2 — Per-slide ACL bypass

Goal: read or edit a slide the user is not granted.

```
Goal: Per-slide ACL bypass (T-E-02, AC-08)
├── 1. Use deck-level grant only (mitigated: deck grant is necessary but not sufficient)
├── 2. Submit CRDT op targeting a non-granted slide_id (mitigated: usecase rebind + Cedar evaluation)
├── 3. Bypass via embed-bridge (mitigated: embed-bridge enforces target-side ACL)
└── 4. Bypass via version-history restore (mitigated: restore replays Cedar evaluation per slide)
```

## Supply-chain threats

| ID | Threat | Mitigation |
|---|---|---|
| T-SC-01 | Compromised Loro 1.x release | version pin + GitHub Security Advisories + RustSec subscription; cargo deny allowlist |
| T-SC-02 | Compromised Leptos 0.7+ release | version pin + advisory feed |
| T-SC-03 | Compromised ffmpeg 7.x | LTS pin + advisory feed; SBOM diff at upgrade; SLSA L3 build |
| T-SC-04 | Compromised WeasyPrint or Chromium-headless | version pin + advisory feed |
| T-SC-05 | Compromised Pandoc 3.x | version pin + advisory feed |
| T-SC-06 | Compromised ImageMagick 7.1 | LTS pin + advisory feed; gVisor sandbox limits blast radius |
| T-SC-07 | Compromised ClamAV / OPSWAT signature database | dual-scanner topology; OPSWAT verdict required if ClamAV verdict missing |
| T-SC-08 | Compromised cargo-leptos toolchain | vendored fallback (raw `cargo` + `wasm-bindgen-cli`) |
| T-SC-09 | Per-pack theme/template signed bundle key compromise | revocation list distributed via CDN; key rotation procedure documented |
| T-SC-10 | LiveKit 1.6.2 LTS compromise | inherited from messenger threat-model + advisory feed |

## Compliance crosswalk

| Standard | Section | How addressed |
|---|---|---|
| ISO 27001:2022 | A.5.7 (threat intelligence), A.8.7 (malware), A.8.16 (monitoring) | RustSec + GitHub Security Advisories + ClamAV + OPSWAT + Sev-1 alarms |
| OWASP ASVS v4 | V1.5 (architecture), V4 (access), V5 (validation), V14 (config), V13 (API) | Cedar v4 default-deny; per-slide ACL; strict CSP; OIDC-bound WS |
| SOC 2 Type 2 | CC7 (system operations) | Sev-1 alarms + audit-chain Ed25519 seal end-to-end |
| NIST SSDF | PO.5, PS.1, PW.5 | SBOM diff + version pin + advisory feeds + cargo-deny |
| SLSA L3 | provenance + isolation | gVisor sandbox + SLSA-L3 build |
| EU AI Act | Art. 16 + Annex III | ADR-SLIDES-0006 risk-class stamp; ai-act-risk-class-stamp CI lane |
| GDPR | Art. 32 | encryption-in-transit (TLS 1.3 + WSS), encryption-at-rest (SSE-KMS) |
| HIPAA | §164.312 | us-healthcare pack additional controls (PHI redaction, retention) |
