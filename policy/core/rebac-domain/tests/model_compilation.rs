#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod model_compilation_support;

use policy_cedar_domain::rebac::{RebacRelation, UsersetRewrite};
use policy_rebac_domain::{ExpansionError, NamespaceCompileError, NamespaceConfig};

use model_compilation_support::assert_inherited_membership;

fn relation(name: &str) -> RebacRelation {
    RebacRelation::new(name).unwrap()
}

fn definition(name: &str, rewrite: UsersetRewrite) -> (String, RebacRelation, UsersetRewrite) {
    ("document".into(), relation(name), rewrite)
}

fn computed(name: &str) -> UsersetRewrite {
    UsersetRewrite::computed_userset(relation(name))
}

fn unknown(defining: &str, referenced: &str) -> NamespaceCompileError {
    NamespaceCompileError::UnknownRelationReference {
        object_type: "document".into(),
        relation: defining.into(),
        referenced_relation: referenced.into(),
    }
}

#[test]
fn equal_and_conflicting_duplicates_are_refused() {
    for replacement in [UsersetRewrite::This, computed("viewer")] {
        assert_eq!(
            NamespaceConfig::compile([
                definition("viewer", UsersetRewrite::This),
                definition("viewer", replacement),
            ]),
            Err(NamespaceCompileError::DuplicateRelation {
                object_type: "document".into(),
                relation: "viewer".into(),
            })
        );
    }
}

#[test]
fn duplicate_selection_is_stable_across_fragment_order() {
    let fragments =
        ["viewer", "editor", "viewer", "editor"].map(|name| definition(name, UsersetRewrite::This));
    let expected = Err(NamespaceCompileError::DuplicateRelation {
        object_type: "document".into(),
        relation: "editor".into(),
    });
    assert_eq!(NamespaceConfig::compile(fragments.clone()), expected);
    assert_eq!(
        NamespaceConfig::compile(fragments.into_iter().rev()),
        expected
    );
}

#[test]
fn forward_references_compile_independently_of_fragment_order() {
    let fragments = [
        definition("viewer", computed("member")),
        definition("member", UsersetRewrite::This),
    ];
    let forward = NamespaceConfig::compile(fragments.clone()).unwrap();
    let backward = NamespaceConfig::compile(fragments.into_iter().rev()).unwrap();
    assert_eq!(forward, backward);
    assert_eq!(
        forward.rewrite("document", &relation("viewer")),
        Ok(&computed("member"))
    );
}

#[test]
fn same_relation_name_on_another_type_does_not_resolve_local_reference() {
    assert_eq!(
        NamespaceConfig::compile([
            definition("viewer", computed("member")),
            ("folder".into(), relation("member"), UsersetRewrite::This),
        ]),
        Err(unknown("viewer", "member"))
    );
}

#[test]
fn missing_tupleset_source_is_refused() {
    assert_eq!(
        NamespaceConfig::compile([definition(
            "viewer",
            UsersetRewrite::tuple_to_userset(relation("parent"), relation("member")),
        )]),
        Err(unknown("viewer", "parent"))
    );
}

#[test]
fn nested_references_are_checked_in_every_composite_arm() {
    let missing = computed("missing");
    let rewrites = [
        UsersetRewrite::Union {
            children: vec![UsersetRewrite::This, missing.clone()],
        },
        UsersetRewrite::Intersection {
            children: vec![UsersetRewrite::This, missing.clone()],
        },
        UsersetRewrite::difference(missing.clone(), UsersetRewrite::This),
        UsersetRewrite::difference(UsersetRewrite::This, missing),
    ];
    for rewrite in rewrites {
        assert_eq!(
            NamespaceConfig::compile([definition("viewer", rewrite)]),
            Err(unknown("viewer", "missing"))
        );
    }
}

#[test]
fn empty_composites_are_refused_even_when_nested() {
    for (kind, rewrite) in [
        ("union", UsersetRewrite::Union { children: vec![] }),
        (
            "intersection",
            UsersetRewrite::Intersection { children: vec![] },
        ),
    ] {
        for candidate in [
            rewrite.clone(),
            UsersetRewrite::difference(UsersetRewrite::This, rewrite),
        ] {
            assert_eq!(
                NamespaceConfig::compile([definition("viewer", candidate)]),
                Err(NamespaceCompileError::EmptyRewrite {
                    object_type: "document".into(),
                    relation: "viewer".into(),
                    kind,
                })
            );
        }
    }
}

#[test]
fn model_rejection_preserves_existing_stratification_error() {
    let error = NamespaceConfig::compile([definition(
        "viewer",
        UsersetRewrite::difference(UsersetRewrite::This, computed("viewer")),
    )])
    .unwrap_err();
    assert_eq!(
        error,
        NamespaceCompileError::Model(ExpansionError::NonStratified {
            object_type: "document".into(),
            relation: "viewer".into(),
        })
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn cross_type_inheritance_compiles_and_drives_real_expansion() {
    let namespace = NamespaceConfig::compile([
        definition(
            "viewer",
            UsersetRewrite::tuple_to_userset(relation("parent"), relation("member")),
        ),
        definition("parent", UsersetRewrite::This),
        ("folder".into(), relation("member"), UsersetRewrite::This),
    ])
    .unwrap();
    assert_inherited_membership(&namespace);
}

#[test]
fn unknown_tuple_target_is_not_given_an_invented_type_rule() {
    assert!(
        NamespaceConfig::compile([
            definition(
                "viewer",
                UsersetRewrite::tuple_to_userset(relation("parent"), relation("not_declared"))
            ),
            definition("parent", UsersetRewrite::This),
        ])
        .is_ok()
    );
}

#[test]
fn legacy_explicit_redefinition_remains_last_wins() {
    let namespace = NamespaceConfig::new()
        .define("document", &relation("viewer"), computed("missing"))
        .define("document", &relation("viewer"), UsersetRewrite::This)
        .validated()
        .unwrap();
    assert_eq!(
        namespace.rewrite("document", &relation("viewer")),
        Ok(&UsersetRewrite::This)
    );
    assert!(
        NamespaceConfig::new()
            .define("document", &relation("viewer"), computed("missing"))
            .validated()
            .is_ok()
    );
}

#[test]
fn empty_model_and_positive_recursion_remain_valid() {
    assert!(NamespaceConfig::compile([]).is_ok());
    assert!(NamespaceConfig::compile([definition("viewer", computed("viewer"))]).is_ok());
}

#[test]
fn compile_errors_have_operator_readable_diagnostics() {
    assert_eq!(
        unknown("viewer", "member").to_string(),
        "document#viewer references undefined relation document#member"
    );
    assert_eq!(
        NamespaceCompileError::DuplicateRelation {
            object_type: "document".into(),
            relation: "viewer".into(),
        }
        .to_string(),
        "duplicate definition for document#viewer"
    );
    assert_eq!(
        NamespaceCompileError::EmptyRewrite {
            object_type: "document".into(),
            relation: "viewer".into(),
            kind: "union",
        }
        .to_string(),
        "document#viewer contains an empty union rewrite"
    );
}
