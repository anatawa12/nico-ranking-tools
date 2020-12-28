pub trait MyIterUtil: Iterator {
    fn with_has_next(self) -> WithHashNextIter<Self>
        where Self: Sized, {
        WithHashNextIter {
            iter: Box::new(self),
            prev: None,
        }
    }
}

pub struct WithHashNextIter<I> where I: Iterator {
    iter: Box<I>,
    prev: Option<I::Item>,
}

impl <I> Iterator for WithHashNextIter<I> where I: Iterator, {
    type Item = (I::Item, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.prev {
            let prev = self.iter.next();
            self.prev = prev;
        }
        return match std::mem::replace(&mut self.prev, None) {
            None => {
                None
            }
            Some(cur) => {
                let next = self.iter.next();
                let is_some = next.is_some();
                self.prev = next;
                Some((cur, is_some))
            }
        }
    }
}

impl<T: ?Sized> MyIterUtil for T where T: Iterator { }

