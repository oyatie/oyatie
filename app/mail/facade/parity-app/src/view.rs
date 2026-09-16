use serde_json::Value;

pub fn render(report: &Value) -> Result<(), String> {
    let text = |v: &Value| {
        v.as_str()
            .unwrap_or("")
            .replace('|', "\\|")
            .replace('\n', " ")
    };
    println!("# Stalwart to Oyatie architecture mapping\n");
    println!(
        "Source: [{}]({}/tree/{}).\n",
        text(&report["commit"]),
        text(&report["repository"]),
        text(&report["commit"])
    );
    println!(
        "Source tree: `{}`. Producer: `mail-parity-app`, schema `{}`, executable SHA-256 `{}`.\n",
        text(&report["tree"]),
        report["schema"],
        text(&report["producer_sha256"])
    );
    println!(
        "This is a proposed responsibility mapping derived from the pinned upstream inventory. Coverage of the map does not establish implementation, feature, ergonomics, or maturity parity. Community and Enterprise capabilities are both in scope; the recreated system has no license-based feature gates.\n"
    );
    println!(
        "Destination faces are `app/mail/core`, `app/mail/ports`, `app/mail/adapters`, and `app/mail/facade`, except where the owner column assigns a responsibility to an existing Oyatie product or capability. The named destinations describe responsibilities; they do not claim those crates already exist.\n"
    );
    println!(
        "| Upstream crate | Core | Ports | Adapters | Facade | Ownership |\n|---|---|---|---|---|---|"
    );
    for entry in report["crates"].as_array().ok_or("missing crates")? {
        let m = &entry["mapping"];
        println!(
            "| `{}` | {} | {} | {} | {} | {} |",
            text(&entry["upstream"]),
            text(&m["core"]),
            text(&m["ports"]),
            text(&m["adapters"]),
            text(&m["facade"]),
            text(&m["owner"])
        );
    }
    println!(
        "\nInventory: {} manifests, {} Rust test files, {} Enterprise-related source files, {} fixture/documentation/build inputs.\n",
        report["crates"].as_array().map_or(0, Vec::len),
        report["test_files"].as_array().map_or(0, Vec::len),
        report["enterprise_surfaces"].as_array().map_or(0, Vec::len),
        report["compatibility_inputs"]
            .as_array()
            .map_or(0, Vec::len)
    );
    println!(
        "Mapping complete: `{}`. Full parity: `{}`.\n",
        report["mapping_complete"], report["scope"]["full_parity"]
    );
    Ok(())
}
