#![allow(dead_code)]

#[path = "../ci/assert-retired-grouping-wording.rs"]
mod gate;

use std::fs;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "oyatie-retired-grouping-wording-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap_or_else(|error| {
        panic!("create temp dir {}: {}", path.display(), error);
    });
    path
}

#[test]
fn good_fixture_allows_flat_packaging_and_connector_terms() {
    let root = temp_dir("good");
    fs::create_dir_all(root.join("evidence/vcs")).unwrap();
    fs::create_dir_all(root.join("crates/oya-connector-netsuite-adapter")).unwrap();
    fs::write(
        root.join("platform.md"),
        "Tenant/RBAC packaging uses entitlement_set_id, test_set_id, and eval_set_id.\n\
Cryptographic cipher suite and C-suite wording are allowed where technically correct.\n\
NetSuite remains an allowed vendor name.\n\
Legitimate cloud platform, developer platform, and platform-approved font policy wording are allowed.\n",
    )
    .unwrap();
    fs::write(
        root.join("evidence/vcs/cs-ent-suite-historical.json"),
        "{\"historical\":\"cs-ent-suite evidence is immutable\"}\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/oya-connector-netsuite-adapter/README.md"),
        "NetSuite adapter keeps vendor spelling.\n",
    )
    .unwrap();
    fs::write(
        root.join("connector-iac.yaml"),
        "name: oya-connector\nrepository: oya-connector\nspiffe: spiffe://oyatie.dev/ns/connector/sa/connector-app\nhost: connector.oyatie.com\ndns: connector.svc.cluster.local\n",
    )
    .unwrap();

    let evaluation = gate::evaluate(&root).expect("fixture scan should run");
    let _ = fs::remove_dir_all(&root);
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.matches);
    assert!(evaluation.matches.is_empty(), "{:?}", evaluation.matches);
}

#[test]
fn bad_fixture_rejects_retired_grouping_wording() {
    let root = temp_dir("bad");
    fs::write(
        root.join("legacy.yaml"),
        "product_class: \"suite\"\nsuite_id: connect\nformer_platform: \"Platform uses platform_id and platform-app\"\nformer_module: \"Module uses module_id\"\nformer_lowercase: \"enterprise platform, connect platform, enterprise module, connect product, and connect suite wrappers are retired\"\nname: oya-connect\nrepository: oya-connect\nspiffe: spiffe://oyatie.dev/ns/connect/sa/connect-webhook-receiver-edge\nhost: connect.oyatie.app\ndns: connect.svc.cluster.local\n",
    )
    .unwrap();

    let evaluation = gate::evaluate(&root).expect("fixture scan should run");
    let _ = fs::remove_dir_all(&root);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .matches
            .iter()
            .any(|line| line.contains("product_class")),
        "{:?}",
        evaluation.matches
    );
    assert!(
        evaluation
            .matches
            .iter()
            .any(|line| line.contains("connect.svc.cluster.local")),
        "{:?}",
        evaluation.matches
    );
}

#[test]
fn line_classifier_preserves_known_good_and_bad_edges() {
    assert!(!gate::line_has_retired_grouping_wording(
        "Legitimate cloud platform and developer platform wording is allowed."
    ));
    assert!(!gate::line_has_retired_grouping_wording(
        "NetSuite connector and oya-connector are allowed names."
    ));
    assert!(gate::line_has_retired_grouping_wording(
        "former_lowercase: enterprise platform, connect product"
    ));
    assert!(gate::line_has_retired_grouping_wording(
        "product_class: \"platform-app\""
    ));
}
