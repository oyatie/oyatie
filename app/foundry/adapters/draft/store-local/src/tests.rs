#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use std::{
    io::{BufRead, Write},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
// Parallel fork/exec can temporarily inherit another test's live descriptor before exec closes it.
static PROCESS_TESTS: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub(super) struct Directory(PathBuf, std::sync::MutexGuard<'static, ()>);
impl Directory {
    pub(super) fn new() -> Self {
        let serial = PROCESS_TESTS.lock().unwrap();
        let path = std::env::temp_dir().join(format!(
            "foundry-store-access-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path, serial)
    }
    pub(super) fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn exclusive(path: &Path) -> Result<LocalStoreSet, StoreAccessError> {
    LocalStoreSet::exclusive(&[(path.to_owned(), StoreOpen::Create)])
}
fn child(path: &Path, mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "tests::child_process", "--nocapture"])
        .env("FOUNDRY_STORE_TEST_PATH", path)
        .env("FOUNDRY_STORE_TEST_MODE", mode);
    command
}
#[test]
fn child_process() {
    let Some(path) = std::env::var_os("FOUNDRY_STORE_TEST_PATH") else {
        return;
    };
    let path = PathBuf::from(path);
    match std::env::var("FOUNDRY_STORE_TEST_MODE").unwrap().as_str() {
        "shared_busy" => assert!(matches!(
            LocalStoreGuard::shared(&path, StoreOpen::Existing),
            Err(StoreAccessError::Busy)
        )),
        "exclusive_busy" => assert!(matches!(exclusive(&path), Err(StoreAccessError::Busy))),
        "hold" => {
            let _guard = exclusive(&path).unwrap();
            println!("LEASE_HELD");
            std::io::stdout().flush().unwrap();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
        }
        _ => panic!("unknown child mode"),
    }
}
#[test]
fn native_lifetime_excludes_cross_process_in_both_opening_orders() {
    let directory = Directory::new();
    let path = directory.path("store");
    let first = LocalStoreGuard::shared(&path, StoreOpen::Create).unwrap();
    let second = LocalStoreGuard::shared(&path, StoreOpen::Existing).unwrap();
    assert!(child(&path, "exclusive_busy").status().unwrap().success());
    drop(first);
    drop(second);
    let set = exclusive(&path).unwrap();
    let guard = set.guard(&path).unwrap();
    assert!(guard.access().exclusive());
    assert!(
        guard
            .access()
            .same_owner(set.guard(&path).unwrap().access())
    );
    assert!(child(&path, "shared_busy").status().unwrap().success());
    drop(set);
    assert!(child(&path, "exclusive_busy").status().unwrap().success());
    drop(guard);
    assert!(LocalStoreGuard::shared(&path, StoreOpen::Existing).is_ok());
}
#[test]
fn process_termination_releases_the_owned_files() {
    let directory = Directory::new();
    let path = directory.path("store");
    let mut process = child(&path, "hold")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = std::io::BufReader::new(process.stdout.take().unwrap());
    loop {
        let mut line = String::new();
        assert!(output.read_line(&mut line).unwrap() > 0);
        if line.trim() == "LEASE_HELD" {
            break;
        }
    }
    assert!(matches!(
        LocalStoreGuard::shared(&path, StoreOpen::Existing),
        Err(StoreAccessError::Busy)
    ));
    process.kill().unwrap();
    process.wait().unwrap();
    assert!(LocalStoreGuard::shared(&path, StoreOpen::Existing).is_ok());
}
#[test]
fn partial_acquisition_and_failed_constructor_release_ownership() {
    let directory = Directory::new();
    let path = directory.path("a");
    let missing = directory.path("z").join("missing");
    assert!(
        LocalStoreSet::exclusive(&[
            (path.clone(), StoreOpen::Create),
            (missing, StoreOpen::Existing)
        ])
        .is_err()
    );
    assert!(LocalStoreGuard::shared(&path, StoreOpen::Existing).is_ok());
    let other = directory.path("other");
    let set = exclusive(&path).unwrap();
    assert!(exclusive(&other).is_ok());
    assert!(matches!(
        set.guard(&other),
        Err(StoreAccessError::WrongOwner)
    ));
}
#[test]
fn alias_and_replaced_identity_are_refused() {
    let directory = Directory::new();
    let path = directory.path("a");
    std::fs::write(&path, []).unwrap();
    let hard = directory.path("hard");
    std::fs::hard_link(&path, &hard).unwrap();
    assert!(matches!(
        LocalStoreSet::exclusive(&[
            (path.clone(), StoreOpen::Existing),
            (hard, StoreOpen::Existing)
        ]),
        Err(StoreAccessError::Aliased)
    ));
    #[cfg(unix)]
    {
        let link = directory.path("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(matches!(
            LocalStoreSet::exclusive(&[
                (path.clone(), StoreOpen::Existing),
                (link, StoreOpen::Existing)
            ]),
            Err(StoreAccessError::Aliased)
        ));
    }
    let set = exclusive(&path).unwrap();
    let guard = set.guard(&path).unwrap();
    std::fs::rename(&path, directory.path("previous")).unwrap();
    std::fs::write(&path, []).unwrap();
    assert_eq!(
        guard.validate(&path),
        Err(StoreAccessError::IdentityChanged)
    );
}

#[test]
fn ownership_is_not_inherited_by_an_unrelated_exec_child() {
    let directory = Directory::new();
    let path = directory.path("store");
    let set = exclusive(&path).unwrap();
    let mut child = Command::new("/bin/cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"EXEC_READY\n")
        .unwrap();
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert_eq!(line, "EXEC_READY\n");
    drop(set);
    assert!(LocalStoreGuard::shared(&path, StoreOpen::Existing).is_ok());
    child.kill().unwrap();
    child.wait().unwrap();
}
#[test]
fn sqlite_uri_and_memory_inputs_are_refused_before_creation() {
    for path in [
        Path::new(":memory:"),
        Path::new("file:store?mode=memory&cache=shared"),
        Path::new("file:store?immutable=1&vfs=unix-none"),
    ] {
        assert!(matches!(
            LocalStoreGuard::shared(path, StoreOpen::Create),
            Err(StoreAccessError::UnsupportedPath)
        ));
        assert!(!path.exists());
    }
}
