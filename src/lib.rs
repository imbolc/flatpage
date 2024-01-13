//! A simple file system based markdown flat page.
//!
//! ## Folder structure
//!
//! Only characters allowed in urls are ASCII, numbers and hyphen with underscore.
//! Urls map to files by simply substituting `/` to `^` and adding `.md` extension.
//! I believe it should eliminate all kinds of security issues.
//!
//! | url            | file name         |
//! |----------------|-------------------|
//! | `/`            | `^.md`            |
//! | `/foo/bar-baz` | `^foo^bar-baz.md` |
//!
//! ## Page format
//!
//! File could provide title and description in a yaml-based frontmatter, if there's no frontmatter
//! the first line would be considered the title (and cleaned from possible header marker `#`).
//!
//! | File content                                         | [`title`] | [`description`] | [`body`] | [`html()`]           |
//! |------------------------------------------------------|---------------------|---------------------------|--------------------|--------------------------------|
//! | `# Foo`<br>`Bar`                                     | `"Foo"`             | `None`                    | `"# Foo\nBar"`     | `"<h1>Foo</h1>\n<p>Bar</p>\n"` |
//! | `---`<br>`description: Bar`<br>`---`<br>`# Foo`      | `"Foo"`             | `Some("Bar")`             | `"# Foo"`          | `"<h1>Foo</h1>\n"`             |
//! | `---`<br>`title: Foo`<br>`description: Bar`<br>`---` | `"Foo"`             | `Some("Bar")`             | `""`               | `""`                           |
//!
//!
//! ## Reading a page
//!
//! ```rust
//! let root_folder = "./";
//! if let Some(home) = flatpage::FlatPage::<()>::by_url(root_folder, "/").unwrap() {
//!     println!("title: {}", home.title);
//!     println!("description: {:?}", home.description);
//!     println!("markdown body: {}", home.body);
//!     println!("html body: {}", home.html());
//! } else {
//!     println!("No home page");
//! }
//! ```
//!
//! ## Extra frontmatter fields
//!
//! You can define extra statically typed frontmatter fields
//!
//! ```rust
//! #[derive(Debug, serde::Deserialize)]
//! struct Extra {
//!     slug: String,
//! }
//!
//! let _page = flatpage::FlatPage::<Extra>::by_url("./", "/").unwrap();
//! ```
//!
//! ## Cached metadata
//!
//! It's a common for a page to have a list of related pages. To avoid reading all the files each
//! time, you can use [`FlatPageStore`] to cache pages [`metadata`] (titles and descriptions).
//!
//! ```rust
//! let root_folder = "./";
//! let store = flatpage::FlatPageStore::read_dir(root_folder).unwrap();
//! if let Some(meta) = store.meta_by_url("/") {
//!     println!("title: {}", meta.title);
//!     println!("description: {:?}", meta.description);
//! } else {
//!     println!("No home page");
//! }
//! ```
//!
//! [`title`]: FlatPage::title
//! [`description`]: FlatPage::description
//! [`body`]: FlatPage::body
//! [`html()`]: FlatPage::html()
//! [`metadata`]: FlatPageMeta
#![warn(clippy::all, missing_docs, nonstandard_style, future_incompatible)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
use frontmatter::Frontmatter;
use serde::de::DeserializeOwned;
use std::{
    fs,
    path::{Path, PathBuf},
};

mod error;
mod frontmatter;
mod store;

pub use error::{Error, Result};
pub use store::{FlatPageMeta, FlatPageStore};

const ALLOWED_IN_URL: &str = "/_-";

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
    fn from_content(content: &str) -> serde_yaml::Result<Self> {
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

/// Considers the first line to be the page title, removes markdown header prefix `#`
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
    use pulldown_cmark::{html, Options, Parser};

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
