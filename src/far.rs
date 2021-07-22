use fancy_regex::Regex;
use crate::dir_iter::{DirIterator, DirIteratorError};
use crate::iter::Concat;
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

pub fn diriter_vec<'a, I: Iterator<Item=&'a str>>(dirs: I) -> Result<impl Iterator<Item=Result<String, DirIteratorError>>, DirIteratorError> {
    let mut vec = Vec::new();

    for dir in dirs {
        match DirIterator::new(dir) {
            Ok(di) => vec.push(di),
            Err(e) => return Err(e)
        }
    };

    Ok(Concat::new(vec))
}

pub fn find_and_replace<'a, I: Iterator<Item=&'a str>>(dirs: I, pattern: &Regex, replacement: &str) {
    let iter = match diriter_vec(dirs) {
        Ok(v) => v,
        Err(e) => return handle_diriteratorerror(e)
    };

    iter.par_bridge()
        .for_each(|r| handle_result(r, pattern, replacement));
}