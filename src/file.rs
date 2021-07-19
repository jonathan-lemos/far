use std::fs;
use std::io::{ErrorKind, Result};

pub struct TempFile {
    pub file: fs::File,
    pub filename: String,
}

impl TempFile {
    pub fn new(filename: &str, suffix: &str) -> Result<Self> {
        let mut index = 0;

        let mut next_fname = || {
            index += 1;
            gen_temp_filename(filename, suffix, index)
        };

        let mut fname = next_fname();

        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&fname)
            {
                Ok(f) => return Ok(TempFile {file: f, filename: fname}),
                Err(e) => match e.kind() {
                    ErrorKind::AlreadyExists => {
                        fname = next_fname();
                    }
                    _ => return Err(e),
                },
            }
        }
    }

    pub fn filename(filename: &str, suffix: &str) -> Result<String> {
        let tmp = Self::new(filename, suffix)?;
        Ok(tmp.filename)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove_file(self.filename);
    }
}

fn gen_temp_filename(filename: &str, suffix: &str, index: u32) -> String {
    let mut ret = filename.to_string();
    ret += suffix;
    ret += &index.to_string();
    ret
}

fn copy_file_and_delete_original(from: &str, to: &str) -> Result<()> {
    fs::copy(from, to)?;
    fs::remove_file(from)?;
    Ok(())
}

pub fn move_file(from: &str, to: &str) -> Result<()> {
    fs::rename(from, to).or_else(|_| copy_file_and_delete_original(from, to))?;
    Ok(())
}

pub fn replace_file(from: &str, to: &str) -> Result<()> {
    let fname = TempFile::filename(to, ".old")?;
    move_file(to, &fname)?;
    move_file(from, to)?;
    fs::remove_file(&fname)?;
    Ok(())
}
