#![cfg(unix)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Process {
    child: Child,
    fixture: PathBuf,
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.fixture);
    }
}

fn orderly_exit(signal: &str) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let fixture = std::env::temp_dir().join(format!(
        "oyatie-iac-signal-{}-{}-{}.json",
        std::process::id(),
        address.port(),
        signal,
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&fixture)
        .unwrap();
    file.write_all(include_bytes!("release-index.json"))
        .unwrap();
    drop(file);
    drop(listener);
    let child = Command::new(env!("CARGO_BIN_EXE_iac-app"))
        .env("OYATIE_CLOUD_IAC_BIND_ADDR", address.to_string())
        .env("OYATIE_CLOUD_IAC_RELEASE_INDEX_PATH", &fixture)
        .env(
            "OYATIE_CLOUD_IAC_MODULE_REGISTRY_BEARER",
            "signal-test-bearer",
        )
        .env(
            "OYATIE_CLOUD_IAC_MODULE_REGISTRY_PRINCIPAL",
            "signal-test-reader",
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut process = Process { child, fixture };
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            process.child.try_wait().unwrap().is_none(),
            "process exited before readiness"
        );
        assert!(Instant::now() < deadline, "process did not become ready");
        if let Ok(mut socket) = TcpStream::connect_timeout(&address, Duration::from_millis(100)) {
            socket
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            socket
                .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .unwrap();
            let mut response = String::new();
            socket.read_to_string(&mut response).unwrap();
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    // This is an explicit Unix integration dependency; absence is a failure,
    // never a silent skip or a claim of hermetic signal-tool provisioning.
    assert!(
        Command::new("/bin/kill")
            .arg(signal)
            .arg(process.child.id().to_string())
            .status()
            .expect("Unix /bin/kill is required")
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = process.child.try_wait().unwrap() {
            assert!(
                status.success(),
                "signal must drain and exit zero, not kill process: {status}"
            );
            break;
        }
        assert!(Instant::now() < deadline, "signal did not stop serving");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn sigterm_drains_actual_iac_executable() {
    orderly_exit("-TERM");
}

#[test]
fn sigint_drains_actual_iac_executable() {
    orderly_exit("-INT");
}

#[test]
fn executable_errors_exit_unsuccessfully_before_serving() {
    let status = Command::new(env!("CARGO_BIN_EXE_iac-app"))
        .env("OYATIE_CLOUD_IAC_BIND_ADDR", "invalid-address")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(!status.success());
}
