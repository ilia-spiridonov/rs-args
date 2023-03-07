pub use parser::{ArgParser, ArgParserError, ArgParserMode, ParsedArg};

mod option;
mod parser;

pub enum OptionalArgKind {
    Flag,
    RequiredValue,
    OptionalValue,
}

pub struct OptionalArg {
    pub(crate) kind: OptionalArgKind,
    pub(crate) multiple: bool,
}
