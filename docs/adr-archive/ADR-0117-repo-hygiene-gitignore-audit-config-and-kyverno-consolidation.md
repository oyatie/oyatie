---
id: ADR-0117
status: Superseded
deciders: council-architecture, council-developer-experience
date: 2026-05-16
owner: council-developer-experience
supersedes: []
superseded_by: [ADR-700]
related: [ADR-0039, ADR-0041, ADR-0052, ADR-0115]
purpose: Document the small hygiene PR that gitignores accidentally tracked session-scoped audit artifacts (.audit/) and consolidates the single-file deploy/gitops/oya-vcs-admission policy under the established infra/kyverno/ admission-policy root. The tracked .config/nextest.toml stays because CI requires [profile.ci].
---

# ADR-0117: Repo hygiene: gitignore .audit/, consolidate kyverno admission

## Context

Two unrelated low-risk hygiene issues surfaced in a recent repo audit:

1. `.audit/agent-read.jsonl` (~52 KB) was accidentally committed. It is a transient session-scoped artifact that should never have been tracked.

   `.config/nextest.toml` was initially suspected to be local-only, but the PR test workflow sets `NEXTEST_PROFILE=ci` and requires its `[profile.ci]` section. It therefore remains tracked.

2. `deploy/gitops/oya-vcs-admission/` was a single-file root containing
   one Argo `application.json`. The repo already has a sibling
   admission-policy root at `infra/kyverno/` (which holds
   `infra/kyverno/policies/require-signed-images.yaml`). Maintaining two
   parallel admission roots — `deploy/gitops/` and `infra/kyverno/` —
   creates discoverability sprawl with no architectural justification.

## Decision

1. Add `.audit/` to `.gitignore` and untrack `.audit/agent-read.jsonl` via `git rm --cached`. Session-scoped audit logs stay local-only. Keep `.config/nextest.toml` tracked because it is CI configuration, not per-developer config.

2. `git mv deploy/gitops/oya-vcs-admission infra/kyverno/oya-vcs-admission`
   (history-preserving), removing the now-empty `deploy/gitops/` and
   `deploy/` parents. Rewrite all 4 inbound path references across 3
   files:
   - `evidence/gitops-vcs/oya-vcs-admission-cutover-2026-05-15.json`
     (2 hits — configuration-target refs, not historical migration
     records; the rewrite preserves the same truth at the new location)
   - `evidence/gitops-vcs/provider-execution-proof-2026-05-15.json`
     (1 hit, same rationale)
   - `evidence/multispectrum/oya-vcs-provider-execution-proof-1778845600.json`
     (1 hit — A3_structure finding asserting where the Argo desired
     state lives; rewritten to track the new location)
   - The moved file itself contains a self-referential `path:` pointer
     used by Argo CD to locate the manifest in the repo; this MUST be
     rewritten since Argo consumes it as a live config value.

## Naming justification

Per `feedback_naming_justification`: `infra/kyverno/` is the
established admission-policy root in this repo. Eliminating the
parallel `deploy/gitops/` root reduces sprawl without introducing a
new name. v4 BNF + 12-layer-enum conformance is unaffected (the move
is path-only, no crate or capability naming changes).

## Consequences

- Future agent-read logs stay ignored automatically — no successor-IP scrubbing required.
- The CI nextest profile remains tracked, preserving `cargo-nextest` in PR tests.
- All admission policy (Kyverno enforcement + Argo GitOps
  application) lives under one root; new admission resources have
  one obvious home.
- Two Rust crates rebuild cleanly with the rewritten constants.
  No public API change.

## Status

Accepted. Implementation lands in the same PR as the ADR.

## Sunset / Reversal

This is a terminal hygiene ADR with no sunset clause — the move and
gitignore are persistent unless explicitly reverted.

**Reversal procedure (if the kyverno consolidation proves wrong):**

1. `git revert <merge-sha-of-PR-12>` — history-preserving rename inverts cleanly; the original `deploy/gitops/oya-vcs-admission/` tree is restored with all 7 inbound refs back to their pre-merge paths in one atomic commit.
2. If the live Argo cluster has already reconciled to `infra/kyverno/oya-vcs-admission/`, `kubectl -n argocd edit application oya-vcs-admission-preview` to point `spec.source.path` back to the legacy path AND force-sync. Without this step a `git revert` alone creates a split-brain window.
3. Confirm no consumer (sibling repo CI, downstream ApplicationSet) hardcodes the new path; restore those if any.

**Data-loss class:** none. `.audit/` blob remains in git object store, recoverable via `git checkout <pre-merge-sha> -- .audit/agent-read.jsonl`.

**Related cross-checks:** ADR-0039 (kyverno admission), ADR-0041 (GitOps topology), ADR-0052 (cutover inventory; now Superseded by ADR-0118), ADR-0115 (registry consolidation precedent for flat-root sprawl removal).
