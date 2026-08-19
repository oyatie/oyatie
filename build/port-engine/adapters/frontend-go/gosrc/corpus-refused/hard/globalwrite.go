package hard

// counter is written by Bump, which is the case the `var` deferral's synchronization argument is
// actually about.
var counter int64

// pooled is both initialised and written.
var pooled = int64(1)

// Bump advances the counter and rescales the pool.
//
// REFUSED, and the reason is a property of the engine rather than of this code: `counter` and
// `pooled` are deferred, so they are not emitted, and a body naming them would produce a crate
// with a dangling name. What the engine emits has to be self-contained.
func Bump(by int64) int64 {
	counter += by
	pooled *= 2
	return counter
}
