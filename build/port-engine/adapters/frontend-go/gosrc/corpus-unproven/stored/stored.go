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
// REFUSED. The operand is a stored field that may be absent, and neither proof the engine has
// applies to it: it is not a call to a declared failure constructor, and it is not the address of a
// fresh composite. What the source means here is a GETTER, and the target spells a getter as an
// optional rather than as a result — but choosing that would be reading intent from a shape, not
// proving it, so this refuses and says which proof is missing.
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
