use super::*;

#[test]
fn test_common_text_files() {
    // Programming languages
    assert!(is_text_file("main.rs"));
    assert!(is_text_file("script.py"));
    assert!(is_text_file("app.js"));
    assert!(is_text_file("component.tsx"));
    assert!(is_text_file("Main.java"));
    assert!(is_text_file("header.h"));
    assert!(is_text_file("script.sh"));

    // Web files
    assert!(is_text_file("index.html"));
    assert!(is_text_file("styles.css"));
    assert!(is_text_file("component.vue"));

    // Configuration files
    assert!(is_text_file("config.json"));
    assert!(is_text_file("settings.yaml"));
    assert!(is_text_file("Cargo.toml"));
    assert!(is_text_file(".gitignore"));
    assert!(is_text_file(".gitmodules"));
    assert!(is_text_file(".dockerignore"));
    assert!(is_text_file(".agent-session-identity"));
    assert!(is_text_file(".env"));
    assert!(is_text_file(".env.local"));

    // Documentation
    assert!(is_text_file("README.md"));
    assert!(is_text_file("docs.txt"));
    assert!(is_text_file("manual.rst"));

    // Build files
    assert!(is_text_file("Dockerfile"));
    assert!(is_text_file("Justfile"));
    assert!(is_text_file("Makefile"));
    assert!(is_text_file("build.gradle"));

    // Files without extensions
    assert!(is_text_file("README"));
    assert!(is_text_file("LICENSE"));
    assert!(is_text_file("Dockerfile"));
}

#[test]
fn test_binary_files() {
    // Images
    assert!(!is_text_file("image.png"));
    assert!(!is_text_file("photo.jpg"));
    assert!(!is_text_file("icon.ico"));
    // Note: SVG might be detected as text by MIME, which is correct

    // Executables
    assert!(!is_text_file("program.exe"));
    assert!(!is_text_file("app.dmg"));

    // Archives
    assert!(!is_text_file("archive.zip"));
    assert!(!is_text_file("package.tar.gz"));
    assert!(!is_text_file("data.7z"));

    // Media files
    assert!(!is_text_file("video.mp4"));
    assert!(!is_text_file("audio.mp3"));
    assert!(!is_text_file("sound.wav"));

    // Document formats (binary)
    assert!(!is_text_file("document.pdf"));
    assert!(!is_text_file("spreadsheet.xlsx"));
    assert!(!is_text_file("presentation.pptx"));
}

#[test]
fn test_edge_cases() {
    // Empty filename
    assert!(!is_text_file(""));

    // Files with multiple extensions
    assert!(is_text_file("backup.tar.gz.txt"));
    assert!(is_text_file("config.local.json"));

    // Mixed case
    assert!(is_text_file("Component.TSX"));
    assert!(is_text_file("README.MD"));

    // Path separators
    assert!(is_text_file("/path/to/file.rs"));
    assert!(is_text_file("..\\windows\\path\\file.py"));

    // Unusual but valid text files
    assert!(is_text_file("script.fish"));
    assert!(is_text_file("data.graphql"));
    assert!(is_text_file("schema.proto"));
}

#[test]
fn test_development_extensions() {
    // Test some specific development file types
    assert!(is_development_text_extension("rs"));
    assert!(is_development_text_extension("py"));
    assert!(is_development_text_extension("dockerfile"));
    assert!(is_development_text_extension("yaml"));

    assert!(!is_development_text_extension("png"));
    assert!(!is_development_text_extension("exe"));
    assert!(!is_development_text_extension("zip"));
}

#[test]
fn test_is_jupyter_notebook_file() {
    assert!(is_jupyter_notebook_file("analysis.ipynb"));
    assert!(is_jupyter_notebook_file("Analysis.IPYNB"));
    assert!(is_jupyter_notebook_file("/path/to/notebook.ipynb"));

    assert!(!is_jupyter_notebook_file("notebook.json"));
    assert!(!is_jupyter_notebook_file("README.md"));
    assert!(!is_jupyter_notebook_file("ipynb"));
    assert!(!is_jupyter_notebook_file("notebook"));
}

#[test]
fn test_extensionless_files() {
    assert!(is_extensionless_text_file("README"));
    assert!(is_extensionless_text_file("LICENSE"));
    assert!(is_extensionless_text_file("Dockerfile"));
    assert!(is_extensionless_text_file("Justfile"));
    assert!(is_extensionless_text_file(".justfile"));
    assert!(is_extensionless_text_file(".gitignore"));
    assert!(is_extensionless_text_file(".gitmodules"));
    assert!(is_extensionless_text_file(".dockerignore"));
    assert!(is_extensionless_text_file(".agent-session-identity"));
    assert!(is_extensionless_text_file(".env"));

    assert!(!is_extensionless_text_file("binary"));
    assert!(!is_extensionless_text_file("unknown"));
    assert!(!is_extensionless_text_file("data"));
}

#[test]
fn test_unknown_file_content_is_classified() {
    let directory = tempfile::tempdir().unwrap();
    let text_path = directory.path().join("unknown");
    std::fs::write(&text_path, b"plain UTF-8 text\n").unwrap();
    assert!(!is_binary_file(&text_path));

    let binary_path = directory.path().join("unknown.data");
    std::fs::write(&binary_path, [0, 159, 146, 150]).unwrap();
    assert!(is_binary_file(&binary_path));
}

#[test]
fn configured_binary_extensions_remain_binary() {
    assert!(is_known_binary_extension("archive.zip"));
    assert!(is_binary_file("archive.zip"));
}
