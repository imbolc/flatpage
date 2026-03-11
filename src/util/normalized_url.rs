use std::borrow::Cow;

use crate::path::is_valid_url_segment;

/// Canonical page URL.
pub(crate) struct NormalizedUrl<'a>(Cow<'a, str>);

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
            if !is_valid_url_segment(segment) {
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

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::NormalizedUrl;

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
}
