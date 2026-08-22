---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud-iac + ops-security
deciders: council-architecture, ops-security, axis-cloud-iac, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154 + SLSA L3
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
review_cadence: quarterly + on every Layer-A or Layer-B architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28, A.8.32"
  - "GDPR Arts. 5, 6, 25, 28, 30, 32, 33"
  - "OpenSSF SLSA L3 (build-provenance + attestation)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.5/2.7/2.11/2.12", "KR PIPA Arts. 24/29 (technical safeguards on system administration)", "KR 전자문서법 Arts. 5/6 (electronic-document integrity)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(3)(i) (workforce security)", "§164.310(a)(2)(iii) (access control)", "§164.312(a)(2)(i) (unique user ID)", "§164.312(c)(1) (integrity)", "§164.312(d) (person/entity authentication)"]
  pack-eu: ["GDPR Art. 25 + 32 (privacy-by-design + security of processing)", "NIS2 2022/2555 (when in scope)", "eIDAS 910/2014 (advanced electronic signatures on attestation)"]
  pack-jp: ["APPI Art. 20 (security control measures)"]
  pack-sg: ["PDPA §11–12 (Protection Obligation)", "MAS-TRM v2021 §11–12 (Technology Risk Management)"]
  pack-au: ["Privacy Act 1988 APP 11 (security of personal information)", "APRA-CPS 234 §29–44"]
  pack-in: ["DPDPA 2023 §8 (reasonable security safeguards)", "RBI Master Direction on Outsourcing IT 2023"]
  pack-br: ["LGPD Arts. 46–49 (security and best practices)", "BACEN Res. 4.893/2021 (when in scope)"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 9 (technical measures)"]
  pack-ksa: ["KSA PDPL Royal Decree M/19/2021 Art. 4 + SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: cloud-iac µservice

## Purpose

Identify, classify, and mitigate threats to the cloud-iac µservice's confidentiality, integrity, availability, and supply-chain posture. Cloud-iac is the meta-IaC pipeline — every other µservice's apply flows through it; a compromise here cascades to every cluster oyatie operates. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, and SLSA L3 attestation auditors at first-tenant onboarding.

## Scope

### In-scope

All components introduced by PRD-cloud-iac + ADR-0131 for the cloud-iac µservice, deployed in a **dedicated cloud-iac control-plane Kubernetes cluster** (decision confirmed 2026-05-17; matches hyperscaler practice — Spacelift / Env0 / Terraform Cloud each run their orchestration on dedicated infra):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| ArgoCD (GitOps reconciler) | `cloud-iac-iac-renderer-*` (12 crates) |
| Flux (alt GitOps; supported for tenant choice via adapter) | `cloud-iac-iac-validator-*` (8 crates) |
| OpenTofu (self-hosted Terraform-compatible) | `cloud-iac-iac-applier-*` (9 crates) |
| Helm-controller (k8s-native helm releases) | `cloud-iac-iac-rollback-*` (8 crates) |
| Kustomize-controller (k8s-native kustomize) | `cloud-iac-iac-registry-*` (10 crates) |
| Postgres (iac-state-index store; per-pack) | iac manifests at `microservices/<ms>/iac/{helm,terraform,kustomize}/` |
| Sigstore Cosign / Fulcio (chart-signing) | iac-state-index schema (apply ledger) |
| SLSA L3 attestation verifier | Cedar policies under `microservices/cloud-iac/policy/` |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS layer — owned by `cloud-k8s` µservice's threat model.
- Threats to per-µservice workload code itself (tenancy, ontology, workflow, …) — each owns its own threat-model.md.
- Threats to OpenBao secret-manager — owned by `cloud-secrets` µservice's threat model. cloud-iac inherits OpenBao threats as upstream.
- Threats to the SLO gate decision authority — owned by `observability` µservice's threat model. cloud-iac is a downstream consumer.
- Threats to GitHub Actions runners — owned by `governance` µservice's threat model.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│   Tenant operators                Customer applications                    │
│         │                                  │                               │
│         │ (HTTPS, OIDC, mTLS)              │ (per-tenant API key)          │
│         ▼                                  ▼                               │
│  ┌─ Public ingress (Envoy/Istio gateway) ──────────────────────────────┐   │
│  │  - TLS termination                                                  │   │
│  │  - WAF (rate-limit + OWASP CRS)                                     │   │
│  │  - DDOS protection (provider-level + Cloudflare)                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Dedicated cloud-iac control-plane cluster ────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress                              │
│                                                                            │
│  ┌─ iac-renderer-rest / iac-registry-rest ───┐    ┌─ ArgoCD UI ─────┐      │
│  │  OIDC tenant-scoped reads                 │    │  SSO + RBAC     │      │
│  └───────────────────────────────────────────┘    └─────────────────┘      │
│             │                                                              │
│  Trust boundary 2: Per-µservice apply-scope (Cedar policy)                 │
│             │                                                              │
│  ┌─ iac-applier-worker ───────────────────────────────────────────────┐    │
│  │  - apply scope = declared µservice ONLY                            │    │
│  │  - cross-µservice apply forbidden (default)                        │    │
│  │  - Cedar policy fragment iac-isolation.md                          │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│             │                                                              │
│  Trust boundary 3: cloud-iac → workload-cluster (Kubernetes API)           │
│             │                                                              │
│  ┌─ ApplyEventEmitter / ClusterMutator ───────────────────────────────┐    │
│  │  - mTLS to workload-cluster apiserver                              │    │
│  │  - per-cluster RBAC: applier SA has namespace-scoped admin within  │    │
│  │    declared µservice namespaces only                               │    │
│  │  - kubeconfig per pack issued by OpenBao with 24h rotation         │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
│  Trust boundary 4: cloud-iac → OpenTofu state buckets                      │
│             │                                                              │
│  ┌─ OpenTofu state in pack-pinned S3-compatible object storage ───────┐    │
│  │  - SSE-KMS at rest (per-pack KMS keyring)                          │    │
│  │  - state-lock via Postgres advisory locks                          │    │
│  │  - state versioning + immutability (bucket versioning enabled)     │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
│  Trust boundary 5: cloud-iac → chart-signing verifier                      │
│             │                                                              │
│  ┌─ Sigstore Cosign / Fulcio / Rekor ─────────────────────────────────┐    │
│  │  - chart digest signature verification (Cosign verify)             │    │
│  │  - SLSA L3 attestation verification (in-toto/Witness)              │    │
│  │  - public-transparency log read (Rekor)                            │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
│  Trust boundary 6: cloud-iac → audit-chain (event seal)                    │
│             │                                                              │
│  ┌─ Every Apply* / Render* / DriftDetected event sealed Ed25519 ──────┐    │
│  │  - per Bominal ADR-0028                                            │    │
│  │  - audit-chain emission required pre-mutation                      │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS).
2. **Per-µservice apply-scope** (Cedar policy; load-bearing isolation between µservices).
3. **cloud-iac → workload-cluster** (mTLS, namespace-scoped RBAC, kubeconfig per pack).
4. **cloud-iac → OpenTofu state buckets** (KMS at rest; state-lock; bucket immutability).
5. **cloud-iac → chart-signing verifier** (Cosign + SLSA L3 + Rekor transparency log).
6. **cloud-iac → audit-chain** (Ed25519 seal mandatory pre-mutation).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| IaC manifest text (Helm charts, OpenTofu modules, Kustomize overlays) | `INTERNAL_ONLY` | Medium | git history append-only | git + `microservices/<ms>/iac/` |
| Rendered manifest output (post-template-resolve) | `INTERNAL_ONLY` + transient | Low | not persisted; content-addressable digest stored in registry | iac-registry (digest only) |
| Apply state index (per-µservice, per-pack, per-env current-SHA + applied-at) | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | append-only Postgres; backed up to S3 per pack; ≥6y for HIPAA pack, ≥3y for KR pack, ≥2y universal | Postgres + S3 |
| Terraform/OpenTofu state files | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `SECRET` (state can contain secret values; we redact + encrypt) | High | per-pack object storage; SSE-KMS; versioned | S3 per pack |
| Chart digest + signature records | `AUDIT` | High | indefinite append-only | iac-registry + Rekor public log |
| SLSA L3 attestation (in-toto / Witness) | `AUDIT` | High | indefinite append-only | iac-registry + Rekor |
| Apply / Render / Rollback / Drift events | `AUDIT` | High | audit-chain ≥1y + Mimir-side metric retention | audit-chain µservice |
| Cluster kubeconfigs (per pack) | `SECRET` | Critical | OpenBao with 24h rotation | OpenBao |
| ArgoCD admin tokens | `SECRET` | Critical | OpenBao with 24h rotation | OpenBao |
| OpenTofu state encryption keys | `SECRET` | Critical | OpenBao with 90d rotation; HSM-backed where available | OpenBao |
| Cosign chart-signing key (oyatie's own; for charts cloud-iac itself ships) | `SECRET` | Critical | OpenBao with 90d rotation; keyless-when-CI-attested per Sigstore best practice | OpenBao or Fulcio (keyless) |
| Cedar policy fragments | `INTERNAL_ONLY` | Medium | git history | `microservices/cloud-iac/policy/*.cedar` |
| Per-µservice apply ledger (which charts/modules/overlays a µservice declares) | `INTERNAL_ONLY` + `AUDIT` | Medium | append-only Postgres + git mirror | iac-registry |
| Drift reports | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` | Medium | ≥1y | iac-registry + audit-chain |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator (human) | Untrusted external | OIDC + MFA | View own µservice's apply state + drift reports; author IaC via PR review |
| Customer application (machine) | Untrusted external | n/a — cloud-iac is internal substrate; tenants don't directly call cloud-iac REST | n/a |
| oyatie µservice author (human) | Trusted internal | OIDC + MFA + signed commits | Author / modify own µservice's IaC under `microservices/<ms>/iac/` via PR |
| iac-renderer-worker (long-lived service) | Trusted internal | SPIFFE service-account | Read git; invoke Helm/Kustomize/OpenTofu binaries; emit RenderCompleted events |
| iac-validator-worker | Trusted internal | SPIFFE | Plan-preview against live cluster; emit ValidationVerdict |
| iac-applier-worker | Trusted internal | SPIFFE + per-pack kubeconfig (OpenBao-issued, 24h rotation) | Apply manifests to declared µservice namespaces only |
| iac-rollback-worker | Trusted internal | SPIFFE | Coordinate with iac-applier to revert to prior apply state |
| iac-registry-worker | Trusted internal | SPIFFE | Update Postgres iac-state-index; verify SLSA L3 attestations |
| GitOps reconciler (ArgoCD / Flux) | Semi-trusted internal | service-account scoped to declared namespaces | Continuous reconciliation; cluster mutation |
| oyatie CI runner (GitHub Actions) | Semi-trusted internal | `WORKFLOW_PAT` + ephemeral OpenBao token | Run plan-preview lane at PR time; emit chart-signing events |
| Council-architecture / ops-security operators (human) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Admin operations; 2-person rule for cross-µservice apply waivers + state-bucket admin |
| External auditor (SOC 2 / ISO 27001 / etc.) | Read-only external on time-boxed window | OIDC + MFA + JIT short-lived token | Read iac-registry + audit-chain; cannot mutate; cannot pivot |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; assume always present |
| Attacker — targeted (supply-chain-aware, financially or geopolitically motivated) | Untrusted | none | Sophisticated; targets chart-signing / SLSA chain / state-buckets; assume present for prod-tier surfaces |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure IaC, break apply-scope, induce drift (mitigated by PR-review + LEAN gates + Cedar policy) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality + integrity; mitigated by least-privilege + audit-chain + separation-of-duties + 2-person rule |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score; mitigations; owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Attacker submits a chart with a forged signature claiming oyatie / trusted-builder origin**
- Asset: chart-signing verifier (Sigstore Cosign + SLSA L3 attestation chain)
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Cosign verify against Fulcio root + Rekor transparency-log inclusion proof; mismatch returns refusal pre-apply.
  - SLSA L3 attestation chain (in-toto / Witness) verified pre-apply; builder-id check against allowlist of CI runners.
  - Cosign keyless signing where possible (per Sigstore best practice) — eliminates long-lived signing keys as an attack surface.
  - Penetration test: synthesize a forged signature each release cycle; assert refusal.
- Owner: ops-security + axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.7, A.5.17, A.8.7, A.8.28; GDPR Art. 32(1)(b); SLSA L3 Provenance

**T-S-02 — Attacker impersonates iac-applier-worker to write false apply-state index entries**
- Asset: iac-state-index Postgres
- Likelihood: M / Impact: H (could cause cloud-iac to think a bad apply succeeded; downstream consumers trust the index) / Risk: **H**
- Mitigations:
  - iac-state-index Postgres write authority bound to `spiffe://oyatie/cloud-iac/iac-applier-worker` SPIFFE identity only.
  - Postgres connection mTLS-pinned; client cert is the worker SA.
  - Apply-state-index writes signed Ed25519; signature stored alongside row; query-time signature verification.
  - Pen-test: attempt to insert apply-state row from foreign SA; should fail.
- Owner: axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.15, A.8.3, A.8.7; GDPR Art. 32(1)(b)

**T-S-03 — Attacker impersonates the rollback engine to revert healthy production**
- Asset: iac-rollback path; live cluster state
- Likelihood: L / Impact: H (availability attack masquerading as rollback) / Risk: **M**
- Mitigations:
  - Rollback invocation requires Ed25519 signature from `slo-engine-worker` (observability) OR `iac-rollback-worker` (cloud-iac); both SPIFFE-identity-bound.
  - Rollback PAT/token scoped to `release/*/production` + `iac-state-index` write only; cannot touch other resources.
  - Audit-chain emission required pre-rollback; rollback attribution chained from triggering signal.
  - 2-person rule for manual rollback (non-automated path); JIT elevation via OpenBao.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.2; ISO 27001 A.5.17, A.8.5, A.8.7; GDPR Art. 32(1)(b)(c)

**T-S-04 — Attacker impersonates a workload-cluster apiserver to receive applied resources**
- Asset: cloud-iac → workload-cluster mTLS path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - mTLS pinning per pack; kubeconfig CA bundle is OpenBao-issued and rotated 24h.
  - Workload-cluster TLS cert SAN pinned to per-pack workload-cluster identity.
  - Detection: applies to unexpected apiserver hostnames fail validation.
- Owner: ops-security + cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5, A.8.20, A.8.21

**T-S-05 — Attacker impersonates ArgoCD reconciler to bypass cloud-iac apply-scope checks**
- Asset: ArgoCD admin API
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - ArgoCD admin token issued only to `iac-applier-worker` SPIFFE identity via OpenBao.
  - ArgoCD Application resources include a `oya/scope-attestation` annotation signed by iac-applier; ArgoCD admission webhook rejects unsigned or scope-mismatched Applications.
  - Network policy: only iac-applier-worker pods may reach ArgoCD API server.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.5.15, A.8.3, A.8.20

### Tampering (T)

**T-T-01 — Supply-chain attack on Helm chart at upstream-source (chart-name typosquat, dep-confusion, etc.)**
- Asset: upstream Helm dependencies declared in `microservices/<ms>/iac/helm/<chart>/Chart.yaml`
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - LEAN check `check-helm-chart-allowlist` (NEW) — per-µservice declared dependencies must appear in `/specs/iac-chart-allowlist.json`; non-allowlisted dependencies refused at PR.
  - Cosign verify against publisher's known key for every upstream dep.
  - SLSA L3 attestation required for every chart in the chain; full chain verified pre-apply.
  - Quarterly review of allowlist; expired entries removed.
- Owner: ops-security + axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.8, CC8.1; ISO 27001 A.5.23, A.8.7, A.8.25, A.8.28; GDPR Art. 32(1)(b); SLSA L3

**T-T-02 — Drift undetected (live cluster mutates; cloud-iac fails to reconcile)**
- Asset: live cluster state vs git-declared state
- Likelihood: M / Impact: H (silent drift erodes audit trail + reproducibility; can hide intrusion) / Risk: **H**
- Mitigations:
  - Drift-detector worker runs ≤1h cycle per cluster; emits `DriftDetected` event on any mismatch.
  - Drift-coverage SLO ≥99.5% per cluster per cycle; SLO breach pages on-call.
  - Two-channel drift reporting: Postgres iac-state-index records drift; observability dashboard surfaces; audit-chain seals the event.
  - LEAN check `check-cluster-drift-baseline` (NEW) — periodically asserts every µservice's cluster footprint matches its declared IaC.
- Owner: axis-cloud-iac + ops-sre-reliability
- Residual: M (engineering discipline floor; some drift is legitimate operator action)
- Frameworks: SOC 2 CC4.1, CC7.1, CC7.2; ISO 27001 A.5.7, A.8.15, A.8.16, A.8.32; GDPR Art. 32(1)(b)

**T-T-03 — Apply-elevation escape (apply mutates resource outside µservice scope)**
- Asset: per-µservice apply-scope boundary (Cedar policy)
- Likelihood: M / Impact: H (cross-µservice mutation breaks tenant isolation; can leak data) / Risk: **H**
- Mitigations:
  - Cedar policy `policy/iac-isolation.md` enforces per-µservice apply scope; cross-µservice mutation refused at apply time.
  - LEAN check `check-iac-apply-scope` (NEW) at PR time — validates declared scope vs manifest body.
  - Applier ServiceAccount RBAC namespace-scoped to declared µservice namespaces only.
  - Penetration test: craft an IaC manifest that mutates outside scope; verify refusal at validator + applier + cluster RBAC layers.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.5.15, A.8.3, A.8.4, A.8.32, A.8.33; GDPR Art. 25, Art. 32

**T-T-04 — Terraform/OpenTofu state-store tampering (S3 bucket object replaced)**
- Asset: per-pack OpenTofu state files in S3-compatible object storage
- Likelihood: L / Impact: H (state corruption can cause re-create of every resource, mass destruction risk) / Risk: **H**
- Mitigations:
  - S3 bucket policy: WORM where supported (S3 Object Lock in Compliance mode); state file overwrite requires explicit version increment.
  - SSE-KMS per-pack keyring; key access audit-logged.
  - State file integrity: every state write produces an Ed25519-signed manifest committed to git (state metadata; not full state body); state read verifies signature.
  - State-lock via Postgres advisory locks prevents concurrent mutation.
  - Bucket access via service-account IAM only; no human direct access without ops-security JIT.
- Owner: cloud-secrets + axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.8.11, A.8.12, A.8.24, A.8.25; GDPR Art. 32(1)(a)(b)

**T-T-05 — Rollback replay attack (rollback to N-2 invoked after N-1 already rolled back)**
- Asset: rollback chain history
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Rollback invocation carries monotonic-counter as part of the signed message; replay fails signature-time check.
  - audit-chain seal includes wall-clock and logical-clock; out-of-order rollbacks detected on read.
  - iac-state-index records rollback chain depth; depth > 1 escalates to ExecSponsor.
  - Per-µservice rollback rate limit: ≤3 rollbacks per µservice per 24h; exceeding triggers alert.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC7.2; ISO 27001 A.8.20, A.8.21; GDPR Art. 32(1)(b)

**T-T-06 — Postgres iac-state-index tampering via direct DB access**
- Asset: iac-state-index Postgres
- Likelihood: L (DB access tightly RBAC'd) / Impact: H / Risk: **M**
- Mitigations:
  - Postgres access via service-account mTLS only; human admin access requires OpenBao JIT + 2-person rule for write.
  - Row-level append-only constraint enforced via DB trigger; UPDATE / DELETE refused.
  - Backup to S3 per pack with WORM; restore path documented in runbook.
  - Anomaly detection on write rate (Mimir metric); spike triggers investigation.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.3, A.8.4

### Repudiation (R)

**T-R-01 — Apply executed but actor denies authorship**
- Asset: ApplyExecuted event chain
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every ApplyExecuted event carries `actor=<github-actions-run-id | spiffe-identity>` + Ed25519 audit-chain seal per Bominal ADR-0028.
  - Per-changeset evidence at `microservices/cloud-iac/evidence/multispectrum/<change_id>-<unix_ts>.json` is git-committed; commit signed by author.
  - Apply-state-index row carries actor + signature; chain reconstructable.
- Owner: axis-cloud-iac + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2), Art. 30; pack-eu eIDAS Art. 26 AdES

**T-R-02 — Drift report disputed (whose mutation caused it?)**
- Asset: DriftDetected event
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Drift report includes diff between live state and last-known-good apply; Kubernetes audit-log correlation traces the actor.
  - Cluster audit-logging mandatory (per cloud-k8s µservice's runbook); cloud-iac drift-detector cross-references.
  - DriftDetected event is itself audit-sealed.
- Owner: axis-cloud-iac + cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.15, A.8.16, A.8.32

**T-R-03 — Rollback executed without traceable trigger**
- Asset: Rollback audit-chain
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Every rollback emits `ApplyRolledBack{microservice, from_sha, to_sha, reason, executed_at}`; `reason` is structured enum (`slo_burn_breach | drift_remediation | manual_override | post_mortem_remediation`) and required.
  - Per-changeset evidence regenerated post-rollback.
  - iac-state-index snapshot at rollback-time persisted with explicit retention extension.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC7.4, CC8.1; ISO 27001 A.5.26, A.5.27, A.8.15, A.8.16

### Information Disclosure (I)

**T-I-01 — Terraform/OpenTofu state file leaks secret values (state contains plaintext secrets)**
- Asset: state file content
- Likelihood: M (state files notoriously include secrets if not redacted) / Impact: H / Risk: **H**
- Mitigations:
  - Convention enforced: secrets in IaC declared via OpenBao SecretReference; never inline.
  - State file content scanned at write time for secret patterns; matches block the write + alert.
  - State files encrypted at rest with per-pack KMS; access via service-account mTLS only.
  - State file read via apply-only path; human read requires OpenBao JIT + 2-person rule.
  - Quarterly secret-scan of historical state files.
- Owner: ops-security + cloud-secrets + axis-cloud-iac
- Residual: M (secret hygiene is engineering discipline; never fully eliminated)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.11, A.8.12; GDPR Art. 32(1)(a)(b); SLSA L3 §"build-isolation"

**T-I-02 — Apply log leaks tenant data (e.g., container image-pull failures show tenant-secret values in error message)**
- Asset: Apply event logs
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Apply event log emission redacts known-secret patterns at source.
  - `Secret<T>` Rust type with stripped Debug impl used for any sensitive value flowing through the applier.
  - Loki log ingest applies tenant-scope label; per-tenant logs visible only to that tenant + auditors.
  - LEAN check `check-iac-event-log-redaction` (NEW) at PR time.
- Owner: axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.8.11, A.8.12; GDPR Art. 32(1)(a)(b)

**T-I-03 — Drift report leaks cross-tenant resource details**
- Asset: DriftReport content
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - DriftReport scoped per-µservice; per-tenant scope inherited from µservice.
  - Cedar policy `policy/tenant-scope.cedar` refuses cross-tenant DriftReport reads.
  - Per-tenant dashboard surfaces only own drift reports.
- Owner: axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15; GDPR Art. 32

**T-I-04 — IaC manifest text leaks via public-read on git (e.g., open-source repo includes tenant-specific config)**
- Asset: per-tenant IaC manifest text in git
- Likelihood: M (engineering discipline gap) / Impact: M (could expose internal architecture details) / Risk: **M**
- Mitigations:
  - Repo is private by default; per-tenant overlay files marked `INTERNAL_ONLY` data-class.
  - LEAN check `check-tenant-config-not-in-public-overlay` (NEW) — refuses tenant-bound config in `regional-packs/*` global overlays.
  - Sensitive values referenced via OpenBao; not embedded in manifest text.
  - Open-source-of-charts decision scheduled-for-distinct-tracked-work (per PRD §"SDK Plan" parallel pattern); default closed-source.
- Owner: ops-security
- Residual: L-M
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12; GDPR Art. 32(1)(b)

**T-I-05 — Secret leaked via apply-event log to Mimir / Loki**
- Asset: cloud-iac apply event logs
- Likelihood: M (human-error baseline) / Impact: H (cascades to broad compromise) / Risk: **H**
- Mitigations:
  - Secret-scanner CI lane (`governance-evidence-secret-scan`) scans every commit + log emission for known secret patterns.
  - OTel SDK redactor strips known-secret patterns at emission time.
  - OpenBao SecretReference materialisation never logs the raw secret; `Secret<T>` wrapper.
  - Rotation policy: 24h cluster kubeconfigs; 90d signing keys; rotate-out before leaked secret expires.
  - Secret-leak runbook: detection → immediate rotation → forensic trace → engineering education.
- Owner: ops-security + cloud-secrets
- Residual: M
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32(1)(a)(b)(c)(d)

### Denial of Service (D)

**T-D-01 — Stuck apply (apply-job hangs; queue depth grows; subsequent applies queue up)**
- Asset: iac-applier worker queue
- Likelihood: H (k8s API can hang on resource conflicts, finalizer loops) / Impact: H (every promotion blocked) / Risk: **H**
- Mitigations:
  - Apply timeout = 15min p999 (per PRD §"Performance Targets"); applies exceeding timeout abort + retry-with-backoff.
  - Retry budget per µservice: max 3 retries within 1h; exceeding moves apply to "stuck" state + on-call paged.
  - Stuck-apply recovery runbook (`runbooks/stuck-apply-recovery.md`).
  - HPA on applier-worker; horizontal capacity scales with queue depth.
  - Apply jobs partition by µservice; one µservice's stuck apply does not block others (no shared serial queue).
- Owner: axis-cloud-iac + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Drift cascade (one mutation cascades into a wave of drift events)**
- Asset: drift-detector worker
- Likelihood: M / Impact: M (alert storm if not de-duplicated) / Risk: **M**
- Mitigations:
  - Drift events grouped by µservice + resource-kind; one event per group per cycle.
  - Backpressure: drift queue depth > threshold pauses new drift-detection cycles + alerts on-call.
  - LEAN check `check-drift-cascade-throttle` (NEW) verifies throttle is in place.
- Owner: axis-cloud-iac + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30

**T-D-03 — Registry Postgres corruption (write race / index bloat / advisory-lock deadlock)**
- Asset: iac-state-index Postgres
- Likelihood: L / Impact: H (registry unavailable → no apply possible) / Risk: **M**
- Mitigations:
  - HA Postgres: primary + read-replica per pack.
  - WAL streaming replication + S3 archive (point-in-time recovery to ≤5min).
  - Advisory-lock timeout = 30s; deadlock auto-aborts longer holder; logged.
  - Index bloat monitored; nightly VACUUM ANALYZE.
  - Recovery runbook (`runbooks/registry-restore.md`).
- Owner: axis-cloud-iac + cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.14; GDPR Art. 32(1)(c)

**T-D-04 — State-lock contention (concurrent OpenTofu applies race on the same state)**
- Asset: OpenTofu state-lock Postgres advisory lock
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - State-lock held per (microservice, pack, environment); concurrent applies serialise per tuple.
  - Lock timeout = 10min; held longer aborts apply + emits `iac_state_lock_timeout_total` metric.
  - LEAN check verifies state-lock release path on every applier exit.
  - Recovery runbook (`runbooks/state-lock-break.md`).
- Owner: axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14

**T-D-05 — GitOps reconciler down (ArgoCD or Flux outage)**
- Asset: ArgoCD / Flux reconciler
- Likelihood: M / Impact: H (no automated reconcile; manual intervention required) / Risk: **H**
- Mitigations:
  - HA ArgoCD: minimum 3 replicas; etcd-backed application state.
  - Standby Flux reconciler available as failover (tenant choice; oyatie operates ArgoCD primary).
  - Reconciler self-SLI (per observability ADR-0139) gates the cloud-iac µservice's own promotion.
  - Manual apply path documented in runbook (`runbooks/gitops-reconciler-restart.md`).
- Owner: ops-sre-reliability + axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.14

### Elevation of Privilege (E)

**T-E-01 — iac-applier-worker SA token compromised → arbitrary cluster mutation**
- Asset: applier worker ServiceAccount token
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SA token bound to pod identity; cannot be used outside cluster.
  - Token rotation 24h.
  - Workload-cluster RBAC: applier SA has namespace-scoped admin within declared µservice namespaces ONLY.
  - Network policy: only iac-applier-worker pods may reach workload-cluster apiservers.
  - LEAN check `check-applier-rbac-scope` validates RBAC bindings stay namespace-scoped.
- Owner: ops-security + axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.17, A.8.5, A.8.7

**T-E-02 — Cedar policy escape (crafted manifest field triggers Cedar evaluation bug)**
- Asset: Cedar policy evaluator in iac-validator
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4+ used (no template-based escape vectors known in v3+).
  - Cedar fragments fuzzed at CI time (`check-cedar-fragment-coverage` lane).
  - Field input lengths bounded at REST API; oversized inputs rejected before policy evaluation.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-03 — ArgoCD admin token leaked → arbitrary application creation**
- Asset: ArgoCD admin token
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Admin token issued only to iac-applier-worker SPIFFE identity via OpenBao.
  - Token rotation 24h.
  - ArgoCD admission webhook rejects applications missing `oya/scope-attestation` signed annotation.
  - Network policy: only iac-applier-worker may reach ArgoCD admin API.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.5

**T-E-04 — Cross-µservice apply privilege escalation (apply mutates another µservice's resources via shared resource)**
- Asset: Apply-scope boundary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Cedar policy `iac-isolation.md` enforces per-µservice apply scope; cross-µservice mutation refused.
  - Cluster RBAC enforces namespace-scoped admin; cross-namespace mutation blocked at apiserver.
  - LEAN check `check-iac-apply-scope` validates at PR time.
  - Penetration test quarterly.
- Owner: axis-cloud-iac + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.5.15, A.8.3, A.8.4

**T-E-05 — Operator-level access to OpenTofu state used to mass-destroy resources**
- Asset: OpenTofu state files + apply path
- Likelihood: L (insider-malicious threat) / Impact: H / Risk: **M**
- Mitigations:
  - State file write requires OpenBao JIT elevation + 2-person rule.
  - Mass-destroy patterns (e.g., tofu destroy across many resources) trigger anomaly alert and require ExecSponsor approval.
  - Soft-deletion: terraform destroys mark resources for deletion + 30-day grace; actual delete scheduled-for-distinct-tracked-work unless override.
  - Bucket versioning enables state restore.
- Owner: ops-security + axis-cloud-iac
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4, A.8.16

## LINDDUN Privacy-Threat Catalog

LINDDUN (Linkability / Identifiability / Non-repudiation / Detectability / Disclosure / Unawareness / Non-compliance) covers privacy-specific threats not fully captured by STRIDE.

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Apply state index rows | Cross-µservice query on (tenant, pack, time-range) could link a tenant's deployment cadence to behaviour patterns | Cedar policy scopes reads per tenant; cross-tenant aggregation forbidden in default queries | L |
| T-L-02 | Identifiability | Per-pack OpenTofu state | State file may carry tenant identifiers as resource names | LEAN check on state-content secret-scan also flags raw tenant identifiers; recommendation to hash | L-M |
| T-L-03 | Non-repudiation | µservice author signed commits | Authors may deny IaC changes that lead to undesirable outcomes | Signed commits + PR review log + per-changeset evidence signed | L |
| T-L-04 | Detectability | Apply event timing | Apply-event timing correlates with tenant business events | Reasonable; reflects tenant business reality; auditor disclosure: BEHAVIORAL_TENANT_PRODUCT and expected | M |
| T-L-05 | Disclosure | Auditor read of apply ledger | Auditor scoped to a single tenant could pivot into adjacent tenants via shared apply-ledger views | Auditor JIT tokens scoped at iac-state-index per-tenant filter; folder isolation tested annually | L |
| T-L-06 | Unawareness | Tenant's end-user (the tenant's user, not the tenant itself) | End-user unaware their tenant's IaC deploys cause changes affecting them | Tenant DPA includes upstream disclosure clause per Art. 26 joint-controllership | M |
| T-L-07 | Non-compliance | GDPR Art. 30 (records of processing) | Apply ledger constitutes a record of processing; must be retained per Art. 30 + ROPA register | iac-state-index append-only + audit-chain seal satisfies; ROPA template at `legal/ropa.md` | L |

## Mitigations Catalog (cross-reference)

Cross-cuts STRIDE + LINDDUN. Each mitigation appears in at least one threat row above.

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Cosign verify + SLSA L3 attestation chain | Preventive | ops-security + axis-cloud-iac | `cloud-iac-provenance-slsa-l3` lane |
| Helm chart allowlist per /specs/iac-chart-allowlist.json | Preventive | ops-security | `check-helm-chart-allowlist` lane |
| Cedar policy enforcement of apply-scope | Preventive | axis-cloud-iac | `check-iac-apply-scope` lane + Cedar fuzz |
| Drift detector ≤1h cycle per cluster | Detective | axis-cloud-iac | `cloud-iac-drift-detection-coverage` lane |
| State file secret-pattern scanning | Detective + Preventive | ops-security + cloud-secrets | `check-iac-state-secret-scan` lane |
| Per-pack SSE-KMS for state-storage | Preventive | cloud-secrets | KMS access audit |
| Ed25519 audit-chain seal on every Apply/Render/Rollback/Drift event | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Per-µservice apply rate-limit | Preventive (DoS) | axis-cloud-iac | Mimir metric `iac_apply_rate_total` |
| 2-person rule on state-bucket admin + manual rollback | Preventive (insider) | ops-security | OpenBao JIT elevation logs |
| Network policy: applier → workload-cluster apiservers only | Preventive | ops-sre-reliability | Kubernetes NetworkPolicy review |
| Stuck-apply timeout (15min p999) + bounded retry budget | Preventive (DoS) | axis-cloud-iac | apply-timeout integration test |
| HA Postgres iac-state-index + WAL-archive + PITR | Recovery | cloud-secrets + axis-cloud-iac | DR drill quarterly |
| Soft-deletion (30d grace) on tofu destroy | Recovery | axis-cloud-iac | terraform-destroy anomaly alert |
| Cosign keyless signing (Fulcio + Rekor) | Preventive | ops-security | Sigstore docs |
| LEAN check check-cluster-drift-baseline | Detective | axis-cloud-iac | per-PR lane |

## Residual Risk Acceptance

Residual risks above L (low) require explicit acceptance signed by `council-architecture` + `ops-security` + `council-privacy`:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-T-02 (drift undetected) | M | Engineering discipline floor; legitimate operator action can mimic drift. | Quarterly |
| T-I-01 (state file secret leak) | M | Engineering discipline floor; mitigated to acceptable via scanning + encryption. | Quarterly |
| T-I-05 (secret leak via apply log) | M | Human-error baseline; mitigated to acceptable. | Quarterly |
| T-I-04 (IaC manifest in public overlay) | L-M | Engineering discipline; LEAN check is the load-bearing control. | Quarterly |
| T-L-04 (apply timing detectability) | M | Tenant business reality; consent at onboarding covers. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant-of-tenant responsibility; joint-controllership clause. | Annually |

Sign-off (this document is RW until council sign-off captured):

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

Compliance frameworks engaged: KR-ISMS-P + KR PIPA + KR 전자문서법.

Additional considerations:
- **KR PIPA Art. 24 (resident-registration-number processing)**: not processed by cloud-iac directly; if IaC manifests reference tenant resident-id columns (e.g., for SQL migration scripts), that reference is via OpenBao secret-reference and the value is never materialised in cloud-iac logs.
- **KR PIPA Art. 29 (technical safeguards)**: mitigations T-S-01..T-E-05 map to the 12 prescribed safeguards.
- **KR-ISMS-P §2.5 (인적보안) + §2.7 (접근통제) + §2.11 (위탁관리) + §2.12 (위반관리)**: 2-person rule + JIT elevation + sub-processor list + audit-chain tampering detection map directly.
- **KR 전자문서법 Art. 5 (electronic document integrity)**: audit-chain Ed25519 seal on every Apply* / Rollback* event satisfies integrity-preservation requirement.

### pack-us-healthcare (HIPAA-scoped)

Compliance frameworks engaged: HIPAA + state-level (CCPA / CMIA per state).

Additional considerations:
- **HIPAA §164.308(a)(3)(i) (workforce security)**: applier ServiceAccount RBAC namespace-scoped; humans use JIT + 2-person rule. Background checks + onboarding training per oyatie HR procedures.
- **HIPAA §164.310(a)(2)(iii) (access control)**: cluster API + state bucket access via service-account IAM only; no human direct access without ops-security JIT.
- **HIPAA §164.312(a)(2)(i) (unique user identification)**: SPIFFE identity per worker SA; signed commits per human author.
- **HIPAA §164.312(c)(1) (integrity)**: Ed25519 audit-chain seals + Cosign chart-signing + SLSA L3 attestation.
- **HIPAA §164.312(d) (person/entity authentication)**: OIDC + MFA + SPIFFE.
- **HIPAA §164.316(b)(2) (retention)**: apply audit retention ≥ 6 years for pack-us-healthcare; cost-budget.md reflects.
- **Business Associate Agreement (BAA)**: when a Covered Entity tenant uses oyatie, BAA cascade includes cloud-iac as sub-processor.

### pack-eu

Compliance frameworks engaged: GDPR + EDPB + NIS2 + eIDAS.

Additional considerations:
- **GDPR Art. 25 (privacy-by-design)**: mitigations map to Schrems-II-compatible TOMs.
- **GDPR Art. 32 (security of processing)**: every T-*-NN mitigation contributes.
- **GDPR Art. 44–50 (transfers)**: pack-eu state buckets EU-resident; cross-pack state replication forbidden by default per residency contract.
- **NIS2 (2022/2555)**: when oyatie crosses NIS2 Annex I/II thresholds, the incident-reporting timelines (24h initial + 72h detailed + 1mo final) become mandatory; `incident-response.md` reflects.
- **eIDAS 910/2014 Art. 26 (Advanced Electronic Signature)**: Ed25519 audit-chain seals satisfy AdES requirements for EU-resident apply-ledger records.

### pack-jp

Compliance frameworks engaged: APPI.

Additional considerations:
- **APPI Art. 20 (security control measures)**: mitigations T-S-01..T-E-05 map.
- **APPI Art. 21 (notification on cross-border transfer)**: pack-jp state JP-resident.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlay sections in `regional-packs/<pack>/cloud-iac-overlay.md`; each follows this document's structure mapped to the local PII law + cybersecurity framework.

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC6.x (logical access) ← Cedar + RBAC; CC7.x (system operations) ← runbooks + alerting; CC8.x (change management) ← PR review + ChangeSet; CC9.x (risk mitigation) ← residual-risk acceptance | `microservices/cloud-iac/compliance.md` |
| ISO 27001:2022 | Annex A.5–A.8 controls covered as cited inline in each threat row | `microservices/cloud-iac/compliance.md` |
| GDPR | Arts. 5, 6, 25, 28, 30, 32, 33 cited inline | `microservices/cloud-iac/dpia.md` + `microservices/cloud-iac/compliance.md` |
| OpenSSF SLSA L3 | Build-provenance + attestation chain; per-apply verification | `microservices/cloud-iac/compliance.md` §"SLSA L3" |

## Re-review Triggers

This threat model re-reviews on:

- Any change to the trust boundary diagram above.
- Any Layer-A version upgrade (ArgoCD / Flux / OpenTofu / Helm-controller / Kustomize-controller / Sigstore Cosign / etc.) where upstream release notes mention security fixes.
- Any new pack activation.
- Annual scheduled review (Q2 each year).
- Post-incident review (any Sev-1 or Sev-2 incident in cloud-iac or any µservice it applies for).
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519); inherited.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (data residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/cloud-iac/PRD.md`.
- `microservices/cloud-iac/dpia.md`.
- `microservices/cloud-iac/compliance.md`.
- `microservices/cloud-iac/policy/iac-isolation.md`.
- `microservices/cloud-iac/policy/data-residency.md`.
- `microservices/observability/threat-model.md` (cross-µservice reference).
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023).
- NIST SP 800-154 (Guide to Data-Centric System Threat Modeling).
- OpenSSF SLSA — `slsa.dev/spec/v1.0/`.
- Sigstore Cosign — `docs.sigstore.dev/cosign/`.
- ArgoCD security — `argo-cd.readthedocs.io/en/stable/operator-manual/security/`.
- OpenTofu state encryption — `opentofu.org/docs/language/state/encryption/`.
