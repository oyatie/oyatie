package main

import (
	"fmt"
	"go/types"
)

// Attributing observed satisfactions to the declarations that will carry the impls.
//
// A flow is observed in the unit whose source shows it, and that is not in general the unit that
// declares the concrete type — the interfaces census's own example is `NewCodec` returning an
// unexported `codec` as a `Codec`, a satisfaction visible only where the constructor is written.
// So collection is per package and attribution is corpus-wide, once every package is in hand.
//
// The same corpus-wide view answers a question that was previously a pack GUESS: how a trait
// method binds its receiver. A Go interface says nothing about it, so P1 made it a declared pack
// decision with a recorded reason. With the implementors observed, it can be derived instead — a
// method is exclusive exactly when some observed implementor mutates through it — and the pack's
// decision falls back to covering the interfaces nothing was seen to implement.

// attributeSatisfactions hangs each observed pair on the declaration of the type that satisfies,
// and stamps each interface method with the receiver its implementors need.
func attributeSatisfactions(
	model *snapshot,
	facts []satisfaction,
	qualifiers map[string]types.Qualifier,
) {
	byUnit := map[string]*pkgNode{}
	for index := range model.Packages {
		byUnit[model.Packages[index].UnitID] = &model.Packages[index]
	}

	modes := receiverModes(facts, byUnit)
	stampInterfaces(facts, byUnit, modes)

	for _, fact := range facts {
		owner := packagePath(fact.concrete)
		unit, present := byUnit[owner]
		var target *node
		if present {
			target = declarationNamed(unit, fact.concrete.Obj().Name())
		}
		if target == nil {
			recordUnattributable(byUnit[fact.observedIn], fact)
			continue
		}
		for _, impl := range implementsNodes([]satisfaction{fact}, qualifiers[owner]) {
			stampReceivers(impl.Children, typeKey(fact.iface), modes)
			target.Children = append(target.Children, impl)
		}
	}

	for index := range model.Packages {
		sortNodes(model.Packages[index].Declarations)
	}
}

// receiverModes derives, for every observed (interface, method) pair, whether the receiver must be
// exclusive.
//
// The union over implementors, and it only ever escalates: one implementor that mutates makes the
// method exclusive for all of them, because a trait fixes one signature. `shared` is therefore a
// claim that NO observed implementor mutates — and since the observed set is a lower bound, a
// later-discovered implementor that does mutate is a compile error in the emitted crate rather
// than a silent aliasing change.
func receiverModes(facts []satisfaction, byUnit map[string]*pkgNode) map[[2]string]string {
	modes := map[[2]string]string{}
	for _, fact := range facts {
		unit, ok := byUnit[packagePath(fact.concrete)]
		if !ok {
			continue
		}
		concrete := declarationNamed(unit, fact.concrete.Obj().Name())
		if concrete == nil {
			continue
		}
		ifaceKey := typeKey(fact.iface)
		for _, method := range concrete.Children {
			if method.Kind != kindMethod {
				continue
			}
			key := [2]string{ifaceKey, method.Name}
			if modes[key] == receiverExclusive {
				continue
			}
			if hasFlag(method.Flags, flagMutated) {
				modes[key] = receiverExclusive
				continue
			}
			modes[key] = receiverShared
		}
	}
	return modes
}

// stampInterfaces writes the derived receiver onto the interface declarations themselves, so the
// trait and every impl of it agree by construction rather than by two rules producing the same
// answer.
func stampInterfaces(facts []satisfaction, byUnit map[string]*pkgNode, modes map[[2]string]string) {
	for _, fact := range facts {
		unit, ok := byUnit[packagePath(fact.iface)]
		if !ok {
			continue
		}
		iface := declarationNamed(unit, fact.iface.Obj().Name())
		if iface == nil {
			continue
		}
		stampReceivers(iface.Children, typeKey(fact.iface), modes)
	}
}

// stampReceivers annotates a method list with the derived receiver for its interface.
func stampReceivers(methods []node, ifaceKey string, modes map[[2]string]string) {
	for index := range methods {
		if methods[index].Kind != kindMethod {
			continue
		}
		mode, ok := modes[[2]string{ifaceKey, methods[index].Name}]
		if !ok {
			continue
		}
		methods[index].Attrs = withAttr(methods[index].Attrs, attrReceiver, mode)
	}
}

// recordUnattributable notes a satisfaction the engine has nowhere to emit.
//
// Recorded rather than dropped: dropping it would make a satisfaction the engine cannot emit
// indistinguishable from one that does not exist, and the difference is exactly what a reader of
// the emitted crate would need to know.
func recordUnattributable(observer *pkgNode, fact satisfaction) {
	if observer == nil {
		return
	}
	observer.Declarations = append(observer.Declarations, node{
		Kind: kindForeignSatisfaction,
		Name: typeKey(fact.concrete),
		Attrs: map[string]string{
			attrGoNode: fmt.Sprintf(
				"satisfaction of %s by %s, which this corpus does not declare",
				typeKey(fact.iface), typeKey(fact.concrete),
			),
			attrSite: fact.site,
		},
	})
}

// declarationNamed finds the declaration a satisfaction attaches to.
func declarationNamed(unit *pkgNode, name string) *node {
	for index := range unit.Declarations {
		if unit.Declarations[index].Name == name {
			return &unit.Declarations[index]
		}
	}
	return nil
}

// packagePath is the import path of the package declaring a named type.
func packagePath(named *types.Named) string {
	if pkg := named.Obj().Pkg(); pkg != nil {
		return pkg.Path()
	}
	return ""
}

func hasFlag(flags []string, want string) bool {
	for _, flag := range flags {
		if flag == want {
			return true
		}
	}
	return false
}

// qualifierFor renders types relative to one package: local names stay bare, and anything from
// elsewhere keeps its full path.
func qualifierFor(tpkg *types.Package) types.Qualifier {
	return func(other *types.Package) string {
		if other == tpkg {
			return ""
		}
		return other.Path()
	}
}
