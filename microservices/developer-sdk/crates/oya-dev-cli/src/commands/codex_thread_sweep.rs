use std::process::{Command, ExitCode};

use serde_json::Value;

const OWNER: &str = "jason931225";
const NAME: &str = "oyatie";
const CODEX_CONNECTOR_LOGIN: &str = "chatgpt-codex-connector";

#[derive(Clone, Debug, Eq, PartialEq)]
struct ThreadSummary {
    id: String,
    priority: String,
    path: String,
    title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ListArgs {
    p1_only: bool,
    pr_filter: Option<u64>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match run_inner(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

fn run_inner(args: Vec<String>) -> Result<(), String> {
    let Some((mode, rest)) = args.split_first() else {
        return Err(codex_thread_sweep_usage());
    };
    match mode.as_str() {
        "list" => {
            let list_args = parse_list_args(rest)?;
            let threads = fetch_threads(list_args.pr_filter)?;
            print_thread_list(&threads, list_args.p1_only);
            Ok(())
        }
        "show" => {
            let thread_id = parse_show_args(rest)?;
            let thread = fetch_thread_body(thread_id)?;
            print_thread_body(&thread);
            Ok(())
        }
        _ => Err(format!("unknown codex-thread-sweep command: {mode}")),
    }
}

fn codex_thread_sweep_usage() -> String {
    "usage: oya codex-thread-sweep list [--p1-only] [--pr N] | show <thread-id>".to_string()
}

fn parse_list_args(args: &[String]) -> Result<ListArgs, String> {
    let mut p1_only = false;
    let mut pr_filter = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--p1-only" => {
                p1_only = true;
                index += 1;
            }
            "--pr" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("error: --pr requires a PR number argument".to_string());
                };
                pr_filter = Some(value.parse::<u64>().map_err(|_| {
                    format!("error: --pr argument must be an integer, got: {value:?}")
                })?);
                index += 2;
            }
            unknown => return Err(format!("unknown codex-thread-sweep list flag: {unknown}")),
        }
    }
    Ok(ListArgs { p1_only, pr_filter })
}

fn parse_show_args(args: &[String]) -> Result<&str, String> {
    match args {
        [thread_id] => Ok(thread_id),
        _ => Err("error: show requires exactly one review-thread id".to_string()),
    }
}

fn fetch_threads(pr_filter: Option<u64>) -> Result<Vec<(u64, Vec<ThreadSummary>)>, String> {
    let mut threads_by_pr = Vec::<(u64, Vec<ThreadSummary>)>::new();
    let mut pr_cursor = None::<String>;
    loop {
        let response = gh_graphql(&open_prs_query(pr_cursor.as_deref()))?;
        let pr_page = response
            .pointer("/data/repository/pullRequests")
            .ok_or_else(|| "missing data.repository.pullRequests in gh response".to_string())?;
        let pr_nodes = required_array(pr_page, "pullRequests.nodes", "/nodes")?;
        for pr_value in pr_nodes {
            let pr = required_u64(pr_value, "pullRequest.number", "/number")?;
            if pr_filter.is_some_and(|filter| filter != pr) {
                continue;
            }
            let mut thread_nodes = required_array(
                pr_value,
                "pullRequest.reviewThreads.nodes",
                "/reviewThreads/nodes",
            )?
            .to_vec();
            let review_threads = pr_value
                .pointer("/reviewThreads")
                .ok_or_else(|| format!("missing reviewThreads for PR {pr}"))?;
            let mut thread_cursor = page_cursor(review_threads, "reviewThreads")?;
            while let Some(cursor) = thread_cursor {
                let page = gh_graphql(&thread_page_query(pr, &cursor))?;
                let threads = page
                    .pointer("/data/repository/pullRequest/reviewThreads")
                    .ok_or_else(|| {
                        format!("missing data.repository.pullRequest.reviewThreads for PR {pr}")
                    })?;
                thread_nodes
                    .extend(required_array(threads, "reviewThreads.nodes", "/nodes")?.to_vec());
                thread_cursor = page_cursor(threads, "reviewThreads")?;
            }

            let summaries = thread_nodes
                .iter()
                .filter_map(parse_codex_thread_summary)
                .collect::<Vec<_>>();
            if !summaries.is_empty() {
                threads_by_pr.push((pr, summaries));
            }
        }

        if !page_has_next(pr_page, "pullRequests")? {
            break;
        }
        pr_cursor = page_end_cursor(pr_page, "pullRequests")?;
    }
    threads_by_pr.sort_by_key(|(pr, _)| *pr);
    Ok(threads_by_pr)
}

fn fetch_thread_body(thread_id: &str) -> Result<Value, String> {
    let response = gh_graphql(&thread_body_query(thread_id, None))?;
    let mut node = response
        .pointer("/data/node")
        .cloned()
        .ok_or_else(|| "missing data.node in gh response".to_string())?;
    if node.is_null() {
        return Err(format!(
            "thread {thread_id:?} not found in GraphQL response"
        ));
    }

    let comments = node
        .pointer("/comments")
        .cloned()
        .ok_or_else(|| format!("thread {thread_id:?} has no comments connection"))?;
    let mut cursor = page_cursor(&comments, "thread comments")?;
    while let Some(next_cursor) = cursor {
        let response = gh_graphql(&thread_body_query(thread_id, Some(&next_cursor)))?;
        let page = response
            .pointer("/data/node/comments")
            .ok_or_else(|| format!("missing comments page for thread {thread_id:?}"))?;
        let additional = required_array(page, "thread comments.nodes", "/nodes")?.to_vec();
        let Some(existing) = node
            .pointer_mut("/comments/nodes")
            .and_then(Value::as_array_mut)
        else {
            return Err(format!(
                "thread {thread_id:?} comments.nodes is not an array"
            ));
        };
        existing.extend(additional);
        cursor = page_cursor(page, "thread comments")?;
    }
    Ok(node)
}

fn gh_graphql(query: &str) -> Result<Value, String> {
    let output = Command::new("gh")
        .args(["api", "graphql", "-f"])
        .arg(format!("query={query}"))
        .output()
        .map_err(|error| format!("error: gh api graphql failed to start: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "error: gh api graphql failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice::<Value>(&output.stdout).map_err(|error| {
        format!(
            "error: failed to parse gh output as JSON: {error}\nraw output:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn open_prs_query(after: Option<&str>) -> String {
    let after_clause = after
        .map(|cursor| format!(", after: {cursor:?}"))
        .unwrap_or_default();
    format!(
        "query {{ repository(owner:{OWNER:?}, name:{NAME:?}) {{ \
         pullRequests(first:50, states:OPEN{after_clause}) {{ pageInfo {{ hasNextPage endCursor }} \
         nodes {{ number reviewThreads(first:100) {{ pageInfo {{ hasNextPage endCursor }} \
         nodes {{ id isResolved path comments(first:1) {{ nodes {{ author {{ login }} body }} }} }} }} }} }} }} }}"
    )
}

fn thread_page_query(pr: u64, cursor: &str) -> String {
    format!(
        "query {{ repository(owner:{OWNER:?}, name:{NAME:?}) {{ pullRequest(number:{pr}) {{ \
         reviewThreads(first:100, after:{cursor:?}) {{ pageInfo {{ hasNextPage endCursor }} \
         nodes {{ id isResolved path comments(first:1) {{ nodes {{ author {{ login }} body }} }} }} }} }} }} }}"
    )
}

fn thread_body_query(thread_id: &str, cursor: Option<&str>) -> String {
    let after_clause = cursor
        .map(|cursor| format!(", after:{cursor:?}"))
        .unwrap_or_default();
    format!(
        "query {{ node(id:{thread_id:?}) {{ ... on PullRequestReviewThread {{ \
         id isResolved path comments(first:100{after_clause}) {{ pageInfo {{ hasNextPage endCursor }} \
         nodes {{ author {{ login }} body }} }} }} }} }}"
    )
}

fn parse_codex_thread_summary(thread: &Value) -> Option<ThreadSummary> {
    if thread.get("isResolved")?.as_bool()? {
        return None;
    }
    let comments = thread.pointer("/comments/nodes")?.as_array()?;
    let first_comment = comments.first()?;
    if author_login(first_comment) != CODEX_CONNECTOR_LOGIN {
        return None;
    }
    let body = first_comment.get("body")?.as_str()?;
    let (priority, title) = summarize_comment_body(body);
    Some(ThreadSummary {
        id: thread.get("id")?.as_str()?.to_string(),
        priority,
        path: thread
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string(),
        title,
    })
}

fn summarize_comment_body(body: &str) -> (String, String) {
    let priority = if body.contains("P1 Badge") {
        "P1"
    } else if body.contains("P2 Badge") {
        "P2"
    } else {
        "?"
    };
    let mut title = String::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**") && !trimmed.contains("Badge") {
            title = trimmed.trim_matches('*').trim().to_string();
            break;
        }
        if trimmed.contains("Badge") && trimmed.ends_with("**") {
            title = trimmed
                .rsplit("</sub></sub>")
                .next()
                .unwrap_or(trimmed)
                .trim()
                .trim_matches('*')
                .trim()
                .to_string();
            break;
        }
    }
    if title.is_empty() {
        title = body.chars().take(80).collect::<String>().replace('\n', " ");
    }
    (priority.to_string(), title)
}

fn author_login(comment: &Value) -> String {
    comment
        .pointer("/author/login")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn print_thread_list(threads: &[(u64, Vec<ThreadSummary>)], p1_only: bool) {
    let total = threads
        .iter()
        .flat_map(|(_, summaries)| summaries.iter())
        .count();
    let p1_total = threads
        .iter()
        .flat_map(|(_, summaries)| summaries.iter())
        .filter(|thread| thread.priority == "P1")
        .count();
    let p2_total = total.saturating_sub(p1_total);
    println!(
        "Open PRs with codex threads: {}  total: {total}  P1: {p1_total}  P2: {p2_total}",
        threads.len()
    );
    for (pr, summaries) in threads {
        let visible = summaries
            .iter()
            .filter(|thread| !p1_only || thread.priority == "P1")
            .collect::<Vec<_>>();
        if visible.is_empty() {
            continue;
        }
        println!("\n#{pr}:");
        for thread in visible {
            println!(
                "  [{}] {} :: {}",
                thread.priority,
                thread.path,
                truncate_chars(&thread.title, 90)
            );
            println!("        id={}", thread.id);
        }
    }
}

fn print_thread_body(thread: &Value) {
    println!(
        "path: {}",
        thread.get("path").and_then(Value::as_str).unwrap_or("?")
    );
    println!(
        "resolved: {}",
        thread
            .get("isResolved")
            .and_then(Value::as_bool)
            .map(|resolved| resolved.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
    if let Some(comments) = thread.pointer("/comments/nodes").and_then(Value::as_array) {
        for comment in comments {
            println!("\n--- [{}]", author_login(comment));
            println!(
                "{}",
                comment.get("body").and_then(Value::as_str).unwrap_or("")
            );
        }
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn required_array<'a>(
    value: &'a Value,
    label: &str,
    pointer: &str,
) -> Result<&'a Vec<Value>, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{label} is missing or not an array"))
}

fn required_u64(value: &Value, label: &str, pointer: &str) -> Result<u64, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{label} is missing or not an integer"))
}

fn page_has_next(value: &Value, label: &str) -> Result<bool, String> {
    value
        .pointer("/pageInfo/hasNextPage")
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{label}.pageInfo.hasNextPage is missing or not a bool"))
}

fn page_end_cursor(value: &Value, label: &str) -> Result<Option<String>, String> {
    Ok(value
        .pointer("/pageInfo/endCursor")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            if page_has_next(value, label).unwrap_or(false) {
                Some(String::new())
            } else {
                None
            }
        }))
}

fn page_cursor(value: &Value, label: &str) -> Result<Option<String>, String> {
    if page_has_next(value, label)? {
        let cursor = page_end_cursor(value, label)?
            .filter(|cursor| !cursor.is_empty())
            .ok_or_else(|| format!("{label}.pageInfo.endCursor is required when hasNextPage"))?;
        Ok(Some(cursor))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_args_parse_p1_and_pr_filter() {
        let args = vec![
            "--p1-only".to_string(),
            "--pr".to_string(),
            "144".to_string(),
        ];
        let parsed = parse_list_args(&args).expect("list args parse");

        assert!(parsed.p1_only);
        assert_eq!(parsed.pr_filter, Some(144));
    }

    #[test]
    fn title_summary_handles_codex_badge_line() {
        let body = "<sub><sub>P2 Badge</sub></sub> **Tighten the validator**\n\nDetails";
        let (priority, title) = summarize_comment_body(body);

        assert_eq!(priority, "P2");
        assert_eq!(title, "Tighten the validator");
    }

    #[test]
    fn codex_summary_ignores_resolved_threads() {
        let thread = serde_json::json!({
            "id": "thread-1",
            "isResolved": true,
            "path": "src/lib.rs",
            "comments": {
                "nodes": [{
                    "author": {"login": "chatgpt-codex-connector"},
                    "body": "P1 Badge"
                }]
            }
        });

        assert_eq!(parse_codex_thread_summary(&thread), None);
    }
}
