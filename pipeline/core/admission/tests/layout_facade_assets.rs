//! A facade crate may carry the static assets it serves; no other face may.

use pipeline_admission::{file_budget_violations, layout_violations};

const NOT_A_DIRECTORY: &str = "must be a directory";
const NOT_A_CRATE_CHILD: &str = "crate content must live under";
const NOT_AN_ASSET: &str = "must use one of these extensions";
const NOT_LOWERCASE: &str = "lowercase";
const NOT_A_RUST_SOURCE: &str = "must be snake_case `.rs` files";
const FORBIDDEN_SUBTREE: &str = "`plan/` is not allowed beneath";
const FROZEN_MARKDOWN: &str = "frozen non-root Markdown";
const OVER_BUDGET: &str = "exceeds the repository 300-line file budget";

/// Every path that fails `check`, so one run names them all.
fn failing(cases: &[&str], check: impl Fn(&str) -> Option<String>) -> Vec<String> {
    cases.iter().filter_map(|path| check(path)).collect()
}

#[test]
fn facade_assets_are_admitted_under_a_facade_crate() {
    let unexpected = failing(
        &[
            "app/application/facade/shell-app/assets/app.css",
            "network/facade/edge-app/assets/fonts/inter.woff2",
            "network/facade/edge-app/assets/logo.svg",
            "network/facade/edge-app/assets/icon.png",
            "network/facade/edge-app/assets/hero.webp",
            "network/facade/edge-app/assets/favicon.ico",
            "network/facade/edge-app/assets/OWNERS",
            "network/facade/edge-app/assets/BUCK",
        ],
        |path| {
            let violations = layout_violations(&[path.to_owned()]);
            (!violations.is_empty()).then(|| format!("{path}: {violations:?}"))
        },
    );
    assert!(
        unexpected.is_empty(),
        "unexpected rejections: {unexpected:#?}"
    );
}

#[test]
fn assets_are_refused_off_the_facade_face_and_off_the_closed_set() {
    let refused = [
        (
            "network/core/query-engine/assets/app.css",
            NOT_A_CRATE_CHILD,
        ),
        ("network/facade/edge-app/Assets/app.css", NOT_A_CRATE_CHILD),
        ("network/facade/edge-app/assets", NOT_A_DIRECTORY),
        ("network/facade/edge-app/assets/foo.rs", NOT_AN_ASSET),
        ("network/facade/edge-app/assets/a.txt", NOT_AN_ASSET),
        (
            "network/facade/edge-app/assets/client-manifest.json",
            NOT_AN_ASSET,
        ),
        ("network/facade/edge-app/assets/App.css", NOT_AN_ASSET),
        ("network/facade/edge-app/assets/x.CSS", NOT_AN_ASSET),
        (
            "network/facade/edge-app/assets/Fonts/x.woff2",
            NOT_LOWERCASE,
        ),
        (
            "network/facade/edge-app/assets/plan/x.css",
            FORBIDDEN_SUBTREE,
        ),
        ("network/facade/edge-app/assets/README.md", FROZEN_MARKDOWN),
        ("network/facade/edge-app/assets/notes.md", FROZEN_MARKDOWN),
        (
            "network/facade/edge-app/src/assets/app.css",
            NOT_A_RUST_SOURCE,
        ),
    ];
    let unexpected: Vec<String> = refused
        .iter()
        .filter_map(|(path, rule)| {
            let violations = layout_violations(&[(*path).to_owned()]);
            let fired = violations.iter().any(|message| message.contains(rule));
            (!fired).then(|| format!("{path}: expected `{rule}`, got {violations:?}"))
        })
        .collect();
    assert!(unexpected.is_empty(), "missing refusals: {unexpected:#?}");
}

#[test]
fn a_large_facade_asset_is_exempt_from_the_file_budget() {
    let css = "a{}\n".repeat(12_600);
    let unexpected = failing(
        &[
            "app/application/facade/shell-app/assets/app.css",
            "network/facade/edge-app/assets/app.css",
        ],
        |path| {
            let violations = file_budget_violations(path, css.as_bytes());
            (!violations.is_empty()).then(|| format!("{path}: {violations:?}"))
        },
    );
    assert!(
        unexpected.is_empty(),
        "unexpected budget refusals: {unexpected:#?}"
    );
}

#[test]
fn the_asset_budget_exemption_is_positional_not_a_prefix() {
    let css = "a{}\n".repeat(12_600);
    let unexpected = failing(
        &[
            "network/core/x/assets/app.css",
            "network/facade/edge-app/src/assets/app.css",
            "network/facade/edge-app/Assets/app.css",
            "base/facade/app/assets/app.css",
            "app/not-a-product/facade/shell-app/assets/app.css",
            "network/facade/edge-app/assets",
        ],
        |path| {
            let violations = file_budget_violations(path, css.as_bytes());
            let fired = violations
                .iter()
                .any(|message| message.contains(OVER_BUDGET));
            (!fired).then(|| format!("{path}: expected `{OVER_BUDGET}`, got {violations:?}"))
        },
    );
    assert!(
        unexpected.is_empty(),
        "missing budget refusals: {unexpected:#?}"
    );
}
