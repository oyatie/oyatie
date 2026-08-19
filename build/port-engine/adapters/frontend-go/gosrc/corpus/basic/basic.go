// Package basic is a hermetic Go fixture for the port engine's bootstrap extractor.
//
// It exercises the declaration kinds the engine translates first: typed constants,
// package-level variables, a type alias, a named (defined) type, and functions with
// named parameters and results. Every type here is deliberately spellable in the
// neutral rule pack's type map — hard cases live in their own fixture package so a
// refusal is never mistaken for a gap in this one.
package basic

// MaxRetries bounds the retry loop.
const MaxRetries int = 3

// DefaultName is the fallback identity for an unnamed record.
const DefaultName string = "anonymous"

// Enabled reports whether the fixture feature is on by default.
var Enabled bool = true

// Threshold is the default cutoff.
var Threshold float64 = 0.75

// ID is an alias for the underlying identity spelling.
type ID = string

// Celsius is a temperature in degrees Celsius.
type Celsius float64

// Add returns the sum of a and b.
func Add(a int, b int) int {
	return a + b
}

// Scale multiplies value by factor.
func Scale(value float64, factor float64) float64 {
	return value * factor
}

// Label renders a display name for the given identity.
func Label(id ID, fallback string) string {
	if id == "" {
		return fallback
	}
	return id
}

// unexported returns n unchanged, and is not part of this package's public surface.
func unexported(n int) int {
	return n
}
