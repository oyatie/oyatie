let meta = #{
    name: "workflow-productivity-watch",
    description: "Ensure all 4 fabric classes are productive; re-arm; fail closed on silent soft reds; heartbeat W4",
    when_to_use: "Always-on fabric W4: meta-health — never fewer than 4 active fabric classes",
    phases: [
        #{ title: "Status", detail: "fabric-status + queue stats" },
        #{ title: "Re-arm", detail: "restart missing classes via agents" },
        #{ title: "Report", detail: "productivity verdict" },
    ],
};

// W4 Productivity — INVARIANT: min 4 fabric workflows active (heartbeats).
// Soft reds on board with no claim = unproductive. Re-arm W1-W3 if stale.

phase("Status");
let st = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Fabric W4 productivity watch.\n"
    + "1) python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "2) python3 .grok/bin/mm-fabric-status --cwd /Users/jasonlee/Developer/oyatie\n"
    + "3) Read task-board: soft_red_ready, ci_red_ready, claimed lanes stuck >2h if timestamps allow\n"
    + "4) gh pr list --base dev --state open --json number,title,statusCheckRollup (summary only)\n"
    + "Return JSON: fabric_ok, missing[], soft_red_ready, ci_red_ready, open_prs, actions_needed[].",
    #{ label: "w4-status", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Re-arm");
let rearms = [];

// Always re-pulse the other three classes with short productive work if missing/stale
rearms.push(#{
    prompt: "Repo /Users/jasonlee/Developer/oyatie. W4 re-arm W1: run portfolio pulse.\n"
        + "python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
        + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
        + "--class portfolio-work-manager --note w4-rearm\n"
        + "If ready_lanes==0 and soft_red>0 something wrong — re-run ingest and report.\n"
        + "If ready_lanes==0 and no reds: add note empty-queue-healthy on board notes.\n"
        + "Return: soft_red, ready_lanes.",
    label: "w4-rearm-w1",
    capability_mode: "execute",
    agent_type: "general-purpose",
});

rearms.push(#{
    prompt: "Repo /Users/jasonlee/Developer/oyatie. W4 re-arm W2: if any ready lane with source soft_red or ci_red "
        + "and not claimed, claim the highest priority one (status=claimed, claimed_by=w4-rearm-nudge) "
        + "OR if already claimed, verify work is progressing (gh pr checks).\n"
        + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
        + "--class implement-claimed-lane --note w4-rearm\n"
        + "Do not full implement unless a soft/ci red is trivial and you can fix+push in one shot.\n"
        + "Return claimed lane id or none.",
    label: "w4-rearm-w2",
    capability_mode: "all",
    agent_type: "general-purpose",
});

rearms.push(#{
    prompt: "Repo /Users/jasonlee/Developer/oyatie. W4 re-arm W3: for open PRs 1574 and 1575 (if open), "
        + "gh pr checks summary; if soft or binding red, note must be fixed by babysit next tick.\n"
        + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
        + "--class pr-babysit-lanes --note w4-rearm\n"
        + "If merge-check would be close, say so. Return red check names.",
    label: "w4-rearm-w3",
    capability_mode: "execute",
    agent_type: "general-purpose",
});

let rearm_results = parallel(rearms);

phase("Report");
let report = agent(
    "Repo /Users/jasonlee/Developer/oyatie.\n"
    + "python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class workflow-productivity-watch --note cycle-complete\n"
    + "python3 .grok/bin/mm-fabric-status --cwd /Users/jasonlee/Developer/oyatie\n"
    + "Verdict: productive if status ok=true OR all 4 classes heartbeating after re-arm.\n"
    + "If still missing heartbeats, list exact workflow names to /workflow run.\n"
    + "Flag silent soft reds (soft_red_ready>0 with no claimed lanes) as UNPRODUCTIVE.\n"
    + "Autonomy reminder: merge on dual-critic APPROVE + CI green without human GH approve.",
    #{ label: "w4-report", capability_mode: "execute", agent_type: "general-purpose" }
);

complete(#{
    workflow: "workflow-productivity-watch",
    fabric_class: "workflow-productivity-watch",
    status_ok: st != () && st.success,
    rearm_slots: rearm_results.len(),
    report_ok: report != () && report.success,
});
