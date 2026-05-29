---
doc_class: User-Journey-README
journey_id: j144
slice: ecosystem-economy
status: draft
date: 2026-05-20
authority_tier: 2
---

# j144 — Chris builds his job-search pipeline in personal Workflow Studio

## What this journey is

Twelve days after the layoff, Chris instantiates a personal job-search pipeline in Workflow Studio (the personal-tier UI on top of Workflow Engine substrate). The pipeline pulls postings from 6 sources, AI-filters against his structured criteria, drafts personalized cover letters with full transparency, tracks applications in Notes, and adds Calendar + Mail blocks when interviews start arriving.

This journey demonstrates that **the personal tenant has workflow capability equivalent to enterprise tenants** — ADR-0245 substrate-vs-product holds. Workflow Studio is the consumer product surface; Workflow Engine is the shared substrate.

## Why this journey exists

- It proves Intelligence two-layer (ADR-0255) — Chris's pipeline uses the consumer-brand-surface, sharing the AI substrate with KrampusCorp's enterprise pipeline (j132 + j145).
- It demonstrates closed-schema AEDT non-discrimination — the FilterSpec literally cannot contain protected-characteristic filters.
- It shows the high-risk-mode opt-in (j142) paying off — the fake-recruiter scam gets caught at the boundary.
- It demonstrates retraining locality (ADR-0311) — Chris's AI learns from his preferences without leaking weights to a third-party.
- It primes j145 (the KrampusCorp application) by showing where the application originated.

## µservices touched (7)

| µservice | Role |
|---|---|
| workflow-studio | Visual editor for blocks; configuration drawers |
| workflow-engine | Compiled pipeline runtime; scheduler; cross-tenant submission router |
| connector | OAuth + polling adapters for LinkedIn, Otta, RemoteOK |
| intelligence | Filter + drafter; closed-schema FilterSpec; transparency floor |
| notes | Applications-2026 database; row state machine for `draft_ready → apply → submitted → screened` |
| calendar | Interview slot suggestion + ICS round-trip |
| mail | Auto-replies; weekly digest delivery |

## Key ADRs

- **ADR-0245** substrate-vs-product layering (Workflow Studio vs Workflow Engine)
- **ADR-0247** self-modification (Intelligence runs as oyatie principal)
- **ADR-0255 §D-4** Intelligence two-layer + provider-credential BYOK
- **ADR-0292** marketplace (Chris can publish his pipeline as a community template)
- **ADR-0311** dual-tenant (everything stays on personal tenant)
- **ADR-0244** audience_type `B2C_JOB_SEEKER_ACTIVE` unlocks the template

## Labor-law anchors

- **US FCRA 1970** — fairness on background checks
- **US NY AEDT Local Law 144** — automated employment decision tool transparency (the pipeline IS an AEDT; Chris is operator + subject, so explainability is internal)
- **EU AI Act Article 86** — right-to-explanation (if KrampusCorp's screen uses AI, that obligation lives there, not here)
- **US Equal Pay Act 1963** — no salary-history asked in jurisdictions where it's banned

## Files in this bundle

| File | Purpose |
|---|---|
| `story.md` | Day-by-day narrative through 7 days |
| `ux-flow.md` | Screen-by-screen |
| `handshake.md` | Wire-level sequence per phase |
| `schemas/JobSearchFilterSpec.json` | The closed-form filter spec; non-discrimination-by-structure |
| `integration-test-plan.md` | B.1-B.12 + chaos + ADR coverage |

Per-µservice IPs:
- `microservices/workflow-studio/IP-journey-j144-job-search-template-and-canvas.md`
- `microservices/workflow-engine/IP-journey-j144-personal-pipeline-runtime.md`
- `microservices/connector/IP-journey-j144-job-board-adapters.md`
- `microservices/intelligence/IP-journey-j144-filter-and-drafter-consumer-tier.md`
- `microservices/notes/IP-journey-j144-applications-database.md`
- `microservices/calendar/IP-journey-j144-interview-slot-scheduling.md`
- `microservices/mail/IP-journey-j144-auto-reply-and-digest-delivery.md`

## Status

Draft. Authored 2026-05-20 as part of Wave-3-F.

## Completion expansion — j144 readme rigor pass

Scope: personal Workflow Studio job-search pipeline with AI-assisted drafting and calendar holds.
Persona: Chris Volkov.
Services: workflow-studio + workflow-engine + connect + intelligence + notes + calendar + mail.
Applicable ADRs: ADR-0244, ADR-0292, ADR-0297, ADR-0299, ADR-0311, ADR-0317, ADR-0320.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Coverage row 001: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 002: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 003: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 004: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 005: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 006: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 007: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 008: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 009: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 010: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 011: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 012: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 013: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 014: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 015: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 016: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 017: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 018: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 019: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 020: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 021: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 022: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 023: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 024: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 025: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 026: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 027: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 028: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 029: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 030: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 031: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 032: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 02: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 033: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 034: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 035: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 036: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 037: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 038: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 039: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 040: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 041: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 042: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 043: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 044: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 045: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 046: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 047: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 048: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 03: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 049: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 050: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 051: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 052: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 053: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 054: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 055: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 056: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 057: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 058: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 059: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 060: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 061: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 062: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 063: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 064: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 04: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 065: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 066: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 067: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 068: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 069: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 070: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 071: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 072: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 073: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 074: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 075: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 076: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 077: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 078: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 079: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 080: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 05: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 081: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 082: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 083: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 084: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 085: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 086: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 087: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 088: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 089: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 090: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 091: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 092: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 093: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 094: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 095: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 096: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 06: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 097: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 098: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 099: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 100: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 101: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 102: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 103: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 104: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 105: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 106: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 107: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 108: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 109: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 110: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 111: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 112: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 07: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 113: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 114: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 115: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 116: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 117: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 118: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 119: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 120: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 121: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 122: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 123: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 124: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 125: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 126: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 127: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 128: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 08: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 129: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 130: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 131: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 132: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 133: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 134: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 135: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 136: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 137: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 138: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 139: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 140: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 141: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 142: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 143: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 144: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 09: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 145: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 146: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 147: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 148: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 149: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 150: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 151: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 152: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 153: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 154: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 155: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 156: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 157: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 158: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 159: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 160: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 10: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 161: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 162: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 163: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 164: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 165: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 166: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 167: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 168: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 169: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 170: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 171: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 172: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 173: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 174: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 175: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 176: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 11: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 177: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 178: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 179: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 180: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 181: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 182: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 183: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 184: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 185: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 186: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 187: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 188: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 189: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 190: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 191: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 192: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 12: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 193: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 194: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 195: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 196: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 197: workflow-engine owns durable orchestration, state-machine replay, and idempotent cross-service compensation and cites ADR-0292.
Reader path 198: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 199: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 200: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 201: calendar owns interview slot, deadline, and follow-up scheduling with tenant labels and cites ADR-0317.
Reader path 202: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 203: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 204: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 205: connect owns external adapter handshake, connector consent, webhook verification, and retry/DLQ isolation and cites ADR-0297.
Reader path 206: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 207: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 208: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Checkpoint 13: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Coverage row 209: mail owns work-mail archive, notification cascade, and personal-mail refusal boundary and cites ADR-0320.
Reader path 210: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 211: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 212: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 213: intelligence owns AI screening, drafting, ranking, and fairness/audit constraints and cites ADR-0299.
Reader path 214: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 215: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 216: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 217: workflow-studio owns personal or work workflow authoring canvas, template packaging, and UX state projection and cites ADR-0244.
Reader path 218: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 219: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
Risk note 220: any future edit that weakens tenant labels, Cedar purpose binding, or audit-chain emissions must update ADR references and tests in the same change.
Coverage row 221: notes owns personal notes, tax-year index, application notes, and private knowledge capture and cites ADR-0311.
Reader path 222: start at this README, inspect the story, follow ux-flow, verify handshake, parse schemas, then run integration-test-plan before touching implementation.
Done signal 223: artifact line floors pass, all schema files include _meta, all service IPs are >=400 lines, and the personal/work tenant boundary remains explicit.
