let meta = #{
    name: "post-3a-implementation-fanout",
    description: "Post-3A unblocked implementation: CAS 3B, CI path-opt, red-PR fixes, post-merge packets",
    when_to_use: "After NativeLink 3A + path-filter merged on origin/dev; dual runners live",
    phases: [
        #{ title: "Inventory", detail: "Confirm unblocked lanes and hard stops" },
        #{ title: "Fan-out", detail: "Parallel implement CAS 3B + CI + packets + red PRs" },
        #{ title: "Synthesize", detail: "Merge readiness and next gates" },
    ],
};

// Sequencing proof (do not violate):
// - Lane 3B starts from promoted 3A on origin/dev (010c132ec or later)
// - W0-B product engine still blocked until G001 #1561 packet
// - No warm CAS, no RE remote_enabled, no secrets, no hand-edit *.generated.json

phase("Inventory");
let inv = agent(
    "Repo /Users/jasonlee/Developer/oyatie. Re-query only (read-only). Report JSON-ish bullets: "
    + "(1) origin/dev tip contains #1563 3A nativelink rehome? (2) #1562 path-filter on trunk? "
    + "(3) live ARC maxRunners for oya-arm64 (kubectl if available). (4) open PRs 1561,1564,1566-1571 "
    + "state. (5) confirm toolchains/cache and infra/ci/buckconfig/warm-cache-*.buckconfig still exist "
    + "for 3B. (6) list unblocked NOW: 3B yes/no, W0-B yes/no, warm CAS yes/no. Fail-closed if git fetch fails.",
    #{ label: "inventory", capability_mode: "execute", agent_type: "explore" }
);

phase("Fan-out");
let jobs = [];

// --- CAS Lane 3B: Buck2 cache package move (PRIMARY unblocked implement) ---
jobs.push(#{
    prompt: "IMPLEMENT CAS Lane 3B (Buck2 cache package move) for Oyatie.\n"
        + "Context: Lane 3A NativeLink rehome MERGED as #1563 on origin/dev. Plan: "
        + ".grok/programs/cas-fabric/evidence/approved-plan-cas-re-20260805.md Lane 3B + "
        + "RE-SANDBOX-READINESS P3.\n"
        + "Hard stops: no warm_reads; no RE remote_enabled; no secrets; no alias/dual-home; "
        + "root .buckconfig stays DARK (no [buck2_re_client]/[oya_cache]); no hand-edit *.generated.json; "
        + "do NOT start 3C; do NOT touch W0-B k8s engine.\n"
        + "Procedure:\n"
        + "1) git fetch origin dev; isolated worktree from origin/dev; branch agent/cas-3b-buck2-cache-YYYYMMDD.\n"
        + "2) Create/update specs/reorg/buck2-cache-move-plan.json as sole active move plan for 3B.\n"
        + "3) Atomically move: toolchains/cache/{BUCK,defs.bzl,OWNERS} and "
        + "infra/ci/buckconfig/warm-cache-ro.buckconfig + warm-cache-rw.buckconfig "
        + "→ exactly build/buck2/cache/ (create tree).\n"
        + "4) Rewrite every active consumer label toolchains//cache:cache-platform → "
        + "//build/buck2/cache:cache-platform; update workflows/CI/tests that reference old paths.\n"
        + "5) Delete empty old source dirs; no compatibility shim.\n"
        + "6) Prove N_pre=N_post=N_promoted>0 with a positive differently-shaped probe; leave .buckconfig dark.\n"
        + "7) Reachability registry + dual-critic APPROVE packet under "
        + ".grok/programs/cas-fabric/evidence/pr-*-dual-critic.json; open PR vs dev; push.\n"
        + "8) Return: PR URL, head SHA, files moved, consumer count, hard-stop check list.",
    label: "cas-3b",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// --- CI opt: materialize-once / extra path-filters (after R2) ---
jobs.push(#{
    prompt: "IMPLEMENT CI optimization lane (post-#1562) for Oyatie.\n"
        + "R2 live-postgres path-filter is MERGED (#1562). ADR-0639 may still be open as #1569.\n"
        + "Worktree from origin/dev; branch agent/ci-path-opt-materialize-YYYYMMDD.\n"
        + "Goals (one concern, docs-safe if unsure): (A) document + if safe implement single materialize "
        + "producer→consumers pattern notes in workflow comments citing ADR-0515 singleton; "
        + "(B) optional second path-optional specialty only if clearly safe and fail-open-to-run "
        + "(not skip) like #1562 — do NOT path-skip firewall/freshness/affected-set/buck2 without "
        + "Accepted ADR-0639 on trunk. Prefer: PLAN + small PR fixing workflow comments / one proven "
        + "path-filter if 0639 already Accepted; else docs-only PR for materialize-once design under "
        + ".grok/programs/hyperscaler-delivery-lanes/.\n"
        + "Hard stops: no second protected context; no weaken face policy; no couple to CAS 3B files.\n"
        + "Dual-critic + open PR. Return PR URL or why docs-only.",
    label: "ci-path-opt",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// --- Post-merge packets for promoted work ---
jobs.push(#{
    prompt: "Write post-merge product-completion packets (R3 template) for recently MERGED work on Oyatie:\n"
        + "- #1563 CAS Lane 3A NativeLink rehome (promoted 010c132ec class)\n"
        + "- #1562 path-filter live-postgres\n"
        + "- #1565 face-decommit residual\n"
        + "- Ensure G039/#1558 packet status updated if trunk green\n"
        + "Use template .grok/programs/hyperscaler-delivery-lanes/R3-postmerge-packet-template.md.\n"
        + "Write under .grok/programs/*/evidence/*post-merge* (json or md). Fill: promoted commit, "
        + "oya-ci-required green on trunk if available, rollout N/A for docs, rollback note, "
        + "observability, release-note impact, agent-observation harvest.\n"
        + "Also draft G001 packet structure ready to fill when #1561 merges "
        + "(.grok/programs/k8s-port/evidence/G001-post-merge-packet-DRAFT.md update).\n"
        + "Do not claim ultragoal complete. Commit on a docs branch agent/postmerge-packets-YYYYMMDD "
        + "from origin/dev and open PR if substantive files. Dual-critic if PR.",
    label: "postmerge-packets",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// --- Red PR fixes (fail-closed, no scope expand) ---
jobs.push(#{
    prompt: "Fix RED open PRs only (Oyatie). Priority: #1569 ADR-0639 (if CI red/fail), #1570 cancel-in-progress, "
        + "#1566 harness residue, #1568 dual-home inventory. For each: gh pr checks, authoritative job log "
        + "root cause, fix in that PR's worktree only, dual-critic re-head, push. Do not implement CAS 3B "
        + "or W0-B here. Do not expand while diagnosing. Return per-PR: fixed / waiting / blocked.\n"
        + "Repo /Users/jasonlee/Developer/oyatie. Use existing worktrees if present.",
    label: "red-pr-fix",
    capability_mode: "all",
    agent_type: "general-purpose",
});

// --- RE sandbox docs only (always unblocked, no code) ---
jobs.push(#{
    prompt: "Update Oyatie CAS/RE sandbox readiness docs ONLY (no implementation):\n"
        + ".grok/programs/cas-fabric/evidence/RE-SANDBOX-READINESS.md — mark 3A MERGED with evidence "
        + "commit/PR #1563; set 3B as NEXT implementable; keep RE blocked. Refresh timestamps. "
        + "Optional: tiny docs PR agent/cas-readiness-refresh-YYYYMMDD from origin/dev. "
        + "Hard stop: zero remote_enabled, zero credentials, zero warm CAS flips.",
    label: "re-sandbox-docs",
    capability_mode: "all",
    agent_type: "general-purpose",
});

let results = parallel(jobs);

phase("Synthesize");
let synth = agent(
    "Synthesize post-3A implementation fan-out. From agent results produce markdown: "
    + "(1) CAS 3B PR status (2) CI path-opt (3) post-merge packets (4) red PR fixes (5) hard stops held "
    + "(6) still blocked: W0-B until G001, warm CAS, RE. Fail-closed if 3B agent did not open PR. "
    + "Results:\n" + json_encode(results),
    #{ label: "synth", capability_mode: "read-only", agent_type: "explore" }
);

let report = "Synthesis failed — inspect fan-out panel.";
if synth != () && synth.success && synth.output != () {
    report = synth.output;
}
complete(#{ report: report, inventory: inv });
