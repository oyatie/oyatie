package main

import (
	"go/types"
	"sort"
)

// Signatures and flags.

func signatureChildren(sig *types.Signature, qualify types.Qualifier) []node {
	children := make([]node, 0, sig.Params().Len()+sig.Results().Len())
	children = append(children, tupleNodes(kindParam, sig.Params(), qualify)...)
	children = append(children, tupleNodes(kindResult, sig.Results(), qualify)...)
	if len(children) == 0 {
		return nil
	}
	return children
}

// tupleNodes preserves tuple order, which IS semantic: parameters and results are
// positional in both Go and Rust.
func tupleNodes(kind string, tuple *types.Tuple, _ types.Qualifier) []node {
	if tuple == nil || tuple.Len() == 0 {
		return nil
	}
	nodes := make([]node, 0, tuple.Len())
	for i := 0; i < tuple.Len(); i++ {
		v := tuple.At(i)
		nodes = append(nodes, node{
			Kind: kind,
			Name: v.Name(),
			Type: typeTree(v.Type()),
		})
	}
	return nodes
}

func sortNodes(nodes []node) {
	sort.Slice(nodes, func(i, j int) bool { return nodes[i].Name < nodes[j].Name })
}

// flagsFor returns the set spelling of the boolean facts about a node. Sorted, so the set
// has exactly one encoding; nil when empty, so the JSON omits the key entirely.
func flagsFor(exported bool, variadic bool, embedded bool, pointerReceiver bool) []string {
	flags := make([]string, 0, 4)
	if embedded {
		flags = append(flags, flagEmbedded)
	}
	if exported {
		flags = append(flags, flagExported)
	}
	if pointerReceiver {
		flags = append(flags, flagPointerReceiver)
	}
	if variadic {
		flags = append(flags, flagVariadic)
	}
	if len(flags) == 0 {
		return nil
	}
	sort.Strings(flags)
	return flags
}
