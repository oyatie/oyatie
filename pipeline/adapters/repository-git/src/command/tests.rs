use std::io::Cursor;
use std::time::{Duration, Instant};

use pipeline_repository::NoCancellation;

use super::*;

#[test]
fn deadline_terminates_and_reaps_the_child() {
    let root = std::env::current_dir().unwrap();
    let runner = GitCommandRunner::new(PathBuf::from("/bin/sleep"), root);
    let control = NoCancellation::until(Instant::now() + Duration::from_millis(20));
    let started = Instant::now();
    let result = runner.run(
        "deadline fixture",
        &[OsString::from("5")],
        Vec::new(),
        1024,
        1024,
        &control,
    );

    assert!(matches!(result, Err(SnapshotFailure::DeadlineExceeded)));
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(runner.invocations(), 1);
}

#[test]
fn bounded_reader_refuses_before_retaining_excess_output() {
    let result = read_bounded(Cursor::new(b"abc"), 2, "stdout bytes", None);

    assert!(matches!(
        result,
        Err(SnapshotFailure::LimitExceeded {
            limit: "stdout bytes",
            maximum: 2,
            observed: 3,
        })
    ));
}

#[test]
fn stdout_limit_terminates_a_child_that_ignores_the_closed_pipe() {
    assert_stream_limit_terminates(
        "stdout bytes",
        "trap '' PIPE; while :; do printf '0123456789abcdef' || :; done 2>/dev/null",
    );
}

#[test]
fn stderr_limit_terminates_a_child_that_ignores_the_closed_pipe() {
    assert_stream_limit_terminates(
        "stderr bytes",
        "trap '' PIPE; exec 3>&2; exec 2>/dev/null; while :; do printf '0123456789abcdef' >&3 || :; done",
    );
}

fn assert_stream_limit_terminates(limit: &'static str, script: &str) {
    let root = std::env::current_dir().unwrap();
    let runner = GitCommandRunner::new(PathBuf::from("/bin/sh"), root);
    let control = NoCancellation::until(Instant::now() + Duration::from_secs(2));
    let started = Instant::now();
    let result = runner.run(
        "output limit fixture",
        &[OsString::from("-c"), OsString::from(script)],
        Vec::new(),
        1024,
        1024,
        &control,
    );

    assert!(matches!(
        result,
        Err(SnapshotFailure::LimitExceeded {
            limit: observed,
            maximum: 1024,
            ..
        }) if observed == limit
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
}
