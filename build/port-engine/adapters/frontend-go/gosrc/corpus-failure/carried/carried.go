// Package carried is a REFUSAL corpus: a failing return that carries a value beside the failure.
//
// The source returns the failure ALONGSIDE the value, and the target returns one or the other. That
// is fine for the overwhelming majority of real code, because the convention is that a failing
// return's other operands are the zero value — a caller that reads them after a non-nil failure is
// reading something the source promised nothing about, so discarding them loses nothing.
//
// `Sized` breaks the convention: it computes a real length and returns it together with a failure.
// Discarding it would silently lose work, and the reader of the emitted crate would have no way to
// see that anything had been lost. So it refuses, and the refusal says which value it would have
// had to drop.
package carried

import "errors"

// Sized reports the length of s and whether s was usable, returning both even on the failure path.
func Sized(s string) (int, error) {
	if s == "" {
		return len(s), errors.New("empty")
	}
	return len(s), nil
}
