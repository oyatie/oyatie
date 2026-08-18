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
		return node{Kind: kindReturn, Children: expressionNodes(typed.Results, ctx)}

	case *ast.BlockStmt:
		return node{Kind: kindBlock, Children: statementNodes(typed.List, ctx)}

	case *ast.IfStmt:
		// An `if` with an init statement (`if x := f(); x != nil`) scopes a binding to the
		// condition, which Rust has no direct form for. Recorded as unsupported rather than
		// silently hoisted, because hoisting changes the binding's lifetime.
		if typed.Init != nil {
			return unsupportedNode(stmt)
		}
		children := []node{
			{Kind: "cond", Children: []node{expressionNode(typed.Cond, ctx)}},
			{Kind: kindThen, Children: statementNodes(typed.Body.List, ctx)},
		}
		if typed.Else != nil {
			children = append(children, node{
				Kind:     kindElse,
				Children: []node{statementNode(typed.Else, ctx)},
			})
		}
		return node{Kind: kindIf, Children: children}

	case *ast.AssignStmt:
		// Multi-assignment and the op-assign forms each carry a tuple-destructuring or
		// read-modify-write question that needs a rule rather than a default.
		if len(typed.Lhs) != 1 || len(typed.Rhs) != 1 {
			return unsupportedNode(stmt)
		}
		switch typed.Tok {
		case token.DEFINE:
			name, ok := typed.Lhs[0].(*ast.Ident)
			if !ok {
				return unsupportedNode(stmt)
			}
			return node{
				Kind:     kindLet,
				Name:     name.Name,
				Children: []node{expressionNode(typed.Rhs[0], ctx)},
			}
		case token.ASSIGN:
			return node{
				Kind: kindAssign,
				Children: []node{
					expressionNode(typed.Lhs[0], ctx),
					expressionNode(typed.Rhs[0], ctx),
				},
			}
		default:
			return unsupportedNode(stmt)
		}

	case *ast.ExprStmt:
		return node{Kind: kindExprStmt, Children: []node{expressionNode(typed.X, ctx)}}

	case *ast.BranchStmt:
		// `break` maps directly. `continue` does NOT, because a three-clause loop lowers to a
		// `while` whose post-statement a `continue` would skip — a different program. `goto` and
		// labelled breaks have no target form at all.
		if typed.Tok == token.BREAK && typed.Label == nil {
			return node{Kind: kindBreak}
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
