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
// W4 = are fabric classes alive?  W5 = adversarial gap vs first-principles north-star.
// Does NOT babysit (W3) or implement (W2). Does NOT accept tree/board/registry multi-absorb as destiny.

phase("Audit");
let audit = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Fabric W5 ADVERSARIAL north-star audit.\n"
    + "Law: NORTH-STAR-SHAPE.md §0 first principles — do NOT rubber-stamp current layout.\n"
    + "  Registry absorbs_current_dirs = migration inventory DEBT, not multi-home design.\n"
    + "  Empty board with dual-homes = false green. W4 heartbeat ≠ north-star progress.\n"
    + "1) python3 .grok/bin/mm-dev-status --cwd /Users/jasonlee/Developer/oyatie --fetch 2>/dev/null || true\n"
    + "2) python3 .grok/bin/mm-northstar-audit --cwd /Users/jasonlee/Developer/oyatie --json\n"
    + "   Prioritize: split_brain_crates (critical), residual_scaffold_plus_live (high), process theater, singleton.\n"
    + "3) Challenge: any ready lane that spans 3+ debt zones → flag mega-lane; demand slice.\n"
    + "4) python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "Return: critical/high counts, top burn-down capabilities, reorg_ready ids, rejected assumptions[].",
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
