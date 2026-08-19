package hard

// Scale shifts n left by the given amount.
//
// REFUSED. The source defines a shift at or beyond the operand width as ZERO and panics on a
// negative count; the target panics on the first in a debug build and masks the count in a release
// one. Three behaviours where the source has two, none of them matching — so the operator has no
// target form until the pack declares one, and emitting the plain shift produced a function that
// aborts where the source returns zero.
func Scale(n int64, by int64) int64 {
	n <<= by
	return n
}
