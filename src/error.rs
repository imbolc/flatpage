use std::{io, path::PathBuf};

/// The crates error type
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum Error {
    /// broken frontmatter in `{1}`
    ParseFrontmatter(#[source] serde_yaml::Error, String),
    /// readdir `{1}`
    ReadDir(#[source] io::Error, PathBuf),
    /// readdir entry
    DirEntry(#[source] io::Error),
}

/// The crates result type
pub type Result<T> = std::result::Result<T, Error>;
