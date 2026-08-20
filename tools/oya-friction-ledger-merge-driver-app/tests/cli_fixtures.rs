//! Process-level fixtures for the merge-driver binary (`%O %A %B` contract): success writes the
//! canonical merge over `%A`; ANY failure leaves `%A` byte-untouched (FRIC-1781370000 incident 2 —
//! the driver never writes garbage, git falls back to a normal conflict).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn driver_bin() -> PathBuf {
    if let Ok(path) = std::env::var("OYA_FRICTION_LEDGER_MERGE_DRIVER") {
        return PathBuf::from(path);
    }
    // cargo (supplementary signal) sets CARGO_BIN_EXE_* for integration tests; buck2 (THE signal)
    // injects the env above via $(location :oya-friction-ledger-merge-driver).
    match option_env!("CARGO_BIN_EXE_oya-friction-ledger-merge-driver") {
        Some(path) => PathBuf::from(path),
        None => panic!("missing OYA_FRICTION_LEDGER_MERGE_DRIVER"),
    }
}

fn unique_dir(label: &str) -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "oya-friction-ledger-merge-driver-{label}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_file(path: &Path, contents: &str) {
    std::fs::write(path, contents).expect("write fixture");
}

fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path).expect("read fixture")
}

fn run(base: &Path, current: &Path, other: &Path) -> std::process::Output {
    Command::new(driver_bin())
        .arg(base)
        .arg(current)
        .arg(other)
        .output()
        .expect("run merge driver")
}

fn primary(id: &str, seen_at: &str, friction: &str) -> String {
    format!(
        "{{\"id\": \"{id}\", \"seen_at\": \"{seen_at}\", \"friction\": \"{friction}\", \
         \"enforcement_fix\": \"wire a gate for {id}\", \"status\": \"open\", \"goal\": \"G011\"}}\n"
    )
}

#[test]
fn successful_merge_overwrites_current_with_the_canonical_union() {
    let dir = unique_dir("success");
    let base = dir.join("base.jsonl");
    let current = dir.join("current.jsonl");
    let other = dir.join("other.jsonl");
    let anchor = primary("FRIC-A", "2026-06-10", "base friction");
    write_file(&base, &anchor);
    write_file(
        &current,
        &format!("{anchor}{}", primary("FRIC-X", "2026-06-11", "ours")),
    );
    write_file(
        &other,
        &format!("{anchor}{}", primary("FRIC-Y", "2026-06-12", "theirs")),
    );

    let output = run(&base, &current, &other);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let merged = read_file(&current);
    let ids: Vec<bool> = ["FRIC-A", "FRIC-X", "FRIC-Y"]
        .iter()
        .map(|id| merged.contains(&format!("\"id\": \"{id}\"")))
        .collect();
    assert_eq!(
        ids,
        vec![true, true, true],
        "all three rows present: {merged}"
    );
    assert_eq!(merged.lines().count(), 3);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn second_author_conversion_end_to_end() {
    let dir = unique_dir("second-author");
    let base = dir.join("base.jsonl");
    let current = dir.join("current.jsonl");
    let other = dir.join("other.jsonl");
    let anchor = primary("FRIC-A", "2026-06-10", "base friction");
    write_file(&base, &anchor);
    write_file(
        &current,
        &format!(
            "{anchor}{}",
            primary("FRIC-N", "2026-06-11", "first author")
        ),
    );
    write_file(
        &other,
        &format!(
            "{anchor}{}",
            primary("FRIC-N", "2026-06-12", "second author")
        ),
    );

    let output = run(&base, &current, &other);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let merged = read_file(&current);
    assert!(
        merged.contains("\"friction\": \"first author\""),
        "{merged}"
    );
    assert!(
        merged.contains("\"status_update\": \"open\"")
            && !merged.contains("\"friction\": \"second author\""),
        "second author converted to an update row: {merged}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn garbage_side_exits_2_and_current_stays_byte_untouched() {
    let dir = unique_dir("garbage");
    let base = dir.join("base.jsonl");
    let current = dir.join("current.jsonl");
    let other = dir.join("other.jsonl");
    let anchor = primary("FRIC-A", "2026-06-10", "base friction");
    let original_current = format!("{anchor}{}", primary("FRIC-X", "2026-06-11", "ours"));
    write_file(&base, &anchor);
    write_file(&current, &original_current);
    write_file(&other, "{\"id\": \"FRIC-B\", \"truncated\n");

    let output = run(&base, &current, &other);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(read_file(&current), original_current, "never write garbage");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn committed_conflict_markers_exit_2_and_current_stays_byte_untouched() {
    let dir = unique_dir("conflict-markers");
    let base = dir.join("base.jsonl");
    let current = dir.join("current.jsonl");
    let other = dir.join("other.jsonl");
    let anchor = primary("FRIC-A", "2026-06-10", "base friction");
    let crashed = format!(
        "<<<<<<< HEAD\n{}=======\n{}>>>>>>> other\n",
        primary("FRIC-B", "2026-06-11", "ours"),
        primary("FRIC-B", "2026-06-11", "theirs"),
    );
    write_file(&base, &anchor);
    write_file(&current, &crashed);
    write_file(&other, &anchor);

    let output = run(&base, &current, &other);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(
        read_file(&current),
        crashed,
        "the crash artifact is refused, not laundered"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn corrupt_base_with_duplicate_primaries_exits_1() {
    let dir = unique_dir("corrupt-base");
    let base = dir.join("base.jsonl");
    let current = dir.join("current.jsonl");
    let other = dir.join("other.jsonl");
    let corrupt = format!(
        "{}{}",
        primary("FRIC-A", "2026-06-10", "first"),
        primary("FRIC-A", "2026-06-11", "second"),
    );
    write_file(&base, &corrupt);
    write_file(&current, &corrupt);
    write_file(&other, &corrupt);

    let output = run(&base, &current, &other);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(read_file(&current), corrupt);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn wrong_argument_count_exits_2() {
    let output = Command::new(driver_bin())
        .arg("only-one-arg")
        .output()
        .expect("run merge driver");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
}
