//! Absolute page-path conversions.

use std::path::{Path, PathBuf};

use super::{NormalizedUrl, RelPagePath};

/// Absolute Markdown path
pub(crate) struct AbsPagePath(PathBuf);

impl AbsPagePath {
    /// Converts a URL into an absolute Markdown path under the given root.
    pub(crate) fn from_raw_url(root: &Path, url: &str) -> Option<Self> {
        let url = NormalizedUrl::try_from(url).ok()?;
        Some(Self::from_normalized_url(root, &url))
    }

    /// Converts a normalized URL into an absolute Markdown path under the given
    /// root.
    pub(crate) fn from_normalized_url(root: &Path, url: &NormalizedUrl<'_>) -> Self {
        let rel = RelPagePath::from(url);
        Self(root.join(rel.as_ref()))
    }
}

impl AsRef<Path> for AbsPagePath {
    /// Returns the wrapped absolute path.
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::AbsPagePath;
    use crate::util::NormalizedUrl;

    #[test]
    fn test_from_normalized_url() {
        let root = Path::new("pages");

        assert_eq!(
            AbsPagePath::from_normalized_url(root, &NormalizedUrl::try_from("/").unwrap()).as_ref(),
            Path::new("pages/index.md")
        );
        assert_eq!(
            AbsPagePath::from_normalized_url(
                root,
                &NormalizedUrl::try_from("/guides/install").unwrap()
            )
            .as_ref(),
            Path::new("pages/guides/install.md")
        );
        assert_eq!(
            AbsPagePath::from_normalized_url(
                root,
                &NormalizedUrl::try_from("/guides/install/").unwrap()
            )
            .as_ref(),
            Path::new("pages/guides/install/index.md")
        );
    }

    #[test]
    fn test_from_raw_url() {
        assert_eq!(
            AbsPagePath::from_raw_url(Path::new("pages"), "").map(|p| p.0),
            None
        );
        assert_eq!(
            AbsPagePath::from_raw_url(Path::new("pages"), "/guides/install").map(|p| p.0),
            Some(PathBuf::from("pages/guides/install.md"))
        );
    }
}
