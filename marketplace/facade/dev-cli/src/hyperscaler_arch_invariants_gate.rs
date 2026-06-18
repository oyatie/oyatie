//! `oya gate validate hyperscaler-arch-invariants` runner.
//!
//! The lane validates the architecture invariant catalog itself. It does not
//! claim product PRD citation enforcement until that validator, fixture tests,
//! workflow, and branch-protection context exist together.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::usage;

const EXPECTED_SPEC_ID: &str = "EXE-HYPERSCALER-ARCH-INVARIANTS";
const EXPECTED_VERSION: &str = "1.0.0";
const EXPECTED_INVARIANT_COUNT: usize = 35;
const ADVISORY_STATUS: &str = "advisory-until-product-prd-validator";

const REQUIRED_PRODUCTS: &[&str] = &[
    "ads",
    "cloud",
    "community",
    "mail",
    "messenger",
    "foundry",
    "ontology",
    "saas",
    "search",
    "vertical",
    "workflow",
    "workflow-studio",
    "workspace",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HyperscalerArchInvariantsValidateArgs {
    spec_path: PathBuf,
}

impl Default for HyperscalerArchInvariantsValidateArgs {
    fn default() -> Self {
        Self {
            spec_path: PathBuf::from("specs/hyperscaler-architecture-invariants.json"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HyperscalerArchInvariantsReport {
    pub invariant_count: usize,
    pub product_count: usize,
    pub planned_lane_count: usize,
}

pub(crate) fn parse_hyperscaler_arch_invariants_validate_args(
    args: Vec<String>,
) -> Result<HyperscalerArchInvariantsValidateArgs, String> {
    let mut parsed = HyperscalerArchInvariantsValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--spec" => parsed.spec_path = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_hyperscaler_arch_invariants_gate(
    args: HyperscalerArchInvariantsValidateArgs,
) -> Result<HyperscalerArchInvariantsReport, String> {
    let spec = read_json(&args.spec_path)?;
    let root = object(&spec, "hyperscaler architecture invariants root")?;
    validate_meta(root)?;
    let invariant_refs = validate_invariants(root)?;
    let product_refs = validate_product_compliance(root, &invariant_refs.by_id)?;
    validate_product_refs_match_applicability(&invariant_refs.by_product, &product_refs)?;
    validate_enforcement_chain(root)?;

    Ok(HyperscalerArchInvariantsReport {
        invariant_count: invariant_refs.by_id.len(),
        product_count: product_refs.len(),
        planned_lane_count: invariant_refs.planned_lanes.len(),
    })
}

fn read_json(path: &PathBuf) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("{} unreadable: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("{} invalid JSON: {error}", path.display()))
}

fn validate_meta(root: &serde_json::Map<String, Value>) -> Result<(), String> {
    let meta = object_field(root, "_meta")?;
    require_string_value(meta, "doc_class", "Machine-Readable-Spec")?;
    require_string_value(meta, "spec_id", EXPECTED_SPEC_ID)?;
    require_string_value(meta, "version", EXPECTED_VERSION)?;
    require_string_value(meta, "status", "Accepted")?;
    require_string_value(meta, "enforcement_status", ADVISORY_STATUS)?;
    require_string_value(meta, "binding_adr", "ADR-0128")?;
    require_string_contains(meta, "purpose", "advisory")?;
    require_string_array_contains(meta, "referenced_by", "specs/hyperscaler-gates.json")?;
    Ok(())
}

#[derive(Debug)]
struct InvariantRefs {
    by_id: BTreeSet<String>,
    by_product: BTreeMap<String, BTreeSet<String>>,
    planned_lanes: BTreeSet<String>,
}

fn validate_invariants(root: &serde_json::Map<String, Value>) -> Result<InvariantRefs, String> {
    let invariants = array_field(root, "invariants")?;
    if invariants.len() != EXPECTED_INVARIANT_COUNT {
        return Err(format!(
            "invariants must contain {EXPECTED_INVARIANT_COUNT} rows for version {EXPECTED_VERSION}, got {}",
            invariants.len()
        ));
    }

    let mut by_id = BTreeSet::new();
    let mut by_product: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut planned_lanes = BTreeSet::new();
    let products = required_products();

    for (index, row) in invariants.iter().enumerate() {
        let invariant = row
            .as_object()
            .ok_or_else(|| format!("invariants[{index}] must be an object"))?;
        reject_field(invariant, "enforced_by", &format!("invariants[{index}]"))?;
        reject_field(
            invariant,
            "verification_command",
            &format!("invariants[{index}]"),
        )?;
        let id = non_empty_string(invariant, "id", &format!("invariants[{index}]"))?;
        if !id.starts_with("INV-") {
            return Err(format!(
                "invariants[{index}].id must start with INV-, got {id:?}"
            ));
        }
        if !by_id.insert(id.to_owned()) {
            return Err(format!("duplicate invariant id {id:?}"));
        }
        non_empty_string(invariant, "category", &format!("invariants[{index}]"))?;
        non_empty_string(invariant, "rule", &format!("invariants[{index}]"))?;
        non_empty_string(invariant, "rationale", &format!("invariants[{index}]"))?;
        let planned_lane = non_empty_string(
            invariant,
            "planned_enforced_by",
            &format!("invariants[{index}]"),
        )?;
        if !planned_lane.starts_with("oya-governance-") {
            return Err(format!(
                "invariants[{index}].planned_enforced_by must name an oya-governance-* lane"
            ));
        }
        planned_lanes.insert(planned_lane.to_owned());
        let planned_command = non_empty_string(
            invariant,
            "planned_verification_command",
            &format!("invariants[{index}]"),
        )?;
        if !planned_command.starts_with("cargo run -p oya-dev-cli -- gate validate ") {
            return Err(format!(
                "invariants[{index}].planned_verification_command must be an oya-dev-cli gate command"
            ));
        }
        let applies_to = string_array_field(
            invariant,
            "applies_to_products",
            &format!("invariants[{index}]"),
        )?;
        if applies_to.is_empty() {
            return Err(format!(
                "invariants[{index}].applies_to_products must be non-empty"
            ));
        }
        for product in applies_to {
            if !products.contains(product.as_str()) {
                return Err(format!(
                    "invariant {id} applies_to_products contains unknown product {product:?}"
                ));
            }
            by_product.entry(product).or_default().insert(id.to_owned());
        }
    }

    Ok(InvariantRefs {
        by_id,
        by_product,
        planned_lanes,
    })
}

fn validate_product_compliance(
    root: &serde_json::Map<String, Value>,
    invariant_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let compliance = object_field(root, "per_product_required_compliance")?;
    let products = required_products();
    let mut product_refs = BTreeMap::new();
    for product in &products {
        let refs = string_array_field(
            compliance,
            product,
            &format!("per_product_required_compliance.{product}"),
        )?;
        if refs.is_empty() {
            return Err(format!(
                "per_product_required_compliance.{product} must be non-empty"
            ));
        }
        let mut seen = BTreeSet::new();
        for invariant_id in refs {
            if !invariant_ids.contains(&invariant_id) {
                return Err(format!(
                    "per_product_required_compliance.{product} references unknown invariant {invariant_id:?}"
                ));
            }
            if !seen.insert(invariant_id.clone()) {
                return Err(format!(
                    "per_product_required_compliance.{product} duplicates invariant {invariant_id:?}"
                ));
            }
        }
        product_refs.insert((*product).to_owned(), seen);
    }
    for product in compliance.keys() {
        if !products.contains(product.as_str()) {
            return Err(format!(
                "per_product_required_compliance contains unknown product {product:?}"
            ));
        }
    }
    Ok(product_refs)
}

fn validate_product_refs_match_applicability(
    by_product: &BTreeMap<String, BTreeSet<String>>,
    product_refs: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), String> {
    for product in required_products() {
        let from_invariants = by_product
            .get(&product)
            .ok_or_else(|| format!("no invariant applies_to_products entry for {product}"))?;
        let from_compliance = product_refs
            .get(&product)
            .ok_or_else(|| format!("missing per-product compliance entry for {product}"))?;
        for invariant_id in from_invariants {
            if !from_compliance.contains(invariant_id) {
                return Err(format!(
                    "product {product} omits applicable invariant {invariant_id} from per_product_required_compliance"
                ));
            }
        }
        for invariant_id in from_compliance {
            if !from_invariants.contains(invariant_id) {
                return Err(format!(
                    "product {product} requires invariant {invariant_id}, but that invariant does not list the product in applies_to_products"
                ));
            }
        }
    }
    Ok(())
}

fn validate_enforcement_chain(root: &serde_json::Map<String, Value>) -> Result<(), String> {
    let chain = object_field(root, "enforcement_chain")?;
    require_string_value(chain, "adr", "ADR-0128")?;
    require_string_value(chain, "enforcement_status", ADVISORY_STATUS)?;
    require_string_contains(chain, "meta_lane", "planned")?;
    require_string_contains(chain, "meta_lane", "branch-protection")?;
    require_string_contains(chain, "per_invariant_lane", "planned_enforced_by")?;
    require_string_value(
        chain,
        "validation_command",
        "cargo run -p oya-dev-cli -- gate validate hyperscaler-arch-invariants",
    )?;
    require_string_contains(chain, "ci_lane", "advisory")?;
    for required in [
        "product-prd-json validator",
        "fixture-tree integration tests",
        "GitHub Actions workflow",
        "branch protection",
    ] {
        require_string_array_contains(chain, "activation_requires", required)?;
    }
    Ok(())
}

fn object<'a>(value: &'a Value, label: &str) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{label} must be a JSON object"))
}

fn object_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    object
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing required object field `{key}`"))
}

fn array_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a Vec<Value>, String> {
    object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing required array field `{key}`"))
}

fn non_empty_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<&'a str, String> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} missing required string field `{key}`"))?;
    if value.trim().is_empty() {
        Err(format!("{label} field `{key}` must be non-empty"))
    } else {
        Ok(value)
    }
}

fn string_array_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<Vec<String>, String> {
    let values = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{label} missing required array field `{key}`"))?;
    let mut parsed = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let Some(item) = value.as_str() else {
            return Err(format!("{label}.{key}[{index}] must be a string"));
        };
        if item.trim().is_empty() {
            return Err(format!("{label}.{key}[{index}] must be non-empty"));
        }
        parsed.push(item.to_owned());
    }
    Ok(parsed)
}

fn require_string_value(
    object: &serde_json::Map<String, Value>,
    key: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = non_empty_string(object, key, key)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "field `{key}` must be {expected:?}, got {actual:?}"
        ))
    }
}

fn require_string_contains(
    object: &serde_json::Map<String, Value>,
    key: &str,
    needle: &str,
) -> Result<(), String> {
    let actual = non_empty_string(object, key, key)?;
    if actual.contains(needle) {
        Ok(())
    } else {
        Err(format!("field `{key}` must contain {needle:?}"))
    }
}

fn require_string_array_contains(
    object: &serde_json::Map<String, Value>,
    key: &str,
    expected: &str,
) -> Result<(), String> {
    let values = string_array_field(object, key, key)?;
    if values.iter().any(|value| value.contains(expected)) {
        Ok(())
    } else {
        Err(format!("field `{key}` must contain {expected:?}"))
    }
}

fn reject_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<(), String> {
    if object.contains_key(key) {
        Err(format!("{label} must not use active field `{key}`"))
    } else {
        Ok(())
    }
}

fn required_products() -> BTreeSet<String> {
    REQUIRED_PRODUCTS
        .iter()
        .map(|product| (*product).to_owned())
        .collect()
}
