use std::{io, path::PathBuf};

/// The crates error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Broken frontmatter
    #[error("broken frontmatter in '{1}'")]
    ParseFrontmatter(#[source] serde_yml::Error, String),
    /// Can't read folder
    #[error("readdir '{1}'")]
    ReadDir(#[source] io::Error, PathBuf),
    /// Can't read folder entry
    #[error("readdir entry")]
    DirEntry(#[source] io::Error),
}

/// The crates result type
pub type Result<T> = std::result::Result<T, Error>;
