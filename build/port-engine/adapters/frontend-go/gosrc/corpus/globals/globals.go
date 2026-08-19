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

// A string literal is a borrow in Rust and the owned form is not a constant expression, so this is
// the case that decides what a never-written string variable becomes.

// prefix is the identity prefix every generated name starts with.
var prefix = "id-"

// No initialiser and never written, so its value is the zero — which is a value rather than work,
// and therefore has no when-does-it-happen question of the kind that defers package init.

// limit is the ceiling this component will not exceed. Zero means unbounded.
var limit int64

// Describe touches no package variable, which is what lets it be emitted beside two that are.

// Describe returns the name of this component.
func Describe() string {
	return "globals"
}
