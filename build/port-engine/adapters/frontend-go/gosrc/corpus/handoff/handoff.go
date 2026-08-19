// Package handoff exists to prove that an ARGUMENT is translated for the destination it reaches.
//
// `f(&x)` is the largest single group of address-of sites in the surveyed corpora — 11 of 33 — and
// it was refused because the body translator did not know what the parameter wanted. The parameter
// is a signature the engine has already translated, so the answer was always available; it just
// had nowhere to be asked from.
//
// The construction the argument takes is the SAME decision the parameter took, seen from the other
// end: a parameter that borrows is fed by a borrow, and one that owns is fed by an owned value.
// Both come from one disposition rule carrying one reason.
package handoff

// Counter counts.
type Counter struct {
	total int64
}

// Bump adds one through the pointer, so the parameter is an exclusive borrow and the argument
// lends exclusively.
func Bump(c *Counter) {
	c.total = c.total + 1
}

// Read returns the count without writing, so the parameter is a shared borrow.
func Read(c *Counter) int64 {
	return c.total
}

// Twice bumps the counter twice, handing the same local to a borrowing parameter each time.
func Twice() int64 {
	c := Counter{total: 0}
	Bump(&c)
	Bump(&c)
	return Read(&c)
}
