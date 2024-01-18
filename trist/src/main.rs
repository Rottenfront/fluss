fn main() {
    let highlight_names = [
        "attribute",
        "constant",
        "function.builtin",
        "function",
        "keyword",
        "operator",
        "property",
        "punctuation",
        "punctuation.bracket",
        "punctuation.delimiter",
        "string",
        "string.special",
        "tag",
        "type",
        "type.builtin",
        "variable",
        "variable.builtin",
        "variable.parameter",
    ];

    use tree_sitter_highlight::Highlighter;

    let mut highlighter = Highlighter::new();
    use tree_sitter_highlight::HighlightConfiguration;

    let rust_language = tree_sitter_rust::language();

    let mut rust_config = HighlightConfiguration::new(
        rust_language,
        tree_sitter_rust::HIGHLIGHT_QUERY,
        tree_sitter_rust::INJECTIONS_QUERY,
        tree_sitter_rust::TAGGING_QUERY,
    )
    .unwrap();

    rust_config.configure(&highlight_names);
    use tree_sitter_highlight::HighlightEvent;

    let highlights = highlighter
        .highlight(
            &rust_config,
            b"fn main() { println!(\"Hello, World!\"); }",
            None,
            |_| None,
        )
        .unwrap();

    for event in highlights {
        match event.unwrap() {
            HighlightEvent::Source { start, end } => {
                eprintln!("source: {}-{}", start, end);
            }
            HighlightEvent::HighlightStart(s) => {
                eprintln!("highlight style started: {:?}", s);
            }
            HighlightEvent::HighlightEnd => {
                eprintln!("highlight style ended");
            }
        }
    }
}
