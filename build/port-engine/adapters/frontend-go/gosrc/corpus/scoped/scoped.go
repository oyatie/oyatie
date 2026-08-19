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

// Width reports the width of s once it clears the minimum, and zero otherwise.
//
// The plain shape: bound in the init clause, read in the condition and the branch.
func Width(s string) int {
	if size := len(s); size > 4 {
		return size
	}
	return 0
}

// Span reports the width of s, counting a short string as one wider than it is.
//
// The binding is read in BOTH branches, which is what makes the block the only faithful shape: a
// name hoisted out of the `if` would still be live after it, and this one must not be.
func Span(s string) int {
	result := 0
	if size := len(s); size > 4 {
		result = size
	} else {
		result = size + 1
	}
	return result
}
