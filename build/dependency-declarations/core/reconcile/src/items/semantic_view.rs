/// Borrowed view of call arguments without a second owned graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallArgumentsRefV1<'a> {
    Positional(&'a [SemanticValueV1]),
    Named(&'a [(Box<str>, SemanticValueV1)]),
}

/// Borrowed view of one typed semantic value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticValueRefV1<'a> {
    None,
    Bool(bool),
    Signed(i128),
    Unsigned(u128),
    String(&'a str),
    Identifier(&'a str),
    List(&'a [SemanticValueV1]),
    Tuple(&'a [SemanticValueV1]),
    Map(&'a [(SemanticValueV1, SemanticValueV1)]),
    Call {
        callee: &'a str,
        arguments: CallArgumentsRefV1<'a>,
    },
}

impl SemanticValueV1 {
    /// Exposes the typed value by immutable borrow.
    #[must_use]
    pub fn view(&self) -> SemanticValueRefV1<'_> {
        match &self.kind {
            SemanticValueKindV1::None => SemanticValueRefV1::None,
            SemanticValueKindV1::Bool(value) => SemanticValueRefV1::Bool(*value),
            SemanticValueKindV1::Signed(value) => SemanticValueRefV1::Signed(*value),
            SemanticValueKindV1::Unsigned(value) => SemanticValueRefV1::Unsigned(*value),
            SemanticValueKindV1::String(value) => SemanticValueRefV1::String(value),
            SemanticValueKindV1::Identifier(value) => SemanticValueRefV1::Identifier(value),
            SemanticValueKindV1::List(values) => SemanticValueRefV1::List(values),
            SemanticValueKindV1::Tuple(values) => SemanticValueRefV1::Tuple(values),
            SemanticValueKindV1::Map(entries) => SemanticValueRefV1::Map(entries),
            SemanticValueKindV1::Call { callee, arguments } => SemanticValueRefV1::Call {
                callee,
                arguments: match arguments {
                    CallArgumentsV1::Positional(values) => CallArgumentsRefV1::Positional(values),
                    CallArgumentsV1::Named(fields) => CallArgumentsRefV1::Named(fields),
                },
            },
        }
    }
}
