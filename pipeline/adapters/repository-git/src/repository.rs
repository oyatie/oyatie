use std::collections::BTreeMap;
use std::ffi::OsString;

use pipeline_repository::{
    ContentSelection, HydratedSnapshot, PreparedSnapshot, ProducerId, RepositoryId,
    RepositoryManifest, RepositorySnapshot, ResolvedRevision, RevisionId, SnapshotFailure,
    SnapshotLimits, SnapshotRequest, SnapshotSession, ToolId, TreeId, WorkControl,
};

use crate::command::{CommandOutput, GitCommandRunner, bounded_lossy, require_success};
use crate::object::verify_tree_identities;
use crate::output_limits::{
    content_stdout_limit, merge_base_stdout_limit, resolved_objects_stdout_limit, tree_stdout_limit,
};
use crate::parse::{
    ResolvedObject, ResolvedObjectKind, parse_batch_contents, parse_ls_tree, parse_merge_base,
    parse_resolved_objects,
};

#[derive(Clone)]
pub struct GitRepository {
    pub(crate) repository: RepositoryId,
    pub(crate) producer: ProducerId,
    pub(crate) tool: ToolId,
    pub(crate) runner: GitCommandRunner,
}

impl GitRepository {
    pub fn process_count(&self) -> u64 {
        self.runner.invocations()
    }

    fn merge_base(
        &self,
        base: RevisionId,
        head: RevisionId,
        limits: SnapshotLimits,
        control: &dyn WorkControl,
    ) -> Result<RevisionId, SnapshotFailure> {
        let output = self.runner.run(
            "resolve merge base",
            &git_arguments([
                "merge-base".to_owned(),
                "--all".to_owned(),
                base.to_hex(),
                head.to_hex(),
            ]),
            Vec::new(),
            merge_base_stdout_limit(base.algorithm(), limits),
            limits.max_stderr_bytes(),
            control,
        )?;
        if !output.status.success() {
            if output.status.code() == Some(1) && output.stdout.is_empty() {
                return Err(SnapshotFailure::MissingMergeBase);
            }
            return Err(tool_failure("resolve merge base", output));
        }
        parse_merge_base(&output.stdout, base.algorithm())
    }

    fn resolve_revisions(
        &self,
        base: RevisionId,
        head: RevisionId,
        merge_base: RevisionId,
        limits: SnapshotLimits,
        control: &dyn WorkControl,
    ) -> Result<[ResolvedRevision; 3], SnapshotFailure> {
        let mut input = Vec::new();
        for revision in [base, head, merge_base] {
            input.extend_from_slice(format!("{}^{{commit}}\n", revision.to_hex()).as_bytes());
            input.extend_from_slice(format!("{}^{{tree}}\n", revision.to_hex()).as_bytes());
        }
        let output = self.runner.run(
            "resolve revisions",
            &git_arguments(["cat-file", "--batch-check=%(objectname) %(objecttype)"]),
            input,
            resolved_objects_stdout_limit(base.algorithm(), limits),
            limits.max_stderr_bytes(),
            control,
        )?;
        let output = require_success("resolve revisions", output)?;
        let objects = parse_resolved_objects(&output, base.algorithm(), 6)?;
        Ok([
            resolved_revision(base, objects[0], objects[1])?,
            resolved_revision(head, objects[2], objects[3])?,
            resolved_revision(merge_base, objects[4], objects[5])?,
        ])
    }

    fn manifest_entries(
        &self,
        tree: TreeId,
        limits: SnapshotLimits,
        control: &dyn WorkControl,
    ) -> Result<Vec<pipeline_repository::Entry>, SnapshotFailure> {
        let output = self.runner.run(
            "enumerate repository tree",
            &git_arguments([
                "ls-tree".to_owned(),
                "-r".to_owned(),
                "-t".to_owned(),
                "-z".to_owned(),
                "--full-tree".to_owned(),
                tree.to_hex(),
            ]),
            Vec::new(),
            tree_stdout_limit(tree.algorithm(), limits),
            limits.max_stderr_bytes(),
            control,
        )?;
        let entries = parse_ls_tree(
            &require_success("enumerate repository tree", output)?,
            tree.algorithm(),
            limits,
        )?;
        verify_tree_identities(tree, &entries)?;
        Ok(entries)
    }
}

impl RepositorySnapshot for GitRepository {
    type Session = GitSnapshotSession;

    fn capture(
        &self,
        request: SnapshotRequest,
        control: &dyn WorkControl,
    ) -> Result<Self::Session, SnapshotFailure> {
        control.checkpoint()?;
        if request.repository() != &self.repository {
            return Err(SnapshotFailure::ObjectMismatch(format!(
                "request names repository {} but adapter is bound to {}",
                request.repository(),
                self.repository
            )));
        }
        let limits = request.profile().limits();
        let merge_base = self.merge_base(request.base(), request.head(), limits, control)?;
        let [resolved_base, resolved_head, resolved_merge] =
            self.resolve_revisions(request.base(), request.head(), merge_base, limits, control)?;

        let mut trees = BTreeMap::new();
        let merge_tree = resolved_merge.tree();
        let head_tree = resolved_head.tree();
        for tree in [merge_tree, head_tree] {
            if let std::collections::btree_map::Entry::Vacant(entry) = trees.entry(tree) {
                entry.insert(self.manifest_entries(tree, limits, control)?);
            }
        }
        let merge_entries = trees
            .remove(&merge_tree)
            .expect("requested merge tree was enumerated");
        let merge_manifest = RepositoryManifest::new(resolved_merge, merge_entries, limits)?;
        let head_manifest = if head_tree == merge_tree {
            RepositoryManifest::at_revision(resolved_head, &merge_manifest)?
        } else {
            let entries = trees
                .remove(&head_tree)
                .expect("requested head tree was enumerated");
            RepositoryManifest::new(resolved_head, entries, limits)?
        };
        let prepared = PreparedSnapshot::new(
            request,
            resolved_base,
            merge_manifest,
            head_manifest,
            self.producer.clone(),
            self.tool.clone(),
        )?;
        Ok(GitSnapshotSession {
            runner: self.runner.clone(),
            prepared,
        })
    }
}

pub struct GitSnapshotSession {
    runner: GitCommandRunner,
    prepared: PreparedSnapshot,
}

impl SnapshotSession for GitSnapshotSession {
    fn prepared(&self) -> &PreparedSnapshot {
        &self.prepared
    }

    fn hydrate(
        self,
        selection: ContentSelection,
        control: &dyn WorkControl,
    ) -> Result<HydratedSnapshot, SnapshotFailure> {
        control.checkpoint()?;
        if selection.ids().is_empty() {
            return HydratedSnapshot::complete(self.prepared, selection, BTreeMap::new());
        }
        let mut input = Vec::new();
        for id in selection.ids() {
            input.extend_from_slice(format!("contents {}\n", id.to_hex()).as_bytes());
        }
        input.extend_from_slice(b"flush\n");
        let limits = self.prepared.request().profile().limits();
        let output = self.runner.run(
            "hydrate repository content",
            &git_arguments(["cat-file", "--batch-command", "--buffer"]),
            input,
            content_stdout_limit(
                self.prepared.head().revision().tree().algorithm(),
                selection.ids().len(),
                limits,
            ),
            limits.max_stderr_bytes(),
            control,
        )?;
        let output = require_success("hydrate repository content", output)?;
        let contents = parse_batch_contents(&output, selection.ids(), limits)?;
        HydratedSnapshot::complete(self.prepared, selection, contents)
    }
}

fn resolved_revision(
    supplied: RevisionId,
    commit: ResolvedObject,
    tree: ResolvedObject,
) -> Result<ResolvedRevision, SnapshotFailure> {
    if commit.kind != ResolvedObjectKind::Commit || tree.kind != ResolvedObjectKind::Tree {
        return Err(SnapshotFailure::ObjectMismatch(
            "revision resolution did not return one commit followed by one tree".to_owned(),
        ));
    }
    let commit = RevisionId::from_object_id(commit.id);
    if commit != supplied {
        return Err(SnapshotFailure::ObjectMismatch(format!(
            "supplied revision {supplied} resolved as {commit}"
        )));
    }
    ResolvedRevision::new(supplied, commit, TreeId::from_object_id(tree.id))
}

pub(crate) fn git_arguments<I, S>(arguments: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    std::iter::once(OsString::from("--no-replace-objects"))
        .chain(arguments.into_iter().map(Into::into))
        .collect()
}

fn tool_failure(operation: &'static str, output: CommandOutput) -> SnapshotFailure {
    SnapshotFailure::ToolFailed {
        operation,
        status: output.status.code(),
        stderr: bounded_lossy(&output.stderr),
    }
}
