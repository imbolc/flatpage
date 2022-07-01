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
//! if let Some(home) = flatpage::FlatPage::by_url(root_folder, "/").unwrap() {
//!     println!("title: {}", home.title);
//!     println!("description: {:?}", home.description);
//!     println!("markdown body: {}", home.body);
//!     println!("html body: {}", home.html());
//! } else {
//!     println!("No home page");
//! }
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
use displaydoc::Display;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

const ALLOWED_IN_URL: &str = "/_-";

/// The crates error type
#[derive(Debug, Display, Error)]
pub enum Error {
    /// broken frontmatter yaml in `{1}`
    Frontmatter(#[source] serde_yaml::Error, String),
    /// readdir `{1}`
    ReadDir(#[source] io::Error, PathBuf),
    /// readdir entry
    DirEntry(#[source] io::Error),
}

/// The crates result type
pub type Result<T> = std::result::Result<T, Error>;

/// A store for [`FlatPageMeta`]
#[derive(Debug)]
pub struct FlatPageStore {
    /// The folder containing flat pages
    root: PathBuf,
    /// Maps file stems to pages metadata
    pages: HashMap<String, FlatPageMeta>,
}

/// Flat page metadata
#[derive(Debug)]
pub struct FlatPageMeta {
    /// Page title
    pub title: String,
    /// Page description
    pub description: Option<String>,
}

/// Flat page
#[derive(Debug)]
pub struct FlatPage {
    /// Title - for html title tag, `og:title`, etc
    pub title: String,
    /// Description - for html meta description, `og:description`, etc
    pub description: Option<String>,
    /// Raw markdown version of the body
    pub body: String,
}

/// Flat page yaml-based frontmatter
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    title: Option<String>,
    description: Option<String>,
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
    pub fn page_by_url(&self, url: &str) -> Result<Option<FlatPage>> {
        let stem = Self::url_to_stem(url);
        self.page_by_stem(&stem)
    }

    /// Returns a page metadata by the file stem
    pub fn meta_by_stem(&self, stem: &str) -> Option<&FlatPageMeta> {
        self.pages.get(stem)
    }

    /// Returns a page by the file stem
    pub fn page_by_stem(&self, stem: &str) -> Result<Option<FlatPage>> {
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

impl FlatPage {
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
            .map_err(|e| Error::Frontmatter(e, path.display().to_string()))
    }

    /// [`FlatPage::body`] rendered to html
    pub fn html(&self) -> String {
        markdown(&self.body)
    }

    /// Parses a page from text
    fn from_content(content: &str) -> serde_yaml::Result<Self> {
        let (maybe_matter, body) = Frontmatter::parse(content)?;
        let page = if let Some(matter) = maybe_matter {
            let title = matter
                .title
                .unwrap_or_else(|| title_from_markdown(body).into());
            let description = matter.description;
            Self {
                title,
                description,
                body: body.to_string(),
            }
        } else {
            let title = title_from_markdown(content).into();
            Self {
                title,
                description: None,
                body: content.to_string(),
            }
        };
        Ok(page)
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

impl Frontmatter {
    /// Parses frontmatter from the file content, returns the frontmatter and the rest of the
    /// content (page body)
    fn parse(content: &str) -> serde_yaml::Result<(Option<Self>, &str)> {
        let content = content.trim();
        let mut parts = content.splitn(3, "---");

        let prefix = parts.next().unwrap(); // `splitn` should always return at least one item
        if !prefix.is_empty() {
            // content doesn't start from the delimeter
            return Ok((None, content));
        }

        let matter_str = if let Some(s) = parts.next() {
            s
        } else {
            // no first opening delimeter
            return Ok((None, content));
        };

        let body = if let Some(s) = parts.next() {
            s
        } else {
            // no closing delimiter
            return Ok((None, content));
        };

        let matter = serde_yaml::from_str(matter_str)?;
        Ok((Some(matter), body.trim()))
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
    fn test_frontmatter_from_content() {
        println!("No starting delimiter");
        let text = "foo";
        let (m, b) = Frontmatter::parse(text).unwrap();
        assert!(m.is_none());
        assert_eq!(b, text);

        println!("Starting delimiter inside content");
        let text = "foo\n---\nfoo: bar\n---\nbody";
        let (m, b) = Frontmatter::parse(text).unwrap();
        assert!(m.is_none());
        assert_eq!(b, text);

        println!("Just the starting delimiter");
        let text = "---";
        let (m, b) = Frontmatter::parse(text).unwrap();
        assert!(m.is_none());
        assert_eq!(b, text);

        println!("No closing delimeter");
        let text = "---\ntitle: bar\nbaz";
        let (m, b) = Frontmatter::parse(text).unwrap();
        assert!(m.is_none());
        assert_eq!(b, text);

        println!("Empty frontmatter");
        assert!(Frontmatter::parse("---\n\n---").is_err());

        println!("Unknown field");
        assert!(Frontmatter::parse("---\nunknown_field: bar\n---").is_err());

        println!("Empty body");
        let text = "---\ntitle: foo\n---";
        let (m, b) = Frontmatter::parse(text).unwrap();
        assert_eq!(m.unwrap().title.unwrap(), "foo");
        assert_eq!(b, "");

        println!("Title with body");
        let text = "---\ntitle: foo\n---\nbar";
        let (m, b) = Frontmatter::parse(text).unwrap();
        assert_eq!(m.unwrap().title.unwrap(), "foo");
        assert_eq!(b, "bar");
    }

    #[test]
    fn test_flatpage_title() {
        let page = FlatPage::from_content("# Foo").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.body, "# Foo");
        assert_eq!(
            FlatPage::from_content("---\ntitle: Bar\n---\n# Foo")
                .unwrap()
                .title,
            "Bar"
        );
    }

    #[test]
    fn test_flatpage_description() {
        assert_eq!(FlatPage::from_content("").unwrap().description, None);
        assert_eq!(
            FlatPage::from_content("---\ndescription: Bar\n---")
                .unwrap()
                .description
                .unwrap(),
            "Bar"
        );
    }

    #[test]
    fn test_doc_table() {
        let page = FlatPage::from_content("# Foo\nBar").unwrap();
        assert_eq!(page.title, "Foo");
        assert!(page.description.is_none());
        assert_eq!(page.body, "# Foo\nBar");
        assert_eq!(page.html(), "<h1>Foo</h1>\n<p>Bar</p>\n");

        let page = FlatPage::from_content("---\ndescription: Bar\n---\n# Foo").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.description.as_deref().unwrap(), "Bar");
        assert_eq!(page.body, "# Foo");
        assert_eq!(page.html(), "<h1>Foo</h1>\n");

        let page = FlatPage::from_content("---\ntitle: Foo\ndescription: Bar\n---").unwrap();
        assert_eq!(page.title, "Foo");
        assert_eq!(page.description.as_deref().unwrap(), "Bar");
        assert_eq!(page.body, "");
        assert_eq!(page.html(), "");
    }
}
