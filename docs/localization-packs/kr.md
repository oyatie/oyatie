---
doc_class: LocalizationPack
pack_code: kr
status: planning-closed-foundational      # `active` requires the activation acceptance criteria in pack.yaml
lifecycle: planning-closed-foundational
foundational: true
lead_milestones: [M01, M02, M03, M04, M05, M06, M07]
languages: [ko, en]
date: 2026-05-13
owners: ["council-architecture", "gtm-customer-success-kr"]
authority_chain: docs/MASTERPLAN.md §2.5, §5.5 → ADR-0064 → docs/localization-packs/INDEX.md → pack.yaml (authoritative) → this file
manifest: docs/localization-packs/kr/pack.yaml
doc_status: published
---

# Korea Localization Pack (`kr`) — Pack #1 (planning-closed, foundational)

The foundational localization pack. M01–M07 ship the canonical base **plus** the KR pack in lock-step because oyatie's first paying tenant is a KR group.

**Status:** `planning-closed/foundational`. Status flips to `active` when the activation acceptance criteria in `pack.yaml` are green:
- `kr/corpus.lock` exists with governed source entries
- Each FD-001 material µservice has overlay crate + overlay PRD + regulatory ADR shipped
- Acceptance evidence for each FD-001 material µservice is signed
- The first KR tenant can run the FD-001 enterprise/SMB workflow set on pack-covered µservices

**Source of truth:** [`pack.yaml`](kr/pack.yaml) is authoritative for scope (`microservices_in_scope`), regulatory bindings, connectors, and lifecycle status. This overview doc and `docs/localization-packs/INDEX.md` MUST stay consistent with the manifest; `lean-a5-doc-coverage` enforces parity.

---

## 1. Pack scope

The KR pack provides the regulatory, statutory, language, document-template, and connector overlay for the following canonical-base µservices:

| Cluster | µservices in KR pack scope |
|---|---|
| Workforce | hr, payroll, accounting, ats, grc, performance, workforce-analytics |
| Healthcare | medical, pharmacy, patient, emergency, clinical, healthcare-portal |
| FinTech | payments, insurance, finance-quant, settlement |
| Industrial | manufacturing, logistics, facility-ops, procurement, security |
| | connect (Pro + Personal); community |
| Hospitality | hospitality, dining, cellar |

Substrate (tenancy / identity / audit-chain / etc.), Workflow, Ontology, Cloud, Foundry, Application are pack-neutral by ADR (their concerns are universal); KR-specific behavior in those layers flows in via pack adapters only.

---

## 2. Regulatory bindings

| ID | Regime | Surface | Milestone | Authority |
|---|---|---|---|---|
| `4dae-edi` | 4대보험 EDI (NPS / NHIS / 고용 / 산재) | EDI v5.0 (취득 / 상실 / 변경 / 보수월액) | M03 | NPS, 건보공단, 근로복지공단 |
| `yearly-tax-settlement` | 연말정산 21-category (소득공제 14 + 세액공제 7) | XML / 국세청 홈택스 API | M03 | 국세청 |
| `k-gaap` | K-GAAP chart of accounts + double-entry | accounting kernel | M03 | 한국채택국제회계기준원 |
| `kgaap-statements` | 재무상태표 / 손익계산서 / 현금흐름표 / 자본변동표 | Typst templates | M03 | 한국채택국제회계기준원 |
| `pipa-b2b` | 개인정보보호법 (B2B posture) | Cedar policy fragments | M03 | 개인정보보호위원회 |
| `pipa-b2c` | 개인정보보호법 (B2C / Personal context) | Cedar + audit chain | M05 | 개인정보보호위원회 |
| `medical-law` | 의료법 (의료기록 보존, 환자 권리, 응급의료법) | medical / patient kernels | M04 | 보건복지부 |
| `hira-dur` | HIRA DUR (의약품안전사용서비스) realtime check | pharmacy adapter | M04 | 건강보험심사평가원 |
| `kfda-recall` | KFDA recall / dispatch notice | pharmacy adapter | M04 | 식품의약품안전처 |
| `nhis-billing` | 건보공단 청구 / 심사 | medical adapter | M04 | 국민건강보험공단 |
| `khira-outcomes` | 진료내역 / 환자안전지표 outcomes | medical adapter | M04 | 건강보험심사평가원 |
| `emr-crosswalk-duzon` | 더존 EMR 데이터 교환 | medical adapter | M04 | 더존비즈온 |
| `emr-crosswalk-ubicare` | 유비케어 EMR 데이터 교환 | medical adapter | M04 | 유비케어 |
| `emr-crosswalk-bitcomputer` | 비트컴퓨터 EMR 데이터 교환 | medical adapter | M04 | 비트컴퓨터 |
| `efl-registration` | 전자금융업 등록 | payments + FSS quarterly | M06 | 금융위원회, FSS |
| `payment-rails-kr` | 간편결제 (토스/카카오페이/네이버페이/페이코) | payments adapter | M06 | 각 PG |
| `card-acquirer-kr` | 카드사 매입 (KB / 신한 / 현대 / 삼성 / 롯데 / BC / 하나) | payments adapter | M06 | 각 카드사 |
| `pci-dss-l1` | PCI DSS L1 service provider | crosscutting | M06 | PCI Security Standards Council |
| `insurance-law` | 보험업법 (손해보험 / 생명보험 분리 license) | insurance kernel | M06 | 금융위원회 |
| `industrial-safety` | 산업안전보건법 | manufacturing / facility-ops | M07 | 고용노동부 |
| `serious-accident-act` | 중대재해처벌법 | manufacturing / security | M07 | 고용노동부 |
| `chemical-substances-act` | 화학물질관리법 | manufacturing adapter | M07 | 환경부 |
| `truck-transport-act` | 화물자동차운수사업법 | logistics adapter | M07 | 국토교통부 |
| `port-transport-act` | 항만운송사업법 | logistics adapter | M07 | 해양수산부 |
| `119-emergency` | 119 응급의료 routing | emergency adapter | M04 | 소방청 |
| `retention-kr` | 메신저 / 메일 보관 의무 (Bominal ADR-0215 KR-mode) | connect adapter | M03 | (Bominal-inherited) |

---

## 3. Pack composition

### 3.1 Manifest

`docs/localization-packs/kr/pack.yaml` (canonical pack manifest per ADR-0064 §4)

### 3.2 Overlay crates (BNF v4.1)

Pack overlays per ADR-0064 §1 — naming options:

- BC-inside-µservice: `payroll-kr-edi-adapter`, `medical-kr-hira-domain`, `pharmacy-kr-dur-adapter`
- Discrete pack crate: `pack-kr-payroll-statutory-domain`, `pack-kr-accounting-kgaap-domain`

Both forms valid. Use BC-inside when overlay is small (≤2 BCs); use discrete pack crate when overlay spans 3+ BCs coherently.

### 3.3 Policy fragments

`crates/policy-kr-*` — Cedar policy fragments enforcing PIPA, 의료법, 보험업법, etc.

### 3.4 Workflow Studio templates

`crates/workflow-templates-kr-*` — 10 canonical templates per M03 launch:

1. KR payroll-run cycle (gross-to-net + 4대보험 EDI + 연말정산 trigger)
2. KR HR onboarding (employee hire + 4대보험 취득 + 근로계약서)
3. KR HR offboarding (퇴직금 + 4대보험 상실 + 이직확인서)
4. KR clinical encounter handoff
5. KR prescription DUR check + dispense
6. KR insurance claim submission
7. KR procurement → accounting auto-journal
8. KR shipment last-mile (CJ대한통운 / 한진 / 롯데 / 우체국)
9. KR connect-pro legal hold initiated → released (four-eyes)
10. KR 연말정산 yearly cycle

### 3.5 Typst document templates

`crates/document-templates-kr-*`:

- 급여명세서 (payslip) — per `급여실태조사` format
- 4대보험 취득 / 상실 / 변경 신고서
- 연말정산 간소화 서식
- 재무상태표 / 손익계산서 / 현금흐름표 / 자본변동표 (K-GAAP statements)
- 처방전 (clinical prescription)
- 진단서 (medical certificate)
- 거래명세서 / 세금계산서 (tax invoice)
- 출하증명서 (shipping certificate)

### 3.6 Acceptance evidence

`docs/localization-packs/kr/evidence/<microservice>.md` — one evidence doc per µservice, signed audit-chain segment + regulatory submission sample + dry-run report.

### 3.7 Corpus lock

`docs/localization-packs/kr/corpus.lock` — governed source-family lock for KR statute, connector, and regulator inputs (per Bominal ADR-0190 inherited). Refreshed quarterly; active promotion requires signed source snapshots and CI signature verification.

---

## 4. Pack-pluggability gates

Per ADR-0064 §2, the KR pack is mandatory for M03 first-paying-tenant GA. The canonical base for hr / payroll / accounting / connect alone cannot ship to a KR tenant — the `kr` pack overlay must be active.

Gate per phase (M01-P05 already green; M02-P22 / M03-P08 forthcoming):

- KR pack manifest governed (`pack.yaml` + `corpus.lock`) and signed at active promotion
- ≥1 µservice in pack scope has overlay crate + overlay PRD + regulatory ADR shipped
- `lean-a5-doc-coverage` green for `kr × <microservice>` for every active µservice
- Acceptance evidence bundle for at least one µservice signed

---

## 5. Pack ownership

| Role | Owner |
|---|---|
| Architecture | `council-architecture` |
| GTM | `gtm-customer-success-kr` |
| Regulatory | `gtm-customer-success-kr` (with founder oversight on FSS / KFDA / 보건복지부 escalations) |
| Engineering lead | `axis-foundry` (pack mechanics) + per-µservice axis teams |

---

## 6. Maintenance cadence

- **Quarterly**: corpus.lock refresh; signed by `council-architecture` + `gtm-customer-success-kr`
- **Monthly**: statutory rate review (NPS / NHIS / 고용 / 산재 rate changes; 소득세법 간이세액표 updates)
- **As-issued**: KFDA recall / dispatch; HIRA DUR list updates (push-handled via outbox)

---

## 7. References

- [ADR-0064 Canonical base + localization packs](../decisions/ADR-0064-canonical-base-and-localization-packs.md)
- [ADR-0063 Documentation set coverage](../decisions/ADR-0063-documentation-set-coverage.md)
- [MASTERPLAN §2.5, §5.5](../MASTERPLAN.md)
- [INDEX](INDEX.md)
- Bominal ADR-0140 (retired per ADR-0145) (inherited regional-pack pattern)
- Bominal ADR-0190 (inherited versioned regulatory corpus.lock)
- Bominal ADR-0210 (M03 KR group payroll + mail launch criteria)
- Bominal ADR-0215 (retention / legal hold dual-context)
