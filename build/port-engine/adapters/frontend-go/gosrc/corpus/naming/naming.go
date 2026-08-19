// Package naming provides display names that satisfy the shapes.Named interface.
//
// Nothing in Label's declaration says it satisfies shapes.Named, and nothing in shapes.Named says
// which types satisfy it: the relation exists only where a concrete value flows into an
// interface-typed position. Two implementors, because a trait method's receiver mode is the union
// over them — one mutating implementor makes the method exclusive for everyone, and one
// implementor alone could not show that the union is a union rather than the first answer found.
package naming

import "oyatie.example/portengine-fixture/corpus/shapes"

// Label is a display name derived from a prefix.
type Label struct {
	// prefix is what the display name is derived from.
	prefix string
	// text is the current display name.
	text string
}

// NewLabel returns a label that derives its display name from prefix.
//
// The display name starts EMPTY and is derived on the first refresh, so the two fields can differ —
// which is what makes Refresh a state change rather than a self-assignment.
func NewLabel(prefix string) Label {
	return Label{prefix: prefix}
}

// Read-only, which is what makes the derived trait receiver interesting: a rule that assumed every
// trait method is exclusive would put `&mut self` on a getter.

// Name returns the display name.
func (l *Label) Name() string {
	return l.text
}

// Mutating, and one mutating implementor is enough to make the whole trait method exclusive: a
// trait fixes one signature for everyone.

// Refresh derives the display name from the prefix.
func (l *Label) Refresh() {
	l.text = l.prefix
}

// Assertion site: the Go compiler checks this line, so the pair is PROVEN rather than inferred.
var _ shapes.Named = (*Label)(nil)

// Tag is a display name that never changes.
type Tag struct {
	// text is the fixed display name.
	text string
}

// NewTag returns a tag with the given fixed display name.
func NewTag(text string) Tag {
	return Tag{text: text}
}

// Name returns the display name.
func (t *Tag) Name() string {
	return t.text
}

// Nothing to recompute, which is what makes this both honest and useful here: the derived receiver
// comes from the UNION over implementors, and an implementor that does not mutate proves the union
// does not simply take the last answer found.

// Refresh recomputes the display name, which for a fixed tag is already current.
func (t *Tag) Refresh() {
}

// Assertion site for the second implementor.
var _ shapes.Named = (*Tag)(nil)

// The parameter is the one interface position with an unambiguous target form: the caller keeps
// the value and the callee only reads it, so a borrow is right and no ownership question arises.

// Describe returns the display name of anything that can name itself.
func Describe(named shapes.Named) string {
	return named.Name()
}
