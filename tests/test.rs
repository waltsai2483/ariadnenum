use std::{io::{Error, ErrorKind}};

use ariadne::{Color, Config, Report, ReportKind, Source};
use ariadnenum::Ariadnenum;

#[derive(Ariadnenum)]
enum LexingError {
    #[report(
        kind = ariadne::ReportKind::Error, // Default = ReportKind::Error
        config = ariadne::Config::new().with_index_type(ariadne::IndexType::Byte), // Default = None
        code = 300 // Default = None
    )]
    #[message("Unexpected closing bracket")] // Error message 
    #[note("remove this bracket")] // Note message below
    BracketMismatch {
        #[colored(ariadne::Color::Yellow)] // Place #[colored] before #[label] to change the
                                           // color of the label, default = Color::Red
        #[label("Bracket {} is here", kind)] // Label "Bracket {kind} is here" pointing at {location}
        #[here] // Determine error main location
        location: std::ops::Range<usize>,
        kind: char,
    },
    
    #[report(
        kind = ariadne::ReportKind::Warning,
    )]
    #[message("Unused Semicolon: '{}'", arg1)] // Get unnamed argument using 'arg{k}' format
    #[note("remove this semicolon")]
    UnusedSemicolon ( // Unnamed variants are supported
        #[colored(ariadne::Color::Yellow)]
        #[label("Here")] 
        #[here]
        std::ops::Range<usize>,
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
