use std::path::{Component, Path, PathBuf};

use super::{NormalizedUrl, is_valid_page_segment};

/// Relative Markdown path
pub(crate) struct RelPagePath(PathBuf);

impl From<&NormalizedUrl<'_>> for RelPagePath {
    fn from(url: &NormalizedUrl<'_>) -> Self {
        if url.as_ref() == "/" {
            return Self(PathBuf::from("index.md"));
        }

        let mut path = PathBuf::new();
        let mut segments = url.as_ref().trim_matches('/').split('/');
        let last_segment = segments.next_back().unwrap_or_default();
        for segment in segments {
            path.push(segment);
        }
        if url.as_ref().ends_with('/') {
            path.push(last_segment);
            path.push("index.md");
        } else {
            path.push(format!("{last_segment}.md"));
        }

        Self(path)
    }
}

impl TryFrom<&Path> for RelPagePath {
    type Error = ();

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
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
            return Ok(Self(path.to_path_buf()));
        }

        let stem = file_name.strip_suffix(".md").ok_or(())?;
        if !is_valid_page_segment(stem) {
            return Err(());
        }

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
