use std::path::Path;

use arborium::tree_sitter::StreamingIterator;
use warp_util::standardized_path::StandardizedPath;

use crate::{SUPPORTED_LANGUAGES, language_by_filename, language_by_local_filename, load_language};

/// Validate that every supported language can be loaded successfully.
/// This catches invalid node types, syntax errors, and other issues in .scm query files
/// (highlights, indents, identifiers) that would otherwise only surface at runtime.
#[test]
fn all_supported_languages_load_successfully() {
    let failures: Vec<_> = SUPPORTED_LANGUAGES
        .iter()
        .filter(|lang| load_language(lang).is_none())
        .collect();

    assert!(
        failures.is_empty(),
        "The following languages failed to load:\n{}",
        failures
            .iter()
            .map(|lang| format!("  - {lang}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Both `.html` and the legacy three-character `.htm` extension should resolve to
/// the same HTML language entry. `.htm` is widely produced by static-site generators
/// and historical web tooling (DOS 8.3 filename limits) and is already treated as
/// an HTML/text file elsewhere in the codebase
/// (see `is_development_text_extension` in `crates/warp_util/src/file_type.rs`).
#[test]
fn html_extensions_resolve_to_html() {
    for filename in ["index.html", "index.htm"] {
        let path = StandardizedPath::try_new(&format!("/tmp/{filename}"))
            .expect("test path should be absolute");
        let language = language_by_filename(&path)
            .unwrap_or_else(|| panic!("expected {filename} to resolve to a language"));
        assert_eq!(
            language.display_name(),
            "HTML",
            "{filename} should resolve to HTML",
        );
    }
}

#[test]
fn local_html_extensions_resolve_to_html() {
    for filename in ["index.html", "index.htm"] {
        let path = Path::new(filename);
        let language = language_by_local_filename(path)
            .unwrap_or_else(|| panic!("expected {filename} to resolve to a language"));
        assert_eq!(
            language.display_name(),
            "HTML",
            "{filename} should resolve to HTML",
        );
    }
}

#[test]
fn mjml_extension_resolves_to_mjml() {
    let path = StandardizedPath::try_new("/tmp/email.mjml").expect("test path should be absolute");
    let language = language_by_filename(&path).expect("`.mjml` files should resolve to MJML");
    assert_eq!(language.display_name(), "MJML");
}

#[test]
fn mjml_standard_fixture_loads() {
    let language = load_language("mjml").expect("MJML should load");
    let fixture = include_str!("../grammars/mjml/standard_fixture.mjml");
    let mut parser = arborium::tree_sitter::Parser::new();
    parser
        .set_language(&language.grammar)
        .expect("MJML should use a compatible HTML grammar");
    let tree = parser
        .parse(fixture, None)
        .expect("MJML fixture should parse");

    assert!(!tree.root_node().has_error());
}

#[test]
fn mjml_standard_fixture_has_html_highlight_captures() {
    let language = load_language("mjml").expect("MJML should load");
    let fixture = include_str!("../grammars/mjml/standard_fixture.mjml");
    let mut parser = arborium::tree_sitter::Parser::new();
    parser
        .set_language(&language.grammar)
        .expect("MJML should use a compatible HTML grammar");
    let tree = parser
        .parse(fixture, None)
        .expect("MJML fixture should parse");

    let capture_names = language.highlight_query.capture_names();
    assert!(
        capture_names.iter().any(|name| *name == "tag"),
        "the HTML highlight query should expose tag captures"
    );
    assert!(
        capture_names.iter().any(|name| *name == "string"),
        "the HTML highlight query should expose string captures"
    );

    let mut cursor = arborium::tree_sitter::QueryCursor::new();
    let mut captures = cursor.captures(
        &language.highlight_query,
        tree.root_node(),
        fixture.as_bytes(),
    );
    let mut capture_count = 0;
    while let Some((query_match, _)) = captures.next() {
        capture_count += query_match.captures.len();
    }

    assert!(
        capture_count > 0,
        "MJML should produce visible highlight captures"
    );
}

#[test]
fn handlebars_extensions_resolve_to_handlebars() {
    for filename in ["email.hbs", "email.handlebars"] {
        let path = StandardizedPath::try_new(&format!("/tmp/{filename}"))
            .expect("test path should be absolute");
        let language = language_by_filename(&path)
            .unwrap_or_else(|| panic!("expected {filename} to resolve to Handlebars"));
        assert_eq!(language.display_name(), "Handlebars");
    }
}

#[test]
fn json_handlebars_extensions_resolve_to_json_handlebars() {
    for filename in ["email.json.hbs", "email.json.handlebars"] {
        let path = StandardizedPath::try_new(&format!("/tmp/{filename}"))
            .expect("test path should be absolute");
        let language = language_by_filename(&path)
            .unwrap_or_else(|| panic!("expected {filename} to resolve to JSON Handlebars"));
        assert_eq!(language.display_name(), "Handlebars JSON");
    }
}

#[test]
fn handlebars_fixtures_load_and_parse() {
    for (language_name, fixture_path) in [
        ("handlebars", "../grammars/handlebars/standard_fixture.hbs"),
        (
            "handlebars-json",
            "../grammars/handlebars/standard_fixture.json.hbs",
        ),
    ] {
        let language = load_language(language_name).expect("Handlebars language should load");
        let fixture = include_str!("../grammars/handlebars/standard_fixture.hbs");
        let fixture = if fixture_path.ends_with("json.hbs") {
            include_str!("../grammars/handlebars/standard_fixture.json.hbs")
        } else {
            fixture
        };
        let mut parser = arborium::tree_sitter::Parser::new();
        parser
            .set_language(&language.grammar)
            .expect("Handlebars host grammar should be compatible");
        let tree = parser.parse(fixture, None).expect("fixture should parse");
        assert_eq!(tree.root_node().byte_range(), 0..fixture.len());
    }
}

/// `.command` is the macOS convention for double-clickable shell scripts.
/// Make sure `language_by_filename` recognizes it as shell so the editor
/// renders syntax highlighting instead of the
/// "Language support is unavailable for this file type" footer.
#[test]
fn command_extension_resolves_to_shell() {
    let path =
        StandardizedPath::try_new("/tmp/script.command").expect("test path should be absolute");
    let language =
        language_by_filename(&path).expect("`.command` files should resolve to a language");
    assert_eq!(language.display_name(), "Shell");
}

#[test]
fn local_command_extension_resolves_to_shell() {
    let language = language_by_local_filename(Path::new("script.command"))
        .expect("`.command` files should resolve to a language");
    assert_eq!(language.display_name(), "Shell");
}

/// `.md` and `.markdown` should resolve to the Markdown language so the editor applies
/// syntax highlighting to Markdown source files.
#[test]
fn markdown_extensions_resolve_to_markdown() {
    for filename in ["README.md", "notes.markdown"] {
        let path = StandardizedPath::try_new(&format!("/tmp/{filename}"))
            .expect("test path should be absolute");
        let language = language_by_filename(&path)
            .unwrap_or_else(|| panic!("expected {filename} to resolve to a language"));
        assert_eq!(
            language.display_name(),
            "Markdown",
            "{filename} should resolve to Markdown",
        );
    }
}
