use std::ffi::OsString;
use std::process::Command;

const BINARY_ENV: &str = "CARGO_BIN_EXE_dependency-declarations-reconciler-app";

fn process_binary() -> OsString {
    option_env!("CARGO_BIN_EXE_dependency-declarations-reconciler-app")
        .map(OsString::from)
        .or_else(|| std::env::var_os(BINARY_ENV))
        .expect("the build must provide the reconciler process binary")
}

#[test]
fn unqualified_process_refuses_without_claiming_readiness() {
    let output = Command::new(process_binary())
        .output()
        .expect("the reconciler process binary must start");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"dependency declaration process adapter is not qualified; use the typed reconciler API\n"
    );
}
