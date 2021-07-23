pub struct Concat<T, A: Iterator<Item=T>, B: IntoIterator<Item=T, IntoIter=A>, C: Iterator<Item=B>> {
    current: Option<A>,
    iter: C,
}

impl<T, A: Iterator<Item=T>, B: IntoIterator<Item=T, IntoIter=A>, C: Iterator<Item=B>, > Concat<T, A, B, C> {
    pub fn new<D: IntoIterator<Item=B, IntoIter=C>>(iter: D) -> Self {
        Concat {
            current: None,
            iter: iter.into_iter(),
        }
    }

    fn next_from_current(&mut self) -> Option<T> {
        match &mut self.current {
            None => None,
            Some(s) => s.next()
        }
    }

    fn replace_current(&mut self) {
        self.current = self.iter.next().map(|s| s.into_iter());
    }
}

impl<T, A: Iterator<Item=T>, B: IntoIterator<Item=T, IntoIter=A>, C: Iterator<Item=B>> Iterator for Concat<T, A, B, C> {
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
