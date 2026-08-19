// Package shapes provides points in two dimensions and the naming interface they satisfy.
package shapes

// Point is a location in two dimensions.
type Point struct {
	// X is the horizontal coordinate.
	X int
	// Y is the vertical coordinate.
	Y int
	// label names the point for display.
	label string
}

// NewPoint returns the point at the given coordinates, labelled for display.
func NewPoint(x int, y int, label string) Point {
	return Point{X: x, Y: y, label: label}
}

// Label returns the point's display name.
func (p Point) Label() string {
	return p.label
}

// Shift returns the point moved by the given deltas, keeping its label.
func (p Point) Shift(dx int, dy int) Point {
	return Point{X: p.X + dx, Y: p.Y + dy, label: p.label}
}

// Area reports the area of the rectangle between the origin and this point.
func (p Point) Area() int {
	return p.X * p.Y
}

// Named is anything that can render its own display name.
type Named interface {
	// Name returns the display name.
	Name() string
	// Refresh recomputes the display name from the current state.
	Refresh()
}

// origin is the point at the coordinate origin.
var origin Point = Point{X: 0, Y: 0}
