use std::path::{Component, Path, PathBuf};

const ALLOWED_IN_URL_SEGMENT: &str = "_-.";

/// Tries to normalize the URL.
pub(crate) fn normalize_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }

    if url == "/" {
        return Some("/".into());
    }

    if !url.starts_with('/') {
        return None;
    }

    let trailing_slash = url.ends_with('/');
    let url = url.strip_prefix('/').unwrap_or(url);
    let url = if trailing_slash {
        url.strip_suffix('/').unwrap_or(url)
    } else {
        url
    };

    if url.is_empty() || url.contains("//") {
        return None;
    }

    let mut normalized = String::from("/");
    for (index, segment) in url.split('/').enumerate() {
        if !is_valid_url_segment(segment) {
            return None;
        }
        if index > 0 {
            normalized.push('/');
        }
        normalized.push_str(segment);
    }
    if trailing_slash {
        normalized.push('/');
    }
    Some(normalized)
}

fn is_valid_url_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ALLOWED_IN_URL_SEGMENT.contains(c))
}

/// Tries to convert the URL into a relative Markdown path.
pub(crate) fn url_to_path(url: &str) -> Option<PathBuf> {
    let url = normalize_url(url)?;
    Some(normalized_url_to_path(&url))
}

pub(crate) fn page_path(root: &Path, url: &str) -> Option<PathBuf> {
    let relative_path = url_to_path(url)?;
    let mut path = root.to_path_buf();
    path.push(relative_path);
    Some(path)
}

pub(crate) fn page_path_from_normalized_url(root: &Path, url: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    path.push(normalized_url_to_path(url));
    path
}

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

fn normalized_url_to_path(url: &str) -> PathBuf {
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
    fn test_url_to_path() {
        assert_eq!(url_to_path(""), None);
        assert_eq!(url_to_path("#"), None);
        assert_eq!(url_to_path("foo"), None);
        assert_eq!(url_to_path("ы"), None);
        assert_eq!(url_to_path("//foo"), None);
        assert_eq!(url_to_path("foo//"), None);
        assert_eq!(url_to_path("/../secret"), None);
        assert_eq!(url_to_path("/foo//bar"), None);
        assert_eq!(url_to_path("/").unwrap(), PathBuf::from("index.md"));
        assert_eq!(
            url_to_path("/foo-bar/baz").unwrap(),
            PathBuf::from("foo-bar/baz.md")
        );
        assert_eq!(
            url_to_path("/foo-bar/baz/").unwrap(),
            PathBuf::from("foo-bar/baz/index.md")
        );
        assert_eq!(
            url_to_path("/foo.bar").unwrap(),
            PathBuf::from("foo.bar.md")
        );
    }

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

    #[test]
    fn test_page_path_from_normalized_url() {
        assert_eq!(
            page_path_from_normalized_url(Path::new("pages"), "/"),
            PathBuf::from("pages/index.md")
        );
        assert_eq!(
            page_path_from_normalized_url(Path::new("pages"), "/guides/install"),
            PathBuf::from("pages/guides/install.md")
        );
        assert_eq!(
            page_path_from_normalized_url(Path::new("pages"), "/guides/install/"),
            PathBuf::from("pages/guides/install/index.md")
        );
    }

    #[test]
    fn test_normalize_url_rejects_empty_segments() {
        assert_eq!(normalize_url("foo"), None);
        assert_eq!(normalize_url("//foo"), None);
        assert_eq!(normalize_url("foo//"), None);
        assert_eq!(normalize_url("foo//bar"), None);
        assert_eq!(normalize_url("////"), None);
        assert_eq!(normalize_url("/foo/").as_deref(), Some("/foo/"));
        assert_eq!(normalize_url("/foo").as_deref(), Some("/foo"));
    }
}
