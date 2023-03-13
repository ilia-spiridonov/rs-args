use super::{OptionalArg, OptionalArgKind};

impl OptionalArg {
    pub fn flag(name: &'static str) -> Self {
        Self::new(name, OptionalArgKind::Flag)
    }

    pub fn required_value(name: &'static str) -> Self {
        Self::new(name, OptionalArgKind::RequiredValue)
    }

    pub fn optional_value(name: &'static str) -> Self {
        Self::new(name, OptionalArgKind::OptionalValue)
    }

    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    pub fn alias(mut self, alias: &'static str) -> Self {
        self.alias = Some(alias);
        self
    }

    fn new(name: &'static str, kind: OptionalArgKind) -> Self {
        Self {
            name,
            alias: None,
            kind,
            multiple: false,
        }
    }
}

impl OptionalArg {
    pub(crate) fn is_valid(name: &str) -> bool {
        Self::is_valid_hyphen_seq(name) && name.len() > 1
    }

    pub(crate) fn is_valid_alias(alias: &str) -> bool {
        Self::is_valid_hyphen_seq(alias) && alias.len() == 1
    }

    fn is_valid_hyphen_seq(name: &str) -> bool {
        let mut allow_hyphen = false;

        for (idx, ch) in name.chars().enumerate() {
            match ch {
                '-' if allow_hyphen && idx + 1 < name.len() => allow_hyphen = false,
                _ if ch.is_ascii_alphanumeric() => allow_hyphen = true,
                _ => return false,
            };
        }

        true
    }
}

#[test]
fn test_is_valid() {
    assert!(!OptionalArg::is_valid(""));
    assert!(!OptionalArg::is_valid("💩"));
    assert!(!OptionalArg::is_valid("-"));
    assert!(!OptionalArg::is_valid("a"));
    assert!(OptionalArg::is_valid("aa"));
    assert!(!OptionalArg::is_valid("-a"));
    assert!(!OptionalArg::is_valid("a-"));
    assert!(!OptionalArg::is_valid("a--a"));
    assert!(OptionalArg::is_valid("a-A-0"));
}

#[test]
fn test_is_valid_alias() {
    assert!(OptionalArg::is_valid_alias("a"));
    assert!(!OptionalArg::is_valid_alias("-"));
    assert!(!OptionalArg::is_valid_alias("aA"));
}
