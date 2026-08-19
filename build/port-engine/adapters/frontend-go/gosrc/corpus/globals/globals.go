// Package globals exists to prove the two shapes a package-scope variable takes.
//
// The synchronization argument that used to defer every `var` — `static` is immutable, `static mut`
// is unsafe, and `OnceLock`/`Mutex` each pick a policy the source never stated — is true, and it
// only bites for a variable the program assigns to. Across the surveyed corpora most package
// variables are never written anywhere in their own package, so the hardest case's reason was being
// applied to variables that do not have the problem.
//
// For the unwritten ones the form is a `static`, not a `const`: the source variable has ONE storage
// location for the life of the program and a `const` is materialised afresh at every use. A
// `static`'s initialiser must be a constant expression, which is the target's own rule rather than
// a stand-in for one, so the two shapes here are a literal and an absent initialiser.
//
// The WRITTEN case lives in the refusal corpus, and not by preference. A function that touches a
// variable the engine cannot emit cannot be emitted either — what the engine emits has to be
// self-contained — so the writer and the variables it writes belong where refusals are proven.
package globals

// Prefix is initialised from a literal and never written. A string literal is a BORROW in the
// target, and the owned form cannot be built by a constant expression at all.
var Prefix = "id-"

// Limit is declared with no initialiser and never written, so it is only ever read as its zero
// value — and a zero is a value rather than work, so nothing about it happens at a time.
var Limit int64

// Describe touches no package variable, which is what lets it be emitted beside two that are.

// Describe returns the name of this package.
func Describe() string {
	return "globals"
}
