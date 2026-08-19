// Package invariant exists to prove the ONE panic shape that ports without loss.
//
// Go's `panic(v)` aborts carrying `v`; Rust's `panic!` aborts carrying a formatted string. Where
// `v` is a string literal the two are the same abort with the same message and the same payload
// type, so nothing is lost. Where `v` is an error or an arbitrary value the payload TYPE is lost,
// and a caller that recovers and type-asserts on it would see a different program — so that shape
// refuses by name and lives in the refusal corpus.
//
// The census sizes the tractable half: the string-literal invariant shapes are 38.2% and 21.0% of
// Kubernetes panic sites, 59% together.
package invariant

// Half returns n halved, and aborts on an odd input.
//
// The invariant shape: a condition the caller is required to have met, and a literal message
// naming what it was.
func Half(n int64) int64 {
	if n%2 != 0 {
		panic("Half requires an even argument")
	}
	return n / 2
}
