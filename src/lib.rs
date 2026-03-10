#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
compile_error!("enable at least one frontmatter feature: json, toml, yaml");

mod error;
mod store;

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

pub use error::{Error, Result};
use serde::de::DeserializeOwned;
pub use store::{FlatPageMeta, FlatPageStore};

const ALLOWED_IN_URL_SEGMENT: &str = "_-.";

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
    /// Returns a page by its URL.
    ///
    /// Accepted forms include `/foo`, `/foo/`, and `foo`.
    ///
    /// Returns `Ok(None)` for invalid URLs and missing pages. Returns `Err` for
    /// I/O failures and frontmatter parsing errors.
    pub fn by_url(root: impl Into<PathBuf>, url: &str) -> Result<Option<Self>> {
        let relative_path = match url_to_path(url) {
            Some(path) => path,
            None => return Ok(None),
        };
        let mut path: PathBuf = root.into();
        path.push(relative_path);
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
        let (
            Frontmatter {
                title,
                description,
                extra,
            },
            body,
        ) = frontmatter_and_body(content)?;
        let title = resolve_title(title, body);
        Ok(Self {
            title,
            description,
            body: body.to_string(),
            extra,
        })
    }
}

fn frontmatter_and_body<Extra: DeserializeOwned>(
    content: &str,
) -> std::result::Result<(Frontmatter<Extra>, &str), markdown_frontmatter::Error> {
    markdown_frontmatter::parse::<Frontmatter<Extra>>(content)
}

fn resolve_title(title: Option<String>, body: &str) -> String {
    title.unwrap_or_else(|| title_from_markdown(body).to_string())
}

/// Considers the first line to be the page title, removes markdown header
/// prefix `#` and optional trailing ATX closing markers
fn title_from_markdown(body: &str) -> &str {
    let line = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    if !line.starts_with('#') {
        return line.trim();
    }

    strip_optional_atx_closing(line.trim_start_matches('#')).trim()
}

fn strip_optional_atx_closing(line: &str) -> &str {
    let trimmed_end = line.trim_end();
    let mut hashes_start = trimmed_end.len();
    for (index, ch) in trimmed_end.char_indices().rev() {
        if ch == '#' {
            hashes_start = index;
        } else {
            break;
        }
    }
    if hashes_start == trimmed_end.len() {
        return line;
    }

    let prefix = &trimmed_end[..hashes_start];
    if prefix.chars().last().is_some_and(char::is_whitespace) {
        prefix.trim_end()
    } else {
        line
    }
}

/// Tries to normalize the URL.
fn normalize_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }

    if url == "/" {
        return Some("/".into());
    }

    let trailing_slash = url.ends_with('/');
    let url = url.strip_prefix('/').unwrap_or(url);
    let url = if trailing_slash {
        url.strip_suffix('/').unwrap_or(url)
    } else {
        url
    };

    if url.is_empty() || url.contains("//") {
        return None;
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

/// Tries to convert the URL into a relative Markdown path.
fn url_to_path(url: &str) -> Option<PathBuf> {
    let url = normalize_url(url)?;
    if url == "/" {
        return Some(PathBuf::from("index.md"));
    }
    let mut path = PathBuf::new();
    let mut segments = url.trim_matches('/').split('/');
    let last_segment = segments.next_back()?;
    for segment in segments {
        path.push(segment);
    }
    if url.ends_with('/') {
        path.push(last_segment);
        path.push("index.md");
    } else {
        path.push(format!("{last_segment}.md"));
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
        components.push(segment);
    }

    let file_name = components.pop()?;
    for segment in &components {
        if !is_valid_url_segment(segment) {
            return None;
        }
    }
    if file_name == "index.md" {
        if components.is_empty() {
            return Some("/".into());
        }
        return Some(format!("/{}/", components.join("/")));
    }

    let stem = file_name.strip_suffix(".md")?;
    if !is_valid_url_segment(stem) {
        return None;
    }
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
        assert_eq!(url_to_path("//foo"), None);
        assert_eq!(url_to_path("foo//"), None);
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
        assert_eq!(
            url_to_path("/foo.bar").unwrap(),
            PathBuf::from("foo.bar.md")
        );
    }

    #[test]
    fn test_path_to_url() {
        assert_eq!(path_to_url(Path::new("index.md")).as_deref(), Some("/"));
        assert_eq!(
            path_to_url(Path::new(".md/index.md")).as_deref(),
            Some("/.md/")
        );
        assert_eq!(
            path_to_url(Path::new("guides/getting-started.md")).as_deref(),
            Some("/guides/getting-started")
        );
        assert_eq!(
            path_to_url(Path::new("guides/index.md")).as_deref(),
            Some("/guides/")
        );
        assert_eq!(
            path_to_url(Path::new("guides/v1.2.md")).as_deref(),
            Some("/guides/v1.2")
        );
        assert_eq!(path_to_url(Path::new("../secret.md")), None);
        assert_eq!(path_to_url(Path::new("guides/../secret.md")), None);
    }

    #[test]
    fn test_normalize_url_rejects_empty_segments() {
        assert_eq!(normalize_url("//foo"), None);
        assert_eq!(normalize_url("foo//"), None);
        assert_eq!(normalize_url("foo//bar"), None);
        assert_eq!(normalize_url("////"), None);
        assert_eq!(normalize_url("/foo/").as_deref(), Some("/foo/"));
        assert_eq!(normalize_url("foo").as_deref(), Some("/foo"));
    }

    #[test]
    fn test_title_from_markdown() {
        assert_eq!(title_from_markdown("# Foo"), "Foo");
        assert_eq!(title_from_markdown("## Foo"), "Foo");
        assert_eq!(title_from_markdown("# Foo #"), "Foo");
        assert_eq!(title_from_markdown("# Foo ##"), "Foo");
        assert_eq!(title_from_markdown("# Foo#"), "Foo#");
        assert_eq!(title_from_markdown("# #"), "");
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
    }

    #[test]
    fn flatpage_store_reads_nested_paths() {
        let root = TestDir::new();
        write_page(root.path(), "index.md", "# Home");
        write_page(root.path(), "guides/index.md", "# Guides");
        write_page(root.path(), "guides/install.md", "# Install");
        write_page(root.path(), "guides/v1.2.md", "# Versioned Guide");

        let store = FlatPageStore::read_dir(root.path()).unwrap();
        assert_eq!(store.meta_by_url("/").unwrap().title, "Home");
        assert_eq!(store.meta_by_url("/guides/").unwrap().title, "Guides");
        assert_eq!(
            store.meta_by_url("/guides/install").unwrap().title,
            "Install"
        );
        assert_eq!(
            store.meta_by_url("/guides/v1.2").unwrap().title,
            "Versioned Guide"
        );
        assert!(store.meta_by_url("/guides").is_none());

        let page = store.page_by_url::<()>("/guides/install").unwrap().unwrap();
        assert_eq!(page.title, "Install");

        let dotted = store.page_by_url::<()>("/guides/v1.2").unwrap().unwrap();
        assert_eq!(dotted.title, "Versioned Guide");
    }

    #[cfg(unix)]
    #[test]
    fn flatpage_store_ignores_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let root = TestDir::new();
        let external = TestDir::new();
        write_page(root.path(), "index.md", "# Home");
        write_page(external.path(), "secret.md", "# Secret");

        symlink(external.path(), root.path().join("linked")).unwrap();

        let store = FlatPageStore::read_dir(root.path()).unwrap();
        assert_eq!(store.meta_by_url("/").unwrap().title, "Home");
        assert!(store.meta_by_url("/linked/secret").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn flatpage_store_reads_symlinked_files() {
        use std::os::unix::fs::symlink;

        let root = TestDir::new();
        let external = TestDir::new();
        write_page(root.path(), "index.md", "# Home");
        write_page(external.path(), "install.md", "# Install");

        symlink(
            external.path().join("install.md"),
            root.path().join("install.md"),
        )
        .unwrap();

        let store = FlatPageStore::read_dir(root.path()).unwrap();
        assert_eq!(store.meta_by_url("/").unwrap().title, "Home");
        assert_eq!(store.meta_by_url("/install").unwrap().title, "Install");
    }

    #[cfg(unix)]
    #[test]
    fn flatpage_store_skips_broken_symlinked_files() {
        use std::os::unix::fs::symlink;

        let root = TestDir::new();
        write_page(root.path(), "index.md", "# Home");

        symlink(
            root.path().join("missing.md"),
            root.path().join("broken.md"),
        )
        .unwrap();

        let store = FlatPageStore::read_dir(root.path()).unwrap();
        assert_eq!(store.meta_by_url("/").unwrap().title, "Home");
        assert!(store.meta_by_url("/broken").is_none());
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
