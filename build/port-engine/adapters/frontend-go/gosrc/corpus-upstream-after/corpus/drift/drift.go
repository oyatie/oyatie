// Package drift is one half of the UPSTREAM DRIFT pair, at the later version.
//
// Two changes from its twin under corpus-upstream-before/, chosen because they are what a real
// dependency bump looks like: an existing function's body changed, and a new function appeared.
// Neither is a change to the engine, the rules, or the toolchain — so the only receipt axis that
// may move is the one that describes the SOURCE, and the emitted difference is explained by it.
package drift

// Scale multiplies value by the tuning factor.
func Scale(value int) int {
	return value * 3
}

// Offset shifts value by the tuning offset.
func Offset(value int) int {
	return value + 1
}
