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

    #[test]
    fn test_try_from_path() {
        assert_eq!(
            RelPagePath::try_from(Path::new("index.md"))
                .unwrap()
                .as_ref(),
            Path::new("index.md")
        );
        assert_eq!(
            RelPagePath::try_from(Path::new(".md/index.md"))
                .unwrap()
                .as_ref(),
            Path::new(".md/index.md")
        );
        assert_eq!(
            RelPagePath::try_from(Path::new("guides/getting-started.md"))
                .unwrap()
                .as_ref(),
            Path::new("guides/getting-started.md")
        );
        assert_eq!(
            RelPagePath::try_from(Path::new("guides/index.md"))
                .unwrap()
                .as_ref(),
            Path::new("guides/index.md")
        );
        assert_eq!(
            RelPagePath::try_from(Path::new("guides/v1.2.md"))
                .unwrap()
                .as_ref(),
            Path::new("guides/v1.2.md")
        );
        assert!(RelPagePath::try_from(Path::new("../secret.md")).is_err());
        assert!(RelPagePath::try_from(Path::new("guides/../secret.md")).is_err());
        assert!(RelPagePath::try_from(Path::new("guides/readme.txt")).is_err());
    }
}
