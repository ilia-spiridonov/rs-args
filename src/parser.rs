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
    InvalidOption { name: &'static str },
    DuplicateOption { name: &'static str },
    InvalidAlias { alias: &'static str },
    DuplicateAlias { alias: &'static str },
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
        alias: Option<&'static str>,
    ) -> Result<(), ArgParserError> {
        use ArgParserError::*;

        if !OptionalArg::is_valid(name) {
            return Err(InvalidOption { name });
        }

        if self.options.contains_key(name) {
            return Err(DuplicateOption { name });
        }

        if let Some(alias) = alias {
            if !OptionalArg::is_valid_alias(alias) {
                return Err(InvalidAlias { alias });
            }

            if self.aliases.contains_key(alias) {
                return Err(DuplicateAlias { alias });
            }

            self.aliases.insert(alias, name);
        }

        self.options.insert(name, option);

        Ok(())
    }
}

#[test]
fn test_add_option() {
    use ArgParserError::*;

    let mut parser = ArgParser::default();
    let get_opt_arg = || OptionalArg::new(OptionalArgKind::Flag, false);

    assert_eq!(
        Err(InvalidOption { name: "--foo" }),
        parser.add_option("--foo", get_opt_arg(), None)
    );
    assert_eq!(
        Err(InvalidAlias { alias: "?" }),
        parser.add_option("foo", get_opt_arg(), Some("?"))
    );
    assert_eq!(Ok(()), parser.add_option("foo", get_opt_arg(), Some("f")));
    assert_eq!(
        Err(DuplicateOption { name: "foo" }),
        parser.add_option("foo", get_opt_arg(), None)
    );
    assert_eq!(
        Err(DuplicateAlias { alias: "f" }),
        parser.add_option("bar", get_opt_arg(), Some("f"))
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
