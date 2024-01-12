[![License](https://img.shields.io/crates/l/flatpage.svg)](https://choosealicense.com/licenses/mit/)
[![Crates.io](https://img.shields.io/crates/v/flatpage.svg)](https://crates.io/crates/flatpage)
[![Docs.rs](https://docs.rs/flatpage/badge.svg)](https://docs.rs/flatpage)

<!-- cargo-sync-readme start -->

A simple file system based markdown flat page.

## Folder structure

Only characters allowed in urls are ASCII, numbers and hyphen with underscore.
Urls map to files by simply substituting `/` to `^` and adding `.md` extension.
I believe it should eliminate all kinds of security issues.

| url            | file name         |
|----------------|-------------------|
| `/`            | `^.md`            |
| `/foo/bar-baz` | `^foo^bar-baz.md` |

## Page format

File could provide title and description in a yaml-based frontmatter, if there's no frontmatter
the first line would be considered the title (and cleaned from possible header marker `#`).

| File content                                         | [`title`] | [`description`] | [`body`] | [`html()`]           |
|------------------------------------------------------|---------------------|---------------------------|--------------------|--------------------------------|
| `# Foo`<br>`Bar`                                     | `"Foo"`             | `None`                    | `"# Foo\nBar"`     | `"<h1>Foo</h1>\n<p>Bar</p>\n"` |
| `---`<br>`description: Bar`<br>`---`<br>`# Foo`      | `"Foo"`             | `Some("Bar")`             | `"# Foo"`          | `"<h1>Foo</h1>\n"`             |
| `---`<br>`title: Foo`<br>`description: Bar`<br>`---` | `"Foo"`             | `Some("Bar")`             | `""`               | `""`                           |


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

## Cached metadata

It's a common for a page to have a list of related pages. To avoid reading all the files each
time, you can use [`FlatPageStore`] to cache pages [`metadata`] (titles and descriptions).

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

<!-- cargo-sync-readme end -->

## Contributing

We appreciate all kinds of contributions, thank you!


### Note on README

Most of the readme is automatically copied from the crate documentation by [cargo-sync-readme][].
This way the readme is always in sync with the docs and examples are tested.

So if you find a part of the readme you'd like to change between `<!-- cargo-sync-readme start -->`
and `<!-- cargo-sync-readme end -->` markers, don't edit `README.md` directly, but rather change
the documentation on top of `src/lib.rs` and then synchronize the readme with:
```bash
cargo sync-readme
```
(make sure the cargo command is installed):
```bash
cargo install cargo-sync-readme
```

If you have [rusty-hook] installed the changes will apply automatically on commit.


## License

This project is licensed under the [MIT license](LICENSE).

[cargo-sync-readme]: https://github.com/phaazon/cargo-sync-readme
[rusty-hook]: https://github.com/swellaby/rusty-hook
