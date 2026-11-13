use std::ops::Range;

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

#[cfg(test)]
mod tests {
    use super::mjml_css_regions;
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
}
