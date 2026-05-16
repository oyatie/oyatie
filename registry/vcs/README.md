# registry/vcs/

Append-only event log for the agentic-VCS pipeline per ADR-0110.

## `changeset-event-log.json`

Canonical event-sourced state log for every changeset traversing the
`opened → working → verified → pr_open → ci_running → ci_passed →
reviewed → merged_dev → staged → produced` pipeline (10 advancing
states) plus the three terminal-fail states (`abandoned`, `rejected`,
`cost_exhausted`).

### Append-only invariant

- Events MUST be appended only. Existing rows MUST NOT be edited or
  removed. Replays + audits depend on the historical integrity of the
  log.
- Every row carries a unique `dedup_key` (`<changeset_id>_<to_state>_<at>`).
  Webhook receivers MUST check the dedup_key before appending; a
  repeated dedup_key is a no-op.
- The non-decreasing-subsequence invariant from ADR-0110 §"Monotonic
  invariant" is enforced by the `oya-foundry-fitness-changeset-state-monotonicity`
  CI lane (`oya gate validate changeset-state-monotonicity`).
- Every `to_state` value MUST be one of the 13 closed-enum values
  emitted by `ChangesetState::as_wire()` in
  `crates/oya-foundry-vcs-changeset-state-kernel`. Drift is caught by
  the `oya-foundry-fitness-changeset-state-enum-closed` CI lane.

### Writer

The canonical writer is `oya-foundry-vcs-changeset-state-app`:

```text
cargo run -q -p oya-foundry-vcs-changeset-state-app -- \
    append --changeset cs_<ulid> --to-state opened \
    --emitted-by <agent-id> [--evidence k=v,k=v]
```

The app reads the current file, runs the monotonic-event-log validator
against `existing || candidate`, then atomic-writes via `tmp + rename`.

### Signatures

Each row carries an Ed25519 `signature` field per ADR-0058. IP-001
(wave-A) stamps an `ed25519-stub:<base64>` placeholder; the
real-Ed25519-keyed wiring is wave-B.
