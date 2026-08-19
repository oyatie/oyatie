package hard

// EXPORTED and never written HERE, which is not the same as never written. Anything that imports
// this package may assign to it, and a package that documents "set this to false" is describing
// exactly that. The engine cannot see those writes, so the mutable-global decision applies to it
// from outside rather than from within — and the const it would otherwise become would delete a
// feature while keeping the prose describing it.

// Coerce reports whether the parser should coerce its input. Set it to false to make parsing
// strict.
var Coerce = true

// Coerced reports the setting.
func Coerced() bool {
	return Coerce
}
