//! Protected-facade controls for the owner-prose compatibility cutover.

mod support;

use std::path::PathBuf;
use std::process::Command;

use pipeline_admission::{
    OwnerProseClaim, OwnerProseClassification, OwnerProseManifest, OwnerProseProducer,
    OwnerProseRepositoryBinding, OwnerProseRevisionBinding, OwnerProseSource, owner_prose_sha256,
};
use support::*;

const LAWS: [&str; 4] = ["ADR.md", "PLAN.md", "PRD.md", "SPEC.md"];

struct MigrationFixture {
    root: PathBuf,
    base: String,
    head: String,
    view: PathBuf,
}

impl MigrationFixture {
    fn new(retained: Option<&str>) -> Self {
        let root = fixture();
        write(
            &root,
            "policy/core/evaluate/Cargo.toml",
            "[package]\nname='policy-evaluate'\nversion='0.1.0'\nedition='2024'\n",
        );
        write(
            &root,
            "policy/core/evaluate/src/lib.rs",
            "pub fn evaluate() -> bool { false }\n",
        );
        let mut source_blobs = Vec::new();
        for name in LAWS {
            let path = format!("policy/{name}");
            let bytes = format!("{name} frozen input\n").into_bytes();
            write(&root, &path, std::str::from_utf8(&bytes).expect("law text"));
            source_blobs.push((path, bytes));
        }
        let base = commit(&root, "base with frozen owner prose");
        for name in LAWS {
            if retained != Some(name) {
                std::fs::remove_file(root.join("policy").join(name)).expect("remove law file");
            }
        }
        let head = commit(&root, "candidate owner prose deletion");
        let binding = OwnerProseRepositoryBinding {
            identity: "https://github.com/oyatie/oyatie.git".to_owned(),
            source: OwnerProseRevisionBinding {
                commit: base.clone(),
                tree: git_text(&root, &["rev-parse", &format!("{base}^{{tree}}")]),
            },
            candidate: OwnerProseRevisionBinding {
                commit: head.clone(),
                tree: git_text(&root, &["rev-parse", &format!("{head}^{{tree}}")]),
            },
        };
        let sources = source_blobs
            .into_iter()
            .map(|(path, bytes)| OwnerProseSource {
                path: path.clone(),
                sha256: owner_prose_sha256(&bytes),
                claims: vec![OwnerProseClaim {
                    id: format!(
                        "{}-classified",
                        path.to_ascii_lowercase().replace(['/', '.'], "-")
                    ),
                    start: 0,
                    end: bytes.len(),
                    sha256: owner_prose_sha256(&bytes),
                    classification: OwnerProseClassification::HistoricalRejected,
                    work_reference: None,
                    projections: Vec::new(),
                }],
            })
            .collect();
        let manifest = OwnerProseManifest {
            schema: "oyatie.owner-prose-classification.v1".to_owned(),
            repository: binding,
            producer: OwnerProseProducer {
                identity: "pipeline-owner-prose-classifier".to_owned(),
                schema: "oyatie.owner-prose-classifier.v1".to_owned(),
            },
            owner: "policy".to_owned(),
            sources,
        };
        let view = root.with_extension("owner-prose-view.json");
        std::fs::write(
            &view,
            serde_json::to_vec_pretty(&manifest).expect("manifest JSON"),
        )
        .expect("write off-tree view");
        Self {
            root,
            base,
            head,
            view,
        }
    }

    fn cleanup(self) {
        let _ = std::fs::remove_file(self.view);
        let _ = std::fs::remove_dir_all(self.root);
    }
}

#[test]
fn ordinary_admission_freezes_and_exact_off_tree_view_authorizes_complete_deletion() {
    let fixture = MigrationFixture::new(None);
    let frozen = admit(&fixture.root, &fixture.base, &fixture.head);
    assert!(!frozen.status.success());
    assert!(String::from_utf8_lossy(&frozen.stderr).contains("frozen non-root Markdown"));

    let admitted = admit_with_view(&fixture.root, &fixture.base, &fixture.head, &fixture.view);
    assert!(
        admitted.status.success(),
        "{}",
        String::from_utf8_lossy(&admitted.stderr)
    );

    let qualified = Command::new(env!("CARGO_BIN_EXE_pipeline-path-layout-app"))
        .current_dir(&fixture.root)
        .args(["qualify-owner-prose", &fixture.base, &fixture.head])
        .arg(&fixture.view)
        .output()
        .expect("run offline qualifier");
    assert!(qualified.status.success());
    let rendered: serde_json::Value =
        serde_json::from_slice(&qualified.stdout).expect("qualified JSON");
    assert_eq!(
        rendered["source_digests"]
            .as_array()
            .expect("source digest array")
            .len(),
        4
    );
    fixture.cleanup();
}

#[test]
fn retaining_each_individual_law_file_refuses_unknown() {
    for retained in LAWS {
        let fixture = MigrationFixture::new(Some(retained));
        let output = admit_with_view(&fixture.root, &fixture.base, &fixture.head, &fixture.view);
        assert!(!output.status.success(), "retained {retained}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("owner prose view Unknown"), "{error}");
        assert!(error.contains(retained), "{error}");
        fixture.cleanup();
    }
}

#[test]
fn missing_relative_and_in_repository_views_refuse_unknown() {
    let fixture = MigrationFixture::new(None);
    let views = [
        PathBuf::from("relative-view.json"),
        fixture.root.join("in-repository.json"),
        fixture.root.with_extension("missing-view.json"),
    ];
    for view in &views {
        if view.starts_with(&fixture.root) {
            std::fs::write(view, std::fs::read(&fixture.view).expect("view bytes"))
                .expect("write in-repository view");
        }
        let output = admit_with_view(&fixture.root, &fixture.base, &fixture.head, view);
        assert!(!output.status.success(), "view {view:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("owner prose view Unknown"));
    }
    fixture.cleanup();
}

#[cfg(unix)]
#[test]
fn symlinked_view_refuses_unknown() {
    use std::os::unix::fs::symlink;

    let fixture = MigrationFixture::new(None);
    let link = fixture.root.with_extension("owner-prose-view-link.json");
    symlink(&fixture.view, &link).expect("view symlink");
    let output = admit_with_view(&fixture.root, &fixture.base, &fixture.head, &link);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("owner prose view Unknown"));
    let _ = std::fs::remove_file(link);
    fixture.cleanup();
}
