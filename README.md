[![License](https://img.shields.io/crates/l/flatpage.svg)](https://choosealicense.com/licenses/mit/)
[![Crates.io](https://img.shields.io/crates/v/flatpage.svg)](https://crates.io/crates/flatpage)
[![Docs.rs](https://docs.rs/flatpage/badge.svg)](https://docs.rs/flatpage)

A simple file system based markdown page loader.

## Folder structure

Only characters allowed in url segments are ASCII letters, numbers, hyphen,
underscore and dot. Urls map to nested markdown files, and `index.md` is used
for `/` and folder index pages. Empty path segments plus `.` and `..` are
rejected.

| url             | file name               |
| --------------- | ----------------------- |
| `/`             | `index.md`              |
| `/foo/bar-baz`  | `foo/bar-baz.md`        |
| `/foo/bar-baz/` | `foo/bar-baz/index.md`  |

## Page format

File could provide title and description in frontmatter. `flatpage` proxies
`markdown-frontmatter` features, so you can parse YAML (`---`), TOML (`+++`)
and JSON (`{ ... }`) depending on enabled features. If there's no frontmatter
the first line would be considered the title (and cleaned from possible header
marker `#`).

| File content                                         | [`title`] | [`description`] | [`body`]       | [`html()`]                     |
| ---------------------------------------------------- | --------- | --------------- | -------------- | ------------------------------ |
| `# Foo`<br>`Bar`                                     | `"Foo"`   | `None`          | `"# Foo\nBar"` | `"<h1>Foo</h1>\n<p>Bar</p>\n"` |
| `---`<br>`description: Bar`<br>`---`<br>`# Foo`      | `"Foo"`   | `Some("Bar")`   | `"# Foo"`      | `"<h1>Foo</h1>\n"`             |
| `---`<br>`title: Foo`<br>`description: Bar`<br>`---` | `"Foo"`   | `Some("Bar")`   | `""`           | `""`                           |

## Features

- `yaml`: enable YAML frontmatter support
- `toml`: enable TOML frontmatter support
- `json`: enable JSON frontmatter support
- `full`: enable all formats (`json`, `toml`, `yaml`) - enabled by default

## Reading a page

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

You can define extra statically typed frontmatter fields

```rust,no_run
#[derive(Debug, serde::Deserialize)]
struct Extra {
    slug: String,
}

let _page = flatpage::FlatPage::<Extra>::by_url("./pages", "/").unwrap();
```

## Cached metadata

It's a common for a page to have a list of related pages. To avoid reading all
the files each time, you can use [`FlatPageStore`] to cache pages [`metadata`]
(titles and descriptions).

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
