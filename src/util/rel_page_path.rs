//! Relative page-path conversions.

use std::path::{Path, PathBuf};

use super::{NormalizedUrl, page_location::PageLocation};

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl From<&NormalizedUrl<'_>> for RelPagePath {
    /// Converts a normalized URL into its relative Markdown path.
    fn from(url: &NormalizedUrl<'_>) -> Self {
        PageLocation::from(url).into()
    }
}

impl TryFrom<&Path> for RelPagePath {
    type Error = ();

    /// Validates and wraps a relative Markdown path.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        PageLocation::try_from(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl AsRef<Path> for RelPagePath {
    /// Returns the wrapped relative path.
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl From<PageLocation<'_>> for RelPagePath {
    /// Builds a relative Markdown path from a classified page location.
    fn from(page_location: PageLocation<'_>) -> Self {
        match page_location {
            PageLocation::Root => Self(PathBuf::from("index.md")),
            PageLocation::File {
                path: parent_segments,
                name,
            } => {
                let mut path = PathBuf::new();
                for segment in parent_segments {
                    path.push(segment);
                }
                path.push(format!("{name}.md"));
                Self(path)
            }
            PageLocation::Index(segments) => {
                let mut path = PathBuf::new();
                for segment in segments {
                    path.push(segment);
                }
                path.push("index.md");
                Self(path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::RelPagePath;
    use crate::util::NormalizedUrl;

    #[test]
    fn test_from_normalized_url() {
        assert_eq!(
            RelPagePath::from(&NormalizedUrl::try_from("/").unwrap()).as_ref(),
            Path::new("index.md")
        );
        assert_eq!(
            RelPagePath::from(&NormalizedUrl::try_from("/install").unwrap()).as_ref(),
            Path::new("install.md")
        );
        assert_eq!(
            RelPagePath::from(&NormalizedUrl::try_from("/guides/getting-started").unwrap())
                .as_ref(),
            Path::new("guides/getting-started.md")
        );
        assert_eq!(
            RelPagePath::from(&NormalizedUrl::try_from("/guides/").unwrap()).as_ref(),
            Path::new("guides/index.md")
        );
    }
}
