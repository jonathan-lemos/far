pub struct Concat<T, A: Iterator<Item=T>, B: IntoIterator<Item=T, IntoIter=A>, C: Iterator<Item=B>, D: IntoIterator<Item=B, IntoIter=C>> {
    current: Option<A>,
    iter: C,
    iter_orig: D
}

impl<T, A: Iterator<Item=T>, B: IntoIterator<Item=T, IntoIter=A>, C: Iterator<Item=B>, D: IntoIterator<Item=B, IntoIter=C>> Concat<T, A, B, C, D> {
    pub fn new(iter: D) -> Self {
        Concat {
            current: None,
            iter: iter.into_iter(),
            iter_orig: iter
        }
    }

    fn next_from_current(&mut self) -> Option<T> {
        self.current.and_then(|c| c.next())
    }

    fn replace_current(&mut self) {
        self.current = self.iter.next().map(|s| s.into_iter());
    }
}

impl<T, A: Iterator<Item=T>, B: IntoIterator<Item=T, IntoIter=A>, C: Iterator<Item=B>, D: IntoIterator<Item=B, IntoIter=C>> Iterator for Concat<T, A, B, C, D> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_from_current() {
                Some(s) => return Some(s),
                None => {
                    self.replace_current();
                    if self.current.is_none() {
                        return None
                    }
                }
            }
        }
    }
}
