// Package naming exists to prove IMPLS FROM USAGE.
//
// Go's interfaces are implicit: nothing in Label's declaration says it satisfies shapes.Named, and
// nothing in shapes.Named says which types satisfy it. The relation exists only where a concrete
// value flows into an interface-typed position, and docs/programs/k8s-port/census/interfaces.md
// measured what happens if the engine guesses instead — 80,042 structural matches against 1,316
// pairs the source declares outright.
//
// Two implementors, because the trait's receiver mode is the UNION over them: one mutating method
// anywhere makes that method exclusive for everyone, and one implementor could not show that the
// union is a union rather than the first answer found.
//
// The sites that do NOT emit live in corpus-interface/ instead — a value flowing into a trait
// position needs a coercion, and returning one needs an owner, and neither is a rule this package
// should smuggle in by having the shape and no rule for it.
package naming

import "oyatie.example/portengine-fixture/corpus/shapes"

// Label is a display name that can rename itself.
type Label struct {
	// text is the current display name.
	text string
}

// Name returns the display name.
//
// Read-only, which is what makes the derived trait receiver interesting: a rule that assumed every
// trait method is exclusive would put `&mut self` on a getter.
func (l *Label) Name() string {
	return l.text
}

// Rename replaces the display name.
//
// Mutating, and one mutating implementor is enough to make the whole trait method exclusive: a
// trait fixes one signature for everyone.
func (l *Label) Rename(next string) {
	l.text = next
}

// Assertion site: the Go compiler checks this line, so the pair is PROVEN rather than inferred.
var _ shapes.Named = (*Label)(nil)

// Tag is a display name that never changes.
type Tag struct {
	// text is the fixed display name.
	text string
}

// Name returns the display name.
func (t *Tag) Name() string {
	return t.text
}

// Rename does nothing: a tag's name is fixed.
//
// A no-op rather than a mutation, so the derived receiver has to come from the UNION over
// implementors — a second implementor that does not mutate proves the union does not simply take
// the last answer.
func (t *Tag) Rename(next string) {
}

// Assertion site for the second implementor.
var _ shapes.Named = (*Tag)(nil)

// Describe renders anything that can name itself.
//
// The parameter is the one interface position with an unambiguous target form: the caller keeps
// the value and the callee only reads it, so a borrow is right and no ownership question arises.
func Describe(named shapes.Named) string {
	return named.Name()
}
