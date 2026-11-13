use std::ops::Range;

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

/// Finds CSS fragments that are embedded in MJML markup.
pub(crate) fn mjml_css_regions(source: &str) -> Vec<Range<usize>> {
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
    use super::{HandlebarsTokenKind, mjml_css_regions};
    use arborium::tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

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
}
