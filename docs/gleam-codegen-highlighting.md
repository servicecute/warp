# Gleam and Codegen Highlighting

Warp's built-in code editor now recognizes `.gleam` files through the existing
Arborium/Tree-sitter language registry. Standard Gleam syntax uses Arborium's
upstream Gleam grammar and highlight query.

## Codegen overlay

Gleam documentation comments are still rendered as comments. Warp adds a
second, metadata-only pass over `doc_comment_content` nodes and recognizes the
shared codegen vocabulary from the workspace contract:

- `annotation.marker.gleam`: known `@codegen`, `@json`, `@firestore`, and related markers;
- `annotation.namespace.gleam`: format or backend values such as `json` and `firestore`;
- `annotation.key.gleam`: directive keys such as `tag_style` and `collection`;
- `annotation.string.gleam`: quoted annotation values;
- `annotation.number.gleam`: numeric annotation values.

Unknown directives remain ordinary comment text. The overlay does not change
Gleam parsing, Markdown behavior, or the language server.

The checked-in visual fixture is
`crates/languages/grammars/gleam/codegen_fixture.gleam`. The syntax-tree test
parses that fixture, verifies that the overlay is restricted to documentation
comments, and checks the semantic colors used for markers and keys.

## User configuration

Warp loads optional language overrides from `~/.warp/languages.yaml`. Tests and
isolated profiles can point to another file with `WARP_LANGUAGE_CONFIG`.

```yaml
languages:
  - language: gleam
    extensions: ["gleamx"]
    colors:
      annotation.marker.gleam: "#ffaff3"
      annotation.namespace.gleam: "#61afef"
    custom_highlights:
      - scope: annotation.key.gleam
        pattern: "@json\\s+([A-Za-z_][A-Za-z0-9_-]*)"
        capture: 1
```

Each `language` value must name a built-in Warp language. `extensions` adds
aliases that use that language's parser and highlighter. `custom_highlights`
appends regex rules to a language's configured custom-highlighting pass;
`capture: 0` highlights the whole match and a positive capture selects a
captured subrange. Configuration is loaded when Warp starts, so changes require
restarting Warp. Invalid user rules are logged and skipped individually; they do
not disable the built-in rules or prevent Warp from starting.

## Verification

The language registry tests cover `.gleam` resolution, embedded codegen rules,
unknown-directive behavior, regex capture validation, and the Dart language
fixture. The syntax-tree tests cover the Gleam overlay and Dart's standard
capture categories.

```bash
cargo test -p languages
TOOLCHAINS=Metal MACOSX_DEPLOYMENT_TARGET=15.0 cargo test -p syntax_tree
```
