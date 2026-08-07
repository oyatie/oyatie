let meta = #{
    name: "northstar-portfolio-audit",
    description: "Audit each dual-home rehome vs NORTH-STAR-SHAPE; board gaps; parallel W2 claims",
    when_to_use: "Always-on fabric W5 (separate from W4 productivity): find work that needs doing",
    phases: [
        #{ title: "Audit", detail: "mm-northstar-audit + per-rehome disposition" },
        #{ title: "Board gaps", detail: "upsert dual-home rehome leaves (delete/reshape/mixed)" },
        #{ title: "Parallel claims", detail: "path-disjoint residual rehomes for W2 fan-out" },
        #{ title: "Report", detail: "findings + heartbeat W5" },
    ],
};

// W5 North-star portfolio audit — SEPARATE from W4 productivity-watch.
// W4 = are fabric classes alive?  W5 = adversarial gap vs first-principles north-star.
// Does NOT babysit (W3) or implement (W2). Does NOT accept tree/board/registry multi-absorb as destiny.
// Each rehome is scored: shape_clean false until one durable home; class = delete|reshape|mixed|move.

phase("Audit");
let audit = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Fabric W5 ADVERSARIAL north-star + REHOME audit.\n"
    + "Law: NORTH-STAR-SHAPE.md §0 first principles — do NOT rubber-stamp current layout.\n"
    + "  Registry absorbs_current_dirs = migration inventory DEBT, not multi-home design.\n"
    + "  Empty board with dual-homes = false green. W4 heartbeat ≠ north-star progress.\n"
    + "1) python3 .grok/bin/mm-dev-status --cwd /Users/jasonlee/Developer/oyatie --fetch 2>/dev/null || true\n"
    + "2) python3 .grok/bin/mm-northstar-audit --cwd /Users/jasonlee/Developer/oyatie --json\n"
    + "   Read rehomes[]: every dual-active capability is NOT shape_clean.\n"
    + "   For each: class (delete|reshape|mixed|move), durable_home, debt_paths, parallel_safe.\n"
    + "   Prioritize: split_brain_crates (critical serial), residual_scaffold_plus_live (high parallel).\n"
    + "3) Challenge: any ready lane that spans 3+ debt zones → flag mega-lane; demand slice.\n"
    + "4) python3 .grok/bin/mm-queue-ingest --cwd /Users/jasonlee/Developer/oyatie --base dev\n"
    + "5) python3 .grok/bin/mm-reorg-enqueue --cwd /Users/jasonlee/Developer/oyatie 2>/dev/null || true\n"
    + "Return: critical/high counts, rehomes_not_clean[], parallel_rehome_claims[], serial_split_brain_claims[],\n"
    + "  rejected assumptions[].",
    #{ label: "w5-audit", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Board gaps");
let gaps = agent(
    "Repo /Users/jasonlee/Developer/oyatie. W5 apply board gaps under north-star reduce-only.\n"
    + "1) python3 .grok/bin/mm-northstar-audit --cwd /Users/jasonlee/Developer/oyatie --write-board-gaps\n"
    + "   Each dual-home becomes REORG-DUAL-<CAP>-BURNDOWN with class/paths/debt_paths/durable_home.\n"
    + "2) Board hygiene: mark soft_red/ci_red ready lanes for MERGED PRs as done (gh pr view N --json state)\n"
    + "3) Do NOT implement product code. Do NOT multi-poll CI. Do NOT open mega-PRs.\n"
    + "4) Ensure dual-home high findings map to ready REORG-* lanes path-disjoint where possible.\n"
    + "Return: upserted lane ids, cleared stale queue lanes, still_blocked (CAS/G002).",
    #{ label: "w5-board-gaps", capability_mode: "all", agent_type: "general-purpose" }
);

phase("Parallel claims");
let claims = agent(
    "Repo /Users/jasonlee/Developer/oyatie. W5 emit parallel rehome claim list for W2 (do NOT implement here).\n"
    + "1) Load .grok/mm-runs/_fabric/northstar-audit.latest.json → parallel_rehome_claims + serial_split_brain_claims.\n"
    + "2) Subtract paths already owned by OPEN PRs (gh pr list --state open --json files,number,title).\n"
    + "   Currently open reorg often owns marketplace/observability/tools/intel — skip those debt_paths.\n"
    + "3) Build up to 8 path-disjoint residual claims (parallel_safe=true):\n"
    + "   prefer audit, flags, compliance, tasks, cell, network, console, billing-scaffold leaves.\n"
    + "   Each claim: {lane_id, class, durable_home, debt_paths, primary_action, open_pr_conflict:bool}.\n"
    + "4) Serial queue (do NOT fan-out same move-plan): ci split_brain, intelligence split_brain.\n"
    + "5) Write .grok/mm-runs/_fabric/parallel-rehome-claims.latest.json\n"
    + "6) Remind W2: mm-preflight-pr before every PR; mm-ci-triage on red (CI-RED-PROCESS.md).\n"
    + "Return: parallel_claims[], serial_claims[], skipped_owned_by_open_pr[].",
    #{ label: "w5-parallel-claims", capability_mode: "execute", agent_type: "general-purpose" }
);

phase("Report");
let report = agent(
    "Repo /Users/jasonlee/Developer/oyatie. W5 report + heartbeat.\n"
    + "1) Pretty-print top rehomes from northstar-audit.latest.json + parallel-rehome-claims.latest.json\n"
    + "2) python3 .grok/bin/mm-fabric-heartbeat --cwd /Users/jasonlee/Developer/oyatie "
    + "--class northstar-portfolio-audit --note cycle-complete-rehomes\n"
    + "3) List top path-disjoint ready lanes W2 should claim next "
    + "(prefer reshape/delete residual over blocked move; never mega cloud/*).\n"
    + "4) Remind: net_path_debt<=0; process only under .grok/; W3 sole babysit; "
    + "GHA flake ≠ product (mm-ci-triage first).\n"
    + "Return: next_claims[], critical_count, high_count, rehomes_not_clean_count.",
    #{ label: "w5-report", capability_mode: "execute", agent_type: "general-purpose" }
);

complete(#{
    workflow: "northstar-portfolio-audit",
    fabric_class: "northstar-portfolio-audit",
    audit_ok: audit != () && audit.success,
    gaps_ok: gaps != () && gaps.success,
    claims_ok: claims != () && claims.success,
    report_ok: report != () && report.success,
    separate_from_w4: true,
    northstar: true,
    rehome_audit: true,
});
