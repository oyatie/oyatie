// Package globals exists to prove what is OBSERVED about a package-scope variable.
//
// The pack defers `var` because Rust's `static` is immutable, `static mut` is unsafe, and
// `OnceLock`/`Mutex` each pick a synchronization policy the source never stated. That argument is
// true — and it only bites for a variable the program actually assigns to. Across the surveyed
// corpora 45 of 67 package variables are never written anywhere in their package, so the hardest
// case's reason was being applied to two thirds of variables that do not have the problem.
//
// Nothing here decides the emitted form. It records the fact that makes the decision possible:
// what the variable is initialised to, and whether anything writes it.
//
// The WRITTEN case lives in the refusal corpus, and not by preference. A function that touches a
// deferred variable cannot be emitted — what the engine emits has to be self-contained — so the
// writer and the variables it writes belong where refusals are proven.
package globals

// Prefix is initialised and never written. A constant with a computed value.
var Prefix = "id-"

// Limit is declared with no initialiser and never written, so it is only ever read as its zero
// value.
var Limit int64

// Describe names the package. It touches no package variable, which is what lets it be emitted
// beside two that are deferred.
func Describe() string {
	return "globals"
}
