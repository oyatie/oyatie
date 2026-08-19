package main

import (
	"go/ast"
	"go/types"
)

// FOLLOWING A CALL, so that passing a pointer onward stops meaning "nothing is known".
//
// The ownership pass is intra-procedural, so any argument rooted at the pointer used to make every
// fact about it unproven — and that swallowed almost everything: of the methods across the seven
// surveyed corpora, most carried `effect_unknown` and nothing else, which is the analysis reporting
// that a call happened rather than reporting anything about the pointer. `unproven_owned` has no
// receiver form, so each of those refused.
//
// Three ways a pointer reaches a call, and only one of them is a positional argument:
//
//   - As an ARGUMENT, answered by what the callee does to the parameter it lands in.
//   - As a method call's RECEIVER, which is not in `Args` at all — so `s.helper()` reached the
//     argument loop with nothing rooted at `s` and was silently treated as though the pointer had
//     not been passed anywhere.
//   - To a BUILTIN or a CONVERSION, neither of which has a body to read. What `len` does is a
//     property of the source language rather than a decision, and a conversion reads its operand
//     and does nothing else — the same "a call is not always a call" confusion that once emitted a
//     conversion as a call to a function with no name, met again from the other side.
//
// A callee with no body still yields `effect_unknown`, because nothing was read and "not read"
// must not become "not mutated".

// calleeParameterFacts reports what the callee does to the parameter an argument lands in.
//
// False when the call cannot be followed at all — a callee this corpus does not declare, one with
// no body, a call through a value of function type, or one already on the stack. The caller then
// records `effect_unknown`, which is what an unread callee has always meant.
func calleeParameterFacts(
	call *ast.CallExpr,
	index int,
	ctx *extractCtx,
	seen map[types.Object]bool,
) ([]string, bool) {
	if ctx == nil || ctx.info == nil {
		return nil, false
	}
	ident := calleeIdent(call.Fun)
	if ident == nil {
		return nil, false
	}
	object := ctx.info.Uses[ident]
	if _, isType := object.(*types.TypeName); isType {
		// A CONVERSION, which the source spells exactly like a call. It reads its operand and does
		// nothing else — the same "a call is not always a call" confusion that once emitted a
		// conversion as a call to a function with no name, met again from the other side.
		return nil, true
	}
	if _, isBuiltin := object.(*types.Builtin); isBuiltin {
		// A builtin has no body, and for the read-only ones that is not an obstacle: what they do
		// is known without reading anything. The writing ones are absent from the set and fall
		// through to unproven.
		if readOnlyBuiltins[ident.Name] {
			return nil, true
		}
		return nil, false
	}
	fn, ok := object.(*types.Func)
	if !ok || seen[object] {
		return nil, false
	}
	body := ctx.bodies[object]
	if body == nil {
		return nil, false
	}
	sig, ok := fn.Type().(*types.Signature)
	if !ok || sig.Params() == nil || index >= sig.Params().Len() {
		// A variadic tail, or a signature that does not line up with the call. Neither is a
		// parameter this can name, so nothing is claimed about it.
		return nil, false
	}
	parameter := sig.Params().At(index).Name()
	if parameter == "" || parameter == "_" {
		// The callee ignores it by not naming it, which proves nothing about what it does — the
		// name is how this pass finds the uses.
		return nil, false
	}

	seen[object] = true
	facts := ownershipFactsSeen(body, parameter, ctx, seen)
	delete(seen, object)
	return facts, true
}

// calleeIdent is the identifier a call names, for a plain call or a qualified one.
//
// Nil for a call through anything else — a value of function type, a method value, an immediately
// invoked literal — none of which names a declaration whose body could be read.
func calleeIdent(fun ast.Expr) *ast.Ident {
	switch typed := fun.(type) {
	case *ast.Ident:
		return typed
	case *ast.SelectorExpr:
		return typed.Sel
	default:
		return nil
	}
}

// readOnlyBuiltins are the source builtins that only READ the argument handed to them.
//
// Stated here rather than declared as pack data: what `len` does is a property of the source
// language, like an operator's meaning, and nobody gets to decide it. The ones deliberately absent
// are the ones that WRITE — `copy` fills its first argument, `delete` removes from its map, `close`
// changes its channel — and those keep leaving the facts unproven, which is what an unexamined
// write has always meant here.
var readOnlyBuiltins = map[string]bool{
	"len":     true,
	"cap":     true,
	"append":  true,
	"complex": true,
	"real":    true,
	"imag":    true,
	"min":     true,
	"max":     true,
	"panic":   true,
	"print":   true,
	"println": true,
}

// calleeReceiverFacts reports what a method call does to the value it is called ON.
//
// False when the call is not a method call on this name, or when the method cannot be followed —
// a foreign type's method, an interface method with no body, or one already on the stack.
func calleeReceiverFacts(
	call *ast.CallExpr,
	name string,
	ctx *extractCtx,
	seen map[types.Object]bool,
) ([]string, bool) {
	selector, ok := call.Fun.(*ast.SelectorExpr)
	if !ok || rootIdent(selector.X) != name || ctx == nil || ctx.info == nil {
		return nil, false
	}
	object := ctx.info.Uses[selector.Sel]
	fn, ok := object.(*types.Func)
	if !ok || seen[object] {
		return nil, false
	}
	sig, ok := fn.Type().(*types.Signature)
	if !ok || sig.Recv() == nil {
		return nil, false
	}
	body := ctx.bodies[object]
	if body == nil {
		return nil, false
	}
	receiver := sig.Recv().Name()
	if receiver == "" || receiver == "_" {
		return nil, false
	}

	seen[object] = true
	facts := ownershipFactsSeen(body, receiver, ctx, seen)
	delete(seen, object)
	return facts, true
}
