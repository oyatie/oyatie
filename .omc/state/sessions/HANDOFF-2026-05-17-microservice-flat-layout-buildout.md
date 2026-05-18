# Session handoff — 2026-05-17 — microservice flat layout buildout

## What's done

**1,515 audit-grade artifacts** across 17 µservices + foundation layer + scope-out for industry-best-practice sweep. All conform to ADR-0131 per-microservice flat layout + ADR-0130 SLO gate + ADR-0132 no-suite policy + ADR-0133 industry-best-practice conformance.

### Foundation layer (88 artifacts; this session originated these)
| Artifact | Path | Purpose |
|---|---|---|
| ADR-0130 | `docs/decisions/ADR-0130-agentic-slo-gated-promotion.md` | Agentic SLO-gated promotion (Layer-A OSS Grafana stack + Layer-B oyatie engine) |
| ADR-0131 | `docs/decisions/ADR-0131-per-microservice-flat-layout.md` | Universal per-microservice flat layout + 30 migration IPs + cost quantification |
| ADR-0132 | `docs/decisions/ADR-0132-product-suite-and-bundle-dissolution.md` | No-suite forward-policy (no industry/vertical bundle µservices) |
| ADR-0133 | `docs/decisions/ADR-0133-industry-best-practice-conformance-program.md` | 6-axis continuous conformance program |
| `/specs/agentic-slo-gated-promotion.json` | machine-readable SLO gate contract + Mimir multi-tenancy spec |
| `/specs/per-microservice-flat-layout.json` | machine-readable layout convention + validator rules |
| `/specs/microservice-migration-tooling.json` | `oya dev migrate-microservice` CLI spec |
| `/specs/industry-best-practice-conformance.json` | machine-readable axis-finding schema |
| `docs/standards/observability-slo.md` | LTS-pinning matrix (Grafana stack 12.0/3.3/3.5/3.0/1.12/1.6, Rust 1.95, OpenSLO v1.0, K8s 1.32, Istio 1.24) |
| `docs/standards/agentic-dev-team-optimization.md` | 8 principles (ChangeSet semantic claim, parallel-safe, idempotent, audit-chain seals, fail-closed, smallest-actionable, no-blanket-sed, no-deeper-hole) |
| `microservices/observability/*` | First-authored µservice pack (~70 artifacts; reference template) |
| `tasks/plan.md`, `tasks/todo.md`, `docs/ideas/agentic-slo-gated-promotion.md`, `docs/ideas/hyperscaler-gap-closure-plan.md` | Plan + idea-refine docs |
| `CLAUDE.md` | Cross-cutting reference to new substrate added |

### Tier A — substrate µservices (657 artifacts, 7 packs)
| µservice | Artifacts | Headline |
|---|---|---|
| observability | ~70 | SLO engine + OpenSLO + Grafana stack + automated rollback |
| tenancy | 73 | RLS + JWT + 99.99% availability SLO; 35 crates |
| ontology | 78 | Palantir-class typed-entity substrate; 13 BCs; ADR-0107 50ms read |
| audit-chain | 96 | Bominal ADR-0028 Merkle/Ed25519 substrate; per-pack chain locality |
| cell | 100 | Cell-boundary defence-in-depth (RLS+Cedar+SPIFFE+LEAN); ADR-0009 inheritance |
| governance | 99 | Bundles ~50 oya-check-* crates; runs ADR-0133 conformance program |
| workflow-engine | 106 | Temporal-class durable execution; 47 crates |
| application | 105 | Application Shell + Leptos WASM frontend; TTI ≤ 2s |

### Tier B — Foundry split (493 artifacts, 6 packs)
| µservice | Artifacts | Headline |
|---|---|---|
| foundry-runtime | 98 | Agent runtime + capability execution; 5 BCs; 35 crates |
| foundry-supervisor | 104 | Control plane + kill-switch (p99 ≤ 1s); 46 crates |
| foundry-eval | 74 | Eval harness per ADR-0024; deterministic replay |
| foundry-evidence | 71 | Evidence frontend bridging to audit-chain; 6 framework profiles |
| foundry-guardrails | 71 | Safety + policy enforcement; OWASP LLM Top 10 + MITRE ATLAS |
| foundry-providers | 75 | Multi-provider adapters (Anthropic/OpenAI/Gemini API+subscription + in-house) + OpenBao SecretReference |

### Tier C — Cloud split (277 artifacts, 3 packs)
| µservice | Artifacts | Headline |
|---|---|---|
| cloud-iac | 106 | Meta-IaC (ArgoCD/Flux/OpenTofu/Helm/Kustomize); SLSA L3 |
| cloud-k8s | 76 | On-prem k8s per ADR-0121 (kubeadm 1.35+containerd 2.3+Istio 1.29+Envoy+Cilium); CIS Benchmark |
| cloud-secrets | 95 | OpenBao 2.x LTS + HSM; SecretReference contract enforced |

## What's blocked (agent budget exhausted)

Tier D + Tier E partial dispatch returned "You've hit your limit · resets 9pm America/New_York" with zero work done:
- **workflow-studio** (Tier D; visual editor product)
- **mail / messenger / calendar / community** (Tier E first batch)

5 agents queued, all paused. Re-dispatch after budget reset.

## What's queued (not yet dispatched)

### Tier E remainder (Connect / Workspace dissolution, 11 µservices)
Per ADR-0132 + parallel-session ADR-0126 Connect super-app expansion:
- Connect surfaces: `social`, `shorts`, `network`, `anonymous` (from parallel ADR-0126; per `feedback_connect_super_app`)
- Workspace surfaces: `docs`, `sheets`, `slides`, `drive`, `meet`, `forms`, `sites`, `tasks`, `notes`, `translate`, `recordings`

### Tier F — Enterprise unbundle (3 µservices)
`hr`, `payroll`, `accounting` — existing PRDs at `docs/prds/{hr,payroll,accounting}.md`; migrate via `oya dev migrate-microservice`.

### Tier G — Industry-vertical unbundles (forward policy per ADR-0132)
Per Task #19 enumeration — activate when each vertical's first tenant signals onboarding:
- healthcare → {ehr, imaging-dicom, lab-integration, prescriptions, clinical-decision-support, telemedicine, medical-billing, patient-portal, genomics}
- fintech → {payments, lending, wallets, kyc-aml, fraud-detection, treasury, tax, financial-reporting}
- GRC → {risk-register, compliance-evidence, policy-enforcement, audit-prep}
- ATS → {job-postings, applicant-pipeline, interviews, offers, onboarding-flow}
- procurement → {vendor-registry, purchase-orders, contract-management, supplier-evaluation, invoicing-receipts}
- retail → {catalog, orders, inventory, loyalty, returns}
- manufacturing → {bill-of-materials, production-planning, iot-sensor-ingest, predictive-maintenance, quality-control}
- legal → {contracts, case-management, ediscovery, matter-management}
- education → {courses, grading, student-records, attendance, lms-content}
- supply-chain → {shipments, warehouse-mgmt, logistics-routing, demand-forecasting}

### Industry-best-practice + hyperscaler-grade sweep (Task #15; foundation shipped; per-axis execution queued)
ADR-0133 program shipped; per-axis remediation IPs queued. Estimated ~36 audit IPs over ~5 working days with parallel-safe DAG.

## How to resume

**Dispatch cadence: 3 agents per batch, not 6+.** Empirically learned this session: 6-agent parallel batches exhaust agent budget before Tier C completes. 3-at-a-time pace keeps the budget sustainable across an entire workday + leaves headroom for the parent session to dispatch the next batch when current batch returns.

1. **Wait for agent budget reset** (4pm or 9pm ET cycle).
2. **Re-dispatch in batches of 3:**
   - **Batch 1 (Tier D + first 2 of Tier E):** `workflow-studio`, `mail`, `messenger`
   - **Batch 2 (Tier E continued):** `calendar`, `community`, `social`
   - **Batch 3 (Tier E continued):** `shorts`, `network`, `anonymous` (per parallel ADR-0126 Connect super-app expansion)
   - **Batch 4 (Workspace 1):** `docs`, `sheets`, `slides`
   - **Batch 5 (Workspace 2):** `drive`, `meet`, `forms`
   - **Batch 6 (Workspace 3):** `sites`, `tasks`, `notes`
   - **Batch 7 (Workspace 4):** `translate`, `recordings`, plus first Tier F migration
   - **Batch 8 (Tier F Enterprise):** `hr`, `payroll`, `accounting` (existing PRDs at `docs/prds/{hr,payroll,accounting}.md` — these are MIGRATIONS, use `oya dev migrate-microservice` per `/specs/microservice-migration-tooling.json`)
   - **Batch 9+ (Tier G verticals):** healthcare/fintech/GRC/etc. — activate when first tenant per vertical signals onboarding (per ADR-0132 forward policy)
3. **Each batch:** wait for all 3 agents to return before dispatching the next batch. Audit-grade depth per the observability template; cross-reference Tier A/B/C packs as additional examples.
4. **Tier F enterprise migration** uses `oya dev migrate-microservice` per `/specs/microservice-migration-tooling.json` (the CLI spec — implementation lives at `crates/oya-dev-cli/src/commands/migrate_microservice.rs` per ADR-0131 follow-up IP).

### Dispatch brief template per µservice

```
Author the full audit-grade artifact pack for the **<ms>** µservice in oyatie.
Mirror microservices/observability/* depth (~70 artifacts).

CWD: /Users/jasonlee/oyatie
Output target: ~70 artifacts under microservices/<ms>/ (scaffold per ADR-0131).

REFERENCE microservices/observability/* for shape — read PRD/PHASE-01/threat-model/
dpia/compliance/policy/*/runbooks/*/contracts/*/capabilities/*/dashboards/*/
IP-001..IP-015/catalog/*.yaml/iac/*.

EXISTING ARTIFACTS:
- <list existing files for the µservice; e.g., docs/prds/<ms>.md, /specs/products/<ms>.json>
- <relevant Bominal ADR inheritance>

<MS>-SPECIFIC SCOPE:
- <one-paragraph scope statement>
- Primary BCs: <list>
- Performance: <targets>
- Layer-A substrate: <Helm-deployable stack>
- Cross-µservice: <integration points>
- Competitor benchmark: <named competitors>

CONVENTIONS: ADR-0131 + ADR-0056 + ADR-0105 + ADR-0106 + ADR-0130 + ADR-0132 +
ADR-0133 + docs/standards/observability-slo.md + docs/standards/agentic-dev-team-optimization.md.

DELIVERABLES (full pack ~70): <enumerate per observability template — PRD + PHASE +
threat-model + DPIA + 6 policy + 6 runbooks + 3 contracts + 3 capabilities +
3 dashboards + cost-budget + failure-modes + capacity-model + compliance +
multi-region + incident-response + backfill-replay + sdk-plan +
competitor-parity-matrix + 15 IPs + ~11 catalog records + 8 IaC>.

QUALITY BAR: audit-grade; concrete legal citations per pack; Cedar v4 + default-deny;
LTS pins; no blanket-sed; no empty stubs.

DO NOT: touch sibling µservices; move existing crates physically.

When complete: report file count + 1-paragraph summary + open questions.
```

Fill `<ms>` + sections; dispatch via `Agent` tool with `subagent_type: general-purpose`.

## Conventions locked this session

- All new µservices: `microservices/<ms>/` with `src/` as canonical code root; `src/crates/oya-<ms>-<bc>-<layer>/` per ADR-0056 BNF v4.1 + ADR-0105 13-layer enum + ADR-0106 usecase rename.
- All OpenAPI: 3.2.0; AsyncAPI: 3.1.0; proto3.
- All Cedar fragments: v4.2 LTS + default-deny + defence-in-depth FORBID.
- All audit-grade artifacts cite named industry source (per ADR-0133 axis-4).
- Versions pinned per `docs/standards/observability-slo.md` §"Version Pinning".
- Reserved Mimir tenants: `oya-ci`, `oya-self`, `oya-aggregate`; X-Scope-OrgID = `tenant:sha256(canonical_tenant_id ++ deployment_salt)[..16]`.
- 4 Cedar policy fragment files per µservice: `tenant-scope`, `ci-scope`, `auditor-scope`, `public-read`.
- 6 runbook files per µservice (operational-posture-driven).
- 3 contracts per µservice: openapi + asyncapi + proto.
- 3 capability records per µservice (autonomy tiers T0-T3).
- 3 Grafana dashboards per µservice.
- ~11-15 catalog records per µservice (one per BNF v4.1 crate; backend-qualified `*-adapter-<backend>` per ADR-0105 Amendment 3).
- 15 IPs per µservice (IP-001 IaC → IP-015 HG-<MS> registration + branch-protection).
- Per-pack overlay sections in threat-model + DPIA + compliance + data-residency + multi-region for all 11 packs (pack-kr/eu/us/us-healthcare/jp/sg/au/in/br/ae/ksa).
- Concrete legal citations per pack (GDPR Arts. 5/6/9/13/14/17/22/25/28/30/32/33/35/44; KR PIPA Arts. 3/15/17/18/22-2/23/23-2/24/25/28/29/29-2/33/34; HIPAA 45 CFR §164.308-§164.530; APPI Arts. 17/18/20/21/23/24/26-2/27; PDPA SG + AU; DPDPA 2023 §6-10; LGPD Arts. 6/7/11/14/18/33/38/46/48; UAE PDPL 45/2021; KSA PDPL M/19/2021; EU AI Act 2024/1689 Arts. 9-15+26+50+73; eIDAS 910/2014; NIS2 2022/2555; DORA 2022/2554; SLSA L3; NIST SSDF SP 800-218; SOC 2 Type 2; ISO 27001:2022 Annex A; OWASP ASVS v4; CIS Kubernetes Benchmark).
- No raw secrets anywhere; OpenBao SecretReference `${openbao:secret/<path>}` is the canonical path.

## Open questions surfaced (deferred follow-ups)

1. Connect umbrella PRD + ADR-0126 parallel-session integration — when does the `connect` µservice folder itself retire? Likely after all 8 sub-µservices land.
2. Healthcare/fintech/etc verticals — first-tenant trigger procedure not yet codified.
3. Multi-cluster federation across packs — deferred to post-M01 ADR per individual pack contracts.
4. The `oya dev migrate-microservice` CLI implementation crate — spec'd at `/specs/microservice-migration-tooling.json` but crate not yet authored.
5. Tier D workflow-studio + Tier E first-batch (mail/messenger/calendar/community) — agent dispatches queued; resume after budget reset.

## Commits + PRs from this session

(See `git log` for the commit history landed by the next step — this handoff written prior to commit.)

---

**Resumption checklist for next session:**
- [ ] Read this handoff
- [ ] Confirm agent budget restored
- [ ] Re-dispatch 5 queued briefs (Tier D + Tier E first batch)
- [ ] Author Tier E remainder + Tier F migrations
- [ ] Activate `oya dev migrate-microservice` CLI implementation
- [ ] Begin ADR-0133 per-axis audit IP authoring under microservices/governance/
