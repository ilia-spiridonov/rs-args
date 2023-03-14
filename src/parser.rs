use super::{OptionalArg, OptionalArgKind, PositionalArg, PositionalArgKind};
use std::{
    collections::{HashMap, VecDeque},
    env, error, fmt,
};

#[derive(Debug, PartialEq)]
pub enum ArgParserMode {
    Mixed,
    OptionsFirst,
}

#[derive(Debug, PartialEq)]
pub struct ArgParser {
    pub(crate) mode: ArgParserMode,
    pub(crate) aliases: HashMap<&'static str, &'static str>,
    pub(crate) options: HashMap<&'static str, OptionalArg>,
    pub(crate) positional: Vec<PositionalArg>,
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
    InvalidAliasValue { alias: &'static str, value: String },
    MissingOptionValue { name: &'static str },
    MissingAliasValue { alias: &'static str },
    InvalidRestArg,
    MissingArgs { actual: usize, expected: usize },
}

impl fmt::Display for ArgParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ArgParserError::*;

        match self {
            InvalidOption { name } => write!(f, "--{} is invalid", name),
            InvalidAlias { alias } => write!(f, "-{} is invalid", alias),
            DuplicateOption { name } => write!(f, "cannot provide --{} again", name),
            DuplicateAlias { alias } => write!(f, "cannot provide -{} again", alias),
            UnknownOption { name } => write!(f, "--{} is undefined", name),
            UnknownAlias { alias } => write!(f, "-{} is undefined", alias),
            InvalidOptionValue { name, value } => {
                write!(f, "--{} cannot accept '{}' as a value", name, value)
            }
            InvalidAliasValue { alias, value } => {
                write!(f, "-{} cannot accept '{}' as a value", alias, value)
            }
            MissingOptionValue { name } => write!(f, "--{} is missing a value", name),
            MissingAliasValue { alias } => write!(f, "-{} is missing a value", alias),
            InvalidRestArg => write!(f, "'rest' positional arg must be placed last"),
            MissingArgs { actual, expected } => {
                write!(f, "{} arg(s) required, but got {}", expected, actual)
            }
        }
    }
}

impl error::Error for ArgParserError {}

impl ArgParser {
    pub fn new(mode: ArgParserMode) -> Self {
        Self {
            mode,
            aliases: HashMap::new(),
            options: HashMap::new(),
            positional: Vec::new(),
        }
    }
}

impl Default for ArgParser {
    fn default() -> Self {
        Self::new(ArgParserMode::Mixed)
    }
}

impl ArgParser {
    pub fn add_option(&mut self, option: OptionalArg) -> Result<&mut Self, ArgParserError> {
        use ArgParserError::*;

        let OptionalArg { name, alias, .. } = option;

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

        Ok(self)
    }
}

#[test]
fn test_add_option() {
    use ArgParserError::*;

    let mut parser = ArgParser::default();

    assert_eq!(
        Err(InvalidOption {
            name: "--foo".to_string()
        }),
        parser.add_option(OptionalArg::flag("--foo"))
    );
    assert_eq!(
        Err(InvalidAlias {
            alias: "?".to_string()
        }),
        parser.add_option(OptionalArg::flag("foo").alias("?"))
    );
    assert!(parser
        .add_option(OptionalArg::flag("foo").alias("f"))
        .is_ok());
    assert_eq!(
        Err(DuplicateOption { name: "foo" }),
        parser.add_option(OptionalArg::flag("foo"))
    );
    assert_eq!(
        Err(DuplicateAlias { alias: "f" }),
        parser.add_option(OptionalArg::flag("bar").alias("f"))
    );
}

impl ArgParser {
    pub fn add_positional(&mut self, arg: PositionalArg) -> Result<&mut Self, ArgParserError> {
        if self.positional.last()
            == Some(&PositionalArg {
                kind: PositionalArgKind::Rest,
            })
        {
            return Err(ArgParserError::InvalidRestArg);
        }

        self.positional.push(arg);

        Ok(self)
    }
}

#[test]
fn test_add_positional() {
    let mut parser = ArgParser::default();

    assert!(parser.add_positional(PositionalArg::named()).is_ok());
    assert!(parser.add_positional(PositionalArg::rest()).is_ok());
    assert_eq!(
        Err(ArgParserError::InvalidRestArg),
        parser.add_positional(PositionalArg::named())
    );
    assert_eq!(
        Err(ArgParserError::InvalidRestArg),
        parser.add_positional(PositionalArg::rest())
    );
}

impl ArgParser {
    pub fn parse_args(&self) -> Result<Vec<ParsedArg>, ArgParserError> {
        let args = env::args().skip(1).collect::<Vec<_>>();
        let str_args = args.iter().map(|s| &s[..]).collect::<Vec<_>>();

        self.parse(&str_args)
    }

    pub fn parse(&self, args: &[&str]) -> Result<Vec<ParsedArg>, ArgParserError> {
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
                    let (name, option, alias) = self.resolve(name_or_alias)?;

                    let value = if alias.is_some() {
                        if let Some(value) = value.strip_prefix('=') {
                            value
                        } else if matches!(option.kind, OptionalArgKind::Flag)
                            && !value.is_empty()
                            && !value.starts_with('-')
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
                                return Err(if let Some(alias) = alias {
                                    InvalidAliasValue {
                                        alias,
                                        value: value.to_string(),
                                    }
                                } else {
                                    InvalidOptionValue {
                                        name,
                                        value: value.to_string(),
                                    }
                                });
                            }

                            parsed_args.push(Flag {
                                name,
                                value: matches!(value, "" | "true"),
                            });
                        }
                        OptionalArgKind::RequiredValue => {
                            let value = if value.is_empty() {
                                args.pop_front()
                                    .and_then(|s| {
                                        if let Ok(Some(_)) = self.parse_option(&s) {
                                            None
                                        } else {
                                            Some(s)
                                        }
                                    })
                                    .ok_or(if let Some(alias) = alias {
                                        MissingAliasValue { alias }
                                    } else {
                                        MissingOptionValue { name }
                                    })?
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
                            return Err(if let Some(alias) = alias {
                                DuplicateAlias { alias }
                            } else {
                                DuplicateOption { name }
                            });
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

        let parsed_positional = parsed_args
            .iter()
            .filter(|arg| matches!(arg, ParsedArg::Positional { value: _ }))
            .count();

        let min_expected_positional = self
            .positional
            .iter()
            .filter(|arg| arg.kind == PositionalArgKind::Named)
            .count();

        if parsed_positional < min_expected_positional {
            return Err(MissingArgs {
                actual: parsed_positional,
                expected: min_expected_positional,
            });
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
    ) -> Result<(&'static str, &OptionalArg, Option<&'static str>), ArgParserError> {
        use ArgParserError::*;

        let (name, alias) = if OptionalArg::is_valid_alias(name_or_alias) {
            let (&alias, &name) =
                self.aliases
                    .get_key_value(name_or_alias)
                    .ok_or(UnknownAlias {
                        alias: name_or_alias.to_string(),
                    })?;

            (name, Some(alias))
        } else {
            (name_or_alias, None)
        };

        let (name, option) = self.options.get_key_value(name).ok_or(UnknownOption {
            name: name.to_string(),
        })?;

        Ok((name, option, alias))
    }
}

#[test]
fn test_parse() -> Result<(), ArgParserError> {
    use ArgParserError::*;
    use ParsedArg::*;

    let mut parser = ArgParser::default();

    parser
        .add_option(OptionalArg::flag("foo").alias("f"))?
        .add_option(OptionalArg::flag("bar").multiple().alias("b"))?
        .add_option(OptionalArg::required_value("baz").multiple().alias("B"))?
        .add_option(OptionalArg::optional_value("qux").multiple().alias("q"))?;

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
        Err(InvalidAliasValue {
            alias: "b",
            value: "no".to_string()
        }),
        parser.parse(&["-b=no"])
    );
    assert_eq!(
        Err(UnknownAlias {
            alias: "a".to_string()
        }),
        parser.parse(&["-a"])
    );
    assert_eq!(
        Err(DuplicateAlias { alias: "f" }),
        parser.parse(&["-f", "-f"])
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
        Err(MissingAliasValue { alias: "B" }),
        parser.parse(&["-B", "--foo"])
    );
    assert_eq!(
        Ok(vec![
            RequiredValue {
                name: "baz",
                value: "123".to_string()
            },
            RequiredValue {
                name: "baz",
                value: "456".to_string()
            }
        ]),
        parser.parse(&["--baz=123", "-B", "456"])
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
        Err(InvalidAliasValue {
            alias: "b",
            value: "-foo".to_string()
        }),
        parser.parse(&["-b-foo"])
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
    use ArgParserError::*;
    use ParsedArg::*;

    let mut parser = ArgParser::new(ArgParserMode::OptionsFirst);

    parser
        .add_positional(PositionalArg::named())?
        .add_positional(PositionalArg::named())?
        .add_positional(PositionalArg::rest())?
        .add_option(OptionalArg::flag("foo"))?;

    assert_eq!(
        Err(MissingArgs {
            actual: 1,
            expected: 2
        }),
        parser.parse(&["--foo", "foo"])
    );
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
