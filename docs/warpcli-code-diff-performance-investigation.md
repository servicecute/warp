# WarpCLI Code-Diff Performance Investigation

Date: 2026-11-17

## Summary

Large `RequestFileEdits` changes can take a long time to appear in the WarpCLI/TUI code-diff panel because the display path performs several full-file operations before the diff is usable. The main issue is that the TUI receives already-resolved file diffs, then reconstructs editor buffers and recomputes the diffs for presentation.

No implementation changes were made as part of this investigation.

## Execution path

1. `RequestFileEditsExecutor` applies the requested edits and creates resolved `FileDiff` values in `app/src/ai/blocklist/action_model/execute/request_file_edits.rs:214-336`.
2. `TuiDiffStorage` receives and stores the resolved diffs in `crates/warp_tui/src/tui_diff_storage.rs:231-315`.
3. `TuiFileEditsView::rebuild_sections` creates one `CodeEditorModel` per file in `crates/warp_tui/src/tui_file_edits_view.rs:358-401`.
4. Each editor loads the complete base content, applies the deltas, enables hidden-line calculation, and expands diff navigation.
5. The editor asynchronously runs another full-text diff in `app/src/code/editor/diff.rs:587-700`.
6. Once the diff is available, hidden ranges and removed-line ghost blocks are calculated, then the TUI lays out the display rows.

## Findings

### 1. The TUI recomputes a diff it already has

The executor has already applied the requested edits and produced `FileDiff` values. The TUI does not directly render those resolved ranges. Instead, it resets a fresh editor with the base content, applies the deltas, and asks `CodeEditorModel` to calculate a new diff:

- `crates/warp_tui/src/tui_file_edits_view.rs:364-379`
- `app/src/code/editor/model.rs:1075-1137`
- `app/src/code/editor/diff.rs:587-700`

For a large file, this means the panel pays for another full comparison of the old and new contents. The diff algorithm uses `similar::Algorithm::Patience`, which runs asynchronously, but the panel waits for each editor's `DiffUpdated` event before showing complete counts and diff state.

### 2. Full editor buffers are created eagerly for every file

`rebuild_sections` creates one complete `CodeEditorModel` per stored file and loads the entire base content before the sections are available. The storage also clones the complete `Vec<FileDiff>` when rebuilding:

- `crates/warp_tui/src/tui_file_edits_view.rs:360-371`
- `crates/warp_tui/src/tui_file_edits_view.rs:362`

This cost scales with total file size and number of files, not only with the number of changed lines.

### 3. Blocked confirmation cards expand every section

When the action is waiting for confirmation, `rebuild_sections` calls `expand_all_sections`. The normal collapsed-section optimization therefore does not protect the initial confirmation view:

- `crates/warp_tui/src/tui_file_edits_view.rs:402-410`

The body itself is built lazily by `render_section`, but every file is marked expanded in the blocked state, so large changes can cause all file bodies to be built together.

### 4. TUI layout is row-oriented rather than viewport-virtualized

The editor element creates a display lattice and iterates through every visible display row. Without a viewport limit, it creates a `TuiText` child for each row:

- `crates/warp_tui/src/editor_element.rs:417-483`
- `crates/warp_tui/src/editor_element.rs:748-765`

The display lattice scans the logical file lines, hidden ranges, and ghost rows:

- `crates/editor/src/render/model/char_cell_display.rs:309-385`

This is especially expensive when `expand_diffs` materializes many removed-line ghost rows.

### 5. Hidden-line calculation scans the complete buffer

After the diff finishes, the editor derives visible hunk ranges and computes their complement across the full line count:

- `app/src/code/editor/model.rs:1462-1493`

This is useful for compact diffs, but it adds another whole-file pass for large files.

## Likely user-visible sequence

```text
resolved edits
    -> create full editor for every file
    -> apply full-file content changes
    -> recompute full diff asynchronously
    -> calculate hidden ranges and ghost rows
    -> expand confirmation sections
    -> build/layout all diff rows
    -> panel becomes responsive
```

## Recommended follow-up order

1. Add timing instrumentation around `rebuild_sections`, `apply_diffs`, each `DiffUpdated` event, hidden-line calculation, and TUI editor layout. This will distinguish diff-algorithm time from buffer/layout time.
2. Reuse the resolved `FileDiff` metadata for TUI display instead of recomputing an independent editor diff where possible.
3. Defer full editor construction until a file section is opened, while showing file names and line counts immediately.
4. Avoid expanding every file by default for very large changes; expand only the selected file or a bounded context window.
5. Add viewport-based row virtualization or a large-diff rendering cap so layout work is proportional to the visible terminal area.
6. Add a characterization test or benchmark covering a large single-file replacement and a multi-file large edit.

## Scope note

This investigation focused on the WarpCLI/TUI path under `crates/warp_tui`. The GUI `CodeDiffView` follows a separate editor/view path and should be profiled independently before applying the same conclusions there.
