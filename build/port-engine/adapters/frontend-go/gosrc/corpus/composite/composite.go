// Package composite proves EMBEDDING, on both sides of it.
//
// Go composes by embedding, and nothing forwards: an anonymous field lifts the embedded type's
// methods into the outer type's method set, and an embedded interface lifts its requirements into
// the outer interface's. The target has neither rule, so both have to become explicit — forwarding
// methods for the first, supertraits for the second.
//
// The two are exercised TOGETHER because their interaction is the part that can go wrong on its
// own. `Driver` satisfies `Job` only through a promoted method, so an engine that emitted the
// supertraits and skipped the promotion would produce an impl that names a method nothing
// implements — which compiles nowhere and is caught by nothing short of compiling it.
package composite

// Runner is anything that can run.
type Runner interface {
	// Run performs one unit of work and reports the running count.
	Run() int
}

// Describer is anything that can describe itself.
type Describer interface {
	// Describe renders a description.
	Describe() string
}

// Embeds two interfaces and declares no method of its own, which is the shape 87.3% of embedding
// interfaces have — and the shape whose emitted trait says nothing at all unless the supertraits
// are carried.

// Job is anything that runs and describes itself.
type Job interface {
	Runner
	Describer
}

// Engine counts the work it has done.
type Engine struct {
	// calls is how many times Run has been called.
	calls int
}

// MUTATING, which is what makes the promoted method interesting: the forwarding method's receiver
// is decided by what the method it forwards to does, and there is no body on the outer type to
// observe.

// Run performs one unit of work and reports the running count.
func (e *Engine) Run() int {
	e.calls = e.calls + 1
	return e.calls
}

// Driver embeds an Engine and gains its methods.
type Driver struct {
	// Engine is embedded, so Driver's method set includes Run without declaring it.
	Engine
	// label describes this driver.
	label string
}

// NewDriver returns a driver labelled for display, driving the given engine.
func NewDriver(engine Engine, label string) Driver {
	return Driver{Engine: engine, label: label}
}

// Describe renders a description.
func (d *Driver) Describe() string {
	return d.label
}

// Assertion site: satisfying Job satisfies Runner and Describer too, and the Go compiler checks
// all three. Run is reached only through the embedded Engine.
var _ Job = (*Driver)(nil)
