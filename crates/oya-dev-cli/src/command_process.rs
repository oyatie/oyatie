use std::io::{self, Write};

pub(crate) fn replay_process_output(output: &std::process::Output) -> Result<(), String> {
    io::stdout()
        .write_all(&output.stdout)
        .map_err(|error| format!("could not write child stdout: {error}"))?;
    io::stderr()
        .write_all(&output.stderr)
        .map_err(|error| format!("could not write child stderr: {error}"))?;
    Ok(())
}

pub(crate) fn process_status_label(status: &std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("exit code {code}"))
        .unwrap_or_else(|| "signal termination".into())
}
