// Package sequence exists to prove SLICE and ARRAY literals, which reached the model as
// `unsupported` 26 times across the surveyed corpora and are the shape a real package uses for
// every buffer and every table.
//
// An EMPTY array literal is the case that matters most — eleven of the twenty-six — and it is not
// an empty array at all. `[20]byte{}` is twenty zero bytes, which the target spells as the type's
// own zero value; answering it with an empty construction would need the engine to invent a length
// it already has.
//
// A map literal is deliberately absent and refuses by name. The source's map has no order and the
// target's ordered map imposes one, so the entry order becomes observable where it was not — a
// decision that needs its own reason rather than a row in a table.
package sequence

// Buffer returns a zeroed buffer of the fixed width the protocol uses.
func Buffer() [4]int64 {
	return [4]int64{}
}

// Seeds returns the fixed seed table.
func Seeds() [3]int64 {
	return [3]int64{11, 13, 17}
}

// Widths returns the configured widths.
func Widths() []int64 {
	return []int64{1, 2, 3}
}

// Empty returns no widths at all.
func Empty() []int64 {
	return []int64{}
}
