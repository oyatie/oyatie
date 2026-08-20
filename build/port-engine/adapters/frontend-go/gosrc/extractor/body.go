package main

import (
	"go/ast"
	"go/token"
)

// Statements.
//
// The body walk is deliberately SMALL and deliberately COMPLETE. Small, because only a few forms
// have a translation the engine can defend today. Complete, because everything else is still
// recorded — as an `unsupported` node naming the Go AST type it stands for — rather than dropped.
// A dropped construct would make an untranslatable function indistinguishable from an empty one.

// ---------------------------------------------------------------------------------
// Bodies
// ---------------------------------------------------------------------------------
//
// The body walk is deliberately SMALL and deliberately COMPLETE. Small, because only a few
// statement and expression forms have a translation the engine can defend today. Complete,
// because everything else is still recorded — as an `unsupported` node naming the Go AST
// type it stands for — rather than dropped. A dropped construct would make an
// untranslatable function indistinguishable from an empty one, and the engine would emit a
// green, silently wrong body. Recorded, it becomes a refusal the transform can name.

func bodyNode(block *ast.BlockStmt, ctx *extractCtx) node {
	return node{Kind: kindBody, Children: statementNodes(block.List, ctx)}
}

// localDeclaration records `var x T`, `var x = e`, `var x T = e` and `const x = e` inside a body.
//
// A single-name spec only. A grouped `var ( a = 1; b = 2 )` is several bindings in one statement,
// and a statement list that silently gained entries would make the tail-expression position — which
// is decided by INDEX — mean something different from what the source wrote.
//
// A body-scoped CONST is recorded as the same binding, and that is a decision about what it means
// rather than about what it is called. Go's untyped constant has no type until it is used and takes
// one from each use; a target `const` must fix a type at the declaration, and a target `let` takes
// one from use exactly as the source's does. So the binding is the faithful form, and the cost is
// stated where the reference is cased: a source constant used at TWO different types in one
// function has no single target binding, and fails to compile rather than meaning something else.
func localDeclaration(stmt *ast.DeclStmt, ctx *extractCtx) node {
	decl, ok := stmt.Decl.(*ast.GenDecl)
	if !ok || (decl.Tok != token.VAR && decl.Tok != token.CONST) || len(decl.Specs) != 1 {
		return unsupportedNode(stmt)
	}
	spec, ok := decl.Specs[0].(*ast.ValueSpec)
	if !ok || len(spec.Names) != 1 || len(spec.Values) > 1 {
		return unsupportedNode(stmt)
	}

	out := node{
		Kind:  kindLet,
		Name:  spec.Names[0].Name,
		Flags: bindingFlags(spec.Names[0], ctx),
	}
	if spec.Type != nil {
		out.Type = typeTree(ctx.info.TypeOf(spec.Type))
	}
	if len(spec.Values) == 1 {
		out.Children = []node{expressionNode(spec.Values[0], ctx)}
	} else if spec.Type != nil {
		// `var x T` INITIALISES. The source guarantees the zero value of the type, so the binding
		// has a value here exactly as `x := T{}` would -- recorded the same way a composite
		// literal's omitted field is, as a `zero` node carrying the type the value comes from.
		//
		// Recording nothing said "a binding the body fills in later", which is what the target's
		// bare `let x: T;` means and is NOT what the source wrote: the target then refuses to read
		// the name on any path that does not assign it first, and the source reads zero there.
		out.Children = []node{{Kind: kindZero, Type: typeTree(ctx.info.TypeOf(spec.Type))}}
	}
	return out
}

func statementNodes(stmts []ast.Stmt, ctx *extractCtx) []node {
	if len(stmts) == 0 {
		return nil
	}
	out := make([]node, 0, len(stmts))
	for _, stmt := range stmts {
		out = append(out, statementNode(stmt, ctx))
	}
	return out
}

func statementNode(stmt ast.Stmt, ctx *extractCtx) node {
	switch typed := stmt.(type) {
	case *ast.ReturnStmt:
		// A function literal among these operands OUTLIVES this frame, which is what decides that
		// its captures must be owned rather than borrowed. Recorded here because this is where the
		// destination is known; `expressionNode` sees only the literal.
		outer := ctx.destination
		ctx.destination = destinationReturn
		results := expressionNodes(typed.Results, ctx)
		ctx.destination = outer
		return node{Kind: kindReturn, Children: results}

	case *ast.BlockStmt:
		return node{Kind: kindBlock, Children: statementNodes(typed.List, ctx)}

	case *ast.IfStmt:
		// An `if` with an init statement (`if x := f(); x != nil`) scopes a binding to the
		// condition and both branches. Recorded as a CHILD, exactly as the `for` loop records its
		// own init clause: the snapshot is a model of the source, and rewriting the shape here
		// would make it a model of the target instead. What the target does with it is a decision
		// the transform makes, where the reason can be written next to the emission.
		children := []node{}
		if typed.Init != nil {
			children = append(children, node{
				Kind:     kindInit,
				Children: []node{statementNode(typed.Init, ctx)},
			})
		}
		children = append(children,
			node{Kind: kindCond, Children: []node{expressionNode(typed.Cond, ctx)}},
			node{Kind: kindThen, Children: statementNodes(typed.Body.List, ctx)},
		)
		if typed.Else != nil {
			children = append(children, node{
				Kind:     kindElse,
				Children: []node{statementNode(typed.Else, ctx)},
			})
		}
		return node{Kind: kindIf, Children: children}

	case *ast.AssignStmt:
		return assignmentNode(typed, ctx)

	case *ast.DeclStmt:
		return localDeclaration(typed, ctx)

	case *ast.IncDecStmt:
		// `x++` is a STATEMENT in the source and has no value, which is why it is recorded as its
		// own kind rather than as an assignment of `x + 1`: the source cannot write `y = x++`, and
		// a shape that says it can would admit a program the source has no way to spell. The
		// operand is a place, so it is recorded as one child and the operator as an attribute.
		return node{
			Kind:     kindIncDec,
			Attrs:    map[string]string{attrOp: typed.Tok.String()},
			Children: []node{expressionNode(typed.X, ctx)},
		}

	case *ast.ExprStmt:
		return node{Kind: kindExprStmt, Children: []node{expressionNode(typed.X, ctx)}}

	case *ast.BranchStmt:
		// `break` and `continue` are RECORDED, not judged. Whether a `continue` is translatable is
		// a property of the LOOP that encloses it — a loop whose post-statement the target spells
		// inside the body would skip it, and one that has no post-statement, or spends it building
		// a range, would not. The extractor cannot answer that without deciding which target loop
		// the enclosing `for` becomes, which is the transform's decision; recording the branch and
		// letting the loop refuse keeps the answer in one place. `goto` and labelled branches have
		// no target form at all.
		if typed.Label == nil {
			switch typed.Tok {
			case token.BREAK:
				return node{Kind: kindBreak}
			case token.CONTINUE:
				return node{Kind: kindContinue}
			}
		}
		return unsupportedNode(stmt)

	case *ast.ForStmt:
		return forNode(typed, ctx)

	case *ast.RangeStmt:
		return rangeNode(typed, ctx)

	case *ast.SwitchStmt:
		return switchNode(typed, ctx)

	default:
		return unsupportedNode(stmt)
	}
}

// forNode records a three-clause or condition-only `for`.
//
// The clauses are recorded SEPARATELY rather than pre-lowered, because which target loop they
// deserve is a translation decision: an ascending integer counter is a range, and anything else is
// a `while` whose post-statement has to run on every path.
func forNode(stmt *ast.ForStmt, ctx *extractCtx) node {
	out := node{Kind: kindFor}
	if stmt.Init != nil {
		out.Children = append(out.Children, node{
			Kind:     kindInit,
			Children: []node{statementNode(stmt.Init, ctx)},
		})
	}
	if stmt.Cond != nil {
		out.Children = append(out.Children, node{
			Kind:     kindCond,
			Children: []node{expressionNode(stmt.Cond, ctx)},
		})
	}
	if stmt.Post != nil {
		out.Children = append(out.Children, node{
			Kind:     kindPost,
			Children: []node{statementNode(stmt.Post, ctx)},
		})
	}
	out.Children = append(out.Children, node{
		Kind:     kindThen,
		Children: statementNodes(stmt.Body.List, ctx),
	})
	return out
}

// rangeNode records a `range` loop, with the key and value names it binds.
func rangeNode(stmt *ast.RangeStmt, ctx *extractCtx) node {
	out := node{Kind: kindRange}
	out.Attrs = withAttr(out.Attrs, "key", identName(stmt.Key))
	out.Attrs = withAttr(out.Attrs, "value", identName(stmt.Value))
	out.Children = append(out.Children,
		node{Kind: "over", Children: []node{expressionNode(stmt.X, ctx)}},
		node{Kind: kindThen, Children: statementNodes(stmt.Body.List, ctx)},
	)
	return out
}

// switchNode records an expression switch.
//
// A switch with an init statement, or a TYPE switch, is not recorded as a switch at all: the first
// scopes a binding to the switch and the second dispatches on dynamic type, and neither has a
// target form that a value match reproduces.
func switchNode(stmt *ast.SwitchStmt, ctx *extractCtx) node {
	if stmt.Init != nil {
		return unsupportedNode(stmt)
	}
	out := node{Kind: kindSwitch}
	if stmt.Tag != nil {
		out.Children = append(out.Children, node{
			Kind:     "tag",
			Children: []node{expressionNode(stmt.Tag, ctx)},
		})
	}
	for _, clause := range stmt.Body.List {
		caseClause, ok := clause.(*ast.CaseClause)
		if !ok {
			return unsupportedNode(clause)
		}
		out.Children = append(out.Children, node{
			Kind: kindCase,
			Children: append(
				[]node{{Kind: "patterns", Children: expressionNodes(caseClause.List, ctx)}},
				node{Kind: kindThen, Children: statementNodes(caseClause.Body, ctx)},
			),
		})
	}
	return out
}
