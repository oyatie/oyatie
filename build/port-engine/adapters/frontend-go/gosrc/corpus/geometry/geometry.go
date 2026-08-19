// Package geometry provides rectangles built from points declared in another package.
package geometry

import "oyatie.example/portengine-fixture/corpus/shapes"

// Origin names the point type this package builds rectangles from.
type Origin = shapes.Point

// Bounds is a rectangle described by its two opposite corners.
type Bounds struct {
	// Min is the lower corner.
	Min shapes.Point
	// Max is the upper corner.
	Max shapes.Point
}

// Widths returns the horizontal extents it was given, unchanged.
func Widths(counts []int) []int {
	return counts
}

// Size reports how many labelled counts the table holds.
func Size(table map[string]int) int {
	return len(table)
}
