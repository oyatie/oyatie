let meta = #{
    name: "northstar-portfolio-audit",
    description: "Audit repo+board+backlog against NORTH-STAR-SHAPE; surface work gaps; heartbeat W5",
    when_to_use: "Always-on fabric W5 (separate from W4 productivity): find work that needs doing",
    phases: [
        #{ title: "Audit", detail: "mm-northstar-audit + origin/dev re-query" },
        #{ title: "Board gaps", detail: "upsert missing dual-home/reorg leaves when justified" },
        #{ title: "Report", detail: "findings + next path-disjoint claims for W2" },
    ],
};

// W5 North-star portfolio audit — SEPARATE from W4 productivity-watch.
// W4 = are fabric classes alive?  W5 = what work exists vs north-star + backlog?
// Does NOT babysit PRs (W3) and does NOT implement (W2) — only discovers/enqueues.

phase("Audit");
let audit = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Fabric W5 NORTH-STAR portfolio audit.\n"
    + "Law: .grok/programs/delivery-fabric/NORTH-STAR-SHAPE.md + REORG-LANE-TAXONOMY.md + BABYSIT-SINGLE-FLIGHT.md\n"
    + "1) python3 .grok/bin/mm-dev-status --cwd /Users/jasonlee/Developer/oyatie --fetch 2>/dev/null || true\n"
    + "2) python3 .grok/bin/mm-northstar-audit --cwd /Users/jasonlee/Developer/oyatie --json\n"
    + "   Read findings: dual_home, move_plan_singleton, generated_faces, backlog_not_on_board, work_exists_but_board_empty\n"
    + "3) python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "4) Optionally: python3 .grok/bin/mm-reorg-enqueue --cwd /Users/jasonlee/Developer/oyatie\n"
    + "Return JSON summary of critical/high findings and reorg_ready lane ids.",
    #{ label: "w5-audit", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Board gaps");
let gaps = agent(
    "Repo /Users/jasonlee/Developer/oyatie. W5 apply board gaps under north-star reduce-only.\n"
    + "1) python3 .grok/bin/mm-northstar-audit --cwd /Users/jasonlee/Developer/oyatie --write-board-gaps\n"
    + "2) Board hygiene: mark soft_red/ci_red ready lanes for MERGED PRs as done (gh pr view N --json state)\n"
    + "3) Do NOT implement product code. Do NOT multi-poll CI. Do NOT open mega-PRs.\n"
    + "4) Ensure dual-home high findings map to ready REORG-* lanes path-disjoint where possible.\n"
    + "Return: upserted lane ids, cleared stale queue lanes, still_blocked (CAS/G002).",
    #{ label: "w5-board-gaps", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Report");
let report = agent(
    "Repo /Users/jasonlee/Developer/oyatie. W5 report + heartbeat.\n"
    + "1) cat .grok/mm-runs/_fabric/northstar-audit.latest.json | head via python pretty top findings\n"
    + "2) python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class northstar-portfolio-audit --note cycle-complete\n"
    + "3) List top 5 path-disjoint ready lanes W2 should claim next (prefer reorg delete/refactor over blocked move).\n"
    + "4) Remind: net_path_debt<=0 in reorg targets; process only under .grok/; W3 sole babysit.\n"
    + "Return: next_claims[], critical_count, high_count.",
    #{ label: "w5-report", capability_mode: "execute", agent_type: "general-purpose" }
);

complete(#{
    workflow: "northstar-portfolio-audit",
    fabric_class: "northstar-portfolio-audit",
    audit_ok: audit != () && audit.success,
    gaps_ok: gaps != () && gaps.success,
    report_ok: report != () && report.success,
    separate_from_w4: true,
    northstar: true,
});
