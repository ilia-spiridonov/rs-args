use super::ParsedArg;

pub struct ArgSelector<'a> {
    pub(crate) args: &'a Vec<ParsedArg>,
}

impl<'a> ArgSelector<'a> {
    pub fn new(args: &'a Vec<ParsedArg>) -> Self {
        Self { args }
    }

    pub fn get_positional(&self) -> Vec<&'a String> {
        self.args
            .iter()
            .filter_map(|arg| match arg {
                ParsedArg::Positional { value } => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn get_flag(&self, name: &str, default: bool) -> bool {
        self.args
            .iter()
            .find_map(|arg| match arg {
                ParsedArg::Flag { name: _name, value } if name == *_name => Some(*value),
                _ => None,
            })
            .unwrap_or(default)
    }

    pub fn get_value(&self, name: &str) -> Option<&'a String> {
        self.args.iter().find_map(|arg| match arg {
            ParsedArg::RequiredValue { name: _name, value } if name == *_name => Some(value),
            _ => None,
        })
    }

    pub fn get_values(&self, name: &str) -> Vec<&'a String> {
        self.args
            .iter()
            .filter_map(|arg| match arg {
                ParsedArg::RequiredValue { name: _name, value } if name == *_name => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn get_optional_value(&self, name: &str, default: &'a String) -> &'a String {
        self.args
            .iter()
            .find_map(|arg| match arg {
                ParsedArg::OptionalValue { name: _name, value } if name == *_name => value.as_ref(),
                _ => None,
            })
            .unwrap_or(default)
    }
}

#[test]
fn test_arg_selector() {
    use ParsedArg::*;

    let args = vec![
        Positional {
            value: "123".to_string(),
        },
        Flag {
            name: "foo",
            value: true,
        },
        RequiredValue {
            name: "bar",
            value: "456".to_string(),
        },
        OptionalValue {
            name: "baz",
            value: Some("789".to_string()),
        },
    ];

    let s = ArgSelector::new(&args);

    assert_eq!(vec!["123"], s.get_positional());

    assert!(s.get_flag("foo", false));
    assert!(!s.get_flag("bar", false));
    assert!(s.get_flag("bar", true));

    assert_eq!(None, s.get_value("foo"));
    assert_eq!(Some(&"456".to_string()), s.get_value("bar"));

    assert_eq!(vec!["456"], s.get_values("bar"));
    assert!(s.get_values("baz").is_empty());

    assert_eq!("abc", s.get_optional_value("bar", &"abc".to_string()));
    assert_eq!("789", s.get_optional_value("baz", &"abc".to_string()));
}
