---
doc_class: Runbook
title: Aggregation Index Rebuild
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry
severity_default: Sev-3 (operational; Sev-2 if PR-blocking)
related_failure_modes: [F-04, F-15]
related_artifacts:
  - microservices/governance/failure-modes.md
review_cadence: quarterly
doc_status: published
---

# Runbook: Aggregation Index Rebuild

## When to invoke

- `oya-check-aggregation-index-generation` lane fails (F-04 divergence detected).
- Aggregation-indexer scoped-PAT overrun (F-15).
- Manual hand-edit of central index detected (any of `docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`).
- Onboarding a new pack (cleanup of stale entries).

## Pre-flight

- You are: axis-foundry on-call (or ops-security if scope-overrun detected).
- You have: GitHub PAT with scope to aggregation-index paths; `cargo run -p oya-dev-cli` workspace ready.

## Decision tree

```text
                Why is rebuild needed?
                  ├─ F-04 divergence       → §A
                  ├─ F-15 scope overrun    → §B
                  ├─ Hand-edit detected    → §C
                  └─ New pack onboarding   → §D
```

## §A — F-04 divergence

1. **Confirm**:
   ```bash
   cargo run -p oya-dev-cli -- gate validate aggregation-index-generation
   ```
   Expect: BLOCKER with `aggregation-divergence` Finding citing specific paths.

2. **Diagnose** which µservice changed:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation diff --output /tmp/diff.json
   ```
   Identify the per-µservice source whose addition/edit triggered divergence.

3. **Rebuild** with lock held:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation rebuild --hold-lock
   ```
   Aggregation-indexer worker holds Postgres advisory lock; concurrent commits queue.

4. **Verify**:
   ```bash
   cargo run -p oya-dev-cli -- gate validate aggregation-index-generation
   ```
   Expect: PASS.

5. **Commit** regenerated indices via scoped PAT:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation commit \
     --branch dev --signed-by axis-foundry-bot
   ```

6. **Verify** post-commit lane re-runs PASS.

## §B — F-15 scope overrun (HIGH severity per T-E-03)

(Sev-2)

1. **Confirm**: `cargo run -p oya-dev-cli -- governance aggregation scope-status` → expect overrun event.
2. **Capture** state for forensics: dump scope-overrun event + diff at the time:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation event-log --since <unix> > /tmp/scope-overrun-<id>.json
   ```
3. **Revert** any unauthorized writes:
   ```bash
   git revert <offending-commit-sha>
   ```
4. **Investigate** root cause: indexer logic bug? PAT scope mis-set? Race condition?
5. **Engage** ops-security: review pre-push hook + PAT scope:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation pat-scope review
   ```
6. **Quarterly review** of PAT scope per `compliance.md` ISO 27001 A.5.15.
7. **Postmortem** at `evidence/audits/postmortems/<incident-id>.md`.

## §C — Hand-edit detected

1. **Identify** the hand-edit commit:
   ```bash
   git log --diff-filter=M --pretty=format:"%H %an %s" -- docs/prds/INDEX.md registry/catalog/ /specs/microservices/ | head -20
   ```
   Look for commits NOT authored by `axis-foundry-bot` per `policy/ci-scope.cedar` P5.

2. **Revert** the hand-edit:
   ```bash
   git revert <hand-edit-sha>
   ```

3. **Regenerate** from per-µservice sources:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation rebuild
   ```

4. **Author notification**: comment on the hand-edit commit + open issue on the offending PR; refer author to per-µservice source-of-truth rule per ADR-0131.

5. **Tighten** branch protection if recurring: refuse direct push to central-index paths absent `axis-foundry-bot` author claim.

## §D — New pack onboarding

1. **Pre-check** that pack-specific aggregation paths exist:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation pack-paths --pack <pack>
   ```
2. **Run** initial rebuild with pack scope:
   ```bash
   cargo run -p oya-dev-cli -- governance aggregation rebuild --pack <pack>
   ```
3. **Verify** per-pack overlay is honoured (e.g., pack-specific KMS keyring; per-pack S3 bucket prefix).
4. **Document** the pack-specific aggregation entry in `multi-region.md` Roadmap row for the new pack.

## Verification

```bash
cargo run -p oya-dev-cli -- gate validate aggregation-index-generation
```

Expect: PASS.

Smoke test:
```bash
# Open a no-op PR; aggregation-index lane should pass cleanly.
git checkout -b sanity-check-<id>
echo "<!-- noop -->" >> README.md
git add README.md && git commit -m "test: sanity check post-rebuild"
gh pr create --title "Sanity check post-rebuild" --body "Empty diff to verify aggregation lane"
```

Verify the new PR's `oya-check-aggregation-index-generation` lane reports PASS.

## Stand-down criteria

- `oya-check-aggregation-index-generation` lane returns PASS.
- No further divergence alerts in 1h.
- Central indices reflect current per-µservice sources.

## Post-incident actions

- File postmortem within 1 week.
- Update this runbook if new pattern observed.
- File successor-IP IP for any structural change to indexer logic or PAT scope.

## References

- `microservices/governance/failure-modes.md` F-04, F-15.
- `microservices/governance/threat-model.md` T-E-03.
- `microservices/governance/policy/ci-scope.cedar` (P5, F4).
- ADR-0131 §"What stays central".
- ADR-0115 (registry consolidation).
