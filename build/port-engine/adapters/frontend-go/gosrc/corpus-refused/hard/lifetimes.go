package hard

// Label renders a display name for the given identity, falling back when it is empty.
//
// REFUSED. Both parameters are strings, which the source shares with the caller, and the result is
// ONE OF THEM — so the target's signature needs a lifetime tying the result to the arguments.
// Nothing here emits lifetimes yet. Taking them owned instead would consume two values the source
// never consumed, which is what this used to do.
func Label(id string, fallback string) string {
	if id == "" {
		return fallback
	}
	return id
}
