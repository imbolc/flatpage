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
