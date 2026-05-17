#!/usr/bin/env python3
"""codex-thread-sweep — REPORT-ONLY codex bot review thread inventory.

Codex bot posts P1/P2 review threads on every PR. With branch-protection
`require_conversation_resolution=true`, every unresolved thread blocks merge.

This script REPORTS unresolved threads grouped by PR + priority. It does NOT
bulk-resolve, because batch-resolving without auditing each thread is the
PR-#82-class anti-pattern (acknowledging-without-fixing) — see
`feedback_pr82_dishonest_exit_gate.md` and `feedback_codex_bulk_resolve_antipattern.md`.

Codex's P2 ("yellow") priority does NOT mean "ignorable." On PR #96 alone, all
5 P2 threads were real correctness defects in the validator script (missing
tools/* path coverage, false-pass when audit-chain.jsonl absent, wrong staged-
vs-branch diff scope, over-broad Cargo.toml grep, ignored staged state for
dep guard). Every codex thread requires per-thread audit + fix OR per-thread
push-back-with-reasoning before resolution.

Usage:
    python3 scripts/codex-thread-sweep.py list           # full inventory
    python3 scripts/codex-thread-sweep.py list --p1-only # P1s only
    python3 scripts/codex-thread-sweep.py list --pr 96   # one PR's threads
    python3 scripts/codex-thread-sweep.py show <thread-id>  # full body of one thread

Workflow expectation:
  1. Run `list` to inventory.
  2. For each thread: either fix the code OR post an inline reply explaining
     why the suggestion doesn't apply (link to ADR / scope decision / existing
     coverage).
  3. After EITHER action, manually resolve via:
       gh api graphql -f query='mutation { resolveReviewThread(input: {threadId:"<id>"}) { thread { id isResolved } } }'
  4. Never batch-resolve.

Per AGENTS.md §"codex review thread handling".
"""
import subprocess, json, sys

REPO = "jason931225/oyatie"
OWNER, NAME = REPO.split("/")


def gh(args):
    r = subprocess.run(["gh"] + args, capture_output=True, text=True)
    return r.stdout, r.returncode


def author_login(comment):
    return (comment.get("author") or {}).get("login") or "unknown"


def fetch_threads(pr_filter=None):
    """Fetch unresolved codex review threads, paginating over PRs and threads."""
    threads_by_pr = {}
    pr_cursor = None
    while True:
        after_clause = f', after: "{pr_cursor}"' if pr_cursor else ""
        query = (
            "query { repository(owner:\"%s\", name:\"%s\") { "
            "pullRequests(first:50, states:OPEN%s) { pageInfo { hasNextPage endCursor } "
            "nodes { number "
            "reviewThreads(first:100) { pageInfo { hasNextPage endCursor } "
            "nodes { id isResolved path "
            "comments(first:1) { nodes { author { login } body } } } } } } } }"
            % (OWNER, NAME, after_clause)
        )
        out, rc = gh(["api", "graphql", "-f", f"query={query}"])
        if rc != 0:
            print(f"error: gh api graphql failed (exit {rc}):\n{out}", file=sys.stderr)
            sys.exit(rc or 1)
        try:
            d = json.loads(out)
        except Exception as exc:
            print(f"error: failed to parse gh output as JSON: {exc}\nraw output:\n{out}", file=sys.stderr)
            sys.exit(1)
        pr_page = d["data"]["repository"]["pullRequests"]
        for p in pr_page["nodes"]:
            pr = p["number"]
            if pr_filter is not None and pr != pr_filter:
                continue
            # Paginate review threads within the PR
            thread_nodes = p["reviewThreads"]["nodes"]
            thread_page_info = p["reviewThreads"]["pageInfo"]
            thread_cursor = thread_page_info["endCursor"] if thread_page_info["hasNextPage"] else None
            while thread_cursor:
                tq = (
                    "query { repository(owner:\"%s\", name:\"%s\") { "
                    "pullRequest(number:%d) { "
                    "reviewThreads(first:100, after:\"%s\") { pageInfo { hasNextPage endCursor } "
                    "nodes { id isResolved path "
                    "comments(first:1) { nodes { author { login } body } } } } } } }"
                    % (OWNER, NAME, pr, thread_cursor)
                )
                tout, trc = gh(["api", "graphql", "-f", f"query={tq}"])
                if trc != 0:
                    print(f"error: gh api graphql failed (exit {trc}) paginating threads for PR {pr}:\n{tout}", file=sys.stderr)
                    sys.exit(trc or 1)
                try:
                    td = json.loads(tout)
                    tp = td["data"]["repository"]["pullRequest"]["reviewThreads"]
                    thread_nodes.extend(tp["nodes"])
                    thread_cursor = tp["pageInfo"]["endCursor"] if tp["pageInfo"]["hasNextPage"] else None
                except Exception as exc:
                    print(f"error: failed to parse thread pagination for PR {pr}: {exc}\nraw: {tout}", file=sys.stderr)
                    sys.exit(1)

            for t in thread_nodes:
                if t["isResolved"]:
                    continue
                comments = t["comments"]["nodes"]
                if not comments:
                    continue
                if author_login(comments[0]) != "chatgpt-codex-connector":
                    continue
                body = comments[0]["body"]
                priority = "P1" if "P1 Badge" in body else ("P2" if "P2 Badge" in body else "?")
                # Extract first-line title after the badge image
                title = ""
                for line in body.splitlines():
                    line = line.strip()
                    if line.startswith("**") and "Badge" not in line:
                        title = line.strip("* ").strip()
                        break
                    if "Badge" in line and line.endswith("**"):
                        title = line.split("</sub></sub>")[-1].strip("* ").strip()
                        break
                threads_by_pr.setdefault(pr, []).append({
                    "id": t["id"],
                    "priority": priority,
                    "path": t.get("path", "?"),
                    "title": title or body[:80].replace("\n", " "),
                })
        if not pr_page["pageInfo"]["hasNextPage"]:
            break
        pr_cursor = pr_page["pageInfo"]["endCursor"]
    return threads_by_pr


def fetch_thread_body(thread_id):
    """Fetch the full body of a single review thread, paginating comments."""
    query = (
        "query { node(id:\"%s\") { ... on PullRequestReviewThread { "
        "id isResolved path "
        "comments(first:100) { pageInfo { hasNextPage endCursor } "
        "nodes { author { login } body } } } } }" % thread_id
    )
    out, rc = gh(["api", "graphql", "-f", f"query={query}"])
    if rc != 0:
        print(f"error: gh api graphql failed (exit {rc}) for thread {thread_id}:\n{out}", file=sys.stderr)
        sys.exit(rc or 1)
    try:
        node = json.loads(out)["data"]["node"]
    except Exception as exc:
        print(f"error: failed to parse gh output for thread {thread_id}: {exc}\nraw: {out}", file=sys.stderr)
        sys.exit(1)
    if node is None:
        print(f"error: thread {thread_id!r} not found in GraphQL response", file=sys.stderr)
        sys.exit(1)
    # Paginate remaining comments if needed
    page_info = node["comments"]["pageInfo"]
    cursor = page_info["endCursor"] if page_info["hasNextPage"] else None
    while cursor:
        cq = (
            "query { node(id:\"%s\") { ... on PullRequestReviewThread { "
            "comments(first:100, after:\"%s\") { pageInfo { hasNextPage endCursor } "
            "nodes { author { login } body } } } } }" % (thread_id, cursor)
        )
        cout, crc = gh(["api", "graphql", "-f", f"query={cq}"])
        if crc != 0:
            print(f"error: gh api graphql failed (exit {crc}) paginating thread {thread_id}:\n{cout}", file=sys.stderr)
            sys.exit(crc or 1)
        try:
            cp = json.loads(cout)["data"]["node"]["comments"]
            node["comments"]["nodes"].extend(cp["nodes"])
            cursor = cp["pageInfo"]["endCursor"] if cp["pageInfo"]["hasNextPage"] else None
        except Exception as exc:
            print(f"error: failed to parse pagination response for thread {thread_id}: {exc}\nraw: {cout}", file=sys.stderr)
            sys.exit(1)
    return node


def cmd_list(p1_only, pr_filter):
    threads = fetch_threads(pr_filter)
    total = sum(len(v) for v in threads.values())
    p1_total = sum(1 for tlist in threads.values() for t in tlist if t["priority"] == "P1")
    p2_total = total - p1_total
    print(f"Open PRs with codex threads: {len(threads)}  total: {total}  P1: {p1_total}  P2: {p2_total}")
    for pr, tlist in sorted(threads.items()):
        if p1_only:
            tlist = [t for t in tlist if t["priority"] == "P1"]
        if not tlist:
            continue
        print(f"\n#{pr}:")
        for t in tlist:
            print(f"  [{t['priority']}] {t['path']} :: {t['title'][:90]}")
            print(f"        id={t['id']}")


def cmd_show(thread_id):
    t = fetch_thread_body(thread_id)
    if not t:
        print(f"thread not found: {thread_id}")
        return
    print(f"path: {t.get('path')}")
    print(f"resolved: {t.get('isResolved')}")
    for c in t["comments"]["nodes"]:
        print(f"\n--- [{author_login(c)}]")
        print(c["body"])


def main():
    if len(sys.argv) < 2:
        print("usage: codex-thread-sweep.py list [--p1-only] [--pr N] | show <thread-id>", file=sys.stderr)
        sys.exit(1)
    mode = sys.argv[1]
    if mode == "list":
        p1_only = "--p1-only" in sys.argv
        pr_filter = None
        if "--pr" in sys.argv:
            i = sys.argv.index("--pr")
            if i + 1 >= len(sys.argv):
                print("error: --pr requires a PR number argument (e.g. --pr 96)", file=sys.stderr)
                sys.exit(1)
            try:
                pr_filter = int(sys.argv[i + 1])
            except ValueError:
                print(f"error: --pr argument must be an integer, got: {sys.argv[i + 1]!r}", file=sys.stderr)
                sys.exit(1)
        cmd_list(p1_only, pr_filter)
    elif mode == "show" and len(sys.argv) >= 3:
        cmd_show(sys.argv[2])
    else:
        print(f"unknown command: {mode}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
