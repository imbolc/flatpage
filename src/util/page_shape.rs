//! Shared page-shape mapping used by URL and path conversions.

use std::path::{Component, Path};

use super::{NormalizedUrl, RelPagePath, is_valid_page_segment};

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

    /// Parses the wrapped relative page path into its logical page shape.
    fn try_from(path: &'a RelPagePath) -> Result<Self, Self::Error> {
        Self::try_from(path.as_ref())
    }
}
