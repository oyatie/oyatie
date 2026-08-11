//! Skeleton CLI: emit the hermetic A1 ABI-probe scaffold receipt to stdout.
//! Does not spawn QEMU and does not claim Asterinas is the canonical node kernel.

use kernel_asterinas_abi_probe::{qemu_probe_coupling_note, run_scaffold, scaffold_summary_receipt};

fn main() {
    match run_scaffold() {
        Ok(run) => {
            let mut summary = scaffold_summary_receipt(&run);
            if let Some(obj) = summary.as_object_mut() {
                obj.insert("qemu_probe_coupling".into(), qemu_probe_coupling_note());
            }
            println!("{}", serde_json::to_string_pretty(&summary).expect("serialize"));
        }
        Err(e) => {
            eprintln!("asterinas-abi-probe scaffold failed: {e}");
            std::process::exit(1);
        }
    }
}
