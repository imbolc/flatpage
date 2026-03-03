#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
compile_error!("enable at least one frontmatter feature: json, toml, yaml");

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
pub use store::{FlatPageMeta, FlatPageStore};

const ALLOWED_IN_URL: &str = "/_-.";

/// The crate's error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Broken frontmatter
    #[error("broken frontmatter in '{path}'")]
    ParseFrontmatter {
        /// The underlying frontmatter error
        #[source]
        source: markdown_frontmatter::Error,
        /// The path to the file
        path: String,
    },
    /// Can't read folder
    #[error("readdir '{path}'")]
    ReadDir {
        /// The underlying I/O error
        #[source]
        source: io::Error,
        /// The path to the folder
        path: PathBuf,
    },
    /// Can't read folder entry
    #[error("readdir entry")]
    DirEntry {
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },
    /// Can't read file
    #[error("read file '{path}'")]
    ReadFile {
        /// The underlying I/O error
        #[source]
        source: io::Error,
        /// The path to the file
        path: PathBuf,
    },
}

/// The crate's result type
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, serde::Deserialize)]
struct Frontmatter<E = ()> {
    title: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    extra: E,
}

/// Flat page
#[derive(Debug)]
pub struct FlatPage<E = ()> {
    /// Page title
    pub title: String,
    /// Description - for html meta description, `og:description`, etc
    pub description: Option<String>,
    /// Raw markdown version of the body
    pub body: String,
    /// Extra frontmatter fields (except `title` and `description`)
    pub extra: E,
}

impl<E: DeserializeOwned> FlatPage<E> {
    /// Returns a page by its url
    pub fn by_url(root: impl Into<PathBuf>, url: &str) -> Result<Option<Self>> {
        let stem = match url_to_stem(url) {
            Some(f) => f,
            None => return Ok(None),
        };
        let mut path: PathBuf = root.into();
        path.push(format!("{stem}.md"));
        Self::by_path(&path)
    }

    /// Returns a page by its file path
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
                path: path.display().to_string(),
            })
    }

    /// [`FlatPage::body`] rendered to html
    pub fn html(&self) -> String {
        markdown(&self.body)
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
        ) = markdown_frontmatter::parse::<Frontmatter<E>>(content)?;
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
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .trim_start_matches('#')
        .trim()
}

/// Tries to convert the url into a file stem
fn url_to_stem(url: &str) -> Option<String> {
    if url.is_empty() {
        None
    } else if url
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ALLOWED_IN_URL.contains(c))
    {
        Some(url.replace('/', "^"))
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
    fn test_url_to_stem() {
        assert_eq!(url_to_stem(""), None);
        assert_eq!(url_to_stem("#"), None);
        assert_eq!(url_to_stem("ы"), None);
        assert_eq!(url_to_stem("/foo-bar/baz/").unwrap(), "^foo-bar^baz^");
    }

    #[test]
    fn test_title_from_markdown() {
        assert_eq!(title_from_markdown("# Foo"), "Foo");
        assert_eq!(title_from_markdown("## Foo"), "Foo");
        assert_eq!(title_from_markdown("Foo"), "Foo");
        assert_eq!(title_from_markdown(""), "");
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

mod store {
    use std::{collections::HashMap, fs, path::PathBuf};

    use serde::de::DeserializeOwned;

    use crate::{Error, FlatPage, Result, url_to_stem};

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
            for entry in fs::read_dir(&root).map_err(|e| Error::ReadDir {
                source: e,
                path: root.clone(),
            })? {
                let entry = entry.map_err(|e| Error::DirEntry { source: e })?;
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
            let stem = url_to_stem(url)?;
            self.meta_by_stem(&stem)
        }

        /// Returns a page by its url
        pub fn page_by_url<E: DeserializeOwned>(&self, url: &str) -> Result<Option<FlatPage<E>>> {
            let stem = match url_to_stem(url) {
                Some(s) => s,
                None => return Ok(None),
            };
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
