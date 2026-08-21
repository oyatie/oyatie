// Package escaping is the fixture whose OWNERSHIP the engine must refuse.
//
// Its sibling `hard` covers constructs with no translation yet. This one covers a construct that
// translates fine and whose ownership cannot be decided: a method whose receiver outlives the
// call. No borrow of `self` can be handed out — a reference would need a lifetime the caller
// cannot supply — so the pack's escaping disposition declares no receiver form and the transform
// refuses rather than picking a borrow that will not hold.
package escaping

// Node is a self-referential structure.
type Node struct {
	// label names the node.
	label string
}

// Itself returns the receiver.
//
// ESCAPES: the pointer outlives the call.
func (n *Node) Itself() *Node {
	return n
}
