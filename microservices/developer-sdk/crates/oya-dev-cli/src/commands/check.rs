use std::process::ExitCode;

pub(crate) fn run(_args: Vec<String>, usage: &str) -> ExitCode {
    eprintln!("oya check is retired; use `oya gate validate ...` lanes instead.\n{usage}");
    ExitCode::from(2)
}
