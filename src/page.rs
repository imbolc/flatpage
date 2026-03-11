//! Page parsing and filesystem loading.

use std::{fs, io, path::Path};

use serde::de::DeserializeOwned;

use crate::{
    Error, Result,
    markdown::{render_markdown, title_from_markdown},
    util::AbsPagePath,
};

/// Parsed frontmatter fields before they are assembled into a [`FlatPage`].
#[derive(Debug, serde::Deserialize)]
struct Frontmatter<Extra = ()> {
    /// Optional explicit page title from frontmatter.
    title: Option<String>,
    /// Optional description from frontmatter.
    description: Option<String>,
    /// Additional caller-defined frontmatter fields.
    #[serde(flatten)]
    extra: Extra,
}

/// Flat page
#[derive(Debug)]
pub struct FlatPage<Extra = ()> {
    /// Page title
    pub title: String,
    /// Description - for HTML meta description, `og:description`, etc
    pub description: Option<String>,
    /// Raw markdown version of the body
    pub body: String,
    /// Extra frontmatter fields (except `title` and `description`)
    pub extra: Extra,
}

impl<Extra: DeserializeOwned> FlatPage<Extra> {
    /// Returns a page by its URL.
    ///
    /// Trailing slashes are significant: `/foo` looks up `foo.md`, while
    /// `/foo/` looks up `foo/index.md`.
    ///
    /// Returns `Ok(None)` for invalid URLs and missing pages. Returns `Err` for
    /// I/O failures and frontmatter parsing errors.
    pub fn by_url(root: impl AsRef<Path>, url: &str) -> Result<Option<Self>> {
        let Some(path) = AbsPagePath::from_raw_url(root.as_ref(), url) else {
            return Ok(None);
        };
        Self::by_path(&path)
    }

    /// Returns a page by its file path.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    pub fn by_path(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = path.as_ref();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(Error::ReadFile {
                    source: e,
                    path: path.to_path_buf(),
                });
            }
        };
        Self::from_content(&content)
            .map(Some)
            .map_err(|e| Error::ParseFrontmatter {
                source: e,
                path: path.to_path_buf(),
            })
    }

    /// [`FlatPage::body`] rendered to HTML
    pub fn html(&self) -> String {
        render_markdown(&self.body)
    }

    /// Parses a page from text
    fn from_content(content: &str) -> std::result::Result<Self, markdown_frontmatter::Error> {
        let (
            Frontmatter {
                title,
                description,
                extra,
            },
            body,
        ) = markdown_frontmatter::parse::<Frontmatter<Extra>>(content)?;
        Ok(Self {
            title: title.unwrap_or_else(|| title_from_markdown(body).to_string()),
            description,
            body: body.to_string(),
            extra,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Error,
        test_helpers::{TestDir, write_page},
    };

    fn assert_parse_frontmatter_error(content: &str) {
        let root = TestDir::new();
        let path = root.path().join("broken.md");
        write_page(root.path(), "broken.md", content);

        assert!(
            matches!(FlatPage::<()>::by_path(&path), Err(Error::ParseFrontmatter { path: error_path, .. }) if error_path == path)
        );
    }

    #[test]
    fn flatpage_title() {
        let page = FlatPage::<()>::from_content("# Foo").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.body, "# Foo");

        #[cfg(feature = "yaml")]
        assert_eq!(
            FlatPage::<()>::from_content("---\ntitle: Bar\n---\n# Foo")
                .unwrap()
                .title,
            "Bar"
        );
    }

    #[test]
    fn flatpage_description() {
        assert_eq!(FlatPage::<()>::from_content("").unwrap().description, None);

        #[cfg(feature = "yaml")]
        assert_eq!(
            FlatPage::<()>::from_content("---\ndescription: Bar\n---")
                .unwrap()
                .description
                .as_deref(),
            Some("Bar")
        );
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn extra_fields() {
        #[derive(Debug, serde::Deserialize)]
        struct Extra {
            slug: String,
        }
        assert!(FlatPage::<Extra>::from_content("").is_err());

        assert_eq!(
            FlatPage::<Extra>::from_content("---\nslug: foo\n---")
                .unwrap()
                .extra
                .slug,
            "foo"
        );
    }

    #[test]
    fn markdown_rendering() {
        let page = FlatPage::<()>::from_content("# Foo\nBar").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.body, "# Foo\nBar");
        assert_eq!(page.html(), "<h1>Foo</h1>\n<p>Bar</p>\n");

        #[cfg(feature = "yaml")]
        {
            let page = FlatPage::<()>::from_content("---\ndescription: Bar\n---\n# Foo").unwrap();
            assert_eq!(page.title, "Foo");
            assert_eq!(page.description.as_deref().unwrap(), "Bar");
            assert_eq!(page.body, "# Foo");
            assert_eq!(page.html(), "<h1>Foo</h1>\n");

            let page =
                FlatPage::<()>::from_content("---\ntitle: Foo\ndescription: Bar\n---").unwrap();
            assert_eq!(page.title, "Foo");
            assert_eq!(page.description.as_deref().unwrap(), "Bar");
            assert_eq!(page.body, "");
            assert_eq!(page.html(), "");
        }
    }

    #[test]
    fn flatpage_by_url_reads_nested_paths() {
        let root = TestDir::new();
        write_page(root.path(), "guides/rust/index.md", "# Rust Guide");
        write_page(root.path(), "guides/install.md", "# Install");
        write_page(root.path(), "guides/v1.2.md", "# Versioned Guide");

        let index = FlatPage::<()>::by_url(root.path(), "/guides/rust/")
            .unwrap()
            .unwrap();
        assert_eq!(index.title, "Rust Guide");

        let page = FlatPage::<()>::by_url(root.path(), "/guides/install")
            .unwrap()
            .unwrap();
        assert_eq!(page.title, "Install");

        let dotted = FlatPage::<()>::by_url(root.path(), "/guides/v1.2")
            .unwrap()
            .unwrap();
        assert_eq!(dotted.title, "Versioned Guide");

        assert!(
            FlatPage::<()>::by_url(root.path(), "guides/install")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn flatpage_by_path_returns_none_for_missing_file() {
        let root = TestDir::new();
        let path = root.path().join("missing.md");

        assert!(FlatPage::<()>::by_path(&path).unwrap().is_none());
    }

    #[test]
    fn flatpage_by_path_reports_read_file_error() {
        let root = TestDir::new();
        let path = root.path().join("guides");
        std::fs::create_dir(&path).unwrap();

        assert!(
            matches!(FlatPage::<()>::by_path(&path), Err(Error::ReadFile { path: error_path, .. }) if error_path == path)
        );
    }

    #[cfg(feature = "json")]
    #[test]
    fn flatpage_by_path_reports_json_frontmatter_error() {
        assert_parse_frontmatter_error("{\n  \"title\": \n}\n# Foo");
    }

    #[cfg(feature = "toml")]
    #[test]
    fn flatpage_by_path_reports_toml_frontmatter_error() {
        assert_parse_frontmatter_error("+++\ntitle = \n+++\n# Foo");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn flatpage_by_path_reports_yaml_frontmatter_error() {
        assert_parse_frontmatter_error("---\ntitle: [\n---\n# Foo");
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_frontmatter() {
        let page = FlatPage::<()>::from_content("{\n  \"title\": \"Foo\"\n}\n# Bar").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.body, "# Bar");
    }

    #[cfg(feature = "toml")]
    #[test]
    fn toml_frontmatter() {
        let page = FlatPage::<()>::from_content("+++\ntitle = \"Foo\"\n+++\n# Bar").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.body, "# Bar");
    }
}
