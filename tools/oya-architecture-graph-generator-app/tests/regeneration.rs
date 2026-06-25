//! Repo-local regeneration proofs (always on, no external golden required):
//!   1. Rendering from the committed template + SSOT + masterplan equals the
//!      committed `product-graph.html` (the drift gate's invariant).
//!   2. The baked `const GRAPH = {...};` literal parses as JSON and carries the
//!      five dashboard keys in order.
//!   3. Rendering is idempotent (render twice -> identical bytes).

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
fn regenerated_matches_committed_html() {
    let rendered = render_dashboard();
    let committed =
        std::fs::read_to_string(repo_root().join("docs/architecture/product-graph.html"))
            .expect("committed product-graph.html readable");
    assert_eq!(
        rendered, committed,
        "committed product-graph.html must equal the regenerated dashboard (run --write to fix drift)"
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

#[test]
fn rendering_is_idempotent() {
    assert_eq!(render_dashboard(), render_dashboard());
}
