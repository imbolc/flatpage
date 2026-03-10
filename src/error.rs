use std::{io, path::PathBuf};

/// The crate's error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse frontmatter.
    #[error("failed to parse frontmatter: {path}")]
    ParseFrontmatter {
        /// The underlying frontmatter error
        #[source]
        source: markdown_frontmatter::Error,
        /// The path to the file
        path: PathBuf,
    },
    /// Failed to read a directory.
    #[error("failed to read directory: {path}")]
    ReadDir {
        /// The underlying I/O error
        #[source]
        source: io::Error,
        /// The path to the directory being read
        path: PathBuf,
    },
    /// Failed to read a file.
    #[error("failed to read file: {path}")]
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
