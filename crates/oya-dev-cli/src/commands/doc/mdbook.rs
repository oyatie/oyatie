use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_mdbook_kernel::{validate_mdbook_source, MdbookSourceFile};

use crate::command_output::OutputFormat as DevCheckOutputFormat;
use crate::slash_path;

pub(super) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match parse_doc_mdbook_args(args, usage) {
        Ok(args) => run_doc_mdbook(args),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DocMdbookArgs {
    site_dir: PathBuf,
    output_format: DevCheckOutputFormat,
}

fn parse_doc_mdbook_args(args: Vec<String>, usage: &str) -> Result<DocMdbookArgs, String> {
    let mut parsed = DocMdbookArgs {
        site_dir: PathBuf::from("docs/site"),
        output_format: DevCheckOutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--site-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.site_dir = PathBuf::from(value);
            }
            "--format" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output_format =
                    DevCheckOutputFormat::parse(&value).ok_or_else(|| usage.to_owned())?;
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn run_doc_mdbook(args: DocMdbookArgs) -> ExitCode {
    match run_doc_mdbook_result(&args.site_dir) {
        Ok(report) => match args.output_format {
            DevCheckOutputFormat::Text => {
                println!(
                    "mdbook source validation passed: {} files, {} chapters, {} local links",
                    report.source_files_checked,
                    report.chapters_checked,
                    report.local_links_checked
                );
                ExitCode::SUCCESS
            }
            DevCheckOutputFormat::Json => {
                println!(
                    "{{\"command\":\"oya doc mdbook\",\"status\":\"passed\",\"files\":{},\"chapters\":{},\"local_links\":{}}}",
                    report.source_files_checked,
                    report.chapters_checked,
                    report.local_links_checked
                );
                ExitCode::SUCCESS
            }
        },
        Err(message) => {
            eprintln!("mdbook source validation failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_doc_mdbook_result(
    site_dir: &Path,
) -> Result<oya_foundry_mdbook_kernel::MdbookSourceReport, String> {
    let files = read_mdbook_source_files(site_dir)?;
    validate_mdbook_source(files).map_err(|error| format!("mdbook source invalid: {error:?}"))
}

fn read_mdbook_source_files(site_dir: &Path) -> Result<Vec<MdbookSourceFile>, String> {
    if !site_dir.is_dir() {
        return Err(format!("mdbook site dir missing {}", site_dir.display()));
    }
    let mut files = Vec::new();
    collect_mdbook_source_files(site_dir, site_dir, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_mdbook_source_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<MdbookSourceFile>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| {
            format!(
                "mdbook source dir unreadable {}: {error}",
                current.display()
            )
        })?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("mdbook source dir entry unreadable: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_mdbook_source_files(root, &path, files)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("mdbook source path outside root: {error}"))?;
        let relative = slash_path(relative);
        if relative == "book.toml" || relative.ends_with(".md") {
            let contents = fs::read_to_string(&path).map_err(|error| {
                format!("mdbook source file unreadable {}: {error}", path.display())
            })?;
            files.push(MdbookSourceFile {
                path: relative,
                contents,
            });
        }
    }
    Ok(())
}
