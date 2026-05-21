---
microservice: recordings
doc_class: README
date: 2026-05-21
owner_team: axis-recordings
status: Accepted
related_adrs: [ADR-0244, ADR-0248, ADR-0251, ADR-0329, ADR-0330, ADR-0331]
---

# recordings

Recordings owns ingest, media segments, transcripts, search, redaction, chapter summaries, retention, legal hold, playback/share links, exports, and translation handoff surfaces.

## Tenant class model

Recordings follows the `tenant_class` model from [ADR-0330](../../docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md). Customer access is expressed as `tenant_class = demo_trial | paid`; paid billing is composed from `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Recording quality and retention correctness are not segmented by customer capability ladders. Demo-trial limits are storage, duration, export, or usage caps; sovereign and regulated retention behavior belongs to compliance packs or cell topology.

## Canonical surfaces

- [PRD.md](PRD.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [manifest.json](manifest.json)
- [policy/](policy/)
- [slos/](slos/)
- [runbooks/](runbooks/)
