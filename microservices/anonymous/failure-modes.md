---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + ops-platform + council-architecture
review_cadence: quarterly + post-incident
doc_status: published
---

# Failure Modes: anonymous µservice

## Methodology

FMEA + chaos-engineering-class scenarios. For each failure: (a) detection signal, (b) blast radius, (c) recovery, (d) preventive measure, (e) runbook.

## Catalog

### F-01 Blind-signature issuer key compromise

- **Detection:** OpenBao key-usage audit + per-issuance audit-chain anomaly + external responsible disclosure
- **Blast radius:** Critical — I1 invariant defeated; any past blind-signed credential potentially forge-replicable
- **Recovery:** Emergency rotation per `runbooks/blind-signature-key-ceremony.md`; all in-flight credentials invalidated; users re-attest; past audit-chain remains valid (signature provenance preserved separately)
- **Prevention:** HSM-backed key storage; key ceremony with split-knowledge + dual-control; quarterly rotation; OpenBao policy bounds
- **Runbook:** `runbooks/blind-signature-key-ceremony.md`

### F-02 Affinity-attestation issuer compromise

- **Detection:** Per-issuer key-usage audit; tenant-IdP-side notification; classifier of forged attestations
- **Blast radius:** High — affected community membership becomes untrustworthy for the affected affinity scope
- **Recovery:** `runbooks/affinity-attestation-key-rotation.md` + tenant-IdP renegotiation; existing attestation bindings revoked
- **Prevention:** Per-issuer trust anchor pinning; DID-method check; periodic issuer-key health-check
- **Runbook:** `runbooks/affinity-attestation-key-rotation.md`

### F-03 DB join executed without legal-process Cedar

- **Detection:** Prometheus `oya_anonymous_db_join_without_legal_process_total > 0`; audit-chain reconciliation
- **Blast radius:** Critical — I1 anonymity defeated for affected rows
- **Recovery:** P0 incident (see `incident-response.md`); preserve evidence; notify regulator + users per pack
- **Prevention:** DB GRANT separation between `anonymous_post_writer` role and `anonymous_identity_reader` role; only `legal_process_disclosure_view` joins, with Cedar gate; LEAN lane `oya-check-blinding-column-isolation`
- **Runbook:** `runbooks/anonymity-leak-incident-response.md`

### F-04 Abuse-classifier rolls out with regression

- **Detection:** Golden-set eval failure or per-tenant feedback loop indicates verdict-quality regression; specifically over-aggressive auto-hide on minority-affinity content
- **Blast radius:** High — content moderation correctness; user appeals backlog
- **Recovery:** `runbooks/abuse-classifier-rollback.md` — restore previous model version + restore auto-hidden content + apologise notification
- **Prevention:** Per-release golden-set eval; 4/5-rule disparity audit; canary rollout (5% → 20% → 100%)
- **Runbook:** `runbooks/abuse-classifier-rollback.md`

### F-05 Hard-delete tombstone corruption

- **Detection:** SLO `hard-delete-propagation-correctness` < 100%; audit-chain tombstone-seal anomaly
- **Blast radius:** High — I3 violated; user's delete request not honoured across all read paths (regulatory exposure under GDPR Art. 17 + KR PIPA Art. 21)
- **Recovery:** `runbooks/hard-delete-tombstone-corruption.md` — manual sweep + replay; identify slipped replicas; force-purge
- **Prevention:** Two-phase commit for delete (mark tombstone + propagate); cross-replica validator; per-replica delete-ack within 5s budget
- **Runbook:** `runbooks/hard-delete-tombstone-corruption.md`

### F-06 Geo-affinity cluster rebalance fails

- **Detection:** k-anonymity floor approached; community cardinality drops below k=50 (geo) / k=20 (employer) / k=10 (small employer)
- **Blast radius:** Medium — small-affinity members at risk of de-anonymization through small-population correlation
- **Recovery:** `runbooks/geo-affinity-cluster-rebalance.md` — merge with adjacent affinity + tenant notification + user UX advisory
- **Prevention:** Continuous monitoring; auto-merge below threshold; tenant alerted before action
- **Runbook:** `runbooks/geo-affinity-cluster-rebalance.md`

### F-07 Employer-affinity employer-domain takeover

- **Detection:** Employer-domain ownership change (acquisition / sale / rename); IdP renegotiation request
- **Blast radius:** Medium — old employer's affinity scope unclear; new owner has different IdP trust anchor
- **Recovery:** `runbooks/employer-affinity-employer-domain-takeover.md` — old-affinity tombstone + new-affinity migration with end-user re-attestation
- **Prevention:** Periodic IdP attestation refresh; tenant-admin notification at IdP-change time
- **Runbook:** `runbooks/employer-affinity-employer-domain-takeover.md`

### F-08 Anonymity-leak incident (general)

- **Detection:** External report (responsible disclosure / user / bug bounty), DB JOIN anomaly, log/metric anomaly with user_id field, third-party tracker in client bundle
- **Blast radius:** P0
- **Recovery:** Full `runbooks/anonymity-leak-incident-response.md` flow
- **Prevention:** 13 LEAN lanes; threat-model T-I-01 ... T-I-13 mitigations
- **Runbook:** `runbooks/anonymity-leak-incident-response.md`

### F-09 Postgres logical replication slot lag exceeds SLO

- **Detection:** Standard Postgres replication-slot lag metric
- **Blast radius:** Medium — hard-delete propagation slips; possible RPO breach
- **Recovery:** Drain slot + replay; if persistent, fail-over to alternate replica
- **Prevention:** Slot count provisioning + replication monitoring + per-slot back-pressure
- **Runbook:** (inherited from cloud-db runbooks; cross-reference)

### F-10 Redis cluster split-brain

- **Detection:** Sentinel split-brain alert; vote-count divergence between nodes
- **Blast radius:** Medium — vote-counts temporarily inconsistent
- **Recovery:** Reconcile via Postgres source-of-truth; reset Redis counters
- **Prevention:** Sentinel quorum config; multi-AZ
- **Runbook:** (inherited from cloud-redis runbooks)

### F-11 Meilisearch index drift

- **Detection:** Search hit-rate anomaly; hashtag-corpus completeness check
- **Blast radius:** Low — search degradation only
- **Recovery:** Rebuild from Postgres source-of-truth
- **Prevention:** Periodic reconciliation worker
- **Runbook:** (inherited)

### F-12 Foundry-runtime classifier endpoint unavailable

- **Detection:** Classifier RPC error rate > 5%; circuit-breaker open
- **Blast radius:** Medium — auto-moderation suspended; abuse reports route to manual reviewer
- **Recovery:** Fail-open to manual review queue; restore foundry-runtime endpoint
- **Prevention:** Multi-instance foundry-runtime cluster; circuit-breaker per classifier client

### F-13 OPSWAT MetaDefender unavailable (T2 attachments only)

- **Detection:** OPSWAT scan error rate > 5%
- **Blast radius:** Low — fallback to ClamAV; if both fail, T2 attachment uploads rejected
- **Recovery:** Restore OPSWAT or fail-over to ClamAV
- **Prevention:** ClamAV fallback + monitoring

### F-14 NCMEC CyberTipline reporting endpoint unavailable

- **Detection:** Reporter queue depth growing; NCMEC API error rate
- **Blast radius:** Medium — 48h SLA at risk
- **Recovery:** Retry with backoff; manual file by ops-security if endpoint extended outage
- **Prevention:** Reporter-queue durability; multi-channel reporting

### F-15 Legal-process disclosure transparency-report skipped quarter

- **Detection:** Quarter cutoff date passed without report publish
- **Blast radius:** Medium — regulatory exposure (EU DSA Art. 24 transparency reports); tenant trust signal
- **Recovery:** Supplementary report published with note; lookback inclusion in next regular report
- **Prevention:** Calendar reminders + reporter-job CI

### F-16 Affinity attestation cache poisoning

- **Detection:** Verify-cache hit returns stale or invalid attestation
- **Blast radius:** Low (cache; refresh available)
- **Recovery:** Cache invalidation + re-verification
- **Prevention:** Cache TTL bounds + cryptographic-proof binding in cache key

### F-17 Cross-µservice ontology read on `Person` accidentally added

- **Detection:** LEAN lane `oya-check-ontology-person-write-refused` (also covers reads of Person — extended); runtime metric `oya_anonymous_ontology_person_read_total > 0`
- **Blast radius:** Critical (would defeat I1)
- **Recovery:** Revert PR; redeploy
- **Prevention:** LEAN lane mandatory at PR-1

### F-18 Third-party tracker accidentally bundled

- **Detection:** SBOM scan finds known-tracker fingerprint; LEAN lane catch at PR
- **Blast radius:** Critical (would defeat I4)
- **Recovery:** Revert + emergency hotfix; if shipped, immediate client-bundle revoke + force-update
- **Prevention:** LEAN lane `oya-check-third-party-tracker-refused`; allowlist of approved deps

### F-19 MLS protocol error on anonymous-DM

- **Detection:** Server logs MLS handshake failure; client retries fail
- **Blast radius:** Low (DM-only; per-group)
- **Recovery:** Client-driven re-handshake; per-group reset
- **Prevention:** MLS library pinned; per-release conformance tests

### F-20 Tenant-side policy update extends retention beyond pack ceiling

- **Detection:** Policy-validation rejects; tenant gets error
- **Blast radius:** None (policy enforced as code)
- **Recovery:** Tenant chooses within-ceiling tier
- **Prevention:** Pack ceiling hard-coded; tenant override forbidden
