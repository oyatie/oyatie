// Package drift is one half of the UPSTREAM DRIFT pair, at the earlier version.
//
// The pair exists to prove the property a continuously maintained port rests on: when upstream
// moves, re-running the engine must produce output whose change is EXPLAINED by a receipt axis
// rather than merely different. An engine that re-ports a moved dependency and reports
// `Unexplained` is telling its operator that something is wrong with the engine — and if it says
// that every time upstream moves, the signal is worthless and nobody will read it again.
//
// Its twin under corpus-upstream-after/ is the same package at a later version: one body changed,
// one declaration added. Both live at the same unit id on purpose, because a port that re-ran
// against a renamed unit would be comparing two different things and would find a difference for
// the wrong reason.
package drift

// Scale multiplies value by the tuning factor.
func Scale(value int) int {
	return value * 2
}
