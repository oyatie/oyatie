//! What each construction emits, and how identifiers are cased.

#![allow(dead_code)]

mod common;

use common::*;
use std::collections::BTreeMap;

use port_engine_api::{Declaration, TargetIr, UnitId};
use port_engine_transform::*;

#[test]
fn claims_transform_readiness() {
    assert!(w0_ready());
}

#[test]
fn unit_level_rule_still_emits_one_region_per_unit() {
    let pack = Pack::default().with_rule("canary_empty_unit", CONSTRUCTION_EMPTY_CANARY, &[]);
    let model = model_with(Vec::new());
    let ir = apply(&plan_with(&["canary_empty_unit"]), &pack, &model).expect("apply");
    assert_eq!(ir.regions().len(), 1);
    assert_eq!(ir.regions()[0].0, "u__canary_empty_unit");
}

#[test]
fn constant_carries_its_type_and_value() {
    let mut max = decl("const", "MaxRetries", "int");
    max.attrs.insert("value".into(), "3".into());
    let pack = Pack::default()
        .with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"])
        .with_types(&[("int", "i64")]);
    let ir = apply(&plan_with(&["consts"]), &pack, &model_with(vec![max])).expect("apply");
    let text = rendered(&ir);
    assert!(text.contains("MAX_RETRIES"), "{text}");
    assert!(text.contains("i64"), "{text}");
    assert!(text.contains("3"), "{text}");
}

#[test]
fn function_renders_named_params_and_a_result() {
    let mut add = decl("func", "Add", "");
    add.children = vec![
        child("param", "a", "int"),
        child("param", "b", "int"),
        child("result", "", "int"),
    ];
    // A BODY, because a function without one is now a refusal rather than a stub: emitting a
    // signature whose body panics compiles and aborts at the caller. The shape this test is about
    // is the signature, and the body is what makes it emittable at all.
    add.children.push(child("body", "", ""));
    let pack = Pack::default()
        .with_rule("funcs", CONSTRUCTION_RUST_FN_BODY, &["func"])
        .with_types(&[("int", "i64")]);
    let ir = apply(&plan_with(&["funcs"]), &pack, &model_with(vec![add])).expect("apply");
    let text = rendered(&ir);
    assert!(text.contains("fn add"), "{text}");
    assert!(
        text.contains("a : i64") || text.contains("a: i64"),
        "{text}"
    );
    assert!(
        text.contains("-> i64") || text.contains("- > i64"),
        "{text}"
    );
}

#[test]
fn struct_renders_fields_and_an_inherent_impl() {
    let mut point = decl("struct", "Point", "");
    let mut x = child("field", "X", "int");
    x.flags.insert("exported".into());
    let mut shift = child("method", "Shift", "");
    shift.flags.insert("exported".into());
    shift.children = vec![child("param", "dx", "int"), child("result", "", "Point")];
    // Same reason as the function above: a method the rung will not translate is a refusal now.
    shift.children.push(child("body", "", ""));
    point.children = vec![x, child("field", "label", "string"), shift];

    let pack = Pack::default()
        .with_rule("structs", CONSTRUCTION_RUST_STRUCT_BODY, &["struct"])
        .with_types(&[("int", "i64"), ("string", "String")]);
    let ir = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point])).expect("apply");
    let text = rendered(&ir);
    assert!(text.contains("struct Point"), "{text}");
    assert!(text.contains("pub x"), "{text}");
    assert!(text.contains("impl Point"), "{text}");
    assert!(text.contains("fn shift"), "{text}");
    // The unexported field must not become part of the public surface.
    assert!(!text.contains("pub label"), "{text}");
}

/// A defined type is a distinct type. Rendering it as an alias would erase the one property it
/// was declared for, and the emitted code would compile while meaning something weaker.
#[test]
fn defined_type_becomes_a_newtype_and_alias_stays_transparent() {
    let celsius = decl("named", "Celsius", "float64");
    let id = decl("alias", "ID", "string");
    let pack = Pack::default()
        .with_rule("named", CONSTRUCTION_RUST_NEWTYPE, &["named"])
        .with_rule("aliases", CONSTRUCTION_RUST_TYPE_ALIAS, &["alias"])
        .with_types(&[("float64", "f64"), ("string", "String")]);
    let ir = apply(
        &plan_with(&["named", "aliases"]),
        &pack,
        &model_with(vec![celsius, id]),
    )
    .expect("apply");
    let text = rendered(&ir);
    assert!(text.contains("struct Celsius"), "{text}");
    // `ID` becomes `Id`, reversing an earlier decision here that it should stay. The old reason
    // was that lowercasing "would rename the type rather than recase it" — which is wrong, because
    // `Id` and `ID` are the same word differently cased and casing is what this does. RFC 430 is
    // explicit: an acronym counts as ONE WORD in upper camel case, `Uuid` rather than `UUID`. Two
    // reviewers read the all-capitals form as the source language's convention carried over, which
    // is exactly what it was.
    assert!(text.contains("type Id = String"), "{text}");
}

/// A locally declared name must win over the pack's map, or a unit declaring a type whose name
/// collides with a mapped one silently emits the mapped type in its place.
#[test]
fn local_declaration_shadows_the_type_map() {
    let mut holder = decl("struct", "Holder", "");
    holder.children = vec![child("field", "Inner", "string")];
    let local = decl("struct", "string", "");
    // The map deliberately sends `string` somewhere the local declaration does not, so the two
    // candidate answers are distinguishable in the output.
    let pack = Pack::default()
        .with_rule("structs", CONSTRUCTION_RUST_STRUCT, &["struct"])
        .with_types(&[("string", "MappedElsewhere")]);
    let ir = apply(
        &plan_with(&["structs"]),
        &pack,
        &model_with(vec![holder, local]),
    )
    .expect("apply");
    let text = rendered(&ir);
    assert!(
        text.contains("String"),
        "the unit's own `string` must win over the mapped one: {text}"
    );
    assert!(
        !text.contains("MappedElsewhere"),
        "the pack's map must not shadow a type the unit declares: {text}"
    );
}

/// Deferral is a decision someone wrote down, and it travels in the pack digest. That is what
/// separates it from the same declaration merely going unselected.
#[test]
fn declared_deferral_admits_what_bare_omission_would_not() {
    let mut max = decl("const", "MaxRetries", "int");
    max.attrs.insert("value".into(), "3".into());
    let pack = Pack::default()
        .with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"])
        .with_types(&[("int", "i64")])
        .with_deferred(&["var"]);
    let model = model_with(vec![max, decl("var", "Enabled", "bool")]);
    let ir = apply(&plan_with(&["consts"]), &pack, &model).expect("deferred kind is accounted");
    assert_eq!(ir.regions().len(), 1, "the deferred var emits nothing");
}

#[test]
fn sanitize_ident_is_rust_safe() {
    assert_eq!(sanitize_ident("example.com/a"), "example_com_a");
    assert_eq!(sanitize_ident("9x"), "_9x");
}

#[test]
fn casing_keeps_capital_runs_together() {
    assert_eq!(to_snake_case("MaxRetries"), "max_retries");
    assert_eq!(to_snake_case("ParseURL"), "parse_url");
    assert_eq!(to_screaming_snake("MaxRetries"), "MAX_RETRIES");
    assert_eq!(to_pascal_case("point"), "Point");
    assert_eq!(to_pascal_case("Point"), "Point");
}

/// A source identifier that is a target KEYWORD is escaped, not refused. Every one of these is a
/// legal Go name, so a translator that cannot emit them cannot translate Go.
#[test]
fn a_keyword_identifier_is_escaped_rather_than_refused() {
    assert_eq!(to_snake_case("Move"), "r#move");
    assert_eq!(to_snake_case("Type"), "r#type");
    assert_eq!(to_snake_case("Loop"), "r#loop");
    // The four the grammar needs everywhere cannot be raw, so they are RENAMED — a real change to
    // the identifier, which is why they are handled separately.
    assert_eq!(escape_keyword("self"), "self_");
    assert_eq!(escape_keyword("crate"), "crate_");
    // And an ordinary name is untouched.
    assert_eq!(to_snake_case("Total"), "total");
}
