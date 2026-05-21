---
microservice: calendar
doc_class: README
date: 2026-05-21
owner_team: axis-calendar
status: Accepted
related_adrs: [ADR-0244, ADR-0248, ADR-0251, ADR-0329, ADR-0330, ADR-0331]
---

# calendar

Calendar owns event storage, recurrence expansion, free/busy resolution, room booking, invitation flows, ICS import/export, CalDAV/JMAP interoperability, and scheduling handoff surfaces.

## Tenant class model

Calendar follows the `tenant_class` model from [ADR-0330](../../docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md). Customer access is expressed as `tenant_class = demo_trial | paid`; paid billing is composed from `billing_components` (`revenue_share`, `per_seat`, `per_usage`). Calendar capabilities are not segmented by customer capability ladders. Demo-trial limits are usage caps, and regulated calendar behavior belongs to compliance packs or cell topology.

## Canonical surfaces

- [PRD.md](PRD.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [manifest.json](manifest.json)
- [policy/](policy/)
- [slos/](slos/)
- [runbooks/](runbooks/)
