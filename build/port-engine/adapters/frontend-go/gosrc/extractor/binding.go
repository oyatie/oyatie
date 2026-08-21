package main

import (
	"go/ast"
	"go/token"
	"go/types"
	"sort"
)

// Statements that INTRODUCE a name or WRITE one.
//
// Two questions live here that the statement walk should not have to hold. The first is whether a
// binding is written again: the source makes every binding mutable and the target makes none of
// them, so assuming either way is wrong for half the bindings in any real body — and the question
// has to be asked of `x := e` and of each name a destructuring bind introduces, not only of `var`.
//
// The second is what a compound assignment carries. `x op= y` means `x = x op y` in both languages
// and evaluates the place expression ONCE in both, so it is an assignment carrying an operator
// rather than a construct of its own — and the one form with no target spelling is recorded with
// its operator anyway, so the refusal downstream can name it.

// assignmentNode records `:=`, `=` and the read-modify-write forms.
func assignmentNode(stmt *ast.AssignStmt, ctx *extractCtx) node {
	// A DESTRUCTURING bind takes several names from one expression. It is the shape every
	// fallible call in the source has, so it is recorded rather than refused — what the target
	// does with it is a rule, and a rule needs the shape to reach it.
	if stmt.Tok == token.DEFINE && len(stmt.Rhs) == 1 && len(stmt.Lhs) > 1 {
		return destructuringBind(stmt, ctx)
	}
	// A PARALLEL assignment writes several places at once. Recorded rather than refused for the
	// same reason the destructuring bind is: the shape has to reach the transform for a rule to
	// answer it, and "some assignment" is not a refusal anybody can act on.
	if stmt.Tok == token.ASSIGN && (len(stmt.Lhs) > 1 || len(stmt.Rhs) > 1) {
		return parallelAssignment(stmt, ctx)
	}
	// The remaining op-assign forms carry a question — read-modify-write against several
	// places — that needs a rule rather than a default.
	if len(stmt.Lhs) != 1 || len(stmt.Rhs) != 1 {
		return unsupportedNode(stmt)
	}
	switch stmt.Tok {
	case token.DEFINE:
		name, ok := stmt.Lhs[0].(*ast.Ident)
		if !ok {
			return unsupportedNode(stmt)
		}
		return node{
			Kind: kindLet,
			Name: name.Name,
			// The BOUND TYPE, which `var x T` already carries and `x := e` did not. What needs it:
			// whether the binding has a drop to delay, which is what decides whether the block that
			// scopes it is necessary or is the source's statement form transliterated.
			Type: typeTree(bindingType(name, ctx)),
			// The SHORT declaration needs the same question asked of it as `var` does. It was
			// not asked, so every `x := e` the body later assigned emitted an immutable
			// binding followed by a write to it — output that does not compile. No fixture
			// had the shape, which is the whole argument for ratcheting against real source.
			// INFERRED: the short declaration writes no type, so the target must not either.
			// Sorted after appending, because a flag set has exactly one encoding and the digest
			// is taken over it.
			Flags:    withFlag(bindingFlags(name, ctx), flagInferred),
			Children: []node{expressionNode(stmt.Rhs[0], ctx)},
		}
	case token.ASSIGN:
		return node{
			Kind: kindAssign,
			Children: []node{
				expressionNode(stmt.Lhs[0], ctx),
				expressionNode(stmt.Rhs[0], ctx),
			},
		}
	default:
		// A COMPOUND assignment is an assignment carrying an operator, and the operator is the
		// same datum a binary expression carries. Recorded that way rather than as its own
		// kind: `x += y` and `x = x + y` differ in the source only by evaluating the place
		// once, which is also true of the target, so a second kind would describe a difference
		// neither language has.
		//
		// `&^=` has no spelling here for the same reason binary `&^` has none, and reaches the
		// transform as an operator no rule answers for — refused by name rather than silently
		// rewritten into `& !`, which is a bit operation nobody reviews.
		op := compoundOperator(stmt.Tok)
		if op == "" {
			return unsupportedNode(stmt)
		}
		return node{
			Kind: kindAssign,
			// The ASSIGNED type, for the same reason a binary expression carries its result type:
			// the source's integer overflow wraps and the target's does not, and `*=` on integers
			// is a different operation from `*=` on floats.
			Type:  typeTree(ctx.info.Types[stmt.Lhs[0]].Type),
			Attrs: map[string]string{attrOp: op},
			Children: []node{
				expressionNode(stmt.Lhs[0], ctx),
				expressionNode(stmt.Rhs[0], ctx),
			},
		}
	}
}

// parallelAssignment records `a, b = x, y` and `a, b = f()` as the places it writes and the values
// it writes from, in that order.
//
// The source evaluates every operand on both sides BEFORE assigning any of them, which is what
// makes `a[i], a[j] = a[j], a[i]` a swap rather than two writes. The target's destructuring
// assignment has the same rule, so the shape carries across — but only where the LHS places are
// themselves side-effect free, because the two languages order the evaluation of a place's own
// subexpressions differently and a place reached through a CALL would be run at a different time.
func parallelAssignment(stmt *ast.AssignStmt, ctx *extractCtx) node {
	out := node{Kind: kindAssignTuple}
	for _, lhs := range stmt.Lhs {
		if !isSimplePlace(lhs) {
			return unsupportedNode(stmt)
		}
		out.Children = append(out.Children, node{
			Kind:     kindPlace,
			Children: []node{expressionNode(lhs, ctx)},
		})
	}
	for _, rhs := range stmt.Rhs {
		out.Children = append(out.Children, node{
			Kind:     kindValue,
			Children: []node{expressionNode(rhs, ctx)},
		})
	}
	return out
}

// isSimplePlace reports whether writing to this place runs no code of the program's own.
//
// A name, a field of one, and an index by a name or a literal. Deliberately narrow: the source
// evaluates every place's own subexpressions before any assignment happens and the target evaluates
// them at the assignment, so a place whose subexpressions have EFFECTS would run them at a
// different time. Nothing here has any.
func isSimplePlace(expr ast.Expr) bool {
	switch typed := expr.(type) {
	case *ast.Ident:
		return true
	case *ast.SelectorExpr:
		return isSimplePlace(typed.X)
	case *ast.IndexExpr:
		if !isSimplePlace(typed.X) {
			return false
		}
		switch typed.Index.(type) {
		case *ast.Ident, *ast.BasicLit:
			return true
		default:
			return false
		}
	default:
		return false
	}
}

// withFlag adds one flag and restores the sorted order a flag set is encoded in.
func withFlag(flags []string, flag string) []string {
	out := append(flags, flag)
	sort.Strings(out)
	return out
}

// bindingType is the type the type-checker gave this binding, or nil where it gave none.
//
// A short declaration DEFINES its name, so `Defs` is where the object is; `Uses` would find a
// different binding of the same name in an outer scope, which is a different variable.
func bindingType(name *ast.Ident, ctx *extractCtx) types.Type {
	obj := ctx.info.Defs[name]
	if obj == nil {
		return nil
	}
	return obj.Type()
}

// destructuringBind records `a, b := expr` as the names it binds and the expression they come
// from, in that order.
func destructuringBind(stmt *ast.AssignStmt, ctx *extractCtx) node {
	out := node{Kind: kindLetTuple}
	for _, lhs := range stmt.Lhs {
		name, ok := lhs.(*ast.Ident)
		if !ok {
			return unsupportedNode(stmt)
		}
		// Each name a destructuring bind introduces is a binding like any other, and `err` being
		// reassigned by a later call is the single most common shape in the source language.
		out.Children = append(out.Children, node{
			Kind:  kindBind,
			Name:  name.Name,
			Flags: bindingFlags(name, ctx),
		})
	}
	out.Children = append(out.Children, node{
		Kind:     kindValue,
		Children: []node{expressionNode(stmt.Rhs[0], ctx)},
	})
	return out
}

// compoundOperator spells the BINARY operator inside a read-modify-write assignment.
//
// Empty for anything with no binary spelling of its own — `&^=`, and `:=`/`=` which are not
// compound at all — so the caller records an `unsupported` naming the statement instead.
func compoundOperator(tok token.Token) string {
	switch tok {
	case token.ADD_ASSIGN:
		return "+"
	case token.SUB_ASSIGN:
		return "-"
	case token.MUL_ASSIGN:
		return "*"
	case token.QUO_ASSIGN:
		return "/"
	case token.REM_ASSIGN:
		return "%"
	case token.AND_ASSIGN:
		return "&"
	case token.OR_ASSIGN:
		return "|"
	case token.XOR_ASSIGN:
		return "^"
	case token.SHL_ASSIGN:
		return "<<"
	case token.SHR_ASSIGN:
		return ">>"
	case token.AND_NOT_ASSIGN:
		// `&^=` has no target form, and it is recorded anyway. The extractor models the SOURCE;
		// dropping the operator here would leave the transform refusing an `AssignStmt` without
		// being able to say which one, and "some assignment" is not a refusal anybody can act on.
		return "&^"
	default:
		return ""
	}
}

// bindingFlags reports what a binding needs from the target, which today is only whether it is
// written again.
//
// Observed rather than assumed. Every binding in the source is mutable and most are never written
// again: assuming mutable warns on each one, and assuming immutable fails to compile the first time
// one is assigned. Only the body knows which, so the body is asked.
func bindingFlags(name *ast.Ident, ctx *extractCtx) []string {
	object := ctx.info.Defs[name]
	// EITHER FACT makes an owned binding mutable: a write to the name, or a write through an index
	// into what it holds. They are recorded apart because a PARAMETER needs different things of
	// them -- see `indexWrittenLocals`.
	if object == nil {
		return nil
	}
	rebound := ctx.assigned != nil && ctx.assigned[object]
	mutated := ctx.indexWritten != nil && ctx.indexWritten[object]
	if !rebound && !mutated {
		return nil
	}
	return []string{flagMutated}
}
