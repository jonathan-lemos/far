use crate::file::*;
use ascii_utils::Check;
use fancy_regex::Regex;
use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::str;

fn replace_string(input: &str, pattern: &Regex, replacement: &str) -> String {
    pattern.replace_all(input, replacement).to_string()
}

fn write_to_file<I: Iterator<Item = io::Result<String>>>(
    file: &mut fs::File,
    strings: I,
) -> io::Result<()> {
    for string in strings {
        file.write_all(string?.as_bytes())?;
    }
    Ok(())
}

fn string_is_printable(s: &str) -> bool {
    s.chars().all(|c| c.is_printable())
}

fn file_is_printable(path: &str) -> io::Result<bool> {
    let cap = 256 * 1024;
    let file = fs::File::open(path)?;
    let mut br = BufReader::with_capacity(cap, file);

    loop {
        let length = {
            let buf = br.fill_buf()?;

            if let Ok(s) = str::from_utf8(buf) {
                if !string_is_printable(s) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
            buf.len()
        };

        if length == 0 {
            return Ok(true);
        }

        br.consume(length);
    }
}

#[derive(Debug)]
pub enum ReplaceError {
    FileTooBig,
    FileNotPrintable,
    IOError(io::Error),
}

impl From<io::Error> for ReplaceError {
    fn from(e: io::Error) -> Self {
        ReplaceError::IOError(e)
    }
}

impl Display for ReplaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                ReplaceError::FileTooBig => "The file is too big.".to_string(),
                ReplaceError::FileNotPrintable => "The file is not printable.".to_string(),
                ReplaceError::IOError(e) => e.to_string(),
            }
        )
    }
}

fn conv_result<T>(res: io::Result<T>) -> Result<T, ReplaceError> {
    res.or_else(|e| Err(ReplaceError::from(e)))
}

fn get_contents_of_file(filename: &str) -> Result<String, ReplaceError> {
    if conv_result(fs::metadata(filename))?.len() > 4 * 1024 * 1024 {
        return Err(ReplaceError::FileTooBig);
    }

    let contents = conv_result(fs::read_to_string(filename))?;
    if !string_is_printable(&contents) {
        return Err(ReplaceError::FileNotPrintable);
    }

    Ok(contents)
}

fn get_lines_of_file(
    filename: &str,
) -> Result<impl Iterator<Item = io::Result<String>>, ReplaceError> {
    if !file_is_printable(filename)? {
        return Err(ReplaceError::FileNotPrintable);
    }

    let file = conv_result(fs::File::open(filename))?;
    return Ok(BufReader::with_capacity(16 * 1024, file).lines());
}

pub fn replace_all_in_file(
    filename: &str,
    pattern: &Regex,
    replacement: &str,
) -> Result<(), ReplaceError> {
    let contents = get_contents_of_file(filename)?;

    let mut tmp = conv_result(TempFile::new(filename, ".new"))?;
    let new_contents = replace_string(&contents, pattern, replacement);
    conv_result(write_to_file(
        &mut tmp.file,
        std::iter::once(Ok(new_contents)),
    ))?;

    conv_result(replace_file(&tmp.filename, filename))?;
    Ok(())
}

pub fn replace_lines_in_file(
    filename: &str,
    pattern: &Regex,
    replacement: &str,
) -> Result<(), ReplaceError> {
    let lines = get_lines_of_file(filename)?;

    let mut tmp = conv_result(TempFile::new(filename, ".new"))?;

    let new_contents = lines.map(|r| r.map(|l| replace_string(&l, pattern, replacement)));
    conv_result(write_to_file(&mut tmp.file, new_contents))?;

    conv_result(replace_file(&tmp.filename, filename))?;
    Ok(())
}
