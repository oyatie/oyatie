//! Sound recursive-descent parser for the Starlark statement/expression subset BUCK gate crates
//! consume (ADR-0549).
//!
//! Modeled statements: `IDENT = expr`, `IDENT[expr] = expr`, and top-level calls
//! (`rust_library(...)`, `glob(...)` assignments, `load(...)`). Modeled expressions: string /
//! int literals, identifiers, `+` chains, list literals, dict literals, dict comprehensions
//! (`{K: V for x in ITER}`), and calls. Anything else is recorded as [`Expr::Opaque`] with an
//! EXACT span — the parser never misinterprets what it does not model (fail-honest), and
//! consumers map opaque shapes to their existing "unparseable"/refusal classifications.
//!
//! Soundness properties pinned by fixtures in `lib.rs`:
//! - a call/target inside a comment or string is NEVER a statement (comment-blind class);
//! - target binding is by the actual `name = "..."` kwarg, never first-occurrence substring
//!   match (the ADR-0545 "first-occurrence name binding" residual);
//! - delimiter depth is computed over TOKENS, so a paren inside a string or comment can never
//!   end a block early (the #691 H5 class) and a backslash-escaped quote cannot leak string
//!   state (the #693 LOW-X2 class);
//! - every node carries a byte span over the original text (edits are span-accurate);
//! - structurally undelimitable input (unbalanced brackets, unterminated strings) is a hard
//!   [`ParseError`] — fail-closed, never a guess.

use crate::lexer::{LexError, LexOutput, Span, Token, TokenKind, lex};

// ponytail: keep this as a parser-owned ceiling until callers need explicit parse budgets.
const MAX_EXPR_NESTING_DEPTH: usize = 128;

/// A parse failure: the text cannot be soundly modeled. Fail-closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError {
            offset: e.offset,
            message: e.message,
        }
    }
}

/// A parsed BUCK/Starlark document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuckDoc {
    pub stmts: Vec<Stmt>,
    /// Comment trivia spans (for edit primitives).
    pub comments: Vec<Span>,
}

/// One top-level statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// `NAME = expr`
    Assign {
        name: String,
        name_span: Span,
        value: ExprNode,
        span: Span,
    },
    /// `NAME[key_expr] = expr`
    IndexAssign {
        base: String,
        key: ExprNode,
        value: ExprNode,
        span: Span,
    },
    /// A top-level call statement: `rust_library(...)`, `load(...)`.
    Call(CallExpr),
    /// A statement shape the subset does not model (`def`, `if`, ...). Exact span; never
    /// interpreted.
    Opaque { span: Span },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Assign { span, .. } | Stmt::IndexAssign { span, .. } | Stmt::Opaque { span } => {
                *span
            }
            Stmt::Call(call) => call.span,
        }
    }
}

/// A call expression with span-accurate arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallExpr {
    pub func: String,
    pub func_span: Span,
    /// From the first byte of `func` to just past the closing `)`.
    pub span: Span,
    /// Byte offset of the opening `(`.
    pub open_paren: usize,
    /// Byte offset of the closing `)`.
    pub close_paren: usize,
    pub args: Vec<Arg>,
}

impl CallExpr {
    /// The FIRST keyword argument named `name` (Starlark forbids duplicates; first is canonical).
    pub fn kwarg(&self, name: &str) -> Option<&Arg> {
        self.args
            .iter()
            .find(|arg| arg.name.as_deref() == Some(name))
    }

    /// True if any argument (transitively) contains an [`Expr::Opaque`] node. Detect-lane
    /// consumers use this to fall back to a raw over-approximating scan of [`CallExpr::span`]
    /// rather than trust an extraction that cannot see into unmodeled content (fail-closed).
    pub fn has_opaque(&self) -> bool {
        self.args.iter().any(|arg| expr_has_opaque(&arg.value))
    }

    /// Depth-first visit of this call and every call nested in its arguments.
    pub fn visit_nested<'a>(&'a self, visit: &mut dyn FnMut(&'a CallExpr)) {
        visit(self);
        for arg in &self.args {
            arg.value.visit_calls(visit);
        }
    }
}

impl BuckDoc {
    /// Depth-first visit of EVERY call expression in the document: top-level call statements,
    /// calls wrapped in assignments (`X = rust_library(...)`), index-assignment values, and
    /// calls nested inside any expression. [`Stmt::Opaque`] statements carry no parsed calls —
    /// detect-lane consumers must raw-scan their spans separately (fail-closed).
    pub fn visit_calls<'a>(&'a self, visit: &mut dyn FnMut(&'a CallExpr)) {
        for stmt in &self.stmts {
            match stmt {
                Stmt::Call(call) => call.visit_nested(visit),
                Stmt::Assign { value, .. } => value.visit_calls(visit),
                Stmt::IndexAssign { key, value, .. } => {
                    key.visit_calls(visit);
                    value.visit_calls(visit);
                }
                Stmt::Opaque { .. } => {}
            }
        }
    }
}

fn expr_has_opaque(node: &ExprNode) -> bool {
    node.has_opaque()
}

impl ExprNode {
    /// True if this expression (transitively) contains an [`Expr::Opaque`] node — content the
    /// modeled subset could not interpret. Detect-lane consumers over-approximate on it;
    /// fixers/parsers-of-record refuse or demote to unparseable (fail-honest).
    pub fn has_opaque(&self) -> bool {
        match &self.expr {
            Expr::Opaque => true,
            Expr::Str(_) | Expr::Int(_) | Expr::Ident(_) => false,
            Expr::Plus(operands) => operands.iter().any(|node| node.has_opaque()),
            Expr::List(list) => list.elements.iter().any(|e| e.value.has_opaque()),
            Expr::Dict(dict) => {
                dict.entries
                    .iter()
                    .any(|entry| entry.key.has_opaque() || entry.value.has_opaque())
                    || dict.comprehension.as_ref().is_some_and(|comp| {
                        comp.key.has_opaque() || comp.value.has_opaque() || comp.iter.is_none()
                    })
            }
            Expr::Call(call) => call.has_opaque(),
        }
    }

    /// Depth-first visit of every [`CallExpr`] nested anywhere in this expression (including
    /// calls inside list/dict/plus operands and other calls' arguments). Detect-lane consumers
    /// use this so a target call WRAPPED in an expression (`X = rust_library(...)`) can never
    /// hide from enumeration (the statement-position-only blind spot).
    pub fn visit_calls<'a>(&'a self, visit: &mut dyn FnMut(&'a CallExpr)) {
        match &self.expr {
            Expr::Call(call) => {
                visit(call);
                for arg in &call.args {
                    arg.value.visit_calls(visit);
                }
            }
            Expr::Plus(operands) => {
                for operand in operands {
                    operand.visit_calls(visit);
                }
            }
            Expr::List(list) => {
                for element in &list.elements {
                    element.value.visit_calls(visit);
                }
            }
            Expr::Dict(dict) => {
                for entry in &dict.entries {
                    entry.key.visit_calls(visit);
                    entry.value.visit_calls(visit);
                }
                if let Some(comp) = &dict.comprehension {
                    comp.key.visit_calls(visit);
                    comp.value.visit_calls(visit);
                }
            }
            Expr::Str(_) | Expr::Int(_) | Expr::Ident(_) | Expr::Opaque => {}
        }
    }
}

/// One call argument (keyword or positional).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    /// The kwarg name, or `None` for a positional argument.
    pub name: Option<String>,
    pub name_span: Option<Span>,
    pub value: ExprNode,
    /// From the first byte of the name (or value) to the last byte of the value.
    pub span: Span,
    /// Byte offset of the `,` following this argument, if present.
    pub comma: Option<usize>,
}

/// An expression with its exact byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprNode {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A string literal (cooked value).
    Str(String),
    /// An integer-ish literal, raw.
    Int(String),
    /// A bare identifier reference.
    Ident(String),
    /// `a + b + c` — operands in order.
    Plus(Vec<ExprNode>),
    /// `[e1, e2, ...]`
    List(ListExpr),
    /// `{k: v, ...}` or `{K: V for x in ITER}`
    Dict(DictExpr),
    /// `func(args...)` as an expression (e.g. `glob([...])`, `select({...})`).
    Call(Box<CallExpr>),
    /// A shape the subset does not model. Exact span; never interpreted.
    Opaque,
}

/// A list literal with element + comma spans (for safe element removal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListExpr {
    pub elements: Vec<ListElement>,
    /// Byte offset of `[`.
    pub open_bracket: usize,
    /// Byte offset of `]`.
    pub close_bracket: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListElement {
    pub value: ExprNode,
    /// Byte offset of the `,` following this element, if present.
    pub comma: Option<usize>,
}

/// A dict literal or comprehension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictExpr {
    /// Literal entries (empty for a comprehension).
    pub entries: Vec<DictEntry>,
    /// The comprehension, when this is `{K: V for x in ITER}`.
    pub comprehension: Option<DictComp>,
    /// Byte offset of `{`.
    pub open_brace: usize,
    /// Byte offset of `}`.
    pub close_brace: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictEntry {
    pub key: ExprNode,
    pub value: ExprNode,
    pub span: Span,
    /// Byte offset of the `,` following this entry, if present.
    pub comma: Option<usize>,
}

/// `{KEY: VALUE for VAR in ITER}` where ITER is a bare identifier (the only iter shape the
/// consumers resolve). A non-ident iter or an `if` clause yields `iter: None` — resolvable to
/// nothing, exactly like the prior gate parsers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictComp {
    pub key: Box<ExprNode>,
    pub value: Box<ExprNode>,
    pub var: String,
    pub iter: Option<String>,
}

/// Parse `text` into a [`BuckDoc`]. Fail-closed on structurally undelimitable input.
pub fn parse(text: &str) -> Result<BuckDoc, ParseError> {
    let LexOutput { tokens, comments } = lex(text)?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        text_len: text.len(),
    };
    let mut stmts = Vec::new();
    while !parser.at_end() {
        if parser.eat_newline() {
            continue;
        }
        stmts.push(parser.parse_stmt()?);
    }
    Ok(BuckDoc { stmts, comments })
}

struct Parser<'t> {
    tokens: &'t [Token],
    pos: usize,
    text_len: usize,
}

impl<'t> Parser<'t> {
    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&'t Token> {
        self.tokens.get(self.pos)
    }

    fn peek_at(&self, offset: usize) -> Option<&'t Token> {
        self.tokens.get(self.pos + offset)
    }

    fn bump(&mut self) -> Option<&'t Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Consume a `,` if it is the next token; returns its byte offset.
    fn eat_comma(&mut self) -> Option<usize> {
        if let Some(Token {
            kind: TokenKind::Punct(','),
            span,
        }) = self.peek()
        {
            let at = span.start;
            self.pos += 1;
            return Some(at);
        }
        None
    }

    fn eat_newline(&mut self) -> bool {
        if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Newline)) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            offset: self.peek().map(|t| t.span.start).unwrap_or(self.text_len),
            message: message.into(),
        }
    }

    fn check_expr_depth(&self, depth: usize, offset: usize) -> Result<(), ParseError> {
        if depth > MAX_EXPR_NESTING_DEPTH {
            return Err(ParseError {
                offset,
                message: format!("expression nesting depth exceeds {MAX_EXPR_NESTING_DEPTH}"),
            });
        }
        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let Some(first) = self.peek() else {
            return Err(self.error("expected a statement"));
        };
        let start = first.span.start;
        if let TokenKind::Ident(name) = &first.kind {
            let name = name.clone();
            let name_span = first.span;
            match self.peek_at(1).map(|t| &t.kind) {
                // NAME = expr
                Some(TokenKind::Punct('=')) => {
                    self.pos += 2;
                    let value = self.parse_expr()?;
                    let span = Span::new(start, value.span.end);
                    if let Some(tail_end) = self.finish_stmt_line() {
                        // Trailing unmodeled tokens: the WHOLE statement is opaque.
                        return Ok(Stmt::Opaque {
                            span: Span::new(start, tail_end),
                        });
                    }
                    return Ok(Stmt::Assign {
                        name,
                        name_span,
                        value,
                        span,
                    });
                }
                // NAME[key] = expr
                Some(TokenKind::Punct('[')) => {
                    let checkpoint = self.pos;
                    self.pos += 2; // NAME [
                    let key = self.parse_expr()?;
                    if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Punct(']')))
                        && matches!(
                            self.peek_at(1).map(|t| &t.kind),
                            Some(TokenKind::Punct('='))
                        )
                    {
                        self.pos += 2; // ] =
                        let value = self.parse_expr()?;
                        let span = Span::new(start, value.span.end);
                        if let Some(tail_end) = self.finish_stmt_line() {
                            return Ok(Stmt::Opaque {
                                span: Span::new(start, tail_end),
                            });
                        }
                        return Ok(Stmt::IndexAssign {
                            base: name,
                            key,
                            value,
                            span,
                        });
                    }
                    // Not the modeled shape: rewind and skip the line as opaque.
                    self.pos = checkpoint;
                    return self.opaque_stmt(start);
                }
                // NAME ( args )
                Some(TokenKind::Punct('(')) => {
                    let call = self.parse_call(name, name_span, 0)?;
                    if let Some(tail_end) = self.finish_stmt_line() {
                        return Ok(Stmt::Opaque {
                            span: Span::new(start, tail_end),
                        });
                    }
                    return Ok(Stmt::Call(call));
                }
                _ => return self.opaque_stmt(start),
            }
        }
        self.opaque_stmt(start)
    }

    /// Consume the rest of the logical line (anything before the next depth-0 newline) so a
    /// trailing construct the subset does not model cannot desynchronize statement parsing.
    /// Returns the byte end of the last consumed NON-newline token, if any were consumed —
    /// the caller MUST then demote the whole statement to [`Stmt::Opaque`] (fail-honest:
    /// trailing unmodeled content is never silently dropped; a `X = 1 if c else target(...)`
    /// tail stays visible to detect-lane over-approximation).
    fn finish_stmt_line(&mut self) -> Option<usize> {
        let mut consumed_end: Option<usize> = None;
        while let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::Newline) {
                self.pos += 1;
                return consumed_end;
            }
            consumed_end = Some(token.span.end);
            self.pos += 1;
        }
        consumed_end
    }

    /// Record an unmodeled statement: consume to the next depth-0 newline (the lexer already
    /// joined newlines inside brackets, so this is sound) and keep its exact span.
    fn opaque_stmt(&mut self, start: usize) -> Result<Stmt, ParseError> {
        let mut end = start;
        while let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::Newline) {
                self.pos += 1;
                break;
            }
            end = token.span.end;
            self.pos += 1;
        }
        Ok(Stmt::Opaque {
            span: Span::new(start, end),
        })
    }

    /// Parse a call whose `func` ident is the CURRENT token and `(` is the next.
    fn parse_call(
        &mut self,
        func: String,
        func_span: Span,
        depth: usize,
    ) -> Result<CallExpr, ParseError> {
        // Consume IDENT and '('.
        self.pos += 1;
        let open = match self.bump() {
            Some(Token {
                kind: TokenKind::Punct('('),
                span,
            }) => span.start,
            _ => return Err(self.error("expected '(' after call identifier")),
        };
        let mut args: Vec<Arg> = Vec::new();
        loop {
            let Some(token) = self.peek() else {
                return Err(ParseError {
                    offset: open,
                    message: format!("unterminated call `{func}(` — no matching `)`"),
                });
            };
            if let TokenKind::Punct(')') = token.kind {
                let close = token.span.start;
                self.pos += 1;
                return Ok(CallExpr {
                    func,
                    func_span,
                    span: Span::new(func_span.start, close + 1),
                    open_paren: open,
                    close_paren: close,
                    args,
                });
            }
            // kwarg: IDENT '=' (not '==' — the lexer emits Op("==") so a bare '=' is unambiguous).
            let arg_start = token.span.start;
            let (name, name_span) = if let TokenKind::Ident(kw) = &token.kind {
                if matches!(
                    self.peek_at(1).map(|t| &t.kind),
                    Some(TokenKind::Punct('='))
                ) {
                    let kw = kw.clone();
                    let kw_span = token.span;
                    self.pos += 2;
                    (Some(kw), Some(kw_span))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };
            let value = self.parse_expr_at_depth(depth + 1)?;
            let mut span = Span::new(arg_start, value.span.end);
            if span.end < span.start {
                span = Span::new(arg_start, arg_start);
            }
            // Optional comma.
            let comma = if let Some(Token {
                kind: TokenKind::Punct(','),
                span: comma_span,
            }) = self.peek()
            {
                let at = comma_span.start;
                self.pos += 1;
                Some(at)
            } else {
                None
            };
            args.push(Arg {
                name,
                name_span,
                value,
                span,
                comma,
            });
            // After an argument: `)` ends, comma already consumed; anything else is a shape we
            // cannot soundly attribute — consume balanced into an opaque tail argument.
            if comma.is_none()
                && let Some(token) = self.peek()
                && !matches!(token.kind, TokenKind::Punct(')'))
            {
                let tail = self.consume_opaque_until_terminator()?;
                let tail_comma = self.eat_comma();
                if let Some(tail_span) = tail {
                    args.push(Arg {
                        name: None,
                        name_span: None,
                        value: ExprNode {
                            expr: Expr::Opaque,
                            span: tail_span,
                        },
                        span: tail_span,
                        comma: tail_comma,
                    });
                }
            }
        }
    }

    /// Parse an expression: a `+`-chain of primaries. A postfix the subset does not model
    /// (`.method(...)`, `[index]`, `% fmt`) widens the node to Opaque (exact span, no guess).
    fn parse_expr(&mut self) -> Result<ExprNode, ParseError> {
        self.parse_expr_at_depth(0)
    }

    fn parse_expr_at_depth(&mut self, depth: usize) -> Result<ExprNode, ParseError> {
        self.check_expr_depth(
            depth,
            self.peek().map(|t| t.span.start).unwrap_or(self.text_len),
        )?;
        let first = self.parse_primary_at_depth(depth)?;
        let mut operands = vec![first];
        loop {
            match self.peek().map(|t| &t.kind) {
                Some(TokenKind::Punct('+')) => {
                    self.pos += 1;
                    operands.push(self.parse_primary_at_depth(depth)?);
                }
                // Unmodeled postfix/binary shape: widen to opaque, consume to terminator.
                Some(TokenKind::Punct('.'))
                | Some(TokenKind::Punct('%'))
                | Some(TokenKind::Punct('['))
                | Some(TokenKind::Punct('(')) => {
                    let start = operands
                        .first()
                        .map(|node| node.span.start)
                        .unwrap_or(self.text_len);
                    let tail = self.consume_opaque_until_terminator()?;
                    let end = tail
                        .map(|s| s.end)
                        .or_else(|| operands.last().map(|node| node.span.end))
                        .unwrap_or(start);
                    return Ok(ExprNode {
                        expr: Expr::Opaque,
                        span: Span::new(start, end),
                    });
                }
                _ => break,
            }
        }
        if operands.len() == 1 {
            // Single operand: hand back the node itself. The Vec holds exactly one element by
            // the check above; `pop` is the no-panic accessor (ADR-0083 Tier-3).
            return operands.pop().ok_or_else(|| ParseError {
                offset: self.text_len,
                message: "internal: empty operand chain".to_owned(),
            });
        }
        let start = operands.first().map(|node| node.span.start).unwrap_or(0);
        let end = operands.last().map(|node| node.span.end).unwrap_or(start);
        Ok(ExprNode {
            expr: Expr::Plus(operands),
            span: Span::new(start, end),
        })
    }

    fn parse_primary_at_depth(&mut self, depth: usize) -> Result<ExprNode, ParseError> {
        let Some(token) = self.peek() else {
            return Err(self.error("expected an expression"));
        };
        let span = token.span;
        match &token.kind {
            TokenKind::Str(value) => {
                let value = value.clone();
                self.pos += 1;
                Ok(ExprNode {
                    expr: Expr::Str(value),
                    span,
                })
            }
            TokenKind::Int(raw) => {
                let raw = raw.clone();
                self.pos += 1;
                Ok(ExprNode {
                    expr: Expr::Int(raw),
                    span,
                })
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                if matches!(
                    self.peek_at(1).map(|t| &t.kind),
                    Some(TokenKind::Punct('('))
                ) {
                    let call = self.parse_call(name, span, depth)?;
                    let call_span = call.span;
                    return Ok(ExprNode {
                        expr: Expr::Call(Box::new(call)),
                        span: call_span,
                    });
                }
                self.pos += 1;
                Ok(ExprNode {
                    expr: Expr::Ident(name),
                    span,
                })
            }
            TokenKind::Punct('[') => self.parse_list(depth),
            TokenKind::Punct('{') => self.parse_dict(depth),
            TokenKind::Punct('(') => {
                // Parenthesized expression: parse the inner expr; the span covers the parens.
                let open = span.start;
                self.pos += 1;
                let inner = self.parse_expr_at_depth(depth + 1)?;
                if let Some(Token {
                    kind: TokenKind::Punct(')'),
                    span: close_span,
                }) = self.peek()
                {
                    let end = close_span.end;
                    self.pos += 1;
                    return Ok(ExprNode {
                        expr: inner.expr,
                        span: Span::new(open, end),
                    });
                }
                Err(ParseError {
                    offset: open,
                    message: "unterminated parenthesized expression".to_owned(),
                })
            }
            // Tokens that can NEVER begin an expression are hard errors — this is what makes a
            // double comma (`deps = ["a",, "b"]`, the historical comment-blind-fixer corruption
            // shape) structurally unparseable, so the harness reparse refuses it.
            TokenKind::Punct(',')
            | TokenKind::Punct(')')
            | TokenKind::Punct(']')
            | TokenKind::Punct('}')
            | TokenKind::Punct(':')
            | TokenKind::Punct('=')
            | TokenKind::Newline => Err(self.error(format!(
                "expected an expression, found `{}`",
                match &token.kind {
                    TokenKind::Punct(c) => c.to_string(),
                    _ => "newline".to_owned(),
                }
            ))),
            _ => {
                // Unmodeled primary: consume balanced to the terminator as one opaque node.
                let start = span.start;
                let tail = self.consume_opaque_until_terminator()?;
                let end = tail.map(|s| s.end).unwrap_or(span.end);
                Ok(ExprNode {
                    expr: Expr::Opaque,
                    span: Span::new(start, end),
                })
            }
        }
    }

    fn parse_list(&mut self, depth: usize) -> Result<ExprNode, ParseError> {
        let open = match self.bump() {
            Some(Token {
                kind: TokenKind::Punct('['),
                span,
            }) => span.start,
            _ => return Err(self.error("expected '['")),
        };
        let mut elements = Vec::new();
        loop {
            let Some(token) = self.peek() else {
                return Err(ParseError {
                    offset: open,
                    message: "unterminated list literal — no matching `]`".to_owned(),
                });
            };
            if let TokenKind::Punct(']') = token.kind {
                let close = token.span.start;
                self.pos += 1;
                return Ok(ExprNode {
                    expr: Expr::List(ListExpr {
                        elements,
                        open_bracket: open,
                        close_bracket: close,
                    }),
                    span: Span::new(open, close + 1),
                });
            }
            let value = self.parse_expr_at_depth(depth + 1)?;
            let comma = if let Some(Token {
                kind: TokenKind::Punct(','),
                span: comma_span,
            }) = self.peek()
            {
                let at = comma_span.start;
                self.pos += 1;
                Some(at)
            } else {
                None
            };
            elements.push(ListElement { value, comma });
            if comma.is_none() {
                // Next must be `]`; anything else is an unmodeled tail — consume it opaquely.
                if let Some(token) = self.peek()
                    && !matches!(token.kind, TokenKind::Punct(']'))
                {
                    let tail = self.consume_opaque_until_terminator()?;
                    let tail_comma = self.eat_comma();
                    if let Some(tail_span) = tail {
                        elements.push(ListElement {
                            value: ExprNode {
                                expr: Expr::Opaque,
                                span: tail_span,
                            },
                            comma: tail_comma,
                        });
                    }
                }
            }
        }
    }

    fn parse_dict(&mut self, depth: usize) -> Result<ExprNode, ParseError> {
        let open = match self.bump() {
            Some(Token {
                kind: TokenKind::Punct('{'),
                span,
            }) => span.start,
            _ => return Err(self.error("expected '{'")),
        };
        let mut entries: Vec<DictEntry> = Vec::new();
        let mut comprehension: Option<DictComp> = None;
        loop {
            let Some(token) = self.peek() else {
                return Err(ParseError {
                    offset: open,
                    message: "unterminated dict literal — no matching `}`".to_owned(),
                });
            };
            if let TokenKind::Punct('}') = token.kind {
                let close = token.span.start;
                self.pos += 1;
                return Ok(ExprNode {
                    expr: Expr::Dict(DictExpr {
                        entries,
                        comprehension,
                        open_brace: open,
                        close_brace: close,
                    }),
                    span: Span::new(open, close + 1),
                });
            }
            let entry_start = token.span.start;
            let key = self.parse_expr_at_depth(depth + 1)?;
            if !matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Punct(':'))) {
                // Not `key: value` — consume to the closing brace as opaque content.
                self.consume_opaque_until_terminator()?;
                self.eat_comma();
                continue;
            }
            self.pos += 1; // ':'
            let value = self.parse_expr_at_depth(depth + 1)?;
            // Comprehension: `for VAR in ITER` after the first key:value.
            if let Some(Token {
                kind: TokenKind::Ident(kw),
                ..
            }) = self.peek()
                && kw == "for"
                && entries.is_empty()
            {
                self.pos += 1;
                let var = match self.bump() {
                    Some(Token {
                        kind: TokenKind::Ident(v),
                        ..
                    }) => v.clone(),
                    _ => String::new(),
                };
                let iter = if matches!(
                    self.peek().map(|t| &t.kind),
                    Some(TokenKind::Ident(kw2)) if kw2 == "in"
                ) {
                    self.pos += 1;
                    match self.peek().map(|t| &t.kind) {
                        Some(TokenKind::Ident(iter_name)) => {
                            let iter_name = iter_name.clone();
                            self.pos += 1;
                            // Anything between the iter ident and `}` (an `if` clause, a
                            // method call) makes the comprehension unresolvable.
                            if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Punct('}'))) {
                                Some(iter_name)
                            } else {
                                self.consume_opaque_until_terminator()?;
                                None
                            }
                        }
                        _ => {
                            self.consume_opaque_until_terminator()?;
                            None
                        }
                    }
                } else {
                    self.consume_opaque_until_terminator()?;
                    None
                };
                comprehension = Some(DictComp {
                    key: Box::new(key),
                    value: Box::new(value),
                    var,
                    iter,
                });
                continue;
            }
            let comma = if let Some(Token {
                kind: TokenKind::Punct(','),
                span: comma_span,
            }) = self.peek()
            {
                let at = comma_span.start;
                self.pos += 1;
                Some(at)
            } else {
                None
            };
            let span = Span::new(entry_start, value.span.end);
            entries.push(DictEntry {
                key,
                value,
                span,
                comma,
            });
            if comma.is_none()
                && let Some(token) = self.peek()
                && !matches!(token.kind, TokenKind::Punct('}'))
            {
                self.consume_opaque_until_terminator()?;
                self.eat_comma();
            }
        }
    }

    /// Consume tokens, balancing brackets, until the enclosing terminator: a `,` at relative
    /// depth 0, a closing bracket at relative depth 0 (NOT consumed), a depth-0 newline (NOT
    /// consumed), or end of input. Returns the consumed span, if anything was consumed.
    /// Unbalanced closers beyond depth 0 are terminators (left for the caller); running out of
    /// input while inside an opened bracket is a hard error (fail-closed).
    fn consume_opaque_until_terminator(&mut self) -> Result<Option<Span>, ParseError> {
        let mut depth: usize = 0;
        let mut consumed: Option<Span> = None;
        loop {
            let Some(token) = self.peek() else {
                if depth > 0 {
                    return Err(ParseError {
                        offset: self.text_len,
                        message: "unbalanced brackets in unmodeled expression".to_owned(),
                    });
                }
                return Ok(consumed);
            };
            match &token.kind {
                TokenKind::Newline => return Ok(consumed),
                TokenKind::Punct('(') | TokenKind::Punct('[') | TokenKind::Punct('{') => {
                    self.check_expr_depth(depth + 1, token.span.start)?;
                    depth += 1;
                }
                TokenKind::Punct(')') | TokenKind::Punct(']') | TokenKind::Punct('}') => {
                    if depth == 0 {
                        return Ok(consumed);
                    }
                    depth -= 1;
                }
                TokenKind::Punct(',') if depth == 0 => return Ok(consumed),
                _ => {}
            }
            let span = token.span;
            consumed = Some(match consumed {
                Some(existing) => Span::new(existing.start, span.end),
                None => span,
            });
            self.pos += 1;
        }
    }
}
