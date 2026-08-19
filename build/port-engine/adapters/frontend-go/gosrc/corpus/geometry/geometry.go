// Package geometry exists to prove CROSS-PACKAGE type resolution.
//
// Everything else in this corpus refers only to its own package's types, which a resolver keyed by
// bare name would handle by accident. Here `shapes.Point` is a named type from another package,
// and resolving it needs the package identity the snapshot now carries and the module path the
// assembler emits — neither of which a flat spelling table could express.
package geometry

import "oyatie.example/portengine-fixture/corpus/shapes"

// Origin is the zero point, referred to across a package boundary.
type Origin = shapes.Point

// Bounds is a rectangle described by two points from another package.
type Bounds struct {
	// Min is the lower corner.
	Min shapes.Point
	// Max is the upper corner.
	Max shapes.Point
}

// Widths returns the horizontal extents of several rectangles.
//
// The slice is the point: a composite type resolves by CONSTRUCTOR now, so one `slice` entry in
// the pack answers this and every other slice, where a spelling table needed a row per element.
func Widths(counts []int) []int {
	return counts
}

// Lookup returns a count for a label, falling back when the table has none.
//
// The map is in the SIGNATURE rather than the body on purpose: indexing is not in the translated
// statement subset yet and refuses by name, and this package's job is to prove type resolution
// rather than to smuggle an untranslated construct into the green corpus.
func Lookup(table map[string]int, fallback int) int {
	return fallback
}
