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

#[derive(Debug, PartialEq)]
pub enum ParsedArg {
    Positional { value: String },
    Flag { name: &'static str, value: bool },
    RequiredValue { name: String, value: String },
    OptionalValue { name: String, value: Option<String> },
}

#[derive(Debug, PartialEq)]
pub enum ArgParserError {
    InvalidOption { name: String },
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
            return Err(InvalidOption {
                name: name.to_string(),
            });
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
        Err(InvalidOption {
            name: "--foo".to_string()
        }),
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
    pub fn parse_args(&self) -> ArgParseResult {
        let args = env::args().skip(1).collect::<Vec<_>>();
        let str_args = args.iter().map(|s| &s[..]).collect::<Vec<_>>();

        self.parse(&str_args)
    }

    pub fn parse(&self, args: &[&str]) -> ArgParseResult {
        use ParsedArg::*;

        let mut parsed_args = vec![];
        let mut parse_options = true;

        for arg in args {
            if *arg == "--" && parse_options {
                parse_options = false;
                continue;
            }

            if parse_options {
                if let Some((name_or_alias, value)) = self.parse_option(arg)? {
                    let (name, option) = self.resolve(name_or_alias)?;

                    parsed_args.push(Flag { name, value: true });
                    continue;
                }
            }

            parsed_args.push(Positional {
                value: arg.to_string(),
            });
        }

        Ok(parsed_args)
    }

    fn parse_option<'a>(&self, arg: &'a str) -> Result<Option<(&'a str, &'a str)>, ArgParserError> {
        use ArgParserError::*;

        if let Some(name) = arg.strip_prefix("--") {
            if !OptionalArg::is_valid(name) {
                return Err(InvalidOption {
                    name: name.to_string(),
                });
            }

            return Ok(Some((name, "")));
        }

        Ok(None)
    }

    fn resolve(
        &self,
        name_or_alias: &str,
    ) -> Result<(&&'static str, &OptionalArg), ArgParserError> {
        self.options
            .get_key_value(name_or_alias)
            .ok_or(ArgParserError::UnknownOption {
                name: name_or_alias.to_string(),
            })
    }
}

#[test]
fn test_parse() -> Result<(), ArgParserError> {
    use ArgParserError::*;
    use ParsedArg::*;

    let mut parser = ArgParser::default();
    parser.add_option("foo", OptionalArg::new(OptionalArgKind::Flag, false), None)?;

    assert_eq!(
        Ok(vec![
            Positional {
                value: "foo".to_string()
            },
            Positional {
                value: "bar".to_string()
            }
        ]),
        parser.parse(&["foo", "bar"])
    );
    assert_eq!(
        Err(InvalidOption {
            name: "-foo".to_string()
        }),
        parser.parse(&["---foo"])
    );
    assert_eq!(
        Err(UnknownOption {
            name: "Foo".to_string()
        }),
        parser.parse(&["--Foo"])
    );
    assert_eq!(
        Ok(vec![
            Flag {
                name: "foo",
                value: true
            },
            Positional {
                value: "--".to_string()
            },
            Positional {
                value: "--foo".to_string()
            }
        ]),
        parser.parse(&["--foo", "--", "--", "--foo"])
    );

    Ok(())
}
