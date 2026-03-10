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

    /// Returns page metadata by URL.
    pub fn meta_by_url(&self, url: &str) -> Option<&FlatPageMeta> {
        let url = normalize_url(url)?;
        self.pages.get(&url)
    }

    /// Returns a page by URL.
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
        let entry = entry.map_err(|e| Error::ReadDir {
            source: e,
            path: dir.to_path_buf(),
        })?;
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
