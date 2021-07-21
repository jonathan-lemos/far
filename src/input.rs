use std::fmt::Display;
use fancy_regex::{self, Regex};
use std::env;

#[derive(Debug)]
pub enum ArgsError {
    InvalidRegex(fancy_regex::Error),
    NoArgsGiven,
    OnlyPatternGiven,
    NoPathsGiven
}

impl Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", match self {
            ArgsError::InvalidRegex(e) => e.to_string()
        })
    }
}

#[derive(Debug)]
pub struct Args {
    pattern: Regex,
    replacement: String,
    paths: Vec<String>
}

#[derive(Debug)]
struct IncompleteArgs {
    pattern: Option<Regex>,
    replacement: Option<String>,
    paths: Vec<String>
}

impl IncompleteArgs {
    fn new() -> Self {
        IncompleteArgs {
            pattern: None,
            replacement: None,
            paths: Vec::new()
        }
    }

    fn set_positional(&mut self, arg: &str) -> Result<&mut IncompleteArgs, ArgsError> {
        let arg = arg.to_string();

        match (&self.pattern, &self.replacement) {
            (None, _) => self.pattern = Some(Regex::new(&arg).map_err(|e| ArgsError::InvalidRegex(e))?),
            (_, None) => self.replacement = Some(arg),
            (_, _) => self.paths.push(arg)
        };

        Ok(self)
    }

    fn to_args(self) -> Result<Args, ArgsError> {
       match (self.pattern, self.replacement, &self.paths[..]) {
            (None, _, _) => Err(ArgsError::NoArgsGiven),
            (_, None, _) => Err(ArgsError::OnlyPatternGiven),
            (_, _, []) => Err(ArgsError::NoPathsGiven),
            (Some(pat), Some(repl), _) => Ok(Args{
                pattern: pat,
                replacement: repl,
                paths: self.paths
            })
        }
    }
}

pub fn parse_args() -> Result<Args, ArgsError> {
    let mut ia = IncompleteArgs::new();
    let _ = env::args().fold(Ok(&mut ia), |a, c| a.and_then(|ia| ia.set_positional(&c)))?;
    Ok(ia.to_args()?)
}