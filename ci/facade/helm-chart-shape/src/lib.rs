//! Hermetic shape checks over Helm chart directories.
//!
//! WHY THIS IS NOT A `helm template` JOB. Rendering every chart would be the stronger check, but it
//! needs the `helm` binary and therefore inline shell in workflow YAML — which this repository
//! retires on a shrink-only ratchet
//! (`rust_first_automation_unbaselined_workflow_inline_shell`: "productize it as a Rust/Buck2
//! step"). There is also no precedent for invoking helm in CI at all. So this checks the shape
//! that caused the real defect, in pure Rust, inside the required `cargo test --workspace` job.
//!
//! WHAT IT DOES AND DOES NOT CATCH, stated plainly: it catches a non-manifest file under
//! `templates/`, which is the class that broke `intelligence`. It does NOT catch template syntax
//! errors, missing required values, or unvendored chart dependencies — a full render would. This is
//! a floor, not a substitute, and the remaining coverage needs a productized Rust renderer.
#![forbid(unsafe_code)]

/// A file under a chart's `templates/` directory that Helm would try to parse as a manifest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub chart: String,
    pub path: String,
    pub reason: String,
}

/// Extensions Helm legitimately reads under `templates/`.
///
/// `.tpl` holds named-template partials; `.txt` is the NOTES convention. Everything else is either
/// a manifest (`.yaml`/`.yml`) or does not belong there.
const RENDERABLE: &[&str] = &["yaml", "yml", "tpl", "txt"];

/// Files Helm ignores by convention even inside `templates/`.
fn is_ignored_by_convention(file_name: &str) -> bool {
    file_name.starts_with('.') || file_name.starts_with('_')
}

/// Would Helm try to parse this `templates/` entry as a manifest?
///
/// `helmignore_entries` are the literal lines of the chart's `.helmignore`, which is how a
/// deliberately non-renderable file (a BUCK build file that must stay for Buck wiring) is kept out
/// of Helm's way without deleting it.
#[must_use]
pub fn template_file_finding(
    chart: &str,
    relative_path: &str,
    helmignore_entries: &[String],
) -> Option<Finding> {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    if is_ignored_by_convention(file_name) {
        return None;
    }
    if helmignore_entries
        .iter()
        .any(|entry| entry == file_name || entry == relative_path)
    {
        return None;
    }
    let extension = file_name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase());
    match extension {
        Some(ext) if RENDERABLE.contains(&ext.as_str()) => None,
        _ => Some(Finding {
            chart: chart.to_string(),
            path: relative_path.to_string(),
            reason: format!(
                "Helm parses every file under templates/ as a manifest; {file_name} is not a \
                 renderable template. Either move it out of templates/ or list it in .helmignore."
            ),
        }),
    }
}

/// The meaningful lines of a `.helmignore` (comments and blanks dropped).
#[must_use]
pub fn helmignore_entries(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buck_file_under_templates_is_flagged() {
        // The exact defect: intelligence/iac/k8s/helm/templates/BUCK.
        let f = template_file_finding("intelligence", "BUCK", &[]);
        assert!(f.is_some());
        assert!(f.unwrap().reason.contains(".helmignore"));
    }

    #[test]
    fn a_helmignored_file_is_accepted_so_deliberate_buck_wiring_can_stay() {
        assert!(template_file_finding("intelligence", "BUCK", &["BUCK".to_string()]).is_none());
    }

    #[test]
    fn ordinary_templates_and_partials_are_accepted() {
        for name in [
            "deployment.yaml",
            "service.yml",
            "_helpers.tpl",
            "NOTES.txt",
        ] {
            assert!(
                template_file_finding("c", name, &[]).is_none(),
                "{name} must be accepted"
            );
        }
    }

    #[test]
    fn dotfiles_and_underscore_partials_are_ignored_by_convention() {
        assert!(template_file_finding("c", ".gitkeep", &[]).is_none());
        assert!(template_file_finding("c", "_partial", &[]).is_none());
    }

    #[test]
    fn an_extensionless_or_odd_file_is_flagged() {
        assert!(template_file_finding("c", "README", &[]).is_some());
        assert!(template_file_finding("c", "script.sh", &[]).is_some());
    }

    #[test]
    fn helmignore_parsing_drops_comments_and_blanks() {
        assert_eq!(
            helmignore_entries("# note\n\nBUCK\n  \nx.txt\n"),
            ["BUCK", "x.txt"]
        );
    }
}

/// Services still carrying their own chart, parsed from the frozen list.
#[must_use]
pub fn bespoke_charts(json: &str) -> Vec<String> {
    json.lines()
        .skip_while(|l| !l.contains("\"bespoke_charts\""))
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with(']'))
        .filter_map(|l| {
            let t = l.trim().trim_end_matches(',');
            t.strip_prefix('"')?.strip_suffix('"').map(str::to_string)
        })
        .collect()
}

#[cfg(test)]
mod bespoke_tests {
    use super::*;

    #[test]
    fn the_frozen_bespoke_list_parses() {
        let json = "{\n  \"bespoke_charts\": [\n    \"a/b\",\n    \"c\"\n  ]\n}";
        assert_eq!(bespoke_charts(json), ["a/b", "c"]);
    }

    #[test]
    fn an_empty_list_parses_as_empty_not_as_everything() {
        assert!(bespoke_charts("{\n  \"bespoke_charts\": [\n  ]\n}").is_empty());
    }
}
