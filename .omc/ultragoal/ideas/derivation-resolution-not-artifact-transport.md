# Derivation resolution, not artifact transport

## Problem Statement

**How might we make CI-derived artifacts available to every consumer without either committing
them (merge conflicts) or shipping them through rented artifact storage (the coupling that
blanked all 44 gates)?**

Both current options are *transports* for the same thing — a derivation. Commit it and you get
conflicts, because a derivation has no authored truth to merge. Ship it and a storage backend
sits in the merge path.

## Recommended Direction

**Stop transporting derivations. Resolve them by content hash.**

A derivation is addressed by the hash of its declared inputs and looked up in a shared
content-addressed store. It is never committed and never attached to a CI run — it is
*resolved*. This is what Bazel/Buck2 remote cache and the Nix store are, and it is exactly what
the NativeLink CAS already running on the cluster exists to be.

Applies to all 7 de-committed derivations (faces, gate baselines, projection surfaces), not just
`generated-faces` — the de-commit already gave them a common shape; this gives them a common
resolver.

**This needs remote CACHE only — not remote execution.** Verified against buck2 source at the
byte-exact commit of our installed binary (1560aca2): scheduler-less REAPI cache-only is an
explicit match arm in the executor builder. NativeLink exposes `capabilities` on both listeners,
and buck2's OSS client reads only `cache_capabilities`, never `execution_capabilities`. So the
scheduler and worker tiers of ADR-0612 stay deferred. That makes this dramatically smaller than
"make RE work" — which is how it was previously scoped, and why it never started.

## Why it has not worked so far (root-caused, not guessed)

Three independent defects, any ONE of which alone yields the measured zero-upload symptom:

1. **Two disjoint config planes.** `remote_enabled` / `remote_cache_enabled` /
   `allow_cache_uploads` are NOT `.buckconfig` keys — they are `CommandExecutorConfig()` Starlark
   params on the execution platform. `[buck2_re_client]` configures only the client CONNECTION.
   Setting either alone is inert. This one misunderstanding explains every previous dead end.
2. **The default platform forecloses it.** `prelude//platforms:default` hardcodes
   `remote_enabled = False`; in OSS builds `remote_cache_enabled` DEFAULTS to `remote_enabled`,
   so it resolves False, the executor collapses to `Executor::Local`, and `Executor::Local` is
   hardwired to `NoOpCacheUploader` with behavior DISCARDED. Zero uploads, no error, no warning.
   Fix: our own execution platform with `remote_cache_enabled = True` explicit.
3. **Daemon staleness.** `[buck2_re_client]` is absent from `DaemonStartupConfig`, the only thing
   the daemon-reuse check compares. Editing `.buckconfig.local` with a live daemon does NOT
   restart it — it serves the stale empty RE config forever. `buck2 killall` (or a fresh
   `--isolation-dir`) is MANDATORY after any RE config change, and any measurement taken without
   it is invalid.

## Key Assumptions to Validate

- [ ] **The materializer is hermetic-izable.** THE decisive assumption. A CAS key hashes DECLARED
      inputs. Face derivation reads git history (ADR-0552 split `scm-facts` precisely because of
      this). If history leaks into a derivation whose key does not include it, the CAS will
      confidently return a STALE face — a false green one layer BELOW every gate, strictly worse
      than today. Test: derive twice from identical trees at different git states; bytes must
      match, or the input closure is wrong.
- [ ] **A CAS fault is detectable.** Four separate false-green classes already found in this
      subsystem; every success signal is client-side. Making gates depend on the CAS converts a
      cache fault into a gate fault in a system with a demonstrated inability to notice. Test:
      paired populate -> fresh-read with BYTES measured server-side (NativeLink ops :50061,
      no TLS, health+admin), never the buck2 client's own report.
- [ ] **The 7 derivations share an input-closure shape.** They were de-committed together, which
      is not evidence they resolve the same way. Test: enumerate each one's actual input set
      before assuming one resolver fits all.

## MVP Scope

Smallest slice that tests assumption 1, which is the one that can kill the idea:

IN — repair the cold integrity canary FIRST (see blockers); one owned execution platform target
with `remote_cache_enabled = True`; `[buck2_re_client]` with `engine_address` pointed at the
writer listener; ONE derivation (`generated-faces`) resolved from CAS; server-side byte proof.

OUT — the other 6 derivations until the first proves out; any warm-license flip; the scheduler
and worker tiers; remote execution.

## Not Doing (and Why)

- **Owned artifact store as a drop-in for GitHub artifacts (SeaweedFS swap)** — feels like
  ownership but keeps artifact-as-transport: still a producer job, a consumer job, and an edge
  between them that can fail. Relocates the dependency instead of removing it.
- **Committing the derivations again** — that is what ADR-0613 de-committed, and it reintroduces
  both the merge-conflict class and the silent-corruption surface.
- **Remote execution** — not needed for this. Cache-only is scheduler-less and supported. RE is a
  separate, larger project governed by ADR-0612; conflating them is why this never started.
- **Flipping `/specs/cache-warm-license.json`** — it ships `warm_reads_licensed: false` and stays
  false until a GREEN canary. Flipping it early converts an unproven cache into a trusted one.
- **cargo-in-CI** — founder closed this: buck2 stays. The measurement would have described a
  broken configuration, not a verdict on buck2.

## Blockers (real, each independently fatal)

1. `root_buckconfig_stays_dark` — a LIVE leg of the 45-leg required matrix. Adding
   `[buck2_re_client]` to root `.buckconfig`, or moving `build.execution_platforms` off
   `prelude//platforms:default`, turns `oya-ci-required` RED. Merge authority, not advisory.
2. `buckconfig_local_is_ignored_and_untracked` — bans the ONLY file that can wire RE.
3. `remote_enabled = False` is a hardcoded literal in the rule impl of `toolchains/cache/defs.bzl`
   — NOT an attr, unlike its two siblings. No config or flag can flip it; it is a Starlark edit.
4. **The cold integrity canary has NEVER been green.** All 5 recent runs failed at "Cold
   from-empty build of the pinned target set" — the cold build itself, before any comparison.
   ADR-0612 D5: "no RE-covering canary, no RE". This is the true critical path.

RESOLVED this week, unnoticed: ADR-0612 OQ1 ratified "there is no RE phase until the runner can
route to the cluster." The `oya-arm64` Talos fleet (#1450) resolves `*.svc.cluster.local`
natively. That gate is now satisfied.

## Open Questions

- Does face derivation actually depend on git history, or only on `scm-facts` (already split out
  as a separate face)? This decides assumption 1 and therefore the whole idea.
- Why does the cold canary fail at the cold build itself? It is a bug, not a verdict — nothing
  can be licensed until it is understood.
- Blockers 1 and 2 are gates WE authored to keep the config dark. Do they get amended (a reviewed
  exception for the RE slice) or is the dark-by-default posture itself the thing to revisit?
