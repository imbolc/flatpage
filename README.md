[![License](https://img.shields.io/crates/l/flatpage.svg)](https://choosealicense.com/licenses/mit/)
[![Crates.io](https://img.shields.io/crates/v/flatpage.svg)](https://crates.io/crates/flatpage)
[![Docs.rs](https://docs.rs/flatpage/badge.svg)](https://docs.rs/flatpage)

A simple filesystem-based Markdown page loader.

## Folder structure

The only characters allowed in URL segments are ASCII letters, numbers,
hyphens, underscores, and dots. URLs map to nested Markdown files, and
`index.md` is used for `/` and folder index pages. Empty path segments plus
`.` and `..` are
rejected.

Accepted forms include `/foo`, `/foo/`, and `foo` (normalized to `/foo`).
Invalid examples include `//foo`, `/foo//bar`, and `/../secret`.

| url             | file name              |
| --------------- | ---------------------- |
| `/`             | `index.md`             |
| `/foo/bar-baz`  | `foo/bar-baz.md`       |
| `/foo/bar-baz/` | `foo/bar-baz/index.md` |

## Page format

A file can provide a title and description in frontmatter. `flatpage` proxies
`markdown-frontmatter` features, so you can parse YAML (`---`), TOML (`+++`),
and JSON (`{ ... }`) depending on the enabled features. If there's no
frontmatter, the first line is considered the title (and cleaned from a
possible header
marker `#`).

| File content                                         | [`title`] | [`description`] | [`body`]       | [`html()`]                     |
| ---------------------------------------------------- | --------- | --------------- | -------------- | ------------------------------ |
| `# Foo`<br>`Bar`                                     | `"Foo"`   | `None`          | `"# Foo\nBar"` | `"<h1>Foo</h1>\n<p>Bar</p>\n"` |
| `+++`<br>`description = "Bar"`<br>`+++`<br>`# Foo`   | `"Foo"`   | `Some("Bar")`   | `"# Foo"`      | `"<h1>Foo</h1>\n"`             |
| `---`<br>`title: Foo`<br>`description: Bar`<br>`---` | `"Foo"`   | `Some("Bar")`   | `""`           | `""`                           |

## Features

- `yaml`: enable YAML frontmatter support
- `toml`: enable TOML frontmatter support
- `json`: enable JSON frontmatter support
- `full`: enable all formats (`json`, `toml`, `yaml`) - enabled by default

## Reading a page

`FlatPage::by_url` returns `Ok(None)` for invalid URLs and missing pages. It
returns `Err` only for I/O failures and frontmatter parsing errors.

```rust,no_run
let root_folder = "./pages";
if let Some(home) = flatpage::FlatPage::<()>::by_url(root_folder, "/").unwrap() {
    println!("title: {}", home.title);
    println!("description: {:?}", home.description);
    println!("markdown body: {}", home.body);
    println!("html body: {}", home.html());
} else {
    println!("No home page");
}
```

## Extra frontmatter fields

You can define extra statically typed frontmatter fields.

```rust,no_run
#[derive(Debug, serde::Deserialize)]
struct Extra {
    slug: String,
}

let _page = flatpage::FlatPage::<Extra>::by_url("./pages", "/").unwrap();
```

## Cached metadata

It's common for a page to have a list of related pages. To avoid reading all
the files each time, you can use [`FlatPageStore`] to cache page [`metadata`]
(titles and descriptions).

`FlatPageStore::read_dir` scans the root folder recursively. Its `pages` map is
keyed by normalized URLs such as `/`, `/guides/`, and `/guides/install`.

```rust,no_run
let root_folder = "./pages";
let store = flatpage::FlatPageStore::read_dir(root_folder).unwrap();
if let Some(meta) = store.meta_by_url("/") {
    println!("title: {}", meta.title);
    println!("description: {:?}", meta.description);
} else {
    println!("No home page");
}
```

[`title`]: FlatPage::title
[`description`]: FlatPage::description
[`body`]: FlatPage::body
[`html()`]: FlatPage::html()
[`metadata`]: FlatPageMeta

## Contributing

Please run [.pre-commit.sh] before sending a PR, it will check everything.

## License

This project is licensed under the [MIT license][license].

[.pre-commit.sh]: https://github.com/imbolc/flatpage/blob/main/.pre-commit.sh
[license]: https://github.com/imbolc/flatpage/blob/main/LICENSE
