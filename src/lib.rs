pub use parser::{ArgParser, ArgParserError, ArgParserMode, ParsedArg};
pub use selector::ArgSelector;

mod option;
mod parser;
mod selector;

pub(crate) enum OptionalArgKind {
    Flag,
    RequiredValue,
    OptionalValue,
}

pub struct OptionalArg {
    pub(crate) name: &'static str,
    pub(crate) alias: Option<&'static str>,
    pub(crate) kind: OptionalArgKind,
    pub(crate) multiple: bool,
}
