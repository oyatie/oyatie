# Flags

Owner: `flags`

Status: deterministic evaluation kernel only; structural cleanup before new
behavior

Flags owns tenant-scoped runtime dynamic configuration: deterministic typed
evaluation, targeting, percentage assignment, emergency kill switches, and
pack-gated overrides supplied through verified Policy C0 context.

The landed `core/evaluation-domain` is a tested pure Rust kernel. No mutation
service, durable source, Connect/OFREP facade, snapshot distribution, deployed
evaluator, authorization/audit path, live metric producer, or qualified SLO has
landed. `core/server` and the retained Cedar/IaC/OpenSLO corpus are retirement
residue, not runtime evidence.

Canonical owner law:

- [ADR.md](ADR.md) — charter and binding decisions
- [PRD.md](PRD.md) — product requirements, SLO objectives, and failure bar
- [SPEC.md](SPEC.md) — current semantics and target authority/evaluation model
- [PLAN.md](PLAN.md) — L1a through L1f implementation sequence

Flags does not own experiment statistics, code-deploy admission, cell topology,
a clock, the Policy engine, or a trusted-tenant bypass. The canonical future
wire surface is Connect; OpenFeature/OFREP remains an adapter.
