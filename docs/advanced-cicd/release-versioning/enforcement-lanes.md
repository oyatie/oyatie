---
doc_class: Enforcement
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Six new fitness lanes that mechanically enforce the release-versioning policy:
  SemVer discipline, API version stability, release-branch cut correctness,
  cherry-pick trail integrity, EOL warning, and deprecation notice.
planned_enforcement_ref: self
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Enforcement Lanes — Release Versioning

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Lane catalogue

| Lane ID | Status | Severity | Scope | What it verifies |
|---|---|---|---|---|
| `governance-semver-discipline` | **NEW** | BLOCKER | every PR touching `crates/oyatie-*/src/**` | cargo-semver-checks clean against `origin/prod` baseline per [`crate-versioning-spec.md`](crate-versioning-spec.md) |
| `governance-api-version-stability` | **NEW** | BLOCKER | every PR touching `contracts/openapi/**` | No removal / type change / rename in stable path-versions per [`api-versioning-spec.md`](api-versioning-spec.md) §6 |
| `governance-version-eol-warning` | **NEW** | HIGH (BLOCKER on EOL day) | nightly + per-PR | 90-day pre-EOL warning emitted per [`version-eol-policy.md`](version-eol-policy.md) §3 |
| `governance-release-branch-cut` | **NEW** | BLOCKER | release-cherry-pick agent invocation | Tag + branch + workspace version + lanes-green pre-cut all consistent per [`release-branch-cut-spec.md`](release-branch-cut-spec.md) §3 |
| `governance-cherry-pick-trail` | **NEW** | HIGH | release branch commits | Every commit on `release/X.Y` traces to a prod SHA via the agent's evidence chain per [`release-cherry-pick-agent-spec.md`](release-cherry-pick-agent-spec.md) §8 |
| `governance-deprecation-notice` | **NEW** | BLOCKER | breaking-change PRs | 180-day sunset row present in SUNSET-LEDGER + ADR + dual reviewer-agent approval per [`breaking-change-process.md`](breaking-change-process.md) §4-§6 |

## 2. Lane wiring

```
pre-merge:
  - semver-discipline           (BLOCKER)
  - api-version-stability       (BLOCKER on contracts/openapi/** diff)
  - deprecation-notice          (BLOCKER on breaking_change PR)

pre-release (cut time):
  - release-branch-cut          (BLOCKER)
  - (re-run) semver-discipline  (BLOCKER on baseline drift)

post-release / nightly:
  - version-eol-warning         (HIGH; BLOCKER on EOL day)
  - cherry-pick-trail           (HIGH; emits divergence report)
```

All lanes run inside `governance-quality-lane-kernel` (existing) and emit
signed evidence rows to D14 per ADR-0050.

## 3. `governance-semver-discipline` (NEW · BLOCKER)

Inputs: PR diff against `origin/prod`, per-crate `Cargo.toml`, public-API
manifest.

Checks:
1. `cargo semver-checks check-release --baseline-rev origin/prod` clean per
   crate touched.
2. If violations exist → fail UNLESS PR carries `breaking_change: true`
   frontmatter + linked ADR + dual reviewer-agent approval.
3. Workspace lockstep invariant (Phase A): all `oyatie-*` versions match.
4. Phase B (post-W-Foundry-Preview): per-crate major bumps consistent with
   layer rules (`platform-*` shared major, `foundry-*` shared major).

Outputs: PR comment with the SemVer report; D14 evidence row.

## 4. `governance-api-version-stability` (NEW · BLOCKER)

Inputs: PR diff on `contracts/openapi/**`, prior commit's OpenAPI spec,
`x-stability` / `x-introduced` / `x-deprecated` / `x-sunset` fields.

Checks:
1. No field removed from a `v1`+ stable path-version.
2. No type change in a stable path-version.
3. No field rename without an alias + deprecation row.
4. Every new operation has `x-introduced` ≥ today.
5. Every `x-deprecated` has matching `x-sunset` exactly 180 days later.
6. Path-version promotion (alpha → beta → stable) carries an ADR link.
7. New major path-version (`v2`) carries the breaking-change ADR.

Outputs: structured diff report; BLOCKER if any check fails.

## 5. `governance-version-eol-warning` (NEW · HIGH)

Runs nightly via cron + per-PR for PRs targeting a release branch.

Checks:
1. Read `EOL-LEDGER.md`.
2. For each active major, compute days-to-EOL.
3. If `≤ 90` → emit `EVT-VERSION-EOL-APPROACHING` + PR comment + notice file.
4. If `≤ 0` → switch branch protection to read-only + escalate to BLOCKER for
   any new PR.
5. For each `x-deprecated` path-version, compute days-to-sunset; same
   thresholds.

Outputs: nightly digest in `docs/release/EOL-DIGEST.md`; D14 row.

## 6. `governance-release-branch-cut` (NEW · BLOCKER)

Invoked at branch-cut time by the `release-cherry-pick` agent or operator.

Checks:
1. All BLOCKER lanes green on the prod SHA being cut.
2. No breaking-change PRs pending sunset on this major.
3. Tag `vX.Y.0` does not already exist.
4. Branch `release/X.Y` does not already exist.
5. Workspace `Cargo.toml` version stamped to `X.Y.0` on the cut commit.
6. Branch-protection policy applied immediately on creation.
7. `EVT-RELEASE-BRANCH-CUT` evidence row emitted with full chain.

Outputs: success → cut proceeds; failure → cut refused, operator paged.

## 7. `governance-cherry-pick-trail` (NEW · HIGH)

Runs weekly + on every cherry-pick commit.

Checks:
1. Every commit on `release/X.Y` (except the cut commit) has a matching
   commit on `origin/prod` (by content hash or `git cherry`).
3. Every patch tag (`vX.Y.Z`, Z ≥ 1) has a `gh release` record.
4. No commit on `release/X.Y` lacks a Cosign keyless OIDC signature from the
   `release-cherry-pick` agent identity.

Outputs: divergence report; HIGH severity findings.

## 8. `governance-deprecation-notice` (NEW · BLOCKER)

Runs on every PR with `breaking_change: true` frontmatter.

Checks:
1. ADR file exists at the path declared in `adr:` frontmatter.
2. ADR uses the breaking-change template (sections 1-7 from
   [`breaking-change-process.md`](breaking-change-process.md) §4).
3. SUNSET-LEDGER row appended with sunset_date exactly 180 days from today.
4. Both `change-class-reviewer` AND `api-stability-reviewer` have approved
   (matched by reviewer-agent identity, not just GH login).
5. Migration guide file exists at the path declared in `migration_guide:`.
6. Successor version / field / endpoint resolves in the current contract.

Outputs: BLOCKER if any check fails; PR comment with the missing items.

## 9. Lane evolution (audit → enforce)

New lanes follow the standard two-phase rollout (per progressive-delivery
`enforcement-lanes.md` §9):

1. **Phase A (audit-only).** Lane runs, emits findings, but doesn't block.
   30-day soak.
2. **Phase B (enforce).** Lane blocks per its declared severity.

Phase A → B promotion gated by per-lane PR with 30-day evidence summary.

## 10. Cross-references

- Crate versioning: [`crate-versioning-spec.md`](crate-versioning-spec.md)
- API versioning: [`api-versioning-spec.md`](api-versioning-spec.md)
- Branch cut: [`release-branch-cut-spec.md`](release-branch-cut-spec.md)
- Cherry-pick agent: [`release-cherry-pick-agent-spec.md`](release-cherry-pick-agent-spec.md)
- EOL: [`version-eol-policy.md`](version-eol-policy.md)
- Breaking change: [`breaking-change-process.md`](breaking-change-process.md)

## 11. Lift target

`oyatie/docs/standards/enforcement-lanes-release-versioning.md` on approval.
