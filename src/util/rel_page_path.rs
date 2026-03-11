//! Relative page-path conversions and shared page-shape mapping.

use std::path::{Component, Path, PathBuf};

use super::{NormalizedUrl, is_valid_page_segment};

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl From<&NormalizedUrl<'_>> for RelPagePath {
    /// Converts a normalized URL into its relative Markdown path.
    fn from(url: &NormalizedUrl<'_>) -> Self {
        PageShape::from_normalized_url(url.as_ref()).into()
    }
}

impl TryFrom<&Path> for RelPagePath {
    type Error = ();

    /// Validates and wraps a relative Markdown path.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        PageShape::try_from_rel_path(path)?;
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

impl<'a> PageShape<'a> {
    /// Splits a normalized URL into its logical page shape.
    pub(super) fn from_normalized_url(url: &'a str) -> Self {
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

    /// Validates and classifies a relative Markdown path.
    pub(super) fn try_from_rel_path(path: &'a Path) -> Result<Self, ()> {
        let mut components = Vec::new();
        for component in path.components() {
            let Component::Normal(segment) = component else {
                return Err(());
            };
            let segment = segment.to_str().ok_or(())?;
            components.push(segment);
        }

        let file_name = components.pop().ok_or(())?;
        for segment in &components {
            if !is_valid_page_segment(segment) {
                return Err(());
            }
        }

        if file_name == "index.md" {
            if components.is_empty() {
                return Ok(Self::Root);
            }
            return Ok(Self::Index(components));
        }

        let stem = file_name.strip_suffix(".md").ok_or(())?;
        if !is_valid_page_segment(stem) {
            return Err(());
        }
        components.push(stem);
        Ok(Self::File(components))
    }
}

impl From<PageShape<'_>> for RelPagePath {
    /// Builds a relative Markdown path from a classified page shape.
    fn from(page_shape: PageShape<'_>) -> Self {
        let path = match page_shape {
            PageShape::Root => PathBuf::from("index.md"),
            PageShape::File(segments) => {
                let mut path = PathBuf::new();
                let mut segments = segments.into_iter();
                let last_segment = segments.next_back().unwrap_or_default();
                for segment in segments {
                    path.push(segment);
                }
                path.push(format!("{last_segment}.md"));
                path
            }
            PageShape::Index(segments) => {
                let mut path = PathBuf::new();
                for segment in segments {
                    path.push(segment);
                }
                path.push("index.md");
                path
            }
        };

        Self(path)
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
