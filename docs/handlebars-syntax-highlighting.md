# Handlebars syntax highlighting

Warp supports Handlebars through a tolerant lexical overlay. The workspace
does not currently bundle a Tree-sitter Handlebars grammar, and template files
are frequently incomplete while being edited. The overlay therefore runs on
top of the host language parser instead of replacing it.

## Supported hosts

- Standalone `.hbs` and `.handlebars` files use the HTML grammar as a useful
  host baseline.
- `.json` files can contain Handlebars expressions while retaining JSON
  highlighting.
- Explicit JSON templates use `.json.hbs` or `.json.handlebars` and use the
  JSON grammar as their host baseline.
- MJML and HTML receive Handlebars overlays without losing markup captures.

## Token coverage

The lexer recognizes escaped and unescaped output, block helpers, inverse
branches, paths, custom helpers, partials, hash arguments, strings, numbers,
boolean/null literals, comments, and incomplete expressions. Helper names are
classified structurally rather than from a closed registry, so application-
specific helpers remain highlightable.

Malformed expressions are highlighted up to the available source boundary. An
escaped `\\{{` sequence remains ordinary host text.

## Scope boundary

This feature highlights template syntax only. It does not evaluate templates,
validate helper names, or change the existing Handlebars rendering crate.
Template token precedence with additional embedded languages belongs to the
general embedded-language architecture follow-up.

## Verification

```bash
cargo test -p languages
cargo test -p syntax_tree
```
