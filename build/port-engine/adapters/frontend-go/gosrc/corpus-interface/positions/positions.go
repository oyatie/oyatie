// Package positions is a REFUSAL corpus: an interface in a position with no declared form.
//
// Go holds an interface value directly. The target cannot — a trait has no size — so it reaches a
// position as `&dyn T`, `Box<dyn T>`, `Rc<dyn T>` or a generic parameter, and those are different
// answers to who owns the value and how long it lives. The pack declares a form for the PARAMETER
// position, where a borrow is unambiguously right and the caller keeps the value, and none for a
// RESULT — which is what this package proves refuses.
//
// It carries the other two SATISFACTION SITE KINDS as well. `Repeat` passes a concrete value into
// an interface-typed parameter and `Voice` returns one, so the front end observes Echo satisfying
// Speaker twice over, by inference rather than by declaration. Neither pair reaches the emitted
// crate — the package refuses first — so the facts are proven on the snapshot, which is the honest
// place to prove a collector whose output the emitter cannot yet consume.
package positions

// Speaker is anything that can say something.
type Speaker interface {
	// Say returns what the speaker says.
	Say() string
}

// Echo repeats what it is given.
type Echo struct {
	// phrase is what this echo repeats.
	phrase string
}

// Say returns the phrase.
func (e *Echo) Say() string {
	return e.phrase
}

// Announce renders a speaker. The parameter position HAS a declared form.
func Announce(speaker Speaker) string {
	return speaker.Say()
}

// Repeat is the ARGUMENT site: a concrete value passed into an interface-typed parameter.
func Repeat(echo *Echo) string {
	return Announce(echo)
}

// Whisper is a second speaker, whose ONLY observed site is a result.
//
// Echo cannot prove the result collector: it is observed at an argument first, and a pair seen
// twice keeps the site that proves the most. A type with nowhere else to be seen is the only way
// to show that the result walk finds anything at all.
type Whisper struct {
	// phrase is what this whisper says.
	phrase string
}

// Say returns the phrase.
func (w *Whisper) Say() string {
	return w.phrase
}

// Voice is the RESULT site, and the position with no declared form: the target has to own what it
// returns, and choosing the owner is an ownership decision the pack has not made.
func Voice(whisper *Whisper) Speaker {
	return whisper
}
