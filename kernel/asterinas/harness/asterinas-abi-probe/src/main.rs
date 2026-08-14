//! Local-bridge harness receipt emitter for the A1 ABI-probe scaffold.
//!
//! Emits the hermetic scaffold receipt to stdout. Does not spawn QEMU and does not
//! claim Asterinas is the canonical node kernel. This binary is retirement-marked
//! local-bridge feedback (same class as `asterinas-real-boot` harness bins), not a
//! product CLI capability surface — the library API is the sanctioned automation
//! surface per CLI-retirement policy.

use kernel_asterinas_abi_probe::{qemu_probe_coupling_note, run_scaffold, scaffold_summary_receipt};

fn main() {
    match run_scaffold() {
        Ok(run) => {
            let mut summary = scaffold_summary_receipt(&run);
            if let Some(obj) = summary.as_object_mut() {
                obj.insert("qemu_probe_coupling".into(), qemu_probe_coupling_note());
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).expect("serialize")
            );
        }
        Err(e) => {
            eprintln!("asterinas-abi-probe scaffold failed: {e}");
            std::process::exit(1);
        }
    }
}
