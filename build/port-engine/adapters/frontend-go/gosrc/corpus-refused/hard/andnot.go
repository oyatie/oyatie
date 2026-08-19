package hard

// Clear turns off the bits of mask in n.
//
// `&^=` (AND NOT) has no single-operator target form, exactly as binary `&^` has none. It is
// spellable as `& !`, and the operand widths differ between the languages — a silent rewrite of a
// bit operation is the class of change nobody reviews, so it is refused by name instead.
func Clear(n int64, mask int64) int64 {
	n &^= mask
	return n
}
