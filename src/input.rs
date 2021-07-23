use fancy_regex::{self, Regex};
use std::env;
use std::fmt::Display;

#[derive(Debug)]
pub enum ArgsError {
    InvalidRegex(fancy_regex::Error),
    NoArgsGiven,
    OnlyPatternGiven,
    UnrecognizedArgument(String)
}

impl Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                ArgsError::InvalidRegex(e) => e.to_string(),
                ArgsError::NoArgsGiven => "No arguments were given.".to_string(),
                ArgsError::OnlyPatternGiven =>
                    "A pattern was given but not a substitution.".to_string(),
                ArgsError::UnrecognizedArgument(s) => format!("The argument '{}' is unrecognized", s)
            }
        )
    }
}

#[derive(Debug)]
pub struct Args {
    pub pattern: Regex,
    pub replacement: String,
    pub paths: Vec<String>,
}

#[derive(Debug)]
struct IncompleteArgs {
    pattern: Option<Regex>,
    replacement: Option<String>,
    paths: Vec<String>,
    process_flags: bool
}

impl IncompleteArgs {
    fn new() -> Self {
        IncompleteArgs {
            pattern: None,
            replacement: None,
            paths: Vec::new(),
            process_flags: true
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
            _ => Err(ArgsError::UnrecognizedArgument(arg.to_string()))
        }
    }

    fn handle_argument(&mut self, arg: &str) -> Result<&mut IncompleteArgs, ArgsError> {
        if self.process_flags && arg.starts_with("-") {
            self.handle_flag(arg)
        }
        else {
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
            }),
        }
    }
}

pub fn parse_args() -> Result<Args, ArgsError> {
    let mut ia = IncompleteArgs::new();
    let _ = env::args().fold(Ok(&mut ia), |a, c| a.and_then(|ia| ia.handle_argument(&c)))?;
    Ok(ia.to_args()?)
}

fn prog_name() -> String {
    env::args().next().unwrap_or("far".to_string())
}

pub fn print_usage() {
    println!("usage: {} [flag...] pattern replacement [path...]", prog_name());
    println!();
    println!("flags:");
    println!("  -h, --help: display the help");
    println!();
}

pub fn print_help() {
    println!("far 1.0.0");
    println!("recursively finds and replaces a regex with a substitution in a directory");
    println!();
    print_usage()
}
