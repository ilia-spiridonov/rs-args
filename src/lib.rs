pub use parser::{ArgParser, ArgParserError, ArgParserMode, ParsedArg};
pub use selector::ArgSelector;

mod option;
mod parser;
mod selector;

pub enum OptionalArgKind {
    Flag,
    RequiredValue,
    OptionalValue,
}

pub struct OptionalArg {
    pub(crate) kind: OptionalArgKind,
    pub(crate) multiple: bool,
}
