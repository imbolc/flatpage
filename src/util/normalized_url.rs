use std::{
    borrow::{Borrow, Cow},
    path::Component,
};

use super::{RelPagePath, is_valid_page_segment};

/// Canonical page URL.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NormalizedUrl<'a>(Cow<'a, str>);

impl<'a> TryFrom<&'a str> for NormalizedUrl<'a> {
    type Error = ();

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

        let trailing_slash = url.ends_with('/');
        let url = url.strip_prefix('/').unwrap_or(url);
        let url = if trailing_slash {
            url.strip_suffix('/').unwrap_or(url)
        } else {
            url
        };

        if url.is_empty() || url.contains("//") {
            return Err(());
        }

        let mut normalized = String::from("/");
        for (index, segment) in url.split('/').enumerate() {
            if !is_valid_page_segment(segment) {
                return Err(());
            }
            if index > 0 {
                normalized.push('/');
            }
            normalized.push_str(segment);
        }
        if trailing_slash {
            normalized.push('/');
        }

        if normalized == original_url {
            return Ok(Self(Cow::Borrowed(original_url)));
        }

        Ok(Self(Cow::Owned(normalized)))
    }
}

impl AsRef<str> for NormalizedUrl<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for NormalizedUrl<'_> {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<&RelPagePath> for NormalizedUrl<'static> {
    type Error = ();

    fn try_from(path: &RelPagePath) -> Result<Self, Self::Error> {
        let mut components = Vec::new();
        for component in path.as_ref().components() {
            let Component::Normal(segment) = component else {
                return Err(());
            };
            let segment = segment.to_str().ok_or(())?;
            components.push(segment);
        }

        let file_name = components.pop().ok_or(())?;
        if file_name == "index.md" {
            if components.is_empty() {
                return Ok(Self(Cow::Borrowed("/")));
            }
            return Ok(Self(Cow::Owned(format!("/{}/", components.join("/")))));
        }

        let stem = file_name.strip_suffix(".md").ok_or(())?;
        components.push(stem);
        Ok(Self(Cow::Owned(format!("/{}", components.join("/")))))
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
