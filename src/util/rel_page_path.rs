//! Relative page-path conversions.

use std::path::{Path, PathBuf};

use super::{NormalizedUrl, page_shape::PageShape};

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl From<&NormalizedUrl<'_>> for RelPagePath {
    /// Converts a normalized URL into its relative Markdown path.
    fn from(url: &NormalizedUrl<'_>) -> Self {
        PageShape::from(url).into()
    }
}

impl TryFrom<&Path> for RelPagePath {
    type Error = ();

    /// Validates and wraps a relative Markdown path.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        PageShape::try_from(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl AsRef<Path> for RelPagePath {
    /// Returns the wrapped relative path.
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl From<PageShape<'_>> for RelPagePath {
    /// Builds a relative Markdown path from a classified page shape.
    fn from(page_shape: PageShape<'_>) -> Self {
        match page_shape {
            PageShape::Root => Self(PathBuf::from("index.md")),
            PageShape::File(segments) => {
                let mut path = PathBuf::new();
                let mut segments = segments.into_iter();
                let last_segment = segments.next_back().unwrap_or_default();
                for segment in segments {
                    path.push(segment);
                }
                path.push(format!("{last_segment}.md"));
                Self(path)
            }
            PageShape::Index(segments) => {
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
