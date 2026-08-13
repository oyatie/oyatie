---
purpose: Verify that GitHub-hosted runners can reach the laptop remote-action cache without licensing warm reads
doc_status: published
---

# Runbook: GitHub-hosted laptop cache reachability proof

> **Owner:** `ops-sre-reliability`
> **Severity supported:** Sev 3
> **Last verified:** 2026-08-12 by the platform owner in a controlled lab run
> **Related:** [Remote cache runbook](buck2-nativelink-remote-cache.md), [SLO catalog](../SLO-CATALOG.md)

---

## Trigger

Open this runbook when a GitHub-hosted runner cannot complete the authenticated remote-execution
API probe through the access tunnel, or before proposing any change that licenses warm cache reads.

## Pre-checks

- [ ] The proof implementation is available from the default branch; GitHub only accepts
      `workflow_dispatch` for workflows present on that branch.
- [ ] The dispatch is bound to the protected default-branch revision before any cache credential is
      exposed.
- [ ] The server certificate authority and expected hostname are configured for both readiness and
      remote-execution API probes; insecure TLS is not an admissible proof.

If any pre-check fails, **STOP**. Repair the productized Rust proof surface through protected review;
do not run branch-controlled shell automation with cache credentials.

## Steps

1. ☐ Dispatch the default-branch cache-reachability workflow from the repository Actions UI.
   Expected: the workflow reports authenticated, hostname-verified connectivity to both cache
   endpoints.
   If differs: revoke the service token, inspect tunnel routing and certificate identity, then
   repeat from a protected revision.

2. ☐ Record the exact workflow run URL and tested commit.
   Expected: both identify the protected default-branch head and the proof artifact binds the same
   commit.
   If differs: discard the run; it is not admission evidence.

3. ☐ Run the canonical cache integrity canary from an empty build state.
   Expected: the cold and warm builds produce byte-identical output digests and the canonical
   `prelicense_probe` verdict is GREEN.
   If differs: keep warm reads unlicensed and quarantine the affected cache endpoint.

4. ☐ Bind the successful integrity-canary run in
   `specs/cache-warm-license.json.licensed_by_canary_run` through a separately reviewed protected
   change.
   Expected: the license record references the exact GREEN integrity run; reachability alone never
   authorizes warm reads.
   If differs: reject the license change.

## Rollback

Keep `specs/cache-warm-license.json` at `warm_reads_licensed: false`, revoke the cache service token,
and disable the affected tunnel route. No reachability or integrity probe requires changing the
default cache posture.

## Verification

- [ ] The authenticated remote-execution API probe verifies the configured certificate authority
      and hostname.
- [ ] The proof run and tested commit are both protected default-branch identities.
- [ ] The from-empty integrity canary reports byte-identical cold and warm outputs.
- [ ] Warm reads remain disabled unless the license record binds that exact integrity run.

## Post-incident updates

- [ ] Record routing, credential, certificate, or cache-integrity defects in the incident review.
- [ ] Update this runbook when the proof surface or rollback path changes.

## Audit-chain emission

Record the runbook invocation with runbook id `laptop-cas-gha-proof`, invoker, timestamp, tested
commit, proof run URL, integrity-canary run URL, and outcome (`verified`, `escalated`, or `failed`).

## Sources scanned

- GitHub Actions default-branch `workflow_dispatch` behavior
- Repository remote-cache license contract
- Repository cache-integrity canary contract
