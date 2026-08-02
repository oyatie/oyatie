//! # corpus-yaml-facts — the I/O adapter a buck2 action invokes
//!
//! One invocation = one buck2 extraction action = one shard of the corpus graph. The pure work
//! lives in `corpus-yaml-kernel`; this binary only reads the files buck2 declared as inputs and
//! writes the shard buck2 declared as an output.
//!
//! ```text
//! corpus-yaml-facts --target //some:target --out <shard.json> <file.yaml>...
//! ```
//!
//! Every input is passed explicitly on the command line — there is NO filesystem walk and no
//! `git ls-files`. That is what makes the action's inputs exactly what buck2 declared, which is the
//! precondition for buck2's action cache to give correct incremental re-extraction for free.
//!
//! Exits non-zero if any declared input is unreadable: a missing input is a wiring fault, and
//! emitting a clean-looking shard for it would be a false green.

use std::path::Path;
use std::process::ExitCode;

use corpus_yaml_kernel::{Edge, EdgeKind, GraphFace, Node, NodeId};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("corpus-yaml-facts: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let (target, out, prefix, inputs) = parse_args()?;

    let target_id = NodeId::target(&target);
    // The Target node's digest is over its label alone: it is the shard's anchor, deliberately NOT
    // a roll-up of its children. A roll-up would make every YAML edit churn the target, which is
    // the parent-churn pathology this schema exists to avoid.
    let mut nodes = vec![Node {
        id: target_id.clone(),
        digest: corpus_core::ContentHash::of(target.as_bytes()),
    }];
    let mut edges = Vec::new();
    let mut opaque = Vec::new();

    for input in &inputs {
        let source = std::fs::read_to_string(Path::new(input))
            .map_err(|error| format!("cannot read declared input {input}: {error}"))?;
        // Read from the path buck2 gave us; ADDRESS by the repo-relative path, so identity is
        // independent of where the action happened to run.
        let container = corpus_yaml_kernel::repo_relative(&prefix, input);
        let facts = corpus_yaml_kernel::extract(&container, &source);

        edges.push(Edge {
            kind: EdgeKind::Contains,
            src: target_id.clone(),
            dst: NodeId::file(&container),
        });
        nodes.extend(facts.nodes);
        edges.extend(facts.edges);
        opaque.extend(facts.opaque);
    }

    let face = GraphFace::new(target, nodes, edges, opaque);
    let json = face
        .to_canonical_json()
        .map_err(|error| format!("cannot serialize shard: {error}"))?;
    std::fs::write(Path::new(&out), json)
        .map_err(|error| format!("cannot write shard {out}: {error}"))?;
    Ok(())
}

/// Parse `--target <label> --out <path> [--prefix <package dir>] <input>...`.
fn parse_args() -> Result<(String, String, String, Vec<String>), String> {
    let mut target = None;
    let mut out = None;
    let mut prefix = String::new();
    let mut inputs = Vec::new();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target" => {
                target = Some(args.next().ok_or("--target needs a value")?);
            }
            "--out" => {
                out = Some(args.next().ok_or("--out needs a value")?);
            }
            "--prefix" => {
                prefix = args.next().ok_or("--prefix needs a value")?;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag {other}"));
            }
            other => inputs.push(other.to_owned()),
        }
    }

    let target = target.ok_or("--target is required")?;
    let out = out.ok_or("--out is required")?;
    // An extraction action with zero declared inputs is a wiring fault, not an empty result: it
    // would write a clean, empty, cacheable shard forever.
    if inputs.is_empty() {
        return Err("no input files given — an empty extraction is a wiring fault".to_owned());
    }
    // Deterministic shard bytes must not depend on the order buck2 expanded `$SRCS`.
    inputs.sort();
    inputs.dedup();
    Ok((target, out, prefix, inputs))
}
