// :registry-drift gate — committed == regenerated (PHASE-0-FIREWALL-PLAN §5.3).
// Re-runs the producer in --stdout (sandbox) mode and byte-diffs against the committed
// accounting-registry.generated.json. A hand-edit to the generated face fails this test.
// ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn faces_dir(root: &Path) -> PathBuf {
    root.join("cloud/cloud-ci/gates/accounting-registry-producer")
}

/// The committed generated faces and the `--face` name that regenerates each. registry-drift
/// extends across ALL of them: the registry + ttl-policy + the GATE-1 decision-crosswalk +
/// the GATE-4 enforcement-inventory faces + the GO-LIVE gate-baseline (the accepted-debt
/// ratchet). A hand-edit to any one fails this gate. The baseline being byte-diff-protected
/// here is what makes laundering debt into the baseline tamper-evident: a hand-edit to widen
/// the accepted-violation set is itself registry_drift RED.
const FACES: [(&str, &str); 5] = [
    ("accounting-registry.generated.json", "registry"),
    ("ttl-policy.generated.json", "ttl-policy"),
    ("decision-crosswalk.generated.json", "decision-crosswalk"),
    ("enforcement-inventory.generated.json", "enforcement-inventory"),
    ("gate-baseline.generated.json", "baseline"),
];

/// Regenerate each face in-memory (sandbox) and assert it byte-matches the committed face.
#[test]
fn committed_faces_equal_regenerated() {
    let root = repo_root();
    let dir = faces_dir(&root);

    for (file, face) in FACES {
        let committed_path = dir.join(file);
        let committed = fs::read_to_string(&committed_path).unwrap_or_else(|e| {
            panic!(
                "committed face missing at {} ({e}); run the producer to generate it",
                committed_path.display()
            )
        });

        let output = Command::new(env!("CARGO"))
            .arg("run")
            .arg("--quiet")
            .arg("-p")
            .arg("accounting-registry-producer")
            .arg("--")
            .arg("--repo-root")
            .arg(&root)
            .arg("--stdout")
            .arg("--face")
            .arg(face)
            .current_dir(&root)
            .output()
            .expect("run accounting-registry-producer");
        assert!(
            output.status.success(),
            "producer failed for face {face}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let regenerated = String::from_utf8(output.stdout).expect("producer stdout utf8");

        assert_eq!(
            committed, regenerated,
            "REGISTRY DRIFT: committed {file} != regenerated. \
             A generated face was hand-edited, or source changed without re-running the producer. \
             Re-run //cloud/cloud-ci/gates:accounting-registry-producer to regenerate."
        );
    }
}
