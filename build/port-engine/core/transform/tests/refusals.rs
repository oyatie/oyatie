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
    // The refusal names the type as the model DESCRIBES it, not as the source spelled it — the
    // description carries the kind and the package, which is what a reader needs to add the
    // missing pack entry.
    assert!(
        matches!(err, TransformError::UnmappedType { ref type_ref, .. } if type_ref.contains("uintptr")),
        "{err}"
    );
}
/// A pointer receiver used to be refused outright, because nothing reported whether the body
/// mutated through it. The front end reports that now, so the guess became a decision — and a
/// pack that has declared no rule for the observed facts still gets a refusal, because an invented
/// default is a decision nobody wrote down.
#[test]
fn a_pointer_receiver_without_a_matching_rule_refuses() {
    let mut point = decl("struct", "Point", "");
    let mut method = child("method", "Move", "");
    method.flags.insert(FLAG_POINTER_RECEIVER.to_owned());
    method.flags.insert("mutated".to_owned());
    point.children = vec![method];

    let pack = Pack::default().with_rule("structs", CONSTRUCTION_RUST_STRUCT_BODY, &["struct"]);
    let err = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point]))
        .expect_err("no declared rule accepts these facts");
    assert!(matches!(err, TransformError::Ownership { .. }), "{err}");
}

/// And with a rule, the facts decide. A mutating receiver becomes an exclusive borrow — which is
/// a HYPOTHESIS about aliasing rather than a proof, and is safe to emit only because an
/// unsatisfiable `&mut` is a borrow-check error the compile proof catches.
#[test]
fn observed_mutation_reaches_an_exclusive_receiver() {
    let mut point = decl("struct", "Point", "");
    let mut mutating = child("method", "Move", "");
    mutating.flags.insert(FLAG_POINTER_RECEIVER.to_owned());
    mutating.flags.insert("mutated".to_owned());
    let mut reading = child("method", "Peek", "");
    reading.flags.insert(FLAG_POINTER_RECEIVER.to_owned());
    // A BODY each, because a rung that translates them needs one: a method without a body is a
    // refusal rather than a stub, and this test is about the RECEIVER.
    mutating.children.push(child("body", "", ""));
    reading.children.push(child("body", "", ""));
    point.children = vec![mutating, reading];

    let pack = Pack::default()
        .with_rule("structs", CONSTRUCTION_RUST_STRUCT_BODY, &["struct"])
        .with_disposition(
            "exclusive",
            Some(true),
            Some(false),
            "&mut {0}",
            Some("&mut self"),
        )
        .with_disposition("shared", Some(false), Some(false), "&{0}", Some("&self"));

    let ir = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point])).expect("apply");
    let text = rendered(&ir);
    assert!(
        text.contains("fn r#move(&mut self)") || text.contains("fn move(&mut self)"),
        "{text}"
    );
    assert!(text.contains("fn peek(&self)"), "{text}");
}

/// A disposition with no receiver form is a REFUSAL for the receiver position, not a fallback: a
/// pointer that outlives the call cannot be handed out as any borrow of `self`.
#[test]
fn a_disposition_without_a_receiver_form_refuses_the_receiver() {
    let mut point = decl("struct", "Point", "");
    let mut method = child("method", "Itself", "");
    method.flags.insert(FLAG_POINTER_RECEIVER.to_owned());
    method.flags.insert("escapes".to_owned());
    point.children = vec![method];

    let pack = Pack::default()
        .with_rule("structs", CONSTRUCTION_RUST_STRUCT_BODY, &["struct"])
        .with_disposition("owned", None, Some(true), "Option<Box<{0}>>", None);

    let err = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point]))
        .expect_err("an escaping receiver cannot be borrowed");
    assert!(matches!(err, TransformError::Ownership { .. }), "{err}");
}

/// A variadic SIGNATURE needs no decision, because the parameter is already a slice.
///
/// The source records `func f(args ...T)` with its last parameter typed `[]T` — that is what it IS
/// inside the function — so the signature translates through the ordinary slice rule. What needs a
/// decision is the CALL, where the trailing arguments have to be collected; that is refused where
/// it happens and fenced end to end by the refusal corpus.
#[test]
fn a_variadic_signature_is_an_ordinary_slice() {
    let mut printf = decl("func", "Printf", "");
    printf.flags.insert(FLAG_VARIADIC.to_owned());
    // A BODY, because the rung that emits one needs it: a function without a body is a refusal
    // rather than a stub, and this test is about the SIGNATURE.
    printf.children.push(child("body", "", ""));
    let pack = Pack::default().with_rule("funcs", CONSTRUCTION_RUST_FN_BODY, &["func"]);
    apply(&plan_with(&["funcs"]), &pack, &model_with(vec![printf]))
        .expect("a variadic signature carries no question of its own");
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

/// A trait method with NEITHER an observed receiver nor a declared one refuses.
///
/// The mode is not recoverable from an interface's own declaration — it says nothing about how an
/// implementation binds its receiver. It IS recoverable from the implementors, and the front end
/// now derives it where it can see them. This is the case where it cannot: nothing was observed to
/// implement `Named`, and the pack has not decided either, so emitting `&self` by default would be
/// the guess that made the fixture's mutating `Rename` unimplementable.
#[test]
fn a_trait_receiver_that_is_neither_observed_nor_declared_refuses() {
    let mut iface = decl("interface", "Named", "");
    iface.children = vec![child("method", "Rename", "")];
    let pack = Pack::default().with_rule("traits", CONSTRUCTION_RUST_TRAIT, &["interface"]);

    let err = apply(&plan_with(&["traits"]), &pack, &model_with(vec![iface]))
        .expect_err("an undeclared receiver mode must refuse");
    assert!(
        matches!(&err, TransformError::Unsupported { detail, .. }
            if detail.contains("no implementor") && detail.contains("guess")),
        "{err}"
    );
}

/// An OBSERVED receiver wins over the pack's declared one, per method.
///
/// The pack declares one mode for every trait method, which is what put `&mut self` on getters.
/// The front end derives it per method from the implementors it saw — exclusive exactly when one of
/// them mutates — so a read-only method binds shared even where the pack says otherwise.
#[test]
fn an_observed_receiver_overrides_the_declared_one() {
    let mut iface = decl("interface", "Named", "");
    let mut getter = child("method", "Name", "");
    getter
        .attrs
        .insert("receiver".to_owned(), "shared".to_owned());
    iface.children = vec![getter, child("method", "Rename", "")];

    let pack = Pack::default()
        .with_rule("traits", CONSTRUCTION_RUST_TRAIT, &["interface"])
        .with_trait_receiver("exclusive");

    let ir = apply(&plan_with(&["traits"]), &pack, &model_with(vec![iface]))
        .expect("a trait with one observed receiver must build");
    let rendered = format!("{ir:?}");
    assert!(
        rendered.contains("Shared"),
        "the observed shared receiver must reach the signature: {rendered}"
    );
    assert!(
        rendered.contains("Exclusive"),
        "the unobserved method must still take the pack's declared mode: {rendered}"
    );
}

/// And a declared one is honoured, so a mutating method is implementable.
#[test]
fn a_declared_exclusive_receiver_reaches_the_signature() {
    let mut iface = decl("interface", "Named", "");
    iface.children = vec![child("method", "Rename", "")];
    let pack = Pack::default()
        .with_rule("traits", CONSTRUCTION_RUST_TRAIT, &["interface"])
        .with_trait_receiver("exclusive");

    let ir = apply(&plan_with(&["traits"]), &pack, &model_with(vec![iface])).expect("apply");
    let text = rendered(&ir);
    assert!(text.contains("fn rename(&mut self)"), "{text}");
}
