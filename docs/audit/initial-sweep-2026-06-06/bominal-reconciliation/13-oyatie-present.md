---
doc_class: BominalReconciliation
title: Oyatie PRESENT-state map — current product/service surface + bominal→oyatie rename/mapping hypothesis
status: synthesized
date: 2026-06-06
inputs:
  - ls /Users/jasonlee/Developer/source/oya/        # 87 product/substrate service dirs (live)
  - ls /Users/jasonlee/Developer/source/cloud/      # 26 cloud-substrate service dirs (live)
  - ls /Users/jasonlee/Developer/source/docs/decisions/  # ADR-0001..0514 titles
  - ADR-0058 (flat catalog — carries the ORIGINAL bominal-era product names)
  - ADR-0060 (bominal-inheritance precedence — the glossary translation table)
  - ADR-0315 (ERP/SAP-parity module→service map), ADR-0321 (B2B leader coverage, 13 new svcs)
  - ADR-0332 (healthcare-integration decomposed into 8 domain svcs)
  - ADR-0237/0238/0334 (Connect super-app dissolved into mail/messenger/calendar/community/social/...)
  - decision-record-oyatie-canon.md (already-ruled founder decisions — NOT re-surfaced here)
method: >
  Diff the LIVE oya/+cloud/ catalog against the bominal-era product names still
  embedded in ADR-0058's catalog block and ADR-0060's glossary/inheritance tables.
  Where a bominal module has no live dir, classify: renamed | decomposed | dissolved |
  retired | NO-HOME-YET. Mappings are HYPOTHESIS grounded in cited ADR text, not guesses.
scope_note: >
  This is a PRESENT-state + rename-hypothesis map only. Already-ruled founder decisions
  (canon record) and the 7 .Trash legacy recoveries (recovery register) are NOT duplicated.
---

# 13 — Oyatie PRESENT state + bominal→oyatie rename/mapping hypothesis

## 0. Headline

Oyatie's PRESENT surface is a **flat catalog of 87 product/substrate microservices** (`oya/`)
+ **26 cloud-substrate microservices** (`cloud/`), no grouping, no "Arms." This is the SAME
project bominal was, after three churning forces that LOST the old names:

1. **The 2026-05-13 rename session** (ADR-0058 + ADR-0060) — retired bominal "Arms"
   (Healthcare/Corporate/FinTech/Platform), flattened the catalog, applied a glossary
   translation (Object Graph→ontology, Workspace→Connect, Platform→shared substrate).
2. **Coverage-doctrine expansion** (ADR-0315 ERP/SAP-parity +9, ADR-0321 B2B-leader +13) —
   exploded coarse bominal modules (manufacturing, logistics) into SAP-shaped flat services.
3. **Two big decompositions** — healthcare-integration→8 clinical services (ADR-0332);
   Connect super-app→8 standalone comms services (ADR-0237/0238).

The net effect: **the live tree carries almost NONE of the original bominal product names.**
ADR-0058's own catalog block is the fossil record of what the names WERE.

---

## 1. Oyatie PRESENT products / verticals (live surface)

### 1a. `oya/` — products + always-on shared substrate (87 dirs)

**Healthcare cluster** (8 svcs — ADR-0332 decomposition of one bominal "medical/healthcare" module):
`emr`, `healthcare-integration`, `imaging`, `diagnostics`, `emergency`, `pharmacy`,
`patient-monitoring` — plus `clinical-decision-support` + `care-management` named in ADR-0332
as the 8th/9th (folders not yet all landed; decomposition is `advisory-until-scaffold-lands`).

**Enterprise / ERP cluster** (SAP-parity, ADR-0315 +9 / ADR-0321 +13):
`accounting`, `treasury`, `crm`, `warehouse`, `supply-chain-planning`, `production-planning`,
`quality-management`, `plant-maintenance`, `global-trade`, `real-estate`,
`contract-lifecycle-management`, `financial-planning`, `finops-portal`, `marketplace`,
`procurement`→(folded into marketplace), `hr`, `payroll`, `performance-management`,
`learning-management`, `itsm`, `incident-management`, `contact-center`,
`marketing-automation`, `data-warehouse`, `data-pipeline`.

**Connect / communications cluster** (ADR-0237/0238 super-app dissolution):
`mail`, `messenger`, `calendar`, `community`, `social` (absorbed `shorts`, ADR-0334),
`meet`, `recordings`, `notes`, `tasks`, `sheets`, `slides`, `sites`, `forms`, `drive`,
`whiteboard`, `design-collaboration`, `translate`, `comms-email`, `connect` (now a thin
shell/registry after dissolution).

**Platform / B2B shell + ecosystem-adapter + dev:**
`application` (B2B unified shell, ADR-0061), `workflow-engine`, `workflow-studio`, `ontology`,
`connector`, `developer-sdk`, `plugin-app-store`, `marketplace`, `feature-flags`,
`workplace-integration`.

**Shared substrate (always-on, underpins every product):**
`tenancy`, `tenant-rbac`, `identity`, `audit-chain`, `eventing`, `policy`, `consent-graph`,
`compliance`, `governance`, `observability`, `search`, `intelligence`, `detection`,
`analytics`, `comms-email`, `ops`, `ops-dashboard-control-center`, `api-gateway`, `docs`.

**Bespoke "oya-*" substrate (ADR-0476/0478/0479/0480/0481 — own-the-endpoint ratchet):**
`oya-identity`, `oya-billing`, `oya-meter`, `oya-cost`, `oya-flags`, `oya-authn-device-firmware`.

**CI/forge surface (ADR-0511/0513/0374):** `ci-controller`, `ci-tide`, `ci-webhook-gateway`.

**Consumer-ish / misc:** `community`, `social`, `messenger` (B2C personal context),
`app-shell-frontend`, `app-store`.

### 1b. `cloud/` — IaaS/PaaS substrate (26 dirs)

The dogfood substrate (D-LAYER canon: products run on cloud as tenant workloads):
`cloud-compute`, `cloud-k8s`, `cloud-capacity`, `cloud-cell`, `cloud-data`, `cloud-storage`,
`cloud-network`, `cloud-network-dns`, `cloud-iam`, `cloud-kms`, `cloud-secrets`,
`cloud-billing`, `cloud-billing-tax`, `cloud-finops`, `cloud-cost`(via oya-cost),
`cloud-iac`, `cloud-intelligence`, `cloud-marketplace`, `cloud-tenancy`, `tenancy`,
`cell-lifecycle`, `cell-rebalancer`,
`managed-k8s-cluster-lifecycle`, `managed-k8s-control-plane-host`,
`managed-k8s-sla-observability`, `managed-k8s-tenant-quota` (the **managed-Kubernetes
PRODUCT surface**, ADR-0376 — a sold cloud product, not just internal).

---

## 2. The bominal→oyatie RENAME / MAPPING hypothesis

> Ground truth: **ADR-0058's catalog block** literally lists the bominal-era product names,
> and **ADR-0060's glossary table** lists the canonical translations. These are the rename
> fossils. Below, "bominal module" = the name as it appears in ADR-0058 / ADR-0060.

### 2a. Glossary-level renames (ADR-0060 §translation table — definitional, not module moves)

| Bominal term | Oyatie term | Evidence |
|---|---|---|
| "Object Graph" | **ontology** | ADR-0060 #2, ADR-0055, ADR-0122 |
| "Workspace" (product) | **Connect** → then dissolved (see 2d) | ADR-0060 #7, ADR-0029 |
| "Platform / Ops Arm" | **shared substrate** (no `platform/` dir) | ADR-0060 #3 |
| "Arm" (Healthcare/Corporate/FinTech/Platform) | **flat catalog entry** (Arms RETIRED) | ADR-0060 #4/#5, ADR-0058 |
| "Modular Product Shell" | **Application** (B2B shell) | ADR-0060 #8, ADR-0061 |
| `oya-platform-*` crates | `oya-<microservice>-*` (BNF v4.1) | ADR-0060, ADR-0056 |
| "Arms" as architecture | **sales/GTM labels only** | ADR-0058, ADR-0060 #9 |

### 2b. Healthcare module → 8 decomposed services (ADR-0332)

Bominal had a coarse **medical / healthcare** module (ADR-0058 listed `medical`,
`healthcare-portal`, `clinical`). Oyatie first collapsed these into one
`healthcare-integration` µservice (215 features / 14 domains), then ADR-0332 **decomposed**
it into single-concern services:

| Bominal/early-oyatie | Oyatie PRESENT service(s) | Status |
|---|---|---|
| `medical` (clinical record-of-truth) | **`emr`** (Patient/Encounter/Med/Order/Result, FHIR R5) | live |
| `clinical` (lab/path) | **`diagnostics`** (lab + pathology) | live |
| (imaging inside medical) | **`imaging`** (PACS/VNA/DICOM) | live |
| `emergency` | **`emergency`** (ED-IS) — name survived | live |
| `pharmacy` | **`pharmacy`** — name survived | live |
| (monitoring inside medical) | **`patient-monitoring`** (ICU/RPM telemetry) | live |
| (CDS inside medical) | **`clinical-decision-support`** | named, folder pending |
| `healthcare-portal` / pop-health | **`care-management`** | named, folder pending |
| (interop) | **`healthcare-integration`** — NARROWED to FHIR/HL7v2/DICOM broker only | live |

### 2c. ERP / enterprise coarse modules → SAP-shaped flat services (ADR-0315/0321)

ADR-0058 listed coarse bominal modules `manufacturing`, `logistics`, `procurement`,
`facility-ops`, `performance`, `ats`, `grc`. ADR-0315 (SAP module map) + ADR-0321 explode
these into SAP-coded flat services:

| Bominal module (ADR-0058) | SAP code | Oyatie PRESENT service(s) | Evidence |
|---|---|---|---|
| `manufacturing` | PP, QM, PM | **`production-planning`** + **`quality-management`** + **`plant-maintenance`** | ADR-0315 §D-1 |
| `logistics` | EWM, SCM/APO, TM | **`warehouse`** + **`supply-chain-planning`** (TM composed) | ADR-0315 §D-1 |
| `procurement` | MM, SRM | **`marketplace`** + `connector` + `warehouse` (no `procurement/` dir) | ADR-0315 §D-1 |
| `facility-ops` | RE-FX, PM | **`real-estate`** + **`plant-maintenance`** | ADR-0315 §D-1 |
| `finance` / `banking` | FI, TRM | **`accounting`** + **`treasury`** + `payments` + `finops-portal` | ADR-0315 §D-1 |
| (sales/CRM) | SD, CRM | **`crm`** + `marketplace` + `payments` | ADR-0315 §D-1 |
| (trade) | GTS | **`global-trade`** | ADR-0315 §D-1 |
| `grc` | EHS/compliance | **`compliance`** + **`governance`** (composed) | ADR-0315/0321 |
| `performance` | HCM-talent | **`performance-management`** | ADR-0321 |
| `ats` (applicant tracking) | HCM-recruiting | **NO dedicated home** (see §3) | — |
| `hr` / `payroll` | HCM | **`hr`** + **`payroll`** — names survived | ADR-0315 §HCM |

New ADR-0321 B2B-leader services with NO direct bominal predecessor (net-new vs bominal):
`marketing-automation`, `contact-center`, `performance-management`, `learning-management`,
`itsm`, `incident-management`, `financial-planning`, `data-warehouse`,
`contract-lifecycle-management`, `whiteboard`, `design-collaboration`, `data-pipeline`,
`healthcare-integration`.

### 2d. Connect ("Workspace") super-app → 8 standalone comms services (ADR-0237/0238)

Bominal "Workspace" → renamed **Connect** (ADR-0060 #7) → ADR-0238 dissolved Connect's
12-app suite into first-class flat services via Strangler migration (ADR-0237):

| Bominal/Connect app | Oyatie PRESENT service | Status |
|---|---|---|
| Workspace mail | **`mail`** + `comms-email` | live |
| Workspace chat | **`messenger`** | live |
| Workspace calendar | **`calendar`** | live |
| Workspace community | **`community`** | live |
| Workspace social / shorts | **`social`** (absorbed `shorts`, ADR-0334) | live |
| Workspace meet/recordings | **`meet`**, **`recordings`** | live |
| Workspace docs/sheets/slides/notes/tasks/sites/forms/drive/whiteboard | same-named live services | live |
| (network / anonymous) | folded into social/identity | partial |
| **`connect`** | thin shell/registry after dissolution | live (vestigial) |

### 2e. Substrate modules — bominal "Platform/Ops Arm" → oyatie shared substrate

Bominal's Platform/Ops Arm (records/identity/audit/eventing/policy/secrets/observability)
→ oyatie's always-on shared substrate (`tenancy`, `identity`, `audit-chain`, `eventing`,
`policy`, `observability`, `consent-graph`, `compliance`, `governance`, `search`,
`intelligence`, `detection`, `analytics`) + the dogfood `cloud/` IaaS tier. (Already
covered/stronger per the recovery register; named here only for completeness of the map.)

---

## 3. Bominal verticals with NO oyatie home yet (the LOSS surface)

These bominal-era product names from ADR-0058's catalog have **no dedicated live service**
and only incidental string mentions — they are the clearest candidates for lost/dropped
context (verified via grep across all README/PRD: only stray references, no owning dir):

| Bominal module (ADR-0058) | Live home? | Disposition hypothesis |
|---|---|---|
| **`insurance`** | NONE (stray refs only) | DROPPED or implicit-in-compliance/payments. The bominal "FinTech Arm" insurance vertical has no flat service. NET-NEW gap if still in scope. |
| **`banking`** | NONE (one stray ref) | DROPPED/deferred. Covered partially by `payments`+`treasury` for rails, but no banking-core (deposits/lending/KYC) service. |
| **`finance`** (consumer fintech, distinct from accounting) | NONE | Ambiguous — `accounting`/`treasury`/`finops-portal` cover B2B finance; consumer-finance vertical absent. |
| **`dining`** | NONE | DROPPED. Consumer hospitality vertical (POS/reservations) — no service, no ADR coverage. |
| **`cellar`** (wine/inventory?) | NONE | DROPPED. Niche consumer vertical, fully lost. |
| **`ats`** (applicant tracking) | NONE dedicated | Likely folded into `hr`/`performance-management` as capability-tier, but no enumerated owner; recruiting/ATS surface is thin. |
| **`security`** (the bominal product, not infra) | ambiguous | Maps partly to `detection`/`compliance`/`governance`; no product-grade physical/corporate-security service. |
| **`grc`** | partial | Split across `compliance`+`governance` as capability, not a named GRC product. |
| **`accounting`** | LIVE but `status: foundation-slice-in-progress` | present-but-thin; ERP FI still partial per ADR-0315. |

**Canon cross-check (do NOT re-rule):** the founder D9 ruling already declares MAXIMAL
VERTICAL SCOPE as the endpoint (sequenced, not cut) and flags **defense** + **power-grid/OT**
as NET-NEW capture-needed. The bominal-lost verticals above (insurance/banking/dining/cellar/
ats) are a DIFFERENT loss class: they were ALREADY-NAMED in bominal/ADR-0058 and then
silently dropped in the flatten/coverage-expansion — i.e. regressions, not net-new ambitions.
They belong in the vertical-coverage map (Task #18) as "named-then-lost," distinct from
"never-named" defense/power-grid.

---

## 4. Confidence / caveats

- **HIGH confidence:** healthcare decomposition (ADR-0332 explicit), ERP module map
  (ADR-0315 §D-1 explicit oyatie-destination column), Connect dissolution (ADR-0237/0238
  explicit), glossary renames (ADR-0060 table). These are author-stated, not inferred.
- **MEDIUM confidence:** `medical`→`emr`, `clinical`→`diagnostics`, `manufacturing`→3-svc
  split, `logistics`→warehouse+scp — inferred from SAP-code + README mission text, not a
  single explicit "X was renamed to Y" line for each.
- **LOWER confidence:** the NO-HOME verticals (§3) — absence-of-evidence (no dir + only stray
  grep hits) rather than an explicit retirement ADR. `insurance`/`banking`/`dining`/`cellar`
  should be confirmed against any bominal-side ADR before being declared truly dropped.
- ADR-0058's catalog block is dated 2026-05-13 and may itself be mid-rename; treat its name
  list as the best-available bominal fossil, not a guaranteed-complete bominal inventory.

---

## 5. One-line digest

Oyatie PRESENT = 87 `oya/` + 26 `cloud/` flat microservices; bominal's coarse modules were
renamed (Object Graph→ontology, Workspace→Connect→dissolved), decomposed (medical→8 clinical
svcs; manufacturing→PP/QM/PM; logistics→warehouse/SCP), or expanded (ERP/B2B-leader +22 svcs);
and **`insurance`, `banking`, `finance`(consumer), `dining`, `cellar`, `ats`** are
bominal-named verticals with **no live oyatie home** — the "named-then-lost" regression set,
distinct from the canon's net-new defense/power-grid ambitions.
