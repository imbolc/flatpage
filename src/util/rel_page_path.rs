use std::path::{Component, Path, PathBuf};

use super::{NormalizedUrl, is_valid_page_segment};

pub(super) enum PageShape<'a> {
    Root,
    File(Vec<&'a str>),
    Index(Vec<&'a str>),
}

impl<'a> PageShape<'a> {
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

    fn to_rel_path_buf(&self) -> PathBuf {
        match self {
            Self::Root => PathBuf::from("index.md"),
            Self::File(segments) => {
                let mut path = PathBuf::new();
                let mut segments = segments.iter().copied();
                let last_segment = segments.next_back().unwrap_or_default();
                for segment in segments {
                    path.push(segment);
                }
                path.push(format!("{last_segment}.md"));
                path
            }
            Self::Index(segments) => {
                let mut path = PathBuf::new();
                for segment in segments {
                    path.push(segment);
                }
                path.push("index.md");
                path
            }
        }
    }
}

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl From<&NormalizedUrl<'_>> for RelPagePath {
    fn from(url: &NormalizedUrl<'_>) -> Self {
        Self(PageShape::from_normalized_url(url.as_ref()).to_rel_path_buf())
    }
}

impl TryFrom<&Path> for RelPagePath {
    type Error = ();

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        PageShape::try_from_rel_path(path)?;
        Ok(Self(path.to_path_buf()))
    }
}

impl AsRef<Path> for RelPagePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
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
