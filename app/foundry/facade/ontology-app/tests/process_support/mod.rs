//! Drives the real `foundry-ontology-app` executable as a child process over a
//! TCP socket: the drills below need a process to kill, not a router to call.

use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

pub const TENANT: &str = "ten_acme";
pub const TOKEN: &str = "operator-token-for-drills";
const READY_DEADLINE: Duration = Duration::from_secs(10);
const EXIT_DEADLINE: Duration = Duration::from_secs(10);
const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

/// The durable files one drill owns: both logs, the projection store and
/// the process's stdout, where the JSON log lines land. Removed when the
/// drill ends.
pub struct Store {
    pub action: PathBuf,
    pub denial: PathBuf,
    pub projection: PathBuf,
    pub stdout: PathBuf,
}

impl Store {
    pub fn new(case: &str) -> Self {
        let stamp = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the epoch")
                .as_nanos()
        );
        let temp = std::env::temp_dir();
        let name = |slot: &str| temp.join(format!("foundry-drill-{case}-{slot}-{stamp}"));
        let store = Self {
            action: name("action.sqlite"),
            denial: name("denial.sqlite"),
            projection: name("projection.sqlite"),
            stdout: name("stdout.log"),
        };
        store.remove();
        store
    }

    /// Removes the logs, their SQLite WAL sidecars (a killed child never
    /// checkpoints them), and the captured stdout.
    fn remove(&self) {
        for path in [&self.action, &self.denial, &self.projection] {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
            }
        }
        let _ = std::fs::remove_file(&self.stdout);
    }

    pub fn stdout(&self) -> String {
        std::fs::read_to_string(&self.stdout).unwrap_or_default()
    }

    pub fn stdout_contains(&self, needle: &str) -> bool {
        self.stdout().contains(needle)
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        self.remove();
    }
}

/// A spawned executable. Dropping it kills and reaps the child so a failed
/// assertion never leaves a server behind.
pub struct Process {
    pub child: Child,
    pub address: SocketAddr,
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Process {
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn is_running(&mut self) -> bool {
        self.child
            .try_wait()
            .expect("the child can be polled")
            .is_none()
    }

    pub fn wait_exit(&mut self) -> ExitStatus {
        let deadline = Instant::now() + EXIT_DEADLINE;
        loop {
            if let Some(status) = self.child.try_wait().expect("the child can be polled") {
                return status;
            }
            assert!(Instant::now() < deadline, "the process did not exit");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Send a POSIX signal by name (`-TERM`, `-INT`) through `/bin/kill`. An
    /// explicit Unix dependency: its absence fails the drill, never skips it.
    pub fn signal(&self, signal: &str) {
        assert!(
            Command::new("/bin/kill")
                .arg(signal)
                .arg(self.pid().to_string())
                .status()
                .expect("Unix /bin/kill is required")
                .success()
        );
    }
}

/// A port nobody holds at the moment of asking. The listener is dropped so
/// the child can bind it; the window between the two is the usual one.
pub fn free_address() -> SocketAddr {
    TcpListener::bind("127.0.0.1:0")
        .expect("a loopback port is free")
        .local_addr()
        .expect("the bound address is readable")
}

pub fn spawn(store: &Store, address: SocketAddr) -> Process {
    let stdout = File::options()
        .create(true)
        .append(true)
        .open(&store.stdout)
        .expect("the stdout capture file opens");
    let child = Command::new(env!("CARGO_BIN_EXE_foundry-ontology-app"))
        .env("OYATIE_FOUNDRY_ONTOLOGY_LISTEN_ADDR", address.to_string())
        .env("OYATIE_FOUNDRY_ONTOLOGY_ACTION_LOG", &store.action)
        .env("OYATIE_FOUNDRY_ONTOLOGY_DENIAL_LOG", &store.denial)
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_PROJECTION_STORE",
            &store.projection,
        )
        .env("OYATIE_FOUNDRY_ONTOLOGY_TENANTS", TENANT)
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_OPERATORS",
            format!("{TOKEN}:{TENANT}:prn_alice:foundry-operator"),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the executable spawns");
    Process { child, address }
}

/// Spawn and block until `/healthz` answers 200. A process that exits first
/// fails here with its status, not later with a connection error.
pub fn spawn_serving(store: &Store, address: SocketAddr) -> Process {
    let mut process = spawn(store, address);
    let deadline = Instant::now() + READY_DEADLINE;
    loop {
        if let Some(status) = process.child.try_wait().expect("the child can be polled") {
            panic!(
                "the process exited before it served: {status}\n{}",
                store.stdout()
            );
        }
        assert!(
            Instant::now() < deadline,
            "the process did not become ready"
        );
        if connect(address).is_some() {
            let response = get(address, "/healthz");
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            return process;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn connect(address: SocketAddr) -> Option<TcpStream> {
    let socket = TcpStream::connect_timeout(&address, Duration::from_millis(200)).ok()?;
    socket
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .expect("read timeout is settable");
    socket
        .set_write_timeout(Some(SOCKET_TIMEOUT))
        .expect("write timeout is settable");
    Some(socket)
}

pub fn read_response(socket: &mut TcpStream) -> String {
    let mut response = String::new();
    socket
        .read_to_string(&mut response)
        .expect("the response is readable to close");
    response
}

pub fn request(address: SocketAddr, raw: &[u8]) -> String {
    let mut socket = connect(address).expect("the process accepts a connection");
    socket.write_all(raw).expect("the request is writable");
    read_response(&mut socket)
}

pub fn get(address: SocketAddr, path: &str) -> String {
    request(
        address,
        format!(
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {TOKEN}\r\nConnection: close\r\n\r\n"
        )
        .as_bytes(),
    )
}

pub fn write_body(object_ref: &str, key: &str, name: &str) -> String {
    format!(
        r#"{{"object_ref":"{object_ref}","action_type":"aty_record_write","idempotency_key":"{key}","occurred_at_epoch_seconds":1700000000,"properties":{{"name":"{name}"}}}}"#
    )
}

pub fn post_head(body_len: usize) -> String {
    format!(
        "POST /v1/actions HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {TOKEN}\r\nContent-Type: application/json\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n"
    )
}

pub fn post_action(address: SocketAddr, body: &str) -> Option<String> {
    let mut socket = connect(address)?;
    let raw = format!("{}{body}", post_head(body.len()));
    socket.write_all(raw.as_bytes()).ok()?;
    let mut response = String::new();
    socket.read_to_string(&mut response).ok()?;
    Some(response)
}

pub fn body_of(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

pub fn json_of(response: &str) -> serde_json::Value {
    serde_json::from_str(body_of(response))
        .unwrap_or_else(|error| panic!("the body is JSON ({error}): {response}"))
}
