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
