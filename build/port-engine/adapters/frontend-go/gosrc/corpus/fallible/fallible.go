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

// The one-value-and-a-failure shape: `(int, error)`, which is what most of a real package looks
// like.

// Length reports the length of s, failing on the empty string.
func Length(s string) (int, error) {
	if s == "" {
		return 0, errors.New("empty")
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
