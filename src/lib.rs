pub use parser::{ArgParser, ArgParserError, ArgParserMode, ParsedArg};
pub use selector::ArgSelector;

mod option;
mod parser;
mod selector;

#[derive(Debug, PartialEq)]
pub enum OptionalArgKind {
    Flag,
    RequiredValue,
    OptionalValue,
}

#[derive(Debug, PartialEq)]
pub struct OptionalArg {
    pub name: &'static str,
    pub alias: Option<&'static str>,
    pub kind: OptionalArgKind,
    pub multiple: bool,
}
