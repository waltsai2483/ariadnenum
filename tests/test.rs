use std::{
    alloc::alloc,
    fmt::Debug,
    ops::{Range, RangeInclusive},
};

use ariadne::{Color, Config, Report, ReportKind, Source};
use ariadnenum::Ariadnenum;

#[derive(Ariadnenum)]
enum MyError {
    Test,
    #[message("Test named: {}", it)]
    #[report(kind = ReportKind::Warning, kind = ReportKind::Error)]
    #[note("Test!!!")]
    TestNamed {
        it: i32,
        #[here]
        #[colored(Color::Green)]
        #[label("span {}", it)]
        span: Range<usize>,
        #[colored(Color::Yellow)]
        #[label("more span {}", it)]
        more_span: Range<usize>,
    },
    TestUnnamed(
        i32,
        #[here]
        #[colored(Color::Yellow)]
        #[label("span {}", arg0)]
        Range<usize>,
        i32,
    ),
}

#[test]
fn test() {
    MyError::TestNamed { it: 1, span: 31..33, more_span: 9..12 }
        .report()
        .unwrap()
        .eprint(Source::from(r#"fn main() {
    println!("test {}");
}
"#))
        .unwrap()
}
