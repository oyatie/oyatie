// Package stored is a REFUSAL corpus: a failure the engine cannot prove is a failure.
//
// The source returns failure as a trailing result, and the target returns `Err(..)`. Those are the
// same claim only when the operand CANNOT be the absent value. Where it can, the source's caller
// compares against nil and sees SUCCESS, while the target's caller sees failure at exactly that
// point — a program that compiles and means something else, which is the one outcome this engine
// exists to prevent.
//
// The two declarations here have the SAME signature, the same field and different answers, because
// the proof is a property of the OPERAND rather than of the signature. An engine that read the
// signature alone would give them one answer and be wrong about one of them.
package stored

import "errors"

// Stored holds a failure rather than returning one.
type Stored struct {
	// cause is why the attempt did not succeed, and it may be absent.
	cause error
}

// Cause returns why the attempt did not succeed.
//
// TRANSLATED, and this comment previously said the opposite. The earlier reading was that a getter's
// optional form would be reading intent from a shape rather than proving it. What changed is that
// there IS a proof: every return of this sole-failure-result function reads a field of the RECEIVER,
// the same proof the borrowed string getter one type over already makes. The claim is not that this
// is semantically a getter — it is that the value handed back is a stored field that may be absent,
// and an optional borrow says exactly that. `Result` would say an operation succeeded or failed,
// which is a claim about an operation that is not here.
func (s *Stored) Cause() error {
	return s.cause
}

// Wrapped reports a fresh failure for the attempt.
//
// TRANSLATABLE, and here so the refusal above is a DISTINCTION rather than a blanket. The operand
// is a call to a declared failure constructor, which has no absent result to return.
func (s *Stored) Wrapped() error {
	return errors.New("attempt failed")
}

// Check reports the stored count and whether the attempt failed.
//
// REFUSED, and this is the fence. The trailing operand is the CHANNEL: the target spells it
// `Err(..)`, which reports failure unconditionally, and this operand is a stored field that may be
// absent — so the emitted program would report failure at exactly the points the source reported
// success. Unlike `Cause` there is no alternative spelling to fall back on, because the companion
// result makes the signature a fallible operation rather than a value handed back. The refusal must
// name the OPERAND rather than the signature, which is what makes the distinction from `Wrapped`
// possible at all.
func (s *Stored) Check() (int, error) {
	return 0, s.cause
}
