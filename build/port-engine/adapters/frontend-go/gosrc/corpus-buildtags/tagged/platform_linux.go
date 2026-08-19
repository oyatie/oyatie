package tagged

// Platform names the target this file is selected for.
//
// Declared IDENTICALLY in platform_darwin.go. Both files cannot be in one package, and nothing but
// the filename says so — which is the point: an extractor that globs the directory type-checks a
// redeclaration and the whole package yields no measurement.
func Platform() string {
	return "linux"
}
