use std::cell::RefCell;
use std::io;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(&self, spec: &CommandSpec) -> io::Result<CommandOutput>;
}

pub struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run(&self, spec: &CommandSpec) -> io::Result<CommandOutput> {
        let output = Command::new(&spec.program).args(&spec.args).output()?;
        Ok(CommandOutput {
            status: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

#[derive(Default)]
pub struct MemoryRunner {
    expected: RefCell<Vec<(CommandSpec, CommandOutput)>>,
    calls: RefCell<Vec<CommandSpec>>,
}

impl MemoryRunner {
    pub fn new(expected: Vec<(CommandSpec, CommandOutput)>) -> Self {
        Self {
            expected: RefCell::new(expected),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<CommandSpec> {
        self.calls.borrow().clone()
    }
}

impl CommandRunner for MemoryRunner {
    fn run(&self, spec: &CommandSpec) -> io::Result<CommandOutput> {
        self.calls.borrow_mut().push(spec.clone());
        let mut expected = self.expected.borrow_mut();
        if expected.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "unexpected command",
            ));
        }
        let (want, output) = expected.remove(0);
        if want != *spec {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unexpected command: got {:?}, want {:?}", spec, want),
            ));
        }
        Ok(output)
    }
}
