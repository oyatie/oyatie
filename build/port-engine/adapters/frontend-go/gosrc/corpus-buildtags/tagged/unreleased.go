//go:build go1.99

package tagged

// Unreleased must NOT appear in any snapshot: no declared release selects it.
//
// It fences the release-tag pin. Left to the host toolchain's own tags, whether this declaration
// exists becomes a property of the machine that ran the extractor.
func Unreleased() bool {
	return true
}
