//! Unit tests for the corpus extractor: determinism (byte-identical facts for identical input),
//! the RED liveness test (a renamed/removed fn changes the fact set), reformatting invariance of
//! the signature anchor, and opaque-category classification.

use super::*;
use corpus_core::{ItemKind, Visibility};

/// Build a single-file source set for a synthetic crate.
fn one_file(source: &str) -> SourceSet {
    SourceSet::new([SourceFile {
        crate_id: "test-crate".to_owned(),
        path: "test-crate/src/lib.rs".to_owned(),
        module_path: String::new(),
        source: source.to_owned(),
    }])
}

fn extract(source: &str) -> CorpusExtraction {
    extract_corpus(&SynAstSource::new(), &one_file(source)).expect("infallible source")
}

#[test]
fn determinism_byte_identical_for_identical_input() {
    let src = r#"
        pub fn alpha(x: u32) -> u32 { x + 1 }
        pub struct Beta { pub n: u32 }
        fn gamma() {}
    "#;
    let a = extract(src).facts.canonical_json().unwrap();
    let b = extract(src).facts.canonical_json().unwrap();
    assert_eq!(a, b, "same input must yield byte-identical fact JSON");
}

#[test]
fn determinism_independent_of_file_order() {
    let f1 = SourceFile {
        crate_id: "c".to_owned(),
        path: "c/src/a.rs".to_owned(),
        module_path: "a".to_owned(),
        source: "pub fn one() {}".to_owned(),
    };
    let f2 = SourceFile {
        crate_id: "c".to_owned(),
        path: "c/src/b.rs".to_owned(),
        module_path: "b".to_owned(),
        source: "pub fn two() {}".to_owned(),
    };
    let forward = extract_corpus(
        &SynAstSource::new(),
        &SourceSet::new([f1.clone(), f2.clone()]),
    )
    .unwrap();
    let reverse = extract_corpus(&SynAstSource::new(), &SourceSet::new([f2, f1])).unwrap();
    assert_eq!(
        forward.facts.canonical_json().unwrap(),
        reverse.facts.canonical_json().unwrap()
    );
}

#[test]
fn red_test_rename_changes_fact_set() {
    let before = extract("pub fn evaluate() {}");
    let after = extract("pub fn evaluate_v2() {}");
    assert_ne!(
        before.facts.canonical_json().unwrap(),
        after.facts.canonical_json().unwrap(),
        "a renamed fn MUST change the fact set (liveness detects rename)"
    );
    // The signature anchor itself must differ (not just the fqpath text).
    let sig_before = &before.facts.facts()[0].signature_hash;
    let sig_after = &after.facts.facts()[0].signature_hash;
    assert_ne!(sig_before, sig_after);
}

#[test]
fn red_test_removal_changes_fact_set() {
    let before = extract("pub fn keep() {}\npub fn remove_me() {}");
    let after = extract("pub fn keep() {}");
    assert_eq!(before.facts.len(), 2);
    assert_eq!(after.facts.len(), 1);
    assert_ne!(
        before.facts.canonical_json().unwrap(),
        after.facts.canonical_json().unwrap(),
        "a removed fn MUST change the fact set (liveness detects removal)"
    );
}

#[test]
fn reformatting_does_not_churn_signature_hash() {
    let tight = "pub fn evaluate(flag:&Flag)->Decision{let x=1;x}";
    let loose = r#"
        // a leading comment that must not affect the signature anchor
        pub fn evaluate(
            flag: &Flag,   // an arg comment
        ) -> Decision {
            let x = 1;

            x
        }
    "#;
    let a = extract(tight);
    let b = extract(loose);
    assert_eq!(a.facts.len(), 1);
    assert_eq!(b.facts.len(), 1);
    assert_eq!(
        a.facts.facts()[0].signature_hash,
        b.facts.facts()[0].signature_hash,
        "whitespace/comment reformatting MUST NOT churn the signature anchor"
    );
}

#[test]
fn impl_anchor_invariant_under_reformatting() {
    // Two impls on the same type must stay distinct AND keep stable signature anchors when blank
    // lines shift their source positions (the regression the line-based anchor caused).
    let tight = "pub struct S;\nimpl S { pub fn a(&self) {} }\nimpl S { pub fn b(&self) {} }";
    let loose = "pub struct S;\n\n\nimpl S {\n\n pub fn a(&self) {}\n}\n\n\n\nimpl S {\n\n pub fn b(&self) {}\n}";
    let a = extract(tight);
    let b = extract(loose);
    let sig_set = |e: &CorpusExtraction| -> Vec<String> {
        let mut v: Vec<String> = e
            .facts
            .facts()
            .iter()
            .filter(|f| f.item_kind == ItemKind::Impl)
            .map(|f| f.signature_hash.to_string())
            .collect();
        v.sort();
        v
    };
    // Two distinct impl facts in each.
    assert_eq!(sig_set(&a).len(), 2);
    // Identical impl signature-hash set across reformatting (no line-number churn).
    assert_eq!(
        sig_set(&a),
        sig_set(&b),
        "impl anchors must not churn on reformat"
    );
}

#[test]
fn body_hash_may_change_when_body_changes() {
    let a = extract("pub fn f() { let x = 1; }");
    let b = extract("pub fn f() { let x = 2; }");
    assert_eq!(
        a.facts.facts()[0].signature_hash,
        b.facts.facts()[0].signature_hash,
        "signature stable across body edit"
    );
    assert_ne!(
        a.facts.facts()[0].body_hash,
        b.facts.facts()[0].body_hash,
        "body hash legitimately churns on a body edit"
    );
}

#[test]
fn classifies_item_kinds() {
    let src = r#"
        pub fn free_fn() {}
        pub struct AStruct;
        pub enum AnEnum { X }
        pub trait ATrait {}
        impl AStruct { pub fn method(&self) {} }
        pub const A_CONST: u32 = 1;
        #[get("/health")]
        pub fn health() {}
    "#;
    let kinds: Vec<ItemKind> = extract(src)
        .facts
        .facts()
        .iter()
        .map(|f| f.item_kind)
        .collect();
    assert!(kinds.contains(&ItemKind::Function));
    assert!(kinds.contains(&ItemKind::Type));
    assert!(kinds.contains(&ItemKind::Impl));
    assert!(kinds.contains(&ItemKind::PubItem));
    assert!(kinds.contains(&ItemKind::Route), "route attr → Route kind");
}

#[test]
fn opaque_macro_generated_counted() {
    // An item-position macro invocation is opaque (generated items invisible to source-level syn).
    let src = "tonic::include_proto!(\"pkg\");";
    let e = extract(src);
    assert_eq!(e.facts.len(), 0);
    assert_eq!(e.report.opaque.len(), 1);
    assert_eq!(
        e.report.by_category.get("macro_generated").copied(),
        Some(1)
    );
}

#[test]
fn opaque_cfg_gated_counted() {
    let src = r#"
        pub fn always() {}
        #[cfg(feature = "x")]
        pub fn maybe() {}
    "#;
    let e = extract(src);
    // `always` is a clean fact; `maybe` is opaque (cfg-gated).
    assert_eq!(e.report.clean_facts, 1);
    assert_eq!(e.report.by_category.get("cfg_gated").copied(), Some(1));
}

#[test]
fn opaque_parse_error_counted_not_fatal() {
    let e = extract("pub fn broken( {{{ this is not rust");
    assert_eq!(e.report.clean_facts, 0);
    assert_eq!(e.report.by_category.get("parse_error").copied(), Some(1));
}

#[test]
fn opaque_rate_bps_computed() {
    // 1 clean + 1 opaque = 50% = 5000 bps.
    let src = r#"
        pub fn clean() {}
        #[cfg(test)]
        pub fn gated() {}
    "#;
    let e = extract(src);
    assert_eq!(e.report.total_units(), 2);
    assert_eq!(e.report.opaque_rate_bps(), 5000);
}

#[test]
fn nested_module_path_in_fqpath() {
    let src = r#"
        pub mod inner {
            pub fn deep() {}
        }
    "#;
    let e = extract(src);
    let fq: Vec<&str> = e.facts.facts().iter().map(|f| f.fqpath.as_str()).collect();
    assert!(fq.contains(&"inner::deep"), "got {fq:?}");
}

#[test]
fn visibility_normalized() {
    let src = r#"
        pub fn p() {}
        pub(crate) fn c() {}
        fn priv_fn() {}
    "#;
    let e = extract(src);
    let by_path = |p: &str| -> Visibility {
        e.facts
            .facts()
            .iter()
            .find(|f| f.fqpath == p)
            .map(|f| f.visibility)
            .expect("fact present")
    };
    assert_eq!(by_path("p"), Visibility::Public);
    assert_eq!(by_path("c"), Visibility::Crate);
    assert_eq!(by_path("priv_fn"), Visibility::Private);
}

#[test]
fn module_path_for_conventional_files() {
    assert_eq!(
        module_path_for("flags/core/x", "flags/core/x/src/lib.rs"),
        ""
    );
    assert_eq!(
        module_path_for("flags/core/x", "flags/core/x/src/main.rs"),
        ""
    );
    assert_eq!(
        module_path_for("flags/core/x", "flags/core/x/src/engine.rs"),
        "engine"
    );
    assert_eq!(
        module_path_for("flags/core/x", "flags/core/x/src/a/mod.rs"),
        "a"
    );
    assert_eq!(
        module_path_for("flags/core/x", "flags/core/x/src/a/b.rs"),
        "a::b"
    );
}

// MEDIUM-1 TEST: the impl fact's signature_hash must vary with the impl body (disambiguator is
// included in the signature pre-image). Without this, two distinct impls whose 32-bit body-hash
// prefix collides would produce byte-identical impl facts and be silently deduped rather than
// caught as AddressCollision. With the disambiguator in the sig pre-image, impls with different
// body content always produce different signature_hashes — the claim in A1 is literally true.
#[test]
fn impl_signature_hash_varies_with_body_content() {
    // Two `impl Foo` blocks in the SAME file; different method bodies → different disambiguators
    // → different fqpaths AND different signature_hashes (disambig is in the sig pre-image).
    let src = "pub struct Foo;\
               impl Foo { pub fn a(&self) -> u32 { 1 } }\
               impl Foo { pub fn b(&self) -> u32 { 2 } }";
    let e = extract(src);
    let impl_facts: Vec<_> = e
        .facts
        .facts()
        .iter()
        .filter(|f| f.item_kind == corpus_core::ItemKind::Impl)
        .collect();
    assert_eq!(
        impl_facts.len(),
        2,
        "two impl blocks must produce two impl facts"
    );
    assert_ne!(
        impl_facts[0].signature_hash, impl_facts[1].signature_hash,
        "impls with different bodies MUST have different signature_hashes (disambig in sig pre-image)"
    );
}

// HIGH-1 RED TEST: two `impl Foo` blocks in SEPARATE FILES of the same crate (same module_path "")
// must produce two DISTINCT facts — not the same fqpath (which would cause silent dedup).
//
// The HIGH-1 defect: `WalkState` was reset per `extract_file` call, so both impls got ordinal 0
// → identical fqpath `Foo#impl[0]` → identical signature_hash → FactSet::from_facts silently
// dropped one. Fix: use a content-hash disambiguator (impl body tokens) that is per-impl, not
// per-file-position.
#[test]
fn cross_file_impls_same_type_produce_distinct_facts() {
    // lib.rs: `impl Foo` with method `a`
    let lib_rs = SourceFile {
        crate_id: "test-crate".to_owned(),
        path: "test-crate/src/lib.rs".to_owned(),
        module_path: String::new(), // crate root
        source: "pub struct Foo; impl Foo { pub fn a(&self) -> u32 { 1 } }".to_owned(),
    };
    // main.rs: another `impl Foo` with method `b` — same crate, same module_path, different body
    let main_rs = SourceFile {
        crate_id: "test-crate".to_owned(),
        path: "test-crate/src/main.rs".to_owned(),
        module_path: String::new(), // also crate root
        source: "impl Foo { pub fn b(&self) -> u32 { 2 } }".to_owned(),
    };

    let set = SourceSet::new([lib_rs, main_rs]);
    let result = extract_corpus(&SynAstSource::new(), &set).unwrap();

    // Collect the impl facts.
    let impl_facts: Vec<_> = result
        .facts
        .facts()
        .iter()
        .filter(|f| f.item_kind == corpus_core::ItemKind::Impl)
        .collect();

    assert_eq!(
        impl_facts.len(),
        2,
        "two `impl Foo` blocks across two files MUST produce 2 distinct impl facts, got {}: {:?}",
        impl_facts.len(),
        impl_facts.iter().map(|f| &f.fqpath).collect::<Vec<_>>()
    );

    // The two impl facts must have different fqpaths (different disambiguators).
    assert_ne!(
        impl_facts[0].fqpath, impl_facts[1].fqpath,
        "cross-file `impl Foo` blocks must have distinct fqpaths"
    );

    // Methods from both impls must be present (4 total: struct Foo + 2 impls + 2 methods).
    let method_fqpaths: Vec<_> = result
        .facts
        .facts()
        .iter()
        .filter(|f| f.item_kind == corpus_core::ItemKind::Function)
        .map(|f| f.fqpath.as_str())
        .collect();
    assert!(
        method_fqpaths.contains(&"Foo::a"),
        "method `a` from lib.rs impl must be present"
    );
    assert!(
        method_fqpaths.contains(&"Foo::b"),
        "method `b` from main.rs impl must be present"
    );
}

// MEDIUM-a TEST: `extern "C" { … }` (ForeignMod) must be counted as opaque (Unhandled),
// not silently dropped. Same for `trait Alias = Bound;` (TraitAlias).
#[test]
fn foreign_mod_and_trait_alias_are_unhandled_opaque() {
    // ForeignMod: extern "C" block — neither a fact nor a silent drop; must be Unhandled.
    let foreign_src = r#"
        extern "C" {
            pub fn c_fn(x: i32) -> i32;
        }
    "#;
    let e = extract(foreign_src);
    assert_eq!(e.facts.len(), 0, "ForeignMod must not produce a clean fact");
    assert_eq!(
        e.report.by_category.get("unhandled").copied(),
        Some(1),
        "ForeignMod must be counted as unhandled opaque"
    );

    // TraitAlias: `trait Foo = Bar;` — must be Unhandled.
    let alias_src = "trait MyAlias = Clone + Send;";
    let e2 = extract(alias_src);
    assert_eq!(
        e2.facts.len(),
        0,
        "TraitAlias must not produce a clean fact"
    );
    assert_eq!(
        e2.report.by_category.get("unhandled").copied(),
        Some(1),
        "TraitAlias must be counted as unhandled opaque"
    );
}

#[test]
fn idl_extractor_family_plan_names_proto_openapi_cedar_sql() {
    let ids: Vec<&str> = IDL_EXTRACTOR_FAMILY_PLAN
        .iter()
        .map(|entry| entry.extractor_id)
        .collect();
    assert_eq!(ids, ["proto", "openapi", "cedar", "sql"]);
    assert_eq!(
        IDL_EXTRACTOR_FAMILY_PLAN[0].status,
        "first-slice-fixture-landed"
    );
    assert!(
        IDL_EXTRACTOR_FAMILY_PLAN[0]
            .measured_blind_spot
            .contains("45% aggregate true-miss")
    );
}

#[test]
fn proto_idl_fixture_closes_include_proto_true_miss() {
    // ADR-0580 A6 measured a 45% aggregate true-miss on proto-heavy crates: `syn` sees only the
    // `tonic::include_proto!` macro invocation, not the generated message/service/RPC surface.
    let rust = extract("tonic::include_proto!(\"oya.messenger.v1\");");
    assert_eq!(rust.facts.len(), 0);
    assert_eq!(
        rust.report.by_category.get("macro_generated").copied(),
        Some(1)
    );

    let proto = r#"
        syntax = "proto3";
        package oya.messenger.v1;

        message SendRequest {
          string body = 1;
        }

        message SendResponse {
          string id = 1;
        }

        service MessengerService {
          rpc SendMessage (SendRequest) returns (SendResponse);
        }
    "#;

    let set = SourceSet::new([SourceFile {
        crate_id: "messenger-proto-contracts".to_owned(),
        path: "comms/messenger/contracts/messenger.proto".to_owned(),
        module_path: "oya.messenger.v1".to_owned(),
        source: proto.to_owned(),
    }]);
    let extracted = extract_corpus(&ProtoIdlAstSource::new(), &set).unwrap();
    assert_eq!(extracted.report.opaque.len(), 0);

    let facts: Vec<_> = extracted
        .facts
        .facts()
        .iter()
        .map(|fact| (fact.fqpath.as_str(), fact.item_kind))
        .collect();
    assert!(facts.contains(&("oya.messenger.v1::SendRequest", ItemKind::Type)));
    assert!(facts.contains(&("oya.messenger.v1::SendResponse", ItemKind::Type)));
    assert!(facts.contains(&("oya.messenger.v1::MessengerService", ItemKind::Type)));
    assert!(facts.contains(&(
        "oya.messenger.v1::MessengerService::SendMessage",
        ItemKind::Function
    )));
}

// ── edges ────────────────────────────────────────────────────────────────────────────────────

use corpus_core::{CoveragePolicy, EdgeKind, NodeId, NodeKind, evaluate_coverage};

fn refs_targets(extraction: &CorpusExtraction) -> Vec<String> {
    extraction
        .graph
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Refs)
        .map(|edge| edge.dst.path.clone())
        .collect()
}

// The derived spine: every clean fact gets an Entry node, and the file that produced it CONTAINS
// it. Without this, a fact is in the fact set but unreachable in the graph.
#[test]
fn every_fact_gets_a_node_and_a_containing_file_edge() {
    let extracted = extract("pub fn alpha() {} pub struct Beta;");
    assert!(!extracted.facts.is_empty(), "precondition: facts exist");
    for fact in extracted.facts.facts() {
        let id = NodeId::entry(&fact.crate_id, &fact.fqpath);
        assert!(
            extracted.graph.contains_node(&id),
            "fact {} has no Entry node",
            fact.fqpath
        );
        assert!(
            extracted.graph.edges().iter().any(|edge| {
                edge.kind == EdgeKind::Contains && edge.src.kind == NodeKind::File && edge.dst == id
            }),
            "fact {} is contained by no file",
            fact.fqpath
        );
    }
}

// Contains, the second level: an impl block contains its methods. This is the only parent/child
// relation where both ends are facts, and it is what lets a reachability BFS descend from a type.
#[test]
fn impl_contains_its_methods() {
    let extracted = extract("pub struct Foo; impl Foo { pub fn method(&self) {} }");
    let impl_fqpath = extracted
        .facts
        .facts()
        .iter()
        .find(|fact| fact.item_kind == ItemKind::Impl)
        .map(|fact| fact.fqpath.clone())
        .expect("an impl fact");
    assert!(
        extracted.graph.edges().iter().any(|edge| {
            edge.kind == EdgeKind::Contains
                && edge.src == NodeId::entry("test-crate", &impl_fqpath)
                && edge.dst == NodeId::entry("test-crate", "Foo::method")
        }),
        "impl→method Contains edge missing: {:#?}",
        extracted.graph.edges()
    );
}

// Refs: a call names its target, an impl names its trait and self type, a signature names its
// types. A method CALL (`x.foo()`) is deliberately absent — resolving it needs the receiver type,
// so there is no honest name to emit.
#[test]
fn refs_edges_name_calls_types_and_traits() {
    let extracted = extract(
        "pub trait Speak {} pub struct Foo; \
         impl Speak for Foo {} \
         pub fn caller() -> Foo { helper(); Foo } \
         pub fn helper() {}",
    );
    let targets = refs_targets(&extracted);
    assert!(targets.contains(&"helper".to_owned()), "call: {targets:?}");
    assert!(targets.contains(&"Foo".to_owned()), "type: {targets:?}");
    assert!(targets.contains(&"Speak".to_owned()), "trait: {targets:?}");
    assert!(
        !targets.iter().any(|target| target == "self"),
        "a receiver is not a reference: {targets:?}"
    );
}

// KNOWN-POSITIVE CONTROL, end to end: every name this crate references is defined in this crate,
// so the COMPUTED coverage is total and nothing dangles.
#[test]
fn coverage_positive_control_self_contained_crate() {
    let extracted = extract("pub fn caller() { helper(); } pub fn helper() {}");
    let coverage = extracted.graph.coverage();
    assert_eq!(
        extracted.graph.unresolved_targets(),
        Vec::new(),
        "targets: {:?}",
        refs_targets(&extracted)
    );
    assert_eq!(coverage.total_targets, 1);
    assert_eq!(coverage.rate_bps(), 10_000);
}

// KNOWN-NEGATIVE CONTROL, end to end: the SAME source with the callee deleted. The reference
// survives as a dangling NAME, coverage drops, and the ratchet names the missing target. A
// fact-pointer `dst` would have made this state unrepresentable.
#[test]
fn coverage_negative_control_deleted_callee_dangles() {
    let extracted = extract("pub fn caller() { helper(); }");
    let coverage = extracted.graph.coverage();
    assert_eq!(coverage.total_targets, 1);
    assert_eq!(coverage.indexed_targets, 0);
    assert_eq!(coverage.rate_bps(), 0);
    assert_eq!(
        extracted.graph.unresolved_targets(),
        vec![NodeId::entry("test-crate", "helper")]
    );

    let findings = evaluate_coverage(
        &extracted.graph,
        &CoveragePolicy {
            baseline_unindexed_targets: 0,
            min_expected_targets: 1,
        },
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.blocking && finding.detail.contains("helper")),
        "the ratchet must name the dangling target: {findings:?}"
    );
}

// Duplication falls out of the shallow file digest for free — the property that matters for the
// markdown families sitting at 179 byte-identical copies each.
#[test]
fn byte_identical_files_share_one_digest_across_distinct_nodes() {
    let body = "pub fn same() {}";
    let set = SourceSet::new((0..3).map(|index| SourceFile {
        crate_id: format!("crate-{index}"),
        path: format!("cap-{index}/src/lib.rs"),
        module_path: String::new(),
        source: body.to_owned(),
    }));
    let extracted = extract_corpus(&SynAstSource::new(), &set).unwrap();
    let families = extracted.graph.duplicate_containers();
    assert_eq!(families.len(), 1);
    assert_eq!(
        families[0].1,
        vec![
            "cap-0/src/lib.rs".to_owned(),
            "cap-1/src/lib.rs".to_owned(),
            "cap-2/src/lib.rs".to_owned()
        ]
    );
}

// Edges must be as deterministic as the facts, or the graph is not a build output.
#[test]
fn graph_is_byte_identical_for_identical_input() {
    let src =
        "pub struct Foo; impl Foo { pub fn a(&self) -> Foo { helper(); Foo } } pub fn helper() {}";
    assert_eq!(extract(src).graph, extract(src).graph);
}
