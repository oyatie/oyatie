// Package accumulate exists to prove READ-MODIFY-WRITE assignment, which reached the model as
// `unsupported` 69 times across the surveyed corpora and blocked four packages.
//
// `x op= y` means `x = x op y` in both languages and evaluates the place expression once in both.
// It introduces no decision the binary operator has not already made, so refusing it refused the
// same decision twice.
//
// What stays refused is the shape that genuinely differs: `&^=` has no single-operator target
// form, and parallel assignment evaluates every right-hand side before assigning any left. Those
// live in the refusal corpus.
package accumulate

// Total sums the values, accumulating in place.
func Total(values []int64) int64 {
	sum := int64(0)
	for i := 0; i < len(values); i++ {
		sum += values[i]
	}
	return sum
}

// `^=`, `|=` and `*=` are the three most common compound forms in the surveyed source after `+=`,
// and all four are the same translation.

// Mix folds the values together with xor, multiply and or.
//
// The three operators are the point rather than the result: each is a compound assignment on the
// same accumulator, and the source spells the read-modify-write once where the target has to decide
// whether the place is read twice.
func Mix(values []int64) int64 {
	acc := int64(1)
	for i := 0; i < len(values); i++ {
		acc ^= values[i]
		acc *= 3
		acc |= 1
	}
	return acc
}
