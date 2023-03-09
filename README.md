# rs-args
Reasonable process arguments parsing

## Usage
General example:
```rust
use rs_args::{ArgParser, ArgParserMode, ArgSelector, OptionalArg, OptionalArgKind};
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

Since `args` is just a vector of `ParsedArg`, you may want to use `ArgSelector` to easily query its contents:
```rust
let sel = ArgSelector::new(&args);
let pos_args = sel.get_positional();

println!("positional arguments: {:?}", pos_args);
```

Another example, extracting a flag:
```rust
let interactive = sel.get_flag("interactive", false);

println!("interactive? {}", interactive);
```

## Notes
* `ArgParser` only handles `String` (and `&str`) and cannot accept `OsString` (and `&OsStr`); consequently, a subset of all valid process arguments lists now cannot be parsed by `rs-args`, although in practice that's not a significant limitation; the convenience of processing well-formed UTF-8 prevails.
* Parser output is a simple `Vec`, not a map of any kind; however, Rust's iterators are so powerful that turning it into anything that makes more sense for your use case shouldn't be an issue (assuming `ArgSelector` doesn't work for you).
* Positional and optional arguments can come in any order, that's what's called the 'mixed' parsing mode and it's the `default()`; to enforce the 'options-first' mode (e.g. for parsing subcommands) pass `ArgParserMode::OptionsFirst` to `ArgParser::new`.

## Features
* An option's name must be a properly hyphenated ASCII alphanumeric string of length 2+. Same for aliases, but length is strictly 1. This isn't too restrictive and permits curious things like camelCased options and numeric aliases.
* Pass `--` to treat everything that follows literally: `foo -- --foo` yields `--foo` as a positional argument, and `-- --` yields a single positional argument `--` (which makes sense, right?).
* Flags can be reset by passing `false` as an explicit value, e.g. `--foo=false`. For symmetry, `true` can also be passed the same way but that's completely redundant. No other values are permitted in the interest of trying to avoid things like the infamous YAML's Norway problem.
* There's a safety check that'll prevent a value from being implicitly consumed if it looks like an option, e.g. `--foo --bar` will fail if `foo` actually requires a value, but the explicit notation `--foo=--bar` will work.
* It's possible to use `=` with aliases too, e.g. `-f=bar`, but unnecessary unless `f` is a flag, as `-fbar` works. Things like `-fbar=baz` are also permitted, but in this case `=` won't be stripped since it's not a prefix.
