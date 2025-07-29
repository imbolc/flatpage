[![License](https://img.shields.io/crates/l/flatpage.svg)](https://choosealicense.com/licenses/mit/)
[![Crates.io](https://img.shields.io/crates/v/flatpage.svg)](https://crates.io/crates/flatpage)
[![Docs.rs](https://docs.rs/flatpage/badge.svg)](https://docs.rs/flatpage)

A simple file system based markdown flat page.

## Folder structure

Only characters allowed in urls are ASCII, numbers and hyphen with underscore.
Urls map to files by simply substituting `/` to `^` and adding `.md` extension.
I believe it should eliminate all kinds of security issues.

| url            | file name         |
| -------------- | ----------------- |
| `/`            | `^.md`            |
| `/foo/bar-baz` | `^foo^bar-baz.md` |

## Page format

File could provide title and description in a yaml-based frontmatter, if there's
no frontmatter the first line would be considered the title (and cleaned from
possible header marker `#`).

| File content                                         | [`title`] | [`description`] | [`body`]       | [`html()`]                     |
| ---------------------------------------------------- | --------- | --------------- | -------------- | ------------------------------ |
| `# Foo`<br>`Bar`                                     | `"Foo"`   | `None`          | `"# Foo\nBar"` | `"<h1>Foo</h1>\n<p>Bar</p>\n"` |
| `---`<br>`description: Bar`<br>`---`<br>`# Foo`      | `"Foo"`   | `Some("Bar")`   | `"# Foo"`      | `"<h1>Foo</h1>\n"`             |
| `---`<br>`title: Foo`<br>`description: Bar`<br>`---` | `"Foo"`   | `Some("Bar")`   | `""`           | `""`                           |

## Reading a page

```rust
let root_folder = "./";
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

```rust
#[derive(Debug, serde::Deserialize)]
struct Extra {
    slug: String,
}

let _page = flatpage::FlatPage::<Extra>::by_url("./", "/").unwrap();
```

## Cached metadata

It's a common for a page to have a list of related pages. To avoid reading all
the files each time, you can use [`FlatPageStore`] to cache pages [`metadata`]
(titles and descriptions).

```rust
let root_folder = "./";
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
