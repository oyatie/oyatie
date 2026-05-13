use std::process::ExitCode;

mod adr_index;
mod mdbook;
mod openapi;
mod rustdoc;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("rustdoc") => rustdoc::run(args.collect(), usage),
        Some("adr-index") => adr_index::run(args.collect(), usage),
        Some("mdbook") => mdbook::run(args.collect(), usage),
        Some("openapi") => openapi::run(args.collect(), usage),
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}
