# pharmacy

Canonical pharmacy clinical and operational substrate for the Oyatie platform — medication catalog, formulary, ePrescribe, drug-interaction CDS, allergy check, dose range checking, pharmacist verification, compounding, inventory, auto-dispensing cabinets, BCMA, IV admixture, controlled substances, reimbursement, operations, interventions, medication reconciliation, outpatient retail, MTM, and DSCSA.

- **Owner team**: `axis-pharmacy`
- **Authority ADR**: `docs/decisions/ADR-0332-pharmacy-substrate.md`
- **Layout authority**: `docs/decisions/ADR-0131-per-microservice-flat-layout.md`
- **Suite policy**: `docs/decisions/ADR-0132-no-grouping-microservices.md`
- **Inter-microservice**: `docs/decisions/ADR-0145-inter-microservice-direct-grpc.md`
- **Criticality tier**: T0 (patient-safety + DEA-regulated)
- **Top-3 counterparts**: Oracle Health (Cerner) Pharmacy Manager / Epic Willow / BD Pyxis

---

## Why this microservice exists

Pharmacy is single-concern: medication lifecycle from order to administration to settlement. It is its own microservice (not a sub-domain of `emr`) because:

1. **Regulatory surface** — DEA EPCS, DSCSA serialization, USP 795/797/800, NCPDP SCRIPT, and state board of pharmacy registration are non-overlapping with EMR HIPAA + Meaningful Use.
2. **Operational tempo** — sub-second hot path for BCMA scans, DDI evaluation, and dispense decrement at RPS materially higher than EMR encounter writes.
3. **Vendor adapter surface** — Surescripts, NCPDP, Pyxis, Omnicell, Alaris, FDB, Multum, Medi-Span are a distinct integration matrix.
4. **Failure domain** — pharmacy MUST remain available when EMR is degraded (you cannot block administration of an already-ordered medication).
5. **Auditor surface** — state inspectors audit pharmacy ledgers independently of EMR.

See `PRD.md` §1.3 for the full justification.

---

## What this microservice owns

20 bounded contexts:

1. **MedicationCatalog** — NDC + RxNorm + GPI + ATC + UNII; FDB / Multum / Medi-Span ingestion; A/B knowledge-package switching.
2. **Formulary** — preferred / non-preferred / restricted; P&T workflow; therapeutic interchange.
3. **ePrescribe** — Surescripts + NCPDP SCRIPT 2017-071+; EPCS for Schedule II–V.
4. **DrugInteraction** — DDI / DAI / DCI / DPI / DDxI / DLI / DFI / DDoseI; severity stratification; tenant suppression with audit.
5. **AllergyCheck** — exact-ingredient + cross-class; structured override path.
6. **DoseCheck** — weight / BSA / renal / hepatic / age-band; cumulative caps.
7. **Verification** — single vs. dual pharmacist; tall-man lettering.
8. **Compounding** — USP 795/797/800; BUD calculator; environmental monitoring evidence.
9. **Inventory** — par/min/max; lot/expiry; recall sequestration.
10. **AutoDispensing** — Pyxis / Omnicell / Carousel / AcuDose / MedDispense via vendor-neutral adapters.
11. **BCMA** — five-rights verification; nurse override with pharmacist callback.
12. **IVAdmixture** — compounding; smart-pump library (Alaris / Plum 360 / Hospira); DERS hard/soft limits.
13. **ControlledSubstance** — DEA Form 222; perpetual inventory; witnessed waste; EPCS; DEA inspection-ready report.
14. **Reimbursement** — 340B; PBM NCPDP D.0; payer adjudication; copay calc; handoff to `cloud-billing`.
15. **Operations** — order / prep / verify / deliver queues; workload balancing; DUR.
16. **Interventions** — clinical interventions with billable MTM codes.
17. **MedRec** — admission / transfer / discharge med rec.
18. **OutpatientRX** — retail / drive-through / mail / specialty / refills.
19. **MTM** — CMR / TMR / MAP / PML; CPT 99605–99607.
20. **DSCSA** — SGTIN-198 + lot + expiry + serial; T1/T2/T3; saleable returns verification; suspect product investigation.

---

## What this microservice does NOT own

- General patient charting — `emr`.
- Lab interpretation — `diagnostics`.
- Encounter scheduling — `application` calendar.
- Payer enrollment — `crm`.
- Generic billing ledger — `cloud-billing` / `cloud-billing-tax`.
- Staff scheduling — HRIS overlay.
- General supply-chain procurement — `warehouse` / `global-trade`.
- Therapeutic-knowledge authoring — pharmacy ingests; it does not author.

---

## Architecture overview

- **Layering** — 13-layer enum per ADR-0105 (kernel / domain / usecase / adapter / api / rest / sdk / worker / app / check / fit / bench / xtask).
- **Cellular topology** — Per ADR-0248. T0 cells with shuffle sharding width 2; Cloud Hypervisor microVMs in Kata pods.
- **Causality** — HLC default; TrueTime tier opt-in for fin-grade dispense ledger.
- **Protocol** — HTTP/3 + QUIC default; gRPC over HTTP/3.
- **Inter-microservice** — Direct gRPC per ADR-0145; no forced Workflow + Ontology adapter.
- **Stores** — PostgreSQL 16.2 + Citus 12.1; Pulsar 3.2; OpenBao 2.0; MinIO / S3-compat for long-term audit + DSCSA.

See `ARCHITECTURE.md` for the full architecture.

---

## Inter-microservice dependencies

| Depends on | Reason |
|---|---|
| `audit-chain` | Seal every controlled event. |
| `emr` | Patient / allergy / problem / labs / MAR. |
| `identity` | Prescriber DEA, pharmacist license, nurse identity. |
| `cloud-iam` | Cedar policy gates. |
| `cloud-kms` | EPCS DEA-bound signing; audit Merkle keys; PBM mTLS material. |
| `cloud-secrets` | Surescripts mTLS, PBM creds, FDB / Multum keys. |
| `observability` | 12 SLOs + RED + USE. |
| `cloud-billing` | Dispense settlement. |
| `cloud-billing-tax` | State drug taxes (rare). |
| `compliance` | HIPAA + DEA + state-board pack hooks. |
| `governance` | Policy-change pipeline. |
| `intelligence` | MTM PML drafting (T3 with redaction contract). |
| `analytics` | Pharmacy KPI fact streams. |
| `community` | B2C refill notifications. |
| `healthcare-integration` | Inbound HL7v2 / FHIR from outside systems. |
| `comms-email` | Refill reminders. |
| `forms` | REMS attestations / EPCS step-up. |
| `consent-graph` | Specialty hub program data-sharing agreements. |
| `cell` | Cell topology + membership. |
| `tenancy` | Per-tenant context. |

---

## Compliance posture

Pack overlays (per ADR-0251):

- HIPAA (45 CFR §164)
- DEA Controlled Substance (21 CFR §1300–§1321)
- DSCSA (Title II FDASIA 2013)
- USP <795> / <797> / <800>
- 340B (HRSA OPA)
- NCPDP SCRIPT 2017-071+
- Surescripts EHR-vendor accreditation
- 42 CFR Part 2 (substance-use-disorder)
- GDPR / LGPD / PIPL / PIPA
- PCI-DSS (outpatient cardholder data)
- EU-AI-Act (intelligence-substrate calls)
- State board of pharmacy (per state of operation)

---

## Observability

- 12 SLOs registered with `observability`.
- RED + USE metrics for every bounded context.
- W3C Trace Context + B3 multi-header.
- See `slos/` for OpenSLO files.

---

## SLOs (12)

1. ePrescribe round-trip latency — p95 ≤ 5 s.
2. DDI check latency — p99 ≤ 200 ms.
3. Dispense cycle latency — p99 ≤ 2 s.
4. BCMA scan latency — p99 ≤ 100 ms.
5. Dispense availability — ≥ 99.99%.
6. BCMA availability — ≥ 99.99%.
7. ePrescribe availability — ≥ 99.9%.
8. Catalog read availability — ≥ 99.95%.
9. Audit chain coverage completeness — 1.0.
10. Controlled-substance witness integrity — 1.0.
11. 340B classification accuracy — ≥ 99.99%.
12. DSCSA saleable return verification rate — ≥ 99.9%.

---

## Contracts

- `contracts/openapi/pharmacy.yaml` — REST surface; FHIR Medication / MedicationRequest / MedicationDispense / MedicationAdministration / MedicationStatement plus pharmacy-specific extensions.
- `contracts/asyncapi/pharmacy-events.yaml` — Pulsar topic surface (`oya.pharmacy.*`).
- `contracts/proto/pharmacy.proto` — Internal gRPC contracts (20 services).

---

## Cedar policies

- `pharmacist-can-verify.cedar`
- `prescriber-can-eprescribe.cedar`
- `nurse-can-administer.cedar`
- `dea-controlled-2x-verify.cedar`
- `allergy-override-requires-justification.cedar`

Plus extended set in `policies/` (see `ARCHITECTURE.md` §6).

---

## Deployment

Per `feedback_zero_handroll_opentofu_only_2026_05_20`, every context lands via OpenTofu modules. No manual steps. Six deployment contexts under `iac/`:

- `aws-guest/`
- `oci-guest/` (plus `oci-guest/always-free/` for sandbox/demo/trial/dev)
- `on-prem/`
- `colo/`
- `oyatie-cloud/`
- `sovereign/`

OCI Always Free deployment is the default sandbox profile (per `feedback_oci_always_free_maximization_2026_05_20`).

---

## Supported OS matrix

See `supported-oses.json`. Tier-1 blocking in CI:

Talos / RHEL / Oracle Linux / SUSE / Ubuntu / Debian / Rocky / AlmaLinux / CentOS Stream / Amazon Linux / Flatcar / Photon / macOS Apple Silicon M5+.

Tier-2 soft gate: linux/ppc64le, linux/s390x.

Out of scope: Intel macOS, pre-M5 Apple Silicon, FreeBSD, OpenBSD, Windows Server (frontend WinUI 3 only), Solaris.

---

## Implementation plans

10 IPs draft-published under `implementation-plans/`:

- IP-001 — MedicationCatalog kernel + FDB ingest adapter.
- IP-002 — Formulary kernel + P&T workflow + therapeutic interchange.
- IP-003 — ePrescribe usecase + Surescripts adapter + EPCS sign envelope.
- IP-004 — DrugInteraction eight-engine fan-out + severity-bands + tenant suppression.
- IP-005 — AllergyCheck mirror + cross-class derivation + override capture.
- IP-006 — DoseCheck weight/BSA/renal/hepatic/age-band/cumulative.
- IP-007 — Verification + tall-man + dual-verify CII.
- IP-008 — Compounding USP 795/797/800 + BUD + environmental evidence.
- IP-009 — Inventory + recall sequestration + expiry stratification + cabinet adapter contracts.
- IP-010 — Reimbursement 340B + NCPDP D.0 PBM + handoff to cloud-billing.

---

## Counterpart parity

See `competitor-parity-matrix.md`. ≥ 100 capabilities enumerated, covering UNION of Cerner Pharmacy Manager + Epic Willow + BD Pyxis + Omnicell + McKesson EnterpriseRx.

---

## Service-level ADRs

- `decisions/ADR-MS-001-ePrescribe-substrate.md`
- `decisions/ADR-MS-002-controlled-substance-DEA-compliance.md`

---

## Local development

```bash
# Layered build via workspace
cargo build -p oya-pharmacy-medication-catalog-kernel \
            -p oya-pharmacy-medication-catalog-domain \
            -p oya-pharmacy-medication-catalog-usecase \
            -p oya-pharmacy-medication-catalog-adapter \
            -p oya-pharmacy-medication-catalog-api \
            -p oya-pharmacy-medication-catalog-rest

# Run tests for the whole microservice
cargo test --workspace -- pharmacy::
```

---

## Audit chain

Every event listed in `ARCHITECTURE.md` §4.3 is sealed to `audit-chain` with bilateral cross-pointer when crossing tenants (specialty pharmacy hub).

---

## Versioning

- OpenAPI under `contracts/openapi/pharmacy.yaml` versioned `1.0.0`.
- AsyncAPI under `contracts/asyncapi/pharmacy-events.yaml` versioned `1.0.0`.
- Proto under `oya.pharmacy.v1` package.

All breaking changes follow `feedback_no_silent_regression`: ADR + version bump + sunset.

---

## References

- ADR-0105 13-layer enum.
- ADR-0131 per-microservice flat layout.
- ADR-0132 no-grouping microservices.
- ADR-0145 inter-microservice direct gRPC.
- ADR-0248 hyperscaler-shape cellular architecture.
- ADR-0251 compliance-pack primitive.
- ADR-0252 HLC default + TrueTime tier.
- ADR-0253 HTTP/3 + QUIC default.
- ADR-0328 multispectrum review v2.4.0.
- ADR-0332 pharmacy substrate authorization.
- 21 CFR §1300–§1321 (DEA).
- 45 CFR §164 (HIPAA).
- USP <795>, <797>, <800>.
- NCPDP SCRIPT 2017-071+.
- DSCSA / Title II FDASIA 2013.
- 42 CFR Part 2.
- 340B HRSA OPA.
- HL7 FHIR R5 Medication resources.
- GS1 SGTIN-198 + EPCIS 2.0.

---

## Quick start for downstream agents

If you are an agent landing here for the first time, the read order is:

1. `manifest.json` — machine-readable; gives you the crate map.
2. `PRD.md` §1 + §2 — mission + 20 bounded contexts.
3. `ARCHITECTURE.md` §1 + §3 — layers + per-context architecture summary.
4. `competitor-parity-matrix.md` — what we promise (150 capability rows).
5. `implementation-plans/IP-001..IP-010` — the next-step work.
6. `decisions/ADR-MS-001` + `ADR-MS-002` — service-level decisions.
7. `policies/` — Cedar surface.
8. `slos/` — SLO surface.

---

## Key invariants (read this twice)

1. **No medication may pass verification without resolving to a known RxCUI.** Free-text medications are forbidden in the verification path.
2. **EPCS keys are per-prescriber, DEA-bound, HSM-backed.** No platform-shared keys.
3. **Schedule II controlled-substance dispense requires dual-pharmacist verification.** Single-pharmacist facilities queue until a second pharmacist signs.
4. **BCMA five-rights failure hard-blocks administration.** Override requires reason code AND pharmacist callback within 5 minutes.
5. **Recall-sequestered lots cannot be dispensed.** Cedar grant required for override; rare.
6. **USP 800 hazardous compounding requires `iso-7-negative-pressure` cell capability.** Pharmacy refuses on cells without the tag.
7. **All cross-tenant projection (specialty hub) requires an active DataSharingAgreement in `consent-graph`.**
8. **Every controlled-substance transaction is witness-signed.**
9. **Knowledge package switching is per-tenant A/B with sub-5-minute rollback.**
10. **HLC for causality; TrueTime tier opt-in for fin-grade dispense ledger.**

---

## Cell tier and shuffle sharding

Pharmacy runs on cell tier **T0** (patient safety + DEA). Tenants are shuffle-sharded across cells with width 2. A single cell failure cannot affect more than 2 tenants beyond the failing cell's home tenants. Per ADR-0248.

Cell capability tags relevant to pharmacy:

- `dea-controlled-substance-vault` — for cells holding CII–CV physical inventory.
- `iso-7-negative-pressure` — for cells supporting USP 800 hazardous compounding.
- `usp-797-iso-5-cleanroom` — for cells supporting USP 797 sterile compounding.
- `epcs-confidential-compute` — for cells running EPCS signing workers under SEV-SNP/TDX/CCA attestation.
- `outpatient-retail-pos` — for cells supporting retail counter dispensing.
- `specialty-hub-data-sharing` — for cells participating in limited-distribution drug hub programs.

---

## Knowledge package A/B switching (operational note)

Knowledge package version pins live at `tenant.config.knowledge_package_version`. Two versions are pinned per tenant at any time (current + previous). Switching is atomic and bounded by:

```
# Pseudo-CLI to switch a tenant from FDB v2026.05 to v2026.06:
oya pharmacy knowledge switch --tenant=<tid> --vendor=fdb --to=2026.06
# Rollback:
oya pharmacy knowledge switch --tenant=<tid> --vendor=fdb --to=2026.05
```

Both invocations emit an `oya.pharmacy.catalog.knowledge-package-activated` event sealed to `audit-chain`. The previous version remains hot for 7 days post-switch for instant rollback. Versions older than 7 days are archived to cold storage but retrievable within 24 h via `oya pharmacy knowledge restore`.

---

## Common scenarios and runbook pointers

| Scenario | Runbook |
|---|---|
| Surescripts outbound queue depth > 5 min | `runbooks/RUN-001-surescripts-outbound-queue.md` |
| EPCS DEA registration verification failure | `runbooks/RUN-002-epcs-dea-registration.md` |
| Cabinet vendor outage | `runbooks/RUN-003-cabinet-vendor-outage.md` |
| BCMA endpoint unreachable | `runbooks/RUN-004-bcma-endpoint-unreachable.md` |
| Knowledge package corrupt → rollback | `runbooks/RUN-005-knowledge-package-rollback.md` |
| Recall feed missing | `runbooks/RUN-006-recall-feed-missing.md` |
| 340B classification drift | `runbooks/RUN-007-340b-classification-drift.md` |
| DSCSA suspect product investigation | `runbooks/RUN-008-dscsa-suspect-product.md` |
| Surescripts certificate rotation | `runbooks/RUN-009-surescripts-cert-rotation.md` |
| Break-glass elevation (governance review) | `runbooks/RUN-010-break-glass-review.md` |

(Runbook stubs land alongside IP-001..IP-010 in their respective execution waves.)

---

## Versioning + sunset policy

Pharmacy enforces `feedback_no_silent_regression`. Every public contract is versioned:

- OpenAPI version path: `/api/v1/pharmacy/...` and `/api/v2/...` (when v2 lands).
- AsyncAPI topics: `oya.pharmacy.*` is v1; v2 events go to `oya.pharmacy.v2.*`.
- Proto package: `oya.pharmacy.v1` then `oya.pharmacy.v2`.

Sunset policy:
- Deprecation announced via ADR + version bump + 90-day notice minimum.
- Per `agent-skills:deprecation-and-migration`: dual-write phase ≥ 30 days; reads from both; cut-over only after 99% client migration.

---

## Reach out

- Slack: `#axis-pharmacy`
- Email: `axis-pharmacy@oyatie.com`
- GitHub: `oyatie/oyatie` issues labeled `microservice:pharmacy`

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
