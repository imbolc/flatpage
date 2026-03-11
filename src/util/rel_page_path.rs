use std::path::{Path, PathBuf};

use crate::{path::normalized_url_to_path, util::NormalizedUrl};

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl RelPagePath {
    /// Converts a URL into a relative Markdown path.
    pub(crate) fn from_url(url: &str) -> Option<Self> {
        let url = NormalizedUrl::try_from(url).ok()?;
        Some(Self::from_normalized_url(&url))
    }

    /// Converts a normalized URL into a relative Markdown path.
    pub(crate) fn from_normalized_url(url: &NormalizedUrl<'_>) -> Self {
        Self(normalized_url_to_path(url.as_ref()))
    }
}

impl AsRef<Path> for RelPagePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
