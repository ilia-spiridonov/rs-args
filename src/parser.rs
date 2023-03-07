use super::OptionalArg;
use std::{collections::HashMap, env};

pub enum ArgParserMode {
    Mixed,
    OptionsFirst,
}

pub struct ArgParser {
    pub(crate) mode: ArgParserMode,
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
    pub fn new(mode: ArgParserMode) -> Self {
        Self {
            mode,
            aliases: HashMap::new(),
            options: HashMap::new(),
        }
    }
}

impl Default for ArgParser {
    fn default() -> Self {
        Self::new(ArgParserMode::Mixed)
    }
}

impl ArgParser {
    pub fn parse_args() -> ArgParseResult {
        let args = env::args().skip(1).collect::<Vec<_>>();
        let str_args = args.iter().map(|s| &s[..]).collect::<Vec<_>>();

        Self::parse(&str_args)
    }

    pub fn parse(args: &[&str]) -> ArgParseResult {
        Ok(vec![])
    }
}
