let meta = #{
    name: "portfolio-work-manager",
    description: "Discover parallel lanes from beads/roadmap/console/open PRs; ingest soft reds; write task-board; heartbeat W1",
    when_to_use: "Always-on fabric W1: keep board stocked with path-disjoint ready work",
    phases: [
        #{ title: "Ingest", detail: "soft reds + blocks onto task board" },
        #{ title: "Discover", detail: "beads, roadmap, console, open PRs" },
        #{ title: "Board", detail: "upsert ready lanes; heartbeat" },
    ],
};

// W1 Portfolio — never idle the board. Soft reds MUST be queued (mm-queue-ingest).
// Not merge authority. Human supervises only.

phase("Ingest");
let ingest = agent(
    "Repo /Users/jasonlee/Developer/oyatie. You are fabric W1 portfolio manager.\n"
    + "MANDATORY commands IN ORDER:\n"
    + "  1) python3 .grok/bin/mm-dev-status --cwd /Users/jasonlee/Developer/oyatie --fetch\n"
    + "     Record origin/dev sha12, subject, oya_ci on tip, prs_behind_dev. ALWAYS be aware of remote dev.\n"
    + "  2) python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "  3) python3 .grok/bin/mm-reorg-enqueue --cwd /Users/jasonlee/Developer/oyatie --waves W0,W1,W2\n"
    + "     Overdue reorg/rewrite/debrand/refactor/remove MUST stay on board (REORG-REBRAND-BACKLOG).\n"
    + "  4) python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class portfolio-work-manager --note ingest-dev-reorg\n"
    + "Report JSON: origin_dev_sha12, soft_red, ci_red, reorg_ready, prs_behind_dev_count.\n"
    + "Use run_terminal_command. Fail-closed if dev-status or queue-ingest hard-fails.",
    #{ label: "w1-ingest", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Discover");
let discover = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Discover path-disjoint READY work AFTER origin/dev awareness.\n"
    + "MUST re-read mm-dev-status / board.origin_dev before dispatching any reorg claim.\n"
    + "Sources (use tools, do not invent):\n"
    + "1) Board lanes already present: soft_red, ci_red, block, reorg (from mm-reorg-enqueue)\n"
    + "2) bd ready (beads)\n"
    + "3) .grok/programs/REORG-DOCTRINE.md + REORG-REBRAND-BACKLOG.md — long-overdue reorg debt is FIRST-CLASS\n"
    + "   Classes: move|refactor|rewrite|delete|rebrand|mixed — not only moves\n"
    + "4) ROADMAP-DURABLE-GOAL / PARALLEL-BOARD / AUTHORITY-AND-MINED-BACKLOG\n"
    + "5) /Users/jasonlee/Developer/console HANDOFF.md DESIGN.md if present\n"
    + "6) Open PRs behind origin/dev — enqueue tip-sync/rebase work as source=block if needed\n"
    + "Prefer order: ci_red > soft_red > reorg W0/W1 ready > beads > console > roadmap W2+ blocked prereqs.\n"
    + "Every reorg lane must include dod: re-query live ADRs on origin/dev; worktree from tip; one concern PR.\n"
    + "Write board: keep claimed/in_progress; upsert ready; set origin_dev from mm-dev-status; updated_by portfolio-work-manager.\n"
    + "Return markdown table of ready lanes including ALL reorg-* ids.",
    #{ label: "w1-discover", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Board");
let hb = agent(
    "Repo /Users/jasonlee/Developer/oyatie.\n"
    + "1) python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class portfolio-work-manager --note discover-complete\n"
    + "2) python3 .grok/bin/mm-fabric-status --cwd /Users/jasonlee/Developer/oyatie\n"
    + "3) Summarize: ready count, soft_red_ready, ci_red_ready, fabric ok?, next implement should claim which lane id.\n"
    + "Do not implement product code. Do not merge.",
    #{ label: "w1-heartbeat", capability_mode: "execute", agent_type: "general-purpose" }
);

let summary = "W1 portfolio-work-manager complete";
if discover != () && discover.success {
    summary = "W1 board updated; see discover agent output";
}
complete(#{
    workflow: "portfolio-work-manager",
    fabric_class: "portfolio-work-manager",
    summary: summary,
    ingest_ok: ingest != () && ingest.success,
    discover_ok: discover != () && discover.success,
    heartbeat_ok: hb != () && hb.success,
});
