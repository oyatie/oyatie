---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity + ops-security
methodology: STRIDE + RFC 8725 (JWT BCP) + RFC 9068 (access-token profile) + Cedar formal properties
related_adrs: [ADR-0002, ADR-0131, ADR-0183]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Threat Model

Workload identity is a top-tier attack surface: a forged token or a confused
authorization decision compromises every downstream µservice that trusts this
PDP. This model enumerates the threats the cited brief (§5) calls out — encoded
as concrete verifier test cases — plus the mitigations and detections.

## Trust boundaries

1. **PEP ↔ PDP** — Envoy waypoint / sidecar / in-process gate calls `/authorize`
   or `WorkloadAuthorizer.authorize`. mTLS via SPIFFE (ADR-0148). The PEP's own
   workload token authenticates the call.
2. **PDP ↔ JWKS source** — JWKS fetched over TLS from the per-trust-domain issuer;
   `jku`/`x5u` allowlist-gated (SSRF boundary).
3. **PDP ↔ policy store** — tenant-scoped Cedar partitions; unreachable store →
   embedded-Cedar default-deny.
4. **PDP ↔ revocation denylist** — suspended/retired principal ids; consulted at
   validate time.
5. **PDP ↔ audit chain** — immutable decision log emission (append-only).

## STRIDE by surface

### S1 — Token validation (`/tokens/validate`)

| STRIDE | Threat | Mitigation (RFC ref) | Detection / test |
|---|---|---|---|
| S | **Algorithm confusion** `RS256→HS256` — attacker signs with the RSA public key as an HMAC secret | Server-side `kid`→alg binding; header `alg` never trusted (RFC 8725 §3.1) | Test AC-W-02; metric `oya_identity_workload_validate_request_total{failure="algorithm-mismatch"}` |
| S | **`alg:none`** — unsigned token accepted | Reject `none` unconditionally (RFC 8725 §3.2) | Test AC-W-01 |
| S | **Key substitution / forgery** — token signed by a key not belonging to the issuer | Key-belongs-to-issuer check (RFC 8725 §3.8) | Test (oidc_validation: key-not-belongs-to-issuer) |
| T | **JWKS poisoning / SSRF** via attacker-controlled `jku`/`x5u` | `jku`/`x5u` allowlist-gated; `kid` sanitized; default static trust-domain→JWKS map (RFC 8725 §3.10) | Test AC-W-05 |
| R | Repudiation of a validation outcome | Every outcome emitted to `identity.authz.decision.v1` with stable never-reused id (brief §9) | Test AC-W-13 |
| I | **Replay** of a captured token | Short TTL + `aud` binding + `jti` for forensics (brief §5, §7) | `jti` recorded; replay window bounded by `exp` |
| I | Token body leaked in logs | Token bodies never logged; only `sub` + `jti` (brief §9) | Log-sieve gate; Test AC-W-17 |
| D | DoS via expensive verification flood | Per-tenant validate quota; key-set capped ≤100 (brief §10) | 429 rate; key-set-size metric |
| E | **Cross-JWT confusion** — an ID token or refresh token replayed as an access token | Explicit `typ` required (RFC 8725 §3.11–12) | Test (oidc_validation: missing `typ`) |

### S2 — Authorization (`/authorize`)

| STRIDE | Threat | Mitigation | Detection / test |
|---|---|---|---|
| S | Caller spoofs a principal it is not | `aud`/audience binding (S1) feeds the principal projection; cross-trust-domain forbid | Test AC-W-08 |
| T | Tampering with the PARC tuple in transit | mTLS PEP↔PDP | mesh handshake metrics |
| R | Repudiation of a decision | One immutable record per authorize, implicit-vs-explicit-deny preserved (brief §9) | Test AC-W-13; `design/audit-evidence-emission.md` |
| I | **Confused deputy** — caller induces the PDP to authorize against the wrong audience/resource | `aud` bound per principal/trust-domain (`forbid-audience-mismatch`); resource trust-domain checked (brief §5) | Test (cedar_authz: audience-mismatch) |
| I | Existence leak via status code | 403, never 404 (brief §2) | Test AC-W-10 |
| D | Authorize flood | Per-tenant quota; `/authorize:batch`; embedded Cedar | 429 rate |
| E | **Privilege escalation** — principal performs an action beyond least-privilege | Default-deny + capability/scope-scoped permits + forbid-overrides-permit (brief §5, §10) | Tests AC-W-06, AC-W-09 |

### S3 — Lifecycle (`:suspend` / `:retire`)

| STRIDE | Threat | Mitigation | Detection / test |
|---|---|---|---|
| S | Unauthorized suspend/retire | Caller authz → 403 never 404 | contract test |
| T | Status tampering via PATCH | Explicit transition sub-resources only (no status PATCH) (brief §2) | contract review |
| R | Repudiation of a transition | `identity.principal.lifecycle.v1` event sealed | Test AC-W-13 |
| E | **Lifecycle abuse / id reuse** — reusing a retired id to inherit grants | Principal-id immutability + non-reuse; retired tombstoned (brief §5) | Test AC-W-14 |

## Attack scenarios → defenses (brief §5 summary)

| Attack | Defense |
|---|---|
| `alg:none`, RS256→HS256 | server-side allowlist, reject none (encoded as mandatory verifier tests) |
| Forgery / key substitution | key-belongs-to-issuer (§3.8) |
| Replay | short TTL + `aud` + `jti` |
| Confused deputy | `aud` bound per principal/trust-domain |
| JWKS poisoning / SSRF | `jku`/`x5u` allowlist; `kid` sanitize (§3.10) |
| Cross-JWT confusion | explicit `typ` (§3.11–12) |
| Privilege escalation | least-privilege + default-deny + forbid-overrides-permit |
| Lifecycle abuse | unique never-reused ids; retired tombstoned |

## NHI 2025 context

Machine identities outnumber humans ~82:1 (brief §5). The mitigation posture is
short-lived, federated, secretless tokens validated by this substrate — never
long-lived shared secrets.

## References

RFC 8725 §3.1–3.2, §3.8, §3.10–12; RFC 9068; Cedar formal properties
(arXiv 2403.04651); the cited brief §5 + load-bearing flags. See the per-test
mapping in `IP-001-identity-design.md` Slice 2/3.
