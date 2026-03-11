//! Markdown title extraction and HTML rendering helpers.

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html, utils::TextMergeWithOffset};

/// Resolves the page title from frontmatter or falls back to Markdown content.
pub(crate) fn resolve_title(title: Option<String>, body: &str) -> String {
    title.unwrap_or_else(|| title_from_markdown(body).to_string())
}

/// Uses the first non-empty line as the page title.
///
/// Valid ATX headings have their opening `#` sequence and any optional closing
/// markers removed, while preserving the remaining Markdown content.
fn title_from_markdown(body: &str) -> &str {
    let line = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();

    atx_heading_title(line).unwrap_or_else(|| line.trim())
}

/// Returns the content range of a valid ATX heading line, if present.
fn atx_heading_title(line: &str) -> Option<&str> {
    let mut events = TextMergeWithOffset::new(Parser::new(line).into_offset_iter());
    let Some((Event::Start(Tag::Heading { .. }), _)) = events.next() else {
        return None;
    };

    let mut content_range: Option<Range<usize>> = None;
    for (event, range) in events {
        if matches!(event, Event::End(TagEnd::Heading(_))) {
            let content = match content_range {
                Some(range) => &line[range],
                None => "",
            };
            return Some(content);
        }

        content_range = Some(match content_range {
            Some(content_range) => {
                content_range.start.min(range.start)..content_range.end.max(range.end)
            }
            None => range,
        });
    }

    None
}

/// Renders Markdown to HTML using the crate's enabled extensions.
pub(crate) fn markdown(text: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(text, options);
    let mut html = String::new();
    html::push_html(&mut html, parser);
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_from_markdown() {
        assert_eq!(title_from_markdown("# Foo"), "Foo");
        assert_eq!(title_from_markdown("## Foo"), "Foo");
        assert_eq!(title_from_markdown("  # Foo"), "Foo");
        assert_eq!(title_from_markdown("#\tFoo"), "Foo");
        assert_eq!(title_from_markdown("# Foo #"), "Foo");
        assert_eq!(title_from_markdown("# Foo ##"), "Foo");
        assert_eq!(title_from_markdown("# Foo#"), "Foo#");
        assert_eq!(title_from_markdown("# *Foo*"), "*Foo*");
        assert_eq!(title_from_markdown("# [Foo](bar)"), "[Foo](bar)");
        assert_eq!(title_from_markdown("# #"), "");
        assert_eq!(title_from_markdown("#"), "");
        assert_eq!(title_from_markdown("#5 bolt"), "#5 bolt");
        assert_eq!(title_from_markdown("###Foo"), "###Foo");
        assert_eq!(title_from_markdown("    # Foo"), "# Foo");
        assert_eq!(title_from_markdown("Foo"), "Foo");
        assert_eq!(title_from_markdown(""), "");
    }

    #[test]
    fn test_markdown_enables_extensions() {
        assert!(markdown("~~gone~~").contains("<del>gone</del>"));
        assert!(markdown("| head |\n| ---- |\n| body |").contains("<table>"));
        assert!(markdown("- [x] done").contains("type=\"checkbox\""));

        let footnotes = markdown("Text[^1]\n\n[^1]: note");
        assert!(footnotes.contains("footnote-reference"));
        assert!(footnotes.contains("footnote-definition"));
    }
}
