use std::path::PathBuf;

const ALLOWED_IN_URL_SEGMENT: &str = "_-.";

/// Returns whether a single URL path segment is accepted by the crate.
pub(crate) fn is_valid_url_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ALLOWED_IN_URL_SEGMENT.contains(c))
}

/// Converts an already normalized URL into its relative Markdown file path.
pub(crate) fn normalized_url_to_path(url: &str) -> PathBuf {
    if url == "/" {
        return PathBuf::from("index.md");
    }

    let mut path = PathBuf::new();
    let mut segments = url.trim_matches('/').split('/');
    let last_segment = segments.next_back().unwrap_or_default();
    for segment in segments {
        path.push(segment);
    }
    if url.ends_with('/') {
        path.push(last_segment);
        path.push("index.md");
    } else {
        path.push(format!("{last_segment}.md"));
    }
    path
}
