use std::fs::{read_dir, DirEntry, ReadDir};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DirIteratorError {
    pub path: String,
    pub err: io::Error,
}

impl DirIteratorError {
    pub fn new(path: &str, err: io::Error) -> Self {
        DirIteratorError {
            path: path.to_string(),
            err: err,
        }
    }
}

pub type Result<T> = std::result::Result<T, DirIteratorError>;

#[derive(Debug)]
pub struct DirIterator {
    rd: ReadDir,
    path: String,
    sub_iter: Option<Box<DirIterator>>,
}

impl DirIterator {
    fn pathbuf_to_string(pathbuf: PathBuf) -> String {
        pathbuf
            .to_str()
            .expect("What OS doesn't use unicode filenames")
            .to_string()
    }

    pub fn new(path: &str) -> Result<DirIterator> {
        match read_dir(path) {
            Ok(rd) => Ok(DirIterator {
                rd: rd,
                path: path.to_string(),
                sub_iter: None,
            }),
            Err(e) => Err(DirIteratorError::new(path, e)),
        }
    }

    fn next_from_sub(&mut self) -> Option<Result<String>> {
        match &mut self.sub_iter {
            None => None,
            Some(s) => s.next()
        }
    }

    fn replace_sub_from_direntry(&mut self, entry: DirEntry) -> Option<Result<()>> {
        let path = DirIterator::pathbuf_to_string(entry.path());
        match DirIterator::new(&path) {
            Ok(di) => {
                self.sub_iter = Some(Box::new(di));
                Some(Ok(()))
            }
            Err(e) => Some(Err(e)),
        }
    }

    fn direntry_is_directory(di: &DirEntry) -> Result<bool> {
        let path = DirIterator::pathbuf_to_string(di.path());
        match di.file_type().to_result(&path) {
            Ok(file_type) => Ok(file_type.is_dir()),
            Err(e) => Err(e),
        }
    }

    fn next_from_direntry(&mut self, di: DirEntry) -> Option<Result<String>> {
        match DirIterator::direntry_is_directory(&di) {
            Err(e) => return Some(Err(e)),
            Ok(false) => return None,
            Ok(true) => {}
        };

        match self.replace_sub_from_direntry(di) {
            None => return None,
            Some(Err(e)) => return Some(Err(e)),
            Some(Ok(_)) => {}
        };

        self.next_from_sub()
    }

    fn next_from_rd(&mut self, path: &str) -> Option<Result<String>> {
        loop {
            let value = self.rd.next()?;

            match value.to_result(path) {
                Ok(di) => match self.next_from_direntry(di) {
                    Some(Ok(r)) => return Some(Ok(r)),
                    Some(Err(e)) => return Some(Err(e)),
                    None => {}
                },
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl Iterator for DirIterator {
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_from_rd(&self.path.clone())
    }
}

trait ToResult<T> {
    fn to_result(self, path: &str) -> Result<T>;
}

impl<T> ToResult<T> for io::Result<T> {
    fn to_result(self, path: &str) -> Result<T> {
        self.or_else(|e| Err(DirIteratorError::new(path, e)))
    }
}
