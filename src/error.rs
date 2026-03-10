use std::{io, path::PathBuf};

/// The crate's error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Broken frontmatter
    #[error("broken frontmatter in: {path}")]
    ParseFrontmatter {
        /// The underlying frontmatter error
        #[source]
        source: markdown_frontmatter::Error,
        /// The path to the file
        path: String,
    },
    /// Cannot scan a folder.
    #[error("readdir: {path}")]
    ReadDir {
        /// The underlying I/O error
        #[source]
        source: io::Error,
        /// The path to the folder being scanned
        path: PathBuf,
    },
    /// Cannot read a file.
    #[error("read file: {path}")]
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
