//go:build go1.13

package tagged

// Recent is declared only from go1.13 onward.
//
// This is the QUIET case. Nothing collides, so an extractor that ignores the constraint admits it
// as an unconditional declaration of the package and no error is ever raised. `pkg/errors` ships
// exactly this shape and its `Is`, `As` and `Unwrap` entered a snapshot that way.
func Recent() bool {
	return true
}
