use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use pipeline_repository::{EvidenceDigest, SnapshotFailure, WorkControl};

use crate::tool::digest_file;

pub(crate) struct CommandOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct GitCommandRunner {
    executable: PathBuf,
    executable_digest: Option<EvidenceDigest>,
    root: PathBuf,
    invocations: Arc<AtomicU64>,
}

impl GitCommandRunner {
    pub(crate) fn new(executable: PathBuf, root: PathBuf) -> Self {
        Self {
            executable,
            executable_digest: None,
            root,
            invocations: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(crate) fn pin_executable(mut self, digest: EvidenceDigest) -> Self {
        self.executable_digest = Some(digest);
        self
    }

    pub(crate) fn executable(&self) -> &Path {
        &self.executable
    }

    pub(crate) fn invocations(&self) -> u64 {
        self.invocations.load(Ordering::Relaxed)
    }

    pub(crate) fn run(
        &self,
        operation: &'static str,
        arguments: &[OsString],
        stdin: Vec<u8>,
        stdout_limit: u64,
        stderr_limit: u64,
        control: &dyn WorkControl,
    ) -> Result<CommandOutput, SnapshotFailure> {
        control.checkpoint()?;
        self.verify_executable(control)?;
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.root)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .env("PATH", "/usr/local/bin:/usr/bin:/bin")
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0");
        self.invocations.fetch_add(1, Ordering::Relaxed);
        let mut child = command.spawn().map_err(|error| {
            SnapshotFailure::ToolUnavailable(format!(
                "spawn {} for {operation}: {error}",
                self.executable.display()
            ))
        })?;

        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| SnapshotFailure::MalformedOutput("missing Git stdin pipe".to_owned()))?;
        let child_stdout = child.stdout.take().ok_or_else(|| {
            SnapshotFailure::MalformedOutput("missing Git stdout pipe".to_owned())
        })?;
        let child_stderr = child.stderr.take().ok_or_else(|| {
            SnapshotFailure::MalformedOutput("missing Git stderr pipe".to_owned())
        })?;

        let (worker_failure_sender, worker_failure_receiver) = mpsc::channel();
        let output_failure_sender = worker_failure_sender.clone();
        let input = thread::spawn(move || write_input(child_stdin, &stdin));
        let output = thread::spawn(move || {
            read_bounded(
                child_stdout,
                stdout_limit,
                "stdout bytes",
                Some(&output_failure_sender),
            )
        });
        let errors = thread::spawn(move || {
            read_bounded(
                child_stderr,
                stderr_limit,
                "stderr bytes",
                Some(&worker_failure_sender),
            )
        });

        let status = wait_for_child(&mut child, control, &worker_failure_receiver);
        let input = join_worker(input, "Git stdin writer")?;
        let stdout = join_worker(output, "Git stdout reader")?;
        let stderr = join_worker(errors, "Git stderr reader")?;
        status?;
        let stdout = stdout?;
        let stderr = stderr?;
        input?;
        let status = child
            .try_wait()
            .map_err(|error| SnapshotFailure::io("read Git exit status", error))?
            .ok_or_else(|| {
                SnapshotFailure::MalformedOutput(
                    "Git child remained live after wait completed".to_owned(),
                )
            })?;
        self.verify_executable(control)?;
        Ok(CommandOutput {
            status,
            stdout,
            stderr,
        })
    }

    fn verify_executable(&self, control: &dyn WorkControl) -> Result<(), SnapshotFailure> {
        let Some(expected) = self.executable_digest else {
            return Ok(());
        };
        let observed = digest_file(&self.executable, control)?;
        if observed == expected {
            Ok(())
        } else {
            Err(SnapshotFailure::ObjectMismatch(
                "Git executable changed after qualification".to_owned(),
            ))
        }
    }
}

fn write_input(mut stdin: impl Write, input: &[u8]) -> Result<(), SnapshotFailure> {
    stdin
        .write_all(input)
        .map_err(|error| SnapshotFailure::io("write Git stdin", error))?;
    stdin
        .flush()
        .map_err(|error| SnapshotFailure::io("flush Git stdin", error))
}

fn read_bounded(
    mut reader: impl Read,
    maximum: u64,
    limit: &'static str,
    failure_sender: Option<&Sender<()>>,
) -> Result<Vec<u8>, SnapshotFailure> {
    let capacity = usize::try_from(maximum.min(64 * 1024)).unwrap_or(64 * 1024);
    let mut output = Vec::with_capacity(capacity);
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = match reader.read(&mut buffer) {
            Ok(read) => read,
            Err(error) => {
                signal_failure(failure_sender);
                return Err(SnapshotFailure::io("read Git output", error));
            }
        };
        if read == 0 {
            return Ok(output);
        }
        let observed = output.len() as u64 + read as u64;
        if observed > maximum {
            signal_failure(failure_sender);
            return Err(SnapshotFailure::LimitExceeded {
                limit,
                maximum,
                observed,
            });
        }
        output.extend_from_slice(&buffer[..read]);
    }
}

fn signal_failure(failure_sender: Option<&Sender<()>>) {
    if let Some(failure_sender) = failure_sender {
        let _ = failure_sender.send(());
    }
}

fn wait_for_child(
    child: &mut Child,
    control: &dyn WorkControl,
    worker_failures: &Receiver<()>,
) -> Result<(), SnapshotFailure> {
    loop {
        if let Err(reason) = control.checkpoint() {
            terminate(child)?;
            return Err(reason);
        }
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(error) => {
                let failure = SnapshotFailure::io("poll Git child", error);
                terminate(child)?;
                return Err(failure);
            }
        }
        match worker_failures.try_recv() {
            Ok(()) => {
                terminate(child)?;
                return Ok(());
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => {}
        }
        thread::sleep(Duration::from_millis(2));
    }
}

fn terminate(child: &mut Child) -> Result<(), SnapshotFailure> {
    match child.kill() {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {}
        Err(error) => return Err(SnapshotFailure::io("terminate Git child", error)),
    }
    child
        .wait()
        .map(|_| ())
        .map_err(|error| SnapshotFailure::io("reap Git child", error))
}

fn join_worker<T>(
    worker: thread::JoinHandle<Result<T, SnapshotFailure>>,
    name: &'static str,
) -> Result<Result<T, SnapshotFailure>, SnapshotFailure> {
    worker.join().map_err(|_| {
        SnapshotFailure::MalformedOutput(format!("{name} panicked before producing a result"))
    })
}

pub(crate) fn require_success(
    operation: &'static str,
    output: CommandOutput,
) -> Result<Vec<u8>, SnapshotFailure> {
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(SnapshotFailure::ToolFailed {
            operation,
            status: output.status.code(),
            stderr: bounded_lossy(&output.stderr),
        })
    }
}

pub(crate) fn bounded_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::time::{Duration, Instant};

    use pipeline_repository::NoCancellation;

    use super::*;

    #[test]
    fn deadline_terminates_and_reaps_the_child() {
        let root = std::env::current_dir().unwrap();
        let runner = GitCommandRunner::new(PathBuf::from("/bin/sleep"), root);
        let control = NoCancellation::until(Instant::now() + Duration::from_millis(20));
        let started = Instant::now();
        let result = runner.run(
            "deadline fixture",
            &[OsString::from("5")],
            Vec::new(),
            1024,
            1024,
            &control,
        );

        assert!(matches!(result, Err(SnapshotFailure::DeadlineExceeded)));
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(runner.invocations(), 1);
    }

    #[test]
    fn bounded_reader_refuses_before_retaining_excess_output() {
        let result = read_bounded(Cursor::new(b"abc"), 2, "stdout bytes", None);

        assert!(matches!(
            result,
            Err(SnapshotFailure::LimitExceeded {
                limit: "stdout bytes",
                maximum: 2,
                observed: 3,
            })
        ));
    }

    #[test]
    fn stdout_limit_terminates_a_child_that_ignores_the_closed_pipe() {
        let root = std::env::current_dir().unwrap();
        let runner = GitCommandRunner::new(PathBuf::from("/bin/sh"), root);
        let control = NoCancellation::until(Instant::now() + Duration::from_secs(2));
        let started = Instant::now();
        let result = runner.run(
            "stdout limit fixture",
            &[
                OsString::from("-c"),
                OsString::from(
                    "trap '' PIPE; while :; do printf '0123456789abcdef' || :; done 2>/dev/null",
                ),
            ],
            Vec::new(),
            1024,
            1024,
            &control,
        );

        assert!(matches!(
            result,
            Err(SnapshotFailure::LimitExceeded {
                limit: "stdout bytes",
                maximum: 1024,
                ..
            })
        ));
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
