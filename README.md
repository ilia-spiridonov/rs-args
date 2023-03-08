# rs-args
Reasonable process arguments parsing

## Usage
General example:
```rust
use rs_args::{ArgParser, ArgParserMode, OptionalArg, OptionalArgKind, ParsedArg};
use std::process;

fn main() {
    // Or ArgParser::new(ArgParserMode::OptionsFirst), see an explanation below
    let mut parser = ArgParser::default();

    {
        use OptionalArgKind::*;

        parser.add_option("user", OptionalArg::new(RequiredValue, false), None).unwrap();
        parser.add_option("interactive", OptionalArg::new(Flag, false), Some("i")).unwrap();
        parser.add_option("verbose", OptionalArg::new(Flag, true), Some("v")).unwrap();
    }

    // Or parser.parse_args() which will use std::env::args().skip(1)
    match parser.parse(&["--user", "foo", "-ivvv", "bar"]) {
        Err(e) => {
            // e is ArgParserError and can be serialized into a human-readable message
            eprintln!("parser error: {}", e);
            process::exit(1);
        }
        Ok(args) => {
            // args is Vec<ParsedArg>, see below
        }
    };
}
```

Extracting all positional arguments from parser output:
```rust
let pos_args = args
    .iter()
    .filter_map(|arg| if let ParsedArg::Positional { value } = arg { Some(value) } else { None })
    .collect::<Vec<_>>();

println!("positional arguments: {:?}", pos_args);
```

Extracting a flag:
```rust
let interactive = args
    .iter()
    .find_map(|arg| {
        if let ParsedArg::Flag { name: "interactive", value } = arg {
            Some(*value)
        } else {
            None
        }
    })
    .unwrap_or(false);

println!("interactive? {}", interactive);
```

## Notes
* `ArgParser` only handles `String` (and `&str`) and cannot accept `OsString` (and `&OsStr`); consequently, a subset of all valid process arguments lists now cannot be parsed by `rs-args`, although in practice that's not a significant limitation; the convenience of processing well-formed UTF-8 prevails.
* Parser output is a simple `Vec`, not a map of any kind; however, Rust's iterators are so powerful that turning it into anything that makes more sense for your use case shouldn't be an issue.
* Positional and optional arguments can come in any order, that's what's called the 'mixed' parsing mode and it's the `default()`; to enforce the 'options-first' mode (e.g. for parsing subcommands) pass `ArgParserMode::OptionsFirst` to `ArgParser::new`.
