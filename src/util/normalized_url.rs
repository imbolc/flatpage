//! Normalized page URL parsing and conversions.

use std::borrow::{Borrow, Cow};

use super::{RelPagePath, is_valid_page_segment, rel_page_path::PageShape};

/// Canonical page URL.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NormalizedUrl<'a>(Cow<'a, str>);

impl<'a> TryFrom<&'a str> for NormalizedUrl<'a> {
    type Error = ();

    /// Validates and wraps a raw page URL string.
    fn try_from(url: &'a str) -> Result<Self, Self::Error> {
        let original_url = url;
        if url.is_empty() {
            return Err(());
        }

        if url == "/" {
            return Ok(Self(Cow::Borrowed("/")));
        }

        if !url.starts_with('/') {
            return Err(());
        }

        let url = url.strip_prefix('/').unwrap_or(url);
        let url = if url.ends_with('/') {
            url.strip_suffix('/').unwrap_or(url)
        } else {
            url
        };

        if url.is_empty() {
            return Err(());
        }

        for segment in url.split('/') {
            if !is_valid_page_segment(segment) {
                return Err(());
            }
        }

        Ok(Self(Cow::Borrowed(original_url)))
    }
}

impl AsRef<str> for NormalizedUrl<'_> {
    /// Returns the normalized URL as a string slice.
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for NormalizedUrl<'_> {
    /// Borrows the normalized URL as a string slice.
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<PageShape<'_>> for NormalizedUrl<'static> {
    /// Converts a validated page shape into a normalized URL.
    fn from(page_shape: PageShape<'_>) -> Self {
        match page_shape {
            PageShape::Root => Self(Cow::Borrowed("/")),
            PageShape::File(segments) => Self(Cow::Owned(format!("/{}", segments.join("/")))),
            PageShape::Index(segments) => Self(Cow::Owned(format!("/{}/", segments.join("/")))),
        }
    }
}

impl TryFrom<&RelPagePath> for NormalizedUrl<'static> {
    type Error = ();

    /// Converts a validated relative page path into a normalized URL.
    fn try_from(path: &RelPagePath) -> Result<Self, Self::Error> {
        Ok(PageShape::try_from(path)?.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, path::Path};

    use super::NormalizedUrl;
    use crate::util::RelPagePath;

    #[test]
    fn test_try_from_rejects_empty_segments() {
        assert!(NormalizedUrl::try_from("foo").is_err());
        assert!(NormalizedUrl::try_from("//foo").is_err());
        assert!(NormalizedUrl::try_from("foo//").is_err());
        assert!(NormalizedUrl::try_from("foo//bar").is_err());
        assert!(NormalizedUrl::try_from("////").is_err());
        assert_eq!(NormalizedUrl::try_from("/foo/").unwrap().as_ref(), "/foo/");
        assert_eq!(NormalizedUrl::try_from("/foo").unwrap().as_ref(), "/foo");
    }

    #[test]
    fn test_try_from_borrows_normalized_input() {
        let root = NormalizedUrl::try_from("/").unwrap();
        assert!(matches!(root.0, Cow::Borrowed("/")));

        let page = NormalizedUrl::try_from("/foo/bar").unwrap();
        assert!(matches!(page.0, Cow::Borrowed("/foo/bar")));

        let index = NormalizedUrl::try_from("/foo/bar/").unwrap();
        assert!(matches!(index.0, Cow::Borrowed("/foo/bar/")));
    }

    #[test]
    fn test_try_from_rel_page_path() {
        let root = NormalizedUrl::try_from(&RelPagePath::try_from(Path::new("index.md")).unwrap())
            .unwrap();
        assert_eq!(root.as_ref(), "/");

        let index =
            NormalizedUrl::try_from(&RelPagePath::try_from(Path::new(".md/index.md")).unwrap())
                .unwrap();
        assert_eq!(index.as_ref(), "/.md/");

        let page = NormalizedUrl::try_from(
            &RelPagePath::try_from(Path::new("guides/getting-started.md")).unwrap(),
        )
        .unwrap();
        assert_eq!(page.as_ref(), "/guides/getting-started");

        let index =
            NormalizedUrl::try_from(&RelPagePath::try_from(Path::new("guides/index.md")).unwrap())
                .unwrap();
        assert_eq!(index.as_ref(), "/guides/");

        let dotted =
            NormalizedUrl::try_from(&RelPagePath::try_from(Path::new("guides/v1.2.md")).unwrap())
                .unwrap();
        assert_eq!(dotted.as_ref(), "/guides/v1.2");
    }
}
