let meta = #{
    name: "pr-babysit-lanes",
    description: "Babysit open PRs + board soft_red/ci_red: fix until dual-critic APPROVE + CI green; auto-merge; heartbeat W3",
    when_to_use: "Always-on fabric W3: never leave soft reds or CI reds silent; merge when agent APPROVE",
    phases: [
        #{ title: "Ingest", detail: "queue soft reds/blocks" },
        #{ title: "Babysit", detail: "fix CI + dual-critic until APPROVE; merge when ok" },
        #{ title: "Heartbeat", detail: "fabric W3 pulse" },
    ],
};

// W3 Babysit — FULL AUTONOMY under human supervise:
// dual-critic APPROVE + oya-ci-required green => mm-drive merge (no human GH APPROVE).
// Non-APPROVE => fix until APPROVE. Soft reds MUST be fixed or tracked on board.

phase("Ingest");
let ingest = agent(
    "Repo /Users/jasonlee/Developer/oyatie.\n"
    + "python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class pr-babysit-lanes --note ingest\n"
    + "List board ready soft_red and ci_red and open PRs. Return counts.",
    #{ label: "w3-ingest", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Babysit");
let jobs = [];

// Always babysit both known fabric PRs when open + any PR with soft_red on board
jobs.push(#{
    prompt: "Repo jason931225/oyatie. Fabric W3 babysit PR #1574 (ADR disposition) if OPEN.\n"
        + "Worktree if exists: /Users/jasonlee/Developer/oyatie-adr-disposition-20260806\n"
        + "1) gh pr view 1574 + checks. mm-queue-ingest already ran — soft_red windows still counts: FIX it.\n"
        + "2) If CI red or soft red: diagnose logs, fix in PR worktree, commit, push (one push).\n"
        + "3) Dual-critic packet at docs/decisions/_disposition/evidence/pr-1574-dual-critic.json "
        + "or .grok/programs/**/evidence/pr-1574-dual-critic.json with verdict APPROVE only if earned; "
        + "else REQUEST_CHANGES and keep fixing. independence=cross_model preferred.\n"
        + "4) python3 .grok/bin/mm-drive merge-check --cwd /Users/jasonlee/Developer/oyatie --pr 1574\n"
        + "5) If merge-check ok: python3 .grok/bin/mm-drive merge --cwd /Users/jasonlee/Developer/oyatie --pr 1574\n"
        + "   (human GH APPROVE NOT required). If not ok: report reasons and fix.\n"
        + "6) Clear board lanes for this PR when checks no longer FAILURE.\n"
        + "Never hand-edit *.generated.json. No force to dev.",
    label: "w3-pr-1574",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "Repo jason931225/oyatie. Fabric W3 babysit PR #1575 (mm-kit land) if OPEN.\n"
        + "Worktree if exists: /Users/jasonlee/Developer/oyatie-kit-land-dev\n"
        + "1) gh pr checks 1575. Fix binding reds AND soft reds (windows soft compile etc).\n"
        + "2) Dual-critic evidence under .grok/programs/delivery-fabric/evidence/pr-1575-dual-critic.json "
        + "verdict APPROVE only if kit is correct; independence document honestly.\n"
        + "3) mm-drive merge-check --pr 1575; if ok, mm-drive merge --pr 1575.\n"
        + "4) Clear matching board lanes when green.\n"
        + "Never hand-edit *.generated.json outside materialize.",
    label: "w3-pr-1575",
    capability_mode: "all",
    agent_type: "general-purpose",
});

jobs.push(#{
    prompt: "Repo /Users/jasonlee/Developer/oyatie. Fabric W3 drain OTHER open PRs vs dev (not 1574/1575) "
        + "if any. For each: checks; if red fix in dedicated worktree; soft_red from board must be addressed; "
        + "merge-check + merge when dual-critic APPROVE + oya-ci-required green. "
        + "If none: say none. Cap 2 PRs this cycle.",
    label: "w3-fleet-other",
    capability_mode: "all",
    agent_type: "general-purpose",
});

let results = parallel(jobs);

phase("Heartbeat");
let hb = agent(
    "Repo /Users/jasonlee/Developer/oyatie.\n"
    + "python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class pr-babysit-lanes --note babysit-cycle\n"
    + "python3 .grok/bin/mm-fabric-status --cwd /Users/jasonlee/Developer/oyatie\n"
    + "Summarize: which PRs merged, remaining soft_red_ready, blockers.",
    #{ label: "w3-heartbeat", capability_mode: "execute", agent_type: "general-purpose" }
);

let ok_n = 0;
for r in results {
    if r != () && r.success { ok_n += 1; }
}
complete(#{
    workflow: "pr-babysit-lanes",
    fabric_class: "pr-babysit-lanes",
    babysit_slots_ok: ok_n,
    heartbeat_ok: hb != () && hb.success,
});
