use std::path::{Path, PathBuf};

use crate::path::url_to_rel_path;

/// Absolute Markdown path
pub(crate) struct AbsPagePath(PathBuf);

impl AbsPagePath {
    /// Converts a URL into an absolute Markdown path under the given root.
    pub(crate) fn from_url(root: &Path, url: &str) -> Option<Self> {
        url_to_rel_path(url).map(|rel| Self(root.join(rel)))
    }
}

impl AsRef<Path> for AbsPagePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
