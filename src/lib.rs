#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use frontmatter::Frontmatter;
use serde::de::DeserializeOwned;
pub use store::{FlatPageMeta, FlatPageStore};

const ALLOWED_IN_URL: &str = "/_-";

/// The crates error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Broken frontmatter
    #[error("broken frontmatter in '{1}'")]
    ParseFrontmatter(#[source] serde_yml::Error, String),
    /// Can't read folder
    #[error("readdir '{1}'")]
    ReadDir(#[source] io::Error, PathBuf),
    /// Can't read folder entry
    #[error("readdir entry")]
    DirEntry(#[source] io::Error),
}

/// The crates result type
pub type Result<T> = std::result::Result<T, Error>;

/// Flat page
/// The generic parameter `E` is used to define extra frontmatter fields
#[derive(Debug)]
pub struct FlatPage<E = ()> {
    /// Title - for html title tag, `og:title`, etc
    pub title: String,
    /// Description - for html meta description, `og:description`, etc
    pub description: Option<String>,
    /// Raw markdown version of the body
    pub body: String,
    /// Extra frontmatter fields (except of `title` and `description`)
    pub extra: E,
}

impl<E: DeserializeOwned> FlatPage<E> {
    /// Returns a page by its url
    pub fn by_url(root: impl Into<PathBuf>, url: &str) -> Result<Option<Self>> {
        let filename = match url_to_filename(url) {
            Some(f) => f,
            None => return Ok(None),
        };
        let mut path: PathBuf = root.into();
        path.push(&filename);
        Self::by_path(&path)
    }

    /// Returns a page by its file path
    pub fn by_path(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = path.as_ref();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };
        Self::from_content(&content)
            .map(Some)
            .map_err(|e| Error::ParseFrontmatter(e, path.display().to_string()))
    }

    /// [`FlatPage::body`] rendered to html
    pub fn html(&self) -> String {
        markdown(&self.body)
    }

    /// Parses a page from text
    fn from_content(content: &str) -> serde_yml::Result<Self> {
        let (
            Frontmatter {
                title,
                description,
                extra,
            },
            body,
        ) = Frontmatter::parse(content)?;
        let title = title.unwrap_or_else(|| title_from_markdown(body).to_string());
        Ok(Self {
            title,
            description,
            body: body.to_string(),
            extra,
        })
    }
}

/// Considers the first line to be the page title, removes markdown header
/// prefix `#`
fn title_from_markdown(body: &str) -> &str {
    body.lines()
        .next()
        .unwrap_or_default()
        .trim_start_matches('#')
        .trim()
}

/// Tries to convert the url into a filename
fn url_to_filename(url: &str) -> Option<String> {
    if url.is_empty() {
        None
    } else if url
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ALLOWED_IN_URL.contains(c))
    {
        Some(format!("{}.md", url.replace('/', "^")))
    } else {
        None
    }
}

fn markdown(text: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(text, options);
    let mut html = String::new();
    html::push_html(&mut html, parser);
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_to_filename() {
        assert_eq!(url_to_filename(""), None);
        assert_eq!(url_to_filename("#"), None);
        assert_eq!(url_to_filename("ы"), None);
        assert_eq!(
            url_to_filename("/foo-bar/baz/").unwrap(),
            "^foo-bar^baz^.md"
        );
    }

    #[test]
    fn test_title_from_markdown() {
        assert_eq!(title_from_markdown(""), "");
        assert_eq!(title_from_markdown("## foo\nbar"), "foo");
    }

    #[test]
    fn flatpage_title() {
        let page = FlatPage::<()>::from_content("# Foo").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.body, "# Foo");
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
        assert_eq!(
            FlatPage::<()>::from_content("---\ndescription: Bar\n---")
                .unwrap()
                .description
                .unwrap(),
            "Bar"
        );
    }

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
    fn docs_table() {
        let page = FlatPage::<()>::from_content("# Foo\nBar").unwrap();
        assert_eq!(page.title, "Foo");
        assert!(page.description.is_none());
        assert_eq!(page.body, "# Foo\nBar");
        assert_eq!(page.html(), "<h1>Foo</h1>\n<p>Bar</p>\n");

        let page = FlatPage::<()>::from_content("---\ndescription: Bar\n---\n# Foo").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.description.as_deref().unwrap(), "Bar");
        assert_eq!(page.body, "# Foo");
        assert_eq!(page.html(), "<h1>Foo</h1>\n");

        let page = FlatPage::<()>::from_content("---\ntitle: Foo\ndescription: Bar\n---").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.description.as_deref().unwrap(), "Bar");
        assert_eq!(page.body, "");
        assert_eq!(page.html(), "");
    }
}

mod frontmatter {

    use serde::{Deserialize, de::DeserializeOwned};

    const EMPTY_YAML: &str = "{}";

    /// Markdown frontmatter
    #[derive(Debug, Deserialize)]
    pub(crate) struct Frontmatter<E = ()> {
        pub title: Option<String>,
        pub description: Option<String>,
        #[serde(flatten)]
        pub extra: E,
    }

    impl<E: DeserializeOwned> Frontmatter<E> {
        /// Parses frontmatter from markdown string.
        /// Returns the frontmatter and the rest of the content (page body)
        pub(crate) fn parse(content: &str) -> serde_yml::Result<(Self, &str)> {
            let (matter, body) =
                split_frontmatter(content).unwrap_or_else(|| (EMPTY_YAML, content.trim()));
            serde_yml::from_str(matter).map(|m| (m, body))
        }
    }

    /// If frontmatter is found returns it and the rest of the body, `None`
    /// otherwise
    fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
        let content = content.trim_start();

        let (prefix, rest) = content.split_once("---\n")?;
        if !prefix.is_empty() {
            // content doesn't start with the delimiter
            return None;
        }

        let (matter, body) = rest.split_once("\n---")?;
        Some((matter, body.trim()))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn deserialize_empty_frontmatter() {
            let parsed: Frontmatter = serde_yml::from_str(EMPTY_YAML).unwrap();
            assert_eq!(parsed.title, None);
            assert_eq!(parsed.description, None);
        }

        #[test]
        fn deserialize_frontmatter_with_unknown_fields() {
            let yaml = "foo: 1\nbar: true";
            let parsed: Frontmatter = serde_yml::from_str(yaml).unwrap();
            assert_eq!(parsed.title, None);
            assert_eq!(parsed.description, None);
        }

        #[test]
        fn deserialize_frontmatter_with_only_title() {
            let yaml = "title: foo";
            let parsed: Frontmatter = serde_yml::from_str(yaml).unwrap();
            assert_eq!(parsed.title.unwrap(), "foo");
            assert_eq!(parsed.description, None);
        }

        #[test]
        fn deserialize_frontmatter_with_extra_fields() {
            #[derive(Debug, Deserialize)]
            struct Extra {
                slug: String,
                active: bool,
            }

            let yaml = "slug: foo\nactive: true";
            let parsed: Frontmatter<Extra> = serde_yml::from_str(yaml).unwrap();
            assert_eq!(parsed.title, None);
            assert_eq!(parsed.description, None);
            assert_eq!(parsed.extra.slug, "foo");
            assert!(parsed.extra.active);
        }

        #[test]
        fn split_frontmatter_empty_page() {
            assert_eq!(split_frontmatter(""), None)
        }

        #[test]
        fn split_frontmatter_no_opening_delimiter() {
            assert_eq!(split_frontmatter("foo"), None)
        }

        #[test]
        fn split_frontmatter_doesnt_start_with_delimiter() {
            assert_eq!(split_frontmatter("foo\n---not a frontmatter\n---"), None)
        }

        #[test]
        fn split_frontmatter_no_closing_delimiter() {
            assert_eq!(split_frontmatter("---\nnot a frontmatter"), None)
        }

        #[test]
        fn split_frontmatter_empty_body() {
            assert_eq!(
                split_frontmatter("---\nmatter\n---").unwrap(),
                ("matter", "")
            )
        }

        #[test]
        fn split_frontmatter_with_body() {
            assert_eq!(
                split_frontmatter("---\nmatter\n---\nbody").unwrap(),
                ("matter", "body")
            )
        }
    }
}

mod store {
    use std::{collections::HashMap, fs, path::PathBuf};

    use serde::de::DeserializeOwned;

    use crate::{Error, FlatPage, Result};

    /// A store for [`FlatPageMeta`]
    #[derive(Debug)]
    pub struct FlatPageStore {
        /// The folder containing flat pages
        root: PathBuf,
        /// Maps file stems to pages metadata
        pub pages: HashMap<String, FlatPageMeta>,
    }

    /// Flat page metadata
    #[derive(Debug)]
    pub struct FlatPageMeta {
        /// Page title
        pub title: String,
        /// Page description
        pub description: Option<String>,
    }

    impl FlatPageStore {
        /// Creates a store from the folder
        pub fn read_dir(root: impl Into<PathBuf>) -> Result<Self> {
            let root = root.into();
            let mut pages = HashMap::new();
            let md_ext = Some(std::ffi::OsStr::new("md"));
            for entry in fs::read_dir(&root).map_err(|e| Error::ReadDir(e, root.clone()))? {
                let entry = entry.map_err(Error::DirEntry)?;
                let path = entry.path();
                if !path.is_file() || path.extension() != md_ext {
                    continue;
                }
                let stem = match path.file_stem().and_then(|x| x.to_str()) {
                    Some(s) => s,
                    None => continue,
                };
                let page = match FlatPage::by_path(&path)? {
                    Some(p) => p,
                    None => continue,
                };
                pages.insert(stem.into(), page.into());
            }
            Ok(Self { root, pages })
        }

        /// Returns a page metadata by its url
        pub fn meta_by_url(&self, url: &str) -> Option<&FlatPageMeta> {
            let stem = Self::url_to_stem(url);
            self.meta_by_stem(&stem)
        }

        /// Returns a page by its url
        pub fn page_by_url<E: DeserializeOwned>(&self, url: &str) -> Result<Option<FlatPage<E>>> {
            let stem = Self::url_to_stem(url);
            self.page_by_stem(&stem)
        }

        /// Returns a page metadata by the file stem
        pub fn meta_by_stem(&self, stem: &str) -> Option<&FlatPageMeta> {
            self.pages.get(stem)
        }

        /// Returns a page by the file stem
        pub fn page_by_stem<E: DeserializeOwned>(&self, stem: &str) -> Result<Option<FlatPage<E>>> {
            if self.pages.contains_key(stem) {
                let mut path = self.root.clone();
                path.push(format!("{stem}.md"));
                FlatPage::by_path(path)
            } else {
                Ok(None)
            }
        }

        /// Converts url to file stem
        fn url_to_stem(url: &str) -> String {
            url.replace('/', "^")
        }
    }

    impl From<FlatPage> for FlatPageMeta {
        fn from(p: FlatPage) -> Self {
            Self {
                title: p.title,
                description: p.description,
            }
        }
    }
}
