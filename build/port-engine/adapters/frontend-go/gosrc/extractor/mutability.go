package main

import (
	"go/ast"
	"go/token"
	"go/types"
)

// Which of a body's own bindings it writes again.
//
// Every binding in the source is mutable; the target makes immutability the default and asks for
// mutability explicitly. Assuming one or the other is wrong in both directions — assume mutable and
// every binding warns, assume immutable and the first assignment fails to compile — and it is not a
// judgement call, because the body says which.
//
// Counted are ASSIGNMENTS and the increment/decrement forms, plus taking a pointer to the binding,
// since a pointer is a licence to write through it. A `:=` is a new binding rather than a write, so
// it is not counted; shadowing therefore reads as what it is, a second binding.

// assignedLocals reports every object the body writes after binding it.
func assignedLocals(body *ast.BlockStmt, ctx *extractCtx) map[types.Object]bool {
	assigned := map[types.Object]bool{}
	mark := func(expr ast.Expr) {
		ident, ok := expr.(*ast.Ident)
		if !ok {
			return
		}
		if object := ctx.info.Uses[ident]; object != nil {
			assigned[object] = true
		}
	}

	ast.Inspect(body, func(n ast.Node) bool {
		switch typed := n.(type) {
		case *ast.AssignStmt:
			// `:=` BINDS rather than writes. Counting it would mark every binding mutable, which
			// is the assumption this pass exists to replace.
			if typed.Tok == token.DEFINE {
				return true
			}
			for _, lhs := range typed.Lhs {
				mark(lhs)
			}
		case *ast.IncDecStmt:
			mark(typed.X)
		case *ast.UnaryExpr:
			// `&x` hands out a licence to write through the binding, and the write itself may be
			// anywhere. Treating it as a write is the conservative reading, and conservative here
			// costs a warning where being wrong costs a compile error.
			if typed.Op == token.AND {
				mark(typed.X)
			}
		case *ast.RangeStmt:
			// `for i, v = range xs` assigns into existing bindings; `:=` again binds.
			if typed.Tok != token.DEFINE {
				mark(typed.Key)
				mark(typed.Value)
			}
		}
		return true
	})
	return assigned
}
