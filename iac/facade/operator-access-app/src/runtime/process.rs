use super::*;

pub(super) static CANCELLED: AtomicBool = AtomicBool::new(false);
pub(super) const LIMIT: usize = 8 * 1024 * 1024;

extern "C" fn cancel(_: libc::c_int) {
    CANCELLED.store(true, Ordering::Relaxed);
}

pub fn install_signal_handlers() -> Result<(), AccessError> {
    // SAFETY: the handler only stores to a lock-free atomic. No allocation, I/O,
    // or Rust unwinding occurs in the signal handler; it lives for the process.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = cancel as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
            if libc::sigaction(signal, &action, std::ptr::null_mut()) != 0 {
                return Err(AccessError::DependencyFailed);
            }
        }
    }
    Ok(())
}

pub(super) fn kill_group(child: &mut Child) {
    if !matches!(child.try_wait(), Ok(None)) {
        return;
    }
    // SAFETY: every child here is spawned into its own process group. This PID
    // remains owned and unreaped until wait(), so it cannot target a reused PID.
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.wait();
}

pub(super) fn read_bounded(mut stream: impl Read) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    let mut bytes = Zeroizing::new(Vec::new());
    stream
        .by_ref()
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| AccessError::DependencyFailed)?;
    if bytes.len() > LIMIT {
        return Err(AccessError::OutputLimit);
    }
    Ok(bytes)
}

pub(super) fn run(
    program: &str,
    args: &[String],
    input: &[u8],
    cleanup: bool,
) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    run_with_timeout(
        program,
        args,
        input,
        cleanup,
        Duration::from_secs(30),
        &CANCELLED,
    )
}

pub(super) fn run_with_timeout(
    program: &str,
    args: &[String],
    input: &[u8],
    cleanup: bool,
    timeout: Duration,
    cancelled: &AtomicBool,
) -> Result<Zeroizing<Vec<u8>>, AccessError> {
    let mut child = Command::new(program)
        .args(args)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| AccessError::DependencyFailed)?;
    let stdin = child.stdin.take().ok_or(AccessError::DependencyFailed)?;
    let stdout = child.stdout.take().ok_or(AccessError::DependencyFailed)?;
    let stderr = child.stderr.take().ok_or(AccessError::DependencyFailed)?;
    let input = Zeroizing::new(input.to_vec());
    let writer = thread::spawn(move || {
        let mut stdin = stdin;
        stdin.write_all(&input)
    });
    let reader = thread::spawn(move || read_bounded(stdout));
    let errors = thread::spawn(move || read_bounded(stderr));
    let deadline = Instant::now() + timeout;
    let result = loop {
        if !cleanup && cancelled.load(Ordering::Relaxed) {
            kill_group(&mut child);
            break Err(AccessError::Cancelled);
        }
        if Instant::now() >= deadline {
            kill_group(&mut child);
            break Err(AccessError::Timeout);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err(AccessError::DependencyFailed)
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(_) => {
                kill_group(&mut child);
                break Err(AccessError::DependencyFailed);
            }
        }
    };
    let written = writer.join().map_err(|_| AccessError::DependencyFailed)?;
    let output = reader.join().map_err(|_| AccessError::DependencyFailed)?;
    let error_bytes = errors.join().map_err(|_| AccessError::DependencyFailed)??;
    if result.is_err() {
        let message = Zeroizing::new(String::from_utf8_lossy(&error_bytes).to_lowercase());
        let conditions: Vec<_> = [
            "illegal seek",
            "permission denied",
            "unknown authority",
            "connection refused",
            "no such file",
            "cannot unmarshal",
            "read /dev/stdin",
            "expired",
            "certificate",
            "config",
            "timeout",
        ]
        .into_iter()
        .filter(|condition| message.contains(condition))
        .collect();
        eprintln!("operator_access_dependency_failure program={program} conditions={conditions:?}");
    }
    result?;
    written.map_err(|_| AccessError::DependencyFailed)?;
    output
}

pub(super) fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_string()).collect()
}

pub(super) struct Oci<'a>(pub(super) &'a Profile);
impl Oci<'_> {
    pub(super) fn run(
        &self,
        args: &[&str],
        cleanup: bool,
    ) -> Result<Zeroizing<Vec<u8>>, AccessError> {
        let mut argv = strings(args);
        argv.extend(strings(&[
            "--config-file",
            &self.0.oci_config_file,
            "--profile",
            &self.0.oci_profile,
            "--auth",
            "security_token",
            "--region",
            &self.0.region,
            "--max-retries",
            "0",
            "--connection-timeout",
            "10",
            "--read-timeout",
            "15",
        ]));
        run("oci", &argv, &[], cleanup)
    }
    pub(super) fn json(&self, args: &[&str], cleanup: bool) -> Result<Value, AccessError> {
        serde_json::from_slice(&self.run(args, cleanup)?).map_err(|_| AccessError::DependencyFailed)
    }
}
