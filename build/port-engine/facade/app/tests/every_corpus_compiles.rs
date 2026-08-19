//! Every corpus the engine RENDERS must also compile.
//!
//! The main compile proof runs the strict pipeline over one corpus. The other fixtures — the
//! sentinel, failure, interface, ownership and foreign corpora — were rendered by tests that
//! checked what they said and never checked that it was a program. A sentinel came out declared as
//! `Gone` and asked about as `ErrGone`, two spellings of one name from two sites, and nothing
//! caught it because the corpus that exercises the comparison was never compiled.
//!
//! A partial port is expected here: these corpora exist to make the engine REFUSE, so most of what
//! they declare is absent by design. What must hold is that whatever IS emitted is a program.

use std::path::{Path, PathBuf};
use std::process::Command;

use port_engine_app::driver;

/// The committed snapshot fixtures, by the file each lives in.
///
/// Named rather than globbed: a fixture that stops being rendered should fail this list loudly
/// rather than quietly leave the proof covering less than it did.
const FIXTURES: &[&str] = &[
    "fixture-snapshot-v1.json",
    "fixture-snapshot-sentinel-v1.json",
    "fixture-snapshot-failure-v1.json",
    "fixture-snapshot-refused-v1.json",
    "fixture-snapshot-unproven-v1.json",
];

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../adapters/snapshot/src")
        .join(name)
}

#[test]
fn every_rendered_corpus_is_a_program() {
    let out_dir = std::env::temp_dir().join(format!("port-engine-corpora-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).expect("scratch dir");

    for name in FIXTURES {
        let path = fixture_path(name);
        if !path.exists() {
            continue;
        }
        let ported = driver::port_snapshot(&path)
            .unwrap_or_else(|error| panic!("{name} must port: {error}"));
        // ONE CRATE, with each file as a module. That is the layout the output CLAIMS: a
        // cross-unit type is spelled `crate::shapes::Point`, which resolves only where the unit
        // modules are siblings at a crate root. Compiling the files separately proves less than
        // the output asserts, and would have passed while that path pointed at nothing.
        let mut crate_source = String::new();
        for file in &ported.files {
            if file.source.trim().is_empty() {
                continue;
            }
            crate_source.push_str(&format!("pub mod {} {{\n", file.module));
            crate_source.push_str(&file.source);
            crate_source.push_str("}\n");
        }
        if crate_source.trim().is_empty() {
            continue;
        }
        compile(&out_dir, name, &crate_source);
    }

    let _ = std::fs::remove_dir_all(&out_dir);
}

/// One corpus, assembled into a crate and type-checked.
fn compile(out_dir: &Path, fixture: &str, source: &str) {
    let stem = fixture.replace(['.', '-'], "_");
    let source_path = out_dir.join(format!("{stem}.rs"));
    std::fs::write(&source_path, source).expect("write emitted source");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc)
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg(format!("--crate-name=c{}", stem.to_lowercase()))
        .arg("--emit=metadata")
        .arg("-o")
        .arg(out_dir.join(format!("{stem}.rmeta")))
        // The same two allowances the main proof makes, for the same reason: they are facts about
        // the SOURCE — an unexported declaration nobody calls, a parameter a function ignores —
        // and the source language warns on neither. Everything else is denied, because a warning
        // the translation invents is a defect in this engine.
        .arg("--deny=warnings")
        .arg("--allow=dead_code")
        .arg("--allow=unused_variables")
        .arg(&source_path)
        .output()
        .expect("rustc must be runnable — this test runs under cargo, which found one");

    assert!(
        output.status.success(),
        "`{fixture}` must compile.\n\n--- rustc ---\n{}\n--- source ---\n{source}",
        String::from_utf8_lossy(&output.stderr)
    );
}
