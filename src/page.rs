use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use crate::{
    Error, Result,
    markdown::{markdown, resolve_title},
    path::url_to_path,
};

#[derive(Debug, serde::Deserialize)]
struct Frontmatter<Extra = ()> {
    title: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    extra: Extra,
}

struct ParsedPage<'a, Extra = ()> {
    title: String,
    description: Option<String>,
    body: &'a str,
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
    pub fn by_url(root: impl Into<PathBuf>, url: &str) -> Result<Option<Self>> {
        let root = root.into();
        let path = match url_to_path(&root, url) {
            Some(path) => path,
            None => return Ok(None),
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

    /// [`FlatPage::body`] rendered to html
    pub fn html(&self) -> String {
        markdown(&self.body)
    }

    /// Parses a page from text
    fn from_content(content: &str) -> std::result::Result<Self, markdown_frontmatter::Error> {
        let ParsedPage {
            title,
            description,
            body,
            extra,
        } = parse_page_content(content)?;
        Ok(Self {
            title,
            description,
            body: body.to_string(),
            extra,
        })
    }
}

/// Parses frontmatter and returns the remaining Markdown body unchanged.
fn frontmatter_and_body<Extra: DeserializeOwned>(
    content: &str,
) -> std::result::Result<(Frontmatter<Extra>, &str), markdown_frontmatter::Error> {
    markdown_frontmatter::parse::<Frontmatter<Extra>>(content)
}

/// Parses a page body into resolved title, description, body, and extra fields.
fn parse_page_content<Extra: DeserializeOwned>(
    content: &str,
) -> std::result::Result<ParsedPage<'_, Extra>, markdown_frontmatter::Error> {
    let (
        Frontmatter {
            title,
            description,
            extra,
        },
        body,
    ) = frontmatter_and_body(content)?;
    Ok(ParsedPage {
        title: resolve_title(title, body),
        description,
        body,
        extra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{TestDir, write_page};

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
