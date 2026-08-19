package hard

import "errors"

// Rethrow aborts carrying an error value.
//
// REFUSED. The target's `panic!` carries a formatted string, so the payload TYPE is lost: a caller
// that recovers and type-asserts on the error would see a different program. The pack answers for
// `panic` only where the payload is a string literal, and says so by name.
func Rethrow() {
	panic(errors.New("boom"))
}
