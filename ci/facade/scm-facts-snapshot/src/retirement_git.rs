/// Streaming visitor for one bounded Git blob body.
pub type BlobVisitor<'a> = dyn FnMut(&str, u64, &mut dyn Read) -> Result<(), String> + 'a;

pub(crate) trait RetirementObjectSource {
    fn resolve_commit(&self, revision: &str) -> Result<String, String>;
    fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String>;
    fn first_parent(&self, commit_oid: &str) -> Result<String, String>;
    fn parents(&self, commit_oid: &str) -> Result<Vec<String>, String>;
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String>;
    fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String>;
    fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String>;
    /// Visit the requested blobs without requiring callers to retain their bodies.
    ///
    /// Sources with an efficient streaming object protocol should override this. The
    /// default keeps test doubles and non-Git sources correct while preserving the
    /// bounded-memory contract for callers.
    fn visit_blobs(&self, blob_oids: &[String], visit: &mut BlobVisitor<'_>) -> Result<(), String> {
        for blob_oid in blob_oids {
            let bytes = self.read_blob(blob_oid)?;
            let size = bytes.len() as u64;
            let mut reader = Cursor::new(bytes);
            visit(blob_oid, size, &mut reader)?;
        }
        Ok(())
    }
    fn commits_touching_path(&self, commit_oid: &str, path: &str) -> Result<Vec<String>, String>;
}

pub(crate) struct GitCliRetirementObjectSource {
    repo_root: PathBuf,
}

impl GitCliRetirementObjectSource {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    fn git(&self, args: &[&str], label: &str) -> Result<Vec<u8>, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(args)
            .output()
            .map_err(|error| format!("{label}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "{label}: git exited {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(output.stdout)
    }
}

/// Stream exact Git blob bodies through one `git cat-file --batch` process.
///
/// Public solely so the package's dedicated integration target can exercise
/// this real Git boundary through the production implementation. It is not an
/// admission-authority API.
pub fn visit_git_blobs(
    repo_root: &Path,
    blob_oids: &[String],
    visit: &mut BlobVisitor<'_>,
) -> Result<(), String> {
    for blob_oid in blob_oids {
        validate_oid(blob_oid, "retirement blob")?;
    }
    if blob_oids.is_empty() {
        return Ok(());
    }

    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("stream retirement blobs: {error}"))?;
    let stderr = child
        .stderr
        .take()
        .map(|mut stderr| {
            std::thread::spawn(move || {
                let mut bytes = Vec::new();
                stderr.read_to_end(&mut bytes).map(|_| bytes)
            })
        })
        .ok_or_else(|| "stream retirement blobs: stderr unavailable".to_owned())?;
    let mut result = (|| {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "stream retirement blobs: stdin unavailable".to_owned())?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| "stream retirement blobs: stdout unavailable".to_owned())?;
        for blob_oid in blob_oids {
            stdin
                .write_all(blob_oid.as_bytes())
                .and_then(|()| stdin.write_all(b"\n"))
                .and_then(|()| stdin.flush())
                .map_err(|error| format!("stream retirement blobs: write {blob_oid}: {error}"))?;

            let mut header = Vec::new();
            loop {
                let mut byte = [0_u8; 1];
                stdout.read_exact(&mut byte).map_err(|error| {
                    format!("stream retirement blobs: read header for {blob_oid}: {error}")
                })?;
                if byte == *b"\n" {
                    break;
                }
                if header.len() == CAT_FILE_HEADER_LIMIT {
                    return Err(format!(
                        "stream retirement blobs: header exceeds {CAT_FILE_HEADER_LIMIT} bytes for {blob_oid}"
                    ));
                }
                header.push(byte[0]);
            }
            let header = std::str::from_utf8(&header).map_err(|error| {
                format!("stream retirement blobs: non-UTF-8 header for {blob_oid}: {error}")
            })?;
            let expected_prefix = format!("{blob_oid} blob ");
            let size = header
                .strip_prefix(&expected_prefix)
                .ok_or_else(|| {
                    format!("stream retirement blobs: unexpected header for {blob_oid}: {header}")
                })?
                .parse::<u64>()
                .map_err(|error| {
                    format!("stream retirement blobs: invalid size for {blob_oid}: {error}")
                })?;
            let mut body = (&mut stdout).take(size);
            visit(blob_oid, size, &mut body)?;
            std::io::copy(&mut body, &mut std::io::sink()).map_err(|error| {
                format!("stream retirement blobs: drain body for {blob_oid}: {error}")
            })?;
            if body.limit() != 0 {
                return Err(format!(
                    "stream retirement blobs: body ended early for {blob_oid} after {} of {size} bytes",
                    size - body.limit()
                ));
            }
            let mut terminator = [0_u8; 1];
            stdout.read_exact(&mut terminator).map_err(|error| {
                format!("stream retirement blobs: read terminator for {blob_oid}: {error}")
            })?;
            if terminator != *b"\n" {
                return Err(format!(
                    "stream retirement blobs: missing body terminator for {blob_oid}"
                ));
            }
        }
        drop(stdin);
        let status = child
            .wait()
            .map_err(|error| format!("stream retirement blobs: wait: {error}"))?;
        if !status.success() {
            return Err(format!(
                "stream retirement blobs: git exited {:?}",
                status.code(),
            ));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let stderr = stderr
        .join()
        .map_err(|_| "stream retirement blobs: stderr reader panicked".to_owned())?
        .map_err(|error| format!("stream retirement blobs: read stderr: {error}"))?;
    if let Err(error) = &mut result {
        let diagnostics = String::from_utf8_lossy(&stderr).trim().to_owned();
        if !diagnostics.is_empty() {
            error.push_str(": ");
            error.push_str(&diagnostics);
        }
    }
    result
}

impl RetirementObjectSource for GitCliRetirementObjectSource {
    fn resolve_commit(&self, revision: &str) -> Result<String, String> {
        let expression = format!("{revision}^{{commit}}");
        let output = self.git(
            &["rev-parse", "--verify", &expression],
            "resolve retirement commit",
        )?;
        parse_oid_text(&output, "resolved retirement commit")
    }

    fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String> {
        let expression = format!("{commit_oid}^{{tree}}");
        let output = self.git(
            &["rev-parse", "--verify", &expression],
            "resolve retirement tree",
        )?;
        parse_oid_text(&output, "resolved retirement tree")
    }

    fn first_parent(&self, commit_oid: &str) -> Result<String, String> {
        let expression = format!("{commit_oid}^1");
        let output = self.git(
            &["rev-parse", "--verify", &expression],
            "resolve retirement first parent",
        )?;
        parse_oid_text(&output, "resolved retirement first parent")
    }

    fn parents(&self, commit_oid: &str) -> Result<Vec<String>, String> {
        let output = self.git(
            &["rev-list", "--parents", "-n", "1", commit_oid],
            "resolve retirement parents",
        )?;
        let line = String::from_utf8(output)
            .map_err(|error| format!("retirement parents are not UTF-8: {error}"))?;
        let mut fields = line.split_whitespace();
        let commit = fields
            .next()
            .ok_or_else(|| "retirement parents are empty".to_owned())?;
        if commit != commit_oid {
            return Err(
                "retirement parent list does not bind the requested evaluated commit".to_owned(),
            );
        }
        fields
            .map(|parent| {
                validate_oid(parent, "retirement parent")?;
                Ok(parent.to_owned())
            })
            .collect()
    }

    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String> {
        let status = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(["merge-base", "--is-ancestor", ancestor, descendant])
            .status()
            .map_err(|error| format!("retirement ancestry: {error}"))?;
        match status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            code => Err(format!("retirement ancestry: git exited {code:?}")),
        }
    }

    fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String> {
        let output = self.git(
            &["ls-tree", "-rz", "--full-tree", "-r", commit_oid],
            "enumerate retirement tree",
        )?;
        parse_ls_tree(&output)
    }

    fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String> {
        self.git(&["cat-file", "blob", blob_oid], "read retirement blob")
    }

    fn visit_blobs(&self, blob_oids: &[String], visit: &mut BlobVisitor<'_>) -> Result<(), String> {
        visit_git_blobs(&self.repo_root, blob_oids, visit)
    }

    fn commits_touching_path(&self, commit_oid: &str, path: &str) -> Result<Vec<String>, String> {
        let output = self.git(
            &["rev-list", commit_oid, "--", path],
            "walk retirement receipt history",
        )?;
        String::from_utf8(output)
            .map_err(|error| format!("retirement history is not UTF-8: {error}"))?
            .lines()
            .map(|line| {
                validate_oid(line, "retirement history commit")?;
                Ok(line.to_owned())
            })
            .collect()
    }
}
