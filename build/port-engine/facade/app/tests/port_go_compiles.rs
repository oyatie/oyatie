//! The compile proof: the Rust this engine emits for the hermetic Go corpus is fed to `rustc`.
//!
//! Every other check in this crate proves the pipeline is STABLE — the same input produces the
//! same bytes, the bytes match a golden, the six receipt axes agree. None of them proves the bytes
//! are CORRECT. A golden over broken output is a golden that pins the breakage in place, and the
//! I3 run produced exactly that shape of defect twice: `pub` on trait methods and
//! `const NAME: String`, both of which `syn` parsed happily and `rustc` rejects. Parsing is not
//! compiling, and only one of them is evidence.
//!
//! `rustc` is invoked directly rather than `cargo check`. The emitted crate has no dependencies,
//! so there is nothing for a manifest to resolve; going through cargo would add a package-cache
//! lock, a target directory, and a second build graph nested inside the one already running this
//! test, for no additional coverage.
//!
//! This spawns a subprocess, which the engine's own libraries may never do — the ADR-0638 D3
//! firewall is about the engine's LIBRARY sources and about the source-language toolchain. This is
//! a test binary invoking the target-language compiler, on the output, after the fact. It is the
//! opposite of the thing the firewall forbids: nothing here feeds back into what the engine emits.

use std::process::Command;

use port_engine_app::driver;

#[test]
fn emitted_rust_compiles() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    let out_dir =
        std::env::temp_dir().join(format!("port-engine-compile-proof-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).expect("scratch dir");
    let source_path = out_dir.join("emitted.rs");
    std::fs::write(&source_path, &source).expect("write emitted source");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc)
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg("--crate-name=port_engine_emitted")
        // Metadata only: this proves the code type-checks, which is the claim. Producing an
        // artifact would prove the same thing more slowly.
        .arg("--emit=metadata")
        .arg("-o")
        .arg(out_dir.join("emitted.rmeta"))
        // EVERY OTHER WARNING IS DENIED, and that is the point of the pair of allowances rather
        // than a weakening of them. A warning the translation invents — a mutable temporary the
        // source did not have, an assignment nothing reads — is a defect in this engine, and one
        // that a reviewer had to find rather than a build. Denying the rest turns that whole class
        // into a build failure.
        //
        // These two are about the SOURCE, not the translation. Go warns on neither an unexported
        // declaration nobody calls nor a parameter a function ignores — `geometry.Lookup` genuinely
        // ignores its table — so denying them would fail the proof for output that is faithful.
        .arg("--deny=warnings")
        .arg("--allow=dead_code")
        .arg("--allow=unused_variables")
        // The third source property, and the one worth naming because a better translation would
        // absorb it. Go writes `x := 0` and then assigns in every branch, and warns on neither the
        // declaration nor the dead initial value; Rust's flow analysis sees the initialiser
        // overwritten before it is read. A faithful port of that Go produces this warning however
        // well it is done. What would remove it is emitting `let x = if c { a } else { b };` — the
        // target's `if` is an expression and the source's is not — which needs the front end to
        // report that the initial value is never read on any path. Until it does, denying this
        // would fail the proof for output that is faithful.
        .arg("--allow=unused_assignments")
        .arg(&source_path)
        .output()
        .expect("rustc must be runnable — this test runs under cargo, which found one");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "emitted Rust must compile.\n\n--- rustc ---\n{stderr}\n--- source ---\n{source}"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

/// The emitted modules must carry the corpus's declarations, named as Rust names them. Compiling
/// proves the output is valid Rust; this proves it is valid Rust ABOUT SOMETHING — an empty file
/// compiles too, and the engine's whole prior state was output that compiled and said nothing.
#[test]
fn emitted_rust_carries_the_corpus() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    for expected in [
        "pub mod basic",
        "pub mod shapes",
        "MAX_RETRIES",
        "DEFAULT_NAME",
        "fn add",
        "fn scale",
        "struct Celsius",
        "type ID",
        "struct Point",
        "impl Point",
        "fn shift",
        "trait Named",
    ] {
        assert!(
            source.contains(expected),
            "emitted source must carry `{expected}`:\n{source}"
        );
    }

    // A package variable NOTHING WRITES is a `static`, and it must be one rather than a `const`:
    // the source variable has one storage location for the life of the program and a `const` is
    // materialised afresh at every use, so `&X` would differ per use. The variable something
    // writes is the one that stays undecided, and it is proven in the refusal corpus.
    for expected in ["pub static ENABLED: bool = true", "pub static THRESHOLD: f64 = 0.75"] {
        assert!(
            source.contains(expected),
            "an unwritten package variable must be a static:\n{source}"
        );
    }
}

/// The third refusal class: a trait in a position the pack declares no form for.
///
/// The source holds an interface value directly and the target cannot — a trait has no size — so it
/// reaches a position as a reference, a box or a shared pointer, and those are different answers to
/// who owns the value. The pack declares the parameter form, where a borrow is unambiguously right,
/// Impls come from OBSERVED satisfaction, and the emitted crate says which observation.
///
/// Nothing in the source declares that `Label` implements `Named` in a form the engine could read
/// off a declaration — Go's interfaces are implicit. The impl exists because the front end saw a
/// concrete value flow into an interface-typed position, and the emitted doc comment carries which
/// kind of position it was, because a declared assertion is compile-checked by the source and an
/// The trait's receiver is DERIVED from its implementors, not declared once for all its methods.
///
/// The pack's declared mode is `exclusive`, which is right for `Rename` and wrong for `Name` — a
/// getter that takes `&mut self` is a signature no shared borrow can call. With the implementors
/// observed, each method takes the mode its implementors need: exclusive exactly when one of them
/// Embedding becomes explicit, on both sides of it.
///
/// The source composes by embedding and nothing forwards — an anonymous field lifts the embedded
/// type's methods into the outer type's method set, and an embedded interface lifts its
/// requirements into the outer interface's. The target has neither rule, so both have to be
/// written out: forwarding methods for the first, supertraits for the second.
///
/// `Driver` satisfies `Job` ONLY through a promoted method, which is why the two are proven
/// together. An engine that emitted the supertraits and skipped the promotion would produce an impl
/// A forwarding method binds the receiver the method it forwards to needs.
///
/// It has no body of its own to observe. `Engine::Run` mutates, so `Driver::run` cannot be a shared
/// borrow — the call through the field would not compile — and the front end carries the embedded
/// The source's FAILURE CONVENTION becomes the target's return type.
///
/// This is the mapping that blocks every real package, and it is not one construct — it is a
/// convention. The source returns failure as an extra result that nothing requires a caller to
/// check; the target says it in the return type, where the compiler requires it. So this is one of
/// the few translations that makes the ported program STRICTER than the original rather than merely
/// The propagation idiom becomes an OPERATOR, which is the whole point of recognising it.
///
/// `n, err := f()` followed by `if err != nil { return 0, err }` is two statements a caller could
/// simply not have written. `f()?` is one expression on a value that cannot be used without
/// addressing the failure — so the translation moves the check from discipline into the type
/// A call the target has no name for is answered by the pack, by the callee's IDENTITY.
///
/// No real package ports without this: every one calls its standard library, and the standard
/// Every method in the corpus carries a TRANSLATED body.
///
/// A stub compiles, matches a golden, and hashes into a stable receipt — so every other check in
/// this file passes over a crate whose methods all abort at the first call. This is the one that
/// notices. It is asserted over the whole emit rather than per declaration on purpose: a stub
/// reintroduced anywhere reds it, including in a declaration nobody thought to name here.
#[test]
fn no_method_body_is_a_stub() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    let source = driver::assemble_modules(&report);

    assert!(
        !source.contains("todo!"),
        "every body in the corpus must translate:\n{source}"
    );
}

/// Determinism, over the real corpus rather than over fakes: two runs produce identical bytes and
/// identical receipts, and the kernel classifies the pair as `Unchanged`/Green.
#[test]
fn porting_twice_is_byte_identical() {
    let verification = driver::port_go_delta().expect("two runs must be identical");
    assert_eq!(verification.verdict, port_engine_kernel::Verdict::Green);
}

/// A planted defect must be RED, and the reason matters: the two receipts are IDENTICAL, so no
/// axis moved, so nothing explains the changed bytes. `Unexplained` is exactly the verdict
/// ADR-0637 D2 assigns that situation, and it is the property that makes the receipt worth having
/// — determinism that cannot detect a change is not determinism, it is a constant.
#[test]
fn a_planted_defect_in_the_ported_corpus_is_unexplained() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");

    let mut defective = report.emitted.clone();
    let target = defective
        .keys()
        .next()
        .cloned()
        .expect("the port emits at least one region");
    defective.insert(target.clone(), b"pub fn planted_defect() {}".to_vec());

    let verification = port_engine_kernel::verify(
        &report.receipt,
        &report.emitted,
        &report.receipt,
        &defective,
    );

    assert_eq!(verification.verdict, port_engine_kernel::Verdict::Red);
    match verification.delta {
        port_engine_kernel::Delta::Unexplained { regions } => {
            assert!(
                regions.contains(&target),
                "the changed region must be named: {regions:?}"
            );
        }
        other => panic!("a defect with no moved axis must be Unexplained, got {other:?}"),
    }
}

/// The refusal path, exercised against REAL Go rather than synthetic nodes.
///
/// `corpus-refused/` holds a `for` loop and a `defer`, neither of which has a translation yet, and
/// both of which the extractor records faithfully as `unsupported` rather than dropping. A dropped
/// construct would make an untranslatable function indistinguishable from an empty one and the
/// engine would emit a green, silently wrong body; recorded, it becomes a refusal that names the
/// construct and points at the census entry where the analysis belongs.
///
/// A translator whose refusals are only ever tested on hand-built inputs has not been shown to
/// The second refusal class, in its own corpus so it is PROVEN rather than shadowed by whichever
/// package the transform reached first.
///
/// A method whose receiver outlives the call cannot be handed out as any borrow of `self` — a
/// reference would need a lifetime the caller cannot supply — so the pack's escaping disposition
/// declares no receiver form and the transform refuses rather than picking a borrow that will not
/// The six receipt axes carry real values for the first time. Before this lane every axis was
/// typed and compared but never populated over a corpus, so the determinism claim held only over
/// in-memory fakes.
#[test]
fn every_receipt_axis_carries_a_value() {
    let report = driver::port_go_pipeline().expect("the Go corpus must port");
    assert!(
        report.receipt.incomplete_axes().is_empty(),
        "receipt axes left empty: {:?}",
        report.receipt.incomplete_axes()
    );
    assert!(report.receipt.snapshot_digest.0.starts_with("sha256:"));
    assert!(report.receipt.rulepack_digest.0.starts_with("sha256:"));
    assert!(report.receipt.engine_digest.0.starts_with("sha256:"));
    assert!(!report.receipt.pin.is_empty());
}
