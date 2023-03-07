use super::{OptionalArg, OptionalArgKind};
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

#[derive(Debug, PartialEq)]
pub enum ArgParserError {
    InvalidOptionName { name: &'static str },
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
    pub fn add_option(
        &mut self,
        name: &'static str,
        option: OptionalArg,
    ) -> Result<(), ArgParserError> {
        if !OptionalArg::is_valid(name) {
            return Err(ArgParserError::InvalidOptionName { name });
        }

        self.options.insert(name, option);

        Ok(())
    }
}

#[test]
fn test_add_option() {
    let mut parser = ArgParser::default();
    let get_opt_arg = || OptionalArg::new(OptionalArgKind::Flag, false);

    assert_eq!(Ok(()), parser.add_option("foo-bar", get_opt_arg()));
    assert_eq!(
        Err(ArgParserError::InvalidOptionName { name: "--baz" }),
        parser.add_option("--baz", get_opt_arg())
    );
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
