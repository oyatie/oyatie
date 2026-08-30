#![allow(clippy::unwrap_used)]

use dependency_declarations_generation::{
    DeclarationProviderCapabilityPort, RenderedDeclarationProjectionPort,
};
use dependency_declarations_generation_reindeer::StarlarkSyntaxProjectionV1;
use dependency_declarations_reconcile::{
    DigestV1, ProjectionPortErrorV1, RenderedRuleGraphV1, RenderedRuleV1, SemanticValueV1,
};

const PREFIX: &str = concat!(
    "# generated\n",
    "load(\"@prelude//rust:cargo_package.bzl\", \"cargo\")\n",
);
const LIBRARY: &str = concat!(
    "cargo.rust_library(\n",
    "    name = \"demo\",\n",
    "    deps = [\"//shared:api\"],\n",
    "    edition = 2024,\n",
    "    platform_flags = [\"common\"] + select({\n",
    "        \"DEFAULT\": [\"fallback\"],\n",
    "        \"prelude//os:linux\": [\"linux\"],\n",
    "    }),\n",
    "    proc_macro = False,\n",
    ")\n",
);
const ALIAS: &str = concat!(
    "alias(\n",
    "    name = \"public-demo\",\n",
    "    actual = \":demo\",\n",
    "    visibility = [\"PUBLIC\"],\n",
    ")\n",
);
const RENDERED: &str = concat!(
    "# generated\n",
    "load(\"@prelude//rust:cargo_package.bzl\", \"cargo\")\n",
    "cargo.rust_library(\n",
    "    name = \"demo\",\n",
    "    deps = [\"//shared:api\"],\n",
    "    edition = 2024,\n",
    "    platform_flags = [\"common\"] + select({\n",
    "        \"DEFAULT\": [\"fallback\"],\n",
    "        \"prelude//os:linux\": [\"linux\"],\n",
    "    }),\n",
    "    proc_macro = False,\n",
    ")\n",
    "\n",
    "alias(\n",
    "    name = \"public-demo\",\n",
    "    actual = \":demo\",\n",
    "    visibility = [\"PUBLIC\"],\n",
    ")\n",
);

#[test]
fn whole_artifact_projection_recovers_every_rendered_semantic_field() {
    let profile = DigestV1::of(b"qualified parser renderer schema grammar profile");
    let adapter = StarlarkSyntaxProjectionV1::new(profile);
    let projection = adapter.project(RENDERED.as_bytes()).unwrap();

    assert!(adapter.supports(&profile));
    assert!(!adapter.supports(&DigestV1::of(b"another profile")));
    assert_eq!(projection.profile_sha256(), profile);
    assert_eq!(
        projection.output_sha256(),
        DigestV1::of(RENDERED.as_bytes())
    );
    assert_eq!(projection.graph(), &expected_graph());
}

#[test]
fn syntax_and_lossy_or_noncanonical_forms_are_distinct_refusals() {
    let adapter = StarlarkSyntaxProjectionV1::new(DigestV1::of(b"profile"));
    assert_eq!(
        adapter.project(b"rule(\n").unwrap_err(),
        ProjectionPortErrorV1::InvalidSyntax
    );

    let unsupported = [
        "value = 1\n",
        "rule(\n    # hidden field\n    name = \"demo\",\n)\n",
        "rule(\"demo\", name = \"demo\")\n",
        "rule(name = \"de\" + \"mo\")\n",
        "rule()\n",
        "rule(name = \"demo\")",
        "rule(name = \"first\")\nrule(name = \"second\")\n",
        "rule(name = \"demo\")\n\nload(\"//rules:defs.bzl\", \"rule\")\n",
        "rule(name = \"demo\", value = 2147483648)\n",
        "rule(name = \"same\")\n\nrule(name = \"same\")\n",
    ];
    for source in unsupported {
        assert_eq!(
            adapter.project(source.as_bytes()).unwrap_err(),
            ProjectionPortErrorV1::UnsupportedSyntax,
            "{source:?}"
        );
    }
}

#[test]
fn bounded_arbitrary_bytes_never_panic_or_create_partial_projection() {
    let adapter = StarlarkSyntaxProjectionV1::new(DigestV1::of(b"profile"));
    for seed in 0_u8..16 {
        for length in 0..128 {
            let bytes: Vec<u8> = (0..length)
                .map(|index| seed.wrapping_add((index as u8).wrapping_mul(31)))
                .collect();
            let result = std::panic::catch_unwind(|| adapter.project(&bytes));
            assert!(result.is_ok());
        }
    }
}

fn expected_graph() -> RenderedRuleGraphV1 {
    let library = SemanticValueV1::call_named(
        "cargo.rust_library",
        vec![
            ("name".to_owned(), SemanticValueV1::string("demo").unwrap()),
            (
                "deps".to_owned(),
                SemanticValueV1::list(vec![SemanticValueV1::string("//shared:api").unwrap()])
                    .unwrap(),
            ),
            ("edition".to_owned(), SemanticValueV1::signed(2024)),
            ("platform_flags".to_owned(), selected_flags()),
            ("proc_macro".to_owned(), SemanticValueV1::boolean(false)),
        ],
    )
    .unwrap();
    let alias = SemanticValueV1::call_named(
        "alias",
        vec![
            (
                "name".to_owned(),
                SemanticValueV1::string("public-demo").unwrap(),
            ),
            (
                "actual".to_owned(),
                SemanticValueV1::string(":demo").unwrap(),
            ),
            (
                "visibility".to_owned(),
                SemanticValueV1::list(vec![SemanticValueV1::string("PUBLIC").unwrap()]).unwrap(),
            ),
        ],
    )
    .unwrap();
    RenderedRuleGraphV1::try_new(
        PREFIX.as_bytes().to_vec(),
        vec![
            RenderedRuleV1::new(0, library, DigestV1::of(LIBRARY.as_bytes())),
            RenderedRuleV1::new(1, alias, DigestV1::of(ALIAS.as_bytes())),
        ],
    )
    .unwrap()
}

fn selected_flags() -> SemanticValueV1 {
    let select = SemanticValueV1::call_positional(
        "select",
        vec![
            SemanticValueV1::map(vec![
                (
                    SemanticValueV1::string("DEFAULT").unwrap(),
                    SemanticValueV1::list(vec![SemanticValueV1::string("fallback").unwrap()])
                        .unwrap(),
                ),
                (
                    SemanticValueV1::string("prelude//os:linux").unwrap(),
                    SemanticValueV1::list(vec![SemanticValueV1::string("linux").unwrap()]).unwrap(),
                ),
            ])
            .unwrap(),
        ],
    )
    .unwrap();
    SemanticValueV1::select_addition(vec![
        SemanticValueV1::list(vec![SemanticValueV1::string("common").unwrap()]).unwrap(),
        select,
    ])
    .unwrap()
}
