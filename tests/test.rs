use std::{io::{Error, ErrorKind}, ops::Range};

use ariadne::{Color, Config, Report, ReportKind, Source};
use ariadnenum::Ariadnenum;

#[derive(Ariadnenum)]
enum LexingError {
    #[report(
        kind = ariadne::ReportKind::Error, // Optional, default = ReportKind::Error
        config = ariadne::Config::new().with_index_type(ariadne::IndexType::Byte), // Optional, default = None
        code = 300 // Optional, default = None
    )]
    #[message("Unexpected closing bracket: '{}'", kind)] // Error message 
    #[note("remove this closing bracket")] // Note message below
    BracketMismatch {
        #[colored(ariadne::Color::Yellow)] // Place #[colored] before #[label] to change the color of the label
        #[label("Bracket {} is here", kind)] // Label message will be "Bracket {kind} is here" pointing at {location}
        #[here] // Determine error main location
        location: Range<usize>,
        kind: char,
    },
    
    #[report(
        kind = ariadne::ReportKind::Warning,
    )]
    #[message("Unused Semicolon")]
    #[note("remove this closing bracket")]
    UnusedSemicolon ( // Unnamed variants are supported
        #[colored(ariadne::Color::Yellow)]
        #[label("Here")] 
        #[here]
        Range<usize>,
        char,
    ),
}

#[test]
fn test() {
    let source = 
    r#"fn main() {
        println!("Hello, world!"));
    }"#;

    let result: Result<(), std::io::Error> = LexingError::BracketMismatch {
        location: 45..46,
        kind: ')'
    }.eprint_report("target.rs", Source::from(source.to_string())); // Print error report to stderr
}
