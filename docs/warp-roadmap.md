# Internal Warp Roadmap

This document tracks editor and syntax-highlighting improvements curated for
the ServiceCute Warp fork. It is an internal planning document, not a promise
of upstream Warp support.

## Mobile file syntax highlighting

Dart syntax highlighting is currently being reviewed in [PR #2](https://github.com/servicecute/warp/pull/2).
The following mobile-development file families remain planned follow-ups:

### Gradle Groovy

Status: planned.

Add syntax highlighting and filename resolution for `.gradle` files used by
Android and Kotlin Multiplatform builds. `.gradle.kts` is already covered by
the Kotlin mapping through its final `.kts` extension.

### Android and mobile configuration files

Status: planned.

Add focused support for `.properties` and ProGuard/R8 `.pro` files. Android
manifest and resource XML files already use the XML highlighter.

### iOS XML resource and project files

Status: planned.

Map `.plist`, `.storyboard`, `.xib`, and `.entitlements` to the XML highlighter
or an appropriate specialized grammar where XML alone is insufficient.

### React Native stylesheets

Status: planned.

Add syntax highlighting and filename resolution for `.scss`, `.sass`, and
`.less` files commonly used by React Native projects and shared mobile web
stacks.

Implementation requirements for each follow-up:

- Register the language or extension mapping in the shared language registry.
- Keep text-file detection and language detection independent.
- Add representative fixtures and filename-resolution tests.
- Verify rendered captures in the editor before release-branch integration.
