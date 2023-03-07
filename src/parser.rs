use super::{OptionalArg, OptionalArgKind};
use std::{
    collections::{HashMap, VecDeque},
    env,
};

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
    Positional {
        value: String,
    },
    Flag {
        name: &'static str,
        value: bool,
    },
    RequiredValue {
        name: &'static str,
        value: String,
    },
    OptionalValue {
        name: &'static str,
        value: Option<String>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ArgParserError {
    InvalidOption { name: String },
    InvalidAlias { alias: String },
    DuplicateOption { name: &'static str },
    DuplicateAlias { alias: &'static str },
    UnknownOption { name: String },
    UnknownAlias { alias: String },
    InvalidOptionValue { name: &'static str, value: String },
    MissingOptionValue { name: &'static str },
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
                return Err(InvalidAlias {
                    alias: alias.to_string(),
                });
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
        Err(InvalidAlias {
            alias: "?".to_string()
        }),
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
        use ArgParserError::*;
        use ParsedArg::*;

        let mut args = VecDeque::from_iter(args.iter().map(|s| s.to_string()));
        let mut parse_options = true;
        let mut parsed_options = HashMap::new();
        let mut parsed_args = vec![];

        while let Some(arg) = args.pop_front() {
            if arg == "--" && parse_options {
                parse_options = false;
                continue;
            }

            if parse_options {
                if let Some((name_or_alias, value)) = self.parse_option(&arg)? {
                    let (name, option) = self.resolve(name_or_alias)?;

                    let value = if OptionalArg::is_valid_alias(name_or_alias) {
                        if let Some(value) = value.strip_prefix('=') {
                            value
                        } else if !value.is_empty() && matches!(option.kind, OptionalArgKind::Flag)
                        {
                            args.push_front(format!("-{}", value));

                            ""
                        } else {
                            value
                        }
                    } else {
                        value
                    };

                    match option.kind {
                        OptionalArgKind::Flag => {
                            if !matches!(value, "" | "true" | "false") {
                                return Err(InvalidOptionValue {
                                    name,
                                    value: value.to_string(),
                                });
                            }

                            parsed_args.push(Flag {
                                name,
                                value: matches!(value, "" | "true"),
                            });
                        }
                        OptionalArgKind::RequiredValue => {
                            let value = if value.is_empty() {
                                args.pop_front().ok_or(MissingOptionValue { name })?
                            } else {
                                value.to_string()
                            };

                            parsed_args.push(RequiredValue { name, value });
                        }
                        OptionalArgKind::OptionalValue => {
                            let value = if value.is_empty() {
                                None
                            } else {
                                Some(value.to_string())
                            };

                            parsed_args.push(OptionalValue { name, value });
                        }
                    };

                    if !option.multiple {
                        if parsed_options.contains_key(name) {
                            return Err(DuplicateOption { name });
                        }

                        parsed_options.insert(name, ());
                    }

                    continue;
                }
            }

            parsed_args.push(Positional {
                value: arg.to_string(),
            });

            if matches!(self.mode, ArgParserMode::OptionsFirst) {
                parse_options = false;
            }
        }

        Ok(parsed_args)
    }

    fn parse_option<'a>(&self, arg: &'a str) -> Result<Option<(&'a str, &'a str)>, ArgParserError> {
        use ArgParserError::*;

        if let Some(name) = arg.strip_prefix("--") {
            let (name, value) = name.split_once('=').unwrap_or((name, ""));

            if !OptionalArg::is_valid(name) {
                return Err(InvalidOption {
                    name: name.to_string(),
                });
            }

            return Ok(Some((name, value)));
        }

        if let Some(alias) = arg.strip_prefix('-') {
            let (alias, value) = if alias.is_char_boundary(1) {
                alias.split_at(1)
            } else {
                (alias, "")
            };

            if !OptionalArg::is_valid_alias(alias) {
                return Err(InvalidAlias {
                    alias: alias.to_string(),
                });
            }

            return Ok(Some((alias, value)));
        }

        Ok(None)
    }

    fn resolve(
        &self,
        name_or_alias: &str,
    ) -> Result<(&&'static str, &OptionalArg), ArgParserError> {
        use ArgParserError::*;

        if OptionalArg::is_valid_alias(name_or_alias) {
            return self.resolve(self.aliases.get(name_or_alias).ok_or(UnknownAlias {
                alias: name_or_alias.to_string(),
            })?);
        }

        self.options
            .get_key_value(name_or_alias)
            .ok_or(UnknownOption {
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
    parser.add_option(
        "bar",
        OptionalArg::new(OptionalArgKind::Flag, true),
        Some("b"),
    )?;
    parser.add_option(
        "baz",
        OptionalArg::new(OptionalArgKind::RequiredValue, true),
        Some("B"),
    )?;
    parser.add_option(
        "qux",
        OptionalArg::new(OptionalArgKind::OptionalValue, true),
        Some("q"),
    )?;

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
        Err(DuplicateOption { name: "foo" }),
        parser.parse(&["--foo", "--foo"])
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
    assert_eq!(
        Err(InvalidOptionValue {
            name: "bar",
            value: "no".to_string()
        }),
        parser.parse(&["--bar=no"])
    );
    assert_eq!(
        Err(UnknownAlias {
            alias: "f".to_string()
        }),
        parser.parse(&["-f"])
    );
    assert_eq!(
        Ok(vec![
            Flag {
                name: "bar",
                value: true
            },
            Flag {
                name: "bar",
                value: false
            },
            Flag {
                name: "bar",
                value: true,
            },
            Positional {
                value: "false".to_string()
            }
        ]),
        parser.parse(&["--bar=true", "-b=false", "-b", "false"])
    );
    assert_eq!(
        Err(MissingOptionValue { name: "baz" }),
        parser.parse(&["--baz"]),
    );
    assert_eq!(
        Ok(vec![
            RequiredValue {
                name: "baz",
                value: "123".to_string()
            },
            RequiredValue {
                name: "baz",
                value: "-C".to_string()
            }
        ]),
        parser.parse(&["--baz=123", "-B", "-C"])
    );
    assert_eq!(
        Ok(vec![
            OptionalValue {
                name: "qux",
                value: None
            },
            Positional {
                value: "foo".to_string()
            },
            OptionalValue {
                name: "qux",
                value: Some("bar".to_string())
            }
        ]),
        parser.parse(&["--qux", "foo", "--qux=bar"])
    );
    assert_eq!(
        Err(UnknownAlias {
            alias: "t".to_string()
        }),
        parser.parse(&["-btrue"])
    );
    assert_eq!(
        Ok(vec![
            Flag {
                name: "bar",
                value: true
            },
            RequiredValue {
                name: "baz",
                value: "q=123".to_string()
            },
            Flag {
                name: "bar",
                value: true
            },
            OptionalValue {
                name: "qux",
                value: Some("123".to_string())
            }
        ]),
        parser.parse(&["-bBq=123", "-bq=123"])
    );

    Ok(())
}

#[test]
fn test_parse_options_first() -> Result<(), ArgParserError> {
    use ParsedArg::*;

    let mut parser = ArgParser::new(ArgParserMode::OptionsFirst);

    parser.add_option("foo", OptionalArg::new(OptionalArgKind::Flag, false), None)?;

    assert_eq!(
        Ok(vec![
            Flag {
                name: "foo",
                value: true
            },
            Positional {
                value: "foo".to_string()
            },
            Positional {
                value: "--foo".to_string()
            }
        ]),
        parser.parse(&["--foo", "foo", "--foo"])
    );

    Ok(())
}
