# Internal Warp Roadmap

This document tracks editor and terminal improvements curated for the
ServiceCute Warp fork. It is an internal planning document, not a promise of
upstream Warp support.

## Editor language support

### Completed or in progress

- Gleam syntax highlighting through Arborium/Tree-sitter.
- Configurable Gleam codegen annotation highlighting in documentation comments.
- Dart syntax highlighting through Arborium's standard Dart grammar and
  highlight query.
- Shared user language configuration for extension aliases, custom scopes, and
  color overrides.

### Erlang syntax highlighting

Status: planned.

Warp currently treats `.erl` and `.hrl` as development text files, but the
language registry does not map them to an Erlang parser or highlight query.
That means Erlang files open as text without structural syntax highlighting.

The implementation should:

1. Select a maintained Tree-sitter Erlang grammar that is compatible with the
   parser version used by Arborium.
2. Prefer an Arborium language package when available; otherwise add a narrowly
   scoped grammar dependency and document its license and maintenance source.
3. Register `erlang` in the language registry and map both `.erl` and `.hrl`.
4. Add Erlang configuration for comments, indentation, and bracket pairs.
5. Use the grammar's standard highlight query for atoms, variables, modules,
   functions, records, macros, strings, numbers, keywords, operators, and
   comments.
6. Add representative fixtures for modules, behaviours, records, macros,
   guards, binary syntax, pattern matching, receive expressions, and Erlang
   header files.
7. Add parser, filename-resolution, and rendered-capture tests before enabling
   the feature in the curated release branch.

The support should cover syntax highlighting only. LSP, compilation, code
navigation, formatting, and semantic analysis remain separate roadmap items.

### SurrealQL syntax highlighting

Status: planned.

Warp should recognize `.surql` files as SurrealQL rather than treating them as
generic SQL or plain text. This is especially useful for the workspace's
SurrealDB schema, query, migration, and test files.

The implementation should:

1. Select a maintained SurrealQL grammar or the closest compatible
   Tree-sitter grammar supported by the Arborium parser version.
2. Register `surrealql` in the language registry and map the `.surql`
   extension.
3. Add configuration for `--` and block comments, indentation, and SQL-style
   bracket pairs.
4. Highlight SurrealQL statements, record IDs, tables, fields, parameters,
   functions, literals, operators, graph relations, permissions, and comments.
5. Add fixtures covering `DEFINE`, `SELECT`, `RELATE`, record links, nested
   objects, arrays, parameters, transactions, and schema/migration syntax.
6. Add parser, filename-resolution, and rendered-capture tests before release
   branch integration.

The initial scope is syntax highlighting only. SurrealDB tooling, query
validation, formatting, and schema-aware completion remain separate items.

### MJML syntax highlighting

Status: planned.

Warp should recognize `.mjml` files and provide structural highlighting for
MJML email templates while preserving the embedded HTML and CSS portions.

The implementation should:

1. Determine whether the existing HTML grammar can support MJML tags and
   attributes through an MJML-specific grammar or a small compatible extension.
2. Register `mjml` in the language registry and map the `.mjml` extension.
3. Highlight MJML components, attributes, expressions, HTML elements, CSS,
   comments, strings, and embedded template values without treating custom
   component names as errors.
4. Add fixtures covering `mjml`, `mj-head`, `mj-body`, sections, columns,
   buttons, images, attributes, embedded styles, and conditional comments.
5. Add parser, filename-resolution, nested-language, and rendered-capture
   tests before release branch integration.

The initial scope is syntax highlighting only. MJML compilation, preview,
email-client compatibility checks, and formatter support remain separate
roadmap items.

## Internal editor fallback for text files

Status: planned.

Files that are not registered in Warp's language registry are currently at
risk of being delegated to the operating-system editor. This is surprising
for repository text files such as `.gitmodules`, `.dockerignore`,
`.agent-session-identity`, extensionless files, and other dotfiles: unknown
language should not be treated as unknown file content.

The opening policy should separate content detection from language detection:

```text
Known binary or explicitly external-only file -> operating-system handler
UTF-8 text file, including unknown/dotfile name -> Warp internal editor
Known language -> Warp internal editor with that language's highlighting
```

The implementation should:

1. Add a centralized `OpenFileIntent`/file classification path used by file
   search, repository browsing, agent actions, drag-and-drop, and restored
   editor tabs.
2. Resolve language by filename and extension when known, but fall back to a
   plain-text internal editor for unknown names rather than delegating to the
   OS.
3. Recognize common repository dotfiles and extensionless files as text,
   including `.gitmodules`, `.dockerignore`, `.editorconfig`, `.env*`,
   `.npmrc`, and agent/project metadata files.
4. Use conservative content sniffing for unrecognized files: valid UTF-8 with
   no NUL bytes should default to text; binary or undecodable content should
   remain eligible for an external application.
5. Preserve an explicit `Open in default application` action for users who
   want the OS editor, without making it the implicit fallback.
6. Keep the detected language and editor choice independent so a plain-text
   file can later gain highlighting without changing its open behavior.
7. Add tests for known source files, dotfiles, extensionless UTF-8 files,
   binary files, files with misleading extensions, and explicit external-open
   actions.

This should also ensure that opening a text file from an agent response or
repository context uses the same routing policy as opening it from the file
picker. The initial fallback can use plain-text highlighting; adding parsers
for individual configuration formats is a separate language-support task.

## Language-aware editor actions

Status: planned.

Warp currently supports global editor selection, keyboard shortcuts, and
workflows, but it does not expose VS Code-style actions that are selected from
the active file's language. Users should be able to discover and run common
language-specific actions from the editor context menu and command palette.

The action model should support project-local configuration, for example:

```text
Language: Gleam
├── Format document -> gleam format
├── Check project   -> gleam check
└── Run tests       -> gleam test

Language: Dart
├── Format document -> dart format
├── Analyze         -> dart analyze
└── Run tests       -> dart test
```

The implementation should:

1. Derive the active language from the same centralized file classification
   used for syntax highlighting and internal-editor routing.
2. Define a small action contract containing a display name, command, scope,
   availability check, working-directory policy, and optional arguments.
3. Support built-in safe defaults for common languages while allowing
   repository-local overrides through a versionable configuration file or
   `.warp/workflows`.
4. Surface actions in the editor context menu, command palette, and an
   optional editor/footer action area without making the footer the only entry
   point.
5. Run actions in the correct project root and clearly show the command,
   output, exit status, and affected file or workspace.
6. Detect missing tools before execution and provide an actionable message
   rather than silently failing.
7. Keep actions explicit: opening a file or selecting a language must not
   automatically run formatters, tests, analyzers, or other commands.
8. Add tests for language resolution, project-local overrides, missing tools,
   working-directory selection, command discovery, and action invocation.

The first useful action sets are Gleam (`gleam format`, `gleam check`,
`gleam test`) and Dart (`dart format`, `dart analyze`, `dart test`). Future
sets can cover SurrealQL, MJML, Rust, TypeScript, and other registered
languages. This feature should complement workflows rather than replace them:
workflows remain the escape hatch for custom project commands.

## Secondary and future proposals

These proposals extend Warp from a terminal with an embedded editor into a
terminal-first agentic development environment. They should follow the core
language registry, internal-editor routing, and language-aware action work.

### Semantic highlighting

Status: future.

Extend syntax highlighting with semantic tokens from language servers so Warp
can distinguish types, functions, parameters, fields, and variables based on
program meaning rather than grammar position alone. Semantic highlighting
should remain optional and fall back cleanly to Arborium syntax captures when
no language server or semantic token provider is available.

### Structural navigation

Status: future.

Expose the language structure already represented by parser symbol queries and
bracket metadata through:

- symbol outline;
- go to definition;
- find references;
- breadcrumbs;
- code folding;
- sticky scope headers; and
- jump to matching bracket.

These features should work from the editor, command palette, and agent context
so a human or agent can navigate large files using the same structural model.

### Inline diagnostics and code actions

Status: future.

Connect language-server diagnostics and language-aware actions to an inline
editor surface:

```text
Error at line 42
├── Explain
├── Fix with agent
├── Apply quick fix
├── Run formatter
└── Open documentation
```

Agent-generated fixes must be presented as reviewable edits or diffs, require
explicit approval before mutation, and preserve the original diagnostic and
tool output.

### Embedded-language highlighting

Status: future.

Support nested language regions instead of treating each file as one flat
language. Initial targets include:

- HTML with CSS and JavaScript;
- MJML with HTML and CSS;
- Markdown with fenced Gleam or other code blocks;
- Dockerfiles with shell fragments; and
- SQL or SurrealQL embedded in application strings.

Embedded-language ownership must be explicit so a child grammar cannot corrupt
the parent grammar's comments, strings, or delimiter highlighting.

### Agent-aware diff and merge review

Status: future.

Provide editor-native review for human and agent changes:

- explain this change;
- accept or reject a hunk;
- show the agent-generated diff;
- compare against the repository base;
- resolve conflicts with agent assistance; and
- identify and run affected tests.

The initial implementation can reuse the existing Git diff model. A later
three-way merge view should distinguish base, current, incoming, and resolved
content and require review before staging or committing changes.

### Workspace task graph

Status: future.

Expand language-aware actions into a project task model with:

- task dependencies;
- problem matchers;
- task status and cancellation;
- affected workspace or package detection;
- rerun-failed-task support; and
- agent-readable output and verification evidence.

This should complement `.warp/workflows` rather than replace them. Workflows
remain the escape hatch for custom commands, while the task graph provides
structured execution for repository checks and language tooling.

### Lower-priority editor affordances

Status: future.

After structural and agent-oriented capabilities are stable, consider:

- minimap and code overview;
- file and symbol breadcrumbs;
- sticky scroll;
- customizable editor toolbars;
- multi-root repository views; and
- worktree-aware source-control navigation.

The priority is deliberate: these visual conveniences should follow
diagnostics, navigation, embedded languages, review, and task verification.

## Agent activity and resource control center

Status: future proposal.

Warp should provide one unified control center for agent-native work instead
of scattering process, tool, context, and verification state across unrelated
panels. The control center should make the active agent's work observable,
interruptible, and reviewable without requiring the user to inspect raw logs.

Recommended views:

```text
Agent Control Center
├── Running
│   ├── agent turns
│   ├── shell commands
│   ├── MCP calls
│   └── language-server processes
├── Waiting
│   ├── user approval
│   ├── authentication
│   └── failed/retryable action
├── Context
└── Resources
```

The first version should expose:

- agent runs, execution profiles, models, current turns, elapsed time,
  cancellation, and waiting-for-user state;
- PTY and process trees with PID, command, working directory, foreground TUI,
  CPU/RAM usage, exit status, stop, and restart controls;
- MCP servers, tool/resource calls, latency, failures, approval state, and
  cancellation;
- attached files, skills, registry documents, indexed repositories, and
  context freshness;
- tests, formatters, analyzers, diagnostics, logs, artifacts, and affected
  packages; and
- memory, swap, disk, simulator, build-process, and reclaimable-cache state.

The control center should reuse existing lifecycle and context models instead
of creating a second process or agent state system. It should also provide a
live event timeline so the user can answer: what is running, what is blocked,
what changed, what failed, and what requires my decision?

### Configurable resource providers

Status: future proposal.

Resource visibility should be implemented through a provider-neutral contract
rather than hard-coding one cleanup utility into the UI. A provider should
declare:

- an identifier and display name;
- executable and version detection;
- read-only report commands;
- structured output schema and refresh interval;
- supported action IDs and argument schemas; and
- permission class for every action: read-only, reversible, or destructive.

The first provider should be `reclaim`, the local macOS resource-reclamation
CLI. It already supports read-only reports, machine-readable `--json` output,
explicit action flags, and actionable candidates for memory and storage.

The initial UI could display:

```text
Resources
└── Reclaim
    ├── reclaimable memory
    ├── reclaimable storage
    ├── forgotten development binaries
    └── headless simulators
```

The provider configuration should be user- and project-configurable without
requiring a source-code change, for example:

```toml
[resource_providers.reclaim]
enabled = true
command = "reclaim"
memory_report = ["memory", "--json"]
storage_report = ["storage", "--json"]
poll_interval_seconds = 30
```

The integration must preserve `reclaim`'s safety boundary:

- polling `reclaim memory --json` and `reclaim storage --json` is read-only;
- storage actions require explicit user confirmation;
- memory actions may be configured as autonomous because they are reversible;
- the UI shows the target, estimated impact, permission level, and result; and
- no action runs solely because a candidate appeared in a report.

Warp must not execute an arbitrary JSON `action` string directly. The provider
adapter should map stable action IDs to fixed argument arrays, validate the
candidate identity, and display the final command before execution. This
prevents a report payload from becoming an unreviewed shell-execution path.

`reclaim` should be the first implementation of a general `ResourceProvider`
interface. Future providers could expose Docker, Xcode, package-manager, CI,
or project-specific resource state while sharing the same panel, polling,
approval, audit, and error-handling behavior.

## Release workflow

- `servicecute/release` is the curated integration branch for all personal
  Warp improvements.
- Feature branches remain focused and independently reviewable.
- Editor language additions should include source fixtures, focused tests, and
  a short implementation note before merging into the release branch.
- The installed stable Warp application is separate from this fork and does
  not consume these changes automatically.

## Full-screen TUI pane management

Status: UX investigation.

Full-screen TUIs can take ownership of keyboard input, which currently makes
workspace actions such as splitting the active terminal difficult to discover
or invoke. Splitting is a pane-level action and should remain available even
when the child process owns the terminal viewport.

Preferred direction:

- Keep split actions in persistent pane or tab chrome that is outside the PTY.
- Keep the action in the pane context menu and command palette.
- Preserve an application-level split shortcut that is intercepted before
  input is forwarded to a full-screen TUI.
- Treat the footer agent toolbelt as an additional discoverability surface,
  not the only place where split terminal actions live.

The action should clearly distinguish splitting the current pane from opening a
new tab, preserve the running TUI in the original pane, and start a shell in
the new pane without sending input to the TUI.

### TUI input ownership and collision policy

Full-screen applications such as Codex, OpenCode, and Grok may assign meaning
to common terminal shortcuts, control sequences, escape sequences, and mouse
events. Warp must not silently steal those inputs.

The routing rule is:

```text
Explicit Warp UI action -> Warp pane command
Everything typed inside the TUI -> TUI owns it
```

The implementation should therefore:

- prioritize pane chrome, pane context menus, and other explicit Warp UI
  controls because they are outside the TUI viewport;
- expose the same actions through a generic `Pane Actions` surface in the
  footer/toolbelt, rather than hiding them inside an agent-specific menu;
- reserve only an explicit platform-level modifier shortcut for splitting;
- avoid intercepting unmodified keys, common `Ctrl`/`Alt`/`Esc` combinations,
  terminal escape sequences, or ordinary mouse events inside the TUI viewport;
- route command-palette actions through the same pane command API; and
- test input routing with Codex, OpenCode, Grok, and representative full-screen
  TUIs to verify their controls remain intact.

The UI-based path is the primary interaction. Keyboard shortcuts are optional
accelerators and must never be the only way to split a TUI-owned terminal.

## Agent context visibility

Status: UX investigation.

The agent toolbelt currently focuses on interaction controls such as rich input,
file exploration, code review, and session sharing. Warp can discover and load
MCP servers and skills, but that context is not presented as a complete live
inventory in the toolbelt.

The toolbelt should provide a compact `Context` or `Resources` panel with
status and management links for:

```text
Agent context
├── MCP
│   ├── Connected servers
│   ├── Available tools
│   └── Available resources
├── Skills
│   ├── Project skills
│   ├── Global skills
│   └── Bundled skills
└── Repository context
    ├── Files and directories
    └── Indexed project metadata
```

The panel should reflect the active agent session and distinguish connected,
discovered, unavailable, and failed MCP servers. Skills should show their
source path or registry origin, including project and global providers such as
`.agents/skills`, `.warp/skills`, `.claude/skills`, and `.codex/skills`.

This should be a popover or side panel rather than a long footer list. The
footer remains a discoverability entry point, while detailed tool, resource,
and skill inventories belong in the expanded context view.

### Local registry integration

The first registry integration target is the local registry workflow at
`/Users/omofolarin/Documents/personal-dev/registry/`. This is separate from a
remote MCP marketplace or a skills directory. It is curated, human-authored
reference context for architecture decisions, design patterns, service
documentation, testing guides, and workspace conventions.

Warp should expose it as a read-only `Local registry` provider:

```text
Agent context
├── Active MCP       -> callable tools and resources
├── Skills           -> executable agent capabilities
├── Repository       -> current project metadata
└── Local registry   -> curated reference documents
```

The provider should:

- discover the registry root from configuration such as
  `WARP_CONTEXT_REGISTRY` rather than hard-coding a personal path;
- index Markdown and document metadata, paths, categories, and freshness;
- search or suggest relevant documents using the active repository, language,
  and task context;
- let the user explicitly attach selected registry documents to an agent
  session;
- show each document's exact local source path and freshness status; and
- keep registry content read-only and separate from executable skills and MCP
  servers.

Warp should not inject the entire registry into every prompt or automatically
activate anything discovered there. The registry is reference context, not an
execution authority. Remote catalogs can be considered later, after the local
registry provider has a clear contract.
