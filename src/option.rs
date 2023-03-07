use super::{OptionalArg, OptionalArgKind};

impl OptionalArg {
    pub fn new(kind: OptionalArgKind, multiple: bool) -> Self {
        Self { kind, multiple }
    }
}

impl OptionalArg {
    pub(crate) fn is_valid(name: &str) -> bool {
        let mut allow_hyphen = false;

        for (idx, ch) in name.chars().enumerate() {
            match ch {
                '-' if allow_hyphen && idx + 1 < name.len() => allow_hyphen = false,
                _ if ch.is_ascii_alphanumeric() => allow_hyphen = true,
                _ => return false,
            };
        }

        !name.is_empty()
    }
}

#[test]
fn test_is_valid() {
    assert!(!OptionalArg::is_valid(""));
    assert!(!OptionalArg::is_valid("💩"));
    assert!(!OptionalArg::is_valid("-"));
    assert!(OptionalArg::is_valid("a"));
    assert!(!OptionalArg::is_valid("-a"));
    assert!(!OptionalArg::is_valid("a-"));
    assert!(!OptionalArg::is_valid("a--a"));
    assert!(OptionalArg::is_valid("a-A-0"));
}

impl OptionalArg {
    pub(crate) fn is_valid_alias(alias: &str) -> bool {
        Self::is_valid(alias) && alias.len() == 1
    }
}

#[test]
fn test_is_valid_alias() {
    assert!(OptionalArg::is_valid_alias("a"));
    assert!(!OptionalArg::is_valid_alias("-"));
    assert!(!OptionalArg::is_valid_alias("aA"));
}
