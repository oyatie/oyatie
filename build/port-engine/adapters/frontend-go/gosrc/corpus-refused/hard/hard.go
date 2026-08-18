// Package hard is the fixture the engine is expected to REFUSE.
//
// It is deliberately NOT part of `corpus/`, because the pipeline over that corpus must stay
// green. This one exists so the refusal path is exercised against real Go rather than against
// synthetic nodes assembled in a test: a translator whose refusals are only ever tested on
// hand-built inputs has not been shown to refuse anything a front end would actually produce.
//
// Every declaration here uses only types the pack maps, so the refusal that fires is the one
// about the CONSTRUCT and not an incidental one about a type.
package hard

// Countdown sums 0..n with a loop.
//
// A `for` statement has no translation yet. Go's three-clause form, its condition-only form, and
// its range form are three different target constructs, and choosing among them is a rule.
func Countdown(n int) int {
	total := 0
	for i := 0; i < n; i++ {
		total = total + i
	}
	return total
}

// Guarded returns n after a deferred cleanup.
//
// `defer` is the subject of docs/programs/k8s-port/census/defer-panic-recover.md. Rust's drop
// glue runs at scope exit like a defer does, but ordering, panics, and named-result mutation all
// differ, so this is a rule to be written rather than a shape to be assumed.
func Guarded(n int) int {
	defer cleanup()
	return n
}

func cleanup() {}
