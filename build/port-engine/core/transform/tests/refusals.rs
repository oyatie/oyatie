//! Everything the transform refuses, and why each refusal exists.

#![allow(dead_code)]

mod common;

use common::*;
use std::collections::BTreeMap;

use port_engine_api::{Declaration, TargetIr, UnitId};
use port_engine_transform::*;

/// The coverage rule. Without it a declaration nothing captures is dropped in silence and the
/// emit is green over a corpus it did not translate.
#[test]
fn refuses_a_declaration_no_rule_captures() {
    let pack = Pack::default().with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"]);
    let model = model_with(vec![decl("var", "Enabled", "bool")]);
    let err = apply(&plan_with(&["consts"]), &pack, &model).expect_err("uncaptured refuses");
    assert!(matches!(
        err,
        TransformError::UncapturedDeclaration { ref kind, .. } if kind == "var"
    ));
}

/// Never guess a type. A passed-through source spelling either fails to compile far from its
/// cause or, worse, resolves to an unrelated target type with the same name.
#[test]
fn refuses_a_type_the_pack_does_not_map() {
    let pack = Pack::default().with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"]);
    let mut value = decl("const", "K", "uintptr");
    value.attrs.insert("value".into(), "0".into());
    let err =
        apply(&plan_with(&["consts"]), &pack, &model_with(vec![value])).expect_err("unmapped");
    assert!(matches!(
        err,
        TransformError::UnmappedType { ref type_ref, .. } if type_ref == "uintptr"
    ));
}

#[test]
fn refuses_a_pointer_receiver_rather_than_guessing_aliasing() {
    let mut point = decl("struct", "Point", "");
    let mut method = child("method", "Move", "");
    method.flags.insert(FLAG_POINTER_RECEIVER.to_owned());
    point.children = vec![method];
    let pack = Pack::default().with_rule("structs", CONSTRUCTION_RUST_STRUCT, &["struct"]);
    let err = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point]))
        .expect_err("pointer receiver refuses");
    assert!(matches!(err, TransformError::Unsupported { .. }));
}

#[test]
fn refuses_a_variadic_signature() {
    let mut printf = decl("func", "Printf", "");
    printf.flags.insert(FLAG_VARIADIC.to_owned());
    let pack = Pack::default().with_rule("funcs", CONSTRUCTION_RUST_FN, &["func"]);
    let err = apply(&plan_with(&["funcs"]), &pack, &model_with(vec![printf]))
        .expect_err("variadic refuses");
    assert!(matches!(err, TransformError::Unsupported { .. }));
}

#[test]
fn refuses_unknown_construction() {
    let pack = Pack::default().with_rule("bad", "not_a_construction", &[]);
    let err = apply(&plan_with(&["bad"]), &pack, &model_with(Vec::new()))
        .expect_err("unknown construction");
    assert!(matches!(err, TransformError::UnknownConstruction { .. }));
}

#[test]
fn refuses_missing_unit_precondition() {
    let pack = Pack::default().with_rule("r", CONSTRUCTION_PASS_THROUGH, &[]);
    let model = Model {
        units: Vec::new(),
        declarations: BTreeMap::new(),
    };
    let err = apply(&plan_with(&["r"]), &pack, &model).expect_err("unit missing");
    assert!(matches!(err, TransformError::Precondition { .. }));
}
