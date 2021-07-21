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

    fn next_sub(&mut self) -> Option<Result<String>> {
        let si = &mut self.sub_iter;
        let si = match si {
            Some(s) => s,
            None => return None
        };

        match si.next() {
            Some(s) => Some(s),
            None => {
                self.sub_iter = None;
                None
            }
        }
    }

    fn next_dir(&mut self, entry: DirEntry) -> Option<Result<String>> {
        let path = DirIterator::pathbuf_to_string(entry.path());
        match DirIterator::new(&path) {
            Ok(di) => {
                self.sub_iter = Some(Box::new(di));
                self.next()
            },
            Err(e) => Some(Err(e))
        }
    }

    fn next_dir_entry(&mut self, di: DirEntry) -> Option<Result<String>> {
        let path = DirIterator::pathbuf_to_string(di.path());
        let file_type = match di.file_type().to_result(&path) {
            Ok(t) => t,
            Err(e) => return Some(Err(e))
        };
        if file_type.is_dir() {
            let din = match DirIterator::new(&path) {
                Ok(d) => d,
                Err(e) => return Some(Err(e))
            };
            self.sub_iter = Some(Box::new(din));
            self.next()
        }
        else if file_type.is_symlink() {
            self.next()
        }
        else {
            Some(Ok(path))
        }
    }

    fn next_rd(&mut self, path: &str) -> Option<Result<String>> {
        let value = self.rd.next()?;

        match value.to_result(path) {
            Ok(v) => self.next_dir_entry(v),
            Err(e) => Some(Err(e))
        }
    }
}

impl Iterator for DirIterator {
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_sub().or_else(|| self.next_rd(&self.path.clone()))
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
