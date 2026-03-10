use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use crate::{
    Error, FlatPage, ParsedPage, Result, normalize_url, page_path, parse_page_content, path_to_url,
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

        let Some(path) = page_path(&self.root, &url) else {
            return Ok(None);
        };
        FlatPage::by_path(path)
    }
}

impl<Extra> From<FlatPage<Extra>> for FlatPageMeta {
    fn from(p: FlatPage<Extra>) -> Self {
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

fn read_page_meta(path: &Path) -> Result<Option<FlatPageMeta>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(Error::ReadFile {
                source: error,
                path: path.to_path_buf(),
            });
        }
    };
    let ParsedPage {
        title, description, ..
    } = parse_page_content::<()>(&content).map_err(|error| Error::ParseFrontmatter {
        source: error,
        path: path.to_path_buf(),
    })?;
    Ok(Some(FlatPageMeta { title, description }))
}
