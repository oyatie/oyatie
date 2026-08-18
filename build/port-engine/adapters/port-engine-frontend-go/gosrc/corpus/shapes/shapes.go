// Package shapes is a hermetic Go fixture covering structural declarations:
// structs with exported and unexported fields, methods bound to a named type,
// and an interface with a method set.
package shapes

// Point is a location in two dimensions.
type Point struct {
	// X is the horizontal coordinate.
	X int
	// Y is the vertical coordinate.
	Y int
	// label is unexported and must survive extraction as such.
	label string
}

// Shift returns a Point moved by the given deltas.
func (p Point) Shift(dx int, dy int) Point {
	return Point{X: p.X + dx, Y: p.Y + dy}
}

// Area reports the rectangle area between the origin and p.
func (p Point) Area() int {
	return p.X * p.Y
}

// Named is anything that can render its own name.
type Named interface {
	// Name returns the display name.
	Name() string
	// Rename replaces the display name.
	Rename(next string)
}

// Origin is the zero point.
var Origin Point = Point{X: 0, Y: 0}
