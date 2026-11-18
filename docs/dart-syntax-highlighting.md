# Dart Syntax Highlighting

Warp recognizes `.dart` files through Arborium's Dart grammar and its bundled
Tree-sitter highlight query. The query follows the Dart grammar's standard
categories for keywords, types, functions, calls, annotations, literals,
operators, properties, variables, constructors, and comments.

The fixture at `crates/languages/grammars/dart/standard_fixture.dart` covers
the common Dart constructs used by Flutter and Dart workspace code, including
annotations, generic types, named arguments, async functions, interpolation,
and null-safe-compatible syntax.

The language registry test verifies that the fixture parses without Tree-sitter
errors. Syntax-tree tests also verify that the standard Dart capture categories
map to Warp's existing theme colors.

## Verification

```bash
cargo test -p languages
TOOLCHAINS=Metal MACOSX_DEPLOYMENT_TARGET=15.0 cargo test -p syntax_tree
```

On a machine with the Metal Toolchain unavailable, the syntax-tree test can use
the existing test-only shader bypass:

```bash
WARP_LOCAL_SKIP_METAL=1 MACOSX_DEPLOYMENT_TARGET=15.0 cargo test -p syntax_tree
```
