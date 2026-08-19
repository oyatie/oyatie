//! Loading a well-formed pack: the fixture gate, the digest, and the seams it answers.

#![allow(dead_code)]

mod common;

use common::*;
use std::collections::BTreeMap;

use port_engine_api::{PackSemantics, RuleId, RulePack, UnitId};
use port_engine_hash::digest_bytes;
use port_engine_rulepack::{LoadedRule, LoadedRulePack, w0_ready};

#[test]
fn slice10_claims_fixture_gated_readiness() {
    assert!(w0_ready());
}

#[test]
fn embedded_v0_loads_with_fixtures_and_digests_bytes() {
    let pack = LoadedRulePack::load_embedded().expect("embedded v0 must load");
    assert_eq!(pack.pair().source, "go");
    assert_eq!(pack.pair().target, "rust");
    assert_eq!(
        pack.digest(),
        digest_bytes(include_str!("../src/rulepack-v0.json").as_bytes()),
        "digest must be SHA-256 of embedded JSON bytes"
    );
    assert_eq!(
        pack.rules(),
        vec![
            RuleId("identity".into()),
            RuleId("canary_empty_unit".into())
        ]
    );
    assert_eq!(pack.selecting_fixture_count(), 2);
    for rule in pack.loaded_rules() {
        assert!(
            rule.selecting_fixtures
                .iter()
                .any(|fixture| fixture.selects),
            "every loaded rule must retain a positive selecting fixture"
        );
    }
    assert_eq!(
        pack.rules_for(&UnitId("example.com/b".into())),
        vec![
            RuleId("identity".into()),
            RuleId("canary_empty_unit".into())
        ]
    );
}

#[test]
fn embedded_v0_plans_with_kernel() {
    let pack = LoadedRulePack::load_embedded().expect("load");
    let model = TinyModel {
        units: vec![
            UnitId("example.com/a".into()),
            UnitId("example.com/b".into()),
        ],
    };
    let plan = port_engine_kernel::plan(&model, &pack).expect("plan must succeed");
    assert_eq!(plan.steps.len(), 3);
    assert_eq!(plan.pair.source, "go");
}

#[test]
fn embedded_go_rust_pack_loads_with_captures_types_and_deferrals() {
    let pack = LoadedRulePack::load_embedded_go_rust().expect("go→rust pack must load");
    assert_eq!(pack.language_pair().source, "go");
    assert_eq!(pack.language_pair().target, "rust");

    let by_id: BTreeMap<&str, &LoadedRule> = pack
        .loaded_rules()
        .iter()
        .map(|rule| (rule.id.0.as_str(), rule))
        .collect();
    assert_eq!(by_id["go_struct"].captures, vec!["struct".to_owned()]);
    assert_eq!(by_id["go_func"].construction, "rust_fn_body");

    assert_eq!(pack.type_map().get("int").map(String::as_str), Some("i64"));
    assert_eq!(
        pack.type_map_overrides("rust_const")
            .and_then(|map| map.get("string"))
            .map(String::as_str),
        Some("&str"),
        "a Go string constant is a borrowed str, not an owned String; the lifetime is elided \
         because a const's reference is 'static by definition and spelling it draws a lint"
    );

    // Every deferral, by KIND rather than by count. A count has to be edited whenever one lands,
    // which makes the edit routine and the check ceremonial; the set says which one went missing.
    let deferred = pack.deferred();
    let kinds: std::collections::BTreeSet<&str> =
        deferred.iter().map(|entry| entry.kind.as_str()).collect();
    assert_eq!(
        kinds,
        std::collections::BTreeSet::from(["foreign_satisfaction", "package_init"]),
        "`var` is no longer among them: the pack now captures it, and only the SHAPE that stays \
         undecided — a package variable something writes — is recorded, as an undecided form"
    );
    // An undecided FORM is a shape within a kind, and it carries a reason for the same reason a
    // deferral does: the engine quotes it when it declines, so the words a reader sees and the
    // words the digest holds are one text.
    let forms: std::collections::BTreeSet<&str> =
        pack.undecided_forms().keys().map(String::as_str).collect();
    assert_eq!(
        forms,
        std::collections::BTreeSet::from(["init_written_package_var", "written_package_var"]),
    );
    for reason in pack.undecided_forms().values() {
        assert!(
            reason.len() > 40,
            "an undecided form's reason is the record; it must say something"
        );
    }

    for entry in deferred {
        assert!(
            entry.reason.len() > 40,
            "a deferral's reason is the record; `{}` must say something",
            entry.kind
        );
    }
}

#[test]
fn accepts_agreeing_negative_fixture_without_counting_it_as_selection() {
    let json = r#"{
  "pair": {"source": "go", "target": "rust"},
  "rules": [{
"id": "conditional",
"version": "0",
"precondition": "unit_present",
"construction": "pass_through",
"selecting_fixtures": [
  {"id": "positive", "unit": "u", "selects": true},
  {"id": "negative", "unit": "v", "selects": false}
]
  }],
  "applies": {"u": ["conditional"]}
}"#;
    let pack = LoadedRulePack::load_from_str(json)
        .expect("an agreeing negative fixture must remain admissible");
    assert_eq!(pack.selecting_fixture_count(), 1);
    assert_eq!(pack.loaded_rules()[0].selecting_fixtures.len(), 2);
}
