# G030-L VCS-registry live-reader and frozen-companion proof — 2026-08-02

State: **PLANNING_ONLY — ONE ROW GRAPH-WIRED; FOUR FROZEN HISTORICAL ROWS RETAINED; NO VCS REACTIVATION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-K-DESIGN-SYSTEM-RESIDUAL-CONSUMER-CATALOG-GAP-PROOF-2026-08-02.md`.  
No registry row, retired VCS implementation, gate, ADR, policy, PR, GitOps declaration, or cluster state was changed.

## Result

The five-row `registry/vcs/*` residual is a mixed family. ADR-0363 freezes the retired agentic-VCS registry as historical evidence, but the current tree still contains two Rust governance gates that default-read the changeset event log. That measured reader promotes only the exact log row; it does not reactivate the retired VCS substrate or imply executable edges for its companions.

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/vcs/changeset-event-log.json` | exact default input to the current Rust changeset-state monotonicity and enum-closed validators; exact lane-input globs in the governance gate catalog | `GRAPH_WIRED_INPUT — LIVE GOVERNANCE GATE READER OF FROZEN EVIDENCE` |
| `registry/vcs/concurrent-safe-paths.yaml` | ADR-0363-frozen historical companion; exact current machine reader not found | `POLICY_PROTECTED_MACHINE_ARTIFACT — FROZEN HISTORICAL COMPANION` |
| `registry/vcs/event-router.yaml` | explicitly frozen with the event log by ADR-0363; retired webhook receiver absent; exact current machine reader not found | `POLICY_PROTECTED_MACHINE_ARTIFACT — FROZEN HISTORICAL EVIDENCE` |
| `registry/vcs/webhook-delivery-log.json` | empty retired webhook-delivery scaffold; retired webhook receiver absent; exact current machine reader not found | `POLICY_PROTECTED_MACHINE_ARTIFACT — FROZEN HISTORICAL COMPANION` |
| `registry/vcs/README.md` | directory-level freeze and historical lifecycle contract; its active-consumer prose is stale; no machine reader | `POLICY_PROTECTED_MACHINE_ARTIFACT — FROZEN CONTRACT DOCUMENTATION` |

This promotes one row from the protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 918 `GRAPH_WIRED_INPUT` + 106 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 87 non-fixture rows. No row becomes a deletion candidate.

## Executable event-log edge

`marketplace/facade/dev-cli/src/changeset_state_gates.rs` remains present at the immutable tip and:

1. defines `DEFAULT_EVENT_LOG` as exactly `registry/vcs/changeset-event-log.json`;
2. reads and parses that file as JSON;
3. requires a top-level `events` array;
4. validates each event's `changeset_id` and `to_state` fields;
5. exposes monotonic-state-progression and closed-enum validators;
6. has tests, including an explicit empty-log success case.

`libs/oya-governance-gate-catalog-domain/src/lib.rs` separately retains both lane IDs and maps each lane to the exact event-log path:

- `changeset-state-monotonicity`;
- `changeset-state-enum-closed`.

The committed log is `{"events":[]}`. It is therefore structurally consumed but vacuous as behavioral evidence: the live validators accept zero events. This proves a source-graph reader, not meaningful changeset-history coverage or protected required-context execution.

No dedicated changeset-state Buck target or affected-set expectation was found in the bounded paths probed at this tip. The Rust module lives inside the broader retired CLI facade. Source presence plus gate-catalog registration does not establish that these lanes execute in every `oya-ci-required` run.

## Frozen companion boundary

ADR-0363 Accepted retirement authority states that:

- the bespoke changeset-state, merge-queue, promotion, webhook, indexing, lockstore, and changebundle crates are retired;
- ADR-0110, ADR-0112, and ADR-0113 are superseded;
- `registry/vcs/changeset-event-log.json` and the event router are frozen as historical evidence — not deleted and not active;
- the former VCS CLI and wrapper are retired in favor of the governance/cloud-ci admission substrate.

Immutable existence probes confirm the named historical implementations are absent, including the old changeset-state kernel, VCS service trees, and merge-queue fix-loop app. The current tree has no exact external machine reference to `concurrent-safe-paths.yaml`, `event-router.yaml`, or `webhook-delivery-log.json`; their remaining exact citations are historical ADR/audit prose. Both JSON logs are empty (`events: []`, `deliveries: []`).

The README itself declares the family frozen but still describes old writers, consumers, Jenkins lanes, and Cargo commands whose named implementations are absent. Those descriptions are historical contract evidence, not proof of current execution.

`concurrent-safe-paths.yaml` has one non-superseded prose citation in ADR-0116's explanation of worktree isolation and admission-time coordination. That citation is retention evidence only: no current parser edge was found.

## Authority/readership tension

The event log is simultaneously:

- **historically frozen and non-active** under ADR-0363; and
- **currently read by live Rust gate code and named as two lane inputs**.

G030 records rather than resolves that tension. It does not infer that the retired VCS state machine should return, and it does not delete a live gate input. The owning governance lifecycle must decide whether to:

1. retain the empty frozen log as an intentional compatibility fixture and make that status executable;
2. move the state-enum/monotonicity contract to an owned governance fixture or policy source; or
3. retire the two residual lanes and their reader if they no longer express admission semantics.

Until that owner decision and independent review, the smallest safe disposition is one graph-wired row plus four policy-protected frozen rows.

## Anti-vacuity and semantic boundary

Proven:

- immutable VCS residual size = five rows;
- the current Rust module defaults to the exact event-log path and parses it;
- the gate catalog maps two lane IDs to the exact path;
- the committed event log is structurally valid and empty;
- the committed webhook delivery log is empty;
- retired VCS implementation paths probed are absent;
- no exact current machine consumer was found for the three non-README companions;
- ADR-0363 explicitly freezes the event log and event router as historical evidence.

Not proven:

- execution of either residual lane in the protected required context;
- any non-vacuous changeset history or transition coverage;
- any current parser for the event router, delivery log, or concurrent-safe paths;
- consistency between the README's historical commands and current repository behavior;
- owner approval to rewrite, move, declassify, or delete any row.

## Verification boundary

Evidence came from immutable tree enumeration, exact path and basename searches, JSON shape/count inspection, current Rust reader and gate-catalog source, Buck/affected-set bounded searches, ADR-0363, and immutable existence probes at `b651080374113aeb57500eecbd9d1326f0404e48`. No local CLI execution is used as merge authority.

An independent Explore audit of this family failed with the encrypted-content transport error. It remains `FAILED_TRANSPORT_NOT_APPROVE`; the mechanical proof is not independent approval.

## Non-actions and non-claims

- No frozen registry row edited or deleted.
- No retired VCS substrate reactivated.
- No claim that an empty log proves state-machine behavior.
- No claim that gate-catalog registration proves required-context execution.
- No companion promoted from historical prose alone.
- No move-plan JSON, generated face, or multispectrum evidence surface added.
- No independent APPROVE; transport failure remains non-approval.
