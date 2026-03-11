use std::path::{Path, PathBuf};

use super::rel_page_path::RelPagePath;

/// Absolute Markdown path
pub(crate) struct AbsPagePath(PathBuf);

impl AbsPagePath {
    /// Converts a URL into an absolute Markdown path under the given root.
    pub(crate) fn from_url(root: &Path, url: &str) -> Option<Self> {
        RelPagePath::from_url(url).map(|rel| Self(root.join(rel.as_ref())))
    }
}

impl AsRef<Path> for AbsPagePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
