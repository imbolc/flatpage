#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
compile_error!("enable at least one frontmatter feature: json, toml, yaml");

mod error;
mod markdown;
mod page;
mod path;
mod store;
#[cfg(test)]
mod test_helpers;
mod util;

pub use error::{Error, Result};
pub use page::FlatPage;
pub use store::{FlatPageMeta, FlatPageStore};
