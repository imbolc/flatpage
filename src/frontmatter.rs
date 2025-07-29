use serde::{Deserialize, de::DeserializeOwned};

const EMPTY_YAML: &str = "{}";

/// Markdown frontmatter
#[derive(Debug, Deserialize)]
pub(crate) struct Frontmatter<E = ()> {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: E,
}

impl<E: DeserializeOwned> Frontmatter<E> {
    /// Parses frontmatter from markdown string.
    /// Returns the frontmatter and the rest of the content (page body)
    pub(crate) fn parse(content: &str) -> serde_yaml::Result<(Self, &str)> {
        let (matter, body) =
            split_frontmatter(content).unwrap_or_else(|| (EMPTY_YAML, content.trim()));
        serde_yaml::from_str(matter).map(|m| (m, body))
    }
}

/// If frontmatter is found returns it and the rest of the body, `None`
/// otherwise
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let content = content.trim_start();

    let (prefix, rest) = content.split_once("---\n")?;
    if !prefix.is_empty() {
        // content doesn't start with the delimiter
        return None;
    }

    let (matter, body) = rest.split_once("\n---")?;
    Some((matter, body.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_empty_frontmatter() {
        let parsed: Frontmatter = serde_yaml::from_str(EMPTY_YAML).unwrap();
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.description, None);
    }

    #[test]
    fn deserialize_frontmatter_with_unknown_fields() {
        let yaml = "foo: 1\nbar: true";
        let parsed: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.description, None);
    }

    #[test]
    fn deserialize_frontmatter_with_only_title() {
        let yaml = "title: foo";
        let parsed: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.title.unwrap(), "foo");
        assert_eq!(parsed.description, None);
    }

    #[test]
    fn deserialize_frontmatter_with_extra_fields() {
        #[derive(Debug, Deserialize)]
        struct Extra {
            slug: String,
            active: bool,
        }

        let yaml = "slug: foo\nactive: true";
        let parsed: Frontmatter<Extra> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.extra.slug, "foo");
        assert!(parsed.extra.active);
    }

    #[test]
    fn split_frontmatter_empty_page() {
        assert_eq!(split_frontmatter(""), None)
    }

    #[test]
    fn split_frontmatter_no_opening_delimiter() {
        assert_eq!(split_frontmatter("foo"), None)
    }

    #[test]
    fn split_frontmatter_doesnt_start_with_delimiter() {
        assert_eq!(split_frontmatter("foo\n---not a frontmatter\n---"), None)
    }

    #[test]
    fn split_frontmatter_no_closing_delimiter() {
        assert_eq!(split_frontmatter("---\nnot a frontmatter"), None)
    }

    #[test]
    fn split_frontmatter_empty_body() {
        assert_eq!(
            split_frontmatter("---\nmatter\n---").unwrap(),
            ("matter", "")
        )
    }

    #[test]
    fn split_frontmatter_with_body() {
        assert_eq!(
            split_frontmatter("---\nmatter\n---\nbody").unwrap(),
            ("matter", "body")
        )
    }
}
