package main

import (
	"go/types"
	"sort"
)

// What an observed pair BECOMES: one `implements` node, after the duplicates collapse.
//
// Kept apart from the walk that finds the pairs, because the two answer different questions. The
// walk is about the source language's syntax — where a concrete value can turn into an interface.
// This is about the snapshot — what a pair has to carry for the transform to emit an impl from it
// without reaching across units for anything.

// implementsNodes builds the `implements` children for one concrete declaration.
//
// The interface's FULL method set is carried on the node, embedded methods included, so the fact
// is self-contained: the impl is emitted in the unit that declares the concrete type, which is not
// in general the unit that declares the interface, and a transform that had to reach across units
// for the method set would be resolving a reference the snapshot does not model.
func implementsNodes(facts []satisfaction, qualify types.Qualifier) []node {
	nodes := make([]node, 0, len(facts))
	for _, fact := range facts {
		iface, ok := fact.iface.Underlying().(*types.Interface)
		if !ok {
			continue
		}
		// The trait's OWN methods, not its full set. An embedded interface becomes a SUPERTRAIT
		// in the target and carries its own impl — recorded separately, because satisfying an
		// interface satisfies everything it embeds — so repeating those methods here would name
		// members the trait does not have.
		methods := make([]node, 0, iface.NumExplicitMethods())
		for i := 0; i < iface.NumExplicitMethods(); i++ {
			method := iface.ExplicitMethod(i)
			sig, ok := method.Type().(*types.Signature)
			if !ok {
				continue
			}
			methods = append(methods, node{
				Kind:     kindMethod,
				Name:     method.Name(),
				Flags:    flagsFor(method.Exported(), sig.Variadic(), false, false),
				Children: signatureChildren(sig, qualify),
			})
		}
		sortNodes(methods)
		// A pure BUNDLE: the interface declares no method of its own and embeds at least one. The
		// source satisfies such an interface STRUCTURALLY, so every type with the embedded method
		// sets has it — which the target says once with a blanket impl rather than once per type.
		// Recorded as a fact because only the type-checker can see it: the interface is routinely
		// declared in a package this observation is not in.
		attrs := map[string]string{attrSite: fact.site}
		if iface.NumExplicitMethods() == 0 && iface.NumEmbeddeds() > 0 {
			attrs[attrBundle] = "true"
		}
		nodes = append(nodes, node{
			Kind: kindImplements,
			// Deliberately unnamed: the front end refuses two same-named declarations in one
			// NAMESPACE, and a type satisfying two interfaces would trip that check on a node
			// whose identity is its type rather than its name.
			Type:     typeTree(fact.iface),
			Attrs:    attrs,
			Children: methods,
		})
	}
	return nodes
}

// dedupeSatisfactions collapses repeated observations of one pair, keeping the site that proves
// the most, and orders the result so the snapshot is stable.
func dedupeSatisfactions(facts []satisfaction) []satisfaction {
	// STRUCTURAL ranks last, so a pair the source also USES keeps the site that proves the most.
	// The impl is identical either way; what differs is only how it came to be known.
	rank := map[string]int{
		siteAssertion: 0, siteAssign: 1, siteArgument: 2, siteResult: 3, siteStructural: 4,
	}
	best := map[[2]string]satisfaction{}
	for _, fact := range facts {
		key := [2]string{typeKey(fact.concrete), typeKey(fact.iface)}
		if seen, ok := best[key]; ok && rank[seen.site] <= rank[fact.site] {
			continue
		}
		best[key] = fact
	}

	out := make([]satisfaction, 0, len(best))
	for _, fact := range best {
		out = append(out, fact)
	}
	sort.Slice(out, func(i, j int) bool {
		left, right := typeKey(out[i].concrete), typeKey(out[j].concrete)
		if left != right {
			return left < right
		}
		return typeKey(out[i].iface) < typeKey(out[j].iface)
	})
	return out
}

// typeKey is a named type's package-qualified identity.
func typeKey(named *types.Named) string {
	obj := named.Obj()
	if obj.Pkg() == nil {
		return obj.Name()
	}
	return obj.Pkg().Path() + "." + obj.Name()
}
