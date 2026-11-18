use std::collections::HashMap;
use std::iter;
use std::ops::Range;
use std::sync::Arc;

use arborium::tree_sitter::{Node, Query, QueryCursor, TextProvider, Tree};
use languages::CustomHighlighting;
use rangemap::RangeMap;
use streaming_iterator::StreamingIterator;
use string_offset::{ByteOffset, CharOffset};
use warp_editor::content::buffer::{Buffer, ToBufferByteOffset, ToBufferCharOffset};
use warp_editor::content::text::Bytes;
use warpui_core::color::ColorU;

/// Color mapping from parsed syntax token name to its corresponding highlighting color.
#[derive(Clone)]
pub struct ColorMap {
    pub keyword_color: ColorU,
    pub function_color: ColorU,
    pub string_color: ColorU,
    pub type_color: ColorU,
    pub number_color: ColorU,
    pub comment_color: ColorU,
    pub property_color: ColorU,
    pub tag_color: ColorU,
    pub custom_highlight_colors: Arc<HashMap<String, ColorU>>,
}

/// Query for retrieving syntax highlighting information on the tokens.
pub struct HighlightQuery {
    highlight_map: Vec<Option<ColorU>>,
    color_map: ColorMap,
}

impl HighlightQuery {
    pub fn new(query: &Query, color_map: ColorMap) -> Self {
        let highlight_map = query
            .capture_names()
            .iter()
            .map(|name| convert_capture_name_to_color(name, &color_map))
            .collect();

        Self {
            highlight_map,
            color_map,
        }
    }

    /// Given the a character range, return its corresponding highlight colors.
    pub fn get_highlighted_chunks(
        &self,
        range: Range<CharOffset>,
        query: &Query,
        buffer: &Buffer,
        tree: &Tree,
    ) -> RangeMap<CharOffset, ColorU> {
        let mut range_map = RangeMap::new();

        let mut cursor = QueryCursor::new();
        let byte_start = range.start.to_buffer_byte_offset(buffer).as_usize();
        let byte_end = range.end.to_buffer_byte_offset(buffer).as_usize();
        cursor.set_byte_range(byte_start..byte_end);
        let mut captures = cursor.captures(query, tree.root_node(), TextBuffer(buffer));

        while let Some(matches) = captures.next() {
            for cap in matches.0.captures {
                let insertion_range = cap.node.byte_range();
                let color = self
                    .highlight_map
                    .get(cap.index as usize)
                    .and_then(|inner| *inner);

                if let Some(color) = color {
                    let char_start =
                        ByteOffset::from(insertion_range.start).to_buffer_char_offset(buffer);
                    let char_end =
                        ByteOffset::from(insertion_range.end).to_buffer_char_offset(buffer);
                    if char_start < char_end {
                        range_map.insert(char_start..char_end, color);
                    }
                }
            }
        }

        range_map
    }

    pub fn get_custom_highlighted_chunks(
        &self,
        range: Range<CharOffset>,
        query: &Query,
        custom_highlighting: &CustomHighlighting,
        buffer: &Buffer,
        tree: &Tree,
    ) -> RangeMap<CharOffset, ColorU> {
        let mut range_map = RangeMap::new();
        let mut cursor = QueryCursor::new();
        let byte_start = range.start.to_buffer_byte_offset(buffer).as_usize();
        let byte_end = range.end.to_buffer_byte_offset(buffer).as_usize();
        cursor.set_byte_range(byte_start..byte_end);
        let mut captures = cursor.captures(query, tree.root_node(), TextBuffer(buffer));

        while let Some(matches) = captures.next() {
            for capture in matches.0.captures {
                let node_range = capture.node.byte_range();
                let text: Vec<u8> = buffer
                    .bytes_in_range(
                        ByteOffset::from(node_range.start),
                        ByteOffset::from(node_range.end),
                    )
                    .flat_map(|chunk| chunk.iter().copied())
                    .collect();
                let Ok(text) = String::from_utf8(text) else {
                    continue;
                };

                for custom_match in custom_highlighting.matches(&text) {
                    let Some(color) = self.color_for_scope(&custom_match.scope) else {
                        continue;
                    };
                    let char_start = ByteOffset::from(
                        (node_range.start + custom_match.range.start).saturating_sub(1),
                    )
                    .to_buffer_char_offset(buffer);
                    let char_end = ByteOffset::from(
                        (node_range.start + custom_match.range.end).saturating_sub(1),
                    )
                    .to_buffer_char_offset(buffer);
                    if char_start < char_end {
                        range_map.insert(char_start..char_end, color);
                    }
                }
            }
        }

        range_map
    }

    fn color_for_scope(&self, scope: &str) -> Option<ColorU> {
        self.color_map
            .custom_highlight_colors
            .get(scope)
            .copied()
            .or_else(|| match scope {
                "annotation.marker.gleam" => Some(self.color_map.keyword_color),
                "annotation.namespace.gleam" => Some(self.color_map.type_color),
                "annotation.key.gleam" => Some(self.color_map.property_color),
                "annotation.string.gleam" => Some(self.color_map.string_color),
                "annotation.number.gleam" => Some(self.color_map.number_color),
                _ => None,
            })
    }
}

fn convert_capture_name_to_color(name: &str, color_map: &ColorMap) -> Option<ColorU> {
    match name {
        "text.title" => return Some(color_map.keyword_color),
        "text.literal" => return Some(color_map.string_color),
        "text.uri" => return Some(color_map.function_color),
        "text.reference" => return Some(color_map.property_color),
        _ => {}
    }
    match name.split('.').next() {
        Some("keyword") => Some(color_map.keyword_color),
        Some("function") => Some(color_map.function_color),
        Some("string") => Some(color_map.string_color),
        Some("type") => Some(color_map.type_color),
        Some("number") => Some(color_map.number_color),
        Some("comment") => Some(color_map.comment_color),
        Some("property") => Some(color_map.property_color),
        Some("tag") => Some(color_map.tag_color),
        Some("punctuation") => Some(color_map.comment_color),
        _ => None,
    }
}

// The default tree-sitter implementation here is unsafe (since the cursor could query invalid ranges outside of content length).
// TODO(kevin): Once we migrate buffer to store ArrayStrings. We should implement the chunks API on buffer directly to avoid collecting
// into a String and then chunking them again for highlighting.
pub struct TextSlice<'a>(pub &'a [u8]);

impl TextSlice<'_> {
    fn get(&self, range: Range<usize>) -> Self {
        Self(self.0.get(range).unwrap_or_default())
    }
}

impl AsRef<[u8]> for TextSlice<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> TextProvider<TextSlice<'a>> for TextSlice<'a> {
    type I = iter::Once<TextSlice<'a>>;

    fn text(&mut self, node: Node) -> Self::I {
        iter::once(self.get(node.byte_range()))
    }
}

pub struct TextBuffer<'a>(pub &'a Buffer);

impl<'a> TextProvider<&'a [u8]> for TextBuffer<'a> {
    type I = Bytes<'a>;

    fn text(&mut self, node: Node) -> Self::I {
        let range = node.range();
        self.0.bytes_in_range(
            ByteOffset::from(range.start_byte),
            ByteOffset::from(range.end_byte),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arborium::tree_sitter::Parser;
    use languages::language_by_name;
    use string_offset::{ByteOffset, CharOffset};
    use warp_editor::content::buffer::{Buffer, BufferSnapshot};
    use warp_editor::content::selection_model::BufferSelectionModel;
    use warp_editor::content::text::IndentBehavior;
    use warpui_core::App;
    use warpui_core::color::ColorU;

    use super::*;

    fn test_color_map() -> ColorMap {
        ColorMap {
            keyword_color: ColorU::new(1, 2, 3, 255),
            function_color: ColorU::new(4, 5, 6, 255),
            string_color: ColorU::new(7, 8, 9, 255),
            type_color: ColorU::new(10, 11, 12, 255),
            number_color: ColorU::new(13, 14, 15, 255),
            comment_color: ColorU::new(16, 17, 18, 255),
            property_color: ColorU::new(19, 20, 21, 255),
            tag_color: ColorU::new(22, 23, 24, 255),
            custom_highlight_colors: Arc::new(HashMap::new()),
        }
    }

    #[test]
    fn gleam_codegen_overlay_is_limited_to_documentation_comments() {
        let language = language_by_name("gleam").expect("Gleam should be registered");
        let fixture = include_str!("../../../languages/grammars/gleam/codegen_fixture.gleam");
        let snapshot = BufferSnapshot::from_plain_text(fixture);
        let tree = {
            let mut parser = Parser::new();
            parser
                .set_language(&language.grammar)
                .expect("Gleam grammar should be compatible");
            let mut bytes = snapshot.bytes();
            let mut callback = |byte_offset: usize, _point| {
                bytes.seek(ByteOffset::from(byte_offset + 1));
                bytes.next().unwrap_or_default()
            };
            parser
                .parse_with_options(&mut callback, None, None)
                .expect("Gleam fixture should parse")
        };

        App::test((), |mut app| async move {
            let buffer_handle =
                app.add_model(|_| Buffer::new(Box::new(|_, _| IndentBehavior::Ignore)));
            let selection = app.add_model(|_| BufferSelectionModel::new(buffer_handle.clone()));
            buffer_handle.update(&mut app, |buffer, ctx| {
                *buffer = Buffer::from_plain_text(
                    fixture,
                    None,
                    Box::new(|_, _| IndentBehavior::Ignore),
                    selection,
                    ctx,
                );
            });

            let highlighting = language
                .custom_highlighting
                .as_ref()
                .expect("Gleam should have custom highlighting rules");
            let custom_query = language
                .custom_highlight_query
                .as_ref()
                .expect("Gleam should select documentation comments");
            let color_map = test_color_map();
            let marker_color = color_map.keyword_color;
            let key_color = color_map.property_color;
            let query = HighlightQuery::new(&language.highlight_query, color_map);

            let chunks = buffer_handle.read(&app, |buffer, _| {
                query.get_custom_highlighted_chunks(
                    CharOffset::zero()..buffer.len(),
                    custom_query,
                    highlighting,
                    buffer,
                    &tree,
                )
            });

            let matched_text: Vec<_> = chunks
                .iter()
                .map(|(range, _)| &fixture[range.start.as_usize()..range.end.as_usize()])
                .collect();
            assert!(matched_text.contains(&"@codegen"));
            assert!(matched_text.contains(&"@json"));
            assert!(matched_text.contains(&"@firestore"));
            assert!(matched_text.contains(&"tag_style"));
            assert!(chunks.iter().any(|(range, color)| {
                &fixture[range.start.as_usize()..range.end.as_usize()] == "@codegen"
                    && *color == marker_color
            }));
            assert!(chunks.iter().any(|(range, color)| {
                &fixture[range.start.as_usize()..range.end.as_usize()] == "tag_style"
                    && *color == key_color
            }));
            assert_eq!(
                matched_text
                    .iter()
                    .filter(|text| **text == "@codegen")
                    .count(),
                1
            );
        });
    }
}
