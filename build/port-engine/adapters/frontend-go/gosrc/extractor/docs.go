package main

import (
	"go/ast"
	"go/types"
	"strings"
)

// Documentation capture.
//
// `parser.ParseComments` is what makes any of this reachable: without it every `Doc` field is nil
// and the loss is total and silent, which is exactly how 18 blocks were dropped before.

func commentText(group *ast.CommentGroup) string {
	if group == nil {
		return ""
	}
	return strings.TrimRight(group.Text(), "\n")
}

// indexGenDeclDocs records documentation for const, var and type declarations.
//
// A GenDecl may carry the comment itself (`// Doc\ntype T struct{}`) or leave it on the single
// spec inside a parenthesised group, so both are checked and the spec's own comment wins — it is
// the more specific of the two.
func indexGenDeclDocs(
	decl *ast.GenDecl,
	tpkg *types.Package,
	docs map[types.Object]string,
	fieldDocs map[string]string,
) {
	groupDoc := commentText(decl.Doc)
	for _, spec := range decl.Specs {
		switch typed := spec.(type) {
		case *ast.TypeSpec:
			if typed.Name == nil {
				continue
			}
			if obj := tpkg.Scope().Lookup(typed.Name.Name); obj != nil {
				if text := firstNonEmpty(commentText(typed.Doc), groupDoc); text != "" {
					docs[obj] = text
				}
			}
			indexMemberDocs(typed, fieldDocs)
		case *ast.ValueSpec:
			for _, name := range typed.Names {
				obj := tpkg.Scope().Lookup(name.Name)
				if obj == nil {
					continue
				}
				if text := firstNonEmpty(commentText(typed.Doc), groupDoc); text != "" {
					docs[obj] = text
				}
			}
		}
	}
}

// indexMemberDocs keys a struct field's or an interface method's documentation by
// "TypeName.MemberName".
//
// Neither is a package-scope object, so there is no types.Object to index it by, and keying by
// source position would break the moment a member moves. Both shapes are an `*ast.FieldList` on a
// TypeSpec, which is why one function answers for them — an earlier version matched only
// `*ast.StructType` and dropped every interface method's documentation in silence.
func indexMemberDocs(spec *ast.TypeSpec, fieldDocs map[string]string) {
	if spec.Name == nil {
		return
	}
	var members *ast.FieldList
	switch typed := spec.Type.(type) {
	case *ast.StructType:
		members = typed.Fields
	case *ast.InterfaceType:
		members = typed.Methods
	}
	if members == nil {
		return
	}
	for _, member := range members.List {
		text := firstNonEmpty(commentText(member.Doc), commentText(member.Comment))
		if text == "" {
			continue
		}
		// An EMBEDDED field has no name in the syntax — its name is its type — so keying by
		// `Names` alone dropped every comment on one, which is the population this whole pass
		// exists to stop losing.
		for _, name := range memberNames(member) {
			fieldDocs[spec.Name.Name+"."+name] = text
		}
	}
}

// memberNames reports the names a field list entry declares, including the implicit name an
// embedded field takes from its type.
func memberNames(member *ast.Field) []string {
	if len(member.Names) > 0 {
		names := make([]string, 0, len(member.Names))
		for _, name := range member.Names {
			names = append(names, name.Name)
		}
		return names
	}
	if name := embeddedName(member.Type); name != "" {
		return []string{name}
	}
	return nil
}

// embeddedName is the field name Go gives an anonymous field: the type's own name, through any
// pointer or qualifier.
func embeddedName(expr ast.Expr) string {
	switch typed := expr.(type) {
	case *ast.Ident:
		return typed.Name
	case *ast.StarExpr:
		return embeddedName(typed.X)
	case *ast.SelectorExpr:
		return typed.Sel.Name
	default:
		return ""
	}
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if value != "" {
			return value
		}
	}
	return ""
}

func withDoc(attrs map[string]string, text string) map[string]string {
	if text == "" {
		return attrs
	}
	return withAttr(attrs, attrDoc, text)
}

func withAttr(attrs map[string]string, key string, value string) map[string]string {
	if attrs == nil {
		attrs = map[string]string{}
	}
	attrs[key] = value
	return attrs
}

// typeTree renders a go/types type as a tree.
//
// Deliberately does NOT unalias: an alias is a name the source chose, and resolving it here would
// discard the author's vocabulary before the pack ever sees it. The pack can unalias if it wants
// to; it cannot re-alias.
