let meta = #{
    name: "implement-ci-substrate",
    description: "Implement a bounded CI substrate fix on an isolated worktree: code + dual-critic + preflight + one push",
    when_to_use: "When fleet reds or admission plane need a code fix (inotify, RUSTUP, soft multi-arch), not docs-only babysit",
    phases: [
        #{ title: "Scope", detail: "confirm objective and hard stops" },
        #{ title: "Implement", detail: "single implementer agent" },
        #{ title: "Verify", detail: "dual-critic + preflight + optional push" },
    ],
};

// args.objective  — required free-text task (e.g. "raise inotify in install-buck2")
// args.worktree   — optional absolute worktree path (default: hosted-ci worktree if present)
// args.pr         — optional PR number for preflight receipt / merge-check
// args.push       — "true" to allow git push (default false; never mid-run thrash)
// args.base_branch — default origin/dev

let objective = if args == () { () } else { args.objective };
if objective == () {
    pause("verification", "Pass args.objective — the CI substrate fix to implement.");
}

let worktree = if args == () || args.worktree == () {
    "/Users/jasonlee/Developer/oyatie-hosted-cas-public-20260805"
} else {
    args.worktree
};
let pr = if args == () || args.pr == () { "0" } else { args.pr.to_string() };
let do_push = if args == () || args.push == () { "false" } else { args.push.to_string() };
let base_branch = if args == () || args.base_branch == () { "origin/dev" } else { args.base_branch };

phase("Scope");
log("implement-ci-substrate: " + objective);
log("worktree=" + worktree + " pr=" + pr + " push=" + do_push);

phase("Implement");
let impl = agent(
    "IMPLEMENT a bounded CI substrate fix for Oyatie. Do not expand into product spines or reorg.\n"
    + "Worktree: " + worktree + "\n"
    + "Base: " + base_branch + "\n"
    + "Objective:\n" + objective + "\n\n"
    + "Hard stops:\n"
    + "- No warm CAS / RE / secrets / hand-edit *.generated.json\n"
    + "- No new shell under infra/* except extending existing install-buck2.sh when needed\n"
    + "- Prefer extend infra/ci/install-buck2.sh over new scripts\n"
    + "- Soft multi-arch failures stay non-binding\n"
    + "- One concern only; do not open unrelated PRs\n\n"
    + "Procedure:\n"
    + "1) cd worktree; git status; implement the objective with the smallest patch.\n"
    + "2) Run static checks (rg/grep) that the fix is present.\n"
    + "3) If PR number is " + pr + " and != 0: run /Users/jasonlee/Developer/oyatie/.grok/bin/preflight-ci-infra --pr " + pr + " --skip-buck (or full without --skip-buck if buck available).\n"
    + "4) Dual-critic packet under .grok/programs/hyperscaler-delivery-lanes/evidence/ if code changed.\n"
    + "5) Commit if there are staged changes (SSH-signed if repo policy requires). push=" + do_push + " — only push if push==true AND no in-progress oya-ci-required for that PR (gh run list); cancel-in-progress would kill FULL.\n"
    + "6) Return: files changed, commit SHA or dirty, preflight path, push yes/no, residual risks.",
    #{ label: "implement", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Verify");
let ver = agent(
    "Adversarially verify the implementer's CI substrate fix (read-only on the worktree).\n"
    + "Worktree: " + worktree + "\n"
    + "Objective was: " + objective + "\n"
    + "Implementer report:\n" + json_encode(impl) + "\n\n"
    + "Check: (1) fix present in files, (2) no secrets, (3) no warm/RE, (4) no hand-edit generated, "
    + "(5) preflight receipt exists if PR set and CI paths touched. "
    + "Verdict APPROVE or REQUEST_CHANGES with concrete evidence paths.",
    #{ label: "verify", capability_mode: "read-only", agent_type: "explore" }
);

complete(#{
    objective: objective,
    worktree: worktree,
    pr: pr,
    push: do_push,
    implement: impl,
    verify: ver,
});
