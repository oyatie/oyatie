# registry/vcs/

> **FROZEN (historical evidence) — ADR-0363 §2.** The bespoke agentic-VCS
> pipeline is retired in favour of git + Jenkins + GitHub (interim).
> The orchestration crates this registry referenced (changeset-state machine,
> merge-queue, promotion-controller, webhook-receiver, ast-index,
> polyglot-indexer, lockstore, changebundle, ci-fix-loop-dispatcher) were
> deleted in the ADR-0363 PR-2 cutover. This directory (`event-router.yaml`,
> `changeset-event-log.json`) is preserved as historical evidence — **not
> active, not deleted, not edited**. ADR-0110/0112/0113 are Superseded.

Append-only event log + canonical config for the agentic-VCS pipeline
per ADR-0110 (changeset state machine) and ADR-0112 (webhook receiver
substrate).

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
  invariant" is enforced by the `changeset-state-monotonicity` governance gate
  lane (`oya gate validate changeset-state-monotonicity`), run by the Jenkins CI
  (ADR-0361; the retired GitHub Actions job is superseded).
- Every `to_state` value MUST be one of the 13 closed-enum values
  emitted by `ChangesetState::as_wire()` in
  `crates/oya-vcs-changeset-state-kernel`. Drift is caught by
  the `governance-changeset-state-enum-closed` CI lane.

### Writer

The canonical writer is `oya-vcs-changeset-state-app`:

```text
cargo run -q -p oya-vcs-changeset-state-app -- \
    append --changeset cs_<ulid> --to-state opened \
    --emitted-by <agent-id> [--evidence k=v,k=v]
```

The app reads the current file, runs the monotonic-event-log validator
against `existing || candidate`, then atomic-writes via `tmp + rename`.

### Signatures

Each row carries an Ed25519 `signature` field per ADR-0058. IP-001
(wave-A) stamps an `ed25519-stub:<base64>` placeholder; the
real-Ed25519-keyed wiring is wave-B.

## `event-router.yaml`

Canonical `(event, action [, conclusion]) -> Foundry-agent` mapping
consumed by `oya-vcs-webhook-receiver-app` per ADR-0112
§"Event-router table". The receiver verifies HMAC, dedups, then looks
up the routed agent against this table. New rows MUST go through an
ADR amendment (no silent additions). Completeness is asserted by the
`governance-event-router-completeness` CI lane (ADR-0112
wave-C).

Schema:

```yaml
rows:
  - event: pull_request
    action: opened     # "" is the row-side wildcard
    conclusion: ""     # optional; matches workflow_run.conclusion
                       # or check_suite.conclusion; "" / missing
                       # means the row ignores conclusion
    agent: <foundry-agent-name>
    purpose: <one-line audit-friendly description>
```

Lookup precedence (most-specific row wins): exact
`(event, action, conclusion)` -> exact `(event, action)` with no
conclusion declared -> row-side action wildcard. ADR-0112 §"Event-router
table" splits `workflow_run.completed` by `conclusion=success` (IP-004
PR review) vs `conclusion=failure` (IP-005 fix-loop); any
other conclusion (`cancelled`, `timed_out`, `skipped`) falls through to
`RoutingFailed` so the completeness lane alerts.

## `webhook-delivery-log.json`

Append-only dedup table for GitHub webhook deliveries per ADR-0112
§"Idempotency contract". Initialized empty (`{"deliveries": []}`).
Every appended row is shaped:

```json
{
  "delivery_id": "<X-GitHub-Delivery uuid>",
  "event":       "<X-GitHub-Event header>",
  "action":      "<payload action field>",
  "dedup_outcome": "accepted | deduplicated | routing_failed | agent_invocation_failed",
  "at_seconds":  1715000000
}
```

The 7-day TTL is applied at lookup time inside
`oya-vcs-webhook-receiver-kernel::find_dedup_status`; expired rows
are reported as `Expired` so the GC + fresh-route path runs without
appending a duplicate. Monotonic invariant (no `delivery_id` appearing
twice with conflicting outcomes) is asserted by the
`governance-webhook-delivery-log-monotonic` CI lane
(ADR-0112 wave-C).
