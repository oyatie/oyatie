#[allow(dead_code)]
#[path = "../ci/assert-result-bundle-output.rs"]
mod json_support;

use json_support::Json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap())
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("oyatie-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn canonical_contexts_live_json(root: &Path) -> String {
    let text = fs::read_to_string(root.join("infra/branch-protection/dev.json")).unwrap();
    let parsed = json_support::parse_json(&text).unwrap();
    let contexts = parsed
        .as_object()
        .and_then(|object| object.get("required_status_checks"))
        .and_then(Json::as_object)
        .and_then(|object| object.get("contexts"))
        .and_then(Json::as_array)
        .expect("required contexts")
        .iter()
        .map(|item| item.as_str().expect("context string"))
        .collect::<Vec<_>>();
    format!(
        "{{\"strict\":false,\"contexts\":[{}]}}\n",
        contexts
            .iter()
            .map(|context| json_string(context))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn fake_gh_source() -> &'static str {
    r#"
use std::env;
use std::fs;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mode = env::var("OYA_TEST_GH_MODE").unwrap_or_else(|_| "missing".to_string());
    if matches!(mode.as_str(), "missing" | "match") {
        if args.first().map(String::as_str) == Some("repo")
            && args.get(1).map(String::as_str) == Some("view")
        {
            println!("jason931225/oyatie");
            return;
        }
        if args.first().map(String::as_str) == Some("api")
            && args
                .get(1)
                .is_some_and(|value| value.starts_with("repos/") && value.ends_with("/branches/dev/protection/required_status_checks"))
        {
            print!("{}", fs::read_to_string(env::var("OYA_TEST_LIVE_CONTEXTS").expect("live contexts")).unwrap());
            return;
        }
        if args.first().map(String::as_str) == Some("pr")
            && args.get(1).map(String::as_str) == Some("list")
        {
            println!("[]");
            return;
        }
    }
    if mode == "forbidden" {
        if args.first().map(String::as_str) == Some("repo")
            && args.get(1).map(String::as_str) == Some("view")
        {
            println!("jason931225/oyatie");
            return;
        }
        if args.first().map(String::as_str) == Some("api")
            && args
                .get(1)
                .is_some_and(|value| value.starts_with("repos/") && value.ends_with("/branches/dev/protection/required_status_checks"))
        {
            eprintln!("Resource not accessible by integration");
            process::exit(1);
        }
    }
    eprintln!("unexpected gh invocation: {}", args.join(" "));
    process::exit(99);
}
"#
}

fn compile_fake_gh(tmp: &Path) -> PathBuf {
    let bin_dir = tmp.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let src = tmp.join("fake-gh.rs");
    let bin = bin_dir.join("gh");
    fs::write(&src, fake_gh_source()).unwrap();
    let output = Command::new("rustc")
        .args(["--edition=2021", "-D", "warnings"])
        .arg(&src)
        .arg("-o")
        .arg(&bin)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fake gh compile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    bin_dir
}

fn run_trigger(
    root: &Path,
    fake_bin: &Path,
    mode: &str,
    live_contexts: &Path,
    extra: &[&str],
) -> Output {
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        env::var("PATH").unwrap_or_default()
    );
    let mut command = Command::new(root.join("scripts/trigger-next-queue-automerge.sh"));
    command
        .current_dir(root)
        .env("PATH", path)
        .env("OYA_TEST_GH_MODE", mode)
        .env("OYA_TEST_LIVE_CONTEXTS", live_contexts)
        .args(["--base-ref", "HEAD", "--dry-run"])
        .args(extra);
    command.output().unwrap()
}

fn assert_contains(haystack: &[u8], needle: &str, label: &str) {
    let text = String::from_utf8_lossy(haystack);
    assert!(
        text.contains(needle),
        "{label} missing {needle:?}; observed:\n{text}"
    );
}

#[test]
fn required_context_drift_and_permissions_fail_closed_before_automerge() {
    let root = repo_root();
    let tmp = temp_dir("trigger-required-contexts");
    let fake_bin = compile_fake_gh(&tmp);
    let live_missing = tmp.join("live-missing.json");
    let live_match = tmp.join("live-match.json");
    fs::write(
        &live_missing,
        "{\"strict\":false,\"contexts\":[\"legacy-feedback\"]}\n",
    )
    .unwrap();
    fs::write(&live_match, canonical_contexts_live_json(&root)).unwrap();

    let missing = run_trigger(&root, &fake_bin, "missing", &live_missing, &[]);
    assert!(
        !missing.status.success(),
        "missing-context scenario should fail closed"
    );
    assert_contains(
        &missing.stderr,
        "live branch-protection required contexts drift",
        "missing stderr",
    );
    assert_contains(&missing.stderr, "missing_from_live=", "missing stderr");
    assert_contains(
        &missing.stderr,
        "github-lane-unlocker-required",
        "missing stderr",
    );

    let forbidden = run_trigger(&root, &fake_bin, "forbidden", &live_missing, &[]);
    assert!(
        !forbidden.status.success(),
        "forbidden branch-protection read should fail closed"
    );
    assert_contains(
        &forbidden.stderr,
        "Administration read permission",
        "forbidden stderr",
    );
    assert_contains(
        &forbidden.stderr,
        "Resource not accessible by integration",
        "forbidden stderr",
    );

    let unsafe_method = run_trigger(
        &root,
        &fake_bin,
        "unsafe-method",
        &live_match,
        &["--merge-method", "merge"],
    );
    assert!(
        !unsafe_method.status.success(),
        "unsafe merge method should fail before automerge"
    );
    assert_contains(
        &unsafe_method.stderr,
        "--merge-method is fixed to squash",
        "unsafe-method stderr",
    );

    let matched = run_trigger(&root, &fake_bin, "match", &live_match, &[]);
    assert!(
        matched.status.success(),
        "matching live contexts should reach no-PR dry-run path\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&matched.stdout),
        String::from_utf8_lossy(&matched.stderr)
    );
    assert_contains(
        &matched.stdout,
        "live branch-protection required contexts match",
        "match stdout",
    );
    assert_contains(
        &matched.stdout,
        "no open PR remains at or after queue floor #1",
        "match stdout",
    );

    fs::remove_dir_all(tmp).unwrap();
}
