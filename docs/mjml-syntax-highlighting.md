# MJML syntax highlighting

MJML files (`*.mjml`) are registered as a supported language in Warp's editor.

The initial implementation intentionally reuses Arborium's HTML grammar and
highlight query. This gives MJML a useful baseline for tag names, attributes,
strings, comments, and HTML-like structure without maintaining a second markup
grammar for the shared syntax.

The implementation includes a representative fixture covering MJML document
structure, attributes, CSS in `mj-style`, and a Handlebars-style interpolation.
Language loading, parsing, and highlight-query captures are covered by the
`languages` crate tests.

## Scope boundary

This is a markup-highlighting integration, not a Handlebars implementation. The
current HTML grammar treats template expressions such as `{{recipient_name}}`
as text. A future Handlebars PR should define delimiter-aware highlighting and
the behavior for Handlebars embedded in HTML, MJML, and other host languages.

Likewise, CSS inside `mj-style` and inline `style="..."` attributes are not yet
switched to a dedicated embedded CSS grammar. They currently inherit the HTML
baseline: block contents are mostly plain text and inline values are highlighted
as attribute strings. That can be added separately once the editor's
embedded-language highlighting path is established.

## Follow-up tasks: embedded CSS

1. Add CSS grammar injection for the contents of `<mj-style>` blocks.
2. Add CSS grammar injection for inline `style="..."` attribute values.
3. Define precedence and recovery when CSS contains MJML/Handlebars template
   expressions or incomplete declarations.
4. Add fixtures and rendered-capture tests for selectors, properties, values,
   comments, colors, nested blocks, and malformed CSS.
5. Verify that CSS captures do not recolor surrounding MJML tags, attributes,
   or template expressions.

## Verification

```bash
cargo test -p languages
```
