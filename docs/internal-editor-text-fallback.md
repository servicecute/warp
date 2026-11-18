# Internal editor text fallback

Warp separates file-content classification from language detection.

A file does not need a registered syntax grammar to be safely editable. The
classifier therefore uses this order:

1. known binary extensions remain binary candidates;
2. known text filenames and extensions remain text candidates;
3. unknown existing files are inspected by content;
4. valid UTF-8 text uses Warp's plain code editor even when no language is
   registered;
5. invalid UTF-8 or binary content remains eligible for the system/default
   application path.

The default `SystemDefault` open choice uses Warp's editor for unknown text.
An explicit `ExternalEditor`, `EnvEditor`, or `Open in default application`
action remains authoritative.

## Covered repository files

The text filename allowlist includes common repository metadata such as:

- `.gitmodules`;
- `.dockerignore` and `.containerignore`;
- `.agent-session-identity`;
- `.editorconfig`, `.gitattributes`, `.gitignore`;
- `.env` and `.env.*`; and
- ripgrep/ignore files such as `.ignore`, `.fdignore`, and `.rgignore`.

The list is intentionally conservative. Unknown names are accepted through
content inspection when the file exists, rather than through an unbounded
dotfile allowlist.

## User configuration

Additional names and simple glob patterns can be added in:

```toml
[code.editor]
additional_text_file_patterns = ["Justfile", ".env.*", "*.template"]
```

Configured patterns are an extension point for repository-specific UTF-8
files. They support exact file names and simple `*` patterns. Known binary
extensions and files whose content is actually binary still remain binary
candidates, even if a configured pattern matches them.

## Verification

Focused tests cover known metadata names, unknown UTF-8 extensionless files,
unknown binary content, the default Warp fallback, and preservation of an
explicit external editor choice.
