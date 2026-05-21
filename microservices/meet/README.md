---
microservice: meet
doc_class: README
date: 2026-05-21
owner_team: axis-meet
status: Accepted
related_adrs: [ADR-0244, ADR-0248, ADR-0251, ADR-0329, ADR-0330, ADR-0331]
---

# meet

Meet owns meeting rooms, participants, media tracks, screen share, recording bridge handoff, transcription, webinars, live-stream egress, and end-to-end meeting encryption surfaces.

## Tenant class model

Meet follows the `tenant_class` model from [ADR-0330](../../docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md). Customer access is expressed as `tenant_class = demo_trial | paid`; paid billing is composed from `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Real-time quality targets stay uniform across tenant classes. Demo-trial limits are participant, duration, or usage caps; regulated meeting behavior belongs to compliance packs or cell topology.

## Canonical surfaces

- [PRD.md](PRD.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [manifest.json](manifest.json)
- [policy/](policy/)
- [slos/](slos/)
- [runbooks/](runbooks/)
