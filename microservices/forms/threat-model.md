---
doc_class: ThreatModel
microservice: forms
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-forms
methodology: STRIDE (per ADR-0133 axis-2) + LINDDUN privacy axis
review_cadence: quarterly + on any new field-type / cross-microservice bridge
doc_status: published
---

# Forms — Threat Model (STRIDE + LINDDUN)

## Trust Boundaries

```
+----------------------------+
| Anonymous Submitter (web)  |   ← lowest trust
+-------------+--------------+
              | TLS 1.3 + hCaptcha/Turnstile/Friendly Captcha
              v
+-------------+--------------+
| OCI CDN edge (per-pack)    |   ← static assets only
+-------------+--------------+
              | TLS 1.3
              v
+-------------+--------------+
| OCI WAF + Istio gateway    |   ← rate-limit + CSP + bot detection
+-------------+--------------+
              | mTLS
              v
+--------------------------------+
| form-rest / response-collector |   ← Cedar default-deny
| (Forms µservice cluster)       |
+--------------------------------+
              | tenant-id-scoped queries (Citus RLS)
              v
+--------------------------------+
| Postgres + Citus (PII-encrypted column) + Valkey + Meilisearch |
+--------------------------------+
              | gRPC + mTLS via Workflow + Ontology adapter
              v
+-------------------------------------------------+
| Sibling µservices (workflow-engine, sheets,     |
| drive, mail, messenger, audit-chain, tenancy,   |
| foundry-providers, foundry-runtime, ontology)   |
+-------------------------------------------------+
```

## STRIDE Mapping

### Spoofing

| ID | Threat | Likelihood | Impact | Mitigation | Verification |
|---|---|---|---|---|---|
| T-S-01 | Anonymous submitter forges pre-fill HMAC link | M | Sensitive PII enters wrong response bucket | HMAC-SHA-256 with per-tenant secret + TTL ≤ 7d; constant-time compare | AC-07; `tests/integration/prefill_hmac.rs` |
| T-S-02 | Forms-CI SPIFFE identity stolen and used to read tenant draft | L | Mass tenant draft leak | SPIFFE rotation ≤ 24h; OpenBao audit; CI never reads draft content (policy/ci-scope.cedar FORBID) | `policy/ci-scope.cedar` test corpus |
| T-S-03 | Submitter spoofs OIDC subject claim on authenticated form | L | Cross-submitter response attribution | OIDC issuer pin per tenant; JWKS cache 5min; `iss + aud + sub` triple-check | tenancy contract test |
| T-S-04 | Template-marketplace publisher key stolen → malicious template | L | Tenant installs trojan template | Per-pack publisher key (multi-sig for top-10 publishers); template-quarantine on signature-drift | `oya-forms-template-signature-conformance` |
| T-S-05 | hCaptcha bypass via solver bot | M | Spam flood into response store | Multi-provider fallback; per-IP rate-limit at WAF; anomaly detection | `runbooks/captcha-degraded.md` |

### Tampering

| ID | Threat | Likelihood | Impact | Mitigation | Verification |
|---|---|---|---|---|---|
| T-T-01 | Submitter tampers with hidden conditional-logic to reveal disabled fields | M | Skip-logic bypass; PHI / Art. 9 data accidentally captured | Server-side authoritative conditional-logic eval (per ADR-FORMS-0004); client eval advisory only | AC-15 |
| T-T-02 | Tenant operator edits a published form to remove required-fields after responses collected | L | Audit drift | Form versioning per ADR-0110; version-isolation enforced | AC-05 |
| T-T-03 | Webhook payload tampered in transit | L | Downstream system trust broken | mTLS + HMAC-SHA-256 on payload; receiver verifies | AC-21 |
| T-T-04 | Response export tampered after generation | L | Tenant export integrity | Ed25519 seal on export manifest; tenant verifies before consume | Export worker test |
| T-T-05 | AI-form-build LLM emits a form definition with `data_class:NORMAL` for an actual PII field | M | PII captured without encryption | dsl-loader rejects LLM output that mismatches inferred data_class; reviewer-agent gate | ADR-FORMS-0005 |

### Repudiation

| ID | Threat | Likelihood | Impact | Mitigation | Verification |
|---|---|---|---|---|---|
| T-R-01 | Tenant operator denies authoring a form that later proves harmful | L | Compliance dispute | Author OIDC sub + timestamp + Ed25519 seal on every form publish | Audit-chain seal |
| T-R-02 | Submitter denies submitting a response | L | Dispute | Submitter hash + IP + UA + audit-chain seal at submit | Same |
| T-R-03 | Tenant denies issuing a DSR | L | Audit dispute | DSR request audit-sealed | DSR runner |
| T-R-04 | E-signature signatory denies signing | L | Legal dispute | eIDAS QES (tier-G+) with QSCD + qualified cert; PAdES-LTA archival | ADR-FORMS-0006 |

### Information Disclosure

| ID | Threat | Likelihood | Impact | Mitigation | Verification |
|---|---|---|---|---|---|
| T-I-01 | Cross-tenant response leak (tenant A reads tenant B's responses) | M | Mass PII breach | Citus RLS + Cedar default-deny + per-tenant DEK | AC-28; chaos drill quarterly |
| T-I-02 | Form-rendered iframe embedded on attacker-controlled origin steals submitter data | M | PII exfiltration | CSP `frame-ancestors` allow-list per tenant; X-Frame-Options as defense-in-depth | AC-10 |
| T-I-03 | Browser XSS via tenant-authored form label injects script into renderer | M | Cookie / submitter token theft | Renderer Trusted Types + strict CSP `script-src 'self'`; sanitised at authoring | `runbooks/embed-iframe-csp-incident.md` |
| T-I-04 | PII column in Postgres readable by DBA without DEK | L | DBA insider risk | Column-level envelope encryption; DEK in OpenBao; DBA has no DEK access | ADR-FORMS-0003; AC-08 |
| T-I-05 | Meilisearch index leaks PII via search log | L | Indirect disclosure | PII columns NOT indexed; only non-PII fields searchable | Index policy |
| T-I-06 | LLM (AI-form-build) sees PII in tenant prompt and learns it | M | Cross-tenant inference | PII redactor before LLM call; BYO-LLM zero-retention; archive purged at TTL | ADR-FORMS-0005 |
| T-I-07 | Auditor JIT pivots to non-scoped tenant | L | Cross-tenant audit leak | Cedar auditor-scope `allowed_tenants` claim enforced | `policy/auditor-scope.cedar` |
| T-I-08 | DSR-erased response remains in audit-chain seal hash | L | Theoretical reverse-lookup | Audit-chain stores hash of response content, not the content itself | Audit-chain design |

### Denial of Service

| ID | Threat | Likelihood | Impact | Mitigation | Verification |
|---|---|---|---|---|---|
| T-D-01 | Spam flood: 1M anonymous submits / hour from botnet | H | Response-store DoS | Per-IP + per-form rate-limit at WAF; captcha; auto-throttle on rate spike | `runbooks/spam-flood-throttle.md` |
| T-D-02 | Tenant publishes a form with 10k cross-field validation rules; submit DoS | M | CPU exhaustion | Validation rule count cap (≤ 200 per form); declarative DAG cycle detection | AC-04 |
| T-D-03 | Captcha provider outage | M | New submits blocked | Multi-provider fallback; manual review queue | `runbooks/captcha-degraded.md` |
| T-D-04 | File-upload 1GB / submission × 10k recipients | H | Storage + scan DoS | Per-form size cap; ClamAV streaming; pre-flight cap check | `runbooks/upload-flood.md` (in runbook set under spam-flood) |
| T-D-05 | Bulk-distribute to 10k recipients × 1k forms / hour | M | Mail queue DoS | Token-bucket per tenant; queue depth SLI; ops runbook | `runbooks/bulk-distribute-overload.md` (folded into runbook 7) |
| T-D-06 | Export 100k responses CSV concurrent × 100 tenants | M | Export-worker DoS | Per-tenant export quota; queue depth; streaming output | `runbooks/export-pipeline-failure.md` |
| T-D-07 | Captcha sidecar crashes → fail-open accepts all submits | L | Spam flood | Fail-closed: missing captcha verification → 503 | hCaptcha sidecar test |

### Elevation of Privilege

| ID | Threat | Likelihood | Impact | Mitigation | Verification |
|---|---|---|---|---|---|
| T-E-01 | Tenant operator escalates to tenant admin to read others' responses | L | Within-tenant breach | Per-form ACL within tenant; Cedar tenant-scope plus role check | tenancy |
| T-E-02 | Submitter elevates to authenticated form path via OIDC token theft | L | Cross-submitter | OIDC re-auth on sensitive forms; MFA recommendation | Tenancy MFA |
| T-E-03 | AI-form-build emits a form that calls a foreign µservice the tenant has no entitlement to | M | Cross-product privilege | ADR-FORMS-0005 + Cedar destination-µservice check | AC-25 |
| T-E-04 | Webhook target receives a tenant_id claim it can replay against another endpoint | L | Replay | Per-delivery nonce + HMAC + audience claim | AC-21 |

## LINDDUN Privacy Axis

| ID | Threat | Mitigation |
|---|---|---|
| L-Link-01 | Submitter responses across forms linkable via submitter_hash | submitter_hash is per-form salted; never raw |
| L-Identify-01 | Quasi-identifying combination of non-PII fields | k-anonymity check at export ≥ 5; redact if < |
| L-Non-repudiation-01 | Whistleblower form: submitter should be repudiable | Anonymous-with-no-logs mode; IP not stored; submitter_hash empty |
| L-Detect-01 | Submitter IP visible in audit-chain | IP hashed (HMAC) before seal; raw IP not stored |
| L-Disclose-01 | Per-pack DPIA covers data-flow disclosure | `dpia.md` |
| L-Unaware-01 | Submitter unaware of AI-authored form / processing | UI banner; Art. 13 GDPR notice |
| L-Non-comply-01 | Tenant uses Forms outside their declared purpose | Cedar PERMIT-9 purpose-binding; cross-purpose re-export blocked |

## Mitigation Coverage Verification

`oya-forms-threat-model-coverage` CI lane asserts every threat ID has a corresponding test or runbook reference. Coverage gap blocks promotion.

## References

- `policy/*.cedar`.
- `runbooks/*.md`.
- ADR-FORMS-0001..0006.
- ADR-0028 audit-chain.
- ADR-0140 (retired per ADR-0145) Cedar default-deny.
- OWASP Top 10 for LLM Applications.
- LINDDUN privacy threat modelling — KU Leuven 2018; ENISA 2023 update.
- STRIDE — Microsoft SDL; "Threat Modeling: Designing for Security" Shostack 2014.
