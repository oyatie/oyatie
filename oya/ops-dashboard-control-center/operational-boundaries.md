# Ops Dashboard / Control Center Operational Boundaries

## Boundary rules

- The control center records decisions; GitOps controllers perform deployment mutation.
- The control center reads cluster health; it does not open SSH sessions or mutate hosts directly.
- The control center requests evidence exports; audit-chain and object-store substrates produce tamper-evident records.
- The control center links regional escalation runbooks; localization packs own pack-specific regulatory content.

## Incident and capacity boundaries

- Incident command may approve remediation only through declared T3 capabilities.
- Capacity views are observed signals and must not silently scale workloads without a separate approved automation changeset.
- Unknown health is not green.

## Acceptance criteria

- Mutating operator actions are distinguishable from observed health signals.
- All state transitions cite command/event contracts.
- Runtime implementation remains blocked until policy fixtures, SLO evidence, and restore evidence exist.
