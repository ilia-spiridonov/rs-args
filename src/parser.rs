use std::{collections::HashMap, env};

pub enum OptionalArgKind {
    Flag,
    RequiredValue,
    OptionalValue,
}

pub struct OptionalArg {
    pub(crate) kind: OptionalArgKind,
    pub(crate) multiple: bool,
}

pub enum ArgParsingMode {
    Mixed,
    OptionsFirst,
}

pub struct ArgParser {
    pub(crate) mode: ArgParsingMode,
    pub(crate) aliases: HashMap<&'static str, &'static str>,
    pub(crate) options: HashMap<&'static str, OptionalArg>,
}

pub enum ParsedArg {
    Positional { value: String },
    Flag { name: String, value: bool },
    RequiredValue { name: String, value: String },
    OptionalValue { name: String, value: Option<String> },
}

pub enum ArgParserError {
    InvalidOptionName { name: String },
    UnknownOption { name: String },
}

type ArgParseResult = Result<Vec<ParsedArg>, ArgParserError>;

impl ArgParser {
    pub fn new(mode: ArgParsingMode) -> Self {
        Self {
            mode,
            aliases: HashMap::new(),
            options: HashMap::new(),
        }
    }

    pub fn parse(args: &[&str]) -> ArgParseResult {
        Ok(vec![])
    }

    pub fn parse_args() -> ArgParseResult {
        let args = env::args().skip(1).collect::<Vec<_>>();

        Self::parse(&args.iter().map(|s| &s[..]).collect::<Vec<_>>())
    }
}
