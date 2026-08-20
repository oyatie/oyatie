package main

import (
	"go/ast"
	"go/types"
)

// Types whose meaning is the SOURCE RUNTIME's memory layout, and which therefore cannot be ported.
//
// `unsafe.Pointer` is the source's escape from its own type system. A type that exists only to be
// reinterpreted through it is not describing a value — it is describing how the source runtime lays
// one out in memory, and the target does not share that layout. `xxhash`'s `sliceHeader` is the
// case: two fields mirroring a slice's representation, so a string can be reinterpreted as a byte
// slice without copying.
//
// Ported naively it comes out as an ordinary struct in a crate that denies `unsafe`, whose fields
// could never mean what they meant, and which a reader has no way to identify as residue. A blind
// reviewer of this engine's output named it the single most decisive piece of evidence that the code
// was mechanically translated, and reached the same conclusion from the other direction: a Rust
// author would never invent it, because `as_bytes()` already does the job for free.
//
// Refusing it BY NAME is right where dropping it for being unreferenced is not — see the R2f entry
// in build/REORG-DRAIN.md, which records that mistake and why reachability was the wrong instrument.
//
// EVERY reference must be inside the escape hatch. A type used both ways is a real type that also
// happens to be reinterpreted somewhere, and refusing it would be refusing the author's work on the
// strength of one use.
func indexUnsafeOnlyTypes(files []*ast.File, info *types.Info, tpkg *types.Package) map[types.Object]bool {
	total := map[types.Object]int{}
	underUnsafe := map[types.Object]int{}
	for _, file := range files {
		ast.Inspect(file, func(n ast.Node) bool {
			expr, ok := n.(ast.Expr)
			if !ok || !mentionsUnsafe(expr, info) {
				return true
			}
			// The whole subtree is inside the hatch. Counted here and NOT descended into, so a
			// nested expression that also mentions `unsafe` cannot count its operands twice.
			for object, count := range typeUses(expr, info, tpkg) {
				underUnsafe[object] += count
			}
			return false
		})
		for object, count := range typeUses(file, info, tpkg) {
			total[object] += count
		}
	}
	only := map[types.Object]bool{}
	for object, count := range total {
		if count > 0 && underUnsafe[object] == count {
			only[object] = true
		}
	}
	return only
}

// mentionsUnsafe reports whether this expression names the `unsafe` package anywhere within it.
//
// The PACKAGE, resolved by the type-checker, never the spelling: a local variable named `unsafe` is
// legal and is not the escape hatch.
func mentionsUnsafe(expr ast.Expr, info *types.Info) bool {
	found := false
	ast.Inspect(expr, func(n ast.Node) bool {
		ident, ok := n.(*ast.Ident)
		if !ok {
			return true
		}
		name, ok := info.Uses[ident].(*types.PkgName)
		if ok && name.Imported().Path() == "unsafe" {
			found = true
		}
		return !found
	})
	return found
}

// typeUses counts references to this package's own named types within a node.
//
// USES only. A type's own declaration is a definition rather than a reference, and counting it would
// mean no type could ever have all of its references inside the hatch.
func typeUses(root ast.Node, info *types.Info, tpkg *types.Package) map[types.Object]int {
	counts := map[types.Object]int{}
	ast.Inspect(root, func(n ast.Node) bool {
		ident, ok := n.(*ast.Ident)
		if !ok {
			return true
		}
		name, ok := info.Uses[ident].(*types.TypeName)
		if ok && name.Pkg() == tpkg {
			counts[name]++
		}
		return true
	})
	return counts
}
