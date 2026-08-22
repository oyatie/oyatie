//! Golden tests: the generator's output must match the VERIFIED dashboard
//! dataset byte-for-byte / structurally.
//!
//! Two proofs:
//!   1. `masterplan_transform_reproduces_golden_byte_for_byte` — the masterplan
//!      projection transform applied to the controller face (or the same
//!      projection rendered into test-only temporary storage) reproduces the
//!      `masterplan` section of the verified `_graph-data.json` byte-for-byte.
//!   2. `merged_graph_deep_equals_golden` — the full merged `GRAPH`
//!      (`_meta, verticals, techstack, masterplan, lanes`) deep-equals the
//!      verified `_graph-data.json`.
//!
//! The verified golden `_graph-data.json` lives OUTSIDE this repo (it is the
//! audit reference at
//! `linux/docs/audit/initial-sweep-2026-06-06/architecture/_graph-data.json`).
//! Its path is supplied via the `OYATIE_ARCH_GRAPH_GOLDEN` env var, defaulting to
//! the known absolute audit location. When the golden is unavailable (e.g. on a
//! CI runner that does not check out the audit tree) these tests skip with a
//! printed notice rather than failing — the regeneration drift gate
//! (`tests/regeneration.rs`) is the always-on, repo-local proof.

use std::path::{Path, PathBuf};

use architecture_graph_generator_app::{masterplan_from_generated, merge_graph};
use serde_json::Value;

mod support;

use support::resolve_masterplan_input;

const DEFAULT_GOLDEN: &str = "/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/architecture/_graph-data.json";

fn repo_root() -> PathBuf {
    std::env::current_dir()
        .expect("current dir readable")
        .ancestors()
        .find(|dir| {
            dir.join("docs/machine-readable/architecture-graph.json")
                .exists()
        })
        .expect("could not locate Oyatie repo root from current dir")
        .to_path_buf()
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

fn golden_path() -> PathBuf {
    std::env::var("OYATIE_ARCH_GRAPH_GOLDEN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_GOLDEN))
}

fn load_golden() -> Option<Value> {
    let path = golden_path();
    if !path.exists() {
        eprintln!(
            "SKIP: golden reference not found at {} (set OYATIE_ARCH_GRAPH_GOLDEN to override)",
            path.display()
        );
        return None;
    }
    Some(read_json(&path))
}

#[test]
fn masterplan_transform_reproduces_golden_byte_for_byte() {
    let Some(golden) = load_golden() else {
        return;
    };
    let golden_masterplan = golden
        .get("masterplan")
        .expect("golden has masterplan section");

    let root = repo_root();
    let masterplan_input =
        resolve_masterplan_input(&root).expect("masterplan projection input resolves");
    let generated = read_json(masterplan_input.path());
    let produced = masterplan_from_generated(&generated).expect("transform succeeds");

    // Structural equality (serde_json::Value::eq is order-insensitive for maps,
    // so also compare the pretty-printed byte stream which IS order-sensitive
    // because both crates serialize with preserve_order).
    assert_eq!(
        &produced, golden_masterplan,
        "transformed masterplan must deep-equal the golden masterplan section"
    );

    let produced_bytes = serde_json::to_string_pretty(&produced).unwrap();
    let golden_bytes = serde_json::to_string_pretty(golden_masterplan).unwrap();
    assert_eq!(
        produced_bytes, golden_bytes,
        "transformed masterplan must serialize byte-for-byte identical to golden"
    );
}

#[test]
fn merged_graph_deep_equals_golden() {
    let Some(golden) = load_golden() else {
        return;
    };

    let root = repo_root();
    let ssot = read_json(&root.join("docs/machine-readable/architecture-graph.json"));
    let masterplan_input =
        resolve_masterplan_input(&root).expect("masterplan projection input resolves");
    let generated = read_json(masterplan_input.path());
    let masterplan = masterplan_from_generated(&generated).expect("transform succeeds");
    let graph = merge_graph(&ssot, masterplan).expect("merge succeeds");

    assert_eq!(
        graph, golden,
        "merged GRAPH must deep-equal the verified _graph-data.json"
    );

    // Key order must match the dashboard exactly.
    let keys: Vec<&str> = graph
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        ["_meta", "verticals", "techstack", "masterplan", "lanes"]
    );
}
