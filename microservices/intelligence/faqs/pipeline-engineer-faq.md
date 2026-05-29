# `foundry` µservice — Pipeline Engineer FAQ

NOTE 2026-05-21: This FAQ is HISTORICAL. The `foundry` µservice is RETIRED per ADR-0335 (Wave 15I); AI substrate absorbed into `microservices/intelligence/`. The agentic-pipeline doctrine referenced below lives in ADRs 0110, 0111, 0112, 0113, 0116, 0247, 0255. The "Hermes" name is RETIRED corpus-wide per ADR-0247 D-10 + ADR-0328 D-9.22 + ADR-0335 D-26..D-36. Q1 in particular is now superseded — see ADR-0335 for the retirement decision.

22 real questions raised against the internal agentic-development pipeline that lands every change at Oyatie.

---

**Q1. Is Foundry a customer-facing product?**

No. ADR-0136-amendment is binding: Foundry is internal-only. There is no consumer sign-up, no SaaS surface, no public docs site.
Customers see Foundry's effects (audit-chain entries, signed releases) but never the pipeline UI itself.

---

**Q2. Why a custom merge queue instead of GitHub Merge Queue or Bors?**

GitHub Merge Queue + Bors don't compute projected merge state across multiple in-flight PRs touching shared crates without O(N²)
rebase cascades. ADR-0111 codifies the fix: project state once + recompute on conflict. Bors gets close but doesn't integrate with
agent reviewer principals + multispectrum verdicts.

---

**Q3. What's the difference between `claim` and `done`?**

`claim` locks a scope (file paths) so other agents don't conflict. `done` marks the scope's work complete and writes evidence to the
audit chain. Between them: `work` (no protocol verb; just edits) and `verify` (run validators locally). The lifecycle is enforced
by `lean-a11-claim-discipline`.

---

**Q4. What is "projected merge state"?**

Per ADR-0111: instead of running validators on the PR's branch tip, run them on a clean in-memory rebase of the PR onto `target`'s
current tip. This catches conflicts that would surface only after merge (e.g. semantic conflicts in shared crates). The projection
is recomputed whenever `target` advances.

---

**Q5. Why webhooks instead of polling?**

ADR-0112: polling at 200 active PRs × 5 events / PR / hour = 1000 polls / hour wasted. Webhooks scale linearly with event volume,
not PR count. Foundry has a dedicated webhook receiver per source forge (GitHub, GitLab, internal `oya vcs`).

---

**Q6. How does the coordinator prevent shared-crate cascades?**

Per `feedback_pipeline_clog_gotchas_2026_05_17`: when ≥ 2 in-flight PRs touch the same shared crate, the coordinator acquires a
crate-level lock and serializes their admission. PRs touching disjoint crates run in parallel. This reduces O(N²) rebase work to
O(N) for the serialized group + O(1) for parallel groups.

---

**Q7. What do `lean-a*` lanes enforce?**

The lean-architecture CI lanes enforce structural invariants:
- `lean-a3-tenant-trace` — every code path carries `tenant_id` in logs + traces.
- `lean-a4-secret-cleartext` — no cleartext secret material in logs.
- `lean-a5-doc-coverage` — every µservice ships full doc set.
- `lean-a7-rotator-substance` — secret rotators mutate backend, not just metadata.
- `lean-a8-module-attestation` — `cloud-iac` modules are cosign-signed.
- `lean-a9-template-substance` — `developer-sdk` templates produce substantive output.
- `lean-a10-no-silent-regression` — breaking changes require ADR + version bump + sunset window.
- `lean-a11-claim-discipline` — edits without active claim are blocked.

The full set lives at `tools/lean/`.

---

**Q8. What's the multispectrum verdict?**

Multispectrum v2.4.0 (per `feedback_multispectrum_review_v22.md` + adherence facets): 11 facets per PR, evaluated by separate
subagent sessions, each with one facet as their lens. The verdict aggregates per-facet outcomes; APPROVE requires ≥ N facets PASS
(N per tier). The full doctrine: F1-F9 critique facets + M1+M2 meta + A1-A7 adherence.

---

**Q9. What does "consensus on high-risk" mean?**

For high-risk PRs (touching `governance`, `cloud-secrets`, `kms`, `audit-chain`, or any ADR-class change), Foundry requires
consensus from ≥ 2 reviewer-agent models (e.g. Claude Sonnet + Claude Opus + Codex). Disagreement escalates to human review.

---

**Q10. How does the audit chain work inside Foundry?**

Every primitive (claim, work, done, verify, promote, admit, merge) writes an event to `audit-chain` µservice. Events chain via
BLAKE3-256 `(prev_hash, payload_hash) → curr_hash`. Foundry verifies the chain head matches before admitting any new PR — this
catches tampering attempts.

---

**Q11. What's a "fence token"?**

A monotonically-increasing per-base-branch token assigned at admit time. When Foundry merges PRs, it merges in fence-token order;
out-of-fence merges are illegal and the queue refuses them. This is the substrate for ADR-0111's projected state.

---

**Q12. Can humans bypass the queue?**

Only via `oyatie.governance.break-glass-operator.*` principal, which fires an immediate audit-chain alert + a 24 h post-hoc
reviewer-agent verdict. Bypasses without break-glass are blocked by GitHub branch protection + the receive-pack hook.

---

**Q13. What happens when reviewer-agent disagrees with itself across runs?**

Flaky verdicts trigger `oyatie.foundry.reviewer-stability` audit events. Three consecutive flaky verdicts on the same PR escalate
to a "stability gate" — the reviewer-agent is run with a higher-quality model (Claude Opus instead of Sonnet) and the verdict
is recorded as authoritative. If still unstable, human escalation.

---

**Q14. How does Foundry interact with `cloud-iac`?**

Foundry uses `cloud-iac` to provision new cells, e.g. for adding a sovereign-region pipeline. Per ADR-0247, Foundry runs as
`oyatie.foundry.*` principals under Cedar; the cloud-iac permits for these principals are deliberately narrow (no destructive
actions outside a tightly-scoped namespace).

---

**Q15. What's the worst-case clog scenario?**

Per `feedback_pipeline_clog_gotchas_2026_05_17` gotcha #17: a malformed PR that consumes a validator slot without making progress.
Mitigations: validator timeouts (30 min), abandonment detection (PR with no comments / commits for 4 h auto-abandoned to a parked
queue), in-flight cap (12 / agent, 200 cluster-wide).

---

**Q16. Can Foundry deploy itself?**

Yes, via the self-modification path (ADR-0247). The `oyatie.foundry.self-modify.*` principal has narrower permits than ordinary
contributors; it cannot bypass the queue, cannot modify governance configuration, and any self-modification PR requires consensus
from ≥ 3 reviewer-agents + a human approver.

---

**Q17. How is pipeline observability done?**

`foundry` emits OTLP traces + metrics + logs to the `observability` µservice. Key metrics:
- `foundry_pr_admit_to_merge_p95_seconds{base_branch}` — the SLO clock.
- `foundry_queue_depth{base_branch}` — clog indicator.
- `foundry_reviewer_agent_verdict_duration_seconds{model, facet}` — perf of reviewers.
- `foundry_shared_crate_locks_held{crate}` — coordinator state.

Dashboards live at `microservices/intelligence/dashboards/`.

---

**Q18. What's a "promote" actually doing?**

For dev: marking the PR's effect promoted to dev branch tip + writing the promotion event. For staging/prod: triggering the
`workflow-engine` deploy workflow with the bundle ID as input. The promote primitive is Cedar-gated separately per environment.

---

**Q19. How does Foundry handle merge conflicts?**

Auto-rebase if the conflict is mechanical (textual). For semantic conflicts (validator failures on rebase), Foundry returns the PR
to the author with a structured conflict report. Agents respond by re-running the work cycle.

---

**Q20. What's the cap on PR size?**

Soft cap: 1,500 lines diff. Hard cap: 5,000 lines. Above hard cap, the PR requires governance pre-approval + a chunking plan.
Bulk-renames are exempt via a `--bulk-rename` flag on `oya vcs claim`.

---

**Q21. What's the relation to `governance`?**

`governance` owns the human approval workflows + Paxos-leased locks. `foundry` calls into `governance` for break-glass, sovereign
pipeline issuance, and ADR-class approvals. Routine PRs don't touch `governance`.

---

**Q22. How do we onboard a new µservice into Foundry?**

Per `microservices/intelligence/runbooks/onboard-new-microservice.md`:
1. Create the `microservices/<name>/` tree with the 7 surface dirs.
2. Add the `oya-<name>-*` crate lane to `Cargo.toml` workspace.
3. Wire the µservice into the lean-a* validators + multispectrum facet selectors.
4. Add the µservice to `tenancy`'s tier matrix.
5. File the µservice ADR.
6. Promote through Foundry like any other PR.
