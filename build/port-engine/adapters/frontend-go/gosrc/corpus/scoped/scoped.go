// Package scoped exists to prove the `if` INIT CLAUSE, which blocked six of seven surveyed
// third-party packages and is the most common statement shape in real Go after the plain call.
//
// `if x := f(); cond` scopes x to the condition and to both branches, and to nothing after. The
// target has exactly one construct with that shape — a block — so the translation is a block whose
// first statement is the binding and whose last is the conditional.
//
// Hoisting the binding into the enclosing scope would also compile, and would be a different
// program: the name would outlive the branch and drop later. That is what makes this a
// translation rather than a rewrite.
package scoped

// The plain shape: bound in the init clause, read in the condition and the branch.

// Width reports the byte length of s once it clears the minimum, and zero otherwise.
func Width(s string) int {
	if size := len(s); size > 4 {
		return size
	}
	return 0
}

// The binding is read in BOTH branches, which is what makes the block the only faithful shape: a
// name hoisted out of the `if` would still be live after it, and this one must not be.

// Span reports the byte length of s, counting a short string as one byte wider than it is.
func Span(s string) int {
	result := 0
	if size := len(s); size > 4 {
		result = size
	} else {
		result = size + 1
	}
	return result
}

// A body-scoped CONST, which is a binding rather than a target constant — and that is a decision
// about what it MEANS, not what it is called. The source's untyped constant has no type until it is
// used and takes one from each use; a target `const` must fix a type at the declaration, and a
// target `let` takes one from use exactly as the source's does.
//
// The cost is stated rather than hidden: a source constant used at TWO different types in one
// function has no single target binding. That does not compile, which is the safe failure — it
// never means something else.

// Scaled reports n scaled by the fixed factor this function works in.
func Scaled(n int64) int64 {
	const factor = 8
	return n * factor
}
