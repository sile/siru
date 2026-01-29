siru
====

[![siru](https://img.shields.io/crates/v/siru.svg)](https://crates.io/crates/siru)
[![Documentation](https://docs.rs/siru/badge.svg)](https://docs.rs/siru)
[![Actions Status](https://github.com/sile/siru/workflows/CI/badge.svg)](https://github.com/sile/siru/actions)
![License](https://img.shields.io/crates/l/siru)

A command-line tool to find and view Rust crate documentation from [rustdoc JSON output][rustdoc-json].

Output is formatted as Markdown, making it easy to pipeline with other commands like `less`, `bat` for paging and syntax highlighting, or save to a file for viewing in your favorite Markdown editor.

"siru" means "知る(to know)" in Japanese.

## Installation

```console
$ cargo install siru

$ siru -h
Usage: siru [OPTIONS] [ITEM_PATH_PART]..

Arguments:
  [ITEM_PATH_PART]... Filter items to only those having all specified path parts

Options:
      --version                             Print version
  -h, --help                                Print help ('--help' for full help, '-h' for summary)
  -x, --ext                                 Enable extended subcommands
  -d, --doc-path <PATH[:PATH]*>             Path(s) to doc files or dirs containing *.json files, separated by colons [env: SIRU_DOC_PATH] [default: target/doc/]
  -c, --crate <CRATE_NAME>                  Filter to specific crate(s) by name (can be specified multiple times)
  -k, --kind <mod|enum|struct|trait|fn|...> Filter to specific item kind(s) (can be specified multiple times)
      --show-inner-json                     Print inner JSON representation before item signature
      --verbose                             Enable verbose output
```

## Usage

```bash
# Build JSON documentation for the current crate
siru -x build-doc

# View all items in target/doc
siru

# Filter to specific crate
siru -c my_crate

# Filter to functions only
siru -k fn

# Filter by item path
siru HashMap

# Combine multiple filters
siru -c std -k fn -k struct String

# View standard library documentation (requires nightly)
rustup component add --toolchain nightly rust-docs-json
siru HashMap -d ~/.rustup/toolchains/nightly-${TARGET}/share/doc/rust/json/

# Pipe output to pager
siru | less

# Save to file and open in editor
siru > docs.md && $EDITOR docs.md

# Pipe to syntax highlighter
siru | bat --language markdown
```

[rustdoc-json]: https://rust-lang.github.io/rfcs/2963-rustdoc-json.html

