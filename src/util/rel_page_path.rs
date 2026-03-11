use std::path::{Path, PathBuf};

use crate::path::{normalize_url, normalized_url_to_path};

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl RelPagePath {
    /// Converts a URL into a relative Markdown path.
    pub(crate) fn from_url(url: &str) -> Option<Self> {
        let url = normalize_url(url)?;
        Some(Self(normalized_url_to_path(&url)))
    }
}

impl AsRef<Path> for RelPagePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::RelPagePath;

    #[test]
    fn test_from_url() {
        assert!(RelPagePath::from_url("").is_none());
        assert!(RelPagePath::from_url("#").is_none());
        assert!(RelPagePath::from_url("foo").is_none());
        assert!(RelPagePath::from_url("ы").is_none());
        assert!(RelPagePath::from_url("//foo").is_none());
        assert!(RelPagePath::from_url("foo//").is_none());
        assert!(RelPagePath::from_url("/../secret").is_none());
        assert!(RelPagePath::from_url("/foo//bar").is_none());
        assert_eq!(
            RelPagePath::from_url("/").unwrap().as_ref(),
            Path::new("index.md")
        );
        assert_eq!(
            RelPagePath::from_url("/foo-bar/baz").unwrap().as_ref(),
            Path::new("foo-bar/baz.md")
        );
        assert_eq!(
            RelPagePath::from_url("/foo-bar/baz/").unwrap().as_ref(),
            Path::new("foo-bar/baz/index.md")
        );
        assert_eq!(
            RelPagePath::from_url("/foo.bar").unwrap().as_ref(),
            Path::new("foo.bar.md")
        );
    }
}
