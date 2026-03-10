use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use crate::{
    Error, FlatPage, Result,
    path::{normalize_url, page_path_from_normalized_url, path_to_url},
};

/// A store for [`FlatPageMeta`]
#[derive(Debug)]
pub struct FlatPageStore {
    /// The folder containing markdown pages
    root: PathBuf,
    /// Maps normalized URLs such as `/guides/install` to metadata.
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
    /// Creates a store by scanning the folder recursively.
    pub fn read_dir(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let mut pages = HashMap::new();
        read_dir_recursive(&root, &root, &mut pages)?;
        Ok(Self { root, pages })
    }

    /// Returns page metadata by URL.
    ///
    /// Trailing slashes are significant: `/foo` looks up `foo.md`, while
    /// `/foo/` looks up `foo/index.md`.
    ///
    /// Returns `None` for invalid URLs and missing pages.
    pub fn meta_by_url(&self, url: &str) -> Option<&FlatPageMeta> {
        let url = normalize_url(url)?;
        self.pages.get(&url)
    }

    /// Returns a page by URL.
    ///
    /// Trailing slashes are significant: `/foo` looks up `foo.md`, while
    /// `/foo/` looks up `foo/index.md`.
    ///
    /// Returns `Ok(None)` for invalid URLs and missing pages.
    pub fn page_by_url<E: DeserializeOwned>(&self, url: &str) -> Result<Option<FlatPage<E>>> {
        let url = match normalize_url(url) {
            Some(url) => url,
            None => return Ok(None),
        };
        // Intentionally check the in-memory index first so missing pages avoid
        // filesystem access.
        if !self.pages.contains_key(&url) {
            return Ok(None);
        }

        let path = page_path_from_normalized_url(&self.root, &url);
        FlatPage::by_path(path)
    }
}

impl<Extra> From<FlatPage<Extra>> for FlatPageMeta {
    /// Converts a full page into the cached metadata representation.
    fn from(p: FlatPage<Extra>) -> Self {
        Self {
            title: p.title,
            description: p.description,
        }
    }
}

/// Recursively walks the page tree and records metadata for valid Markdown
/// files.
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
        let entry = entry.map_err(|e| Error::ReadDir {
            source: e,
            path: dir.to_path_buf(),
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| Error::ReadDir {
            source: e,
            path: dir.to_path_buf(),
        })?;
        let is_markdown_file = if file_type.is_symlink() {
            if path.extension() != md_ext {
                continue;
            }
            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(Error::ReadMetadata {
                        source: error,
                        path: path.clone(),
                    });
                }
            };
            metadata.is_file()
        } else if file_type.is_dir() {
            read_dir_recursive(root, &path, pages)?;
            continue;
        } else {
            file_type.is_file() && path.extension() == md_ext
        };
        if !is_markdown_file {
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
        let page_meta = match read_page_meta(&path)? {
            Some(page_meta) => page_meta,
            None => continue,
        };
        pages.insert(url, page_meta);
    }
    Ok(())
}

/// Reads a page file and extracts the metadata stored in the index.
fn read_page_meta(path: &Path) -> Result<Option<FlatPageMeta>> {
    FlatPage::<()>::by_path(path).map(|page| page.map(Into::into))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{TestDir, write_page};

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
        assert!(store.meta_by_url("guides/install").is_none());

        let page = store.page_by_url::<()>("/guides/install").unwrap().unwrap();
        assert_eq!(page.title, "Install");

        let dotted = store.page_by_url::<()>("/guides/v1.2").unwrap().unwrap();
        assert_eq!(dotted.title, "Versioned Guide");
        assert!(store.page_by_url::<()>("guides/install").unwrap().is_none());
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
}
