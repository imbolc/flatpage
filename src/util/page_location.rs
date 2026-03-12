//! Shared page-location mapping used by URL and path conversions.

use std::path::{Component, Path};

use super::{NormalizedUrl, RelPagePath, is_valid_page_segment};

/// Shared page-location representation used by URL and path conversions.
pub(super) enum PageLocation<'a> {
    /// The root page (`/` or `index.md`).
    Root,
    /// A leaf page such as `/guides/install`.
    File {
        /// Parent path segments such as `["guides"]`.
        path: Vec<&'a str>,
        /// The last page segment without the `.md` suffix, such as `"install"`.
        name: &'a str,
    },
    /// A directory index page such as `/guides/`.
    Index(Vec<&'a str>),
}

impl<'a> From<&'a NormalizedUrl<'_>> for PageLocation<'a> {
    /// Splits a normalized URL into its logical page location.
    fn from(url: &'a NormalizedUrl<'_>) -> Self {
        let url = url.as_ref();
        if url == "/" {
            return Self::Root;
        }

        let trimmed = url.trim_matches('/');
        if url.ends_with('/') {
            Self::Index(trimmed.split('/').collect())
        } else if let Some((path, name)) = trimmed.rsplit_once('/') {
            Self::File {
                path: path.split('/').collect(),
                name,
            }
        } else {
            Self::File {
                path: Vec::new(),
                name: trimmed,
            }
        }
    }
}

impl<'a> TryFrom<&'a Path> for PageLocation<'a> {
    type Error = ();

    /// Parses a relative Markdown path into its logical page location.
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
        Ok(Self::File {
            path: components,
            name: stem,
        })
    }
}

impl<'a> TryFrom<&'a RelPagePath> for PageLocation<'a> {
    type Error = ();

    /// Parses the wrapped relative page path into its logical page location.
    fn try_from(path: &'a RelPagePath) -> Result<Self, Self::Error> {
        Self::try_from(path.as_ref())
    }
}
