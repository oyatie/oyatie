# G026 retirement-candidate census — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Deletion: **not activated for any row**.
Independent explore transport: `FAILED_TRANSPORT`; coordinator mechanical census against origin/dev follows.

## `tools/oya-bot-autofix-app`

- Exact-name references: 16.
- Live authority: `docs/decisions/ADR-0531-auto-remediation-delivery-oya-bot-autofix.md:71-78` names its catalog, BUCK, Cargo, OWNERS, library, binary, and dry-run test.
- Live registry membership: `specs/capability-registry.json:627`.
- Disposition: **KEEP_PENDING / MOVE to the delivery-fabric remediation face after ADR-0531 reconciliation**. Not DELETE_CANDIDATE: explicit product ADR outweighs generic tooling-retirement pressure until superseded mechanically.

## `tools/oya-tooling-agent-read`

- Exact-name references: 243.
- Live root contract: `Cargo.toml:101` names it as the sole doctrinal carve-out locked by CLAUDE.md.
- Live gates/docs include `ci/facade/baseline-ratchet/tests/gate_registration.rs:986`, `ci/facade/crate-layer-suffix/src/lib.rs:13,298`, module membership policy, and numerous active agent contracts.
- Contradiction: ADR-0116/historical audit text says external coordination tooling is retired, but live root/gate contracts still special-case this crate.
- Disposition: **KEEP_PENDING** until the root contract and every live consumer are atomically rewritten/retired. Not DELETE_CANDIDATE. Absence-of-use cannot be inferred from command practice while 243 tracked references remain.

## `tools/oya-lane-supervisor-app`

- Exact-name and alias census found tracked membership/self surfaces plus delivery-fabric lineage; no admissible proof of zero live semantic consumers.
- Disposition: **KEEP_PENDING / MOVE only after ADR-0516..0535 ownership mapping**. Not DELETE_CANDIDATE without an owned consumer graph and explicit supersession.

## Result

The earlier draft's two DELETE_CANDIDATE labels are withdrawn. Current totals become:
- MOVE / destination class known: 16 (including architecture graph generator and adapter substitution test after importer census)
- KEEP_PENDING / authority reconciliation: 4 (`tooling-agent-read`, `lane-supervisor`, `bot-autofix`, `fabric-loop-state`)
- KEEP_THEN_MOVE_LATE: 1 (`reorg-codemod`)
- DELETE_CANDIDATE: 0

No executable tools move plan is authorized yet; destination-class batches and independent review remain prerequisites.
