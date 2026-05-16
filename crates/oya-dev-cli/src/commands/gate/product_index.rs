// Purpose: validate that `docs/products/README.md` axis-product table stays in
// sync with `docs/machine-readable/catalog.json`. Ported from
// `scripts/check-product-index.py` per
// `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-5.
// Naming-justification: handler lives at `commands/gate/product_index.rs`
// (no `_gate` suffix — redundant inside the `commands/gate/` module path);
// surface command `gate validate product-index` is canonical kebab-case
// verb-noun (ADR-0105 v4 BNF). Mirrors the Wave 2 placement pattern in
// `commands/gate/architecture_boundaries.rs`.

use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProductIndexValidateArgs {
    products_readme: PathBuf,
    catalog_path: PathBuf,
}

const REQUIRED_AXIS_ROWS: &[(&str, &str)] = &[
    ("saas-platform/PRD.md", "SaaS Platform"),
    ("workspace/PRD.md", "Workspace"),
    ("foundry/PRD.md", "Foundry"),
    ("cloud/PRD.md", "Cloud Provider"),
    ("search/PRD.md", "Search"),
    ("ads-analytics/PRD.md", "Ads + Analytics"),
    ("Vertical Industry Cloud", "Vertical Industry Cloud"),
];

const REQUIRED_CATALOG_PRODUCTS: &[&str] = &[
    "saas-platform",
    "workspace",
    "foundry",
    "cloud",
    "search",
    "ads-analytics",
];

const AXIS_SECTION_HEADER: &str = "### Axis products (7)";
const VERTICAL_SECTION_HEADER: &str = "### Vertical products";

pub(crate) fn run(args: Vec<String>) -> ExitCode {
    match parse_product_index_validate_args(args) {
        Ok(parsed) => match validate_product_index(parsed) {
            Ok(report) => {
                println!(
                    "product-index validation passed: {} axis rows, {} catalog products",
                    report.axis_rows, report.catalog_products,
                );
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("product-index validation failed: {message}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn parse_product_index_validate_args(
    args: Vec<String>,
) -> Result<ProductIndexValidateArgs, String> {
    let mut parsed = ProductIndexValidateArgs {
        products_readme: PathBuf::from("docs/products/README.md"),
        catalog_path: PathBuf::from("docs/machine-readable/catalog.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(
                "usage: oya gate validate product-index [--products-readme <path>] [--catalog <path>]"
                    .to_string(),
            );
        };
        match flag.as_str() {
            "--products-readme" => parsed.products_readme = PathBuf::from(value),
            "--catalog" => parsed.catalog_path = PathBuf::from(value),
            _ => {
                return Err(format!(
                    "unknown flag {flag}; usage: oya gate validate product-index [--products-readme <path>] [--catalog <path>]"
                ));
            }
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProductIndexReport {
    axis_rows: usize,
    catalog_products: usize,
}

fn validate_product_index(args: ProductIndexValidateArgs) -> Result<ProductIndexReport, String> {
    let readme = std::fs::read_to_string(&args.products_readme).map_err(|error| {
        format!(
            "product README unreadable {}: {error}",
            args.products_readme.display()
        )
    })?;
    let catalog_text = std::fs::read_to_string(&args.catalog_path).map_err(|error| {
        format!(
            "machine-readable catalog unreadable {}: {error}",
            args.catalog_path.display()
        )
    })?;
    validate_product_index_strings(&readme, &catalog_text, |relative_path| {
        let candidate = args
            .catalog_path
            .parent()
            .and_then(std::path::Path::parent)
            .and_then(std::path::Path::parent)
            .map(|parent| parent.join(relative_path));
        match candidate {
            Some(path) => path.exists(),
            None => std::path::Path::new(relative_path).exists(),
        }
    })
}

fn validate_product_index_strings(
    readme: &str,
    catalog_text: &str,
    prd_exists: impl Fn(&str) -> bool,
) -> Result<ProductIndexReport, String> {
    let axis_section = readme
        .split_once(AXIS_SECTION_HEADER)
        .ok_or_else(|| "product README is missing the axis section header".to_string())?
        .1;
    let axis_section = axis_section
        .split_once(VERTICAL_SECTION_HEADER)
        .ok_or_else(|| "product README is missing the vertical section header".to_string())?
        .0;

    let axis_rows: Vec<&str> = axis_section
        .lines()
        .filter(|line| {
            line.starts_with("| ") && !line.starts_with("| Product") && !line.starts_with("|---")
        })
        .collect();
    if axis_rows.len() != 7 {
        return Err(format!("expected 7 axis rows, found {}", axis_rows.len()));
    }

    for (needle, label) in REQUIRED_AXIS_ROWS {
        if !axis_rows.iter().any(|row| row.contains(needle)) {
            return Err(format!("missing axis product row for {label}"));
        }
    }

    let foundry_count = axis_rows
        .iter()
        .filter(|row| row.contains("foundry/PRD.md"))
        .count();
    if foundry_count != 1 {
        return Err("Foundry appears more than once in the axis product table".to_string());
    }

    let catalog: serde_json::Value = serde_json::from_str(catalog_text)
        .map_err(|error| format!("machine-readable catalog JSON invalid: {error}"))?;
    let products = catalog
        .get("products")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "machine-readable catalog missing products object".to_string())?;

    for product_id in REQUIRED_CATALOG_PRODUCTS {
        if !products.contains_key(*product_id) {
            return Err(format!(
                "machine-readable catalog missing product {product_id}"
            ));
        }
    }

    let mut missing_paths: Vec<String> = Vec::new();
    for (product_id, record) in products {
        let Some(path) = record.get("prd_path").and_then(|value| value.as_str()) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        if !prd_exists(path) {
            missing_paths.push(format!("{product_id}:{path}"));
        }
    }
    if !missing_paths.is_empty() {
        return Err(format!(
            "machine-readable catalog references missing PRDs: {}",
            missing_paths.join(", ")
        ));
    }

    Ok(ProductIndexReport {
        axis_rows: axis_rows.len(),
        catalog_products: products.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_README: &str = "\
intro

### Axis products (7)

| Product | PRD |
|---|---|
| SaaS Platform | saas-platform/PRD.md |
| Workspace | workspace/PRD.md |
| Foundry | foundry/PRD.md |
| Cloud Provider | cloud/PRD.md |
| Search | search/PRD.md |
| Ads + Analytics | ads-analytics/PRD.md |
| Vertical Industry Cloud | n/a |

### Vertical products

later content
";

    const VALID_CATALOG: &str = r#"{
        "products": {
            "saas-platform": {"prd_path": "docs/products/saas-platform/PRD.md"},
            "workspace": {"prd_path": "docs/products/workspace/PRD.md"},
            "foundry": {"prd_path": "docs/products/foundry/PRD.md"},
            "cloud": {"prd_path": "docs/products/cloud/PRD.md"},
            "search": {"prd_path": "docs/products/search/PRD.md"},
            "ads-analytics": {"prd_path": "docs/products/ads-analytics/PRD.md"}
        }
    }"#;

    #[test]
    fn product_index_passes_on_complete_inputs() {
        let report = validate_product_index_strings(VALID_README, VALID_CATALOG, |_| true)
            .expect("valid inputs must pass");
        assert_eq!(report.axis_rows, 7);
        assert_eq!(report.catalog_products, 6);
    }

    #[test]
    fn product_index_rejects_missing_section_header() {
        let readme = "no axis section here";
        let error = validate_product_index_strings(readme, VALID_CATALOG, |_| true)
            .expect_err("missing header must fail");
        assert!(error.contains("axis section header"));
    }

    #[test]
    fn product_index_rejects_missing_axis_row() {
        let readme = VALID_README.replace("foundry/PRD.md", "other/PRD.md");
        let error = validate_product_index_strings(&readme, VALID_CATALOG, |_| true)
            .expect_err("missing Foundry row must fail");
        assert!(error.contains("Foundry"));
    }

    #[test]
    fn product_index_rejects_missing_catalog_prd() {
        let catalog = VALID_CATALOG.replace("\"foundry\"", "\"foundry-orphan\"");
        let error = validate_product_index_strings(VALID_README, &catalog, |_| true)
            .expect_err("missing foundry product must fail");
        assert!(error.contains("foundry"));
    }

    #[test]
    fn product_index_rejects_missing_prd_file() {
        let error = validate_product_index_strings(VALID_README, VALID_CATALOG, |_| false)
            .expect_err("missing PRD files must fail");
        assert!(error.contains("missing PRDs"));
    }
}
