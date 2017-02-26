use std::iter::{Iterator, Zip, Skip};
use std::ops::{Range};

pub struct RangeIteratorAdapter<I: Iterator>{
    iter: Zip<I, Skip<I>>
}

impl <I> RangeIteratorAdapter<I> where I: Iterator + Clone {
    pub fn new(it: I) -> Self {
        Self {
            iter: it.clone().zip(it.skip(1))
        }
    }
}

// impl <'i, I> Iterator for RangeIteratorAdapter<I> where I: Iterator<Item=&'i usize> + Clone {
//     type Item=Range<usize>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next().map(|(start, end)| *start..*end)
//     }
// }

impl <'i, I,T> Iterator for RangeIteratorAdapter<I> where I: Iterator<Item=&'i T> + Clone, T: 'i + Copy {
    type Item=Range<T>;

    fn next(&mut self) -> Option<Range<T>> {
        self.iter.next().map(|(start, end)| *start..*end)
    }
}
