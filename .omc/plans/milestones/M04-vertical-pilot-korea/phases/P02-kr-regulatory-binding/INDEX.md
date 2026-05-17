---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M04-P02
title: KR Regulatory Pack Binding (PIPA + CSAP + K-ISMS-P + KCMVP)
status: stub
purpose: Bind KR regional pack to the elected vertical's capability pack with full control evidence.
execution_variant: merge-into-existing-crates
decided_at: 2026-05-17
decided_by: user-directive-option-2
execution_variant_note: "Delta-1 ships PipaDataClassification enum + KrRegulatoryBinding struct as a new kr_regulatory module inside oya-regional-pack-domain. No new crate scaffolding, no new workspace deps. Multispectrum evidence at evidence/multispectrum/m04-p02-kr-reg-mv-delta1-1778999979.json."
---

# M04-P02 — KR Regulatory Binding

## Purpose
Per [`../../../../../docs/PRIVACY-PROGRAM.md`](../../../../../docs/PRIVACY-PROGRAM.md). KR-as-launch-locale.

## Acceptance
- KR pack bindings for: PIPA Art-23 sensitive data exclusion; CSAP control evidence; K-ISMS-P control evidence; KCMVP HSM operational.
- 전자세금계산서 (KR e-tax invoice) format integration via `cloud.billing.invoice.generate`.
- 휴일/야간 근로 (KR holiday/night work premium) and 실명인증 (KR real-name verification) and 사업자등록 (KR business registration) surfaces per glossary anchors.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | PIPA + CSAP control evidence pack | stub | [`IP-001-pipa-csap-evidence.md`](IP-001-pipa-csap-evidence.md) |
| IP-002 | K-ISMS-P + KCMVP HSM operational | stub | [`IP-002-isms-p-kcmvp-hsm.md`](IP-002-isms-p-kcmvp-hsm.md) |
| IP-003 | KR vertical-specific surfaces (전자세금계산서 / 휴일·야간 근로 / 실명인증 / 사업자등록) | stub | [`IP-003-kr-vertical-surfaces.md`](IP-003-kr-vertical-surfaces.md) |

## Estimated parallelism
3 agents in parallel.

## Symbols-touched
`regional-packs/kr/`, `crates/oya-vertical-corporate-{payroll,kyc,tax-invoice}-*`, `docs/COMPLIANCE-MATRIX.md` rows.

## Agent-handoff
```
icm store -t context-oyatie -c "M04-P02 complete: KR pack bound; PIPA/CSAP/K-ISMS-P/KCMVP evidence on file" -i critical -k "M04,P02,kr-regulatory,complete"
```
