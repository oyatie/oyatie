//! Generate first-party Buck2 Rust targets from local Cargo manifests.
//!
//! This is the Rust/Buck2-owned replacement for the retired Python first-party
//! BUCK generator. It is intentionally conservative: existing BUCK files are
//! skipped unless `--force` is passed, generated/vendored trees are ignored, and
//! only manifest shapes used by first-party Oyatie crates are materialized.

use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

const SKIP_PREFIXES: &[&str] = &[
    "third-party/",
    "buck-out/",
    "tools/agent-skills/",
    "target/",
    ".git/",
];
const GENERATED_SRC_GLOB: &str = "glob([\"src/**/*.rs\", \"migrations/**/*.sql\", \"**/*.cedar\", \"**/*.sql\", \"**/*.json\", \"**/*.toml\", \"**/*.yaml\", \"**/*.yml\", \"**/*.proto\", \"**/*.graphql\", \"**/*.html\", \"**/*.css\", \"**/*.txt\"])";
const BUILDSCRIPT_OVERRIDES: &[(&str, bool)] = &[
    ("oya-shared-backbone-grpc-generated-adapter", true),
    ("oya-identity-workload-rest", true),
    ("oya-intelligence-supervisor-app", false),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    repo_root: PathBuf,
    subsystem: Option<PathBuf>,
    dry_run: bool,
    force: bool,
    version: bool,
    help: bool,
}

impl Options {
    fn default_with_repo_root(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            subsystem: None,
            dry_run: false,
            force: false,
            version: false,
            help: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub req: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryTarget {
    pub name: String,
    pub crate_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestModel {
    pub package_name: String,
    pub edition: String,
    pub lib_crate_root: Option<String>,
    pub proc_macro: bool,
    pub bins: Vec<BinaryTarget>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThirdPartyResolver {
    target_names: Vec<String>,
    public_aliases: HashSet<String>,
}

impl ThirdPartyResolver {
    fn empty() -> Self {
        Self {
            target_names: Vec::new(),
            public_aliases: HashSet::new(),
        }
    }

    pub fn from_buck_text(text: &str) -> Self {
        let target_names = extract_buck_names(text)
            .into_iter()
            .filter(|name| !name.ends_with(".crate") && !name.contains("build-script"))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let public_aliases = alias_blocks(text)
            .into_iter()
            .filter(|block| block.contains("visibility") && block.contains("PUBLIC"))
            .filter_map(|block| first_buck_name(block))
            .collect::<HashSet<_>>();
        Self {
            target_names,
            public_aliases,
        }
    }

    pub fn load(repo_root: &Path) -> Self {
        fs::read_to_string(repo_root.join("third-party/BUCK"))
            .map(|text| Self::from_buck_text(&text))
            .unwrap_or_else(|_| Self::empty())
    }

    pub fn resolve(&self, crate_name: &str, version_req: &str) -> Option<String> {
        let normalized = crate_name.replace('_', "-");
        let target_set = self.target_names.iter().collect::<HashSet<_>>();
        let major = first_digit_run(version_req);

        for candidate in [&normalized, crate_name] {
            if self.public_aliases.contains(candidate) {
                return Some(candidate.to_string());
            }
        }

        if let Some(major) = major {
            for candidate in [
                format!("{normalized}-{major}"),
                format!("{crate_name}-{major}"),
            ] {
                if target_set.contains(&candidate) {
                    return Some(candidate);
                }
            }
        }

        for candidate in [&normalized, crate_name] {
            if target_set.contains(&candidate.to_string()) {
                return Some(candidate.to_string());
            }
        }

        let mut matches = self
            .target_names
            .iter()
            .filter(|name| {
                *name == &normalized
                    || name.starts_with(&format!("{normalized}-"))
                    || *name == crate_name
                    || name.starts_with(&format!("{crate_name}-"))
            })
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        matches
            .iter()
            .find(|name| name.chars().any(|ch| ch.is_ascii_digit()))
            .cloned()
            .or_else(|| matches.first().cloned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub rel_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub generated_files: usize,
    pub skipped_existing: usize,
    pub skipped_no_targets: usize,
    pub errors: Vec<String>,
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Package,
    Dependencies,
    DevDependencies,
    BuildDependencies,
    Lib,
    Bin,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PartialBin {
    name: Option<String>,
    path: Option<String>,
}

impl PartialBin {
    fn into_target(self, package_name: &str) -> BinaryTarget {
        BinaryTarget {
            name: self.name.unwrap_or_else(|| package_name.to_string()),
            crate_root: self.path.unwrap_or_else(|| "src/main.rs".to_string()),
        }
    }
}

fn usage(program: &str) -> String {
    format!(
        "\
Usage: {program} [--repo-root PATH] [--subsystem DIR] [--dry-run] [--force]\n\n\
Generate first-party Buck2 rust_library, rust_test, and rust_binary rules\n\
from local Cargo manifests. Existing BUCK files are skipped unless --force\n\
is set. Cargo metadata is treated as an input format only; Buck2 remains the\n\
build/test/check authority.\n\n\
Options:\n\
  --repo-root PATH   Repository root to scan (default: current directory)\n\
  --subsystem DIR    Restrict scan to a repository-relative path prefix\n\
  --dry-run          Print candidate BUCK content without writing files\n\
  --force            Overwrite existing first-party BUCK files\n\
  --version          Print tool identity and exit\n\
  --help             Print this help text and exit\n"
    )
}

fn default_repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing {flag} value"))
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default_with_repo_root(default_repo_root());
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--repo-root" => {
                options.repo_root = PathBuf::from(take_value(args, &mut index, "--repo-root")?)
            }
            "--subsystem" => {
                options.subsystem =
                    Some(PathBuf::from(take_value(args, &mut index, "--subsystem")?));
            }
            "--dry-run" => options.dry_run = true,
            "--force" => options.force = true,
            "--version" => options.version = true,
            "--help" | "-h" => options.help = true,
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
        index += 1;
    }
    Ok(options)
}

fn validate_relative_scan_path(path: &Path) -> Result<(), String> {
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "subsystem must be repository-relative and cannot escape the repo: {}",
            path.display()
        ));
    }
    Ok(())
}

fn normalize_rel(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn clean_join(base: &Path, child: &str) -> PathBuf {
    let mut result = base.to_path_buf();
    for component in Path::new(child).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(part) => result.push(part),
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    result
}

fn is_skipped_rel(rel: &str) -> bool {
    SKIP_PREFIXES
        .iter()
        .any(|prefix| rel == prefix.trim_end_matches('/') || rel.starts_with(prefix))
}

fn collect_manifests(root: &Path, rel: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let abs = root.join(rel);
    let mut entries = fs::read_dir(&abs)
        .map_err(|error| format!("read_dir {} failed: {error}", abs.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read_dir {} failed: {error}", abs.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file_type {} failed: {error}", entry.path().display()))?;
        let entry_rel = rel.join(entry.file_name());
        let normalized = normalize_rel(&entry_rel);
        if is_skipped_rel(&normalized) {
            continue;
        }
        if file_type.is_dir() {
            collect_manifests(root, &entry_rel, files)?;
        } else if file_type.is_file() && entry.file_name() == "Cargo.toml" {
            files.push(entry_rel);
        }
    }
    Ok(())
}

fn line_without_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let line = line_without_comment(line);
    if line.is_empty() || line.starts_with('[') {
        return None;
    }
    let (key, value) = line.split_once('=')?;
    Some((key.trim().trim_matches('"'), value.trim()))
}

fn quoted_value(value: &str) -> Option<String> {
    let value = value.trim();
    let rest = value.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn bool_value(value: &str) -> Option<bool> {
    match value.trim().trim_end_matches(',') {
        "true" | "True" => Some(true),
        "false" | "False" => Some(false),
        _ => None,
    }
}

fn inline_attr(value: &str, attr: &str) -> Option<String> {
    let trimmed = value.trim();
    let body = trimmed.strip_prefix('{')?.trim_end_matches('}');
    for part in body.split(',') {
        let (key, value) = part.split_once('=')?;
        if key.trim() == attr {
            return quoted_value(value.trim());
        }
    }
    None
}

fn first_digit_run(value: &str) -> Option<String> {
    let mut run = String::new();
    let mut started = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            started = true;
            run.push(ch);
        } else if started {
            break;
        }
    }
    if run.is_empty() { None } else { Some(run) }
}

fn crate_name_to_ident(name: &str) -> String {
    name.replace('-', "_")
}

fn buildscript_override(package_name: &str) -> Option<bool> {
    BUILDSCRIPT_OVERRIDES
        .iter()
        .find_map(|(name, is_proto)| (*name == package_name).then_some(*is_proto))
}

fn is_dependency_section(section: Section) -> bool {
    matches!(
        section,
        Section::Dependencies | Section::DevDependencies | Section::BuildDependencies
    )
}

fn dependency_from_line(key: &str, value: &str) -> Dependency {
    let req = quoted_value(value)
        .or_else(|| inline_attr(value, "version"))
        .unwrap_or_else(|| "*".to_string());
    Dependency {
        name: key.to_string(),
        req,
        path: inline_attr(value, "path"),
    }
}

pub fn parse_manifest(text: &str, manifest_dir: &Path) -> ManifestModel {
    let mut section = Section::None;
    let mut package_name = manifest_dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "crate".to_string());
    let mut edition = "2024".to_string();
    let mut lib_declared = false;
    let mut lib_path = None;
    let mut proc_macro = false;
    let mut bins = Vec::new();
    let mut current_bin: Option<PartialBin> = None;
    let mut dependencies = Vec::new();

    for raw_line in text.lines() {
        let line = line_without_comment(raw_line);
        if line.is_empty() {
            continue;
        }
        match line {
            "[package]" => section = Section::Package,
            "[dependencies]" => section = Section::Dependencies,
            "[dev-dependencies]" => section = Section::DevDependencies,
            "[build-dependencies]" => section = Section::BuildDependencies,
            "[lib]" => {
                section = Section::Lib;
                lib_declared = true;
            }
            "[[bin]]" => {
                if let Some(bin) = current_bin.take() {
                    bins.push(bin.into_target(&package_name));
                }
                section = Section::Bin;
                current_bin = Some(PartialBin {
                    name: None,
                    path: None,
                });
            }
            _ if line.starts_with('[') => section = Section::Other,
            _ => {}
        }

        let Some((key, value)) = split_key_value(line) else {
            continue;
        };
        match section {
            Section::Package => match key {
                "name" => {
                    if let Some(value) = quoted_value(value) {
                        package_name = value;
                    }
                }
                "edition" => {
                    if let Some(value) = quoted_value(value) {
                        edition = value;
                    }
                }
                _ => {}
            },
            Section::Lib => match key {
                "path" => lib_path = quoted_value(value),
                "proc-macro" => proc_macro = bool_value(value).unwrap_or(false),
                "crate-type" if value.contains("proc-macro") => proc_macro = true,
                _ => {}
            },
            Section::Bin => {
                let bin = current_bin.get_or_insert(PartialBin {
                    name: None,
                    path: None,
                });
                match key {
                    "name" => bin.name = quoted_value(value),
                    "path" => bin.path = quoted_value(value),
                    _ => {}
                }
            }
            section if is_dependency_section(section) => {
                dependencies.push(dependency_from_line(key, value))
            }
            _ => {}
        }
    }
    if let Some(bin) = current_bin.take() {
        bins.push(bin.into_target(&package_name));
    }

    ManifestModel {
        package_name,
        edition,
        lib_crate_root: if lib_declared {
            Some(lib_path.unwrap_or_else(|| "src/lib.rs".to_string()))
        } else {
            None
        },
        proc_macro,
        bins,
        dependencies,
    }
}

fn extract_buck_names(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let after_name = trimmed.strip_prefix("name")?;
            if !after_name.starts_with('=')
                && !after_name.chars().next().is_some_and(char::is_whitespace)
            {
                return None;
            }
            let (_, value) = trimmed.split_once('=')?;
            quoted_value(value)
        })
        .collect()
}

fn first_buck_name(text: &str) -> Option<String> {
    extract_buck_names(text).into_iter().next()
}

fn alias_blocks(text: &str) -> Vec<&str> {
    rule_blocks(text, "alias")
}

fn rule_blocks<'a>(text: &'a str, rule: &str) -> Vec<&'a str> {
    let needle = format!("{rule}(");
    let mut blocks = Vec::new();
    let mut search_offset = 0;
    while let Some(relative_start) = text[search_offset..].find(&needle) {
        let start = search_offset + relative_start;
        let line_start = text[..start].rfind('\n').map_or(0, |position| position + 1);
        if !text[line_start..start].trim().is_empty() {
            search_offset = start + needle.len();
            continue;
        }

        let mut depth = 0i32;
        let mut end = None;
        for (offset, ch) in text[start..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + offset + ch.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(end) = end {
            blocks.push(&text[start..end]);
            search_offset = end;
        } else {
            break;
        }
    }
    blocks
}

fn render_deps(
    dependencies: &[Dependency],
    manifest_rel_dir: &Path,
    resolver: &ThirdPartyResolver,
) -> Vec<String> {
    let mut deps = dependencies
        .iter()
        .map(|dep| {
            if let Some(path) = &dep.path {
                let dep_rel = clean_join(manifest_rel_dir, path);
                format!("        \"//{}:{}\",", normalize_rel(&dep_rel), dep.name)
            } else if let Some(target) = resolver.resolve(&dep.name, &dep.req) {
                format!("        \"third-party//:{target}\",")
            } else {
                format!("        # UNRESOLVED: {} {}", dep.name, dep.req)
            }
        })
        .collect::<Vec<_>>();
    deps.sort();
    deps.dedup();
    deps
}

fn render_proto_buildscript(package_name: &str, edition: &str) -> Vec<String> {
    let mut lines = vec![
        "load(\"@prelude//rust:cargo_buildscript.bzl\", \"buildscript_run\")".to_string(),
        String::new(),
        "rust_binary(".to_string(),
        format!("    name = \"{package_name}-build-script\","),
        "    srcs = [\"build.rs\"],".to_string(),
        "    crate = \"build_script_build\",".to_string(),
        "    crate_root = \"build.rs\",".to_string(),
    ];
    if edition != "2024" {
        lines.push(format!("    edition = \"{edition}\","));
    }
    lines.extend([
        "    visibility = [],".to_string(),
        "    deps = [".to_string(),
        "        \"third-party//:protoc-bin-vendored-3\",".to_string(),
        "        \"third-party//:tonic-prost-build-0.14\",".to_string(),
        "    ],".to_string(),
        ")".to_string(),
        String::new(),
        "buildscript_run(".to_string(),
        format!("    name = \"{package_name}-build-script-run\","),
        format!("    script = \":{package_name}-build-script\","),
        ")".to_string(),
        String::new(),
    ]);
    lines
}

fn render_rust_rule(
    rule: &str,
    rule_name: &str,
    crate_root: &str,
    edition: &str,
    crate_ident: Option<&str>,
    proc_macro: bool,
    extra_attrs: &[String],
    deps: &[String],
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("{rule}("));
    lines.push(format!("    name = \"{rule_name}\","));
    lines.push(format!("    srcs = {GENERATED_SRC_GLOB},"));
    if matches!(rule, "rust_library" | "rust_test") {
        let crate_name = crate_ident
            .map(str::to_string)
            .unwrap_or_else(|| crate_name_to_ident(rule_name));
        lines.push(format!("    crate = \"{crate_name}\","));
    }
    lines.push(format!("    crate_root = \"{crate_root}\","));
    if edition != "2024" {
        lines.push(format!("    edition = \"{edition}\","));
    }
    if proc_macro && rule == "rust_library" {
        lines.push("    proc_macro = True,".to_string());
    }
    lines.extend(extra_attrs.iter().map(|attr| format!("    {attr}")));
    lines.push("    visibility = [\"PUBLIC\"],".to_string());
    if !deps.is_empty() {
        lines.push("    deps = [".to_string());
        lines.extend(deps.iter().cloned());
        lines.push("    ],".to_string());
    }
    lines.push(")".to_string());
    lines
}

pub fn render_buck_content(
    model: &ManifestModel,
    manifest_rel_dir: &Path,
    _repo_root: &Path,
    resolver: &ThirdPartyResolver,
    lib_exists: bool,
    main_exists: bool,
) -> Option<String> {
    let deps = render_deps(&model.dependencies, manifest_rel_dir, resolver);
    let mut lines = Vec::new();
    let buildscript_is_proto = buildscript_override(&model.package_name).unwrap_or(false);
    if buildscript_is_proto {
        lines.extend(render_proto_buildscript(
            &model.package_name,
            &model.edition,
        ));
    }
    let lib_crate_root = model
        .lib_crate_root
        .clone()
        .or_else(|| lib_exists.then(|| "src/lib.rs".to_string()));

    if let Some(crate_root) = &lib_crate_root {
        lines.extend(render_rust_rule(
            "rust_library",
            &model.package_name,
            crate_root,
            &model.edition,
            Some(&crate_name_to_ident(&model.package_name)),
            model.proc_macro,
            &buildscript_is_proto
                .then(|| {
                    format!(
                        "env = {{\"OUT_DIR\": \"$(location :{}-build-script-run[out_dir])\"}},",
                        model.package_name
                    )
                })
                .into_iter()
                .collect::<Vec<_>>(),
            &deps,
        ));
        if !model.proc_macro {
            lines.push(String::new());
            lines.extend(render_rust_rule(
                "rust_test",
                &format!("{}-unittest", model.package_name),
                crate_root,
                &model.edition,
                Some(&crate_name_to_ident(&model.package_name)),
                false,
                &buildscript_is_proto
                    .then(|| {
                        format!(
                            "env = {{\"OUT_DIR\": \"$(location :{}-build-script-run[out_dir])\"}},",
                            model.package_name
                        )
                    })
                    .into_iter()
                    .collect::<Vec<_>>(),
                &deps,
            ));
        }
    }

    let mut bins = model.bins.clone();
    if bins.is_empty() && main_exists {
        bins.push(BinaryTarget {
            name: model.package_name.clone(),
            crate_root: "src/main.rs".to_string(),
        });
    }

    for bin in bins {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        let buck_bin_name = if lib_crate_root.is_some() && bin.name == model.package_name {
            format!("{}-bin", bin.name)
        } else {
            bin.name.clone()
        };
        let mut bin_deps = Vec::new();
        if lib_crate_root.is_some() {
            bin_deps.push(format!(
                "        \"//{}:{}\",",
                normalize_rel(manifest_rel_dir),
                model.package_name
            ));
        }
        bin_deps.extend(deps.clone());
        lines.extend(render_rust_rule(
            "rust_binary",
            &buck_bin_name,
            &bin.crate_root,
            &model.edition,
            None,
            false,
            &[],
            &bin_deps,
        ));
    }

    if lines.is_empty() {
        None
    } else {
        Some(format!("{}\n", lines.join("\n")))
    }
}

fn generate_for_manifest(
    repo_root: &Path,
    manifest_rel: &Path,
    resolver: &ThirdPartyResolver,
    force: bool,
) -> Result<Option<String>, String> {
    let manifest_abs = repo_root.join(manifest_rel);
    let manifest_dir = manifest_rel.parent().unwrap_or_else(|| Path::new(""));
    let crate_abs_dir = manifest_abs.parent().unwrap_or(repo_root);
    let buck_abs = crate_abs_dir.join("BUCK");
    if buck_abs.exists() && !force {
        return Ok(None);
    }
    let text = fs::read_to_string(&manifest_abs)
        .map_err(|error| format!("read {} failed: {error}", manifest_abs.display()))?;
    let mut model = parse_manifest(&text, manifest_dir);
    let lib_exists = crate_abs_dir.join("src/lib.rs").is_file();
    let main_exists = crate_abs_dir.join("src/main.rs").is_file();
    if model.lib_crate_root.is_none() && lib_exists {
        model.lib_crate_root = Some("src/lib.rs".to_string());
    }
    Ok(render_buck_content(
        &model,
        manifest_dir,
        repo_root,
        resolver,
        lib_exists,
        main_exists,
    ))
}

fn run(options: &Options) -> Result<RunSummary, String> {
    let base_rel = options
        .subsystem
        .as_deref()
        .unwrap_or_else(|| Path::new(""));
    validate_relative_scan_path(base_rel)?;

    let mut manifests = Vec::new();
    collect_manifests(&options.repo_root, base_rel, &mut manifests)?;
    let resolver = ThirdPartyResolver::load(&options.repo_root);

    let mut summary = RunSummary {
        generated_files: 0,
        skipped_existing: 0,
        skipped_no_targets: 0,
        errors: Vec::new(),
        changes: Vec::new(),
    };

    for manifest_rel in manifests {
        let manifest_dir = manifest_rel.parent().unwrap_or_else(|| Path::new(""));
        match generate_for_manifest(&options.repo_root, &manifest_rel, &resolver, options.force) {
            Ok(Some(content)) => {
                let buck_rel = manifest_dir.join("BUCK");
                if options.dry_run {
                    println!("[DRY-RUN] Would write {}", normalize_rel(&buck_rel));
                    println!("{content}---");
                } else {
                    fs::write(options.repo_root.join(&buck_rel), content).map_err(|error| {
                        format!("write {} failed: {error}", normalize_rel(&buck_rel))
                    })?;
                    println!("  wrote {}", normalize_rel(&buck_rel));
                }
                summary.generated_files += 1;
                summary.changes.push(FileChange {
                    rel_path: normalize_rel(&buck_rel),
                });
            }
            Ok(None) => {
                if options.repo_root.join(manifest_dir).join("BUCK").exists() && !options.force {
                    summary.skipped_existing += 1;
                } else {
                    summary.skipped_no_targets += 1;
                }
            }
            Err(error) => summary
                .errors
                .push(format!("{}: {error}", normalize_rel(&manifest_rel))),
        }
    }

    Ok(summary)
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let program = args
        .first()
        .map(String::as_str)
        .unwrap_or("generate-first-party-buck");
    let options = match parse_args(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\n\n{}", usage(program));
            std::process::exit(2);
        }
    };

    if options.help {
        print!("{}", usage(program));
        return;
    }
    if options.version {
        println!("generate-first-party-buck 1.0.0");
        return;
    }

    let summary = match run(&options) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let action = if options.dry_run {
        "would be generated"
    } else {
        "generated"
    };
    eprintln!(
        "Summary: {} BUCK files {action}, {} skipped existing, {} skipped without targets, {} errors.",
        summary.generated_files,
        summary.skipped_existing,
        summary.skipped_no_targets,
        summary.errors.len()
    );
    for error in &summary.errors {
        eprintln!("  ERROR: {error}");
    }
    if !summary.errors.is_empty() {
        std::process::exit(1);
    }
}
