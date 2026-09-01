use dependency_declarations_generation::RenderedDeclarationProjectionPort;
use dependency_declarations_reconcile::{
    DigestV1, ParsedBuckProjectionV1, ProjectionPortErrorV1, RenderedRuleGraphV1,
    RenderedRuleV1, SemanticValueV1, ValidationBoundsV1,
};
use starlark_syntax::codemap::Span;
use starlark_syntax::syntax::ast::{
    ArgumentP, AstExpr, AstLiteral, AstStmt, BinOp, ExprP, Stmt,
};
use starlark_syntax::syntax::{AstModule, Dialect, DialectTypes};

/// Whole-artifact projection through the exactly pinned maintained parser.
pub struct StarlarkSyntaxProjectionV1 {
    profile_sha256: DigestV1,
}

impl StarlarkSyntaxProjectionV1 {
    /// Binds this adapter instance to one independently qualified projection profile.
    #[must_use]
    pub const fn new(profile_sha256: DigestV1) -> Self {
        Self { profile_sha256 }
    }
}

impl RenderedDeclarationProjectionPort for StarlarkSyntaxProjectionV1 {
    type Profile = DigestV1;
    type Projection = ParsedBuckProjectionV1;
    type Error = ProjectionPortErrorV1;

    fn profile(&self) -> &Self::Profile {
        &self.profile_sha256
    }

    fn project(&self, rendered: &[u8]) -> Result<Self::Projection, Self::Error> {
        let graph = project_reindeer_buck_v1(rendered)?;
        Ok(ParsedBuckProjectionV1::for_projection(
            self.profile_sha256,
            graph,
            rendered,
        ))
    }
}
