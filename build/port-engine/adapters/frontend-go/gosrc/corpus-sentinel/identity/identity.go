// Package identity is about what a sentinel failure IS, and what a caller may ask of it.
//
// A sentinel becomes a TYPE: a unit struct that displays the source's message and implements the
// target's error trait. That is what makes `err == ErrGone` translatable — the source compares
// identity because its sentinel is a pointer, and the target asks the trait object what concrete
// type it holds, which is true in exactly the same cases.
//
// It did not start there. While a sentinel was its MESSAGE, a fresh failure built from a shared
// string was equal to nothing and this comparison refused by name, with the loss recorded as the
// cost of that decision. The cost was paid; this file is where it shows.
package identity

import "errors"

// ErrGone is a sentinel, and this file is about what a sentinel CANNOT do.
var ErrGone = errors.New("gone")

// Missing reports whether the given failure is the sentinel one.
//
// The comparison the whole decision turns on. In the source it is pointer equality against a
// package-level value; in the target it is a question asked of the trait object, and a failure that
// is not that sentinel answers no in both.
func Missing(err error) bool {
	return err == ErrGone
}
