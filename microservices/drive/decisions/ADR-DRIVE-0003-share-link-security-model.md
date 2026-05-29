---
id: ADR-DRIVE-0003
status: Accepted
date: 2026-05-17
microservice: drive
deciders: axis-drive, council-architecture, ops-security, council-privacy
owner: ops-security + axis-drive
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0135, ADR-0131, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-DRIVE-0004, ADR-DRIVE-0006]
related_artifacts:
  - microservices/drive/PRD.md (§FR-04 share-link)
  - microservices/drive/policy/public-read.cedar
  - microservices/drive/policy/tenant-scope.cedar
  - microservices/drive/runbooks/share-link-takeover-incident.md
purpose: |
  Pick a share-link security model: signing primitive, password-protection
  KDF, TTL semantics, view-count cap mechanism, revocation cascade semantics.
  Match Box / Dropbox / Proton Drive feature parity (per
  `competitor-parity-matrix.md`) while satisfying the PRD's audit-chain +
  GDPR + KR PIPA + HIPAA constraints.
---

# ADR-DRIVE-0003: Share-link security — Ed25519 + HKDF signing; Argon2id KDF for password-protected links; strict-TTL; view-count cap; revocation cascade

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-drive §FR-04 + threat-model §T-S-02 + §T-I-01 + §T-D-02 require share-links that resist signature forgery, enumeration attacks, and timing side-channels — while supporting password protection, expiration, view-count caps, and per-link revocation.

Industry precedent:

| Competitor | Signing | Password | TTL | View-cap |
|---|---|---|---|---|
| Google Drive | HMAC-SHA-256 internal | yes | yes | no |
| Dropbox | HMAC-SHA-256 | yes | yes | yes |
| OneDrive | HMAC-SHA-256 | yes | yes | no |
| Box | HMAC + per-tenant key | yes | yes | yes (7 levels) |
| Proton Drive | Ed25519 over content | yes (zero-knowledge) | yes | yes |
| Tresorit | Argon2 + Ed25519 | yes | yes | yes |
| AWS S3 presigned URL | HMAC-SHA-256 SigV4 | no (presigned only) | yes | no |

Candidate signing primitives:
- **HMAC-SHA-256** — symmetric; AWS S3 presigned URL standard.
- **Ed25519** — asymmetric; faster verification than RSA/ECDSA; smaller signatures; resistant to nonce-reuse attacks. ([cr.yp.to/ed25519](https://cr.yp.to/ed25519))

Candidate password KDFs:
- **PBKDF2** — older; tunable but slower than alternatives at same security level.
- **bcrypt** — older; widely deployed; 72-byte input limit.
- **scrypt** — memory-hard; configurable; older than Argon2.
- **Argon2id** — RFC 9106; current OWASP recommendation; memory-hard + parallel-resistant. ([datatracker.ietf.org/doc/html/rfc9106](https://datatracker.ietf.org/doc/html/rfc9106))

Per ADR-0133 axis-2 security-conformance, the chosen primitive must be a current OWASP / NIST recommendation; legacy primitives (PBKDF2, bcrypt, scrypt) acceptable only with explicit justification.

## Decision

The drive µservice ships **Ed25519 + HKDF for signing share-links + Argon2id (RFC 9106) for password-protected links + strict-TTL + view-count cap with deterministic decrement + revocation cascade**:

### Signing

- **Primitive**: Ed25519 over a canonicalised JSON serialisation of `(link_id, file_or_folder_id, expires_at, scope, view_cap_initial, password_protected_flag)`.
- **Per-tenant signing key**: 256-bit Ed25519 private key stored in OpenBao Transit (FIPS 140-3 module per ADR-DRIVE-0004); HKDF derives per-link signing key from per-tenant key + `link_id`.
- **Rotation**: per-tenant signing key rotates every 30d; rotation invalidates all extant links (tenant comms required).

### Password protection (optional)

- **KDF**: Argon2id (RFC 9106).
- **Parameters**:
  - `memory_cost = 64 MiB`
  - `time_cost = 3`
  - `parallelism = 4`
- **Tuned for**: ≤ 50ms per verification at p99 on a 2 vCPU pod; bounds attacker brute-force rate at ≤ 20 attempts/sec/CPU.
- **Salt**: per-link random 128-bit; never reused.
- **Storage**: derived hash + salt; cleartext password never stored.

### TTL

- **Strict TTL** — no clock-skew tolerance. Legacy (`oya-drive-domain`) had a 1s tolerance; this is removed per `feedback_no_silent_regression`.
- **Maximum TTL**: 1 year (31,536,000 seconds).
- **Default TTL**: 7 days.

### View-count cap

- **Initial value**: tenant-set at mint time; default unlimited; max 10⁶ (10 million views per link).
- **Decrement**: atomic via Postgres `UPDATE share_link SET view_cap_remaining = view_cap_remaining - 1 WHERE link_id = $1 AND view_cap_remaining > 0 RETURNING *`. Race-free.
- **Audit-chain**: every decrement emits.

### Revocation cascade

- **Trigger**: tenant explicit revoke; per-tenant signing-key rotation; tenant offboarding; legal-hold open (per ADR-DRIVE-0006); DLP-flag raise.
- **Effect**: link marked revoked in Postgres; in-memory cache invalidated; subsequent access returns `410 Gone`.

### Anti-enumeration

- **Link IDs**: 256-bit cryptographic randoms, URL-safe base64 (43 chars).
- **Rate limit**: per-IP 100 verify-requests / minute (Cedar `public-read.cedar` enforces).
- **Anomaly detection**: per-IP enumeration pattern → auto-block at WAF.
- **Constant-time response**: signature_invalid + link_not_found return same status code + same response shape + same timing (within ±2ms).

## Alternatives Considered

### A. HMAC-SHA-256 (AWS S3 presigned style)

- **Pros**:
  - AWS S3 reference standard.
  - Symmetric → faster signing (~10× Ed25519 per sign-op).
  - Single primitive to learn.
- **Cons**:
  - Symmetric — signing key compromise allows arbitrary link forgery; revocation requires per-tenant key rotation cascading to ALL extant links.
  - Less resistant to side-channel attacks than Ed25519.
- **Rejected** in favour of Ed25519's asymmetric properties (verification doesn't require holding the signing key).

### B. RSA-PSS

- **Pros**:
  - Asymmetric; verification doesn't require signing key.
  - PKCS standard.
- **Cons**:
  - ~10× slower signature than Ed25519.
  - Larger signatures (RSA-2048 → 256 bytes vs Ed25519 64 bytes).
- **Rejected** in favour of Ed25519 for speed + signature size.

### C. PBKDF2 for password (legacy)

- **Pros**:
  - PKCS#5 standard.
  - Widely supported.
- **Cons**:
  - Not memory-hard; vulnerable to GPU brute-force.
  - OWASP recommendation has moved to Argon2id.
- **Rejected** in favour of Argon2id.

### D. bcrypt for password

- **Pros**:
  - Mature.
  - Wide language support.
- **Cons**:
  - 72-byte input limit (passwords > 72 bytes get truncated).
  - Not memory-hard; less resistant to GPU brute-force than Argon2id.
- **Rejected** in favour of Argon2id.

### E. scrypt for password

- **Pros**:
  - Memory-hard.
  - Parameter-tunable.
- **Cons**:
  - Older than Argon2.
  - Less explicit GPU-resistance guarantees than Argon2id.
- **Rejected** in favour of Argon2id (RFC 9106 supersedes).

### F. Ed25519 + Argon2id + strict-TTL + view-cap + revocation cascade  ← **CHOSEN**

- **Pros**:
  - Asymmetric signing (Ed25519) — verification doesn't expose signing key.
  - Argon2id RFC 9106 — current OWASP / NIST recommendation.
  - Strict TTL — eliminates the legacy 1s clock-skew side-channel.
  - View-cap atomic decrement — race-free.
  - Revocation cascade — closes Hyrum-bound zombie-link surface.
- **Cons**:
  - Stricter than legacy (1s skew tolerance removed); migration consumer notification required.
  - Argon2id at 50ms/verify caps share-link verify throughput at ~20 verifications/sec/CPU; CPU sizing must account.
- **Accepted**.

## Consequences

### Positive

- **Forgery-resistant**: Ed25519 signature over canonicalised payload; HKDF per-link key isolation.
- **Brute-force-resistant**: Argon2id at 50ms/verify + per-IP rate limit + anomaly detection.
- **Race-free view-cap**: Postgres atomic decrement.
- **Revocation cascade**: in-memory + DB; subsequent access returns 410 Gone with audit-chain entry.
- **Audit-trail**: every mint + access + revoke emits to audit-chain (Ed25519-sealed; per Bominal ADR-0028).

### Negative

- **Hyrum's-Law surface #2 + #3**: legacy blob shape (HMAC over fixed-field serialisation) differs from new Ed25519 over canonicalised JSON. The HTTP GET URL is the public contract, so blob shape is internal. Strict TTL eliminates the 1s skew tolerance; consumers with timing-dependent tests may need a ≤ 1s tolerance allowance.
- **Argon2id CPU cost**: 50ms/verify caps throughput; mitigated by HPA on share-link-rest pods + per-IP rate limit at WAF.

### Hyrum's Law

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law":
- **Share-link blob format**: legacy HMAC over fixed-field serialisation; new Ed25519 over canonicalised JSON. Consumers that pattern-matched on the legacy blob see different bytes; HTTP-level GET URL is the public contract. Documented in `migration-from-connect.md` Hyrum #2.
- **TTL boundary**: legacy 1s clock-skew tolerance removed; consumers with timing-dependent tests adjust. Documented in `migration-from-connect.md` Hyrum #3.

### Operational

- **New CI lane**: `oya-governance-share-link-signing-conformance` (BLOCKER) — validates Ed25519 + Argon2id KAT test vectors.
- **OpenBao Transit FIPS 140-3 module** required for per-tenant signing key generation + storage.
- **Per-tenant signing key rotation**: 30d; rotation event emits cascade event.
- **Runbook** `share-link-takeover-incident.md` documents key-compromise mitigation.

## Verification

- [ ] Ed25519 KAT test vectors — `cargo nextest run -p oya-drive-share-link-domain -- ed25519_kat`.
- [ ] Argon2id KAT test vectors — `cargo nextest run -p oya-drive-share-link-domain -- argon2id_kat`.
- [ ] Strict-TTL boundary — `cargo nextest run -p oya-drive-share-link-domain -- strict_ttl`.
- [ ] View-cap atomic decrement — `cargo nextest run -p oya-drive-share-link-adapter-postgres -- view_cap_race`.
- [ ] Constant-time response timing — `cargo nextest run -p oya-drive-share-link-rest -- constant_time_response`.
- [ ] Anti-enumeration rate-limit — `cargo nextest run -p oya-drive-share-link-rest -- enumeration_rate_limit`.

## References

- Bernstein, D. J. et al. "High-speed high-security signatures (Ed25519)." `cr.yp.to/ed25519`.
- RFC 8032 — Edwards-Curve Digital Signature Algorithm (EdDSA).
- RFC 9106 — Argon2 Memory-Hard Function.
- RFC 5869 — HKDF.
- OWASP Password Storage Cheat Sheet (2024) — Argon2id recommendation.
- AWS S3 SigV4 specification — presigned URL reference.
- Box developer docs — share-link levels.
- Dropbox developer docs — share-link API.
- Tresorit security whitepaper — share-link model.
- ADR-0140 — Cedar policy enforcement (`public-read.cedar` + `tenant-scope.cedar`).
- ADR-DRIVE-0004 — encryption-at-rest (OpenBao Transit FIPS 140-3).
- ADR-DRIVE-0006 — WORM (revocation cascade includes WORM-bound files).
- `microservices/drive/PRD.md` §FR-04; §"Performance" share-link generation.
- `microservices/drive/policy/public-read.cedar`.
- `microservices/drive/policy/tenant-scope.cedar`.
- `microservices/drive/runbooks/share-link-takeover-incident.md`.
- `microservices/drive/migration-from-connect.md` Hyrum #2 + #3.
- `feedback_no_silent_regression.md`.
