//! File type detection utilities for determining if files can be opened in Warp.

use std::path::Path;

use serde::{Deserialize, Serialize};
use warp_core::features::FeatureFlag;
pub use warp_util::file_type::{
    is_binary_file, is_file_content_binary, is_jupyter_notebook_file, is_known_binary_extension,
    is_markdown_file,
};

#[cfg(feature = "local_fs")]
use crate::util::file::external_editor::{Editor, EditorSettings, settings::EditorChoice};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Layout used when opening files in the editor.",
    rename_all = "snake_case"
)]
pub enum EditorLayout {
    SplitPane,
    NewTab,
}

/// The type of file that can be opened in Warp. The in-product treatment for "opening" a file
/// depends on its type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenableFileType {
    /// A Markdown file, which should be opened in a Markdown viewer pane.
    Markdown,
    /// A code file, which should be opened in a code editor pane.
    Code,
    /// Other types of text files, e.g. txt, csv, svg files, which can still be opened in a code editor pane.
    Text,
}

/// The target application or viewer to use when opening a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTarget {
    /// Open in Warp's Markdown viewer.
    MarkdownViewer(EditorLayout),
    /// Open in Warp's Code Editor.
    CodeEditor(EditorLayout),
    /// Open in an external editor (e.g. VS Code, Emacs).
    #[cfg(feature = "local_fs")]
    ExternalEditor(Editor),
    /// Open in the environment editor ($EDITOR).
    EnvEditor,
    /// Open in the system default application.
    SystemDefault,
    /// Open in the system default application (generic open, e.g. for binary files).
    SystemGeneric,
}

/// Named options for resolving the surface used to open a file.
///
/// Keeping these values together prevents new editor behavior from expanding
/// the resolver's positional argument list.
#[cfg(feature = "local_fs")]
pub struct FileOpenOptions<'a> {
    pub editor_choice: EditorChoice,
    pub prefer_markdown_viewer: bool,
    pub default_layout: EditorLayout,
    pub layout: Option<EditorLayout>,
    pub additional_text_file_patterns: &'a [String],
}

/// Checks if a file is a code file with language support.
#[cfg(feature = "local_fs")]
pub fn is_supported_code_file(path: impl AsRef<Path>) -> bool {
    languages::language_by_local_filename(path.as_ref()).is_some()
}

#[cfg(not(feature = "local_fs"))]
pub fn is_supported_code_file(_path: impl AsRef<Path>) -> bool {
    false
}

/// Whether `path` renders in Warp's notebook viewer (with a Rendered/Raw
/// toggle): Markdown files always, Jupyter notebooks when the feature flag is
/// enabled.
pub fn renders_in_warp_notebook_viewer(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    is_markdown_file(path)
        || (FeatureFlag::JupyterNotebookRendering.is_enabled() && is_jupyter_notebook_file(path))
}

pub fn is_supported_image_file(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg"
            )
        })
        .unwrap_or(false)
}

/// Returns true if `path` looks like a shell script the user intends to run when
/// "Open with Warp" is invoked from Finder/another app via a `file://` URL.
///
/// Policy: extension in {sh, bash, zsh, fish, ksh} with the user-execute bit set on Unix,
/// or extension in {ps1, bat, cmd} on Windows (no x-bit concept). On Unix, files with no
/// extension but a `#!` shebang and the user-execute bit set also qualify.
///
/// Narrow on purpose: this only affects the URI entry point, not "Open in New Tab" from
/// other UI surfaces, which still want shell scripts viewable in the editor.
/// Returns true if `path` exists and starts with a `#!` shebang. Reads only the
/// first two bytes — the URI entry point is reached from a `file://` URL, so the
/// file is attacker-controlled in size and `std::fs::read` would risk an OOM.
pub(crate) fn starts_with_shebang(path: &Path) -> bool {
    use std::io::Read;
    let mut prefix = [0u8; 2];
    match std::fs::File::open(path) {
        Ok(mut file) => file.read_exact(&mut prefix).is_ok() && prefix == [b'#', b'!'],
        Err(_) => false,
    }
}

#[cfg(unix)]
pub fn is_runnable_shell_script(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    // Match the documented routing policy: only the owner's execute bit counts.
    // A file `chmod 070` belongs to a group, not to the user invoking Warp.
    let has_user_x_bit = std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o100 != 0)
        .unwrap_or(false);
    if !has_user_x_bit {
        return false;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    if let Some(ext) = ext.as_deref() {
        return matches!(ext, "sh" | "bash" | "zsh" | "fish" | "ksh" | "command");
    }
    starts_with_shebang(path)
}

#[cfg(windows)]
pub fn is_runnable_shell_script(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .is_some_and(|ext| matches!(ext.as_str(), "ps1" | "bat" | "cmd"))
}

#[cfg(not(any(unix, windows)))]
pub fn is_runnable_shell_script(_path: &Path) -> bool {
    false
}

/// Determines if a file can be opened in Warp and returns its type.
/// Returns `None` if the file is binary and should not be opened.
pub fn is_file_openable_in_warp(path: &Path) -> Option<OpenableFileType> {
    is_file_openable_in_warp_with_patterns(path, &[])
}

/// Determines if a file can be opened in Warp, including user-configured
/// names and simple glob patterns that should be treated as text.
pub fn is_file_openable_in_warp_with_patterns(
    path: &Path,
    additional_text_file_patterns: &[String],
) -> Option<OpenableFileType> {
    let matches_configured_pattern =
        matches_additional_text_file_pattern(path, additional_text_file_patterns);
    if matches_configured_pattern && !is_known_binary_extension(path) && !path.exists() {
        return Some(OpenableFileType::Text);
    }

    if is_binary_file(path) {
        return None;
    }

    if is_markdown_file(path) {
        Some(OpenableFileType::Markdown)
    } else if is_supported_code_file(path) {
        Some(OpenableFileType::Code)
    } else {
        // We allow opening the file, even if we don't have particular syntax highlighting support
        // for it e.g. txt files.
        Some(OpenableFileType::Text)
    }
}

fn matches_additional_text_file_pattern(path: &Path, patterns: &[String]) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let file_name = file_name.to_ascii_lowercase();

    patterns.iter().any(|pattern| {
        let pattern = pattern.trim().to_ascii_lowercase();
        !pattern.is_empty() && simple_glob_matches(&file_name, &pattern)
    })
}

fn simple_glob_matches(value: &str, pattern: &str) -> bool {
    let Some((prefix, suffix)) = pattern.split_once('*') else {
        return value == pattern;
    };
    value.starts_with(prefix)
        && value.ends_with(suffix)
        && value.len() >= prefix.len() + suffix.len()
}

/// Only use this for UI elements that must explicitly open a file in Warp (i.e. "Open in New Tab").
/// Prefer `resolve_file_target` for all other cases to respect users' preferences.
/// This would also force any binary file to be opened in Warp's Code Editor, so you should likely check
/// `is_file_openable_in_warp` before rendering any such UI Elements.
#[cfg(feature = "local_fs")]
pub fn resolve_file_target_to_open_in_warp(
    path: &Path,
    settings: &EditorSettings,
    layout: Option<EditorLayout>,
) -> FileTarget {
    let openable_file_type =
        is_file_openable_in_warp_with_patterns(path, &settings.additional_text_file_patterns);
    let is_markdown = matches!(openable_file_type, Some(OpenableFileType::Markdown));
    let layout = layout.unwrap_or(*settings.open_file_layout);

    // Jupyter notebooks render in Warp's notebook viewer unconditionally when
    // the feature flag is enabled (the whole point is to avoid raw JSON).
    if openable_file_type.is_some()
        && FeatureFlag::JupyterNotebookRendering.is_enabled()
        && is_jupyter_notebook_file(path)
    {
        return FileTarget::MarkdownViewer(layout);
    }
    if is_markdown && *settings.prefer_markdown_viewer {
        return FileTarget::MarkdownViewer(layout);
    }
    FileTarget::CodeEditor(layout)
}

/// Resolves the target application or viewer for opening a file based on its path and editor settings.
#[cfg(feature = "local_fs")]
pub fn resolve_file_target(
    path: &Path,
    settings: &EditorSettings,
    layout: Option<EditorLayout>,
) -> FileTarget {
    resolve_file_target_with_options(
        path,
        FileOpenOptions {
            editor_choice: *settings.open_file_editor,
            prefer_markdown_viewer: *settings.prefer_markdown_viewer,
            default_layout: *settings.open_file_layout,
            layout,
            additional_text_file_patterns: &settings.additional_text_file_patterns,
        },
    )
}

#[cfg(feature = "local_fs")]
pub fn resolve_file_target_with_editor_choice(
    path: &Path,
    editor_choice: EditorChoice,
    prefer_markdown_viewer: bool,
    default_layout: EditorLayout,
    layout: Option<EditorLayout>,
) -> FileTarget {
    resolve_file_target_with_options(
        path,
        FileOpenOptions {
            editor_choice,
            prefer_markdown_viewer,
            default_layout,
            layout,
            additional_text_file_patterns: &[],
        },
    )
}

#[cfg(feature = "local_fs")]
pub fn resolve_file_target_with_options(path: &Path, options: FileOpenOptions<'_>) -> FileTarget {
    let openable_file_type =
        is_file_openable_in_warp_with_patterns(path, options.additional_text_file_patterns);
    let is_markdown = matches!(openable_file_type, Some(OpenableFileType::Markdown));
    let layout = options.layout.unwrap_or(options.default_layout);
    let is_openable_in_warp = openable_file_type.is_some();

    // 0. Jupyter notebooks render in Warp's notebook viewer unconditionally
    // when the feature flag is enabled (not gated on `prefer_markdown_viewer`
    // or `editor_choice`, since rendering-instead-of-JSON is the whole point).
    if is_openable_in_warp
        && FeatureFlag::JupyterNotebookRendering.is_enabled()
        && is_jupyter_notebook_file(path)
    {
        return FileTarget::MarkdownViewer(layout);
    }

    // 1. Markdown Viewer (only if user preference specified)
    if is_markdown && options.prefer_markdown_viewer {
        return FileTarget::MarkdownViewer(layout);
    }

    // Unknown but readable text should have a useful in-Warp fallback when
    // the user has not explicitly selected an external editor. This keeps
    // dotfiles and extensionless UTF-8 files editable without changing the
    // explicit ExternalEditor, EnvEditor, or Warp choices below.
    if matches!(openable_file_type, Some(OpenableFileType::Text))
        && matches!(options.editor_choice, EditorChoice::SystemDefault)
    {
        return FileTarget::CodeEditor(layout);
    }

    // 2. Warp Code Editor (Explicit user preference)
    if is_openable_in_warp && matches!(options.editor_choice, EditorChoice::Warp) {
        return FileTarget::CodeEditor(layout);
    }

    // 3. Env Editor
    if matches!(options.editor_choice, EditorChoice::EnvEditor) {
        return FileTarget::EnvEditor;
    }

    // 4. Binary files -> System Default
    if !is_openable_in_warp {
        return FileTarget::SystemGeneric;
    }

    // 5. External Editor or System Default (for text files)
    match options.editor_choice {
        EditorChoice::ExternalEditor(editor) => FileTarget::ExternalEditor(editor),
        EditorChoice::SystemDefault => FileTarget::SystemDefault,
        EditorChoice::Warp | EditorChoice::EnvEditor => unreachable!("Already matched above"),
    }
}

#[cfg(test)]
#[path = "openable_file_type_tests.rs"]
mod tests;
