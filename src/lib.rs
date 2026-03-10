#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
compile_error!("enable at least one frontmatter feature: json, toml, yaml");

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use serde::de::DeserializeOwned;
pub use store::{FlatPageMeta, FlatPageStore};

const ALLOWED_IN_URL_SEGMENT: &str = "_-.";

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
struct Frontmatter<Extra = ()> {
    title: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    extra: Extra,
}

/// Flat page
#[derive(Debug)]
pub struct FlatPage<Extra = ()> {
    /// Page title
    pub title: String,
    /// Description - for html meta description, `og:description`, etc
    pub description: Option<String>,
    /// Raw markdown version of the body
    pub body: String,
    /// Extra frontmatter fields (except `title` and `description`)
    pub extra: Extra,
}

impl<Extra: DeserializeOwned> FlatPage<Extra> {
    /// Returns a page by its url
    pub fn by_url(root: impl Into<PathBuf>, url: &str) -> Result<Option<Self>> {
        let relative_path = match url_to_path(url) {
            Some(path) => path,
            None => return Ok(None),
        };
        let mut path: PathBuf = root.into();
        path.push(relative_path);
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
        ) = markdown_frontmatter::parse::<Frontmatter<Extra>>(content)?;
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

/// Tries to normalize the url
fn normalize_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    let trailing_slash = url.ends_with('/');
    let url = url.trim_matches('/');
    if url.is_empty() {
        return Some("/".into());
    }
    let mut normalized = String::from("/");
    for (index, segment) in url.split('/').enumerate() {
        if !is_valid_url_segment(segment) {
            return None;
        }
        if index > 0 {
            normalized.push('/');
        }
        normalized.push_str(segment);
    }
    if trailing_slash {
        normalized.push('/');
    }
    Some(normalized)
}

fn is_valid_url_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ALLOWED_IN_URL_SEGMENT.contains(c))
}

/// Tries to convert the url into a relative markdown path
fn url_to_path(url: &str) -> Option<PathBuf> {
    let url = normalize_url(url)?;
    if url == "/" {
        return Some(PathBuf::from("index.md"));
    }
    let mut path = PathBuf::new();
    for segment in url.trim_matches('/').split('/') {
        path.push(segment);
    }
    if url.ends_with('/') {
        path.push("index.md");
    } else {
        path.set_extension("md");
    }
    Some(path)
}

fn path_to_url(path: &Path) -> Option<String> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(segment) = component else {
            return None;
        };
        let segment = segment.to_str()?;
        if !is_valid_url_segment(segment.strip_suffix(".md").unwrap_or(segment)) {
            return None;
        }
        components.push(segment);
    }

    let file_name = components.pop()?;
    if file_name == "index.md" {
        if components.is_empty() {
            return Some("/".into());
        }
        return Some(format!("/{}/", components.join("/")));
    }

    let stem = file_name.strip_suffix(".md")?;
    components.push(stem);
    Some(format!("/{}", components.join("/")))
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
    use std::{
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn test_url_to_path() {
        assert_eq!(url_to_path(""), None);
        assert_eq!(url_to_path("#"), None);
        assert_eq!(url_to_path("ы"), None);
        assert_eq!(url_to_path("/../secret"), None);
        assert_eq!(url_to_path("/foo//bar"), None);
        assert_eq!(url_to_path("/").unwrap(), PathBuf::from("index.md"));
        assert_eq!(
            url_to_path("/foo-bar/baz").unwrap(),
            PathBuf::from("foo-bar/baz.md")
        );
        assert_eq!(
            url_to_path("/foo-bar/baz/").unwrap(),
            PathBuf::from("foo-bar/baz/index.md")
        );
    }

    #[test]
    fn test_path_to_url() {
        assert_eq!(path_to_url(Path::new("index.md")).as_deref(), Some("/"));
        assert_eq!(
            path_to_url(Path::new("guides/getting-started.md")).as_deref(),
            Some("/guides/getting-started")
        );
        assert_eq!(
            path_to_url(Path::new("guides/index.md")).as_deref(),
            Some("/guides/")
        );
        assert_eq!(path_to_url(Path::new("../secret.md")), None);
        assert_eq!(path_to_url(Path::new("guides/../secret.md")), None);
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

    #[test]
    fn flatpage_by_url_reads_nested_paths() {
        let root = TestDir::new();
        write_page(root.path(), "guides/rust/index.md", "# Rust Guide");
        write_page(root.path(), "guides/install.md", "# Install");

        let index = FlatPage::<()>::by_url(root.path(), "/guides/rust/")
            .unwrap()
            .unwrap();
        assert_eq!(index.title, "Rust Guide");

        let page = FlatPage::<()>::by_url(root.path(), "/guides/install")
            .unwrap()
            .unwrap();
        assert_eq!(page.title, "Install");
    }

    #[test]
    fn flatpage_store_reads_nested_paths() {
        let root = TestDir::new();
        write_page(root.path(), "index.md", "# Home");
        write_page(root.path(), "guides/index.md", "# Guides");
        write_page(root.path(), "guides/install.md", "# Install");

        let store = FlatPageStore::read_dir(root.path()).unwrap();
        assert_eq!(store.meta_by_url("/").unwrap().title, "Home");
        assert_eq!(store.meta_by_url("/guides/").unwrap().title, "Guides");
        assert_eq!(
            store.meta_by_url("/guides/install").unwrap().title,
            "Install"
        );
        assert!(store.meta_by_url("/guides").is_none());

        let page = store.page_by_url::<()>("/guides/install").unwrap().unwrap();
        assert_eq!(page.title, "Install");
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("flatpage-test-{}-{unique}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            drop(fs::remove_dir_all(&self.path));
        }
    }

    fn write_page(root: &Path, relative_path: &str, content: &str) {
        let path = root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
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
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
    };

    use serde::de::DeserializeOwned;

    use crate::{Error, FlatPage, Result, normalize_url, path_to_url, url_to_path};

    /// A store for [`FlatPageMeta`]
    #[derive(Debug)]
    pub struct FlatPageStore {
        /// The folder containing markdown pages
        root: PathBuf,
        /// Maps page urls to metadata
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
            read_dir_recursive(&root, &root, &mut pages)?;
            Ok(Self { root, pages })
        }

        /// Returns a page metadata by its url
        pub fn meta_by_url(&self, url: &str) -> Option<&FlatPageMeta> {
            let url = normalize_url(url)?;
            self.pages.get(&url)
        }

        /// Returns a page by its url
        pub fn page_by_url<E: DeserializeOwned>(&self, url: &str) -> Result<Option<FlatPage<E>>> {
            let url = match normalize_url(url) {
                Some(url) => url,
                None => return Ok(None),
            };
            if self.pages.contains_key(&url) {
                let mut path = self.root.clone();
                path.push(url_to_path(&url).unwrap());
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

    fn read_dir_recursive(
        root: &Path,
        dir: &Path,
        pages: &mut HashMap<String, FlatPageMeta>,
    ) -> Result<()> {
        let md_ext = Some(std::ffi::OsStr::new("md"));
        for entry in fs::read_dir(dir).map_err(|e| Error::ReadDir {
            source: e,
            path: dir.to_path_buf(),
        })? {
            let entry = entry.map_err(|e| Error::DirEntry { source: e })?;
            let path = entry.path();
            if path.is_dir() {
                read_dir_recursive(root, &path, pages)?;
                continue;
            }
            if !path.is_file() || path.extension() != md_ext {
                continue;
            }
            let relative_path = match path.strip_prefix(root) {
                Ok(relative_path) => relative_path,
                Err(_) => continue,
            };
            let url = match path_to_url(relative_path) {
                Some(url) => url,
                None => continue,
            };
            let page = match FlatPage::by_path(&path)? {
                Some(page) => page,
                None => continue,
            };
            pages.insert(url, page.into());
        }
        Ok(())
    }
}
