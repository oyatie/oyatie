package main

import (
	"go/ast"
	"go/types"
)

// Composite literals, and the fields the source leaves out.
//
// The source zero-fills what a literal omits; the target rejects an incomplete literal outright.
// Which fields a struct HAS is a fact go/types holds and the engine does not, so the omitted ones
// are recorded here as `zero` nodes carrying their type — leaving the target's spelling of that
// zero to the rule pack.

// compositeNode records a struct literal with every DECLARED field present.
//
// Go fills the fields a literal omits with their type's zero value; the target rejects an
// incomplete literal. Which fields a struct has is a fact go/types holds and the engine does not,
// so the omitted ones are recorded HERE, as `zero` nodes carrying the field's type — leaving the
// target's spelling of that zero to the rule pack.
func compositeNode(lit *ast.CompositeLit, ctx *extractCtx) node {
	fields := compositeStruct(lit, ctx)
	if fields == nil {
		// A slice, map or array literal. RECORDED with its type and its elements rather than
		// refused: what the target spells for a sequence is a pack decision, and a rule cannot
		// fire on a shape the snapshot never carries. An earlier version emitted an empty struct
		// literal here, which silently constructed nothing.
		return sequenceNode(lit, ctx)
	}

	// A POSITIONAL composite gives its values in FIELD ORDER, and that order is a fact go/types
	// holds right here — `fields.Field(i)` is the field element `i` fills. Naming them is a proof
	// rather than a hope, which is what makes it safe: the danger was never the positional form,
	// it was reproducing an order the target does not guarantee, and a named field reproduces no
	// order at all.
	//
	// The source forbids mixing the two forms in one literal, so a literal is entirely keyed or
	// entirely positional and there is nothing to reconcile.
	written := make(map[string]node, len(lit.Elts))
	for index, element := range lit.Elts {
		keyed, ok := element.(*ast.KeyValueExpr)
		if !ok {
			if index >= fields.NumFields() {
				// More values than the struct has fields. go/types would have rejected this, so
				// reaching it means the type behind the literal is not the one indexed here.
				return unsupportedNode(element)
			}
			written[fields.Field(index).Name()] = expressionNode(element, ctx)
			continue
		}
		key, ok := keyed.Key.(*ast.Ident)
		if !ok {
			return unsupportedNode(keyed)
		}
		written[key.Name] = expressionNode(keyed.Value, ctx)
	}

	out := node{Kind: kindComposite, Type: compositeType(lit, ctx)}
	for index := 0; index < fields.NumFields(); index++ {
		field := fields.Field(index)
		value, present := written[field.Name()]
		if !present {
			value = node{Kind: kindZero, Name: field.Name(), Type: typeTree(field.Type())}
		}
		out.Children = append(out.Children, node{
			Kind:     kindKeyed,
			Name:     field.Name(),
			Children: []node{value},
		})
	}
	return out
}

// compositeStruct reports the struct a composite literal constructs, or nil when it constructs
// something else.
func compositeStruct(lit *ast.CompositeLit, ctx *extractCtx) *types.Struct {
	tv, ok := ctx.info.Types[lit]
	if !ok || tv.Type == nil {
		return nil
	}
	underlying := tv.Type.Underlying()
	if pointer, ok := underlying.(*types.Pointer); ok {
		underlying = pointer.Elem().Underlying()
	}
	structured, ok := underlying.(*types.Struct)
	if !ok {
		return nil
	}
	return structured
}

// compositeType records what a composite literal constructs.
func compositeType(lit *ast.CompositeLit, ctx *extractCtx) *typeNode {
	if tv, ok := ctx.info.Types[lit]; ok && tv.Type != nil {
		return typeTree(tv.Type)
	}
	return nil
}

// sequenceNode records a slice, array or map literal.
//
// Its ELEMENTS are children in source order, which is semantic for a sequence and is the order the
// target reproduces. A map's entries are `keyed` nodes carrying key and value as two children; a
// map literal in the source has no order at all, and the target's ordered map imposes one — which
// is a decision the pack makes and not a fact recorded here.
//
// An EMPTY literal keeps its type and gains no children, so the transform can answer it with the
// type's zero value rather than with an empty construction it would have to invent.
func sequenceNode(lit *ast.CompositeLit, ctx *extractCtx) node {
	out := node{Kind: kindComposite, Type: compositeType(lit, ctx)}
	for _, element := range lit.Elts {
		keyed, ok := element.(*ast.KeyValueExpr)
		if !ok {
			out.Children = append(out.Children, expressionNode(element, ctx))
			continue
		}
		out.Children = append(out.Children, node{
			Kind: kindKeyed,
			Children: []node{
				expressionNode(keyed.Key, ctx),
				expressionNode(keyed.Value, ctx),
			},
		})
	}
	return out
}
