use fancy_regex::{self, Regex};
use std::env;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarMode {
    Lines,
    All,
}

#[derive(Debug)]
pub enum ArgsError {
    InvalidRegex(fancy_regex::Error),
    NoArgsGiven,
    OnlyPatternGiven,
    UnrecognizedArgument(String),
}

impl Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                ArgsError::InvalidRegex(e) => format!("Invalid regex: {}", e.to_string()),
                ArgsError::NoArgsGiven => "No arguments were given.".to_string(),
                ArgsError::OnlyPatternGiven =>
                    "A pattern was given but not a substitution.".to_string(),
                ArgsError::UnrecognizedArgument(s) =>
                    format!("The argument '{}' is unrecognized", s),
            }
        )
    }
}

#[derive(Debug)]
pub struct Args {
    pub pattern: Regex,
    pub replacement: String,
    pub paths: Vec<String>,
    pub mode: FarMode,
}

#[derive(Debug)]
struct IncompleteArgs {
    pattern: Option<Regex>,
    replacement: Option<String>,
    paths: Vec<String>,
    mode: FarMode,
    process_flags: bool,
}

impl IncompleteArgs {
    fn new() -> Self {
        IncompleteArgs {
            pattern: None,
            replacement: None,
            paths: Vec::new(),
            mode: FarMode::Lines,
            process_flags: true,
        }
    }

    fn handle_positional(&mut self, arg: &str) -> Result<&mut IncompleteArgs, ArgsError> {
        let arg = arg.to_string();

        match (&self.pattern, &self.replacement) {
            (None, _) => {
                self.pattern = Some(Regex::new(&arg).map_err(|e| ArgsError::InvalidRegex(e))?)
            }
            (_, None) => self.replacement = Some(arg),
            (_, _) => self.paths.push(arg),
        };

        Ok(self)
    }

    fn handle_flag(&mut self, arg: &str) -> Result<&mut IncompleteArgs, ArgsError> {
        match arg {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0)
            }
            "--multiline" | "-m" => {
                self.mode = FarMode::All;
                Ok(self)
            }
            "--singleline" | "-s" => {
                self.mode = FarMode::Lines;
                Ok(self)
            }
            "--" => {
                self.process_flags = false;
                Ok(self)
            }
            _ => Err(ArgsError::UnrecognizedArgument(arg.to_string())),
        }
    }

    fn handle_argument(&mut self, arg: &str) -> Result<&mut IncompleteArgs, ArgsError> {
        if self.process_flags && arg.starts_with("-") {
            self.handle_flag(arg)
        } else {
            self.handle_positional(arg)
        }
    }

    fn to_args(mut self) -> Result<Args, ArgsError> {
        if self.paths.is_empty() {
            self.paths = vec![".".to_string()]
        }

        match (self.pattern, self.replacement, &self.paths[..]) {
            (None, _, _) => Err(ArgsError::NoArgsGiven),
            (_, None, _) => Err(ArgsError::OnlyPatternGiven),
            (Some(pat), Some(repl), _) => Ok(Args {
                pattern: pat,
                replacement: repl,
                paths: self.paths,
                mode: self.mode,
            }),
        }
    }
}

pub fn parse_cmdline() -> Result<Args, ArgsError> {
    parse_args(env::args().skip(1))
}

pub fn parse_args<S, I>(args: I) -> Result<Args, ArgsError>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
{
    let mut ia = IncompleteArgs::new();
    let _ = args
        .into_iter()
        .fold(Ok(&mut ia), |a, c| a.and_then(|ia| ia.handle_argument(c.as_ref())))?;

    Ok(ia.to_args()?)
}

fn prog_name() -> String {
    env::args().next().unwrap_or("far".to_string())
}

pub fn print_usage() {
    println!(
        "usage: {} [flag...] pattern replacement [path...]",
        prog_name()
    );
    println!();
    println!("flags:");
    println!("  -h, --help:       display the help");
    println!("  -m, --multiline:  match the whole file instead of line-by-line");
    println!("  -s, --singleline: match line-by-line. this is the default");
    println!();
}

pub fn print_help() {
    println!("far 1.0.0");
    println!("recursively finds and replaces a regex with a substitution in a directory");
    println!();
    print_usage()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_with_single_argument() -> Result<(), ArgsError> {
        let cmdline = "abc def /tmp".split(char::is_whitespace);
        let args = parse_args(cmdline)?;

        assert_eq!(args.paths, vec!["/tmp"]);
        debug_assert!(args.pattern.is_match("abc").unwrap());
        debug_assert!(!args.pattern.is_match("ab").unwrap());
        debug_assert!(args.pattern.is_match("abcd").unwrap());
        assert_eq!(args.replacement, "def");
        assert_eq!(args.mode, FarMode::Lines);

        Ok(())
    }

    #[test]
    fn test_args_gives_cd_if_no_dirs_are_given() -> Result<(), ArgsError> {
        let cmdline = "abc def".split(char::is_whitespace);
        let args = parse_args(cmdline)?;

        assert_eq!(args.paths, vec!["."]);
        debug_assert!(args.pattern.is_match("abc").unwrap());
        debug_assert!(!args.pattern.is_match("ab").unwrap());
        debug_assert!(args.pattern.is_match("abcd").unwrap());
        assert_eq!(args.replacement, "def");
        assert_eq!(args.mode, FarMode::Lines);

        Ok(())
    }

    #[test]
    fn test_args_with_multiple_directories() -> Result<(), ArgsError> {
        let cmdline = "abc def /tmp /var/tmp".split(char::is_whitespace);
        let args = parse_args(cmdline)?;

        assert_eq!(args.paths, vec!["/tmp", "/var/tmp"]);
        debug_assert!(args.pattern.is_match("abc").unwrap());
        debug_assert!(!args.pattern.is_match("ab").unwrap());
        debug_assert!(args.pattern.is_match("abcd").unwrap());
        assert_eq!(args.replacement, "def");
        assert_eq!(args.mode, FarMode::Lines);

        Ok(())
    }

    #[test]
    fn test_args_rejects_invalid_regex() {
        let cmdline = vec!["(a", "def"];
        let args_err = parse_args(cmdline).unwrap_err();

        match args_err {
            ArgsError::InvalidRegex(fancy_regex::Error::UnclosedOpenParen) => {},
            ArgsError::InvalidRegex(_) => panic!("The error should be for an unclosed opening paren"),
            _ => panic!("The error should be for an invalid regex.")
        }
    }

    #[test]
    fn test_args_rejects_no_args() {
        let cmdline: Vec<&str> = Vec::new();
        let args_err = parse_args(cmdline).unwrap_err();

        match args_err {
            ArgsError::NoArgsGiven => {},
            _ => panic!("The error should be for no args given.")
        }
    }

    #[test]
    fn test_args_rejects_single_argument() {
        let cmdline = vec!["abc"];
        let args_err = parse_args(cmdline).unwrap_err();

        match args_err {
            ArgsError::OnlyPatternGiven => {},
            _ => panic!("The error should be for only a single arg.")
        }
    }

    #[test]
    fn test_args_multiline() -> Result<(), ArgsError> {
        let cmdline = "abc def --multiline".split(char::is_whitespace);
        let args = parse_args(cmdline)?;

        assert_eq!(args.paths, vec!["."]);
        debug_assert!(args.pattern.is_match("abc").unwrap());
        debug_assert!(!args.pattern.is_match("ab").unwrap());
        debug_assert!(args.pattern.is_match("abcd").unwrap());
        assert_eq!(args.replacement, "def");
        assert_eq!(args.mode, FarMode::All);

        Ok(())
    }

    #[test]
    fn test_args_dash_dash() -> Result<(), ArgsError> {
        let cmdline = "abc def -- --multiline".split(char::is_whitespace);
        let args = parse_args(cmdline)?;

        assert_eq!(args.paths, vec!["--multiline"]);
        debug_assert!(args.pattern.is_match("abc").unwrap());
        debug_assert!(!args.pattern.is_match("ab").unwrap());
        debug_assert!(args.pattern.is_match("abcd").unwrap());
        assert_eq!(args.replacement, "def");
        assert_eq!(args.mode, FarMode::Lines);

        Ok(())
    }
}
