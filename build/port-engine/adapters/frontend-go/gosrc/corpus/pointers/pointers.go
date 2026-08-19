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

// Add adds n to the counter.
//
// MUTATED, does not escape: the body assigns through the receiver and nothing outlives the call.
func (c *Counter) Add(n int) {
	c.total = c.total + n
}

// Total reports the accumulated value.
//
// Neither mutated nor escaping: a read-only borrow is enough.
func (c *Counter) Total() int {
	return c.total
}

// Label reports the counter's name, by value receiver.
//
// No pointer at all — the receiver is a copy, so no disposition question arises.
func (c Counter) Label() string {
	return c.label
}

// Merge folds other into the receiver.
//
// The receiver is mutated; `other` is read-only. Two pointers in one signature, with different
// answers — a rule that keys on the type rather than on the facts gives them the same one.
func (c *Counter) Merge(other *Counter) {
	c.total = c.total + other.total
}
