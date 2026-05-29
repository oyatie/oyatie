# IP-008 — Compounding USP 795/797/800 + BUD + environmental evidence binding

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001
- **Estimated complexity**: L

## Goal

Implement compounding workflows for USP 795 (non-sterile), USP 797 (sterile), USP 800 (hazardous), with BUD calculator and environmental monitoring evidence binding.

## Acceptance criteria

- AC-1: Kernel: `MasterFormulationRecord`, `CompoundingRecord`, `EnvironmentalMonitoring`, `USPClass` enum.
- AC-2: Domain BUD calculator: USP-table fallback explicit; in-pharmacy stability reference for exceptions.
- AC-3: USP 800 compounding requires cell capability tag `iso-7-negative-pressure`; Cedar policy `compounding-usp800-cell-capability.cedar` enforces.
- AC-4: Environmental monitoring evidence (particle counts, viable counts, cleaning log, gowning log) linked at completion.
- AC-5: REST `POST /Compounding`.
- AC-6: AsyncAPI `oya.pharmacy.compounding.completed`.
- AC-7: Tests covering USP class enforcement, BUD calculation, environmental evidence presence.

## Tasks

1. Kernel + domain.
2. BUD calculator with USP table.
3. Cell-capability Cedar gate.
4. Environmental evidence binding.
5. REST + AsyncAPI.
6. Tests.

## Risks

- USP table updates (2024 BUD overhaul) → vendor-neutral table version pin.
- Hazardous-drug NIOSH list updates → quarterly refresh.
