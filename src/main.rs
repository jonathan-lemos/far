mod dir_iter;
mod file;
mod replace;

use fancy_regex::Regex;
use dir_iter::{DirIterator, DirIteratorError};
use replace::{replace_lines_in_file, ReplaceError};

use rayon::prelude::*;

fn handle_diriteratorerror(die: DirIteratorError) {
    eprintln!("{}: {}", die.path, die.err)
}

fn handle_replaceerror(path: &str, re: ReplaceError) {
    eprintln!("{}: {}", path, re)
}

fn handle_result(result: Result<String, DirIteratorError>, pattern: &Regex, replacement: &str) {
    let r = result.and_then(|s| {
        replace_lines_in_file(&s, pattern, replacement)
    };
}

fn replace_in_directory(dir: &str) {
    DirIterator::new(dir).unwrap()
        .par_bridge()
        .for_each();
}

fn main() {
    println!("Hello, world!");
}
