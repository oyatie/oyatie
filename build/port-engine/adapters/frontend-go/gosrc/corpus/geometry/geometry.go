// Package geometry provides rectangles built from points declared in another package.
package geometry

import "oyatie.example/portengine-fixture/corpus/shapes"

// Origin names the point type rectangles are built from.
type Origin = shapes.Point

// Bounds is a rectangle described by its two opposite corners.
type Bounds struct {
	// Min is the lower corner.
	Min shapes.Point
	// Max is the upper corner.
	Max shapes.Point
}

// Size reports how many labelled counts the table holds.
func Size(table map[string]int) int {
	return len(table)
}
