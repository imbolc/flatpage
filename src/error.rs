//! Error types for page loading and directory scanning.

use std::{io, path::PathBuf};

/// The crate's error type
#[non_exhaustive]
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
    /// Failed to read path metadata.
    #[error("failed to read filesystem metadata: {path}")]
    ReadMetadata {
        /// The underlying I/O error
        #[source]
        source: io::Error,
        /// The path whose metadata could not be read
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

impl Error {
    /// Creates a frontmatter parsing error for the given path.
    pub fn parse_frontmatter(
        source: markdown_frontmatter::Error,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self::ParseFrontmatter {
            source,
            path: path.into(),
        }
    }

    /// Creates a directory-reading error for the given path.
    pub fn read_dir(source: io::Error, path: impl Into<PathBuf>) -> Self {
        Self::ReadDir {
            source,
            path: path.into(),
        }
    }

    /// Creates a filesystem metadata error for the given path.
    pub fn read_metadata(source: io::Error, path: impl Into<PathBuf>) -> Self {
        Self::ReadMetadata {
            source,
            path: path.into(),
        }
    }

    /// Creates a file-reading error for the given path.
    pub fn read_file(source: io::Error, path: impl Into<PathBuf>) -> Self {
        Self::ReadFile {
            source,
            path: path.into(),
        }
    }
}

/// The crate's result type
pub type Result<T> = std::result::Result<T, Error>;
