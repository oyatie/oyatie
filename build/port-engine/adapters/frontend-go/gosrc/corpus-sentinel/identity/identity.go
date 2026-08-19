// Package identity is a REFUSAL corpus: what a sentinel failure CANNOT do.
//
// A sentinel becomes its MESSAGE, which is everything a `return ErrGone` needs. What a message
// cannot carry is IDENTITY. The source's `errors.New` returns a pointer, so `err == ErrGone`
// compares identity and is a line real code writes; the target's failure is a boxed trait object
// with no equality at all, so nothing means what that line means — and a comparison against a
// freshly built value would be FALSE at every call.
//
// Its OWN corpus, because a refusal shares a package with nothing: the transform reports the first
// refusal it reaches, and a class proven in a shared corpus is one that stops being proven the day
// another refusal lands beside it.
package identity

import "errors"

// ErrGone is a sentinel, and this file is about what a sentinel CANNOT do.
var ErrGone = errors.New("gone")

// Missing reports whether the given failure is the sentinel one.
//
// REFUSED, and this is the cost the sentinel decision names. The source's `errors.New` returns a
// POINTER, so `err == ErrGone` compares identity and is a comparison real code writes. The target's
// sentinel is its message and its failure is a boxed trait object, which has no equality at all —
// so there is no target expression that means what this line means. It refuses here rather than
// emitting a comparison against a freshly built value, which would be false at every call.
func Missing(err error) bool {
	return err == ErrGone
}
