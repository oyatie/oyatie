let meta = #{
    name: "pr-babysit-lanes",
    description: "Single-flight babysit of open PRs: fix CI + dual-critic; merge-on-green; never multi-arm one PR",
    when_to_use: "Always-on fabric W3: sole babysit owner; soft/binding reds must not go silent",
    phases: [
        #{ title: "Ingest", detail: "queue soft reds/blocks + single-flight claims" },
        #{ title: "Babysit", detail: "one fleet agent; claim (pr,head); fix/merge; no parallel waiters per PR" },
        #{ title: "Heartbeat", detail: "fabric W3 pulse" },
    ],
};

// W3 sole babysit owner (see BABYSIT-SINGLE-FLIGHT.md).
// dual-critic APPROVE + oya-ci-required => mm-drive merge under human supervise.
// FORBIDDEN: spawn multiple poll/merge loops for the same (pr, head_sha).

phase("Ingest");
let ingest = agent(
    "Repo /Users/jasonlee/Developer/oyatie.\n"
    + "Read .grok/programs/delivery-fabric/BABYSIT-SINGLE-FLIGHT.md (law).\n"
    + "python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "Ensure dir .grok/mm-runs/_fabric exists. Load or create babysit-claims.json map pr->claim.\n"
    + "List open PRs vs dev + board soft_red/ci_red. Return JSON: open_prs, soft_red_ready, ci_red_ready, claims.",
    #{ label: "w3-ingest", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Babysit");
// ONE agent owns the whole tick fleet — not N parallel PR agents that all poll.
let fleet = agent(
    "Repo jason931225/oyatie. Fabric W3 SOLE babysit owner (BABYSIT-SINGLE-FLIGHT.md).\n"
    + "Claims file: .grok/mm-runs/_fabric/babysit-claims.json\n"
    + "ALGORITHM:\n"
    + "1) gh pr list --state open --base dev --limit 20\n"
    + "2) Rank: binding FAILURE first, then soft_red board lanes, then BLOCKED merge without green oya-ci-required.\n"
    + "3) Cap 2 PRs this tick. For each candidate:\n"
    + "   a) head=$(gh pr view N --json headRefOid -q .headRefOid)\n"
    + "   a0) python3 .grok/bin/mm-ci-triage --pr N  (CI-RED-PROCESS.md)\n"
    + "      If ALL fails GHA_INFRA_FLAKE or oya-ci queued/missing: re-run jobs / wait ARC — "
    + "DO NOT product thrash or dual-critic tip pin for flakes.\n"
    + "      If REAL_PRODUCT: fix in worktree after mm-preflight-pr green; one signed push.\n"
    + "   b) If claims[N].head==head and claims[N].owner==pr-babysit-lanes and armed_at < 45m ago:\n"
    + "      SKIP arming another waiter; one-shot gh pr checks only; if merge-check ok then merge once.\n"
    + "   c) Else write claim {head, owner:pr-babysit-lanes, armed_at:UTC, mode:merge_on_green}\n"
    + "   d) Only for REAL_PRODUCT: diagnose; fix in that PR worktree; ONE signed push if code fix.\n"
    + "   e) Dual-critic evidence for tip (local pin OK; no thrash push unless needed for process).\n"
    + "   f) mm-drive merge-check --pr N; if ok: mm-drive merge --pr N; clear claim + board lanes.\n"
    + "4) FORBIDDEN: sleep-poll loops in implement worktrees; FORBIDDEN: second background poller for same (N,head);\n"
    + "   FORBIDDEN: dual-critic tip thrash (>1 evidence pin push per head without CI green).\n"
    + "5) If no open PRs needing work: say idle-healthy.\n"
    + "Return: acted_on[], skipped_single_flight[], merged[], remaining_blockers[].",
    #{ label: "w3-fleet-single-flight", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Heartbeat");
let hb = agent(
    "Repo /Users/jasonlee/Developer/oyatie.\n"
    + "python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class pr-babysit-lanes --note single-flight-cycle\n"
    + "python3 .grok/bin/mm-fabric-status --cwd /Users/jasonlee/Developer/oyatie\n"
    + "Summarize: merged, single-flight skips, soft_red_ready, open PRs still needing W3.",
    #{ label: "w3-heartbeat", capability_mode: "execute", agent_type: "general-purpose" }
);

complete(#{
    workflow: "pr-babysit-lanes",
    fabric_class: "pr-babysit-lanes",
    babysit_slots_ok: fleet != () && fleet.success,
    heartbeat_ok: hb != () && hb.success,
    single_flight: true,
});
