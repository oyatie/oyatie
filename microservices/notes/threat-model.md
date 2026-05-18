---
doc_class: ThreatModel
title: notes µservice STRIDE threat model
microservice: notes
status: Accepted
classification: CONFIDENTIAL
date: 2026-05-17
owner_team: ops-security + axis-notes + council-privacy
review_cadence: quarterly + on every BC change
related_adrs: [ADR-0008, ADR-0028, ADR-0135, ADR-0131, ADR-NOTES-0001, ADR-NOTES-0004, ADR-NOTES-0005]
related_artifacts:
  - microservices/notes/PRD.md
  - microservices/notes/policy/dual-context-isolation.md
  - microservices/notes/policy/e2e-personal-tier-default.md
  - microservices/notes/dpia.md
references:
  - OWASP ASVS v4
  - NIST SP 800-53 Rev. 5
  - RFC 9420 (MLS)
  - Standard Notes Threat Model whitepaper
  - Obsidian End-to-End Encryption Sync threat model
doc_status: published
---

# Threat Model — notes µservice

## Methodology

STRIDE (Spoofing / Tampering / Repudiation / Information disclosure / Denial of service / Elevation of privilege) per-asset + adversary capability matrix; followed by mitigation mapping to controls per OWASP ASVS v4 and NIST SP 800-53 Rev. 5. Asset enumeration scoped per ADR-0131 per-microservice flat layout. Adversary capability set inherited from Bominal threat-model template + sharpened for E2E-default Personal-pillar posture (cf. Standard Notes whitepaper + Apple Notes Lockable threat model + Obsidian E2E Sync model).

## Assets

| Asset ID | Asset | Classification | Owner BC |
|---|---|---|---|
| A-01 | Personal-tier note plaintext | E2E-protected; oyatie holds ciphertext only | note-store |
| A-02 | Personal-tier MLS group keys (client-derived) | client-held only; oyatie distributes public KeyPackages | e2e-key-management |
| A-03 | Personal-tier recovery seed | client-held; user-printed paper backup | e2e-key-management |
| A-04 | Professional-tier note plaintext | tenant-DEK envelope; four-eyes-disclosable | note-store |
| A-05 | Tag adjacency + tag-graph | BEHAVIORAL_TENANT_PRODUCT | tag-graph |
| A-06 | Backlink adjacency | BEHAVIORAL_TENANT_PRODUCT | backlink-graph |
| A-07 | Share-link tokens | secret material (URL-safe random) | share-link |
| A-08 | Web-clipper extension install token | secret per-installation | web-clipper-bridge |
| A-09 | Search index (Professional only) | BEHAVIORAL_TENANT_PRODUCT (no PHI ungated) | search-index |
| A-10 | Loro CRDT op-log (Professional only) | BEHAVIORAL_TENANT_PRODUCT | collab-edit |
| A-11 | Import / Export job artifacts | varies (BEHAVIORAL + PII + PHI) | import-pipeline / export-pipeline |
| A-12 | Audit-chain seals | AUDIT | (cross-BC) |
| A-13 | Workflow event stream | varies | (cross-BC) |

## Adversaries

| ID | Adversary | Capability | Notes |
|---|---|---|---|
| ADV-01 | External unauthenticated attacker | network adjacency; can issue HTTP/WSS | OWASP T10 baseline |
| ADV-02 | External authenticated attacker (account compromise) | valid user JWT for victim user | credential-stuffing / phishing |
| ADV-03 | Insider — tenant admin | tenant scope authority + escalation potential | dual-context invariant guards Personal tier |
| ADV-04 | Insider — oyatie operator | infra access (k8s, Postgres root, OpenBao admin) | Personal-tier E2E ciphertext only; structural impossibility for plaintext |
| ADV-05 | Hostile co-tenant | valid principal in *different* tenant | tenant-scope Cedar default-deny |
| ADV-06 | Malicious browser extension (third-party) | injects script into page where oyatie clipper installed | extension isolation; MV3 minimum permissions |
| ADV-07 | Compromised foundry-runtime model provider | can exfiltrate inputs sent for AI assist | E2E refusal invariant; Professional-tier requires consent + audit |
| ADV-08 | Supply-chain attacker (npm / cargo dependency) | introduces malicious code at build time | SLSA L3 + signed builds + dep pinning |
| ADV-09 | Regulator / law-enforcement compulsion | subpoena to oyatie for content | Personal-tier: oyatie cannot produce plaintext; Professional: four-eyes + legal-hold escrow |
| ADV-10 | Quantum-equipped adversary (future) | breaks classical ECDH | PQ-MLS migration path tracked in ADR-NOTES-0001 §Future |

## STRIDE Per Asset

### A-01 Personal-tier note plaintext

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-S-01 Spoofing | Attacker poses as user; tries to fetch plaintext | low | catastrophic | E2E client-derived keys; oyatie has ciphertext only; MLS KeyPackage authn; OIDC + Cedar | residual = 0 (oyatie cannot decrypt regardless) |
| T-T-01 Tampering | Attacker mutates ciphertext blob | medium | catastrophic | MLS authenticated encryption (RFC 9420 §6.2); ciphertext MAC tag detects tamper | residual = low |
| T-R-01 Repudiation | User denies authoring a note | low | low | client-signed MLS commit chain; per-device signature | residual = low |
| T-I-01 Info disclosure | server-side read of plaintext | n/a (impossible by construction) | catastrophic if violated | ADR-NOTES-0001 + Cedar `e2e-ai-refusal` + LEAN lane + code-review gates | residual = 0 if invariant holds |
| T-D-01 Denial of service | mass ciphertext upload to fill quota | medium | medium | per-user storage quota; rate-limit | residual = low |
| T-E-01 Elev. of privilege | escape from Personal to Professional context | low | catastrophic | dual-context-isolation invariant (DCI-07: immutable context_kind) | residual = 0 by data model |

### A-02 Personal-tier MLS group keys

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-S-02 Spoofing KeyPackage | Attacker registers KeyPackage for victim | low | catastrophic | OIDC-bound KeyPackage signing; one-package-per-device-per-user invariant | residual = low |
| T-T-02 Tampering | Attacker mutates KeyPackage in transit | low | high | TLS + KeyPackage signature; revocation list | residual = low |
| T-I-02 Info disclosure | server reads private keys | n/a | catastrophic | client-only key derivation; openmls 0.6 zeroize-on-drop | residual = 0 |
| T-D-02 Denial of service | flood server with KeyPackage requests | medium | medium | per-tenant rate-limit; ContentTypeFilter | residual = low |

### A-04 Professional-tier note plaintext

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-I-04 Info disclosure (admin abuse) | tenant admin reads notes without authority | medium | high | Cedar `professional-channel-legal-hold.cedar`-style four-eyes; audit-chain Ed25519 seal; ADR-0215 inherited | residual = low |
| T-T-04 Tampering | content mutation by adversary in transit | low | high | TLS + envelope MAC; audit-chain immutable trail | residual = low |
| T-R-04 Repudiation | tenant denies disclosure occurred | low | medium | audit-chain Merkle + Ed25519; Bominal ADR-0028 | residual = 0 |
| T-E-04 Elev. of privilege | end-user reads notes outside membership | low | high | Cedar `tenant-scope.cedar` + `ci-scope.cedar` default-deny + member-check | residual = low |

### A-07 Share-link tokens

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-S-07 Spoofing | guess share-link token | low | medium | 128-bit URL-safe random; rate-limit on prefix scan | residual = low |
| T-I-07 Info disclosure | accidental token leak in URL referrers | medium | high | strip Referer header; optional passphrase (PBKDF2-SHA256 600k iter); short-lived TTL default | residual = medium |
| T-D-07 Denial of service | enumeration | medium | low | rate-limit + CAPTCHA after threshold | residual = low |

### A-08 Web-clipper installation token

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-S-08 Spoofing | replay clipper token from compromised extension | medium | medium | per-installation token; rotation 90d; per-request nonce | residual = low |
| T-I-08 Info disclosure | malicious page reads token via DOM | medium | high | MV3 isolated world; minimum-permission extension manifest; never expose token via DOM | residual = low |
| T-E-08 Elev. of privilege | clipper writes to other users' inbox | low | medium | token scoped to user; server rejects mismatch | residual = low |

### A-09 Search index (Professional only)

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-I-09 Info disclosure | search leaks via non-Cedar-scoped query | medium | high | Cedar-scoped server-side filter; never client-trusted; per-tenant index partition | residual = low |
| T-I-09b Cross-tenant index bleed | Meilisearch misconfiguration leaks tenant index | low | catastrophic | per-tenant Meilisearch namespace (`tenant_<id>`) + index-isolation lint at IaC; weekly Meilisearch index-perm audit | residual = low |
| T-T-09 Tampering | adversarial query injects to corrupt index | low | medium | strict query syntax; deny unsanitised faceted filter; Meilisearch hardened-config | residual = low |

### A-11 Import / Export artifacts

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-I-11 Info disclosure | malicious ENEX with embedded payload exfiltrates other notes during import | medium | high | sandboxed import worker; ENEX validation per Evernote schema; CSP for any rendered HTML | residual = low |
| T-T-11 Tampering | tampered Obsidian vault overwrites user's existing notes | low | high | merge-with-suffix on conflict; explicit replace-confirmation UX | residual = low |
| T-E-11 Elev. of privilege | export pipeline reads notes outside requester scope | low | high | requester-scoped Cedar evaluation per row; export job per-user binding | residual = low |

### A-13 Workflow event stream

| Threat | Vector | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| T-I-13 Info disclosure | Personal-tier event body leaks to downstream | low | high | Personal-tier events carry opaque `note_id` only; never title or body; LEAN lane verifies | residual = 0 by event-schema |
| T-T-13 Tampering | adversary forges event into workflow-engine | low | high | event-source authn (mTLS + per-µservice token); workflow-engine signature verification | residual = low |

## E2E + AI Interaction (Cross-Asset)

Per ADR-NOTES-0005, AI assist on E2E notes is structurally impossible because:

1. Cedar `e2e-ai-refusal.cedar` policy unconditionally `forbid`s `Action::ai_call` over `Resource` with `context_kind=Personal` AND `e2e=true`.
2. The `AssistInvoker` port trait signature accepts only `ProfessionalNoteRef`, not `PersonalNoteRef` — the type system refuses cross-tier construction.
3. The CI lane `oya-check-e2e-ai-refusal` greps for any path from `PersonalNoteRef` to `AssistInvoker::invoke` and BLOCKS on match.
4. Runtime metric `oya_notes_ai_call_blocked_e2e_total` increments on any forbidden attempt and alarms at > 0.

This is a sharper invariant than messenger's E2E posture because messenger E2E covers transport + at-rest; notes' E2E additionally covers *AI processing* — the highest-likelihood future-misuse vector.

## DDoS / Abuse

| Vector | Mitigation |
|---|---|
| Mass note-create flooding | per-user rate-limit (100 notes/min default); per-tenant rate-limit; Cedar deny on rate-violation |
| Tag-graph cardinality bomb | per-tenant tag-count limit (10M); rejection at API |
| Backlink fan-in bomb | per-note backlink fan-in cap (50k); reject append beyond cap |
| Web-clipper enumeration | per-installation rate-limit + per-tenant ceiling |
| Search-query CPU | Meilisearch query timeout 1s; degraded-mode reduces fuzziness |

## Supply Chain (SLSA L3)

- All Cargo deps pinned with checksum in `Cargo.lock` per LTS policy (Postgres 16, Redis 7.2, Meilisearch 0.10.0, Loro 1.x, openmls 0.6, Cedar v4.2).
- Browser extension build artifacts signed with HSM-bound code-signing cert; reproducible builds verified at release.
- `oya gate validate version-pinning-conformance` exit 0 blocking on `dev`.

## Quantum Posture

MLS RFC 9420 currently uses ECDH (X25519) + Ed25519. Future PQ-MLS draft (`draft-ietf-mls-architecture-pq-mls`) tracked; migration path = epoch-bump-on-new-cipher-suite per RFC 9420 §11.6. Tracked in ADR-NOTES-0001 §Future.

## Monitoring

| Metric | Alert threshold | Runbook |
|---|---|---|
| `oya_notes_ai_call_blocked_e2e_total` | > 0 over 5m | runbooks/ai-classifier-rollback-e2e-respect.md |
| `oya_notes_dual_context_denied_total` | > 0 over 1m | policy/dual-context-isolation.md |
| `oya_notes_tag_graph_corruption_detected_total` | > 0 over 5m | runbooks/tag-graph-corruption.md |
| `oya_notes_attachment_loss_detected_total` | > 0 over 5m | runbooks/attachment-loss-recovery.md |
| `oya_notes_share_link_brute_force_attempts_total` | > 100/min per tenant | rate-limit auto-engages |
| `oya_notes_e2e_key_rotation_failure_total` | > 0 over 1h | runbooks/e2e-key-rotation-and-recovery.md |
| `oya_notes_web_clipper_invalid_token_total` | > 50/min per tenant | runbooks/web-clipper-degraded.md |
| `oya_notes_import_pipeline_failure_total` | > 0 over 1h | runbooks/import-pipeline-failure.md |

## Verification

- Annual external pen-test focused on E2E-bypass attempts and dual-context drift.
- Quarterly chaos-test: synthetic cross-context routing attempt verifies rejection + alert.
- LEAN lanes BLOCKER on `dev` and `staging`.

## References

- OWASP ASVS v4.
- NIST SP 800-53 Rev. 5 + SP 800-57 Part 1 Rev. 5.
- RFC 9420 (MLS).
- Standard Notes Threat Model Whitepaper (publicly available).
- Apple Notes Lockable Notes white-paper (within iCloud Security Guide).
- Obsidian End-to-End Encryption Sync threat model (`obsidian.md/sync-encryption`).
- ADR-NOTES-0001, ADR-NOTES-0004, ADR-NOTES-0005.
- Bominal ADR-0028 audit-chain.
- Bominal ADR-0111 envelope encryption.
- Bominal ADR-0208 dual-context.
- Bominal ADR-0215 four-eyes disclosure.
