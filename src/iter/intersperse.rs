pub struct Intersperse<T, I, F>
    where F: FnMut() -> T, I: Iterator<Item=T> {
    buffer: Option<T>,
    join_with: F,
    iter: I,
    do_join: bool,
}

impl<T, I, F> Intersperse<T, I, F>
    where F: FnMut() -> T, I: Iterator<Item=T> {
    pub fn new<V: IntoIterator<Item=T, IntoIter=I>>(iter: V, sep_factory: F) -> Intersperse<T, I, F> {
        Intersperse {
            buffer: None,
            join_with: sep_factory,
            iter: iter.into_iter(),
            do_join: false,
        }
    }

    fn load_buffer(&mut self) -> Option<T> {
        match self.iter.next() {
            Some(s) => {
                self.buffer.replace(s)
            }
            None => {
                self.buffer.take()
            }
        }
    }

    fn next_iter_elem(&mut self) -> Option<T> {
        let prev = self.load_buffer();
        let latest = &self.buffer;
        match (prev, latest) {
            (None, Some(_)) => self.next(),
            (Some(s), Some(_)) => {
                self.do_join = true;
                Some(s)
            }
            (Some(s), None) => Some(s),
            (None, None) => None
        }
    }

    fn next_join_elem(&mut self) -> Option<T> {
        self.do_join = false;
        Some((self.join_with)())
    }
}

impl<T, I, F> Iterator for Intersperse<T, I, F>
    where F: FnMut() -> T, I: Iterator<Item=T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.do_join {
            self.next_join_elem()
        } else {
            self.next_iter_elem()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_join(strings: Vec<&str>, sep: &str, expected: Vec<&str>) {
        let actual: Vec<&str> = Intersperse::new(strings, || sep).collect();

        debug_assert_eq!(actual.len(), expected.len());

        for (a, b) in actual.iter().zip(expected.iter()) {
            debug_assert_eq!(a, b);
        }
    }

    #[test]
    fn test_join_basic() {
        let strings = vec!["abc", "def", "ghi"];
        let expected = vec!["abc", "\n", "def", "\n", "ghi"];

        test_join(strings, "\n", expected);
    }

    #[test]
    fn test_join_one_string() {
        let strings = vec!["abc"];
        let expected = strings.clone();

        test_join(strings, "\n", expected);
    }

    #[test]
    fn test_join_empty() {
        test_join(vec![], "\n", vec![]);
    }
}