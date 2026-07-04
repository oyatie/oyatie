#![forbid(unsafe_code)]

use kernel_asterinas_boundary::run_soak_from_env;

fn main() {
    match run_soak_from_env() {
        Ok(output) => {
            println!(
                "asterinas soak verdict={} aggregate_receipt_path={} aggregate_receipt_sha256={}",
                output.verdict,
                output.aggregate_receipt_path.display(),
                output.aggregate_receipt_sha256
            );
            for attempt in &output.attempts {
                println!(
                    "asterinas soak attempt_id={} verdict={} clean_boots={} receipt_path={} receipt_sha256={}",
                    attempt.attempt_id,
                    attempt.verdict,
                    attempt.clean_boots,
                    attempt.receipt_path,
                    attempt.receipt_sha256
                );
            }
            if output.verdict != "pass" {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("asterinas soak harness failed: {error}");
            std::process::exit(1);
        }
    }
}
