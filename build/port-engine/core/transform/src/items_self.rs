//! Spelling the implementing type as `Self` inside its own impl block.
//!
//! Not merely shorter. `Self` survives a rename, where the name written twice has two places to
//! miss — and a constructor moved onto its type names that type twice, in the result and in the
//! literal it builds. The source has no such spelling and therefore always writes the name.
//!
//! Structural rather than textual: the result is a resolved type and the literal carries its path,
//! so a name that merely resembles the type is not touched.

use port_engine_rust_ir::{RustExpr, RustFn, RustStmt, RustType};

/// Spell the implementing type as `Self` inside its own impl block.
///
/// The RESULT type and any struct literal of that type, which between them are every place a
/// constructor names the type it builds. Structural rather than textual: the result is a resolved
/// type and the literal carries its path, so a name that merely resembles the type is not touched.
pub(crate) fn rename_own_type(mut rendered: RustFn, self_ty: &str, spelling: &str) -> RustFn {
    if rendered
        .ret
        .as_ref()
        .is_some_and(|ret| ret.spelling() == self_ty)
    {
        rendered.ret = Some(RustType::path(spelling.to_owned()));
    }
    rendered.body = rendered.body.map(|body| {
        body.into_iter()
            .map(|statement| rename_in_statement(statement, self_ty, spelling))
            .collect()
    });
    rendered
}

fn rename_in_statement(statement: RustStmt, self_ty: &str, spelling: &str) -> RustStmt {
    match statement {
        RustStmt::Tail(expr) => RustStmt::Tail(rename_in_expr(expr, self_ty, spelling)),
        RustStmt::Return(Some(expr)) => {
            RustStmt::Return(Some(rename_in_expr(expr, self_ty, spelling)))
        }
        other => other,
    }
}

fn rename_in_expr(expr: RustExpr, self_ty: &str, spelling: &str) -> RustExpr {
    match expr {
        RustExpr::StructLiteral { path, fields } if path == self_ty => RustExpr::StructLiteral {
            path: spelling.to_owned(),
            fields,
        },
        other => other,
    }
}
