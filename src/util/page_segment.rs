const ALLOWED_IN_PAGE_SEGMENT: &str = "_-.";

/// Returns whether a single page path segment is accepted by the crate.
pub(crate) fn is_valid_page_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ALLOWED_IN_PAGE_SEGMENT.contains(c))
}

#[cfg(test)]
mod tests {
    use super::is_valid_page_segment;

    #[test]
    fn test_is_valid_page_segment() {
        for segment in ["foo", "foo-bar", "foo_bar", "v1.2", ".md"] {
            assert!(
                is_valid_page_segment(segment),
                "{segment:?} should be valid"
            );
        }

        for segment in ["", ".", "..", "foo/bar", "foo?", "ы"] {
            assert!(
                !is_valid_page_segment(segment),
                "{segment:?} should be invalid"
            );
        }
    }
}
