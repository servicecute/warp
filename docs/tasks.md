# Warp editor and syntax tasks

This file tracks independently reviewable tasks for the ServiceCute Warp fork.
Each language feature should remain focused on one parser/registry surface,
its fixtures, and its verification. Larger editor architecture changes should
not be mixed into grammar-only pull requests.

## PR sequencing

| Order | Task | Status | Dependency |
| --- | --- | --- | --- |
| 1 | Dart syntax highlighting | In PR #2 | None |
| 2 | Custom Warp release workflow | In PR #3 | None |
| 3 | Internal editor text fallback | In PR #8 | None |
| 4 | Erlang syntax highlighting | Planned | None |
| 5 | SurrealQL syntax highlighting | Planned | None |
| 6 | MJML syntax highlighting | In PR #4 | None |
| 7 | Embedded CSS highlighting | In PR #5 | MJML syntax highlighting |
| 8 | Handlebars syntax highlighting | In PR #6 | MJML baseline; template-token design |
| 9 | Embedded-language highlighting foundation | In PR #7 | Tasks 7 and 8 |
| 10 | Semantic highlighting | Future | LSP/semantic-token design |

## Task 1 — Dart syntax highlighting

Status: In PR #2.

Keep this PR limited to the Dart language registry, Arborium grammar/query,
fixtures, tests, and Dart-specific documentation.

Acceptance criteria:

- `.dart` resolves to Dart in the language registry.
- Standard Dart constructs parse without errors.
- Imports, annotations, classes, generics, fields, constructors, async code,
  interpolation, literals, and operators receive expected captures.
- Parser, filename-resolution, and rendered-capture tests pass.
- Manual review is completed in the local Warp GUI build.

## Task 2 — Custom Warp release workflow

Status: In PR #3.

Keep this PR separate from language features. It must build the current
checkout as `WarpOss.app`, preserve the `dev.warp.WarpOss` identity, and install
beside stable `/Applications/Warp.app` without sharing its app profile.

Acceptance criteria:

- The workflow supports build-only, force replacement, custom install
  directory, and open-after-install modes.
- The workflow never replaces `/Applications/Warp.app`.
- Shell syntax and help checks pass.
- The resulting app and DMG paths are documented.

## Task 3 — Internal editor text fallback

Status: In PR #8.

Fix routing for repository text files that do not have a registered language.
Unknown language must not imply unknown content or cause an implicit handoff
to the operating-system editor.

Implementation scope:

- Add a centralized file classification/open-intent path.
- Route valid UTF-8 unknown files to Warp's internal plain-text editor.
- Recognize dotfiles and extensionless files such as `.gitmodules`,
  `.dockerignore`, `.editorconfig`, `.env*`, `.npmrc`, and agent metadata.
- Treat NUL-containing or undecodable files as binary candidates.
- Preserve an explicit `Open in default application` action.
- Keep content classification independent from language detection.

Acceptance tests:

- Known source file opens with its language.
- Unknown UTF-8 dotfile opens internally.
- Unknown extensionless UTF-8 file opens internally.
- Binary file remains eligible for external opening.
- Misleading extension is handled by the documented classification policy.
- Explicit external-open action still works.

## Task 4 — Erlang syntax highlighting

Status: Planned.

Add `.erl` and `.hrl` support in a focused grammar PR.

Implementation scope:

- Select a maintained Tree-sitter grammar compatible with Arborium.
- Prefer an Arborium package; document any direct grammar dependency and
  license if one is required.
- Register `erlang` and map `.erl` and `.hrl`.
- Add comments, indentation, and bracket-pair configuration.
- Cover atoms, variables, modules, functions, records, macros, strings,
  numbers, keywords, operators, guards, binaries, pattern matching, and
  receive expressions.

Acceptance tests:

- Representative Erlang module and header fixtures parse.
- Filename resolution works for both extensions.
- Rendered captures match the expected categories.
- The focused language test suite passes before release integration.

## Task 5 — SurrealQL syntax highlighting

Status: Planned.

Add `.surql` support without coupling the first PR to a broader embedded-
language framework.

Implementation scope:

- Select a compatible maintained SurrealQL grammar.
- Register `surrealql` and map `.surql`.
- Configure line/block comments, indentation, and SQL-style brackets.
- Highlight statements, record IDs, tables, fields, parameters, functions,
  literals, operators, graph relations, permissions, and comments.
- Add fixtures for `DEFINE`, `SELECT`, `RELATE`, record links, nested objects,
  arrays, parameters, transactions, schemas, and migrations.

Acceptance tests:

- `.surql` resolves correctly.
- Fixtures parse without errors.
- Structured-query captures render as expected.
- Registry and focused language tests pass.

## Task 6 — MJML syntax highlighting

Status: Planned.

Add a reliable baseline for `.mjml` before introducing a general embedded-
language system.

Implementation scope:

- Decide whether the HTML grammar is sufficient or an MJML-specific grammar
  is required.
- Register `mjml` and map `.mjml`.
- Highlight MJML components, attributes, HTML, comments, and strings.
- Leave embedded CSS and Handlebars template semantics to Tasks 7 and 8.
- Cover `mjml`, `mj-head`, `mj-body`, sections, columns, buttons, images,
  embedded styles, and conditional comments.

Acceptance tests:

- MJML files resolve to the intended language.
- Parent HTML/CSS content remains readable.
- Custom MJML component names are not falsely treated as syntax errors.
- MJML fixtures and rendered-capture tests pass.

## Task 7 — Embedded CSS highlighting

Status: In PR #5.

Add CSS grammar parsing for the two CSS forms used by MJML. Keep this PR
focused on CSS range discovery and capture merging; do not introduce the full
general embedded-language architecture yet.

Implementation scope:

- Parse the contents of `<mj-style>` blocks with the CSS grammar.
- Parse declarations in inline `style="..."` attributes.
- Map embedded captures back to the parent MJML buffer without recoloring
  surrounding tags, attributes, or template text.
- Recover from incomplete style blocks, declarations, and malformed CSS.

Acceptance criteria:

- Block and inline CSS fixtures produce CSS captures.
- Incomplete embedded regions do not prevent the parent document from loading.
- Focused language and syntax-tree tests pass.
- Manual review confirms CSS colors appear in the MJML editor.

## Task 8 — Handlebars syntax highlighting

Status: In PR #6.

Add delimiter-aware highlighting for standalone Handlebars templates and
template expressions embedded in MJML. Keep this separate from CSS because
template parsing has different escaping, nesting, helper, and recovery rules.

Implementation scope:

- Register `.hbs` and `.handlebars` files.
- Cover escaped and unescaped expressions, comments, block helpers, inverse
  branches, paths, parameters, hash arguments, literals, and partials.
- Define how Handlebars expressions are highlighted inside MJML text and HTML
  attributes without corrupting the host language's captures.
- Decide how malformed or incomplete `{{...}}` expressions recover.

Acceptance criteria:

- Standalone Handlebars fixtures parse or lex without blocking surrounding text.
- Delimiters, helpers, paths, literals, comments, and block boundaries receive
  stable categories.
- MJML fixtures preserve both MJML and Handlebars highlighting.
- Escaping and incomplete-expression cases have focused tests.
- Manual review confirms template colors are distinct from markup and CSS.

## Task 9 — Embedded-language highlighting foundation

Status: In PR #7.

Generalize the region discovery and capture-merging path proven by Tasks 7 and
8 into an explicit grammar-composition/injection model. This PR is limited to
the composition boundary and must not add new language grammars.

Initial targets:

- Keep the existing MJML CSS and Handlebars highlighters behind one pipeline.
- Make each embedded language declare its owned regions or token stream.
- Make precedence explicit and deterministic: later configured injections win
  overlapping ranges.
- Leave future HTML/CSS/JavaScript, Markdown fences, Dockerfile shell
  fragments, and SQL/SurrealQL injections for follow-up language PRs.

Acceptance criteria:

- Parent and child grammar ownership is explicit.
- Child highlighting cannot corrupt parent comments, strings, or delimiters.
- Nested regions have deterministic precedence and recovery behavior.
- Fixtures cover valid, incomplete, and malformed embedded regions.
- Existing flat-language highlighting remains unchanged.

## Task 10 — Semantic highlighting

Status: Future.

Add optional LSP semantic-token support as a separate PR from grammar work.
Syntax highlighting must remain the fallback when a language server is absent,
unavailable, or returns incomplete semantic tokens.

Implementation scope:

- Define semantic-token categories and theme mapping.
- Connect language-server token updates to the editor buffer.
- Handle cancellation, stale responses, server restarts, and partial results.
- Preserve Arborium captures underneath semantic overlays.
- Provide a user setting to enable, disable, or defer to the theme.

Acceptance criteria:

- Semantic tokens render for at least one supported language.
- Syntax-only fallback remains correct when the LSP is unavailable.
- Stale token responses cannot overwrite newer document state.
- Performance remains acceptable for large files.
- Manual review covers theme contrast and agent-opened files.

## Pull request rules

- Keep one grammar or one editor architecture concern per PR.
- Include fixtures and focused tests with every language addition.
- Do not combine semantic highlighting or the general embedded-language
  architecture with a basic extension mapping.
- Keep CSS and Handlebars as separate PRs even when both are embedded in MJML.
- Keep the roadmap and task status updated when a PR merges or is superseded.
- Run the smallest relevant tests first, then format and perform the required
  build/GUI verification before opening the PR.
