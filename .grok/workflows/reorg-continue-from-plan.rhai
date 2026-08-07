let meta = #{
    name: "reorg-continue-from-plan",
    description: "Continue interrupted reorg from frozen cell plan: Own execute_now → Integrate → Package; skip full audit",
    when_to_use: "After reorg-northstar-single interrupted post plan-revise; artifacts under .grok/mm-runs/_fabric/",
    phases: [
        #{ title: "Load", detail: "re-validate frozen plan + tip; fail-closed if plan_ok false" },
        #{ title: "Prefleet", detail: "babysit path-disjoint deletes #1611 (and #1605 if reopened)" },
        #{ title: "Own", detail: "parallel owners for execute_now cells only" },
        #{ title: "Integrate", detail: "shared surfaces if needed; verify" },
        #{ title: "Adversarial", detail: "dual split-context review" },
        #{ title: "Package", detail: "one-concern PR; dual-critic; board note next residual wave" },
    ],
};

let PROCESS =
    "PROCESS: Continue from FROZEN plan only. Law files: "
    + ".grok/mm-runs/_fabric/reorg-cell-plan.latest.md, reorg-full-audit.latest.md, scout-brief.latest.md, "
    + "pr-lens-review-board.latest.md, SCOUT-CELL-REORG-DIRECTION.md, NORTH-STAR-SHAPE.md. "
    + "Do NOT re-audit archive ADRs. Do NOT open residual dual leaf PRs. "
    + "execute_now only from plan; residual_single_pr is NEXT wave not this unless plan says so. "
    + "Dir-level git mv only; no content-hash merge; no hand-edit *.generated.json. "
    + "Fix process = edit this workflow or mm-*. No git stash thrash.\n";

let REPO = "/Users/jasonlee/Developer/oyatie";
let APPLY = "/Users/jasonlee/Developer/oyatie-reorg-continue-20260807";
let PLAN = REPO + "/.grok/mm-runs/_fabric/reorg-cell-plan.latest.json";
let MAX_CELLS = 8;

phase("Load");
let load = agent(
    PROCESS
    + "LOAD frozen plan. Cwd " + REPO + ".\n"
    + "1) git fetch origin dev; print origin/dev SHA.\n"
    + "2) Read " + PLAN + " — require plan_ok true, landing_mode, execute_now list.\n"
    + "3) List execute_now cell ids, fates, paths; confirm files still exist or already disposed.\n"
    + "4) Note parked residual count and prefleet (#1611 open? #1605 merged?).\n"
    + "5) Re-harvest map: python3 .grok/bin/mm-reorg-harvest-map --json if available.\n"
    + "Return fenced JSON: origin_dev, plan_ok, execute_now_count, execute_now[], prefleet{}, load_ok.\n"
    + "load_ok=false if plan missing or plan_ok false.",
    #{ label: "load-plan", capability_mode: "execute", agent_type: "general-purpose" }
);

let load_ok = load != () && load.success;
if load != () && load.success && load.output != () && type_of(load.output) == "string" {
    if load.output.contains("\"load_ok\": false") || load.output.contains("\"load_ok\":false") {
        load_ok = false;
    }
    if load.output.contains("\"plan_ok\": false") || load.output.contains("\"plan_ok\":false") {
        load_ok = false;
    }
}
log("load_ok=" + load_ok.to_string());
if !load_ok {
    complete(#{
        workflow: "reorg-continue-from-plan",
        load_ok: false,
        next: "repair frozen plan or re-run reorg-northstar-single Audit→Plan",
    });
}

phase("Prefleet");
let prefleet = agent(
    PROCESS
    + "PREFLEET babysit path-disjoint deletes. Repo " + REPO + ".\n"
    + "1) #1605: if MERGED, note done; if open, babysit only.\n"
    + "2) #1611: gh pr checks; if REAL product red fix at PR head; if oya-ci FAILURE diagnose logs; "
    + "dual-critic re-head if needed; do NOT expand residual scope.\n"
    + "3) Do not merge without oya-ci-required SUCCESS + dual-critic APPROVE head-match + mm-drive merge-check.\n"
    + "Return: pr_1605, pr_1611, prefleet_ok, blockers[].",
    #{ label: "prefleet-deletes", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Own");
let own_jobs = [];
let i = 0;
while i < MAX_CELLS {
    let idx = i.to_string();
    own_jobs.push(#{
        prompt: PROCESS
            + "OWN execute_now cell index=" + idx + " from " + PLAN + ".\n"
            + "If fewer than " + idx + "+1 execute_now cells, skipped=true.\n"
            + "Else take that cell only. Worktree: " + APPLY + " or isolation; branch from origin/dev.\n"
            + "Apply fate: delete → git rm after consumer check; supersede → banner pointing at specs/masterplan.json "
            + "and/or delete if consumers retargeted in Integrate handoff.\n"
            + "No residual map moves in this cell. Return: skipped, cell_id, fate, paths_touched[], verify_ok, integrate_handoff[].",
        label: "own-" + idx,
        capability_mode: "all",
        agent_type: "general-purpose",
        isolation_worktree: true,
    });
    i += 1;
}
let owned = parallel(own_jobs);

phase("Integrate");
let integrate = agent(
    PROCESS
    + "INTEGRATE on " + APPLY + " from origin/dev.\n"
    + "1) Merge successful own cell outputs.\n"
    + "2) Retarget consumers: specs/masterplan.json, specs/cloud-production-quality-kits-target.json "
    + "must not point at deleted plan/* as live authority; leave historical audit TSV alone or note.\n"
    + "3) Verify execute_now verify_commands.\n"
    + "4) Do NOT apply residual REHOME-MAP in this PR.\n"
    + "Return: integrate_ok, head_sha, consumers_fixed[], measure_notes.",
    #{ label: "integrate", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Adversarial");
let adv_jobs = [];
adv_jobs.push(#{
    prompt: PROCESS
        + "Adversarial A on " + APPLY + ". Assume wrong. Hunt: deleted plan still referenced as live authority; "
        + "residual moves smuggled in; broken masterplan JSON. findings max 10.",
    label: "adv-a",
    capability_mode: "execute",
    agent_type: "general-purpose",
});
adv_jobs.push(#{
    prompt: PROCESS
        + "Adversarial B independent on " + APPLY + ". Hunt mega-PR residual mix; silent keep of invalid plans; "
        + "net_path_debt growth. findings max 10.",
    label: "adv-b",
    capability_mode: "execute",
    agent_type: "general-purpose",
});
let adv = parallel(adv_jobs);

phase("Package");
let pkg = agent(
    PROCESS
    + "PACKAGE one-concern PR from " + APPLY + ".\n"
    + "Title: docs(reorg): dispose invalid hyperscaler plan/* drafts (frozen execute_now)\n"
    + "Body: continues interrupted reorg-northstar-single-3; cites reorg-cell-plan; next residual_single_pr wave; "
    + "prefleet #1611 status.\n"
    + "SSH-signed commit; push; gh pr create/edit; dual-critic evidence.\n"
    + "Update board note: residual next.\n"
    + "Return: pr_number, package_ok, next_residual[].",
    #{ label: "package-pr", capability_mode: "all", agent_type: "general-purpose" }
);

complete(#{
    workflow: "reorg-continue-from-plan",
    load_ok: load_ok,
    prefleet_ok: prefleet != () && prefleet.success,
    own_ok: owned != (),
    integrate_ok: integrate != () && integrate.success,
    adv_ok: adv != (),
    package_ok: pkg != () && pkg.success,
    continues: "reorg-northstar-single-3",
    frozen_plan: true,
});
