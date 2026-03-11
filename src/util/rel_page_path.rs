//! Relative page-path conversions and shared page-shape mapping.

use std::path::{Component, Path, PathBuf};

use super::{NormalizedUrl, is_valid_page_segment};

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

/// Shared page-shape representation used by URL and path conversions.
pub(super) enum PageShape<'a> {
    /// The root page (`/` or `index.md`).
    Root,
    /// A leaf page such as `/guides/install`.
    File(Vec<&'a str>),
    /// A directory index page such as `/guides/`.
    Index(Vec<&'a str>),
}

impl<'a> From<&'a NormalizedUrl<'_>> for PageShape<'a> {
    /// Splits a normalized URL into its logical page shape.
    fn from(url: &'a NormalizedUrl<'_>) -> Self {
        let url = url.as_ref();
        if url == "/" {
            return Self::Root;
        }

        let segments = url.trim_matches('/').split('/').collect();
        if url.ends_with('/') {
            Self::Index(segments)
        } else {
            Self::File(segments)
        }
    }
}

impl<'a> TryFrom<&'a Path> for PageShape<'a> {
    type Error = ();

    /// Parses a relative Markdown path into its logical page shape.
    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        let mut components = Vec::new();
        for component in path.components() {
            let Component::Normal(segment) = component else {
                return Err(());
            };
            components.push(segment.to_str().ok_or(())?);
        }

        let file_name = components.pop().ok_or(())?;
        for segment in &components {
            if !is_valid_page_segment(segment) {
                return Err(());
            }
        }

        if file_name == "index.md" {
            return if components.is_empty() {
                Ok(Self::Root)
            } else {
                Ok(Self::Index(components))
            };
        }

        let stem = file_name.strip_suffix(".md").ok_or(())?;
        if !is_valid_page_segment(stem) {
            return Err(());
        }
        components.push(stem);
        Ok(Self::File(components))
    }
}

impl<'a> TryFrom<&'a RelPagePath> for PageShape<'a> {
    type Error = ();

    /// Reparses a relative page path into its logical page shape.
    fn try_from(path: &'a RelPagePath) -> Result<Self, Self::Error> {
        Self::try_from(path.as_ref())
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
