use fancy_regex::Regex;
use crate::dir_iter::{DirIterator, DirIteratorError};
use crate::replace::{replace_lines_in_file, ReplaceError};

use rayon::prelude::*;

fn handle_diriteratorerror(die: DirIteratorError) {
    eprintln!("{}: {}", die.path, die.err)
}

fn handle_replaceerror(path: &str, re: ReplaceError) {
    eprintln!("{}: {}", path, re)
}

fn handle_result(result: Result<String, DirIteratorError>, pattern: &Regex, replacement: &str) {
    let path = match result {
        Ok(v) => v,
        Err(e) => return handle_diriteratorerror(e)
    };

    match replace_lines_in_file(&path, pattern, replacement) {
        Ok(_) => {},
        Err(e) => return handle_replaceerror(&path, e)
    }
}

pub fn find_and_replace<'a, I: Iterator<Item=&'a str>>(dirs: I, pattern: &Regex, replacement: &str) {
    let it = dirs.map(|a| DirIterator::new(a).unwrap()).reduce(|a, c| a.chain(c))

    DirIterator::new(dir).unwrap()
        .par_bridge()
        .for_each(|r| handle_result(r, pattern, replacement));
}