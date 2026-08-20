#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

const BUCK: &str = include_str!("../BUCK");
const CARGO_CONFIG: &str = include_str!("../../../../.cargo/config.toml");

const EXPECTED: [(&str, &str, &str); 15] = [
    (
        "OYA_API_CONTRACT_SSOT",
        "//specs:api-contract-ssot-canonical.json",
        "specs/api-contract-ssot-canonical.json",
    ),
    (
        "OYA_ENDPOINT_TRANSPORT_PROFILE",
        "//network/ports/transport-profile:endpoint-transport-profile.contract.json",
        "network/ports/transport-profile/endpoint-transport-profile.contract.json",
    ),
    (
        "OYA_MASTER_PLAN_SEQUENCING",
        "//specs:master-plan-sequencing.json",
        "specs/master-plan-sequencing.json",
    ),
    (
        "OYA_MICROSERVICE_MANIFEST_SCHEMA",
        "//specs/microservices:manifest-schema.json",
        "specs/microservices/manifest-schema.json",
    ),
    (
        "OYA_PRODUCT_PROTOCOL_CONTRACT",
        "//specs:product-protocol-contract.json",
        "specs/product-protocol-contract.json",
    ),
    (
        "OYA_PRODUCT_PROTOCOL_NEGATIVE_CASES",
        ":negative-cases.json",
        "ci/facade/product-protocol-policy/fixtures/negative-cases.json",
    ),
    (
        "OYA_PRODUCT_PROTOCOL_POLICY",
        ":product-protocol-policy.json",
        "ci/facade/product-protocol-policy/product-protocol-policy.json",
    ),
    (
        "OYA_ROOT_HUB_POINTERS",
        "//specs:root-hub-pointers.json",
        "specs/root-hub-pointers.json",
    ),
    (
        "OYA_ADR_0632",
        "//docs/adr-archive:ADR-0632-product-protocol-and-owned-fabric-posture.md",
        "docs/adr-archive/ADR-0632-product-protocol-and-owned-fabric-posture.md",
    ),
    (
        "OYA_ADR_0157",
        "//docs/adr-archive:ADR-0157-api-gateway-tier.md",
        "docs/adr-archive/ADR-0157-api-gateway-tier.md",
    ),
    (
        "OYA_ADR_0167",
        "//docs/adr-archive:ADR-0167-tenant-cli.md",
        "docs/adr-archive/ADR-0167-tenant-cli.md",
    ),
    (
        "OYA_ADR_0176",
        "//docs/adr-archive:ADR-0176-brownout-degradation-signal-api.md",
        "docs/adr-archive/ADR-0176-brownout-degradation-signal-api.md",
    ),
    (
        "OYA_ADR_0182",
        "//docs/adr-archive:ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md",
        "docs/adr-archive/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md",
    ),
    (
        "OYA_ADR_0246",
        "//docs/adr-archive:ADR-0246-policy-engine-substrate-promotion.md",
        "docs/adr-archive/ADR-0246-policy-engine-substrate-promotion.md",
    ),
    (
        "OYA_ADR_0258",
        "//docs/adr-archive:ADR-0258-api-versioning-model.md",
        "docs/adr-archive/ADR-0258-api-versioning-model.md",
    ),
];

#[derive(Debug, Eq, PartialEq)]
struct CargoBinding {
    value: String,
    relative: bool,
    force_false: bool,
}

fn buck_env_bindings(document: &str) -> BTreeMap<String, String> {
    let body = document
        .split_once("    env = {\n")
        .expect("product-protocol Buck target must declare env")
        .1
        .split_once("    resources = [")
        .expect("product-protocol Buck env must precede resources")
        .0;
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with('"') {
                return None;
            }
            let (key, value) = line.split_once(": ").expect("Buck env entry");
            let location = value
                .strip_prefix("\"$(location ")
                .and_then(|value| value.strip_suffix("\","))
                .and_then(|value| value.strip_suffix(')'))
                .expect("Buck env value must be a location");
            Some((key.trim_matches('"').to_owned(), location.to_owned()))
        })
        .collect()
}

fn cargo_env_bindings(document: &str) -> BTreeMap<String, CargoBinding> {
    document
        .split_once("[env]\n")
        .expect("Cargo config env table")
        .1
        .lines()
        .take_while(|line| !line.starts_with('['))
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, table) = line.split_once(" = ")?;
            if !table.starts_with('{') {
                return None;
            }
            let value = table
                .split_once("value = \"")?
                .1
                .split_once('"')?
                .0
                .to_owned();
            Some((
                key.to_owned(),
                CargoBinding {
                    value,
                    relative: table.contains("relative = true"),
                    force_false: table.contains("force = false"),
                },
            ))
        })
        .collect()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root")
}

#[test]
fn cargo_bindings_are_exact_portable_mirror_of_buck_locations() {
    let expected_buck = EXPECTED
        .iter()
        .map(|(key, label, _)| ((*key).to_owned(), (*label).to_owned()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        buck_env_bindings(BUCK),
        expected_buck,
        "the code-owned resource set must move with the Buck target"
    );

    let cargo = cargo_env_bindings(CARGO_CONFIG);
    let root = repo_root();
    for (key, _, relative_path) in EXPECTED {
        let binding = cargo
            .get(key)
            .unwrap_or_else(|| panic!("Cargo is missing Buck resource {key}"));
        assert_eq!(binding.value, relative_path, "{key} path drifted");
        assert!(binding.relative, "{key} must be config-relative");
        assert!(
            binding.force_false,
            "{key} must preserve explicit CI/Buck authority"
        );
        assert!(
            Path::new(&binding.value)
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
            "{key} must be a portable repository-relative path"
        );
        assert!(
            root.join(&binding.value).is_file(),
            "{key} source is missing"
        );

        let runtime = std::env::var_os(key)
            .map(PathBuf::from)
            .unwrap_or_else(|| panic!("Cargo did not export {key}"));
        assert!(
            runtime.is_file(),
            "Cargo exported a missing resource for {key}: {}",
            runtime.display()
        );
    }
}
