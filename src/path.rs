use std::path::{Component, Path, PathBuf};

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

/// Converts a relative Markdown path into its canonical URL form.
pub(crate) fn path_to_url(path: &Path) -> Option<String> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(segment) = component else {
            return None;
        };
        let segment = segment.to_str()?;
        components.push(segment);
    }

    let file_name = components.pop()?;
    for segment in &components {
        if !is_valid_url_segment(segment) {
            return None;
        }
    }
    if file_name == "index.md" {
        if components.is_empty() {
            return Some("/".into());
        }
        return Some(format!("/{}/", components.join("/")));
    }

    let stem = file_name.strip_suffix(".md")?;
    if !is_valid_url_segment(stem) {
        return None;
    }
    components.push(stem);
    Some(format!("/{}", components.join("/")))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_url() {
        assert_eq!(path_to_url(Path::new("index.md")).as_deref(), Some("/"));
        assert_eq!(
            path_to_url(Path::new(".md/index.md")).as_deref(),
            Some("/.md/")
        );
        assert_eq!(
            path_to_url(Path::new("guides/getting-started.md")).as_deref(),
            Some("/guides/getting-started")
        );
        assert_eq!(
            path_to_url(Path::new("guides/index.md")).as_deref(),
            Some("/guides/")
        );
        assert_eq!(
            path_to_url(Path::new("guides/v1.2.md")).as_deref(),
            Some("/guides/v1.2")
        );
        assert_eq!(path_to_url(Path::new("../secret.md")), None);
        assert_eq!(path_to_url(Path::new("guides/../secret.md")), None);
    }
}
