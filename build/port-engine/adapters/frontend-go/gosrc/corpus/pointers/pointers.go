// Package pointers exists to exercise OWNERSHIP DISPOSITION.
//
// Go is garbage-collected, so a `*T` says nothing about ownership: it may be a borrow that does
// not outlive the call, an owned value passed by pointer for efficiency, or a shared structure
// with live aliases. Rust needs that decision made, and the facts a front end can observe about
// it are what this package varies — one declaration per combination, so a disposition rule that
// collapses two of them shows up as a wrong answer rather than as an untested path.
package pointers

// Counter accumulates a total.
type Counter struct {
	// total is the running sum.
	total int
	// label names the counter.
	label string
}

// MUTATED, does not escape: the body assigns through the receiver and nothing outlives the call.

// NewCounter returns a counter starting at zero, named for display.
func NewCounter(label string) Counter {
	return Counter{label: label}
}

// Add adds n to the counter.
func (c *Counter) Add(n int) {
	c.total = c.total + n
}

// Neither mutated nor escaping: a read-only borrow is enough.

// Total reports the accumulated value.
func (c *Counter) Total() int {
	return c.total
}

// No pointer at all — the receiver is a copy, so no disposition question arises.

// Label reports the counter's name, by value receiver.
func (c Counter) Label() string {
	return c.label
}

// The receiver is mutated; `other` is read-only. Two pointers in one signature, with different
// answers — a rule that keys on the type rather than on the facts gives them the same one.

// Merge folds other into the receiver.
func (c *Counter) Merge(other *Counter) {
	c.total = c.total + other.total
}

// Tally is a counter allocated away from the caller's frame.
type Tally struct {
	// label names the tally.
	label string
}

// TWO things at once, because they only occur together. The address of a value the expression
// itself creates: no caller owns it and nothing else can alias it, so the owned form is the only
// one available — the one case where `&` needs no signature to decide it, and `&c` of an existing
// binding stays refused. And `func New(..) *T`, which is the commoner constructor shape of the
// two: what it constructs is the pointer's target, so the impl block stands on that.

// NewTally returns a tally named for display.
func NewTally(label string) *Tally {
	return &Tally{label: label}
}
