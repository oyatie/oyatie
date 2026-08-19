package hard

// Join concatenates the given counts into a total.
//
// The SIGNATURE ports: `counts` is a slice inside the function, which is what the source records
// and what the target's slice rule already answers.
func Join(base int64, counts ...int64) int64 {
	return base
}

// Total joins two counts onto a base.
//
// REFUSED, and at the CALL rather than at the declaration above it. The target has no variadic
// call, so the trailing arguments need collecting into a sequence — and neither which sequence
// form that is, nor what a forwarded slice becomes, has been decided.
func Total(base int64) int64 {
	return Join(base, 1, 2)
}
