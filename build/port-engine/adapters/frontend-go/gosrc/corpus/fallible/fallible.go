// Package fallible exists to prove the FAILURE CONVENTION, which is the thing that blocks every
// real package rather than any single construct.
//
// Go returns failure as an extra result. Nothing in a signature says the value must be checked and
// nothing in the type system stops a caller from dropping it — it is a convention held up by
// discipline. The target says the same thing in the return type, where the compiler holds it up
// instead, so this translation is one of the few that makes the ported program stricter than the
// original rather than merely equivalent to it.
//
// Every declaration here is one shape of the convention, so a rule that handles the easy one and
// guesses at the rest shows up as a wrong answer rather than as an untested path.
package fallible

import "errors"

// The SENTINEL: one failure value declared once and returned from many places, which is the
// commonest error-typed package variable in real code. It becomes its MESSAGE, and each return
// builds a failure from that through the same mapping the pack declares for the constructor.
//
// What is lost is IDENTITY. The source's `errors.New` returns a pointer and a caller may write
// `err == ErrEmpty` to compare against it; the target's boxed trait object has no equality, so that
// comparison has no translation and refuses where it is written rather than comparing something
// else. Returning a sentinel ports; comparing against one does not.

// ErrEmpty is returned when the input string has no content.
var ErrEmpty = errors.New("empty")

// The one-value-and-a-failure shape: `(int, error)`, which is what most of a real package looks
// like.

// Length reports the length of s, failing on the empty string.
func Length(s string) (int, error) {
	if s == "" {
		return 0, ErrEmpty
	}
	return len(s), nil
}

// The no-value shape: `error` alone. The target has to invent the success value here, because a
// function that returns only a failure still has to say it succeeded.

// Check reports whether s is usable, returning only a failure.
func Check(s string) error {
	if s == "" {
		return errors.New("empty")
	}
	return nil
}

// The PROPAGATION shape, and the point of the whole exercise: `n, err := Length(s)` followed by
// `if err != nil { return 0, err }` is two statements the caller could simply not have written.
// The target spells it as one operator on a value that cannot be used without addressing the
// failure.

// Twice returns the length of s doubled.
func Twice(s string) (int, error) {
	n, err := Length(s)
	if err != nil {
		return 0, err
	}
	return n + n, nil
}

// Propagation with NO value bound: the call is run for its failure alone, so there is nothing to
// name and the target keeps it as a statement.

// Validate checks s and reports nothing else.
func Validate(s string) error {
	err := Check(s)
	if err != nil {
		return err
	}
	return nil
}

// A failure STORED rather than returned. The source's error is an interface value and can sit
// anywhere a value can, including inside another type — which is the shape a trait cannot take in
// the target, because a trait has no size. The failure convention already answers it: it chose an
// owned boxed form because a failure outlives the call that produced it, and a field holding one
// has exactly the same problem for exactly the same reason.

// Report records a failure alongside what was being attempted.
type Report struct {
	// action names what was attempted.
	action string
	// cause is why it did not succeed.
	cause error
}

// Propagation with NO CHECK, which real code writes constantly. Returning the failure when it is
// absent IS returning success, so the source omits the test — and the target spells the same
// program as the operator followed by a success. `func FromBytes` in three of the surveyed corpora
// is this exact shape.

// Measure reports the length of s, failing on the empty string.
func Measure(s string) (int, error) {
	err := Check(s)
	return len(s), err
}
