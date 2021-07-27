#[cfg(test)]
pub mod testdir {
    use rand::{distributions::Alphanumeric, Rng};
    use std::env;
    use std::fs;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};

    fn random_name() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect()
    }

    fn pathbuf_to_str(pb: &PathBuf) -> String {
        pb.to_str().expect("The OS should be using UTF-8 strings").to_string()
    }

    fn pathbuf_concat(pb: &PathBuf, s: &str) -> PathBuf {
        let mut ret = pb.clone();
        ret.push(s);
        ret
    }

    fn make_until_valid_path<T, F: FnMut(PathBuf) -> io::Result<T>>(dir: &PathBuf, mut func: F) -> (T, PathBuf) {
        loop {
            let try_pb = pathbuf_concat(dir, &random_name());
            let try_str = pathbuf_to_str(&try_pb);

            match func(try_pb.clone()) {
                Ok(t) => return (t, try_pb),
                Err(e) => match e.kind() {
                    io::ErrorKind::AlreadyExists => {},
                    _ => panic!("Could not create {}: {}", try_str, e)
                }
            };
        }
    }

    pub struct TestDir {
        path: PathBuf,
        subdirs: Vec<TestDir>
    }

    impl TestDir {
        pub fn new() -> TestDir {
            let (_, path) = make_until_valid_path(&env::temp_dir(), fs::create_dir);
            TestDir {
                path: path,
                subdirs: Vec::new()
            }
        }

        pub fn file(&mut self, name: &str, contents: &str) -> &mut TestDir {
            let mut fname = self.path.clone();
            fname.push(name);

            let path_str = fname.to_str().expect("Imagine using an OS without UTF-8 filenames").to_string();

            let _ = fs::File::create(&fname)
                .expect(&format!(
                    "Could not create file {}",
                    path_str
                ))
                .write(contents.as_bytes())
                .expect(&format!(
                    "Couldn't write to file {}",
                    path_str
                ));

            self
        }

        pub fn subdir<F: FnOnce(&mut TestDir) -> ()>(&mut self, name: &str, func: F) -> &mut TestDir {
            let path = pathbuf_concat(&self.path, name);
            let _ = fs::create_dir(&path).expect(&format!(
                "Could not create subdir {}. Does it already exist?",
                name
            ));

            let mut dir = TestDir {
                path: path,
                subdirs: Vec::new()
            };

            func(&mut dir);
            self.subdirs.push(dir);
            self
        }

        pub fn path(&self) -> &Path {
            &self.path
        }

        pub fn path_str(&self) -> &str {
            self.path.as_path().to_str().unwrap()
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    pub struct TestFile {
        path: PathBuf
    }

    impl TestFile {
        pub fn new(contents: &str) -> TestFile {
            let (_, pb) = make_until_valid_path(&env::temp_dir(), |pb| {
                fs::write(pb, contents)
            });

            TestFile {
                path: pb
            }
        }

        pub fn path_str(&self) -> String {
            pathbuf_to_str(&self.path)
        }
    }
}
