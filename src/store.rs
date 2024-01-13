use crate::{Error, FlatPage, Result};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, fs, path::PathBuf};

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
