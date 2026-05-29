// Per-milestone exit-checklist renderer dispatch. Keeps the milestone
// identifier out of the parent module name so adding M03/M04/... is a
// single new sub-file, not a parent-module rename.

use std::process::ExitCode;

mod m02;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("m02") => m02::run(args.collect(), usage),
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}
