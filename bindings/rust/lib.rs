//! This crate provides JavaScript language support for the [tree-sitter] parsing library.
//!
//! Typically, you will use the [`LANGUAGE`] constant to add this language to a
//! tree-sitter [`Parser`], and then use the parser to parse some code:
//!
//! ```
//! let code = r#"
//! function double(x) {
//!     return x * 2;
//! }
//! "#;
//! let mut parser = tree_sitter::Parser::new();
//! let language = brokk_tree_sitter_javascript::LANGUAGE;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading JavaScript parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [`Parser`]: https://docs.rs/tree-sitter/0.25.8/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

extern "C" {
    fn brokk_tree_sitter_javascript() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for this grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(brokk_tree_sitter_javascript) };

/// The content of the [`node-types.json`] file for this grammar.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers/6-static-node-types
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

/// The syntax highlighting query for this language.
pub const HIGHLIGHT_QUERY: &str = include_str!("../../queries/highlights.scm");

/// The syntax highlighting query for languages injected into this one.
pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");

/// The syntax highlighting query for JSX.
pub const JSX_HIGHLIGHT_QUERY: &str = include_str!("../../queries/highlights-jsx.scm");

/// The local-variable syntax highlighting query for this language.
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");

/// The symbol tagging query for this language.
pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    fn parse_bytes(language: tree_sitter_language::LanguageFn, source: &[u8]) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    fn has_node_bytes(
        node: tree_sitter::Node<'_>,
        source: &[u8],
        kind: &str,
        expected: &[u8],
    ) -> bool {
        if node.kind() == kind && source.get(node.start_byte()..node.end_byte()) == Some(expected) {
            return true;
        }

        (0..node.child_count())
            .any(|index| has_node_bytes(node.child(index).unwrap(), source, kind, expected))
    }

    fn count_nodes(node: tree_sitter::Node<'_>, kind: &str) -> usize {
        usize::from(node.kind() == kind)
            + (0..node.child_count())
                .map(|index| count_nodes(node.child(index).unwrap(), kind))
                .sum::<usize>()
    }

    fn assert_complete_source(tree: &tree_sitter::Tree, source: &[u8]) {
        let root = tree.root_node();
        assert_eq!(root.start_byte(), 0);
        assert_eq!(root.end_byte(), source.len());
        assert_eq!(root.utf8_text(source).unwrap().as_bytes(), source);
    }

    fn assert_node_bytes(tree: &tree_sitter::Tree, source: &[u8], kind: &str, expected: &[u8]) {
        assert!(
            has_node_bytes(tree.root_node(), source, kind, expected),
            "missing {kind} node with exact bytes {expected:?}: {}",
            tree.root_node().to_sexp()
        );
    }

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading JavaScript parser");
    }

    #[test]
    fn test_can_coexist_with_the_upstream_javascript_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Brokk JavaScript parser");
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Error loading upstream JavaScript parser");
    }

    #[test]
    fn test_raw_nul_content_is_preserved() {
        // NUL in a quoted module specifier is still string content. This checks
        // syntax parsing only; it makes no claim about resolving that module.
        let cases: &[(&[u8], &str, &[u8])] = &[
            (
                b"const double_raw = \"before\0after\";\nconst after = 1;",
                "string",
                b"\"before\0after\"",
            ),
            (
                b"const single_raw = 'before\0after';\nconst after = 1;",
                "string",
                b"'before\0after'",
            ),
            (
                b"const double_escape = \"before\\0after\";\nconst after = 1;",
                "string",
                b"\"before\\0after\"",
            ),
            (
                b"const single_escape = 'before\\0after';\nconst after = 1;",
                "string",
                b"'before\\0after'",
            ),
            (
                b"const slash_nul = \"before\\\0after\";\nconst after = 1;",
                "escape_sequence",
                b"\\\0",
            ),
            (
                b"const adjacent = \"\0\0\";\nconst after = 1;",
                "string",
                b"\"\0\0\"",
            ),
            (
                b"const only_nul = '\0';\nconst after = 1;",
                "string",
                b"'\0'",
            ),
            (
                b"// before\0after\nconst after = 1;",
                "comment",
                b"// before\0after",
            ),
            (b"//\0\nconst after = 1;", "comment", b"//\0"),
            (
                b"const block = 1; /* before\0after */\nconst after = 1;",
                "comment",
                b"/* before\0after */",
            ),
            (b"/*\0*/\nconst after = 1;", "comment", b"/*\0*/"),
            (b"<!--\0-->\nconst after = 1;", "html_comment", b"<!--\0-->"),
            (
                b"const template = `before\0after`;\nconst after = 1;",
                "template_string",
                b"`before\0after`",
            ),
            (
                b"const only_nul_template = `\0`;\nconst after = 1;",
                "template_string",
                b"`\0`",
            ),
            (
                b"const template = `before\0${value}after`;\nconst after = 1;",
                "string_fragment",
                b"before\0",
            ),
            (
                b"import imported from \"pkg\0suffix\";\nconst after = 1;",
                "string",
                b"\"pkg\0suffix\"",
            ),
            (
                b"const loaded = import(\"pkg\0suffix\");\nconst after = 1;",
                "string",
                b"\"pkg\0suffix\"",
            ),
        ];

        for language in [super::LANGUAGE] {
            for (source, kind, raw_node) in cases {
                let tree = parse_bytes(language, source);
                assert!(
                    !tree.root_node().has_error(),
                    "unexpected parse error for {source:?}: {}",
                    tree.root_node().to_sexp()
                );
                assert_complete_source(&tree, source);
                assert_node_bytes(&tree, source, kind, raw_node);
                assert_node_bytes(&tree, source, "lexical_declaration", b"const after = 1;");

                if source.starts_with(b"const template = `before\0${") {
                    assert_eq!(count_nodes(tree.root_node(), "template_substitution"), 1);
                }
            }
        }

        let line_comment_at_eof = b"// before\0";
        for language in [super::LANGUAGE] {
            let tree = parse_bytes(language, line_comment_at_eof);
            assert!(
                !tree.root_node().has_error(),
                "{}",
                tree.root_node().to_sexp()
            );
            assert_complete_source(&tree, line_comment_at_eof);
            assert_node_bytes(&tree, line_comment_at_eof, "comment", line_comment_at_eof);
        }
    }

    #[test]
    fn test_raw_nul_is_preserved_in_jsx_attributes_and_text() {
        let cases: &[(&[u8], &str, &[u8])] = &[
            (
                b"const view = <Widget title=\"before\0after\" />;\nconst after = 1;",
                "string",
                b"\"before\0after\"",
            ),
            (
                b"const view = <p>before\0after</p>;\nconst after = 1;",
                "jsx_text",
                b"before\0after",
            ),
            (
                b"const view = <Widget title=\"&\0x\" />;\nconst after = 1;",
                "string",
                b"\"&\0x\"",
            ),
            (
                b"const view = <Widget title='&\0' />;\nconst after = 1;",
                "string",
                b"'&\0'",
            ),
        ];

        for (source, kind, raw_node) in cases {
            let tree = parse_bytes(super::LANGUAGE, source);
            assert!(
                !tree.root_node().has_error(),
                "unexpected JSX parse error for {source:?}: {}",
                tree.root_node().to_sexp()
            );
            assert_complete_source(&tree, source);
            assert_node_bytes(&tree, source, kind, raw_node);
            assert_node_bytes(&tree, source, "lexical_declaration", b"const after = 1;");
            assert_eq!(count_nodes(tree.root_node(), "html_character_reference"), 0);
        }
    }

    #[test]
    fn test_raw_nul_in_executable_syntax_is_an_error() {
        let cases: &[&[u8]] = &[
            b"const na\0me = 1;\nconst after = 2;",
            b"const result = left\0+ right;\nconst after = 2;",
            b"const result = `raw\0${left\0+right}`;\nconst after = 2;",
            b"const loaded = import\0(\"pkg\");\nconst after = 2;",
            b"import\0 imported from \"pkg\";\nconst after = 2;",
        ];

        for language in [super::LANGUAGE] {
            for source in cases {
                let tree = parse_bytes(language, source);
                assert!(
                    tree.root_node().has_error(),
                    "unexpectedly accepted executable NUL in {source:?}: {}",
                    tree.root_node().to_sexp()
                );
                assert_complete_source(&tree, source);
                assert_node_bytes(&tree, source, "lexical_declaration", b"const after = 2;");
            }
        }
    }

    #[test]
    fn test_unterminated_raw_nul_content_errors_at_eof() {
        let cases: &[&[u8]] = &[
            b"const broken = \"before\0after",
            b"const broken = 'before\0after",
            b"const broken = 1; /* before\0after",
            b"const broken = `before\0after",
        ];

        for language in [super::LANGUAGE] {
            for source in cases {
                let tree = parse_bytes(language, source);
                assert!(
                    tree.root_node().has_error(),
                    "unterminated raw-NUL content unexpectedly parsed: {source:?}: {}",
                    tree.root_node().to_sexp()
                );
                assert_complete_source(&tree, source);
            }
        }
    }

    #[test]
    fn test_incremental_raw_nul_edits_match_fresh_parses() {
        let content_cases: &[(&[u8], &[u8], &[u8])] = &[
            (b"const text = \"A\";\nconst tail = 1;", b"\"\0\"", b"\"A\""),
            (b"/*A*/\nconst tail = 1;", b"/*\0*/", b"/*A*/"),
            (b"const text = `A${value}`;\nconst tail = 1;", b"\0", b"A"),
        ];

        for language in [super::LANGUAGE] {
            for (source, nul_node, original_node) in content_cases {
                let mut parser = tree_sitter::Parser::new();
                parser.set_language(&language.into()).unwrap();
                let offset = source.iter().position(|byte| *byte == b'A').unwrap();
                let old_tree = parser.parse(*source, None).unwrap();
                assert!(!old_tree.root_node().has_error());

                let mut with_nul = source.to_vec();
                with_nul[offset] = 0;
                let incremental =
                    parse_after_single_byte_edit(&mut parser, &old_tree, &with_nul, offset);
                let fresh = parser.parse(with_nul.as_slice(), None).unwrap();
                assert!(!incremental.root_node().has_error());
                assert_complete_source(&incremental, &with_nul);
                assert_incremental_tree_matches_fresh(&incremental, &fresh, &with_nul);
                let node_kind = if source.starts_with(b"/*") {
                    "comment"
                } else if source.starts_with(b"const text = `") {
                    "string_fragment"
                } else {
                    "string"
                };
                assert_node_bytes(&incremental, &with_nul, node_kind, nul_node);

                let mut restored = with_nul.clone();
                restored[offset] = b'A';
                let incremental =
                    parse_after_single_byte_edit(&mut parser, &incremental, &restored, offset);
                let fresh = parser.parse(restored.as_slice(), None).unwrap();
                assert!(!incremental.root_node().has_error());
                assert_complete_source(&incremental, &restored);
                assert_incremental_tree_matches_fresh(&incremental, &fresh, &restored);
                assert_node_bytes(&incremental, &restored, node_kind, original_node);
            }

            let identifier_source = b"const name = 1;\nconst tail = 1;";
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language.into()).unwrap();
            let offset = identifier_source
                .iter()
                .position(|byte| *byte == b'm')
                .unwrap();
            let old_tree = parser.parse(identifier_source.as_slice(), None).unwrap();
            assert!(!old_tree.root_node().has_error());
            let mut invalid_identifier = identifier_source.to_vec();
            invalid_identifier[offset] = 0;
            let incremental =
                parse_after_single_byte_edit(&mut parser, &old_tree, &invalid_identifier, offset);
            let fresh = parser.parse(invalid_identifier.as_slice(), None).unwrap();
            assert!(incremental.root_node().has_error());
            assert_complete_source(&incremental, &invalid_identifier);
            assert_incremental_tree_matches_fresh(&incremental, &fresh, &invalid_identifier);
            assert_node_bytes(
                &incremental,
                &invalid_identifier,
                "lexical_declaration",
                b"const tail = 1;",
            );
        }
    }

    fn parse_after_single_byte_edit(
        parser: &mut tree_sitter::Parser,
        old_tree: &tree_sitter::Tree,
        new_source: &[u8],
        offset: usize,
    ) -> tree_sitter::Tree {
        let mut edited_tree = old_tree.clone();
        edited_tree.edit(&tree_sitter::InputEdit {
            start_byte: offset,
            old_end_byte: offset + 1,
            new_end_byte: offset + 1,
            start_position: tree_sitter::Point {
                row: 0,
                column: offset,
            },
            old_end_position: tree_sitter::Point {
                row: 0,
                column: offset + 1,
            },
            new_end_position: tree_sitter::Point {
                row: 0,
                column: offset + 1,
            },
        });
        parser.parse(new_source, Some(&edited_tree)).unwrap()
    }

    fn assert_incremental_tree_matches_fresh(
        incremental: &tree_sitter::Tree,
        fresh: &tree_sitter::Tree,
        source: &[u8],
    ) {
        let incremental_root = incremental.root_node();
        let fresh_root = fresh.root_node();
        assert_eq!(incremental_root.to_sexp(), fresh_root.to_sexp());
        assert_eq!(incremental_root.start_byte(), fresh_root.start_byte());
        assert_eq!(incremental_root.end_byte(), fresh_root.end_byte());
        assert_eq!(incremental_root.has_error(), fresh_root.has_error());
        assert_eq!(
            incremental_root.utf8_text(source).unwrap(),
            fresh_root.utf8_text(source).unwrap()
        );
    }
}
