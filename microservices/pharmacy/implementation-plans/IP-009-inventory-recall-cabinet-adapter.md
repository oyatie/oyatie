# IP-009 — Inventory + recall sequestration + expiry stratification + cabinet vendor-neutral adapter contract

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001
- **Estimated complexity**: L

## Goal

Build inventory management (lot/expiry/location/par), recall sequestration with hard-block on dispense, expiry stratification, and the vendor-neutral cabinet adapter contract (Pyxis / Omnicell / Carousel / AcuDose / MedDispense).

## Acceptance criteria

- AC-1: Kernel: `InventoryLot`, `ParLevel`, `RecallNotice`, `Cabinet`, `CabinetTransaction`, `CabinetDiscrepancy`.
- AC-2: Domain: par/min/max state machine; expiry stratification (<7d use-first; <14d alert; <30d watch).
- AC-3: Recall sequestration usecase: `sequester_recall(ndc, lot)` blocks dispense within window.
- AC-4: Cabinet adapter trait `CabinetVendorAdapter`; concrete `oya-pharmacy-auto-dispensing-adapter-pyxis`, `-omnicell`, `-carousel` stubs.
- AC-5: Offline mode supported; reconciliation on reconnect.
- AC-6: AsyncAPI `oya.pharmacy.inventory.recall-sequestered`, `oya.pharmacy.cabinet.discrepancy`.
- AC-7: Switch-vendor smoke test (swap Pyxis ↔ Omnicell adapter; verify no upstream code change).
- AC-8: Tests covering recall window, expiry strata, cabinet discrepancy reconciliation.

## Tasks

1. Kernel + domain.
2. State machine for par/min/max.
3. Recall sequestration.
4. Cabinet adapter trait + 3 vendor stubs.
5. Offline + reconciliation.
6. AsyncAPI.
7. Switch-vendor smoke.
8. Tests.

## Risks

- Cabinet vendor APIs proprietary → adapter contracts must abstract before any vendor-specific code.
- Recall window ambiguity → use vendor recall notice authoritative window.
