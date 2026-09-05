use super::{Expr, Model, Object, Relation, Subject, Tuple};

pub const BOOLEAN_MODELS: usize = 1 << 8;
pub const CYCLE_MODELS: usize = 1 << 6;
pub const INHERITANCE_MODELS: usize = 1 << 8;

fn object(object_type: &'static str, object_id: &'static str) -> Object {
    Object {
        object_type,
        object_id,
    }
}

fn concrete(object: Object, relation: &'static str, subject: Object) -> Tuple {
    Tuple {
        object,
        relation,
        subject: Subject::Concrete(subject),
    }
}

fn userset(
    object: Object,
    relation: &'static str,
    subject: Object,
    subject_relation: &'static str,
) -> Tuple {
    Tuple {
        object,
        relation,
        subject: Subject::Userset {
            object: subject,
            relation: subject_relation,
        },
    }
}

fn selected(candidates: Vec<Tuple>, mask: usize) -> Vec<Tuple> {
    candidates
        .into_iter()
        .enumerate()
        .filter_map(|(bit, tuple)| ((mask & (1 << bit)) != 0).then_some(tuple))
        .collect()
}

pub fn boolean(mask: usize) -> Model {
    assert!(mask < BOOLEAN_MODELS);
    let alice = object("user", "alice");
    let bob = object("user", "bob");
    let one = object("document", "one");
    let two = object("document", "two");
    let mut candidates = Vec::new();
    for document in [&one, &two] {
        for relation in ["member", "editor"] {
            for user in [&alice, &bob] {
                candidates.push(concrete(document.clone(), relation, user.clone()));
            }
        }
    }
    let computed = |name| Expr::Computed(name);
    Model {
        objects: vec![one, two],
        users: vec![alice, bob],
        relations: vec![
            Relation {
                object_type: "document",
                name: "member",
                stratum: 0,
                expr: Expr::This,
            },
            Relation {
                object_type: "document",
                name: "editor",
                stratum: 0,
                expr: Expr::This,
            },
            Relation {
                object_type: "document",
                name: "alias",
                stratum: 0,
                expr: computed("member"),
            },
            Relation {
                object_type: "document",
                name: "either",
                stratum: 0,
                expr: Expr::Union(vec![computed("member"), computed("editor")]),
            },
            Relation {
                object_type: "document",
                name: "both",
                stratum: 0,
                expr: Expr::Intersection(vec![computed("member"), computed("editor")]),
            },
            Relation {
                object_type: "document",
                name: "unblocked",
                stratum: 1,
                expr: Expr::Difference(Box::new(computed("member")), Box::new(computed("editor"))),
            },
            Relation {
                object_type: "document",
                name: "nested",
                stratum: 2,
                expr: Expr::Intersection(vec![
                    computed("either"),
                    Expr::Union(vec![computed("both"), computed("unblocked")]),
                ]),
            },
        ],
        tuples: selected(candidates, mask),
    }
}

pub fn cycle(mask: usize) -> Model {
    assert!(mask < CYCLE_MODELS);
    let alice = object("user", "alice");
    let bob = object("user", "bob");
    let a = object("group", "a");
    let b = object("group", "b");
    let candidates = vec![
        concrete(a.clone(), "member", alice.clone()),
        concrete(a.clone(), "member", bob.clone()),
        concrete(b.clone(), "member", alice.clone()),
        concrete(b.clone(), "member", bob.clone()),
        userset(a.clone(), "member", b.clone(), "member"),
        userset(b.clone(), "member", a.clone(), "member"),
    ];
    Model {
        objects: vec![a, b],
        users: vec![alice, bob],
        relations: vec![Relation {
            object_type: "group",
            name: "member",
            stratum: 0,
            expr: Expr::This,
        }],
        tuples: selected(candidates, mask),
    }
}

pub fn inheritance(mask: usize) -> Model {
    assert!(mask < INHERITANCE_MODELS);
    let alice = object("user", "alice");
    let bob = object("user", "bob");
    let first = object("folder", "first");
    let second = object("folder", "second");
    let one = object("document", "one");
    let two = object("document", "two");
    let mut candidates = Vec::new();
    for folder in [&first, &second] {
        for user in [&alice, &bob] {
            candidates.push(concrete(folder.clone(), "member", user.clone()));
        }
    }
    for document in [&one, &two] {
        for folder in [&first, &second] {
            candidates.push(concrete(document.clone(), "parent", folder.clone()));
        }
    }
    Model {
        objects: vec![first, second, one, two],
        users: vec![alice, bob],
        relations: vec![
            Relation {
                object_type: "folder",
                name: "member",
                stratum: 0,
                expr: Expr::This,
            },
            Relation {
                object_type: "document",
                name: "parent",
                stratum: 0,
                expr: Expr::This,
            },
            Relation {
                object_type: "document",
                name: "viewer",
                stratum: 0,
                expr: Expr::TupleToUserset {
                    tupleset: "parent",
                    computed: "member",
                },
            },
        ],
        tuples: selected(candidates, mask),
    }
}
