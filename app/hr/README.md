# HR

Owner: `app/hr`

Status: portable-app migration; domain foundation only

HR is the tenant-portable People and employment application. It owns employee
and employment records, organization/manager relations, onboarding readiness,
leave policy projections, labor-compliance decisions, sensitive-read policy,
and HR evidence references.

The landed Rust domain and in-process adapters are test foundations. They do
not yet constitute a durable service, sold network facade, installed-pack
integration, downstream delivery path, or measured SLO. The current direct
Data/Gateway dependencies and volatile storage are migration debt.

Canonical owner law:

- [ADR.md](ADR.md) — decisions and portability boundaries
- [PRD.md](PRD.md) — product requirements, acceptance, and SLO objectives
- [SPEC.md](SPEC.md) — current contract and target transaction/fault semantics
- [PLAN.md](PLAN.md) — L2a through L2f implementation sequence

HR does not own payroll calculation/disbursement, accounting, workflow
execution, audit-chain persistence, IAM/PDP, Data/Storage/Gateway engines,
notification delivery, or deployment infrastructure. Those effects cross
HR-owned ports and replaceable adapters.
