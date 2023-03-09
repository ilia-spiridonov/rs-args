use super::ParsedArg;

pub struct ArgSelector<'a> {
    pub(crate) args: &'a Vec<ParsedArg>,
}

impl<'a> ArgSelector<'a> {
    pub fn new(args: &'a Vec<ParsedArg>) -> Self {
        Self { args }
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
}

#[test]
fn test_arg_selector() {
    use ParsedArg::*;

    let args = vec![
        Flag {
            name: "foo",
            value: true,
        },
        Flag {
            name: "foo",
            value: false,
        },
        Flag {
            name: "foo",
            value: true,
        },
        RequiredValue {
            name: "bar",
            value: "123".to_string(),
        },
    ];

    let s = ArgSelector::new(&args);

    assert!(s.get_flag("foo", false));
    assert!(!s.get_flag("bar", false));
    assert!(s.get_flag("bar", true));
}
