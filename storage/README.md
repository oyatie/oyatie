# Storage

`storage/` owns durable bytes: the object/CAS engine and, after separate
promotion, an EBS-class block facade. Relational and analytical records belong
to `data/`; Drive and other end-user products belong under `app/`.

Read the owner law before changing this directory:

- [ADR.md](ADR.md) — decisions in force
- [PRD.md](PRD.md) — product requirements and promotion targets
- [SPEC.md](SPEC.md) — behavior and protocol contract
- [PLAN.md](PLAN.md) — remaining sequenced work

The current tree is a pre-production contract baseline: typed Rust domain/API
libraries, an in-memory CAS reference, and non-network compatibility adapter
projections. It is not yet a persistent distributed storage service.
