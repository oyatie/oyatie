---
purpose: Oyatie Runbook — Forgejo Board Verification Checklist
doc_status: published
---

# Oyatie Runbook — Forgejo Board Verification Checklist

> **Status:** Active checklist for the Forgejo board spike and future
> implementation acceptance.
> **Owner:** Governance / Intelligence delivery leads per RACI.
> **Last verified:** 2026-05-27 (docs-only checklist authored from the
> Forgejo board spike scope).

## Scope

Use this checklist to decide whether the Forgejo-backed agent board is ready to
move from spike evidence to implementation acceptance. Record each check as
PASS, FAIL, or BLOCKED with the command, timestamp, actor, Forgejo URL, and raw
response artifact.

This checklist is documentation-only. It does not authorize edits to ADR-0377,
code, tests, Cargo files, `.omx`, or `.omc`.

## Required evidence packet

Create one evidence packet per verification run containing:

1. Forgejo base URL, version, commit SHA if exposed, and authentication mode.
2. Redacted HTTP transcripts for every REST probe.
3. Git command transcripts for claim-ref compare-and-swap (CAS) tests.
4. Webhook delivery payload samples with headers and receiver logs.
5. Affected-only local verification command transcript and file list.
6. Final checklist table with PASS/FAIL/BLOCKED and owner for every failure.

## Checklist

### 1. Live Forgejo health/version

- **PASS:** Authenticated and unauthenticated probes reach the intended
  self-hosted Forgejo instance and return a stable version/build identity.
- **FAIL:** A probe reaches the wrong host, requires undocumented credentials,
  omits version/build identity, or only works on a local developer machine.
- **Evidence:** `GET /api/healthz` or equivalent health endpoint,
  `GET /api/v1/version`, response headers, and TLS certificate summary.

### 2. Projects REST endpoint absence

- **PASS:** Probes confirm the target Forgejo version does not expose a stable
  Projects REST endpoint suitable for board automation, or document the exact
  supported endpoint if it appears.
- **FAIL:** Automation assumes a Projects REST endpoint without live proof, or
  endpoint behavior differs between authenticated users.
- **Evidence:** `GET /api/v1/repos/{owner}/{repo}/projects`,
  `GET /api/v1/projects`, status codes, and API docs/version note.

### 3. Exclusive label projection

- **PASS:** A deliverable maps to exactly one active board-state label in the
  configured exclusive label family, and re-projection is idempotent.
- **FAIL:** Two active state labels remain on one issue, projection removes
  unrelated labels, or repeated sync changes the issue after the first run.
- **Evidence:** Before/after issue label JSON, sync command transcript, and
  second-run no-op transcript.

### 4. Non-atomic assignee race

- **PASS:** Concurrent assignee updates are proven non-atomic or safely
  serialized by the chosen implementation guard; losing writes are detected and
  retried or refused.
- **FAIL:** Two writers silently overwrite each other, final assignee differs
  from both recorded intents, or no race evidence exists.
- **Evidence:** Two concurrent update transcripts, final issue JSON,
  retry/refusal log, and timing notes.

### 5. Git-ref CAS exactly-one-wins

- **PASS:** Two concurrent claim attempts against the same claim ref produce
  exactly one successful ref update and one rejected stale update.
- **FAIL:** Both writers succeed, both fail without a clear winner, or the loser
  cannot identify the winning ref value.
- **Evidence:** `git update-ref` or push-with-lease transcript, old/new SHAs,
  loser error, and final ref value.

### 6. Push webhook sender identity

- **PASS:** Push webhook payload identifies the actor, repository, ref,
  before/after SHAs, and delivery ID needed to project board state and audit who
  changed it.
- **FAIL:** Sender is anonymous or ambiguous, ref/SHA fields are missing,
  delivery cannot be replayed, or receiver logs cannot correlate the delivery.
- **Evidence:** Raw webhook payload, headers, receiver log line, and redaction
  note.

### 7. Affected-only local verification

- **PASS:** The local verifier derives an affected file/surface set from the
  change and runs only the relevant checks before broader CI.
- **FAIL:** Verifier runs unrelated global checks by default, skips changed
  surfaces, or cannot explain why each check was selected.
- **Evidence:** Changed-file list, affected-surface mapping, command
  transcript, and selected-check rationale.

### 8. ADR-0377 conditional lift criteria

- **PASS:** Every checklist item is PASS, remaining blockers have owner/date,
  and the implementation plan names the evidence needed to lift ADR-0377 from
  conditional.
- **FAIL:** Any required check is FAIL/BLOCKED without an owner/date, or
  ADR-0377 is proposed for lift based on assumptions instead of live evidence.
- **Evidence:** Completed table, blocker register, proposed ADR-0377 lift note,
  and reviewer sign-off.

## Probe commands

Adjust host, owner, repo, token, and ref names for the target environment. Store
full output in the evidence packet; do not paste secrets into the runbook.

```bash
forgejo_url="https://forgejo.example.invalid"
owner="oyatie"
repo="oyatie"
token="${FORGEJO_TOKEN:?set FORGEJO_TOKEN}"

curl -fsS "$forgejo_url/api/healthz"
curl -fsS "$forgejo_url/api/v1/version"
curl -isS -H "Authorization: token $token" \
  "$forgejo_url/api/v1/repos/$owner/$repo/projects"
curl -isS -H "Authorization: token $token" \
  "$forgejo_url/api/v1/projects"
```

For claim-ref CAS, run two writers against the same expected old SHA and prove
only one update wins:

```bash
claim_ref="refs/heads/claims/fd001-example"
old_sha="$(git rev-parse "$claim_ref" 2>/dev/null || printf '%040d' 0)"
new_sha_a="$(git rev-parse HEAD)"
new_sha_b="$(git rev-parse HEAD~1)"

git update-ref "$claim_ref" "$new_sha_a" "$old_sha" &
git update-ref "$claim_ref" "$new_sha_b" "$old_sha" &
wait

git rev-parse "$claim_ref"
```

## Acceptance gate

ADR-0377 may be proposed for lift from conditional only when:

1. All required checklist rows are PASS against a live self-hosted Forgejo
   instance, not mocks alone.
2. Projects REST endpoint behavior is proven and the board design does not
   depend on an absent endpoint.
3. Exclusive label projection is idempotent and preserves unrelated labels.
4. Assignee races and claim-ref CAS races have deterministic loser handling.
5. Push webhook identity is sufficient for audit-chain attribution.
6. Affected-only local verification has a reproducible changed-file mapping.
7. The evidence packet is linked from the implementation acceptance record.
8. Reviewer sign-off cites the evidence packet and names any deferred risks.

## Failure handling

- Mark the row FAIL and stop lift-to-implementation claims for that surface.
- File a bounded follow-up with owner, date, and required evidence.
- Re-run only failed rows plus any row affected by the fix.
- Do not lift ADR-0377 from conditional while any required row is FAIL or
  BLOCKED without an accepted exception.

## Sources

- ADR-0377 (conditional authority for the Forgejo board spike).
- Forgejo live instance API responses captured during each verification run.
- Git ref CAS transcripts captured from the implementation worktree.
