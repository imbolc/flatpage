//! Internal typed helpers for page URLs and paths.

pub(crate) mod abs_page_path;
pub(crate) mod normalized_url;
mod page_location;
pub(crate) mod page_segment;
pub(crate) mod rel_page_path;

pub(crate) use abs_page_path::AbsPagePath;
pub(crate) use normalized_url::NormalizedUrl;
pub(crate) use page_segment::is_valid_page_segment;
pub(crate) use rel_page_path::RelPagePath;
