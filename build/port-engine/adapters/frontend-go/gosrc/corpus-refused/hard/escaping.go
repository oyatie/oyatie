package hard

// Widths returns the horizontal extents it was given, unchanged.
//
// REFUSED. The source's slice parameter SHARES the caller's backing array, so it is a reference
// the caller keeps — and returning it hands that share out past the call. A borrow would need a
// lifetime the caller cannot supply, and an owned parameter would consume the caller's slice,
// which the source never does. What an escaping sequence becomes has not been decided.
func Widths(counts []int64) []int64 {
	return counts
}
