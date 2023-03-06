use std::{collections::HashMap, env};

pub enum OptionalArgKind {
    Flag,
    Value,
}

pub struct OptionalArg {
    kind: OptionalArgKind,
    multiple: bool,
}

pub struct ArgParser {
    pub(crate) aliases: HashMap<&'static str, &'static str>,
    pub(crate) options: HashMap<&'static str, OptionalArg>,
}

pub enum ParsedArg {
    Positional { value: String },
    OptionalFlag { name: String, value: bool },
    OptionalValue { name: String, value: Option<String> },
}

impl ArgParser {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            options: HashMap::new(),
        }
    }

    pub fn parse(args: &[&str]) -> Vec<ParsedArg> {
        vec![]
    }

    pub fn parse_args() -> Vec<ParsedArg> {
        let args = env::args().skip(1).collect::<Vec<_>>();

        Self::parse(&args.iter().map(|s| &s[..]).collect::<Vec<_>>())
    }
}
