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

// enabled reports whether the fixture feature is on by default.
var enabled bool = true

// threshold is the default cutoff.
var threshold float64 = 0.75

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

// Lower-case, so the source does not export it. Here because the emitted crate must keep the
// distinction: a private helper that becomes public is a wider API than the source declared.

// normalize returns n in its canonical form, which for a plain count is n itself.
func normalize(n int) int {
	return n
}
