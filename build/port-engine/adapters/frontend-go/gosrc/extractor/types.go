package main

import (
	"fmt"
	"go/types"
	"strconv"
	"strings"
)

// Types, as TREES.
//
// A flat spelling worked exactly as long as every type was primitive or had its own table row.
// A tree resolves by CONSTRUCTOR, so one rule for `slice` answers every slice, and it carries the
// PACKAGE that declares a named type, so two packages declaring `Point` do not collide.

func typeTree(t types.Type) *typeNode {
	switch typed := t.(type) {
	case *types.Basic:
		return &typeNode{Kind: typeBasic, Name: typed.Name()}

	case *types.Alias:
		return namedNode(typed.Obj())

	case *types.Named:
		out := namedNode(typed.Obj())
		if _, ok := typed.Underlying().(*types.Interface); ok {
			out.Kind = typeNamedInterface
		}
		for i := 0; i < typed.TypeArgs().Len(); i++ {
			out.Args = append(out.Args, typeTree(typed.TypeArgs().At(i)))
		}
		return out

	case *types.Pointer:
		return &typeNode{Kind: typePointer, Args: []*typeNode{typeTree(typed.Elem())}}

	case *types.Slice:
		return &typeNode{Kind: typeSlice, Args: []*typeNode{typeTree(typed.Elem())}}

	case *types.Array:
		// The length is part of the type. It is carried as a name rather than an argument because
		// it is not a type, and putting a non-type in the argument list would make the arity of
		// every other kind ambiguous.
		return &typeNode{
			Kind: typeArray,
			Name: strconv.FormatInt(typed.Len(), 10),
			Args: []*typeNode{typeTree(typed.Elem())},
		}

	case *types.Map:
		return &typeNode{
			Kind: typeMap,
			Args: []*typeNode{typeTree(typed.Key()), typeTree(typed.Elem())},
		}

	case *types.Chan:
		return &typeNode{
			Kind: typeChan,
			Name: chanDirection(typed.Dir()),
			Args: []*typeNode{typeTree(typed.Elem())},
		}

	case *types.Signature:
		out := &typeNode{Kind: typeFunc}
		out.Args = append(out.Args, tupleTypeNode(typed.Params()), tupleTypeNode(typed.Results()))
		return out

	case *types.Interface:
		return &typeNode{Kind: typeInterface}

	case *types.Struct:
		return &typeNode{Kind: typeStruct}

	case *types.Tuple:
		return tupleTypeNode(typed)

	case *types.TypeParam:
		return &typeNode{Kind: typeParam, Name: typed.Obj().Name()}

	default:
		return &typeNode{Kind: typeUnsupported, Name: strings.TrimPrefix(fmt.Sprintf("%T", t), "*types.")}
	}
}

func namedNode(obj *types.TypeName) *typeNode {
	out := &typeNode{Kind: typeNamed, Name: obj.Name()}
	if pkg := obj.Pkg(); pkg != nil {
		out.Package = pkg.Path()
	}
	return out
}

func tupleTypeNode(tuple *types.Tuple) *typeNode {
	out := &typeNode{Kind: typeTuple}
	if tuple == nil {
		return out
	}
	for i := 0; i < tuple.Len(); i++ {
		out.Args = append(out.Args, typeTree(tuple.At(i).Type()))
	}
	return out
}

func chanDirection(dir types.ChanDir) string {
	switch dir {
	case types.SendOnly:
		return "send"
	case types.RecvOnly:
		return "recv"
	default:
		return "both"
	}
}

// annotateParameterFacts records the ownership facts for each parameter that names something.
//
// Applied to every parameter and not only pointer-typed ones: whether a disposition is meaningful
// for a given type is the ENGINE's question, and deciding it here would put the target language's
// borrow model in the front end.
