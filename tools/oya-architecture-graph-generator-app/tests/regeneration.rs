//! Repo-local regeneration proofs (always on, no external golden required):
//!   1. Rendering from the committed template + SSOT + controller-materialized
//!      masterplan is deterministic (render twice -> identical bytes). The
//!      de-committed `product-graph.html` is no longer a git-tracked golden.
//!   2. The baked `const GRAPH = {...};` literal parses as JSON and carries the
//!      five dashboard keys in order.

use std::path::PathBuf;

use oya_architecture_graph_generator_app::render;
use serde_json::Value;

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

fn render_dashboard() -> String {
    let root = repo_root();
    render(
        &root.join("docs/machine-readable/architecture-graph.json"),
        &root.join("docs/machine-readable/masterplan.generated.json"),
        &root.join("docs/architecture/product-graph.template.html"),
    )
    .expect("render succeeds")
}

#[test]
fn regenerated_product_graph_is_deterministic() {
    let first = render_dashboard();
    let second = render_dashboard();
    assert_eq!(
        first, second,
        "regenerating product-graph.html from identical source inputs must be byte-deterministic; failures indicate nondeterministic generation or source-input drift, not a missing committed HTML golden"
    );
}

#[test]
fn baked_graph_literal_parses_with_dashboard_keys() {
    let rendered = render_dashboard();
    let prefix = "const GRAPH = ";
    let start = rendered.find(prefix).expect("const GRAPH present") + prefix.len();
    // Brace-match to extract the JSON object literal.
    let bytes = rendered.as_bytes();
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    let mut end = start;
    for (offset, &b) in bytes[start..].iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + offset + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    let literal = &rendered[start..end];
    let parsed: Value = serde_json::from_str(literal).expect("baked GRAPH parses as JSON");
    let keys: Vec<&str> = parsed
        .as_object()
        .expect("GRAPH is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        ["_meta", "verticals", "techstack", "masterplan", "lanes"]
    );
}
