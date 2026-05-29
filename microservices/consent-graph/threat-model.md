# consent-graph threat model

- Owner: axis-consent-graph + security-axis
- Methodology: STRIDE + LINDDUN (privacy) + custom cross-tenant adversary classes
- Date: 2026-05-18
- Authority: ADR-0214 §7 verification, ADR-0090, ADR-0072, ADR-0003.

This document enumerates every attack vector against the consent-graph µservice, the mitigations
present, and the residual risk. The threat model is deeper than for an internal-only µservice because
consent-graph mediates cross-organizational trust — adversary classes include external partners
acting in bad faith.

---

## 1. Adversary classes

| Class | Position | Capabilities |
|-------|----------|--------------|
| **A1 External-untrusted** | outside oyatie | DDoS, network sniff, social engineering |
| **A2 Partner-tenant-rogue** | grantee or grantor tenant in good faith with one compromised account | mTLS valid creds; valid agreement; tries to exceed scope |
| **A3 Partner-tenant-malicious** | grantee or grantor tenant intentionally adversarial | forges grants, replay attacks, audit-chain tampering, sovereignty bypass |
| **A4 Internal-low-privilege** | oyatie employee with bounded role | read internal dashboards; cannot directly modify Postgres |
| **A5 Internal-high-privilege** | oyatie SRE / platform engineer | k8s admin; can read OpenBao if not bound by audit |
| **A6 Insider-rogue** | oyatie engineer acting in bad faith | full code + secrets access for their µservice |
| **A7 Nation-state** | sophisticated, persistent | supply-chain compromise; multi-stage; targeted |

Each subsection below maps threats to these classes.

---

## 2. STRIDE — Spoofing

### 2.1 Spoofed grantor identity
- **Threat**: A3 forges agreement claiming to grant from a tenant they don't control.
- **Mitigation**:
  - Agreement create requires mTLS-authenticated session bound to grantor tenant_id.
  - Pulsar JWT issuer key pinned per tenant in partner-directory.
  - Audit-chain bilateral entry requires grantor-chain seal — cannot fake without grantor's
    audit-chain HSM key.
- **Residual**: low. A3 must compromise grantor's mTLS key + HSM key simultaneously.

### 2.2 Spoofed grantee identity at projection-subscribe
- **Threat**: A3 subscribes to a projection topic claiming to be the grantee tenant.
- **Mitigation**:
  - Pulsar topic ACL is grantee-tenant-only (per IP-010).
  - JWT token audience binds to specific topic + grantee tenant.
  - mTLS chain to Pulsar broker verifies grantee's SPKI from partner-directory.
- **Residual**: low. A3 must compromise grantee's mTLS + Pulsar JWT private key.

### 2.3 Spoofed principal within grantee tenant
- **Threat**: A2 (rogue user in grantee org) acts as a higher-privileged principal.
- **Mitigation**:
  - Identity µservice issues per-principal mTLS certs via SPIFFE.
  - Cedar policy `principal_id` resolution uses SPIFFE workload identity, not self-claim.
  - Audit-chain entry records actual SPIFFE identity, not claimed principal.
- **Residual**: bounded by identity-µservice security posture (separate threat model).

---

## 3. STRIDE — Tampering

### 3.1 Audit chain entry tampering on grantor side
- **Threat**: A6 (insider) modifies grantor's chain to hide a grant.
- **Mitigation**:
  - Audit-chain Merkle seal makes tampering detectable (ADR-0003).
  - Bilateral chain: tampering grantor alone doesn't hide it from grantee's chain.
  - Daily reconciliation (IP-013) detects divergence within 24h.
- **Residual**: very low. Tamper requires modifying *both* chains + cross-pointer + HMAC, which
  requires *two* HSM-key compromises.

### 3.2 Cross-pointer table tampering
- **Threat**: A6 modifies `consent_graph_cross_pointers` to point one grantor entry at a different
  grantee entry.
- **Mitigation**:
  - `paired_hmac` field is HMAC-signed by per-pair OpenBao key; tampering detectable.
  - Recursive seal: cross-pointer rows are themselves audit-chained (IP-013 §8 reconciliation report
    is sealed).
- **Residual**: low. Tamper requires OpenBao key compromise.

### 3.3 Pulsar message tampering in flight
- **Threat**: A7 MITMs between grantor's projection-gateway and Pulsar broker.
- **Mitigation**:
  - mTLS on all Pulsar connections (Cilium L4 + Istio ambient ztunnel).
  - Pulsar message-level signing optional; not enabled by default (signed wrapper increases overhead
    20%).
- **Residual**: moderate. PHASE-02 considers per-message signing as ADR-SVC-CG-*.

### 3.4 Postgres row tampering
- **Threat**: A5 (SRE) modifies a `consent_graph_agreements` row directly.
- **Mitigation**:
  - Postgres logical replication to audit-chain seals every commit (per ADR-0003).
  - RLS prevents most tenants from cross-reading; SRE bypasses RLS.
  - Audit-chain diverges → reconciliation P0 alert.
- **Residual**: detectable but not preventable; relies on detect-and-respond.

---

## 4. STRIDE — Repudiation

### 4.1 Grantor denies issuing a grant
- **Threat**: Grantor claims "I never granted this access."
- **Mitigation**:
  - Bilateral chain: grantee's chain has an independent entry signed by grantee's side.
  - mTLS + JWT bind to grantor's HSM-protected key — grantor's signing key generated the grant
    payload.
  - Audit-chain provides non-repudiation per ADR-0003.
- **Residual**: none. Repudiation requires HSM key compromise + reconciliation gap.

### 4.2 Grantee denies reading data
- **Threat**: Grantee claims "I never accessed this."
- **Mitigation**:
  - Every projection-read emits an audit-chain entry on grantee's side (sampled per agreement config).
  - Pulsar broker logs subscriber activity; cross-referenced in IP-013 reconciliation.
- **Residual**: bounded by sample rate; high-stakes agreements config'd to 100% read auditing.

---

## 5. STRIDE — Information Disclosure (cross-tenant data leakage)

### 5.1 Field beyond scope leaks via projection emission
- **Threat**: Grantor's `projection-gateway-worker` emits a field not permitted by `EntityScope`.
- **Mitigation**:
  - `ProjectionScopeNarrower` is the only path that builds projection payloads (IP-011).
  - `redaction_applied` field on each event audits exactly which fields were redacted.
  - Kernel invariant `RedactionAppliedConsistentWithScope` checked pre-emit.
  - Unit + property tests cover narrowing logic.
- **Residual**: bounded by code-review discipline + property testing.

### 5.2 Aggregate-mode k-anonymity bypass
- **Threat**: Grantor's `Aggregator` emits a bucket with observed k below `k_anonymity` threshold.
- **Mitigation**:
  - Kernel invariant `AggregateModeKAnonHoldsAtEmit` checked pre-emit.
  - Below-threshold buckets suppressed + audit-event emitted (`aggregate_suppressed`).
  - DP noise on top adds defense-in-depth.
- **Residual**: low if k≥5; reconsidered if user sets k=2.

### 5.3 AttestedQuery query injection
- **Threat**: Grantee crafts query that bypasses agreement scope.
- **Mitigation**:
  - Query parsed by ontology query-domain; raw query text never passes through.
  - Re-check against agreement scope post-parse.
  - Cedar enforcement on the parsed AST.
- **Residual**: bounded by ontology query-domain hardening.

### 5.4 Differential-privacy budget exhaustion
- **Threat**: Adversarial grantee runs many aggregate queries to defeat DP guarantee.
- **Mitigation**:
  - Per-agreement DP budget tracked in `consent_graph_dp_budget` table.
  - Budget exhaustion → Indeterminate (effectively Deny).
- **Residual**: bounded by budget setting; default ε=1.0 total budget over agreement lifetime.

### 5.5 Cache-side-channel timing attack on Cedar evaluation
- **Threat**: Grantee infers scope by timing Permit vs Deny responses (different cache paths).
- **Mitigation**:
  - Constant-time response path: Cedar evaluator returns within budget regardless of Permit/Deny.
  - Cache hit/miss both within 10ms p99 → negligible timing channel.
- **Residual**: very low.

### 5.6 PII leak via predicate field reflection
- **Threat**: Grantor encodes PII in predicate (e.g., predicate references a customer name).
- **Mitigation**:
  - Predicate field is for Cedar conditions only; references `principal.*`, `resource.*`,
    `context.*` — never literal PII.
  - Linter on agreement-domain rejects predicates with string literals > 32 chars (likely-PII heuristic).
- **Residual**: bounded by linter precision.

---

## 6. STRIDE — Denial of Service

### 6.1 Cedar evaluation hot-path DoS
- **Threat**: A1 floods enforcement with requests.
- **Mitigation**:
  - Per-tenant rate limit (1K RPS default, configurable per agreement).
  - Ambient waypoint envoy rate-limiter ahead of consent-graph.
  - Cedar cache hit-rate ≥80% keeps p99 ≤10ms even under load.
- **Residual**: bounded by upstream API gateway rate-limit.

### 6.2 Revocation DDoS
- **Threat**: A3 (rogue partner) revokes 1M agreements / s.
- **Mitigation**:
  - Per-actor revocation rate limit (1K/min per tenant).
  - Pulsar topic priority lane ensures legitimate revocations process first.
- **Residual**: bounded by rate limit.

### 6.3 Cache stampede on cold start
- **Threat**: enforcement-app cold-start triggers compile storm.
- **Mitigation**:
  - WarmCache reads materialized compiled artifacts (IP-006).
  - Compile pool size 100 concurrent caps storm.
  - 30s ready budget tolerates worst case.
- **Residual**: bounded by warm-cache table freshness.

### 6.4 Pulsar broker overload from projection emission
- **Threat**: A3 grants 100K agreements emitting at max rate → Pulsar saturated.
- **Mitigation**:
  - Per-topic partition cap (16 default).
  - Topic-level rate limit (10K msg/s default).
  - Per-grantor agreement count cap (10K active default).
- **Residual**: bounded by caps; auto-scale Pulsar broker pool above 70% saturation.

---

## 7. STRIDE — Elevation of Privilege

### 7.1 Grantee escalates from project.subscribe to project.read on out-of-scope entity
- **Threat**: Cedar policy bug permits broader read than intended.
- **Mitigation**:
  - Cedar policy auto-compiled from agreement; no manual policy authoring.
  - Policy compilation in `enforcement-domain` is unit-tested for known patterns.
  - Cedar evaluator's `Authorizer` is deny-by-default.
- **Residual**: bounded by compiler-correctness — comprehensive test set required (IP-005 §8).

### 7.2 Grantee gains projection-publish (write) capability
- **Threat**: Grantee tries to publish to projection topic.
- **Mitigation**:
  - Topic ACL is explicit Subscribe-only (IP-010 §4).
  - Pulsar authorization plugin enforces ACL.
- **Residual**: very low.

### 7.3 Tenant gains agreement-manage on agreement they don't own
- **Threat**: A2 in tenant C attempts to manage agreement between A and B.
- **Mitigation**:
  - RLS on `consent_graph_agreements`: only grantor or grantee can read; only grantor can write
    (except revoke from grantee).
- **Residual**: bounded by Postgres + RLS correctness.

---

## 8. LINDDUN — Privacy threats

### 8.1 Linkability across agreements
- **Threat**: Grantee correlates data across multiple agreements with same data subject.
- **Mitigation**:
  - Per-agreement redaction salt prevents deterministic hash linkage.
  - DP noise on aggregate mode breaks small-N inference.
- **Residual**: bounded by salt + DP discipline.

### 8.2 Identifiability of data subject via cohort attack
- **Threat**: Aggregate cohort small enough to identify individual.
- **Mitigation**: k-anonymity ≥5; suppress below-threshold buckets.
- **Residual**: bounded by k; legal verticals require k≥10.

### 8.3 Non-repudiation harms data subject
- **Threat**: Audit-chain entries themselves leak PII to future audit reviewers.
- **Mitigation**:
  - Audit entries store agreement_id + event_type + redacted-fields-list, NOT the raw row data.
  - Original row data is reconstructable only via the ontology row (subject to retention + tombstone).
- **Residual**: very low.

### 8.4 Detectability of agreement existence
- **Threat**: Third party observes Pulsar topic name and infers a partnership exists.
- **Mitigation**:
  - Topic names use tenant *short-hash*, not human-readable names.
  - Pulsar admin API ACL restricts topic listing to grantor + grantee tenants only.
- **Residual**: low.

### 8.5 Disclosure of grant terms via Cedar policy text
- **Threat**: Cedar compiled policy stored in cache leaks terms.
- **Mitigation**:
  - Cache in-memory only; not persisted to disk.
  - Cache snapshot in `consent_graph_compiled_policies` table is RLS-protected.
- **Residual**: bounded by Postgres + RLS.

### 8.6 Unawareness — data subject doesn't know data is shared
- **Threat**: Consumer (data subject) unaware of B2C grant.
- **Mitigation**:
  - B2C grants are consumer-initiated (not standing); consumer literally clicks "Share with X."
  - Standing-consent model (where pre-authorized) requires upfront disclosure in privacy notice;
    DPIA §3.4 documents.
- **Residual**: regulatory; covered in dpia.md.

### 8.7 Non-compliance with data-subject rights (DSAR / erasure)
- **Threat**: DSAR request not honored across all grants.
- **Mitigation**:
  - DSAR triggers cascade: consent-graph enumerates all active agreements containing subject; emits
    tombstone signals; audit-chains the cascade (per runbook `GDPR-DSAR-cross-tenant.md`).
- **Residual**: bounded by 30-day regulatory clock; consent-graph completes within 7 days target.

---

## 9. Custom — cross-tenant adversary classes

### 9.1 Consent forgery
- **Threat**: A3 forges an "Accepted" event without grantee's actual acceptance.
- **Mitigation**:
  - Acceptance requires mTLS + JWT from grantee's identity (not grantor's).
  - Audit-chain entry on grantee side is signed by grantee's HSM key.
- **Residual**: requires grantee HSM compromise.

### 9.2 Replay attack on revocation event
- **Threat**: A7 captures a revocation event in flight + replays after revoke window closes.
- **Mitigation**:
  - Idempotency on revocation_id (rev_id ULID is unique).
  - Pulsar message dedup via sequence_id within partition.
- **Residual**: very low.

### 9.3 Revocation latency exploit
- **Threat**: A3 (grantee) reads aggressively in the gap between revoke event publish + propagation
  to enforcement-app.
- **Mitigation**:
  - 200ms freshness check in EnforcementContext (`prior_revocation_check_ms_ago` ≤200ms required).
  - Pulsar priority lane on revocation topic → ≤1s p99 propagation.
  - Sample reads above baseline trigger anomaly review (observability rule).
- **Residual**: ≤1s window of stale reads possible; documented in SLA.

### 9.4 Projection topic ACL bypass
- **Threat**: A3 (grantee in good faith) somehow gains topic.Manage permission.
- **Mitigation**:
  - ACL set by projection-gateway-app only; admin API token in OpenBao; rotated every 90d.
  - Pulsar admin audit log + consent-graph daily sovereignty reconciliation cross-checks.
- **Residual**: very low.

### 9.5 Sovereignty bypass via geo-replication misconfiguration
- **Threat**: A6 misconfigures Pulsar georep → KR data replicated to EU.
- **Mitigation**:
  - `mint` algorithm reads agreement.geo_replicate_to_grantee_region flag explicitly.
  - Pack overlay (`kr` pack) hardcodes geo_replicate=false.
  - Nightly sovereignty audit job (HG-CONSENT alert if mismatch).
- **Residual**: detectable within 24h.

### 9.6 Audit-chain divergence (bilateral gap)
- **Threat**: One side's chain has the event, the other doesn't.
- **Mitigation**: IP-013 hourly reconciliation; P0 auto-suspend on divergence.
- **Residual**: ≤1h window of latent divergence possible.

### 9.7 Cedar policy injection via predicate field
- **Threat**: A3 crafts predicate that includes Cedar functions not in the safe sublanguage.
- **Mitigation**:
  - Predicate parsed by typed AST in `agreement-domain`; renderer in `enforcement-domain` emits
    only well-formed Cedar.
  - Raw string never passes to Cedar.
- **Residual**: very low.

---

## 10. Cross-references

- Cedar policy stack: docs/decisions/ADR-0090, microservices/consent-graph/policy/*.cedar.
- Audit-chain seal mechanism: microservices/audit-chain/threat-model.md.
- Pulsar threat model: docs/decisions/ADR-0078-pulsar-substrate.md.
- Identity µservice principal authentication: microservices/identity/threat-model.md.
- Ontology cross-tenant projection threats: microservices/ontology/IP-CT-005-sovereignty-zero-copy.md
  §risks.

---

## 11. Open items / PHASE-02 follow-ups

- Per-message Pulsar signing (defense in depth for §3.3).
- Predicate-field linter heuristics tightening (false-positive review).
- ε differential-privacy budget reset cadence — currently per-agreement lifetime; PHASE-02 considers
  rolling-window reset.
- A7 nation-state threat — adopt hardware-rooted attestation (TPM, AWS Nitro, GCP Confidential VMs)
  for consent-graph workloads as PHASE-03+ hardening.
- 100% Permit-event audit emission (default 0.1% sample) — cost/benefit analysis for high-stakes
  verticals.
