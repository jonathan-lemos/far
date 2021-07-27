use crate::FarMode;
use fancy_regex::Regex;
use crate::dir_iter::{DirIterator, DirIteratorError};
use crate::iter::Concat;
use crate::replace::{replace_all_in_file, replace_lines_in_file, ReplaceError};

use rayon::prelude::*;

fn handle_diriteratorerror(die: DirIteratorError) {
    eprintln!("{}: {}", die.path, die.err)
}

fn handle_replaceerror(path: &str, re: ReplaceError) {
    eprintln!("{}: {}", path, re)
}

fn handle_result(result: Result<String, DirIteratorError>, pattern: &Regex, replacement: &str, mode: FarMode) {
    let path = match result {
        Ok(v) => v,
        Err(e) => return handle_diriteratorerror(e)
    };

    let f = match mode {
        FarMode::Lines => replace_lines_in_file,
        FarMode::All => replace_all_in_file
    };

    match f(&path, pattern, replacement) {
        Ok(_) => {},
        Err(e) => return handle_replaceerror(&path, e)
    }
}

pub fn diriter_vec<S: AsRef<str>, I: Iterator<Item=S>>(dirs: I) -> Result<impl Iterator<Item=Result<String, DirIteratorError>>, DirIteratorError> {
    let mut vec = Vec::new();

    for dir in dirs {
        match DirIterator::new(dir.as_ref()) {
            Ok(di) => vec.push(di),
            Err(e) => return Err(e)
        }
    };

    Ok(Concat::new(vec))
}

pub fn find_and_replace<S: AsRef<str>, I: IntoIterator<Item=S>>(dirs: I, pattern: &Regex, replacement: &str, mode: FarMode) {
    let iter = match diriter_vec(dirs.into_iter()) {
        Ok(v) => v,
        Err(e) => return handle_diriteratorerror(e)
    };

    iter.par_bridge()
        .for_each(|r| handle_result(r, pattern, replacement, mode));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdir::testdir::{TestFile};
    use std::fs::read_to_string;

    #[test]
    pub fn test_handle_result_basic() {
        let file = TestFile::new("abc def abc");
        let re = fancy_regex::Regex::new("abc").unwrap();

        handle_result(Ok(file.path_str()), &re, "def", FarMode::All);

        let new_contents = read_to_string(&file.path_str()).unwrap();

        debug_assert_eq!(new_contents, "def def def");
    }
}