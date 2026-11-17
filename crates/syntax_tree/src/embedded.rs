use std::ops::Range;

use languages::EmbeddedLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BuildTokenKind {
    Comment,
    Function,
    Keyword,
    Property,
    Punctuation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BuildToken {
    pub range: Range<usize>,
    pub kind: BuildTokenKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HandlebarsTokenKind {
    Delimiter,
    Keyword,
    Helper,
    Path,
    String,
    Number,
    Comment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HandlebarsToken {
    pub range: Range<usize>,
    pub kind: HandlebarsTokenKind,
}

/// Finds regions owned by an embedded language.
///
/// The caller supplies the language explicitly so a parent grammar cannot
/// accidentally run an unrelated child parser over the whole document.
pub(crate) fn regions_for(source: &str, language: EmbeddedLanguage) -> Vec<Range<usize>> {
    match language {
        EmbeddedLanguage::Css => mjml_css_regions(source),
        EmbeddedLanguage::Handlebars => Vec::new(),
        EmbeddedLanguage::Makefile | EmbeddedLanguage::Justfile => Vec::new(),
        EmbeddedLanguage::Sql => query_regions(source),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StringLiteral {
    content_ranges: Vec<Range<usize>>,
    full_range: Range<usize>,
}

/// Finds static string fragments passed to query-like calls or referenced by
/// a local query variable. The scanner deliberately stays lexical: it does
/// not attempt whole-program data-flow or infer arbitrary runtime values.
fn query_regions(source: &str) -> Vec<Range<usize>> {
    let strings = string_literals(source);
    let calls = query_call_arguments(source);
    let mut regions = Vec::new();

    for argument in calls {
        let direct_strings = strings
            .iter()
            .filter(|literal| argument.contains(&literal.full_range.start))
            .collect::<Vec<_>>();
        if !direct_strings.is_empty() {
            regions.extend(
                direct_strings
                    .into_iter()
                    .flat_map(|literal| literal.content_ranges.iter().cloned()),
            );
            continue;
        }

        let argument_text = source[argument.clone()].trim();
        let Some(identifier) = simple_identifier(argument_text) else {
            continue;
        };
        collect_local_query_regions(
            source,
            strings.as_slice(),
            identifier,
            argument.start,
            &mut Vec::new(),
            &mut regions,
        );
    }

    regions.sort_by_key(|range| (range.start, range.end));
    regions.dedup();
    regions
}

fn collect_local_query_regions(
    source: &str,
    strings: &[StringLiteral],
    identifier: &str,
    before: usize,
    visited: &mut Vec<String>,
    regions: &mut Vec<Range<usize>>,
) {
    if visited.iter().any(|seen| seen == identifier) {
        return;
    }
    visited.push(identifier.to_string());
    let Some(assignment) = local_assignment(source, identifier, before) else {
        visited.pop();
        return;
    };
    let expression_start = source[assignment.start..]
        .find('=')
        .map_or(assignment.end, |offset| assignment.start + offset + 1);
    let expression = source[expression_start..assignment.end].trim();
    if let Some(alias) = simple_identifier(expression) {
        collect_local_query_regions(source, strings, alias, assignment.start, visited, regions);
    } else {
        regions.extend(
            strings
                .iter()
                .filter(|literal| {
                    literal.full_range.start >= assignment.start
                        && literal.full_range.end <= assignment.end
                })
                .flat_map(|literal| literal.content_ranges.iter().cloned()),
        );
    }
    visited.pop();
}

pub(crate) fn is_surrealql_query(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    [
        "define ",
        "relate ",
        "record<",
        "fetch ",
        "live select",
        "upsert ",
        "info for",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || lower.contains("->")
        || lower.contains("::")
}

fn simple_identifier(text: &str) -> Option<&str> {
    let text = text.trim();
    (!text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric()))
    .then_some(text)
}

fn query_call_arguments(source: &str) -> Vec<Range<usize>> {
    let mut arguments = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor should be in source");
        if character == '/' && source[cursor..].starts_with("//") {
            cursor = source[cursor..]
                .find('\n')
                .map_or(source.len(), |offset| cursor + offset);
            continue;
        }
        if character == '/' && source[cursor..].starts_with("/*") {
            cursor = source[cursor + 2..]
                .find("*/")
                .map_or(source.len(), |offset| cursor + offset + 4);
            continue;
        }
        if character == 'r'
            && let Some(literal) = raw_string_literal(source, cursor)
        {
            cursor = literal.full_range.end;
            continue;
        }
        if matches!(character, '"' | '\'' | '`') {
            cursor = skip_string(source, cursor);
            continue;
        }
        if !source[cursor..].starts_with("query") {
            cursor += character.len_utf8();
            continue;
        }
        let start = cursor;
        let end = start + "query".len();
        let before_is_identifier = start > 0
            && source[..start]
                .chars()
                .next_back()
                .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        let is_query_with = source[end..].starts_with("_with")
            && !source[end + "_with".len()..]
                .chars()
                .next()
                .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        let after_is_identifier = source[end..]
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        if before_is_identifier || (after_is_identifier && !is_query_with) {
            cursor = end;
            continue;
        }

        let mut open = end;
        while source[open..].starts_with("_with") {
            open += "_with".len();
        }
        while source[open..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            open += source[open..]
                .chars()
                .next()
                .expect("whitespace character should exist")
                .len_utf8();
        }
        if !source[open..].starts_with('(') {
            cursor = end;
            continue;
        }
        if let Some(close) = matching_delimiter(source, open, '(', ')') {
            let argument_start = open + 1;
            let argument_end = first_argument_end(source, argument_start, close);
            arguments.push(argument_start..argument_end);
            cursor = close + 1;
        } else {
            arguments.push(open + 1..source.len());
            break;
        }
    }
    arguments
}

fn first_argument_end(source: &str, start: usize, close: usize) -> usize {
    let mut cursor = start;
    let mut depth: usize = 0;
    while cursor < close {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor should be in source");
        match character {
            '"' | '\'' | '`' => {
                cursor = skip_string(source, cursor).min(close);
                continue;
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return cursor,
            _ => {}
        }
        cursor += character.len_utf8();
    }
    close
}

fn matching_delimiter(source: &str, open: usize, opening: char, closing: char) -> Option<usize> {
    let mut depth = 0;
    let mut cursor = open;
    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor should be in source");
        match character {
            '"' | '\'' | '`' => cursor = skip_string(source, cursor),
            character if character == opening => {
                depth += 1;
                cursor += character.len_utf8();
            }
            character if character == closing => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor += character.len_utf8();
            }
            _ => cursor += character.len_utf8(),
        }
    }
    None
}

fn local_assignment(source: &str, identifier: &str, before: usize) -> Option<Range<usize>> {
    let mut search_from = 0;
    let mut result = None;
    while search_from < before {
        let Some(relative) = source[search_from..before].find(identifier) else {
            break;
        };
        let name_start = search_from + relative;
        let name_end = name_start + identifier.len();
        let valid_name = (name_start == 0
            || !source[..name_start]
                .chars()
                .next_back()
                .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric()))
            && (name_end == source.len()
                || !source[name_end..].chars().next().is_some_and(|character| {
                    character == '_' || character.is_ascii_alphanumeric()
                }));
        if !valid_name {
            search_from = name_end;
            continue;
        }
        let mut equals = name_end;
        while source[equals..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            equals += source[equals..]
                .chars()
                .next()
                .expect("whitespace character should exist")
                .len_utf8();
        }
        if source[equals..].starts_with(':') {
            let type_end = source[equals..]
                .find(['=', ';', '\n'])
                .map_or(before, |offset| equals + offset);
            let Some(relative) = source[equals..=type_end].find('=') else {
                search_from = name_end;
                continue;
            };
            equals += relative;
        }
        if source[equals..].starts_with('=') {
            let end = source[equals..]
                .find(';')
                .map_or(before, |offset| equals + offset);
            result = Some(name_start..end);
        }
        search_from = name_end;
    }
    result
}

fn string_literals(source: &str) -> Vec<StringLiteral> {
    let mut literals = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor should be in source");
        if character == '/' && source[cursor..].starts_with("//") {
            cursor = source[cursor..]
                .find('\n')
                .map_or(source.len(), |offset| cursor + offset);
            continue;
        }
        if character == '/' && source[cursor..].starts_with("/*") {
            cursor = source[cursor + 2..]
                .find("*/")
                .map_or(source.len(), |offset| cursor + offset + 4);
            continue;
        }
        if character == 'r'
            && source[cursor..].contains('"')
            && let Some(literal) = raw_string_literal(source, cursor)
        {
            cursor = literal.full_range.end;
            literals.push(literal);
            continue;
        }
        if matches!(character, '"' | '\'' | '`') {
            let literal = quoted_string_literal(source, cursor, character);
            cursor = literal.full_range.end;
            literals.push(literal);
            continue;
        }
        cursor += character.len_utf8();
    }
    literals
}

fn raw_string_literal(source: &str, start: usize) -> Option<StringLiteral> {
    let mut quote = start;
    while source[quote..].starts_with('#') || source[quote..].starts_with('r') {
        quote += 1;
    }
    if !source[quote..].starts_with('"') {
        return None;
    }
    let hashes = quote - start - 1;
    let close = format!("\"{}", "#".repeat(hashes));
    let content_start = quote + 1;
    let content_end = source[content_start..]
        .find(&close)
        .map_or(source.len(), |offset| content_start + offset);
    Some(StringLiteral {
        content_ranges: std::iter::once(content_start..content_end).collect(),
        full_range: start..(content_end + close.len()).min(source.len()),
    })
}

fn quoted_string_literal(source: &str, start: usize, quote: char) -> StringLiteral {
    let content_start = start + quote.len_utf8();
    let mut cursor = content_start;
    let mut interpolation_start = content_start;
    let mut content_ranges = Vec::new();
    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor should be in source");
        if character == '\\' {
            cursor += 1;
            if cursor < source.len() {
                cursor += source[cursor..]
                    .chars()
                    .next()
                    .expect("escaped character should exist")
                    .len_utf8();
            }
            continue;
        }
        if quote == '`' && source[cursor..].starts_with("${") {
            content_ranges.push(interpolation_start..cursor);
            let interpolation_end = matching_delimiter(source, cursor + 1, '{', '}')
                .unwrap_or(source.len().saturating_sub(1));
            cursor = (interpolation_end + 1).min(source.len());
            interpolation_start = cursor;
            continue;
        }
        if character == quote {
            content_ranges.push(interpolation_start..cursor);
            return StringLiteral {
                content_ranges,
                full_range: start..cursor + quote.len_utf8(),
            };
        }
        cursor += character.len_utf8();
    }
    content_ranges.push(interpolation_start..source.len());
    StringLiteral {
        content_ranges,
        full_range: start..source.len(),
    }
}

fn skip_string(source: &str, start: usize) -> usize {
    if source[start..].starts_with('r')
        && let Some(literal) = raw_string_literal(source, start)
    {
        return literal.full_range.end;
    }
    let quote = source[start..].chars().next().expect("quote should exist");
    quoted_string_literal(source, start, quote).full_range.end
}

pub(crate) fn build_tokens(source: &str, language: EmbeddedLanguage) -> Vec<BuildToken> {
    let mut tokens = Vec::new();
    let mut line_start = 0;

    for line in source.split_inclusive('\n') {
        let line_end = line_start + line.trim_end_matches(['\n', '\r']).len();
        let content = &source[line_start..line_end];
        let trimmed = content.trim_start();
        let leading = content.len() - trimmed.len();

        if trimmed.starts_with('#') {
            tokens.push(BuildToken {
                range: line_start + leading..line_end,
                kind: BuildTokenKind::Comment,
            });
        } else if !trimmed.is_empty() && !content.starts_with('\t') && !content.starts_with(' ') {
            let first_word_end = trimmed
                .find(|c: char| c.is_whitespace())
                .unwrap_or(trimmed.len());
            let first_word = &trimmed[..first_word_end];
            if is_build_keyword(first_word, language) {
                tokens.push(BuildToken {
                    range: line_start + leading..line_start + leading + first_word_end,
                    kind: BuildTokenKind::Keyword,
                });
            }

            if let Some(colon) = find_unquoted(content, ':')
                && !content[..colon].contains('=')
                && colon > 0
            {
                let target_end = content[..colon].trim_end().len();
                if target_end > 0 {
                    tokens.push(BuildToken {
                        range: line_start..line_start + target_end,
                        kind: BuildTokenKind::Function,
                    });
                    tokens.push(BuildToken {
                        range: line_start + colon..line_start + colon + 1,
                        kind: BuildTokenKind::Punctuation,
                    });
                }
            }
        }

        if let Some((operator_start, operator_end)) = find_assignment(content) {
            let name_end = content[..operator_start].trim_end().len();
            let name_start = content[..name_end]
                .rfind(|character: char| character.is_whitespace())
                .map_or(0, |index| index + 1);
            if name_start < name_end {
                tokens.push(BuildToken {
                    range: line_start + name_start..line_start + name_end,
                    kind: BuildTokenKind::Property,
                });
                tokens.push(BuildToken {
                    range: line_start + operator_start..line_start + operator_end,
                    kind: BuildTokenKind::Punctuation,
                });
            }
        }

        variable_tokens(source, line_start, line_end, &mut tokens);
        if language == EmbeddedLanguage::Justfile {
            interpolation_tokens(source, line_start, line_end, &mut tokens);
        }

        line_start += line.len();
    }

    tokens
}

fn is_build_keyword(word: &str, language: EmbeddedLanguage) -> bool {
    match language {
        EmbeddedLanguage::Makefile => matches!(
            word,
            "define"
                | "endef"
                | "include"
                | "-include"
                | "export"
                | "override"
                | "private"
                | "vpath"
                | "undefine"
                | "ifeq"
                | "ifneq"
                | "ifdef"
                | "ifndef"
                | "else"
                | "endif"
        ),
        EmbeddedLanguage::Justfile => matches!(
            word,
            "set" | "import" | "mod" | "export" | "alias" | "private" | "script"
        ),
        EmbeddedLanguage::Css | EmbeddedLanguage::Handlebars | EmbeddedLanguage::Sql => false,
    }
}

fn find_assignment(line: &str) -> Option<(usize, usize)> {
    ["?=", ":=", "+=", "!=", "="]
        .iter()
        .filter_map(|operator| {
            line.find(operator)
                .map(|start| (start, start + operator.len()))
        })
        .min_by_key(|(start, _)| *start)
}

fn find_unquoted(line: &str, needle: char) -> Option<usize> {
    let mut quote = None;
    for (index, character) in line.char_indices() {
        match character {
            '\'' | '"' if quote == Some(character) => quote = None,
            '\'' | '"' if quote.is_none() => quote = Some(character),
            _ if character == needle && quote.is_none() => return Some(index),
            _ => {}
        }
    }
    None
}

fn variable_tokens(source: &str, start: usize, end: usize, tokens: &mut Vec<BuildToken>) {
    let line = &source[start..end];
    let mut offset = 0;
    while let Some(relative) = line[offset..]
        .find("$(")
        .or_else(|| line[offset..].find("${"))
    {
        let variable_start = offset + relative;
        let close = line[variable_start + 2..]
            .find(if line[variable_start..].starts_with("$(") {
                ')'
            } else {
                '}'
            })
            .map(|index| variable_start + 3 + index)
            .unwrap_or(line.len());
        tokens.push(BuildToken {
            range: start + variable_start..start + close,
            kind: BuildTokenKind::Property,
        });
        offset = close.max(variable_start + 2);
        if offset >= line.len() {
            break;
        }
    }
}

fn interpolation_tokens(source: &str, start: usize, end: usize, tokens: &mut Vec<BuildToken>) {
    let line = &source[start..end];
    let mut offset = 0;
    while let Some(relative) = line[offset..].find("{{") {
        let interpolation_start = offset + relative;
        let interpolation_end = line[interpolation_start + 2..]
            .find("}}")
            .map(|index| interpolation_start + 4 + index)
            .unwrap_or(line.len());
        tokens.push(BuildToken {
            range: start + interpolation_start..start + interpolation_start + 2,
            kind: BuildTokenKind::Punctuation,
        });
        if interpolation_end > interpolation_start + 2 {
            tokens.push(BuildToken {
                range: start + interpolation_start + 2..start + interpolation_end - 2,
                kind: BuildTokenKind::Property,
            });
        }
        if interpolation_end <= line.len()
            && line[interpolation_end.saturating_sub(2)..].starts_with("}}")
        {
            tokens.push(BuildToken {
                range: start + interpolation_end - 2..start + interpolation_end,
                kind: BuildTokenKind::Punctuation,
            });
        }
        offset = interpolation_end.max(interpolation_start + 2);
        if offset >= line.len() {
            break;
        }
    }
}

/// Finds CSS fragments that are embedded in MJML markup.
fn mjml_css_regions(source: &str) -> Vec<Range<usize>> {
    let lower = source.to_ascii_lowercase();
    let mut regions = Vec::new();

    let mut search_from = 0;
    while let Some(open_start) = lower[search_from..].find("<mj-style") {
        let open_start = search_from + open_start;
        let Some(open_end) = lower[open_start..].find('>') else {
            break;
        };
        let content_start = open_start + open_end + 1;
        let Some(close_start) = lower[content_start..].find("</mj-style") else {
            regions.push(content_start..source.len());
            break;
        };
        let close_start = content_start + close_start;
        regions.push(content_start..close_start);
        search_from = close_start + "</mj-style".len();
    }

    let mut search_from = 0;
    while let Some(style_start) = lower[search_from..].find("style=") {
        let style_start = search_from + style_start + "style=".len();
        let Some(quote) = source[style_start..].chars().next() else {
            break;
        };
        if quote != '\'' && quote != '"' {
            search_from = style_start;
            continue;
        }
        let content_start = style_start + quote.len_utf8();
        let Some(content_end) = source[content_start..].find(quote) else {
            regions.push(content_start..source.len());
            break;
        };
        let content_end = content_start + content_end;
        regions.push(content_start..content_end);
        search_from = content_end + quote.len_utf8();
    }

    regions
}

/// Lexes Handlebars expressions without requiring a Handlebars tree-sitter grammar.
/// The lexer is deliberately tolerant so incomplete templates remain highlightable.
pub(crate) fn handlebars_tokens(source: &str) -> Vec<HandlebarsToken> {
    let mut tokens = Vec::new();
    let mut search_from = 0;

    while let Some(relative_start) = source[search_from..].find("{{") {
        let open_start = search_from + relative_start;
        if is_escaped(source, open_start) {
            search_from = open_start + 2;
            continue;
        }

        let triple = source[open_start..].starts_with("{{{");
        let content_start = open_start + if triple { 3 } else { 2 };
        let close_marker = if triple { "}}}" } else { "}}" };
        let content_end = source[content_start..]
            .find(close_marker)
            .map(|offset| content_start + offset)
            .unwrap_or(source.len());
        let expression_end = if content_end < source.len() {
            content_end + close_marker.len()
        } else {
            source.len()
        };

        let content = source[content_start..content_end].trim_start();
        let trimmed_start =
            content_start + source[content_start..content_end].len() - content.len();
        if content.starts_with("!--") || content.starts_with('!') {
            tokens.push(HandlebarsToken {
                range: open_start..expression_end,
                kind: HandlebarsTokenKind::Comment,
            });
        } else {
            tokens.push(HandlebarsToken {
                range: open_start..content_start,
                kind: HandlebarsTokenKind::Delimiter,
            });
            lex_expression(source, trimmed_start, content_end, &mut tokens);
            if content_end < source.len() {
                tokens.push(HandlebarsToken {
                    range: content_end..expression_end,
                    kind: HandlebarsTokenKind::Delimiter,
                });
            }
        }

        search_from = expression_end.max(open_start + 2);
    }

    tokens
}

fn is_escaped(source: &str, byte_offset: usize) -> bool {
    let mut slash_count = 0;
    let mut cursor = byte_offset;
    while cursor > 0 && source.as_bytes()[cursor - 1] == b'\\' {
        slash_count += 1;
        cursor -= 1;
    }
    slash_count % 2 == 1
}

fn lex_expression(source: &str, start: usize, end: usize, tokens: &mut Vec<HandlebarsToken>) {
    #[derive(Clone, Copy)]
    enum LexemeKind {
        Bare,
        Quoted,
        Number,
        Equals,
    }

    let mut lexemes = Vec::new();
    let mut cursor = start;
    let mut prefix = None;
    while cursor < end {
        let Some(ch) = source[cursor..end].chars().next() else {
            break;
        };
        if ch.is_whitespace() {
            cursor += ch.len_utf8();
            continue;
        }
        if prefix.is_none() && matches!(ch, '#' | '/' | '^' | '>' | '&') {
            prefix = Some(ch);
            tokens.push(HandlebarsToken {
                range: cursor..cursor + ch.len_utf8(),
                kind: HandlebarsTokenKind::Keyword,
            });
            cursor += ch.len_utf8();
            continue;
        }
        if ch == '=' {
            lexemes.push((cursor..cursor + 1, LexemeKind::Equals));
            cursor += 1;
            continue;
        }
        if ch == '\'' || ch == '"' {
            let quote = ch;
            let token_start = cursor;
            cursor += quote.len_utf8();
            while cursor < end {
                let next = source[cursor..end].chars().next().expect("cursor in range");
                cursor += next.len_utf8();
                if next == quote && !is_escaped(source, cursor - next.len_utf8()) {
                    break;
                }
            }
            lexemes.push((token_start..cursor, LexemeKind::Quoted));
            continue;
        }

        let token_start = cursor;
        while cursor < end {
            let next = source[cursor..end].chars().next().expect("cursor in range");
            if next.is_whitespace() || next == '=' || next == '\'' || next == '"' {
                break;
            }
            cursor += next.len_utf8();
        }
        if token_start == cursor {
            cursor += ch.len_utf8();
            continue;
        }
        let text = &source[token_start..cursor];
        let kind = if text.parse::<f64>().is_ok() {
            LexemeKind::Number
        } else {
            LexemeKind::Bare
        };
        lexemes.push((token_start..cursor, kind));
    }

    let bare_count = lexemes
        .iter()
        .filter(|(_, kind)| matches!(kind, LexemeKind::Bare))
        .count();
    let first_bare_position = lexemes
        .iter()
        .position(|(_, kind)| matches!(kind, LexemeKind::Bare));
    let first_bare_is_hash_key = first_bare_position
        .and_then(|position| lexemes.get(position + 1))
        .is_some_and(|(_, kind)| matches!(kind, LexemeKind::Equals));
    let mut bare_index = 0;
    for (range, kind) in lexemes {
        let token_kind = match kind {
            LexemeKind::Quoted => HandlebarsTokenKind::String,
            LexemeKind::Number => HandlebarsTokenKind::Number,
            LexemeKind::Equals => HandlebarsTokenKind::Delimiter,
            LexemeKind::Bare => {
                let text = &source[range.clone()];
                let is_first = bare_index == 0;
                bare_index += 1;
                if is_keyword(text) || text == "else" {
                    HandlebarsTokenKind::Keyword
                } else if is_first
                    && !first_bare_is_hash_key
                    && (prefix.is_some() && prefix != Some('&')
                        || is_helper(text)
                        || bare_count > 1)
                {
                    HandlebarsTokenKind::Helper
                } else if text == "true" || text == "false" || text == "null" {
                    HandlebarsTokenKind::Keyword
                } else {
                    HandlebarsTokenKind::Path
                }
            }
        };
        tokens.push(HandlebarsToken {
            range,
            kind: token_kind,
        });
    }
}

fn is_keyword(text: &str) -> bool {
    matches!(text, "if" | "unless" | "each" | "with" | "else")
}

fn is_helper(text: &str) -> bool {
    matches!(text, "lookup" | "log" | "helper" | "include")
}

#[cfg(test)]
mod tests {
    use super::{
        BuildTokenKind, HandlebarsTokenKind, build_tokens, is_surrealql_query, mjml_css_regions,
        query_regions, regions_for,
    };
    use arborium::tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};
    use languages::EmbeddedLanguage;

    #[test]
    fn finds_block_and_inline_css_regions() {
        let source = r#"<mj-style>.title { color: red; }</mj-style>
<mj-text style="color: blue;">Hello</mj-text>"#;
        let regions = mjml_css_regions(source);

        assert_eq!(regions.len(), 2);
        assert_eq!(&source[regions[0].clone()], ".title { color: red; }");
        assert_eq!(&source[regions[1].clone()], "color: blue;");
    }

    #[test]
    fn embedded_regions_are_scoped_to_the_declared_language() {
        let source = r#"<mj-style>.title { color: red; }</mj-style>
<mj-text style="color: blue;">{{name}}</mj-text>"#;
        let css_regions = regions_for(source, EmbeddedLanguage::Css);

        assert_eq!(css_regions.len(), 2);
        assert_eq!(&source[css_regions[0].clone()], ".title { color: red; }");
        assert_eq!(&source[css_regions[1].clone()], "color: blue;");
        assert!(regions_for(source, EmbeddedLanguage::Handlebars).is_empty());
    }

    #[test]
    fn recovers_unclosed_style_block() {
        let source = "<mj-style>.title { color: red; }";
        let regions = mjml_css_regions(source);

        assert_eq!(regions, vec!["<mj-style>".len()..source.len()]);
    }

    #[test]
    fn css_fragments_produce_highlight_captures() {
        let source = r#"<mj-style>.title { color: red; }</mj-style>
<mj-text style="color: blue;">Hello</mj-text>"#;
        let css_grammar = arborium::get_language("css").expect("CSS grammar should load");
        let css_query = Query::new(&css_grammar, arborium::lang_css::HIGHLIGHTS_QUERY)
            .expect("CSS highlight query should be valid");
        let mut parser = Parser::new();
        parser
            .set_language(&css_grammar)
            .expect("CSS parser should accept CSS fragments");

        let captures = mjml_css_regions(source)
            .into_iter()
            .map(|region| {
                let fragment = &source[region];
                let tree = parser.parse(fragment, None).expect("CSS should parse");
                let mut cursor = QueryCursor::new();
                let mut captures =
                    cursor.captures(&css_query, tree.root_node(), fragment.as_bytes());
                let mut count = 0;
                while let Some((query_match, _)) = captures.next() {
                    count += query_match.captures.len();
                }
                count
            })
            .collect::<Vec<_>>();

        assert!(captures.iter().all(|count| *count > 0));
    }

    #[test]
    fn handlebars_lexer_covers_blocks_helpers_paths_literals_and_comments() {
        let source = r#"{{!-- comment --}}{{#each items}}{{lookup ../prices sku}} {{format_currency amount currency="USD"}} {{name}} {{true}} {{42}} {{"ok"}}{{/each}}{{> footer order=order}}"#;
        let tokens = super::handlebars_tokens(source);

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::Comment)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::Helper)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::Path)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::String)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::Number)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::Delimiter)
        );
        assert!(!tokens.iter().any(|token| {
            token.kind == HandlebarsTokenKind::Helper && &source[token.range.clone()] == "order"
        }));
    }

    #[test]
    fn handlebars_lexer_recovers_incomplete_and_escaped_expressions() {
        let source = r#"literal \{{not-a-template}} {{#if user"#;
        let tokens = super::handlebars_tokens(source);

        assert!(tokens.iter().all(|token| token.range.end <= source.len()));
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == HandlebarsTokenKind::Keyword)
        );
        assert!(
            !tokens
                .iter()
                .any(|token| source[token.range.clone()].contains("not-a-template"))
        );
    }

    #[test]
    fn makefile_tokens_cover_targets_directives_assignments_and_variables() {
        let source = "include config.mk\nCC := cc\n.PHONY: all\nall: $(OBJECTS)\n\t$(CC) -o app\n# comment\n";
        let tokens = build_tokens(source, EmbeddedLanguage::Makefile);

        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Keyword && &source[token.range.clone()] == "include"
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Property && &source[token.range.clone()] == "CC"
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Function && &source[token.range.clone()] == ".PHONY"
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Property && &source[token.range.clone()] == "$(OBJECTS)"
        }));
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == BuildTokenKind::Comment)
        );
    }

    #[test]
    fn justfile_tokens_cover_recipes_directives_assignments_and_interpolation() {
        let source = "set shell := [\"bash\", \"-cu\"]\nname := \"warp\"\nbuild: name\n    echo {{name}}\n# comment\n";
        let tokens = build_tokens(source, EmbeddedLanguage::Justfile);

        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Keyword && &source[token.range.clone()] == "set"
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Function && &source[token.range.clone()] == "build"
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Punctuation && &source[token.range.clone()] == "{{"
        }));
        assert!(tokens.iter().any(|token| {
            token.kind == BuildTokenKind::Property && &source[token.range.clone()] == "name"
        }));
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == BuildTokenKind::Comment)
        );
    }

    #[test]
    fn query_regions_find_direct_and_local_variable_queries() {
        let source = r##"
let query = r#"SELECT * FROM person WHERE name = $name"#;
let sql = query;
let typed_sql: &str = "SELECT * FROM typed_users";
let prose = "SELECT this is ordinary prose";
// db.query("SELECT from a comment");
let comment = "db.query(\\\"SELECT inside a string\\\")";
db.query(query);
db.query(sql);
db.query(typed_sql);
db.query_with("DEFINE TABLE person SCHEMAFULL;", vars);
"##;
        let regions = query_regions(source);

        assert_eq!(regions.len(), 3);
        assert_eq!(
            &source[regions[0].clone()],
            "SELECT * FROM person WHERE name = $name"
        );
        assert_eq!(&source[regions[1].clone()], "SELECT * FROM typed_users");
        assert_eq!(
            &source[regions[2].clone()],
            "DEFINE TABLE person SCHEMAFULL;"
        );
        assert!(
            !regions
                .iter()
                .any(|range| { source[range.clone()].contains("ordinary prose") })
        );
    }

    #[test]
    fn query_regions_preserve_template_interpolations_and_recover_incomplete_strings() {
        let source = "client.query(`SELECT * FROM person WHERE id = ${id} AND active = true`);\nclient.query(\"SELECT * FROM\");";
        let regions = query_regions(source);

        assert_eq!(regions.len(), 3);
        assert_eq!(
            &source[regions[0].clone()],
            "SELECT * FROM person WHERE id = "
        );
        assert_eq!(&source[regions[1].clone()], " AND active = true");
        assert_eq!(&source[regions[2].clone()], "SELECT * FROM");
        assert!(!regions.iter().any(|range| &source[range.clone()] == "id"));
    }

    #[test]
    fn surrealql_query_detection_is_specific_to_surreal_features() {
        assert!(is_surrealql_query("DEFINE TABLE person SCHEMAFULL;"));
        assert!(is_surrealql_query(
            "SELECT * FROM person:tobie->likes->person:jaime"
        ));
        assert!(!is_surrealql_query("SELECT * FROM users WHERE id = $1"));
    }
}
