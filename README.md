# Brokk's JavaScript Grammar for Tree-sitter

[![CI][ci]](https://github.com/BrokkAi/tree-sitter-javascript/actions/workflows/ci.yml)
[![crates][crates]](https://crates.io/crates/brokk-tree-sitter-javascript)
[![docs.rs][docs]](https://docs.rs/brokk-tree-sitter-javascript)

This is the **Brokk-owned and independently maintained fork** of
[`tree-sitter/tree-sitter-javascript`](https://github.com/tree-sitter/tree-sitter-javascript),
a JavaScript and JSX grammar for [Tree-sitter](https://tree-sitter.github.io/tree-sitter/).
Brokk maintains this fork for its code-intelligence tooling and publishes the
Rust package as
[`brokk-tree-sitter-javascript`](https://crates.io/crates/brokk-tree-sitter-javascript).
It intentionally diverges where Brokk needs language support that is not yet
available upstream. Unless you specifically need Brokk's changes, you may
prefer the upstream project.

## Installation

Add the Brokk-maintained JavaScript crate to your project:

```sh
cargo add brokk-tree-sitter-javascript@=0.25.1
```

Or add it directly to `Cargo.toml`:

```toml
[dependencies]
brokk-tree-sitter-javascript = "=0.25.1"
```

The npm and Python bindings retain their upstream-compatible package names but
are not published by this fork. The Brokk-namespaced Rust crate prefixes its
native symbols, so it can coexist with the upstream `tree-sitter-javascript`
crate in one executable.

This grammar intends to be a close approximation of the [ECMAScript](https://ecma-international.org/publications-and-standards/standards/ecma-262/)
specification, with some extensions to support JSX syntax. We try to support the
latest version of the spec, though it is possible that some very new features may
not be supported yet.

## References

- [The ESTree Spec](https://github.com/estree/estree)
- [The ECMAScript 2025 Spec](https://tc39.es/ecma262/2025/)

[ci]: https://img.shields.io/github/actions/workflow/status/BrokkAi/tree-sitter-javascript/ci.yml?logo=github&label=CI
[crates]: https://img.shields.io/crates/v/brokk-tree-sitter-javascript?logo=rust
[docs]: https://img.shields.io/docsrs/brokk-tree-sitter-javascript?logo=docs.rs
