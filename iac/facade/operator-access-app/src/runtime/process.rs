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

pub(super) fn kill_group(child: &mut OwnedProcess) {
    let _ = child.terminate();
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
    let child = Command::new(program)
        .args(args)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| AccessError::DependencyFailed)?;
    let mut child = OwnedProcess::new(child);
    let mut stdin = child.child()?.stdin.take();
    let mut stdout = child.child()?.stdout.take();
    let mut stderr = child.child()?.stderr.take();
    pipe_io::nonblocking(stdin.as_ref().ok_or(AccessError::DependencyFailed)?)?;
    pipe_io::nonblocking(stdout.as_ref().ok_or(AccessError::DependencyFailed)?)?;
    pipe_io::nonblocking(stderr.as_ref().ok_or(AccessError::DependencyFailed)?)?;
    let mut output = Zeroizing::new(Vec::new());
    let mut errors = Zeroizing::new(Vec::new());
    let mut written = 0;
    let deadline = Instant::now() + timeout;
    loop {
        if !cleanup && cancelled.load(Ordering::Relaxed) {
            return Err(AccessError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(AccessError::Timeout);
        }
        pipe_io::write_step(&mut stdin, input, &mut written)?;
        pipe_io::read_step(&mut stdout, &mut output)?;
        pipe_io::read_step(&mut stderr, &mut errors)?;
        if stdin.is_none() && stdout.is_none() && stderr.is_none() && child.exited()? {
            let status = child.terminate()?;
            if !status.success() {
                return Err(AccessError::DependencyFailed);
            }
            return Ok(output);
        }
        thread::sleep(Duration::from_millis(5));
    }
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
        #[cfg(test)]
        if let Some(result) = super::session_tests::intercept(args, cleanup) {
            return result.and_then(|v| {
                serde_json::to_vec(&v)
                    .map(Zeroizing::new)
                    .map_err(|_| AccessError::DependencyFailed)
            });
        }
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
