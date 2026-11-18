# Embedded-language highlighting foundation

Warp now treats embedded highlighting as an ordered composition of child
highlighters rather than as special cases beside the parent grammar.

## Composition contract

Each `Language` declares its embedded languages in precedence order. The
syntax-tree layer builds one child-highlighter pipeline from that declaration:

1. Parse and highlight the parent language.
2. Discover only the regions owned by the declared child language, or produce
   a tolerant token stream when the child does not have a tree-sitter grammar.
3. Translate child captures back into parent-buffer character ranges.
4. Apply child overlays in declaration order; later entries win overlaps.

For MJML the current order is CSS followed by Handlebars. This means a
Handlebars expression inside an MJML style region remains a template token if
the two overlays overlap, while ordinary CSS remains CSS-highlighted.

## Recovery and ownership

Child parsers never receive the whole parent document. An incomplete child
region is bounded by the available source and cannot recolor unrelated parent
markup. A child highlighter may be tolerant and lexical, as Handlebars is,
when a complete grammar is unavailable.

The composition layer owns range mapping and precedence. Individual language
implementations own only their region discovery, parsing/tokenization, and
token categories.

## Current and future consumers

The foundation currently composes:

- MJML + embedded CSS;
- MJML + embedded Handlebars; and
- HTML/JSON hosts + Handlebars.

Future PRs may add HTML/CSS/JavaScript, Markdown fenced languages,
Dockerfile shell fragments, or SQL/SurrealQL injections without changing the
parent highlighting contract.

## Verification

```bash
cargo test -p languages
cargo test -p syntax_tree
```

The syntax-tree tests cover language-scoped region discovery, CSS capture
mapping, Handlebars token recovery, and the existing parent-language tests.
