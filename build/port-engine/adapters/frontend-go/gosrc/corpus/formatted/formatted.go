// Package formatted exists to prove the FORMATTING call, which appears in six of the seven
// third-party packages surveyed and refused in every one of them before this rule.
//
// Building a string from a template is the most common call in real Go after the plain one, and it
// is how nearly every error message in a real package is made. The target has a macro that does the
// same thing — but the two write their templates differently, so this is a translation of the
// template and not only of the call.
//
// The verbs the pack maps are a CLOSED set. What is left out is left out because it PRINTS
// something else, and a verb rendered as the wrong placeholder produces a program that compiles and
// prints something different — which is the one failure this engine exists to prevent.
package formatted

import "fmt"

// The plain shape: a template with one value in it.

// Describe renders n with a label.
func Describe(n int64) string {
	return fmt.Sprintf("count %d", n)
}

// SEVERAL values, in order. The target's macro checks the correspondence between placeholders and
// arguments at compile time and the source does not, so a template that disagrees with its call is
// a defect the source hid and the target surfaces.

// Pair renders two values as one string.
func Pair(name string, n int64) string {
	return fmt.Sprintf("%s has %d", name, n)
}

// The QUOTED verb, which is a different placeholder rather than the same one. Both render the
// value; only one escapes it and puts it in quotes, and rendering this as the plain placeholder
// would silently drop the quoting a caller reads.

// Quoted renders s in quotes.
func Quoted(s string) string {
	return fmt.Sprintf("got %q", s)
}

// A template with NO values at all, which is still a template — and the target's macro takes one
// argument in that case rather than a trailing empty list.

// Fixed renders the fixed message.
func Fixed() string {
	return fmt.Sprintf("nothing to report")
}

// The ESCAPED percent, which is the source's way of writing one literally. The target writes it
// plainly, because the target gives no meaning to a percent at all.

// Percentage renders n as a percentage.
func Percentage(n int64) string {
	return fmt.Sprintf("%d%% done", n)
}

// A literal BRACE, which is data in the source's template and opens a placeholder in the target's.
// Doubling it is how the target spells one; skipping that would turn this message into a parse
// error, or worse into a placeholder that consumes an argument meant for something else.

// Braced renders n inside braces.
func Braced(n int64) string {
	return fmt.Sprintf("{%d}", n)
}

// The FAILURE form, which is the same template wrapped in the pack's failure type. This is how
// nearly every error in a real package is built, and it is why the formatting call ranked first by
// packages blocked.

// Checked reports n, failing when it is negative.
func Checked(n int64) (int64, error) {
	if n < 0 {
		return 0, fmt.Errorf("negative count %d", n)
	}
	return n, nil
}
